use std::fs::{self, create_dir_all};
use {
    crate::cmdline, crate::die, crate::esp, crate::install, crate::kernels, crate::mkconfig,
    crate::uki,
};
pub fn cmd_mkconfig(dry_run: bool, if_installed: bool) {
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
