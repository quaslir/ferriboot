use std::{fs, path::Path};

use crate::{die, esp::find_esp, require_root};

pub fn cmd_update(efi: &Path, if_installed: bool, dry_run: bool) {
    if !dry_run {
        require_root();
    }

    if !efi.exists() {
        die(&format!("{} not found", efi.display()));
    }

    let Some(esp) = find_esp() else {
        if if_installed {
            return;
        }

        die("ESP not found (checked /boot, /efi, /boot/efi)");
    };

    let dest = esp.join("EFI/ferriboot/ferriboot.efi");

    if !dest.exists() {
        if if_installed {
            return;
        }

        die("ferriboot is not installed (run: sudo ferriboot install)");
    }

    match is_same_file(efi, &dest) {
        Ok(true) => {
            eprintln!("ferriboot:   is up to date");
            return;
        }
        Ok(false) => {}
        Err(e) => die(&format!(
            "cannot compare {} with {}: {e}",
            efi.display(),
            dest.display(),
        )),
    };

    if dry_run {
        eprintln!("ferriboot: would update {}", dest.display());
        return;
    }

    let backup = dest.with_extension("efi.old");
    if let Err(e) = fs::copy(&dest, &backup) {
        die(&format!("cannot back up {}: {}", dest.display(), e))
    }

    let tmp = dest.with_extension("efi.new");

    if let Err(e) = fs::copy(efi, &tmp) {
        die(&format!("cannot write {}: {}", tmp.display(), e));
    }

    if let Err(e) = fs::rename(&tmp, &dest) {
        die(&format!("cannot replace {}: {}", dest.display(), e));
    }
    eprintln!(
        "ferriboot: updated {} (previous version saved as ferriboot.efi.old)",
        dest.display()
    );
}

pub fn is_same_file(a: &Path, b: &Path) -> std::io::Result<bool> {
    let a_text = fs::read(a)?;
    let b_text = fs::read(b)?;
    Ok(a_text == b_text)
}
