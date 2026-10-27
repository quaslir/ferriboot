use std::path::{Path, PathBuf};

use crate::kernels::Kernel;

fn to_uefi_path(esp: &Path, path: &Path) -> Option<String> {
    let uefi_path = path.strip_prefix(esp).ok()?.to_str()?.replace('/', "\\");
    Some(format!("\\{}", uefi_path))
}

fn distro_name() -> String {
    let path = PathBuf::from("/etc/os-release");
    if let Ok(text) = std::fs::read_to_string(&path) {
        for line in text.lines() {
            let value = line.strip_prefix("NAME=");
            if let Some(name) = value {
                return name.trim_matches('"').to_string();
            }
        }
    }
    "Linux".to_string()
}

pub fn build_config(kernels: &[Kernel], ukis: &[PathBuf], cmd: &str, esp: &Path) -> config::Config {
    let distro_name = distro_name();
    let mut entries = Vec::new();

    for uki in ukis {
        let Some(path) = to_uefi_path(esp, uki) else {
            eprintln!(
                "ferriboot: warning: skipping {}: not on the ESP",
                uki.display()
            );
            continue;
        };

        let stem = uki.file_stem().and_then(|s| s.to_str()).unwrap_or("UKI");

        entries.push(config::Entry {
            name: format!("{} (UKI: {})", distro_name, stem),
            protocol: config::Protocol::Efi,
            kernel: None,
            initrd: None,
            cmdline: None,
            path: Some(path),
        });
    }

    for kernel in kernels {
        let Some(kernel_path) = to_uefi_path(esp, &kernel.image) else {
            eprintln!(
                "ferriboot: skipping {}: not on the ESP",
                kernel.image.display()
            );
            continue;
        };
        let initrd = match &kernel.initrd {
            None => {
                continue;
            }
            Some(p) => {
                let Some(path) = to_uefi_path(esp, p) else {
                    eprintln!(
                        "ferriboot: warning: skipping {}: initramfs is not on the ESP",
                        p.display()
                    );
                    continue;
                };
                Some(path)
            }
        };
        let name = format!("{} ({})", distro_name, kernel.variant);
        let protocol = config::Protocol::Linux;
        let cmdline = Some(cmd.to_string());
        entries.push(config::Entry {
            name,
            protocol,
            kernel: Some(kernel_path.clone()),
            initrd,
            cmdline,
            path: None,
        });

        if let Some(fallback) = kernel.fallback.as_ref().and_then(|f| to_uefi_path(esp, f)) {
            entries.push(config::Entry {
                name: format!("{} ({}, fallback)", distro_name, kernel.variant),
                protocol,
                kernel: Some(kernel_path),
                initrd: Some(fallback),
                cmdline: Some(cmd.to_string()),
                path: None,
            });
        }
    }

    config::Config {
        timeout: 3,
        default: 0,
        entries,
    }
}
