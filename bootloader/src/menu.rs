use core::time::Duration;

use uefi::boot;
use uefi::proto::console::text::{Key, ScanCode};
use uefi::system::{with_stdin, with_stdout};
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

    loop {
        if redraw {
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
