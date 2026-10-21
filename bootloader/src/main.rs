#![no_main]
#![no_std]
mod banner;
mod graphics;
use core::time::Duration;
use uefi::{
    boot::{AllocateType, MemoryType, exit_boot_services},
    prelude::*,
    proto::console::gop::PixelFormat,
};
mod fs;
mod linux;
mod loader;
mod memory;
extern crate alloc;
#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    banner::print_banner();
    let image = linux::load_linux().unwrap();
    unsafe {
        let mut map = exit_boot_services(Some(MemoryType::LOADER_DATA));
        let mut entries = [bzimage::e820::E820Entry::default(); bzimage::e820::E820_MAX_ENTRIES];
        let size = memory::build_e820(&mut map, &mut entries).unwrap();
        let raw = core::slice::from_raw_parts_mut(image.boot_params_addr as *mut u8, 4096);
        bzimage::e820::fill_e820(raw, &entries).unwrap();
        linux::boot_linux(image.kernel_base, image.boot_params_addr);
    }
}
