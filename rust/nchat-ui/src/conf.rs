use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Conf {
    entries: HashMap<String, String>,
}

impl Conf {
    pub fn parse(text: &str) -> Self {
        let mut entries = HashMap::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = trimmed.split_once('=') {
                entries.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        Conf { entries }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|s| s.as_str())
    }

    pub fn str_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).unwrap_or(default)
    }

    pub fn bool_or(&self, key: &str, default: bool) -> bool {
        match self.get(key) {
            Some("1") | Some("true") | Some("yes") => true,
            Some("0") | Some("false") | Some("no") => false,
            _ => default,
        }
    }

    pub fn num_or<T: std::str::FromStr>(&self, key: &str, default: T) -> T {
        self.get(key).and_then(|v| v.parse::<T>().ok()).unwrap_or(default)
    }
}

pub fn default_confdir_from(home: Option<&Path>, xdg: Option<&Path>, legacy_exists: bool) -> PathBuf {
    if legacy_exists {
        if let Some(h) = home {
            return h.join(".nchat");
        }
    }
    if let Some(x) = xdg {
        return x.join("nchat");
    }
    if let Some(h) = home {
        return h.join(".config").join("nchat");
    }
    PathBuf::from(".nchat")
}

pub fn resolve_confdir(override_dir: &str) -> PathBuf {
    if !override_dir.is_empty() {
        return PathBuf::from(override_dir);
    }
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let xdg = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from);
    let legacy = home.as_ref().map(|h| h.join(".nchat").is_dir()).unwrap_or(false);
    default_confdir_from(home.as_deref(), xdg.as_deref(), legacy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_key_value_pairs() {
        let src = "
        # Comment
        list_width = 32
        mouse_enabled=1
        layout_mode = sidebar
        empty_key=
        ";
        let c = Conf::parse(src);
        assert_eq!(c.num_or("list_width", 30), 32);
        assert_eq!(c.bool_or("mouse_enabled", false), true);
        assert_eq!(c.str_or("layout_mode", "classic"), "sidebar");
        assert_eq!(c.str_or("non_existent", "def"), "def");
    }

    #[test]
    fn resolves_default_confdir() {
        let home = Path::new("/home/user");
        let xdg = Path::new("/home/user/.custom_config");

        assert_eq!(
            default_confdir_from(Some(home), None, true),
            PathBuf::from("/home/user/.nchat")
        );
        assert_eq!(
            default_confdir_from(Some(home), Some(xdg), false),
            PathBuf::from("/home/user/.custom_config/nchat")
        );
        assert_eq!(
            default_confdir_from(Some(home), None, false),
            PathBuf::from("/home/user/.config/nchat")
        );
    }
}
