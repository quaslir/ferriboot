use std::path::{Path, PathBuf};

pub fn find_esp() -> Option<PathBuf> {
    let candidates = ["/boot", "efi", "/boot/efi"];

    for curr in candidates {
        let path = PathBuf::from(curr);
        let efi_path = path.join("EFI");
        if efi_path.is_dir() {
            return Some(efi_path);
        }
    }

    None
}
