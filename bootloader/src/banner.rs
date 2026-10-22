use uefi::proto::console::text::Color;
use uefi::system::with_stdout;

const BANNER: [&str; 5] = [
    r"  __                    _  _                    _   ",
    r" / _|  ___  _ __  _ __ (_)| |__    ___    ___  | |_ ",
    r"| |_  / _ \| '__|| '__|| || '_ \  / _ \  / _ \ | __|",
    r"|  _||  __/| |   | |   | || |_) || (_) || (_) || |_ ",
    r"|_|   \___||_|   |_|   |_||_.__/  \___/  \___/  \__|",
];

pub fn set_color(fg: Color) {
    with_stdout(|out| {
        let _ = out.set_color(fg, Color::Black);
    });
}

pub fn print_banner() {
    with_stdout(|out| {
        let _ = out.clear();
    });

    set_color(Color::LightCyan);
    for line in BANNER {
        uefi::println!("{}", line);
    }

    set_color(Color::DarkGray);
    uefi::println!("{:>52}", concat!("v", env!("CARGO_PKG_VERSION")));

    set_color(Color::White);
    uefi::println!("  a Linux bootloader written in Rust");
    uefi::println!();

    set_color(Color::LightGray);
}
