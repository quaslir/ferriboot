use std::{fs, ops::RangeBounds, path::Path};

pub fn run(program: &str, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new(program).args(args).output();

    let output = match output {
        Ok(o) => o,
        Err(e) => return Err(format!("cannot run {}: {}", program, e.kind())),
    };

    match output.status.success() {
        true => return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string()),
        false => return Err(String::from_utf8_lossy(&output.stderr).trim().to_string()),
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
