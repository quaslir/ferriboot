use std::fs::{self, create_dir_all};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::bootentry::LOADER_PATH;
use crate::esp::find_esp;
mod bootentry;
mod cmdline;
mod esp;
mod hooks;
mod install;
mod kernels;
mod mkconfig;
mod uki;
const USAGE: &str = "usage: ferriboot <command> [options]

commands:
  install     install the bootloader to the ESP, generate the config,
              register a firmware boot entry, copy ferriboot to
              /usr/local/bin and install the kernel update hook
  mkconfig    find kernels, copy them to the ESP if needed and write
              EFI/ferriboot/ferriboot.conf
  uninstall   remove the boot entry, EFI/ferriboot on the ESP, the
              kernel update hook and /usr/local/bin/ferriboot
  help        show this help

options:
  --dry-run   show what would be done without changing anything
  --efi PATH  (install) bootloader image to install
              [default: /usr/lib/ferriboot/ferriboot.efi]
  --if-installed  (mkconfig) do nothing unless ferriboot is installed on the ESP

install, mkconfig and uninstall need root: sudo ferriboot <command>
";

fn installed_by_package(exe: &Path) -> bool {
    let Ok(res) = fs::canonicalize(exe) else {
        return false;
    };

    res.starts_with("/usr/bin")
}

fn die(msg: &str) -> ! {
    eprintln!("ferriboot: {}", msg);
    std::process::exit(1);
}
fn require_root() {
    let Ok(meta) = fs::metadata("/proc/self") else {
        die("cannot check user id (/proc/self)")
    };

    let uid = meta.uid();
    if uid != 0 {
        die("must be run as root (try: sudo ferriboot <command>)")
    }
}
fn cmd_mkconfig(dry_run: bool, if_installed: bool) {
    let Some(esp) = esp::find_esp() else {
        if if_installed {
            return;
        }
        die("ESP not found (checked /boot, /efi, /boot/efi)");
    };
    if if_installed && !esp.join("EFI/ferriboot/ferriboot.efi").exists() {
        return;
    }
    let find_kernels = kernels::find_all_kernels(&esp);

    let mut kernels = match find_kernels {
        Ok(k) => k,
        Err(e) => die(&e),
    };

    kernels.retain(|k| {
        if k.initrd.is_none() {
            eprintln!(
                "ferriboot: warning: skipping {}: no initramfs found",
                k.image.display()
            );
            return false;
        }

        true
    });
    let ukis = uki::find_ukis(&esp);
    let windows = uki::find_windows(&esp);
    if kernels.is_empty() && ukis.is_empty() && windows.is_none() {
        die("no kernels or UKIs found in the ESP or /boot");
    }
    let mut planned = Vec::new();
    let mut copies = Vec::new();

    for kernel in &kernels {
        let (k, c) = install::plan_kernel(&esp, kernel);
        planned.push(k);
        copies.extend(c);
    }
    copies.retain(|(from, to)| !install::is_up_to_date(from, to));
    let stale = install::plan_cleanup(&esp, &kernels);
    let get_cmdline = cmdline::get_cmdline();
    let cmdline = match get_cmdline {
        Ok(cmd) => cmd,
        Err(e) => die(e.as_str()),
    };

    let config = mkconfig::build_config(&planned, &ukis, windows, &cmdline, &esp);
    let text = config.to_string();
    let parse_result = config::parse(&text);
    if let Err(e) = parse_result {
        die(format!(
            "generated config is invalid (line {}: {})",
            e.line,
            e.kind.message()
        )
        .as_str());
    }

    if dry_run {
        for (from, to) in &copies {
            eprintln!(
                "ferriboot: would copy {} -> {}",
                from.display(),
                to.display()
            );
        }

        for path in &stale {
            eprintln!("ferriboot: would remove {}", path.display());
        }
        print!("{}", text);
    } else {
        for (from, to) in &copies {
            if let Some(dir) = to.parent()
                && let Err(e) = create_dir_all(dir)
            {
                die(&format!("cannot create {}: {}", dir.display(), e));
            }
            if let Err(e) = fs::copy(from, to) {
                die(&format!(
                    "cannot copy {} -> {}: {}",
                    from.display(),
                    to.display(),
                    e
                ));
            };

            eprintln!("ferriboot: copied {} -> {}", from.display(), to.display());
        }
        let conf_path = esp.join("EFI/ferriboot/ferriboot.conf");
        let parent = conf_path.parent().unwrap();
        if let Err(e) = std::fs::create_dir_all(parent) {
            die(format!("cannot create {}: {}", parent.display(), e.kind()).as_str());
        };
        if conf_path.exists() {
            let backup = conf_path.with_extension("conf.bak");
            if let Err(e) = std::fs::copy(&conf_path, &backup) {
                die(format!("cannot back up {}: {}", conf_path.display(), e.kind(),).as_str());
            }
        }
        if let Err(e) = std::fs::write(&conf_path, &text) {
            die(format!("cannot write {}: {}", conf_path.display(), e.kind()).as_str());
        };

        eprintln!(
            "ferriboot: wrote {} ({} entries)",
            conf_path.display(),
            config.entries.len()
        );
        for path in &stale {
            if let Err(e) = fs::remove_dir_all(path) {
                eprintln!(
                    "ferriboot: warning: cannot remove {}: {}",
                    path.display(),
                    e
                );
                continue;
            };
            eprintln!("ferriboot: removed {}", path.display());
        }
    }
}

fn cmd_install(args: &[String], dry_run: bool) {
    if !dry_run {
        require_root();
    }

    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => die(&format!("cannot find own binary: {}", e)),
    };

    let packaged = installed_by_package(&exe);

    if packaged {
        eprintln!(
            "ferriboot: installed by a package, binary and kernel hook are managed by pacman"
        );
    }

    let efi = args.iter().position(|s| s == "--efi");

    let path_str = match efi {
        Some(i) => {
            let Some(p) = args.get(i + 1) else {
                die("--efi needs a path")
            };
            p.as_str()
        }
        None => "/usr/lib/ferriboot/ferriboot.efi",
    };
    let path = PathBuf::from(path_str);
    if !path.is_file() {
        die(format!("{} not found", path.display()).as_str());
    }

    let Some(esp) = esp::find_esp() else {
        die("ESP not found (checked /boot, /efi, /boot/efi)");
    };
    let dest = esp.join("EFI/ferriboot/ferriboot.efi");

    if dry_run {
        eprintln!(
            "ferriboot: would copy {} -> {}",
            path.display(),
            dest.display()
        );
    } else {
        let Some(dir) = dest.parent() else {
            die("cannot determine directory of destination path")
        };
        if let Err(e) = create_dir_all(dir) {
            die(&format!("cannot create {}: {}", dir.display(), e));
        };
        if let Err(e) = fs::copy(&path, &dest) {
            die(&format!(
                "cannot copy {} -> {}: {}",
                path.display(),
                dest.display(),
                e
            ));
        }
        eprintln!("ferriboot: installed {}", dest.display());
    }

    cmd_mkconfig(dry_run, false);
    let find_entry_result = bootentry::find_entry(LOADER_PATH);

    let label = match find_entry_result {
        Ok(l) => l,
        Err(e) => die(&e),
    };

    if let Some(entry) = label {
        eprintln!("ferriboot: boot entry Boot{} already exists", entry);
    } else {
        let esp_partition_result = bootentry::esp_partition(&esp);

        let partition = match esp_partition_result {
            Ok(p) => p,
            Err(e) => die(&e),
        };

        if dry_run {
            eprintln!(
                "ferriboot: would create boot entry ferriboot ({}, partition {})",
                partition.0, partition.1
            );
        } else {
            if let Err(e) =
                bootentry::create_entry(&partition.0, partition.1, LOADER_PATH, "ferriboot")
            {
                die(&e);
            };
            eprintln!("ferriboot: created boot entry ferriboot");
        }
    }
    if !packaged {
        let target = PathBuf::from("/usr/local/bin/ferriboot");
        let same = target.exists() && fs::canonicalize(&exe).ok() == fs::canonicalize(&target).ok();
        if same {
            eprintln!("ferriboot: /usr/local/bin/ferriboot is already the running binary");
        } else if dry_run {
            eprintln!(
                "ferriboot: would install {} -> {}",
                exe.display(),
                target.display()
            );
        } else {
            if let Err(e) = fs::copy(&exe, &target) {
                die(format!(
                    "cannot copy {} -> {}: {}",
                    exe.display(),
                    target.display(),
                    e
                )
                .as_str());
            };

            eprintln!("ferriboot: installed {}", target.display());
        }

        if let Err(e) = hooks::install_hooks(dry_run) {
            die(&e)
        }
    }
}

fn cmd_help() {
    print!("{}", USAGE);
}

fn cmd_uninstall(dry_run: bool) {
    if !dry_run {
        require_root();
    }
    let find_entry = bootentry::find_entry(LOADER_PATH);
    let entry = match find_entry {
        Ok(entr) => entr,
        Err(e) => die(&e),
    };

    if let Some(entr) = entry {
        if dry_run {
            eprintln!("ferriboot: would remove boot entry Boot{}", entr);
        } else {
            if let Err(e) = bootentry::delete_entry(&entr) {
                die(&e)
            };

            eprintln!("ferriboot: removed boot entry Boot{}", entr);
        }
    } else {
        eprintln!("ferriboot: no boot entry found");
    }

    let Some(esp) = find_esp() else {
        die("ESP not found (checked /boot, /efi, /boot/efi)");
    };

    let dir = esp.join("EFI/ferriboot");

    if dir.exists() {
        if dry_run {
            eprintln!("ferriboot: would remove {}", dir.display());
        } else {
            if let Err(e) = fs::remove_dir_all(&dir) {
                die(&format!("cannot remove {}: {}", dir.display(), e));
            };

            eprintln!("ferriboot: removed {}", dir.display());
        }
    }

    if let Err(e) = hooks::remove_hooks(dry_run) {
        die(&e)
    };

    let bin = Path::new("/usr/local/bin/ferriboot");
    if bin.exists() {
        if dry_run {
            eprintln!("ferriboot: would remove {}", bin.display());
        } else {
            if let Err(e) = fs::remove_file(bin) {
                die(&format!("cannot remove {}: {}", bin.display(), e));
            };

            eprintln!("ferriboot: removed {}", bin.display());
        }
    }

    eprintln!(
        "ferriboot: make sure another bootloader is installed and registered in the firmware, otherwise the system will not boot"
    );
}

fn main() {
    let it = std::env::args();
    let args = it.collect::<Vec<String>>();

    let first = args.get(1).map(|s| s.as_str());
    let dry_run = args.iter().any(|e| e == "--dry-run");
    let if_installed = args.iter().any(|s| s == "--if-installed");
    match first {
        Some("mkconfig") => {
            cmd_mkconfig(dry_run, if_installed);
        }
        Some("install") => {
            cmd_install(&args, dry_run);
        }
        Some("uninstall") => {
            cmd_uninstall(dry_run);
        }
        Some("help") | Some("--help") | Some("-h") => {
            cmd_help();
        }
        _ => {
            if let Some(cmd) = first {
                eprintln!("ferriboot: unknown command '{}'", cmd);
            }
            eprint!("{}", USAGE);
            std::process::exit(1);
        }
    };
}
