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

#[cfg(test)]

mod tests {
    use crate::settings::parse;

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
}
