use crate::cmd_update;
use crate::efivars;
use std::path::Path;
const FILE_PATH: &str = "/usr/lib/ferriboot/ferriboot.efi";
pub fn cmd_status() {
    let Some(esp) = crate::esp::find_esp() else {
        println!("ESP:          not found (checked /boot, /efi, /boot/efi)");
        return;
    };

    println!("ESP:          {}", esp.display());
    let file = esp.join("EFI/ferriboot/ferriboot.efi");
    if file.exists() {
        let copy = Path::new(FILE_PATH);
        if !copy.exists() {
            println!("Bootloader:   installed (EFI/ferriboot/ferriboot.efi)");
        } else {
            match cmd_update::is_same_file(&file, copy) {
                Ok(true) => println!("Bootloader: installed, up to date"),
                Ok(false) => {
                    println!("Bootloader:   installed, outdated (run: sudo ferriboot update)")
                }
                Err(e) => println!("Bootloader:   installed (cannot compare: {e})"),
            };
        }
    } else {
        println!("Bootloader:   not installed (run: sudo ferriboot install)");
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

    let loader = efivars::read_string("LoaderInfo", efivars::BLI_GUID);
    match loader {
        None => println!("Last boot:    unknown (booted without Boot Loader Interface)"),
        Some(info) => {
            let entry = efivars::read_string("LoaderEntrySelected", efivars::BLI_GUID)
                .unwrap_or_else(|| "?".to_string());
            let init = efivars::read_string("LoaderTimeInitUSec", efivars::BLI_GUID)
                .and_then(|t| t.parse::<u64>().ok());
            let exec = efivars::read_string("LoaderTimeExecUSec", efivars::BLI_GUID)
                .and_then(|t| t.parse::<u64>().ok());
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
