use uefi::boot::{self, open_protocol_exclusive};
use uefi::proto::console::gop::{self};
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Rgb = 0,
    Bgr = 1,
}

#[repr(C)]
pub struct Framebuffer {
    pub base: u64,
    pub size: usize,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub pixel_format: PixelFormat,
}
pub fn graphics_info() -> uefi::Result<Framebuffer> {
    let handle = boot::get_handle_for_protocol::<gop::GraphicsOutput>()?;
    let mut protocol = open_protocol_exclusive::<gop::GraphicsOutput>(handle)?;
    let info = protocol.current_mode_info();

    let (w, h) = info.resolution();
    let stride = info.stride();
    let format = info.pixel_format();

    let pixel_format = match format {
        gop::PixelFormat::Rgb => PixelFormat::Rgb,
        gop::PixelFormat::Bgr => PixelFormat::Bgr,
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
