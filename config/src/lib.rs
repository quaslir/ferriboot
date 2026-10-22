#![no_std]

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::ErrorKind::{MissingColon, NoEntries};
extern crate alloc;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Linux,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub protocol: Protocol,
    pub kernel: String,
    pub initrd: Option<String>,
    pub cmdline: Option<String>,
}
#[derive(Debug, Default)]
struct PartialEntry {
    pub name: String,
    pub protocol: Option<Protocol>,
    pub kernel: Option<String>,
    pub initrd: Option<String>,
    pub cmdline: Option<String>,
    pub start_line: usize,
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

    pub fn new(name: &str, start_line: usize) -> Self {
        PartialEntry {
            name: name.to_string(),
            protocol: None,
            kernel: None,
            initrd: None,
            cmdline: None,
            start_line,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub timeout: u32,
    pub default: usize,
    pub entries: Vec<Entry>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigError {
    pub line: usize,
    pub kind: ErrorKind,
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

fn parse_protocol(value: &str) -> Option<Protocol> {
    match value {
        "linux" => Some(Protocol::Linux),
        _ => None,
    }
}

fn apply_entry_key(
    entry: &mut PartialEntry,
    key: &str,
    value: &str,
    line: usize,
) -> Result<(), ConfigError> {
    match key {
        "protocol" => {
            let protocol = parse_protocol(value).ok_or(ConfigError {
                line: line,
                kind: ErrorKind::UnknownProtocol,
            })?;
            entry.protocol = Some(protocol);
        }

        "kernel" => entry.kernel = Some(String::from(value)),
        "initrd" => entry.initrd = Some(String::from(value)),
        "cmdline" => entry.cmdline = Some(String::from(value)),
        "timeout" | "default" => {
            return Err(ConfigError {
                line: line,
                kind: ErrorKind::GlobalKeyInsideEntry,
            });
        }

        _ => {
            return Err(ConfigError {
                line: line,
                kind: ErrorKind::UnknownKey,
            });
        }
    }

    Ok(())
}

pub fn parse(text: &str) -> Result<Config, ConfigError> {
    let mut timeout = 3;
    let mut default = 0;
    let mut entries = Vec::new();
    let mut current: Option<PartialEntry> = None;

    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with("#") {
            continue;
        }
        if let Some(name) = line.strip_prefix('/') {
            if let Some(prev) = current.take() {
                entries.push(prev.finish()?);
            }

            current = Some(PartialEntry::new(name.trim(), index + 1));
            continue;
        }

        let (key, value) = split_key_value(line).ok_or(ConfigError {
            line: index + 1,
            kind: ErrorKind::MissingColon,
        })?;

        match current.as_mut() {
            Some(entry) => apply_entry_key(entry, key, value, index + 1)?,
            None => apply_global(key, value, index + 1, &mut timeout, &mut default)?,
        }
    }

    if let Some(prev) = current.take() {
        entries.push(prev.finish()?);
    }

    if entries.is_empty() {
        return Err(ConfigError {
            line: 0,
            kind: ErrorKind::NoEntries,
        });
    }

    if default >= entries.len() {
        return Err(ConfigError {
            line: 0,
            kind: ErrorKind::DefaultOutOfRange,
        });
    }

    Ok(Config {
        timeout,
        default,
        entries,
    })
}
