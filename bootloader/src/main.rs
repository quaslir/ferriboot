#![no_main]
#![no_std]
mod console;
mod firmware;
mod graphics;
mod print;
use uefi::prelude::*;
mod fs;
mod loader;
mod memory;
extern crate alloc;
#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    console::print_banner();
    firmware::print_firmware_info();
    let kernel_buffer = fs::read_file(cstr16!("kernel.elf")).unwrap();
    let entry = loader::load_kernel(&kernel_buffer).unwrap();
    drop(kernel_buffer);
    uefi::println!("Kernel loaded, entry: {:#x}", entry);
    unsafe {
        let _memory_map = boot::exit_boot_services(None);
        let casted: extern "sysv64" fn() -> ! = core::mem::transmute(entry as usize);
        casted();
    }

    Status::SUCCESS
}
