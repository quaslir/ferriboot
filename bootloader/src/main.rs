#![no_main]
#![no_std]
mod console;
mod firmware;
mod graphics;
mod print;
use core::time::Duration;

use boot_info::{BootInfo, Framebuffer};
use bzimage::fill_screen_info;
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
    /*console::print_banner();
    firmware::print_firmware_info();
    let kernel_buffer = fs::read_file(cstr16!("kernel.elf")).unwrap();
    let entry = loader::load_kernel(&kernel_buffer).unwrap();
    drop(kernel_buffer);
    uefi::println!("Kernel loaded, entry: {:#x}", entry);
    let framebuffer = graphics::graphics_info().unwrap();
    let boot_info_page =
        boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1).unwrap(); // Memory for allocating BootInfo.
    let boot_info = boot_info_page.cast::<BootInfo>().as_ptr();

    unsafe {
        core::ptr::write(
            boot_info,
            BootInfo {
                framebuffer: framebuffer,
            },
        );
    }

    unsafe {
        let _memory_map = boot::exit_boot_services(None);
        let casted: extern "sysv64" fn(*const BootInfo) -> ! = core::mem::transmute(entry as usize);
        casted(boot_info);
    }
    */

    let image = linux::load_linux().unwrap();
    unsafe {
        let mut map = exit_boot_services(Some(MemoryType::LOADER_DATA));
        let mut entries = [bzimage::E820Entry::default(); bzimage::E820_MAX_ENTRIES];
        let size = memory::build_e820(&mut map, &mut entries).unwrap();
        let raw = core::slice::from_raw_parts_mut(image.boot_params_addr as *mut u8, 4096);
        bzimage::fill_e820(raw, &entries).unwrap();
        linux::boot_linux(image.kernel_base, image.boot_params_addr);
    }
}
