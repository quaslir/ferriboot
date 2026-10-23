use std::fs::{self, create_dir_all};

mod cmdline;
mod esp;
mod install;
mod kernels;
mod mkconfig;
fn die(msg: &str) -> ! {
    eprintln!("ferriboot: {}", msg);
    std::process::exit(1);
}

fn main() {
    let it = std::env::args();
    let args = it.collect::<Vec<String>>();

    let first = args.get(1).map(|s| s.as_str());
    match first {
        Some("mkconfig") => {}
        _ => {
            eprintln!("usage: ferriboot mkconfig [--dry-run]");
            std::process::exit(1);
        }
    };

    let dry_run = args.iter().any(|e| e == "--dry-run");
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
