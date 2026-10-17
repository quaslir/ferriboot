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
#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    console::print_banner();
    firmware::print_firmware_info();
    let mut kernel_buffer = [0u8; 65536];
    uefi::println!("Trying to open kernel.elf...");
    let bytes_read = fs::read_test_file(cstr16!("kernel.elf"), &mut kernel_buffer).unwrap();
    uefi::println!("Volume opened, trying to open file...");
    let entry = loader::load_kernel(&kernel_buffer[..bytes_read]);
    uefi::println!("Kernel loaded, entry: {:#x}", entry.unwrap());
    boot::stall(Duration::from_secs(10));
    Status::SUCCESS
}
