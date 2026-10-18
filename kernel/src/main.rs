#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "sysv64" fn _start(boot_info: &'static boot_info::BootInfo) -> ! {
    unsafe {
        asm!("out dx, al", in("dx") 0x3F8u16, in("al") b'K');
    }

    let base = boot_info.framebuffer.base as *mut u32;
    let width = boot_info.framebuffer.width;
    let height = boot_info.framebuffer.height;
    let stride = boot_info.framebuffer.stride;

    for y in 0..height {
        for x in 0..width {
            let pixel_idx = y * stride + x;
            unsafe {
                core::ptr::write_volatile(base.add(pixel_idx), 0x00FF0000);
            }
        }
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[panic_handler]

fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
