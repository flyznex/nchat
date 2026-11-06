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
    ScreenHigh,
    ScreenMiddle,
    ScreenLow,
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
    last_pending_time: Option<std::time::Instant>,
    pub vim_keys: bool,
}

impl NavResolver {
    pub fn new(vim_keys: bool) -> Self {
        let mut r = Self {
            bindings: Vec::new(),
            pending: Vec::new(),
            last_pending_time: None,
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
        self.add(vec![Self::k('H')], list, Action::ScreenHigh, true);
        self.add(vec![Self::k('M')], list, Action::ScreenMiddle, true);
        self.add(vec![Self::k('L')], list, Action::ScreenLow, true);
        self.add(vec![Self::k('o')], list, Action::Open, true);

        // Page navigation & scrolling
        self.add(vec![Self::ctrl('f')], list_hist, Action::HalfDown, true);
        self.add(vec![Self::ctrl('b')], list_hist, Action::HalfUp, true);
        self.add(vec![Self::ctrl('e')], list_hist, Action::Down, true);
        self.add(vec![Self::ctrl('y')], list_hist, Action::Up, true);

        // History actions
        let hist = Focus::History as u8;
        self.add(vec![Self::k('r')], hist, Action::Reply, true);
        self.add(vec![Self::k('e')], hist, Action::Edit, true);
        self.add(vec![Self::k('y')], hist, Action::Copy, true);
        self.add(vec![Self::k('x')], hist, Action::Delete, true);

        // Ctrl-W window navigation
        // FocusLeft: h, Ctrl-H, Backspace (often sent for Ctrl-H), Left
        self.add(vec![ctrl_w, Self::k('h')], all, Action::FocusLeft, true);
        self.add(vec![ctrl_w, Self::ctrl('h')], all, Action::FocusLeft, true);
        self.add(vec![ctrl_w, Key { code: KeyCode::Backspace, mods: KeyModifiers::NONE }], all, Action::FocusLeft, true);
        self.add(vec![ctrl_w, Key { code: KeyCode::Backspace, mods: KeyModifiers::CONTROL }], all, Action::FocusLeft, true);
        self.add(vec![ctrl_w, Key::LEFT], all, Action::FocusLeft, true);

        // FocusRight: l, Ctrl-L, Right
        self.add(vec![ctrl_w, Self::k('l')], all, Action::FocusRight, true);
        self.add(vec![ctrl_w, Self::ctrl('l')], all, Action::FocusRight, true);
        self.add(vec![ctrl_w, Key::RIGHT], all, Action::FocusRight, true);

        // FocusEntry (Down): j, Ctrl-J, Down
        self.add(vec![ctrl_w, Self::k('j')], list_hist, Action::FocusEntry, true);
        self.add(vec![ctrl_w, Self::ctrl('j')], list_hist, Action::FocusEntry, true);
        self.add(vec![ctrl_w, Key::DOWN], list_hist, Action::FocusEntry, true);

        // FocusHistory (Up): k, Ctrl-K, Up
        self.add(vec![ctrl_w, Self::k('k')], Focus::Entry as u8, Action::FocusHistory, true);
        self.add(vec![ctrl_w, Self::ctrl('k')], Focus::Entry as u8, Action::FocusHistory, true);
        self.add(vec![ctrl_w, Key::UP], Focus::Entry as u8, Action::FocusHistory, true);

        // FocusNext: w, Ctrl-W
        self.add(vec![ctrl_w, Self::k('w')], all, Action::FocusNext, true);
        self.add(vec![ctrl_w, Self::ctrl('w')], all, Action::FocusNext, true);
        self.add(vec![ctrl_w, ctrl_w], all, Action::FocusNext, true);
    }

    fn add(&mut self, keys: Vec<Key>, mask: u8, action: Action, vim_only: bool) {
        self.bindings.push(Binding { keys, mask, action, vim_only });
    }

    /// True while a multi-key sequence (e.g. Ctrl-W pane prefix) is waiting
    /// on its next key. Callers must route the next keystroke to `resolve`
    /// instead of their own focus-specific handling (e.g. typing into the
    /// entry buffer), or the chord can never complete.
    pub fn has_pending(&self) -> bool {
        if let Some(t) = self.last_pending_time {
            if t.elapsed() > std::time::Duration::from_millis(1000) {
                return false;
            }
        }
        !self.pending.is_empty()
    }

    pub fn resolve(&mut self, key: Key, focus: Focus) -> NavResolveResult {
        // Tab / BackTab ALWAYS cycles focus, regardless of active pane
        if key == Key::TAB {
            self.pending.clear();
            self.last_pending_time = None;
            return NavResolveResult::Matched(Action::FocusNext);
        }
        if key == Key::BACKTAB {
            self.pending.clear();
            self.last_pending_time = None;
            return NavResolveResult::Matched(Action::FocusPrev);
        }

        // Auto-cancel stale pending chord if > 1000ms has passed
        if let Some(t) = self.last_pending_time {
            if t.elapsed() > std::time::Duration::from_millis(1000) {
                self.pending.clear();
            }
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
                self.last_pending_time = None;
                return NavResolveResult::Matched(b.action);
            }
            if b.keys.starts_with(&self.pending) {
                has_prefix = true;
            }
        }

        if has_prefix {
            self.last_pending_time = Some(std::time::Instant::now());
            return NavResolveResult::Pending;
        }

        let was_ctrl_w = !self.pending.is_empty() && self.pending[0] == Self::ctrl('w');

        // Check single key fallback if multi-key pending failed
        if self.pending.len() > 1 {
            self.pending.clear();
            self.last_pending_time = None;
            if was_ctrl_w {
                return NavResolveResult::Matched(Action::None);
            }
            return self.resolve(key, focus);
        }

        self.pending.clear();
        self.last_pending_time = None;

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
    fn resolves_ctrl_w_with_ctrl_held_down_and_arrows() {
        let mut r = NavResolver::new(true);
        let ctrl_w = Key { code: KeyCode::Char('w'), mods: KeyModifiers::CONTROL };
        let ctrl_h = Key { code: KeyCode::Char('h'), mods: KeyModifiers::CONTROL };
        let ctrl_l = Key { code: KeyCode::Char('l'), mods: KeyModifiers::CONTROL };
        let ctrl_j = Key { code: KeyCode::Char('j'), mods: KeyModifiers::CONTROL };
        let ctrl_k = Key { code: KeyCode::Char('k'), mods: KeyModifiers::CONTROL };
        let backspace = Key { code: KeyCode::Backspace, mods: KeyModifiers::NONE };

        // Ctrl-W Ctrl-H
        assert_eq!(r.resolve(ctrl_w, Focus::History), NavResolveResult::Pending);
        assert_eq!(r.resolve(ctrl_h, Focus::History), NavResolveResult::Matched(Action::FocusLeft));

        // Ctrl-W Backspace (terminal representation of Ctrl-H)
        assert_eq!(r.resolve(ctrl_w, Focus::History), NavResolveResult::Pending);
        assert_eq!(r.resolve(backspace, Focus::History), NavResolveResult::Matched(Action::FocusLeft));

        // Ctrl-W Ctrl-L
        assert_eq!(r.resolve(ctrl_w, Focus::List), NavResolveResult::Pending);
        assert_eq!(r.resolve(ctrl_l, Focus::List), NavResolveResult::Matched(Action::FocusRight));

        // Ctrl-W Ctrl-J
        assert_eq!(r.resolve(ctrl_w, Focus::History), NavResolveResult::Pending);
        assert_eq!(r.resolve(ctrl_j, Focus::History), NavResolveResult::Matched(Action::FocusEntry));

        // Ctrl-W Ctrl-K
        assert_eq!(r.resolve(ctrl_w, Focus::Entry), NavResolveResult::Pending);
        assert_eq!(r.resolve(ctrl_k, Focus::Entry), NavResolveResult::Matched(Action::FocusHistory));

        // Arrow keys following Ctrl-W
        assert_eq!(r.resolve(ctrl_w, Focus::Entry), NavResolveResult::Pending);
        assert_eq!(r.resolve(Key::LEFT, Focus::Entry), NavResolveResult::Matched(Action::FocusLeft));

        assert_eq!(r.resolve(ctrl_w, Focus::Entry), NavResolveResult::Pending);
        assert_eq!(r.resolve(Key::UP, Focus::Entry), NavResolveResult::Matched(Action::FocusHistory));

        assert_eq!(r.resolve(ctrl_w, Focus::List), NavResolveResult::Pending);
        assert_eq!(r.resolve(Key::RIGHT, Focus::List), NavResolveResult::Matched(Action::FocusRight));

        assert_eq!(r.resolve(ctrl_w, Focus::List), NavResolveResult::Pending);
        assert_eq!(r.resolve(Key::DOWN, Focus::List), NavResolveResult::Matched(Action::FocusEntry));
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
    fn failed_ctrl_w_is_swallowed() {
        let mut r = NavResolver::new(true);
        let ctrl_w = Key { code: KeyCode::Char('w'), mods: KeyModifiers::CONTROL };
        assert_eq!(r.resolve(ctrl_w, Focus::List), NavResolveResult::Pending);
        assert_eq!(r.resolve(key('1'), Focus::List), NavResolveResult::Matched(Action::None));
        assert!(!r.has_pending());
    }

    #[test]
    fn unmapped_char_returns_ignored_in_list_and_history() {
        let mut r = NavResolver::new(true);
        assert_eq!(r.resolve(key('a'), Focus::List), NavResolveResult::Ignored);
        assert_eq!(r.resolve(key('m'), Focus::List), NavResolveResult::Ignored);
        assert_eq!(r.resolve(key('x'), Focus::List), NavResolveResult::Ignored);

        assert_eq!(r.resolve(key('a'), Focus::History), NavResolveResult::Ignored);
        assert_eq!(r.resolve(key('m'), Focus::History), NavResolveResult::Ignored);
        assert_eq!(r.resolve(key('x'), Focus::History), NavResolveResult::Matched(Action::Delete));
    }

    #[test]
    fn resolves_screen_and_scrolling_vim_actions_in_list() {
        let mut r = NavResolver::new(true);
        assert_eq!(r.resolve(key('H'), Focus::List), NavResolveResult::Matched(Action::ScreenHigh));
        assert_eq!(r.resolve(key('M'), Focus::List), NavResolveResult::Matched(Action::ScreenMiddle));
        assert_eq!(r.resolve(key('L'), Focus::List), NavResolveResult::Matched(Action::ScreenLow));
        assert_eq!(r.resolve(key('o'), Focus::List), NavResolveResult::Matched(Action::Open));

        let ctrl_f = Key { code: KeyCode::Char('f'), mods: KeyModifiers::CONTROL };
        let ctrl_b = Key { code: KeyCode::Char('b'), mods: KeyModifiers::CONTROL };
        let ctrl_e = Key { code: KeyCode::Char('e'), mods: KeyModifiers::CONTROL };
        let ctrl_y = Key { code: KeyCode::Char('y'), mods: KeyModifiers::CONTROL };

        assert_eq!(r.resolve(ctrl_f, Focus::List), NavResolveResult::Matched(Action::HalfDown));
        assert_eq!(r.resolve(ctrl_b, Focus::List), NavResolveResult::Matched(Action::HalfUp));
        assert_eq!(r.resolve(ctrl_e, Focus::List), NavResolveResult::Matched(Action::Down));
        assert_eq!(r.resolve(ctrl_y, Focus::List), NavResolveResult::Matched(Action::Up));

        let ctrl_w = Key { code: KeyCode::Char('w'), mods: KeyModifiers::CONTROL };
        // Ctrl-w j moves from List to Entry
        assert_eq!(r.resolve(ctrl_w, Focus::List), NavResolveResult::Pending);
        assert_eq!(r.resolve(key('j'), Focus::List), NavResolveResult::Matched(Action::FocusEntry));

        // Ctrl-w w cycles window
        assert_eq!(r.resolve(ctrl_w, Focus::List), NavResolveResult::Pending);
        assert_eq!(r.resolve(key('w'), Focus::List), NavResolveResult::Matched(Action::FocusNext));
    }

    #[test]
    fn standard_fallbacks_when_vim_disabled() {
        let mut r = NavResolver::new(false);
        assert_eq!(r.resolve(key('j'), Focus::List), NavResolveResult::Ignored);
        assert_eq!(r.resolve(Key::DOWN, Focus::List), NavResolveResult::Matched(Action::Down));
    }
}
