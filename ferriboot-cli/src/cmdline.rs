use std::{path::PathBuf, process::Command};

fn find_mnt(field: &str) -> Result<String, String> {
    let output = Command::new("findmnt")
        .args(["-no", field, "/"])
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from("findmnt failed"));
    }

    let value = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    Ok(value.trim().to_string())
}

pub fn get_cmdline() -> Result<String, String> {
    let path = PathBuf::from("/etc/kernel/cmdline");

    if path.exists() {
        return Ok(std::fs::read_to_string(path)
            .map_err(|e| e.to_string())?
            .trim()
            .to_string());
    }

    let uuid = find_mnt("UUID")?;
    let mut cmdline = format!("root=UUID={} rw", uuid);
    let fstype = find_mnt("FSTYPE")?;

    if fstype == "btrfs" {
        let fsroot = find_mnt("FSROOT")?;

        if fsroot != "/" {
            cmdline
                .push_str(format!(" rootflags=subvol={}", fsroot.trim_start_matches('/')).as_str());
        }
    }

    Ok(cmdline)
}
