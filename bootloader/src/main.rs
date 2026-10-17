#![no_main]
#![no_std]
mod console;
mod firmware;
mod graphics;
mod print;
use core::time::Duration;
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
    let entry = loader::load_kernel(&kernel_buffer);
    uefi::println!("Kernel loaded, entry: {:#x}", entry.unwrap());
    boot::stall(Duration::from_secs(10));
    Status::SUCCESS
}
