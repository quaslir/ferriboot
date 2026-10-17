use alloc::vec;
use alloc::vec::Vec;
use uefi::Error;
use uefi::boot;
use uefi::data_types::CStr16;
use uefi::proto::media::file::File;
use uefi::proto::media::file::FileAttribute;
use uefi::proto::media::file::FileInfo;
use uefi::proto::media::file::FileMode;
pub fn read_file(filename: &CStr16) -> uefi::Result<Vec<u8>> {
    let handle = boot::image_handle();
    let mut system = boot::get_image_file_system(handle)?;
    let mut root = system.open_volume()?;
    let result = root.open(filename, FileMode::Read, FileAttribute::empty())?;
    let mut regular = result
        .into_regular_file()
        .ok_or_else(|| Error::from(uefi::Status::LOAD_ERROR))?;
    let bxinfo = regular.get_boxed_info::<FileInfo>()?;
    let mut buffer = vec![0; bxinfo.file_size() as usize];
    let bytes_read = regular.read(&mut buffer)?;
    if bytes_read != bxinfo.file_size() as usize {
        return Err(uefi::Error::from(uefi::Status::END_OF_FILE));
    }
    Ok(buffer)
}
