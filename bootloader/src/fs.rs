use uefi::Error;
use uefi::boot;
use uefi::data_types::CStr16;
use uefi::proto::media::file::File;
use uefi::proto::media::file::FileAttribute;
use uefi::proto::media::file::FileMode;
pub fn read_test_file(filename: &CStr16, buffer: &mut [u8]) -> uefi::Result<usize> {
    let handle = boot::image_handle();
    let mut system = boot::get_image_file_system(handle)?;
    let mut root = system.open_volume()?;
    let result = root.open(filename, FileMode::Read, FileAttribute::empty())?;
    let mut regular = result
        .into_regular_file()
        .ok_or_else(|| Error::from(uefi::Status::LOAD_ERROR))?;
    let bytes_read = regular.read(buffer)?;
    Ok(bytes_read)
}
