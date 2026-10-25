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

pub fn build_config(kernels: &[Kernel], cmd: &str, esp: &Path) -> config::Config {
    let distro_name = distro_name();
    let mut entries = Vec::new();
    for kernel in kernels {
        let Some(kernel_path) = to_uefi_path(esp, &kernel.image) else {
            eprintln!(
                "ferriboot: skipping {}: not on the ESP",
                kernel.image.display()
            );
            continue;
        };
        let initrd = match &kernel.initrd {
            None => None,
            Some(p) => {
                let Some(path) = to_uefi_path(esp, p) else {
                    eprintln!();
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
            kernel: kernel_path.clone(),
            initrd,
            cmdline,
        });

        if let Some(fallback) = kernel.fallback.as_ref().and_then(|f| to_uefi_path(esp, f)) {
            entries.push(config::Entry {
                name: format!("{} ({}, fallback)", distro_name, kernel.variant),
                protocol,
                kernel: kernel_path,
                initrd: Some(fallback),
                cmdline: Some(cmd.to_string()),
            });
        }
    }

    config::Config {
        timeout: 3,
        default: 0,
        entries,
    }
}
