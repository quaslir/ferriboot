use core::fmt::Write;
use uefi::system;
pub fn print_banner() {
    system::with_stdout(|stdout| {
        writeln!(stdout, "__            _ _                 _").unwrap();
        writeln!(stdout, "/ _| ___ _ __ _(_) |__   ___   ___ | |_").unwrap();
        writeln!(stdout, r"| |_ / _ \ '__| | | '_ \ / _ \ / _ \| __|").unwrap();
        writeln!(stdout, "|  _|  __/ |  | | | |_) | (_) | (_) | |_").unwrap();
        writeln!(stdout, r"|_|  \___|_|  |_|_|_.__/ \___/ \___/ \__|").unwrap();
    });
}
