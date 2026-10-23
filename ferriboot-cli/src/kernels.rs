use std::path::{Path, PathBuf};
#[derive(Clone)]
pub struct Kernel {
    pub variant: String,
    pub image: PathBuf,
    pub initrd: Option<PathBuf>,
    pub fallback: Option<PathBuf>,
}

const INITRD_PATTERNS: [(&str, &str); 2] = [
    ("initramfs-", ".img"), // Arch (mkinitcpio), Fedora (dracut)
    ("initrd.img-", ""),    // Ubuntu, Debian
];

fn find_initrd(dir: &Path, variant: &str) -> Option<PathBuf> {
    for (prefix, suffix) in INITRD_PATTERNS {
        let file_name = format!("{}{}{}", prefix, variant, suffix);
        if dir.join(&file_name).exists() {
            return Some(dir.join(&file_name));
        }
    }

    None
}

pub fn find_all_kernels(esp: &Path) -> Result<Vec<Kernel>, String> {
    let mut dirs = Vec::new();
    dirs.push(esp.to_path_buf());

    if let (Ok(boot), Ok(esp_real)) = (std::fs::canonicalize("/boot"), std::fs::canonicalize(esp)) {
        if boot != esp_real {
            dirs.push(boot);
        }
    }

    let mut all = Vec::<Kernel>::new();

    for (i, dir) in dirs.iter().enumerate() {
        match find_kernels(dir) {
            Ok(found) => {
                for kernel in found {
                    if all.iter().any(|k| k.variant == kernel.variant) {
                        continue;
                    }
                    all.push(kernel);
                }
            }
            Err(e) => {
                if i == 0 {
                    return Err(format!("cannot read {}: {}", dir.display(), e));
                }
                eprintln!("cannot read {}: {}", dir.display(), e);
            }
        }
    }
    all.sort_by(|a, b| a.variant.cmp(&b.variant));
    Ok(all)
}

pub fn find_kernels(esp: &Path) -> std::io::Result<Vec<Kernel>> {
    let mut kernels = Vec::new();
    let result = std::fs::read_dir(esp)?;

    for entry in result {
        if entry.is_err() {
            continue;
        }
        let dir_entry = entry.unwrap();
        let file_name_os = dir_entry.file_name();
        let file_name = file_name_os.to_str();
        if let Some(converted) = file_name {
            if let Some(variant) = converted.strip_prefix("vmlinuz-") {
                let initrd = find_initrd(esp, variant);
                let fallback_path = format!("initramfs-{}-fallback.img", variant);
                let fallback = esp.join(&fallback_path);
                let fallback = fallback.exists().then_some(fallback);
                kernels.push(Kernel {
                    variant: variant.to_string(),
                    image: esp.join(converted),
                    initrd,
                    fallback,
                });
            } else {
                continue;
            }
        } else {
            continue;
        }
    }

    Ok(kernels)
}
