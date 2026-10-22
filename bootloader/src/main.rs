#![no_main]
#![no_std]
mod banner;
mod graphics;
use core::time::Duration;
use uefi::{
    boot::{AllocateType, MemoryType, exit_boot_services},
    mem::memory_map::MemoryMap,
    prelude::*,
    proto::console::gop::PixelFormat,
};
mod conf;
mod fs;
mod linux;
mod loader;
mod memory;
use config;
extern crate alloc;
#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    banner::print_banner();
    let conf = conf::load_config().unwrap();
    let image = linux::load_linux(conf.default_entry()).unwrap();
    let table = uefi::table::system_table_raw().unwrap().as_ptr() as u64;

    unsafe {
        let mut map = exit_boot_services(Some(MemoryType::LOADER_DATA));
        let mut entries = [bzimage::e820::E820Entry::default(); bzimage::e820::E820_MAX_ENTRIES];
        let size = memory::build_e820(&mut map, &mut entries).unwrap();
        let raw = core::slice::from_raw_parts_mut(image.boot_params_addr as *mut u8, 4096);
        bzimage::e820::fill_e820(raw, &entries).unwrap();
        let meta = map.meta();
        let map_buffer = map.buffer().as_ptr() as u64;
        bzimage::efi_info::fill_efi_info(
            raw,
            table,
            map_buffer,
            meta.map_size as u32,
            meta.desc_size as u32,
            meta.desc_version as u32,
        );
        linux::boot_linux(image.kernel_base, image.boot_params_addr);
    }
}
