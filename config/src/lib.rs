#![no_std]

use core::time;

use alloc::{string::String, vec::Vec};
extern crate alloc;

pub enum Protocol {
    Linux,
}

pub struct Entry {
    name: String,
    protocol: Protocol,
    kernel: String,
    initrd: Option<String>,
    cmdline: Option<String>,
}
struct PartialEntry {
    name: String,
    protocol: Option<Protocol>,
    kernel: Option<String>,
    initrd: Option<String>,
    cmdline: Option<String>,
    start_line: usize,
}

impl PartialEntry {
    pub fn finish(self) -> Result<Entry, ConfigError> {
        let kernel = self.kernel.ok_or(ConfigError {
            line: self.start_line,
            kind: ErrorKind::MissingKernel,
        })?;
        let protocol = self.protocol.ok_or(ConfigError {
            line: self.start_line,
            kind: ErrorKind::MissingProtocol,
        })?;
        Ok(Entry {
            name: self.name,
            protocol,
            kernel,
            initrd: self.initrd,
            cmdline: self.cmdline,
        })
    }
}

pub struct Config {
    timeout: u32,
    default: usize,
    entries: Vec<Entry>,
}

pub enum ErrorKind {
    MissingColon,
    UnknownKey,
    KeyOutsideEntry,
    GlobalKeyInsideEntry,
    InvalidNumber,
    UnknownProtocol,
    MissingKernel,
    MissingProtocol,
    NoEntries,
    DefaultOutOfRange,
}

pub struct ConfigError {
    line: usize,
    kind: ErrorKind,
}

fn split_key_value(line: &str) -> Option<(&str, &str)> {
    line.split_once(":").map(|(k, v)| (k.trim(), v.trim()))
}
fn apply_global(
    key: &str,
    value: &str,
    line: usize,
    timeout: &mut u32,
    default: &mut usize,
) -> Result<(), ConfigError> {
    match key {
        "timeout" => {
            let v = value.parse::<u32>().map_err(|_| ConfigError {
                line,
                kind: ErrorKind::InvalidNumber,
            })?;
            *timeout = v;
        }
        "default" => {
            let v = value.parse::<usize>().map_err(|_| ConfigError {
                line,
                kind: ErrorKind::InvalidNumber,
            })?;

            if v == 0 {
                return Err(ConfigError {
                    line,
                    kind: ErrorKind::DefaultOutOfRange,
                });
            }

            *default = v - 1;
        }

        "kernel" | "initrd" | "cmdline" | "protocol" => {
            return Err(ConfigError {
                line,
                kind: ErrorKind::KeyOutsideEntry,
            });
        }
        _ => {
            return Err(ConfigError {
                line,
                kind: ErrorKind::UnknownKey,
            });
        }
    }

    Ok(())
}
