#![no_main]
#![no_std]
mod console;
mod firmware;
mod graphics;
mod print;
use core::time::Duration;
use uefi::prelude::*;
mod memory;
#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    memory::print_memory_info();
    boot::stall(Duration::from_secs(10));
    Status::SUCCESS
}
