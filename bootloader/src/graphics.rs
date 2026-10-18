use boot_info::Framebuffer;
use uefi::boot::{self, open_protocol_exclusive};
use uefi::proto::console::gop::{self, PixelFormat};
pub fn graphics_info() -> uefi::Result<boot_info::Framebuffer> {
    let handle = boot::get_handle_for_protocol::<gop::GraphicsOutput>()?;
    let mut protocol = open_protocol_exclusive::<gop::GraphicsOutput>(handle)?;
    let info = protocol.current_mode_info();

    let (w, h) = info.resolution();
    let stride = info.stride();
    let format = info.pixel_format();

    let pixel_format = match format {
        PixelFormat::Rgb => boot_info::PixelFormat::Rgb,
        PixelFormat::Bgr => boot_info::PixelFormat::Bgr,
        _ => return Err(uefi::Error::from(uefi::Status::UNSUPPORTED)),
    };

    let mut fb = protocol.frame_buffer();
    let addr = fb.as_mut_ptr();
    let size = fb.size();

    let framebuffer = Framebuffer {
        base: addr as u64,
        width: w,
        height: h,
        size: size,
        stride: stride,
        pixel_format: pixel_format,
    };
    Ok(framebuffer)
}
