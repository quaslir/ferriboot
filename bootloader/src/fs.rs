use uefi::Error;
use uefi::boot;
use uefi::data_types::CStr16;
use uefi::proto::media::file::File;
use uefi::proto::media::file::FileAttribute;
use uefi::proto::media::file::FileInfo;
use uefi::proto::media::file::FileMode;
use uefi::proto::media::file::RegularFile;
pub fn open_file(filename: &CStr16) -> uefi::Result<RegularFile> {
    let handle = boot::image_handle();
    let mut system = boot::get_image_file_system(handle)?;
    let mut root = system.open_volume()?;
    let result = root.open(filename, FileMode::Read, FileAttribute::empty())?;
    let regular = result
        .into_regular_file()
        .ok_or_else(|| Error::from(uefi::Status::LOAD_ERROR))?;
    Ok(regular)
}

pub fn read_file(file: &mut RegularFile, buffer: &mut [u8]) -> uefi::Result<()> {
    let mut count = 0;

    while count < buffer.len() {
        let read = file
            .read(&mut buffer[count..])
            .map_err(|e| uefi::Error::from(e.status()))?;
        if read == 0 {
            return Err(uefi::Error::from(uefi::Status::END_OF_FILE));
        }
        count += read;
    }
    Ok(())
}

pub fn file_size(file: &mut RegularFile) -> uefi::Result<usize> {
    let size = file.get_boxed_info::<FileInfo>()?.file_size();
    Ok(size as usize)
}
