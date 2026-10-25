use std::os::unix::fs::PermissionsExt;
use std::{
    fs::{self, Permissions},
    path::Path,
};
const PACMAN_HOOK: &str = "[Trigger]
Type = Path
Operation = Install
Operation = Upgrade
Operation = Remove
Target = usr/lib/modules/*/vmlinuz

[Action]
Description = Updating ferriboot configuration...
When = PostTransaction
Exec = /usr/local/bin/ferriboot mkconfig\n";

const PACMAN_HOOK_PATH: &str = "/etc/pacman.d/hooks/95-ferriboot.hook";

const KERNEL_INSTALL_HOOK: &str = "#!/bin/sh
case \"$1\" in
    add|remove) /usr/local/bin/ferriboot mkconfig ;;
esac
exit 0\n";

const KERNEL_INSTALL_HOOK_PATH: &str = "/etc/kernel/install.d/95-ferriboot.install";

const DEBIAN_HOOK: &str = "#!/bin/sh
/usr/local/bin/ferriboot mkconfig
exit 0\n";

const DEBIAN_POSTINST_PATH: &str = "/etc/kernel/postinst.d/zz-ferriboot";
const DEBIAN_POSTRM_PATH: &str = "/etc/kernel/postrm.d/zz-ferriboot";

pub fn install_hooks(dry_run: bool) -> Result<(), String> {
    if Path::new("/etc/pacman.d").exists() {
        write_file(PACMAN_HOOK_PATH, PACMAN_HOOK, 0o644, dry_run)?;
        return Ok(());
    }

    if Path::new("/etc/debian_version").exists() {
        write_file(DEBIAN_POSTINST_PATH, DEBIAN_HOOK, 0o755, dry_run)?;
        write_file(DEBIAN_POSTRM_PATH, DEBIAN_HOOK, 0o755, dry_run)?;
        return Ok(());
    }

    if Path::new("/usr/lib/kernel/install.d").exists() {
        write_file(
            KERNEL_INSTALL_HOOK_PATH,
            KERNEL_INSTALL_HOOK,
            0o755,
            dry_run,
        )?;
        return Ok(());
    }

    eprintln!(
        "ferriboot: warning: no kernel hook for this distribution, run ferriboot mkconfig after kernel updates"
    );
    Ok(())
}

fn write_file(path: &str, content: &str, mode: u32, dry_run: bool) -> Result<(), String> {
    if dry_run {
        eprintln!("ferriboot: would write {}", path);
        return Ok(());
    }
    let path = Path::new(path);
    let Some(parent) = path.parent() else {
        return Err("cannot determine directory of hook path".to_string());
    };

    fs::create_dir_all(parent)
        .map_err(|e| format!("cannot create {}: {}", parent.display(), e.to_string()))?;
    fs::write(path, content)
        .map_err(|e| format!("cannot write {} : {}", path.display(), e.to_string()))?;

    fs::set_permissions(path, Permissions::from_mode(mode)).map_err(|e| {
        format!(
            "cannot set permissions on {}: {}",
            path.display(),
            e.to_string()
        )
    })?;
    eprintln!("ferriboot: installed {}", path.display());
    Ok(())
}
