use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::kernels::Kernel;

fn machine_id() -> String {
    let Ok(text) = fs::read_to_string("/etc/machine-id") else {
        return "default".to_string();
    };

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "default".to_string();
    }

    String::from(trimmed)
}

fn kernels_dir(esp: &Path) -> PathBuf {
    esp.join("EFI/ferriboot").join(machine_id())
}

pub fn plan_kernel(esp: &Path, kernel: &Kernel) -> (Kernel, Vec<(PathBuf, PathBuf)>) {
    if kernel.image.starts_with(esp) {
        return (kernel.clone(), Vec::new());
    }

    let target = esp.join(kernels_dir(esp)).join(&kernel.variant);
    let mut result = Vec::<(PathBuf, PathBuf)>::new();

    let new_kernel = target.join("linux");
    result.push((kernel.image.clone(), new_kernel.clone()));
    let initrd = match &kernel.initrd {
        Some(old) => {
            let new_path = target.join("initrd");
            result.push((old.clone(), new_path.clone()));
            Some(new_path)
        }
        None => None,
    };

    let fallback = match &kernel.fallback {
        Some(old) => {
            let new_path = target.join("initrd-fallback");
            result.push((old.clone(), new_path.clone()));
            Some(new_path)
        }
        None => None,
    };

    (
        Kernel {
            variant: kernel.variant.clone(),
            image: new_kernel,
            initrd,
            fallback,
        },
        result,
    )
}

pub fn plan_cleanup(esp: &Path, kernels: &[Kernel]) -> Vec<PathBuf> {
    let dir = kernels_dir(esp);
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut stale = Vec::<PathBuf>::new();
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };

        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(file_name) = path.file_name() else {
            continue;
        };

        let Some(name) = file_name.to_str() else {
            continue;
        };
        if !kernels.iter().any(|k| k.variant == name) {
            stale.push(path);
        }
    }

    stale
}

pub fn is_up_to_date(from: &Path, to: &Path) -> bool {
    let (Ok(a), Ok(b)) = (fs::metadata(from), fs::metadata(to)) else {
        return false;
    };

    if a.len() != b.len() {
        return false;
    }

    let (Ok(ma), Ok(mb)) = (a.modified(), b.modified()) else {
        return false;
    };

    mb >= ma
}
