use std::path::{self, Path, PathBuf};

use crate::kernels;

pub fn find_ukis(esp: &Path) -> Vec<PathBuf> {
    let dir = esp.join("EFI/Linux");
    if !dir.exists() {
        return Vec::new();
    }

    let Ok(files) = dir.read_dir() else {
        return Vec::new();
    };

    let mut ukis = Vec::new();

    for file in files {
        let Ok(entry) = file else {
            continue;
        };

        let path = entry.path();

        if path
            .extension()
            .is_some_and(|p| p.eq_ignore_ascii_case("efi"))
        {
            ukis.push(path);
        }
    }

    ukis.sort_by(|a, b| {
        kernels::kernel_cmp(
            a.file_stem().and_then(|s| s.to_str()).unwrap_or(""),
            b.file_stem().and_then(|s| s.to_str()).unwrap_or(""),
        )
    });

    ukis
}

pub fn find_windows(esp: &Path) -> Option<PathBuf> {
    let path = esp.join("EFI/Microsoft/Boot/bootmgfw.efi");
    if !path.is_file() {
        return None;
    }

    return Some(path);
}
