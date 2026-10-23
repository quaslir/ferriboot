use std::path::PathBuf;

use crate::kernels::Kernel;

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
    return "Linux".to_string();
}

pub fn build_config(kernels: &[Kernel], cmd: &str) -> config::Config {
    let distro_name = distro_name();
    let mut entries = Vec::new();
    for kernel in kernels {
        let name = format!("{} ({})", distro_name, kernel.variant);
        let protocol = config::Protocol::Linux;
        let image = kernel.image.clone();
        let initrd = kernel.initrd.clone();
        let cmdline = Some(cmd.to_string());

        entries.push(config::Entry {
            name,
            protocol,
            kernel: image,
            initrd,
            cmdline,
        });

        if let Some(fallback) = &kernel.fallback {
            entries.push(config::Entry {
                name: format!("{} ({}, fallback)", distro_name, kernel.variant),
                protocol,
                kernel: kernel.image.clone(),
                initrd: Some(fallback.clone()),
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
