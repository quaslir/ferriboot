use crate::fs;
use uefi::CString16;
use uefi::boot;
use uefi::boot::LoadImageSource;
pub fn start_efi(entry: &config::Entry) -> uefi::Result<()> {
    let Some(path) = entry.path.as_deref() else {
        uefi::println!("ferriboot: Missing path");
        return Err(uefi::Error::from(uefi::Status::LOAD_ERROR));
    };

    let converted_path = CString16::try_from(path).map_err(|_| {
        uefi::println!("ferriboot: invalid path '{}'", path);
        uefi::Error::from(uefi::Status::LOAD_ERROR)
    })?;

    let mut file = fs::open_file(&converted_path).inspect_err(|e| {
        uefi::println!(
            "ferriboot: cannot open path '{}' ({:?})",
            converted_path,
            e.status()
        );
    })?;

    let file_size = fs::file_size(&mut file).inspect_err(|e| {
        uefi::println!(
            "ferriboot: cannot read path '{}' ({:?})",
            converted_path,
            e.status()
        );
    })?;

    let mut buffer = alloc::vec![0u8; file_size];

    fs::read_file(&mut file, &mut buffer).inspect_err(|e| {
        uefi::println!(
            "ferriboot: cannot read path '{}' ({:?})",
            converted_path,
            e.status()
        );
    })?;

    let handle = boot::load_image(
        boot::image_handle(),
        LoadImageSource::FromBuffer {
            buffer: &buffer,
            file_path: None,
        },
    )
    .inspect_err(|e| {
        uefi::println!(
            "ferriboot: cannot load image '{}' ({:?})",
            converted_path,
            e.status()
        );
    })?;

    boot::start_image(handle)
}
