use crate::conf::Conf;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key {
    pub code: KeyCode,
    pub mods: KeyModifiers,
}

impl Key {
    pub const ENTER: Key = Key { code: KeyCode::Enter, mods: KeyModifiers::NONE };
    pub const ESC: Key = Key { code: KeyCode::Esc, mods: KeyModifiers::NONE };
    pub const TAB: Key = Key { code: KeyCode::Tab, mods: KeyModifiers::NONE };
    pub const BACKTAB: Key = Key { code: KeyCode::BackTab, mods: KeyModifiers::SHIFT };
    pub const UP: Key = Key { code: KeyCode::Up, mods: KeyModifiers::NONE };
    pub const DOWN: Key = Key { code: KeyCode::Down, mods: KeyModifiers::NONE };
    pub const LEFT: Key = Key { code: KeyCode::Left, mods: KeyModifiers::NONE };
    pub const RIGHT: Key = Key { code: KeyCode::Right, mods: KeyModifiers::NONE };
    pub const HOME: Key = Key { code: KeyCode::Home, mods: KeyModifiers::NONE };
    pub const END: Key = Key { code: KeyCode::End, mods: KeyModifiers::NONE };
    pub const PAGE_UP: Key = Key { code: KeyCode::PageUp, mods: KeyModifiers::NONE };
    pub const PAGE_DOWN: Key = Key { code: KeyCode::PageDown, mods: KeyModifiers::NONE };
}

pub fn parse_key_spec(s: &str) -> Option<Key> {
    let t = s.trim();
    if t.is_empty() || t == "KEY_NONE" {
        return None;
    }

    // Ctrl-[ is Escape
    if t == "KEY_CTRL[" {
        return Some(Key::ESC);
    }

    if let Some(letter) = t.strip_prefix("KEY_CTRL") {
        if letter.len() == 1 {
            let c = letter.chars().next().unwrap().to_ascii_lowercase();
            return Some(Key { code: KeyCode::Char(c), mods: KeyModifiers::CONTROL });
        }
    }

    match t {
        "KEY_RETURN" | "KEY_ENTER" => Some(Key::ENTER),
        "KEY_ESC" => Some(Key::ESC),
        "KEY_TAB" => Some(Key::TAB),
        "KEY_BTAB" => Some(Key::BACKTAB),
        "KEY_SPACE" => Some(Key { code: KeyCode::Char(' '), mods: KeyModifiers::NONE }),
        "KEY_UP" => Some(Key::UP),
        "KEY_DOWN" => Some(Key::DOWN),
        "KEY_LEFT" => Some(Key::LEFT),
        "KEY_RIGHT" => Some(Key::RIGHT),
        "KEY_HOME" => Some(Key::HOME),
        "KEY_END" => Some(Key::END),
        "KEY_PPAGE" => Some(Key::PAGE_UP),
        "KEY_NPAGE" => Some(Key::PAGE_DOWN),
        "KEY_BACKSPACE" => Some(Key { code: KeyCode::Backspace, mods: KeyModifiers::NONE }),
        "KEY_DC" => Some(Key { code: KeyCode::Delete, mods: KeyModifiers::NONE }),
        "KEY_SLEFT" => Some(Key { code: KeyCode::Left, mods: KeyModifiers::SHIFT }),
        "KEY_SRIGHT" => Some(Key { code: KeyCode::Right, mods: KeyModifiers::SHIFT }),
        _ => {
            // Octal sequences like \33\142
            if t.starts_with('\\') {
                let parts: Vec<&str> = t.split('\\').filter(|p| !p.is_empty()).collect();
                if parts.len() == 2 && parts[0] == "33" {
                    // ESC + char
                    if let Ok(byte_val) = u8::from_str_radix(parts[1], 8) {
                        return Some(Key {
                            code: KeyCode::Char(byte_val as char),
                            mods: KeyModifiers::ALT,
                        });
                    }
                }
            }
            if t.len() == 1 {
                return Some(Key {
                    code: KeyCode::Char(t.chars().next().unwrap()),
                    mods: KeyModifiers::NONE,
                });
            }
            None
        }
    }
}

pub fn normalize_crossterm_key(ev: KeyEvent) -> Option<Key> {
    if ev.kind == KeyEventKind::Release {
        return None;
    }
    let mut mods = ev.modifiers;
    let code = match ev.code {
        KeyCode::Char(c) => {
            if mods.contains(KeyModifiers::CONTROL) {
                KeyCode::Char(c.to_ascii_lowercase())
            } else if mods.contains(KeyModifiers::SHIFT) && c.is_ascii_uppercase() {
                // Keep shift or absorb into uppercase char
                mods.remove(KeyModifiers::SHIFT);
                KeyCode::Char(c)
            } else {
                KeyCode::Char(c)
            }
        }
        KeyCode::BackTab => {
            mods.insert(KeyModifiers::SHIFT);
            KeyCode::BackTab
        }
        other => other,
    };
    Some(Key { code, mods })
}

#[derive(Debug, Clone)]
pub struct KeyConfig {
    bindings: HashMap<String, Key>,
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self::from_conf(&Conf::default())
    }
}

impl KeyConfig {
    pub fn from_conf(conf: &Conf) -> Self {
        let mut bindings = HashMap::new();
        let defaults = [
            ("cancel", "KEY_ESC"),
            ("quit", "KEY_CTRLQ"),
            ("left", "KEY_LEFT"),
            ("right", "KEY_RIGHT"),
            ("ok", "KEY_RETURN"),
            ("prev_page", "KEY_PPAGE"),
            ("next_page", "KEY_NPAGE"),
            ("down", "KEY_DOWN"),
            ("up", "KEY_UP"),
            ("end", "KEY_END"),
            ("home", "KEY_HOME"),
            ("toggle_list", "KEY_CTRLL"),
            ("folder_next", "KEY_SRIGHT"),
            ("folder_prev", "KEY_SLEFT"),
            ("focus_next", "KEY_TAB"),
            ("focus_prev", "KEY_BTAB"),
            ("focus_entry", "KEY_CTRL["),
            ("pane_prefix", "KEY_CTRLW"),
            ("transfer", "KEY_CTRLT"),
            ("select_emoji", "KEY_CTRLS"),
            ("select_sticker", "\\33\\147"),
            ("select_contact", "\\33\\156"),
            ("select_mention", "\\33\\62"),
            ("unread_chat", "KEY_CTRLF"),
            ("goto_chat", "KEY_CTRLN"),
            ("other_commands_help", "KEY_CTRLO"),
        ];

        for (action, def) in defaults {
            let spec = conf.str_or(action, def);
            if let Some(key) = parse_key_spec(spec) {
                bindings.insert(action.to_string(), key);
            }
        }
        KeyConfig { bindings }
    }

    pub fn get(&self, action: &str) -> Option<Key> {
        self.bindings.get(action).copied()
    }

    pub fn matches(&self, action: &str, key: Key) -> bool {
        self.get(action) == Some(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_key_names() {
        assert_eq!(parse_key_spec("KEY_RETURN"), Some(Key { code: KeyCode::Enter, mods: KeyModifiers::NONE }));
        assert_eq!(parse_key_spec("KEY_ESC"), Some(Key { code: KeyCode::Esc, mods: KeyModifiers::NONE }));
        assert_eq!(parse_key_spec("KEY_TAB"), Some(Key { code: KeyCode::Tab, mods: KeyModifiers::NONE }));
        assert_eq!(parse_key_spec("KEY_BTAB"), Some(Key { code: KeyCode::BackTab, mods: KeyModifiers::SHIFT }));
    }

    #[test]
    fn parses_ctrl_and_octal_sequences() {
        assert_eq!(parse_key_spec("KEY_CTRLC"), Some(Key { code: KeyCode::Char('c'), mods: KeyModifiers::CONTROL }));
        assert_eq!(parse_key_spec("KEY_CTRLQ"), Some(Key { code: KeyCode::Char('q'), mods: KeyModifiers::CONTROL }));
        assert_eq!(parse_key_spec("KEY_CTRL["), Some(Key { code: KeyCode::Esc, mods: KeyModifiers::NONE }));
        // Octal escape \33\142 = ESC + 'b' (Alt+b)
        assert_eq!(parse_key_spec(r"\33\142"), Some(Key { code: KeyCode::Char('b'), mods: KeyModifiers::ALT }));
    }

    #[test]
    fn normalizes_crossterm_event() {
        let press = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(normalize_crossterm_key(press), Some(Key { code: KeyCode::Char('a'), mods: KeyModifiers::NONE }));

        let release = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: crossterm::event::KeyEventState::NONE,
        };
        assert_eq!(normalize_crossterm_key(release), None);
    }

    #[test]
    fn default_key_config_matches_nchat_standard_shortcuts() {
        let keys = KeyConfig::default();
        let ctrl_t = Key { code: KeyCode::Char('t'), mods: KeyModifiers::CONTROL };
        let ctrl_s = Key { code: KeyCode::Char('s'), mods: KeyModifiers::CONTROL };
        let alt_g = Key { code: KeyCode::Char('g'), mods: KeyModifiers::ALT };
        let ctrl_q = Key { code: KeyCode::Char('q'), mods: KeyModifiers::CONTROL };

        assert!(keys.matches("transfer", ctrl_t));
        assert!(keys.matches("select_emoji", ctrl_s));
        assert!(keys.matches("select_sticker", alt_g));
        assert!(keys.matches("quit", ctrl_q));
    }

    #[test]
    fn maps_folder_next_and_prev() {
        let keys = KeyConfig::default();
        let s_right = Key { code: KeyCode::Right, mods: KeyModifiers::SHIFT };
        let s_left = Key { code: KeyCode::Left, mods: KeyModifiers::SHIFT };
        assert!(keys.matches("folder_next", s_right));
        assert!(keys.matches("folder_prev", s_left));
    }
}
