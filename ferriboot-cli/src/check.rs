use std::path::{Path, PathBuf};

use crate::{efivars, esp};
pub fn cmd_check() {
    let Some(esp) = esp::find_esp() else {
        eprintln!("ferriboot: error: ESP not found (checked /boot, /efi, /boot/efi)");
        std::process::exit(1);
    };

    let conf_str_result = std::fs::read_to_string(esp.join("EFI/ferriboot/ferriboot.conf"));
    let conf_str = match conf_str_result {
        Ok(r) => r,
        Err(e) => {
            eprintln!("ferriboot: error: cannot read config ({e})");
            std::process::exit(1);
        }
    };

    let parsed_result = config::parse(&conf_str);

    let conf = match parsed_result {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "ferriboot: error: config is invalid, line {}: {}",
                e.line,
                e.kind.message()
            );
            std::process::exit(1);
        }
    };

    let efi_var_result = efivars::read_bytes("SecureBoot", efivars::GLOBAL_GUID);
    if let Some(bytes) = efi_var_result
        && bytes.first() == Some(&1)
    {
        eprintln!(
            "ferriboot: warning: Secure Boot is enabled, the firmware will refuse to start ferriboot (it is not signed)"
        );
    }

    let mut errors = 0;
    for entry in &conf.entries {
        for info in [&entry.kernel, &entry.initrd, &entry.path] {
            let Some(i) = info else {
                continue;
            };

            if !esp_path(&esp, i).is_file() {
                eprintln!(
                    "ferriboot: error: \"{}\": {} not found on the ESP",
                    entry.name, i
                );
                errors += 1;
            }
        }
    }
    if errors == 0 {
        println!("ok: {} entries, all files present", conf.entries.len());
        return;
    }
    eprintln!("ferriboot: {} problem(s) found", errors);
    std::process::exit(1);
}

fn esp_path(esp: &Path, uefi: &str) -> PathBuf {
    esp.join(uefi.trim_start_matches('\\').replace('\\', "/"))
}
