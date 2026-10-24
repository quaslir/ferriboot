use std::fs::{self, create_dir_all};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
mod bootentry;
mod cmdline;
mod esp;
mod install;
mod kernels;
mod mkconfig;
fn die(msg: &str) -> ! {
    eprintln!("ferriboot: {}", msg);
    std::process::exit(1);
}

fn cmd_mkconfig(dry_run: bool) {
    let Some(esp) = esp::find_esp() else {
        die("ESP not found (checked /boot, /efi, /boot/efi)");
    };

    let find_kernels = kernels::find_all_kernels(&esp);

    let kernels = match find_kernels {
        Ok(k) => k,
        Err(e) => die(&e),
    };

    if kernels.is_empty() {
        die("no kernels (vmlinuz-*) found in the ESP or /boot");
    }
    let mut planned = Vec::new();
    let mut copies = Vec::new();

    for kernel in &kernels {
        let (k, c) = install::plan_kernel(&esp, kernel);
        planned.push(k);
        copies.extend(c);
    }
    let stale = install::plan_cleanup(&esp, &kernels);
    let get_cmdline = cmdline::get_cmdline();
    let cmdline = match get_cmdline {
        Ok(cmd) => cmd,
        Err(e) => die(e.as_str()),
    };

    let config = mkconfig::build_config(&planned, &cmdline, &esp);
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
            if let Some(dir) = to.parent() {
                if let Err(e) = create_dir_all(dir) {
                    die(&format!("cannot create {}: {}", dir.display(), e));
                }
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
            die(format!(
                "cannot create {}: {}",
                parent.display(),
                e.kind().to_string()
            )
            .as_str());
        };
        if conf_path.exists() {
            let backup = conf_path.with_extension("conf.bak");
            if let Err(e) = std::fs::copy(&conf_path, &backup) {
                die(format!(
                    "cannot back up {}: {}",
                    conf_path.display(),
                    e.kind().to_string(),
                )
                .as_str());
            }
        }
        if let Err(e) = std::fs::write(&conf_path, &text) {
            die(format!(
                "cannot write {}: {}",
                conf_path.display(),
                e.kind().to_string()
            )
            .as_str());
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
        let Ok(meta) = fs::metadata("/proc/self") else {
            die("cannot check user id (/proc/self)");
        };

        let uid = meta.uid();
        if uid != 0 {
            die("install must be run as root (try: sudo ferriboot install)")
        }
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

    cmd_mkconfig(dry_run);
    let find_entry_result = bootentry::find_entry("\\EFI\\ferriboot\\ferriboot.efi");

    let label = match find_entry_result {
        Ok(l) => l,
        Err(e) => die(&e),
    };

    if let Some(entry) = label {
        eprintln!("boot entry {} already exists", entry);
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
            if let Err(e) = bootentry::create_entry(
                &partition.0,
                partition.1,
                "\\EFI\\ferriboot\\ferriboot.efi",
                "ferriboot",
            ) {
                die(&e);
            };
        }
    }

    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => die(format!("cannot find own binary {}", e).as_str()),
    };
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
}

fn cmd_help() {
    eprintln!(
        "usage: ferriboot <command> [options]

    commands:
      install     install the bootloader to the ESP, register it in the
                  firmware, generate the config and copy ferriboot to
                  /usr/local/bin
      mkconfig    find kernels, copy them to the ESP if needed and write
                  EFI/ferriboot/ferriboot.conf
      help        show this help

    options:
      --dry-run   show what would be done without changing anything
      --efi PATH  (install) bootloader image to install
                  [default: /usr/lib/ferriboot/ferriboot.efi]

    most commands need root: sudo ferriboot <command>"
    );
}

fn main() {
    let it = std::env::args();
    let args = it.collect::<Vec<String>>();

    let first = args.get(1).map(|s| s.as_str());
    let dry_run = args.iter().any(|e| e == "--dry-run");
    match first {
        Some("mkconfig") => {
            cmd_mkconfig(dry_run);
        }
        Some("install") => {
            cmd_install(&args, dry_run);
        }
        Some("help") => {
            cmd_help();
        }
        _ => {
            eprintln!("usage: ferriboot <mkconfig|install>");
            std::process::exit(1);
        }
    };
}
