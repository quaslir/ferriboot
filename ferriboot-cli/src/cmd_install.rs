use std::fs::{self, create_dir_all};
use std::path::{Path, PathBuf};
use {
    crate::LOADER_PATH, crate::bootentry, crate::cmd_mkconfig, crate::die, crate::hooks,
    crate::installed_by_package, crate::require_root,
};

pub fn cmd_install(path: &Path, dry_run: bool) {
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
        if let Err(e) = fs::copy(path, &dest) {
            die(&format!(
                "cannot copy {} -> {}: {}",
                path.display(),
                dest.display(),
                e
            ));
        }
        eprintln!("ferriboot: installed {}", dest.display());
    }

    cmd_mkconfig::cmd_mkconfig(dry_run, false);
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
