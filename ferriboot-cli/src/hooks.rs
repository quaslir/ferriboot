use std::{fs, path::Path};
pub const PACMAN_HOOK: &str = "[Trigger]
Type = Path
Operation = Install
Operation = Upgrade
Operation = Remove
Target = usr/lib/modules/*/vmlinuz

[Action]
Description = Updating ferriboot configuration...
When = PostTransaction
Exec = /usr/local/bin/ferriboot mkconfig\n";

pub const PACMAN_HOOK_PATH: &str = "/etc/pacman.d/hooks/95-ferriboot.hook";

pub fn install_hooks(dry_run: bool) -> Result<(), String> {
    if Path::new("/etc/pacman.d").exists() {
        let path = Path::new(PACMAN_HOOK_PATH);
        if dry_run {
            eprintln!("ferriboot: would write {}", path.display());
        } else {
            let Some(parent) = path.parent() else {
                return Err("cannot determine directory of hook path".to_string());
            };

            fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create {}: {}", parent.display(), e.to_string()))?;
            fs::write(path, PACMAN_HOOK)
                .map_err(|e| format!("cannot write {} : {}", path.display(), e.to_string()))?;
            eprintln!("ferriboot: installed {}", path.display());
        }
        return Ok(());
    }
    eprintln!(
        "ferriboot: warning: no kernel hook for this distribution, run ferriboot mkconfig after kernel updates"
    );
    Ok(())
}
