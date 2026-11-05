use std::{fs, io::ErrorKind, path::PathBuf};

pub const PATH: &str = "/etc/ferriboot.conf";

#[derive(Debug, Default, PartialEq)]
pub struct Settings {
    pub timeout: Option<u32>,
    pub default: Option<String>,
}

pub fn parse(text: &str) -> Result<Settings, String> {
    let mut settings = Settings::default();

    for (i, line) in text.lines().enumerate() {
        let normalized = line.trim();
        if normalized.is_empty() || normalized.starts_with('#') {
            continue;
        }
        let Some((key, val)) = normalized.split_once(':') else {
            return Err(format!("line {}: expected 'key: value'", i + 1));
        };

        let key = key.trim();
        let val = val.trim();

        match key {
            "timeout" => {
                settings.timeout = Some(
                    val.parse::<u32>()
                        .map_err(|_| format!("line {}: timeout must be a number", i + 1))?,
                )
            }
            "default" => settings.default = Some(val.to_string()),
            _ => return Err(format!("line {}: unknown key '{}'", i + 1, key)),
        }
    }

    Ok(settings)
}

pub fn apply(config: &mut config::Config, settings: &Settings) -> Option<String> {
    if let Some(timeout) = settings.timeout {
        config.timeout = timeout;
    }
    if let Some(default) = &settings.default {
        let Some(i) = config.entries.iter().position(|s| &s.name == default) else {
            return Some(format!(
                "default entry '{}' not found, using the first entry",
                default
            ));
        };

        config.default = i;
    }

    None
}

pub fn load() -> Result<Settings, String> {
    let read_res = fs::read_to_string(PathBuf::from(PATH));

    match read_res {
        Ok(text) => parse(&text).map_err(|e| format!("{}: {}", PATH, e)),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Ok(Settings::default()),
            _ => Err(format!("{}: {}", PATH, e)),
        },
    }
}

pub fn to_text(settings: &Settings) -> String {
    let mut result = String::new();
    if let Some(timeout) = &settings.timeout {
        result += &format!("timeout: {}\n", timeout);
    };
    if let Some(default) = &settings.default {
        result += &format!("default: {}\n", default);
    }
    result
}

#[cfg(test)]

mod tests {
    use crate::settings::{Settings, apply, parse, to_text};
    use config::Config;
    use config::Entry;
    use config::Protocol;

    #[test]

    pub fn empty_str() {
        let result = parse("");
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.timeout, None);
        assert_eq!(settings.default, None);
    }

    #[test]
    pub fn correct_settings() {
        let str: &str = "timeout: 5\ndefault: Arch Linux (linux-lts)";
        let result = parse(str);
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.timeout, Some(5));
        assert_eq!(settings.default.as_deref(), Some("Arch Linux (linux-lts)"));
    }
    #[test]

    pub fn complex_name() {
        let str: &str = "default: Arch Linux (UKI: arch)";
        let result = parse(str);
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.default.as_deref(), Some("Arch Linux (UKI: arch)"));
    }

    #[test]

    pub fn str_ignore() {
        let str: &str = "#ingore\n      \n";
        let result = parse(str);
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.timeout, None);
        assert_eq!(settings.default, None);
    }

    #[test]

    pub fn invalid_timeout() {
        let str: &str = "timeout: five";
        let result = parse(str);
        assert!(result.is_err());
    }

    #[test]

    pub fn invalid_key() {
        let str: &str = "colour: red";
        let result = parse(str);
        assert!(result.is_err());
    }

    #[test]

    pub fn missing_del() {
        let str: &str = "timeout 5";
        let result = parse(str);
        assert!(result.is_err());
    }

    fn fake_config() -> Config {
        Config {
            timeout: 3,
            default: 0,
            entries: vec![
                Entry {
                    name: "Arch Linux (linux)".to_string(),
                    protocol: Protocol::Linux,
                    kernel: Some(r"\vmlinuz-linux".to_string()),
                    initrd: Some(r"\initramfs-linux.img".to_string()),
                    cmdline: None,
                    path: None,
                },
                Entry {
                    name: "Arch Linux (linux-lts)".to_string(),
                    protocol: Protocol::Linux,
                    kernel: Some(r"\vmlinuz-linux-lts".to_string()),
                    initrd: Some(r"\initramfs-linux-lts.img".to_string()),
                    cmdline: None,
                    path: None,
                },
            ],
        }
    }
    #[test]

    pub fn settings_default() {
        let mut config = fake_config();
        assert_eq!(apply(&mut config, &Settings::default()), None);
        assert_eq!(config.timeout, 3);
        assert_eq!(config.default, 0);
    }

    #[test]

    pub fn settings_set_timeout() {
        let mut config = fake_config();
        let settings = Settings {
            timeout: Some(10),
            default: None,
        };
        assert_eq!(apply(&mut config, &settings), None);
        assert_eq!(config.timeout, 10);
    }

    #[test]

    pub fn settings_set_default() {
        let mut config = fake_config();
        let settings = Settings {
            timeout: None,
            default: Some("Arch Linux (linux-lts)".to_string()),
        };
        assert_eq!(apply(&mut config, &settings), None);
        assert_eq!(config.default, 1);
    }

    #[test]
    pub fn settings_invalid_name() {
        let mut config = fake_config();
        let settings = Settings {
            timeout: None,
            default: Some("not exists".to_string()),
        };
        assert!(apply(&mut config, &settings).is_some());
    }

    #[test]

    pub fn circle_test() {
        let settings = Settings {
            timeout: Some(3),
            default: Some("Arch Linux (linux-lts)".to_string()),
        };
        let parse_result = parse(&to_text(&settings));
        assert!(parse_result.is_ok());
        let s = parse_result.unwrap();
        assert_eq!(s.default, settings.default);
        assert_eq!(s.timeout, settings.timeout);
    }
}
