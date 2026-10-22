use core::fmt::Write;
use core::time::Duration;
use uefi::boot;
use uefi::proto::console::text::Color;
use uefi::proto::console::text::{Key, ScanCode};
use uefi::system::{with_stdin, with_stdout};

use crate::banner::{self};
pub fn select_entry(config: &config::Config) -> usize {
    if config.timeout == 0 {
        return config.default;
    }
    let mut ticks_left = config.timeout * 10;
    let mut countdown = true;
    let mut redraw = true;
    let mut selected = config.default;

    with_stdin(|f| f.reset(false)).ok();
    with_stdout(|out| out.enable_cursor(false)).ok();
    banner::print_banner();
    let cursor_position = with_stdout(|out| out.cursor_position());

    loop {
        if redraw {
            let seconds_left = match countdown {
                true => Some(ticks_left.div_ceil(10)),
                false => None,
            };
            draw(config, selected, seconds_left, cursor_position.1);
            redraw = false;
        }

        let input = with_stdin(|input| input.read_key()).ok().flatten();

        match input {
            Some(k) => {
                if countdown {
                    countdown = false;
                    redraw = true;
                }

                match k {
                    Key::Special(ScanCode::UP) => {
                        selected = selected.saturating_sub(1);
                        redraw = true;
                    }
                    Key::Special(ScanCode::DOWN) => {
                        if selected < config.entries.len() - 1 {
                            selected += 1;
                        }
                        redraw = true;
                    }
                    Key::Printable(c) => {
                        if char::from(c) == '\r' {
                            return selected;
                        }
                    }
                    _ => {}
                }
            }

            None => {
                boot::stall(Duration::from_millis(100));
                if countdown {
                    ticks_left -= 1;
                    if ticks_left == 0 {
                        return selected;
                    } else if ticks_left % 10 == 0 {
                        redraw = true;
                    }
                }
            }
        }
    }
}

fn draw(config: &config::Config, selected: usize, seconds_left: Option<u32>, cursor_row: usize) {
    with_stdout(|out| {
        for (i, entry) in config.entries.iter().enumerate() {
            out.set_cursor_position(0, cursor_row + i).ok();
            if i == selected {
                out.set_color(Color::Black, Color::LightGray).ok();
                write!(out, " > {}", entry.name).ok();
            } else {
                out.set_color(Color::LightGray, Color::Black).ok();
                write!(out, "   {}", entry.name).ok();
            }

            out.set_color(Color::LightGray, Color::Black).ok();
        }
        out.set_cursor_position(0, cursor_row + config.entries.len() + 1)
            .ok();
        if let Some(n) = seconds_left {
            write!(out, "Booting in {} s. Up/Down select, Enter boot", n).ok();
        } else {
            write!(out, "{:<50}", "Up/Down select, Enter boot").ok();
        }
    });
}
