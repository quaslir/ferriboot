use std::fs;

use crate::{SettingKey, cmd_mkconfig, die, esp, require_root, settings};

pub fn cmd_set(key: SettingKey, val: &str, dry_run: bool) {
    if !dry_run {
        require_root();
    }
    let mut settings = match settings::load() {
        Ok(s) => s,
        Err(e) => die(&e),
    };
    match key {
        SettingKey::Timeout => {
            settings.timeout = match val.parse::<u32>() {
                Ok(value) => Some(value),
                Err(_) => die("timeout must be a number"),
            }
        }

        SettingKey::Default => {
            require_entry_name(val);
            settings.default = Some(val.to_string());
        }
    }
    let text = settings::to_text(&settings);

    if dry_run {
        println!("ferriboot: would write {}:\n{}", settings::PATH, text);
    } else {
        if let Err(e) = fs::write(settings::PATH, text) {
            die(&format!("cannot write {}: {}", settings::PATH, e));
        }

        cmd_mkconfig::cmd_mkconfig(dry_run, false);

        let name = match key {
            SettingKey::Default => "default",
            SettingKey::Timeout => "timeout",
        };

        eprintln!("ferriboot: {} set to {}", name, val);
    }
}

fn require_entry_name(name: &str) {
    let Some(esp) = esp::find_esp() else {
        die("ESP not found (checked /boot, /efi, /boot/efi)");
    };

    let Ok(text) = fs::read_to_string(esp.join("EFI/ferriboot/ferriboot.conf")) else {
        die("cannot read the current menu, run: sudo ferriboot mkconfig");
    };
    let conf = match config::parse(&text) {
        Ok(config) => config,
        Err(_) => die("cannot read the current menu, run: sudo ferriboot mkconfig"),
    };

    if conf.entries.iter().any(|entry| entry.name == name) {
        return;
    }

    let list = conf
        .entries
        .iter()
        .map(|entry| entry.name.as_str())
        .collect::<Vec<_>>()
        .join("\n ");
    die(&format!(
        "no menu entry named '{}'. Available entries:\n {}",
        name, list
    ));
}
