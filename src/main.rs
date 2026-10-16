#![no_main]
#![no_std]
mod console;
mod firmware;
mod graphics;
mod print;
use core::time::Duration;
use uefi::prelude::*;
mod fs;
mod memory;
#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    console::print_banner();
    firmware::print_firmware_info();
    graphics::print_graphics_info();
    memory::print_memory_info();
    fs::read_test_file();
    boot::stall(Duration::from_secs(10));
    Status::SUCCESS
}
