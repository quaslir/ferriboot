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

    let first =
        fs::read(efi).unwrap_or_else(|f| die(&format!("cannot read {}: {}", efi.display(), f)));
    let second =
        fs::read(&dest).unwrap_or_else(|f| die(&format!("cannot read {}: {}", dest.display(), f)));

    if first == second {
        eprintln!("ferriboot: {} is up to date", dest.display());
        return;
    }

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
