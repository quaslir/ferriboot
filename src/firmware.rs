use uefi::system::{self, firmware_revision, firmware_vendor, uefi_revision};

use core::fmt::Write;

pub fn print_firmware_info() {
    let info = firmware_vendor();
    let firmware_rev = firmware_revision();
    let uefi_rev = uefi_revision();
    system::with_stdout(|stdout| {
        writeln!(stdout, "{}", info).unwrap();
        writeln!(stdout, "{}", firmware_rev).unwrap();
        writeln!(stdout, "{}", uefi_rev).unwrap();
    });
}
