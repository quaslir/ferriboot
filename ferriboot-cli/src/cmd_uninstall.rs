use std::fs::{self};
use std::path::Path;
use {
    crate::LOADER_PATH, crate::bootentry, crate::die, crate::find_esp, crate::hooks,
    crate::require_root,
};

pub fn cmd_uninstall(dry_run: bool) {
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
