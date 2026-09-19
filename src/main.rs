#![no_main]
#![no_std]
mod console;
use core::time::Duration;
use log::info;
use uefi::prelude::*;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    console::print_banner();
    boot::stall(Duration::from_secs(10));
    Status::SUCCESS
}
