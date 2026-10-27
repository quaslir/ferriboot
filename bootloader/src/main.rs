#![no_main]
#![no_std]
mod banner;
mod graphics;

use core::time::Duration;

use uefi::Error;
use uefi::proto::console::text::Color;
use uefi::{
    boot::{MemoryType, exit_boot_services},
    mem::memory_map::MemoryMap,
    prelude::*,
};
mod chainload;
mod conf;
mod fs;
mod linux;
mod memory;
mod menu;
extern crate alloc;

fn fail(e: uefi::Error) -> Status {
    banner::set_color(Color::LightRed);
    uefi::println!(
        "ferriboot: boot failed ({:?}), returning to firmware in 5 seconds...",
        e.status()
    );
    banner::set_color(Color::LightGray);
    boot::stall(Duration::from_secs(5));
    e.status()
}

fn hlt() -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}

#[entry]
fn main() -> Status {
    if uefi::helpers::init().is_err() {
        return Status::ABORTED;
    }

    let load_config = conf::load_config();
    let conf = match load_config {
        Ok(c) => c,
        Err(e) => return fail(e),
    };

    let index = menu::select_entry(&conf);
    system::with_stdout(|out| out.set_color(Color::LightGray, Color::Black)).ok();
    system::with_stdout(|out| out.clear()).ok();
    uefi::println!("Loading {}", conf.entries[index].name);
    let load_linux = linux::load_linux(&conf.entries[index]);

    let image = match load_linux {
        Ok(img) => img,
        Err(e) => return fail(e),
    };

    let raw_table = uefi::table::system_table_raw();
    let table = match raw_table {
        Some(t) => t.as_ptr() as u64,
        None => return fail(Error::from(uefi::Status::NOT_FOUND)),
    };

    unsafe {
        let mut map = exit_boot_services(Some(MemoryType::LOADER_DATA));
        let mut entries = [bzimage::e820::E820Entry::default(); bzimage::e820::E820_MAX_ENTRIES];
        let e820 = memory::build_e820(&mut map, &mut entries);

        let size = match e820 {
            Ok(s) => s,
            Err(_) => hlt(),
        };

        let raw = core::slice::from_raw_parts_mut(image.boot_params_addr as *mut u8, 4096);
        if bzimage::e820::fill_e820(raw, &entries[..size]).is_err() {
            hlt();
        }
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
