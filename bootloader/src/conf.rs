use alloc::vec;
use uefi::cstr16;

use crate::fs;
pub fn load_config() -> uefi::Result<config::Config> {
    let mut file = fs::open_file(cstr16!(r"\EFI\ferriboot\ferriboot.conf"))
        .map_err(|_| uefi::Error::from(uefi::Status::NOT_FOUND))?;
    let file_size =
        fs::file_size(&mut file).map_err(|_| uefi::Error::from(uefi::Status::LOAD_ERROR))?;

    let mut buffer = vec![0u8; file_size];
    fs::read_file(&mut file, &mut buffer)
        .map_err(|_| uefi::Error::from(uefi::Status::LOAD_ERROR))?;
    let converted =
        core::str::from_utf8(&buffer).map_err(|_| uefi::Error::from(uefi::Status::LOAD_ERROR))?;
    config::parse(converted).map_err(|_| uefi::Error::from(uefi::Status::LOAD_ERROR))
}
