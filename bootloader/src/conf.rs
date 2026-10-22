use alloc::vec;
use uefi::cstr16;

use crate::fs;
pub fn load_config() -> uefi::Result<config::Config> {
    let mut file = fs::open_file(cstr16!(r"\EFI\ferriboot\ferriboot.conf")).map_err(|e| {
        uefi::println!(
            "ferriboot: cannot open {} ({:?})",
            r"\EFI\ferriboot\ferriboot.conf",
            e.status()
        );
        e
    })?;
    let file_size = fs::file_size(&mut file).map_err(|e| {
        uefi::println!("ferriboot: cannot read ferriboot.conf ({:?})", e.status());
        e
    })?;

    let mut buffer = vec![0u8; file_size];
    fs::read_file(&mut file, &mut buffer).map_err(|e| {
        uefi::println!("ferriboot: cannot read ferriboot.conf ({:?})", e.status());
        e
    })?;
    let converted = core::str::from_utf8(&buffer).map_err(|e| {
        uefi::println!("ferriboot: ferriboot.conf is not valid UTF-8");
        uefi::Error::from(uefi::Status::LOAD_ERROR)
    })?;
    config::parse(converted).map_err(|e| {
        if e.line == 0 {
            uefi::println!("ferriboot: ferriboot.conf: {}", e.kind.message());
        } else {
            uefi::println!(
                "ferriboot: ferriboot.conf, line {}: {}",
                e.line,
                e.kind.message()
            );
        }
        uefi::Error::from(uefi::Status::LOAD_ERROR)
    })
}
