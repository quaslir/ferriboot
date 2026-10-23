use std::path::Path;

pub struct Kernel {
    pub variant: String,
    pub image: String,
    pub initrd: Option<String>,
    pub fallback: Option<String>,
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
                let initramfs_path = format!("initramfs-{}.img", variant);
                let initrd = esp.join(&initramfs_path).exists().then_some(initramfs_path);

                let fallback_path = format!("initramfs-{}-fallback.img", variant);
                let fallback = esp.join(&fallback_path).exists().then_some(fallback_path);

                kernels.push(Kernel {
                    variant: variant.to_string(),
                    image: converted.to_string(),
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
