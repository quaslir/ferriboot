use uefi::Error;

use uefi::boot;
use uefi::cstr16;
use uefi::proto::media::file::File;
use uefi::proto::media::file::FileAttribute;
use uefi::proto::media::file::FileMode;

pub fn read_test_file(filename: &str) -> uefi::Result {
    let handle = boot::image_handle();
    let mut system = boot::get_image_file_system(handle)?;
    let mut root = system.open_volume()?;
    let result = root.open(cstr16!("test.txt"), FileMode::Read, FileAttribute::empty())?;
    let regular = result.into_regular_file();
    let mut buffer = [0u8; 512];

    let bytes_read = regular.unwrap().read(&mut buffer)?;

    if let Ok(text) = core::str::from_utf8(&buffer[..bytes_read]) {
        uefi::println!("{}", text);
    }

    Ok(())
}
