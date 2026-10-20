#![no_main]
#![no_std]
mod console;
mod firmware;
mod graphics;
mod print;
use core::time::Duration;

use boot_info::{BootInfo, Framebuffer};
use uefi::{
    boot::{AllocateType, MemoryType},
    prelude::*,
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

    uefi::println!("[linux] kernel_base:  {:#x}", image.kernel_base);
    uefi::println!(
        "[linux] initrd:       {:#x}, size {:#x}",
        image.initrd_addr,
        image.initrd_size
    );
    uefi::println!("[linux] cmdline:      {:#x}", image.cmdline_addr);
    uefi::println!("[linux] boot_params:  {:#x}", image.boot_params_addr);

    let bp = unsafe { core::slice::from_raw_parts(image.boot_params_addr as *const u8, 4096) };
    let rd32 = |off: usize| u32::from_le_bytes(bp[off..off + 4].try_into().unwrap());

    uefi::println!("[bp] magic        {:#x} (expect 0x53726448)", rd32(0x202));
    uefi::println!("[bp] type_loader  {:#x} (expect 0xff)", bp[0x210]);
    uefi::println!("[bp] loadflags    {:#x} (bit0 = 1)", bp[0x211]);
    uefi::println!("[bp] ramdisk_img  {:#x} (expect initrd)", rd32(0x218));
    uefi::println!("[bp] ramdisk_size {:#x}", rd32(0x21C));
    uefi::println!("[bp] cmd_line_ptr {:#x} (expect cmdline)", rd32(0x228));
    boot::stall(Duration::from_millis(100000));
    Status::SUCCESS
}
