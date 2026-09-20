use uefi::Result;
use uefi::boot::{self, open_protocol_exclusive};
use uefi::proto;
use uefi::proto::console::gop::{self, GraphicsOutput, ModeInfo};
pub fn print_graphics_info() -> Result {
    let handle = boot::get_handle_for_protocol::<gop::GraphicsOutput>()?;
    let mut protocol = open_protocol_exclusive::<gop::GraphicsOutput>(handle)?;
    let info = protocol.current_mode_info();

    let (w, h) = info.resolution();
    let stride = info.stride();
    let format = info.pixel_format();

    let mut fb = protocol.frame_buffer();
    let addr = fb.as_mut_ptr();
    let size = fb.size();

    uefi::println!("Resolution:  {}x{}", w, h);
    uefi::println!("Stride:      {} pixels", stride);
    uefi::println!("Pixel format: {:?}", format);
    uefi::println!("Framebuffer: {:p}, {} bytes", addr, size);
    Ok(())
}
