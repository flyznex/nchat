use crate::keyconf::Key;
use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    List = 1,
    History = 2,
    Entry = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    None,
    Down,
    Up,
    First,
    Last,
    HalfDown,
    HalfUp,
    FocusEntry,
    FocusList,
    FocusHistory,
    FocusLeft,
    FocusRight,
    FocusNext,
    FocusPrev,
    Open,
    Reply,
    Edit,
    Copy,
    Delete,
    React,
    Search,
    UnreadOnly,
    ToggleGroup,
    FolderNext,
    FolderPrev,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavResolveResult {
    Matched(Action),
    Pending,
    Ignored,
}

#[derive(Clone)]
struct Binding {
    keys: Vec<Key>,
    mask: u8,
    action: Action,
    vim_only: bool,
}

pub struct NavResolver {
    bindings: Vec<Binding>,
    pending: Vec<Key>,
    pub vim_keys: bool,
}

impl NavResolver {
    pub fn new(vim_keys: bool) -> Self {
        let mut r = Self {
            bindings: Vec::new(),
            pending: Vec::new(),
            vim_keys,
        };
        r.load_defaults();
        r
    }

    fn k(c: char) -> Key {
        Key { code: KeyCode::Char(c), mods: KeyModifiers::NONE }
    }

    fn ctrl(c: char) -> Key {
        Key { code: KeyCode::Char(c), mods: KeyModifiers::CONTROL }
    }

    fn load_defaults(&mut self) {
        let list_hist = Focus::List as u8 | Focus::History as u8;
        let list = Focus::List as u8;
        let all = Focus::List as u8 | Focus::History as u8 | Focus::Entry as u8;
        let ctrl_w = Self::ctrl('w');

        // Defaults ported from lib/ncutil/src/navkeys.cpp
        self.add(vec![Self::k('j')], list_hist, Action::Down, true);
        self.add(vec![Self::k('k')], list_hist, Action::Up, true);
        self.add(vec![Self::k('G')], list_hist, Action::Last, true);
        self.add(vec![Self::ctrl('d')], list_hist, Action::HalfDown, true);
        self.add(vec![Self::ctrl('u')], list_hist, Action::HalfUp, true);
        self.add(vec![Self::k('i')], list_hist, Action::FocusEntry, true);
        self.add(vec![Self::k('/')], list_hist, Action::Search, true);
        self.add(vec![Self::k('g'), Self::k('g')], list_hist, Action::First, true);
        self.add(vec![Self::k('h')], list_hist, Action::FocusLeft, true);
        self.add(vec![Self::k('l')], list_hist, Action::FocusRight, true);

        self.add(vec![Self::k('n')], list, Action::FolderNext, true);
        self.add(vec![Self::k('b')], list, Action::FolderPrev, true);
        self.add(vec![Self::k('u')], list, Action::UnreadOnly, true);
        self.add(vec![Self::k('z')], list, Action::ToggleGroup, true);

        // History actions
        let hist = Focus::History as u8;
        self.add(vec![Self::k('r')], hist, Action::Reply, true);
        self.add(vec![Self::k('e')], hist, Action::Edit, true);
        self.add(vec![Self::k('y')], hist, Action::Copy, true);
        self.add(vec![Self::k('x')], hist, Action::Delete, true);

        // Ctrl-W window navigation
        self.add(vec![ctrl_w, Self::k('h')], all, Action::FocusLeft, true);
        self.add(vec![ctrl_w, Self::k('l')], all, Action::FocusRight, true);
        self.add(vec![ctrl_w, Self::k('j')], Focus::History as u8, Action::FocusEntry, true);
        self.add(vec![ctrl_w, Self::k('k')], Focus::Entry as u8, Action::FocusHistory, true);
    }

    fn add(&mut self, keys: Vec<Key>, mask: u8, action: Action, vim_only: bool) {
        self.bindings.push(Binding { keys, mask, action, vim_only });
    }

    /// True while a multi-key sequence (e.g. Ctrl-W pane prefix) is waiting
    /// on its next key. Callers must route the next keystroke to `resolve`
    /// instead of their own focus-specific handling (e.g. typing into the
    /// entry buffer), or the chord can never complete.
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    pub fn resolve(&mut self, key: Key, focus: Focus) -> NavResolveResult {
        // Tab / BackTab ALWAYS cycles focus, regardless of active pane
        if key == Key::TAB {
            self.pending.clear();
            return NavResolveResult::Matched(Action::FocusNext);
        }
        if key == Key::BACKTAB {
            self.pending.clear();
            return NavResolveResult::Matched(Action::FocusPrev);
        }

        self.pending.push(key);
        let mask = focus as u8;

        let mut has_prefix = false;
        for b in &self.bindings {
            if b.vim_only && !self.vim_keys {
                continue;
            }
            if (b.mask & mask) == 0 {
                continue;
            }

            if b.keys == self.pending {
                self.pending.clear();
                return NavResolveResult::Matched(b.action);
            }
            if b.keys.starts_with(&self.pending) {
                has_prefix = true;
            }
        }

        if has_prefix {
            return NavResolveResult::Pending;
        }

        // Check single key fallback if multi-key pending failed
        if self.pending.len() > 1 {
            self.pending.clear();
            return self.resolve(key, focus);
        }

        self.pending.clear();

        if focus == Focus::Entry {
            return NavResolveResult::Ignored;
        }

        // Standard navigation fallbacks
        match key {
            Key::DOWN => NavResolveResult::Matched(Action::Down),
            Key::UP => NavResolveResult::Matched(Action::Up),
            Key::HOME => NavResolveResult::Matched(Action::First),
            Key::END => NavResolveResult::Matched(Action::Last),
            Key::PAGE_DOWN => NavResolveResult::Matched(Action::HalfDown),
            Key::PAGE_UP => NavResolveResult::Matched(Action::HalfUp),
            Key::ENTER => match focus {
                Focus::List => NavResolveResult::Matched(Action::Open),
                Focus::History => NavResolveResult::Matched(Action::Reply),
                Focus::Entry => NavResolveResult::Ignored,
            },
            _ => NavResolveResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(c: char) -> Key {
        Key { code: KeyCode::Char(c), mods: KeyModifiers::NONE }
    }

    #[test]
    fn resolves_single_key_vim_actions_in_list() {
        let mut r = NavResolver::new(true);
        assert_eq!(r.resolve(key('j'), Focus::List), NavResolveResult::Matched(Action::Down));
        assert_eq!(r.resolve(key('k'), Focus::List), NavResolveResult::Matched(Action::Up));
        assert_eq!(r.resolve(key('G'), Focus::List), NavResolveResult::Matched(Action::Last));
    }

    #[test]
    fn resolves_multi_key_sequence_gg() {
        let mut r = NavResolver::new(true);
        assert_eq!(r.resolve(key('g'), Focus::List), NavResolveResult::Pending);
        assert_eq!(r.resolve(key('g'), Focus::List), NavResolveResult::Matched(Action::First));
    }

    #[test]
    fn resolves_ctrl_w_pane_prefix() {
        let mut r = NavResolver::new(true);
        let ctrl_w = Key { code: KeyCode::Char('w'), mods: KeyModifiers::CONTROL };
        assert_eq!(r.resolve(ctrl_w, Focus::List), NavResolveResult::Pending);
        assert_eq!(r.resolve(key('l'), Focus::List), NavResolveResult::Matched(Action::FocusRight));
    }

    #[test]
    fn resolves_tab_and_ctrl_w_in_entry() {
        let mut r = NavResolver::new(true);
        assert_eq!(r.resolve(Key::TAB, Focus::Entry), NavResolveResult::Matched(Action::FocusNext));
        assert_eq!(r.resolve(Key::BACKTAB, Focus::Entry), NavResolveResult::Matched(Action::FocusPrev));

        let ctrl_w = Key { code: KeyCode::Char('w'), mods: KeyModifiers::CONTROL };
        assert_eq!(r.resolve(ctrl_w, Focus::Entry), NavResolveResult::Pending);
        assert_eq!(r.resolve(key('k'), Focus::Entry), NavResolveResult::Matched(Action::FocusHistory));
    }

    #[test]
    fn has_pending_tracks_an_open_ctrl_w_chord() {
        // app.rs relies on has_pending() to know when a Ctrl-W chord is
        // waiting on its next key, so it can route that key here instead of
        // typing it into the entry buffer. If this regresses, Ctrl-W pane
        // navigation silently breaks from Entry focus.
        let mut r = NavResolver::new(true);
        assert!(!r.has_pending());

        let ctrl_w = Key { code: KeyCode::Char('w'), mods: KeyModifiers::CONTROL };
        assert_eq!(r.resolve(ctrl_w, Focus::Entry), NavResolveResult::Pending);
        assert!(r.has_pending());

        assert_eq!(r.resolve(key('k'), Focus::Entry), NavResolveResult::Matched(Action::FocusHistory));
        assert!(!r.has_pending());
    }

    #[test]
    fn standard_fallbacks_when_vim_disabled() {
        let mut r = NavResolver::new(false);
        assert_eq!(r.resolve(key('j'), Focus::List), NavResolveResult::Ignored);
        assert_eq!(r.resolve(Key::DOWN, Focus::List), NavResolveResult::Matched(Action::Down));
    }
}
