#![no_std]

#[repr(C)]
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
#[repr(C)]
pub struct BootInfo {
    pub framebuffer: Framebuffer,
}
