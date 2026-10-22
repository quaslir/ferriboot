#![no_std]

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

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

impl Config {
    pub fn default_entry(&self) -> &Entry {
        return &self.entries[self.default];
    }
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

impl ErrorKind {
    pub fn message(&self) -> &'static str {
        match self {
            ErrorKind::MissingColon => "expected 'key: value'",
            ErrorKind::UnknownKey => "unknown key",
            ErrorKind::KeyOutsideEntry => "key must be inside an entry (add '/Name' above it)",
            ErrorKind::GlobalKeyInsideEntry => {
                "'timeout' and 'default' must be before the first entry"
            }
            ErrorKind::InvalidNumber => "value is not a number",
            ErrorKind::UnknownProtocol => "unknown protocol (expected 'linux')",
            ErrorKind::MissingKernel => "entry has no 'kernel'",
            ErrorKind::MissingProtocol => "entry has no 'protocol'",
            ErrorKind::NoEntries => "no boot entries defined",
            ErrorKind::DefaultOutOfRange => "'default' points to an entry that does not exist",
        }
    }
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

#[cfg(test)]

mod tests {
    use alloc::format;

    use crate::*;
    const BASE: &str = "/Arch\nprotocol: linux\nkernel: vmlinuz";

    const FULL: &str = "# ferriboot test config\n\
    timeout: 5\n\
    default: 2\n\
    \n\
    /Arch Linux\n\
        protocol: linux\n\
        kernel: vmlinuz-linux\n\
        initrd: initramfs-linux.img\n\
        cmdline: root=UUID=1234 rw console=ttyS0,115200:8n1\n\
    \n\
    /Arch Linux (fallback)\n\
        protocol: linux\n\
        kernel: vmlinuz-linux\n";

    fn append(extra: &str) -> String {
        format!("{}\n{}", BASE, extra)
    }

    fn prepend(extra: &str) -> String {
        format!("{}\n{}", extra, BASE)
    }
    #[test]

    pub fn min_config() {
        let result = parse(BASE);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.timeout, 3);
        assert_eq!(parsed.default, 0);
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].name, "Arch");
        assert_eq!(parsed.entries[0].kernel, "vmlinuz");
        assert_eq!(parsed.entries[0].protocol, Protocol::Linux);
    }

    #[test]

    pub fn full_working_config() {
        let result = parse(FULL);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.timeout, 5);
        assert_eq!(parsed.default, 1);
        let entries = parsed.entries;
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].protocol, Protocol::Linux);
        assert_eq!(entries[0].name, "Arch Linux");
        assert_eq!(entries[0].kernel, "vmlinuz-linux");
        assert_eq!(
            entries[0].cmdline.as_deref(),
            Some("root=UUID=1234 rw console=ttyS0,115200:8n1")
        );
        assert_eq!(entries[0].initrd.as_deref(), Some("initramfs-linux.img"));

        assert_eq!(entries[1].protocol, Protocol::Linux);
        assert_eq!(entries[1].name, "Arch Linux (fallback)");
        assert_eq!(entries[1].kernel, "vmlinuz-linux");
        assert_eq!(entries[1].cmdline, None);
        assert_eq!(entries[1].initrd, None);
    }

    #[test]

    pub fn key_outside_entry() {
        let config = prepend("kernel: vmlinuz");
        let result = parse(&config);
        assert_eq!(
            result,
            Err(ConfigError {
                line: 1,
                kind: ErrorKind::KeyOutsideEntry
            })
        );
    }

    #[test]

    pub fn global_key_inside_entry() {
        let config = append("timeout: 5");
        let result = parse(&config);
        assert_eq!(
            result,
            Err(ConfigError {
                line: 4,
                kind: ErrorKind::GlobalKeyInsideEntry
            })
        );
    }

    #[test]

    pub fn unknown_key() {
        let config = append("kernal: vmlinuz");
        let result = parse(&config);
        assert_eq!(
            result,
            Err(ConfigError {
                line: 4,
                kind: ErrorKind::UnknownKey
            })
        );
    }

    #[test]

    pub fn missing_colon() {
        let config = append("kernal vmlinuz");
        let result = parse(&config);
        assert_eq!(
            result,
            Err(ConfigError {
                line: 4,
                kind: ErrorKind::MissingColon
            })
        );
    }

    #[test]

    pub fn invalid_number() {
        let config = prepend("timeout: abc");
        let result = parse(&config);
        assert_eq!(
            result,
            Err(ConfigError {
                line: 1,
                kind: ErrorKind::InvalidNumber
            })
        );
    }

    #[test]

    pub fn unknown_protocol() {
        let config = append("protocol: windows");
        let result = parse(&config);
        assert_eq!(
            result,
            Err(ConfigError {
                line: 4,
                kind: ErrorKind::UnknownProtocol
            })
        );
    }

    #[test]

    pub fn missing_kernel() {
        let result = parse("/Arch\nprotocol: linux");
        assert_eq!(
            result,
            Err(ConfigError {
                line: 1,
                kind: ErrorKind::MissingKernel
            })
        );
    }

    #[test]

    pub fn missing_protocol() {
        let result = parse("/Arch\nkernel: vmlinuz");
        assert_eq!(
            result,
            Err(ConfigError {
                line: 1,
                kind: ErrorKind::MissingProtocol
            })
        );
    }

    #[test]

    pub fn no_entries() {
        let result = parse("");
        assert_eq!(
            result,
            Err(ConfigError {
                line: 0,
                kind: ErrorKind::NoEntries
            })
        );
    }

    #[test]

    pub fn default_out_of_range() {
        let config = prepend("default: 5");
        let result = parse(&config);
        assert_eq!(
            result,
            Err(ConfigError {
                line: 0,
                kind: ErrorKind::DefaultOutOfRange
            })
        );
    }
}
