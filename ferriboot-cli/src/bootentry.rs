use std::{
    fs,
    path::{Path, PathBuf},
};
pub const LOADER_PATH: &str = "\\EFI\\ferriboot\\ferriboot.efi";
pub fn run(program: &str, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new(program).args(args).output();

    let output = match output {
        Ok(o) => o,
        Err(e) => return Err(format!("cannot run {}: {}", program, e.kind())),
    };

    match output.status.success() {
        true => Ok(String::from_utf8_lossy(&output.stdout).trim().to_string()),
        false => Err(String::from_utf8_lossy(&output.stderr).trim().to_string()),
    }
}

pub fn esp_partition(esp: &Path) -> Result<(String, u32), String> {
    let Some(esp_str) = esp.to_str() else {
        return Err("ESP path is not valid UTF-8".to_string());
    };
    let findmnt = run("findmnt", &["-no", "SOURCE", esp_str])?;
    let lsblk = run("lsblk", &["-no", "PKNAME", findmnt.as_str()])?;

    let disk = format!("/dev/{}", lsblk);
    let Some(file_name) = Path::new(&findmnt).file_name() else {
        return Err("cannot get partition name".to_string());
    };
    let Some(file_name_str) = file_name.to_str() else {
        return Err("partition name is not valid UTF-8".to_string());
    };
    let part = fs::read_to_string(format!("/sys/class/block/{}/partition", file_name_str))
        .map_err(|e| e.to_string())?
        .trim()
        .parse::<u32>()
        .map_err(|e| e.to_string())?;

    Ok((disk, part))
}

pub fn find_entry(label: &str) -> Result<Option<String>, String> {
    let dir = PathBuf::from("/sys/firmware/efi");
    if !dir.exists() {
        return Err("not booted in UEFI mode (/sys/firmware/efi missing)".to_string());
    }

    let efibootmgr = run("efibootmgr", &["-v"])?;
    for line in efibootmgr.lines() {
        let Some(entry) = line.strip_prefix("Boot") else {
            continue;
        };
        let mut found = true;
        let Some(number) = entry.get(..4) else {
            continue;
        };
        let Some(rest) = entry.get(4..) else {
            continue;
        };
        for c in number.chars() {
            if !c.is_ascii_hexdigit() {
                found = false;
                break;
            }
        }

        if found {
            let normalized = rest.trim_start_matches('*').trim_start();
            if normalized
                .to_ascii_lowercase()
                .contains(label.to_ascii_lowercase().as_str())
            {
                return Ok(Some(number.to_string()));
            }
        }
    }

    Ok(None)
}

pub fn create_entry(disk: &str, part: u32, loader: &str, label: &str) -> Result<(), String> {
    let part_str = part.to_string();

    run(
        "efibootmgr",
        &[
            "--create",
            "--disk",
            disk,
            "--part",
            part_str.as_str(),
            "--loader",
            loader,
            "--label",
            label,
        ],
    )?;
    Ok(())
}

pub fn delete_entry(num: &str) -> Result<(), String> {
    run("efibootmgr", &["-b", num, "-B"])?;
    Ok(())
}

pub fn boot_order() -> Result<Vec<String>, String> {
    let result = run("efibootmgr", &[])?;

    for line in result.lines() {
        let Some(target) = line.strip_prefix("BootOrder:") else {
            continue;
        };

        let order = target.trim().split(',').map(|s| s.to_string());
        return Ok(order.collect());
    }

    Ok(Vec::new())
}
