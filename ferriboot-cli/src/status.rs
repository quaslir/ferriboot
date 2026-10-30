use std::path::PathBuf;

pub fn cmd_status() {
    let Some(esp) = crate::esp::find_esp() else {
        eprintln!("ESP:          not found (checked /boot, /efi, /boot/efi)");
        return;
    };

    println!("ESP:          {}", esp.display());
    let file = esp.join("EFI/ferriboot/ferriboot.efi");
    if file.exists() {
        println!("Bootloader:   installed (EFI/ferriboot/ferriboot.efi)");
    } else {
        println!("Bootloader:   not installed (run: sudo ferriboot install");
        return;
    }

    let entry = crate::bootentry::find_entry("ferriboot");

    match entry {
        Ok(Some(num)) => {
            let boot_order = crate::bootentry::boot_order();
            match boot_order {
                Ok(order) => {
                    let position = order.iter().position(|n| *n == num);
                    match position {
                        Some(0) => println!("Boot entry:   Boot{num}, first in BootOrder"),
                        Some(i) => {
                            println!("Boot entry:   Boot{num}, position {} in BootOrder", i + 1)
                        }
                        None => println!("Boot entry:   Boot{num}, not in BootOrder"),
                    }
                }
                Err(_) => println!("Boot entry:   Boot{num}"),
            }
        }
        Ok(None) => {
            println!("Boot entry: not found (run: sudo ferriboot install)");
        }
        Err(e) => println!("Boot entry:   cannot check ({e})"),
    };

    let internals = std::fs::read_to_string(esp.join("EFI/ferriboot/ferriboot.conf"));
    match internals {
        Ok(text) => {
            let config = config::parse(&text);
            match config {
                Err(e) => println!(
                    "Config:       invalid, line {}: {}",
                    e.line,
                    e.kind.message()
                ),
                Ok(conf) => {
                    let name = match conf.entries.get(conf.default) {
                        Some(e) => e.name.clone(),
                        None => "none".to_string(),
                    };
                    println!(
                        "Config:       {} entries, default \"{}\", timeout {}s",
                        conf.entries.len(),
                        name,
                        conf.timeout
                    );
                }
            }
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                println!("Config:       not found (run: sudo ferriboot mkconfig)")
            }
            _ => println!("Config:       cannot read ({e})"),
        },
    }

    let loader = read_efivar("LoaderInfo");
    match loader {
        None => println!("Last boot:    unknown (booted without Boot Loader Interface)"),
        Some(info) => {
            let entry = read_efivar("LoaderEntrySelected").unwrap_or_else(|| "?".to_string());
            let init = read_efivar("LoaderTimeInitUSec").and_then(|t| t.parse::<u64>().ok());
            let exec = read_efivar("LoaderTimeExecUSec").and_then(|t| t.parse::<u64>().ok());
            if let Some(init_time) = init
                && let Some(exec_time) = exec
                && let Some(diff) = exec_time.checked_sub(init_time)
            {
                println!(
                    "Last boot:    {info}, \"{entry}\", loader {} ms",
                    diff / 1000
                );
            } else {
                println!("Last boot:    {info}, \"{entry}\"");
            }
        }
    }
}

const GUID: &str = "4a67b082-0a4c-41cf-b6c7-440b29bb8c4f";
fn read_efivar(name: &str) -> Option<String> {
    let path = PathBuf::from(format!("/sys/firmware/efi/efivars/{}-{}", name, GUID));
    let bytes = std::fs::read(&path).ok()?;
    let result = bytes.get(4..)?;
    let units = result
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect::<Vec<u16>>();
    let text = String::from_utf16(&units).ok()?;
    Some(text.trim_end_matches('\0').to_string())
}
