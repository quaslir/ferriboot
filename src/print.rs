use core::fmt::Write;
use uefi::system;

pub fn print_to_console(text: &str) {
    system::with_stdout(|stdout| {
        writeln!(stdout, "{}", text).unwrap();
    });
}
