#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        asm!("out dx, al", in("dx") 0x3F8u16, in("al") b'K');
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
