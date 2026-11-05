use super::{centered_rect, Dialog, DialogResult};
use crate::colorconf::Theme;
use crate::keyconf::Key;
use crate::text::split_row;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::Frame;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

pub struct FileDialog {
    pub current_dir: PathBuf,
    pub filter: String,
    pub entries: Vec<FileEntry>,
    pub selected_idx: usize,
    pub scroll_first: usize,
}

impl FileDialog {
    pub fn new(start_dir: Option<PathBuf>) -> Self {
        let dir = start_dir
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let abs_dir = normalize_path(&dir);
        let mut d = Self {
            current_dir: abs_dir,
            filter: String::new(),
            entries: Vec::new(),
            selected_idx: 0,
            scroll_first: 0,
        };
        d.refresh_entries();
        d
    }

    pub fn refresh_entries(&mut self) {
        self.entries.clear();
        self.selected_idx = 0;
        self.scroll_first = 0;

        if let Some(parent) = self.current_dir.parent() {
            self.entries.push(FileEntry {
                path: parent.to_path_buf(),
                name: "..".to_string(),
                is_dir: true,
                size: 0,
            });
        }

        self.entries.extend(read_dir_entries(&self.current_dir));
    }

    pub fn candidates(&self) -> Vec<FileEntry> {
        let trimmed = self.filter.trim();
        if is_path_input(trimmed) {
            let (dir_part, prefix) = match trimmed.rfind(|c| c == '/' || c == '\\') {
                Some(idx) => (&trimmed[..=idx], &trimmed[idx + 1..]),
                None => (trimmed, ""),
            };

            let target_dir = resolve_path(&self.current_dir, dir_part);
            if target_dir.is_dir() {
                let mut list = Vec::new();
                if let Some(parent) = target_dir.parent() {
                    if prefix.is_empty() || "..".starts_with(prefix) {
                        list.push(FileEntry {
                            path: parent.to_path_buf(),
                            name: "..".to_string(),
                            is_dir: true,
                            size: 0,
                        });
                    }
                }
                let mut entries = read_dir_entries(&target_dir);
                let q = prefix.to_lowercase();
                if !prefix.is_empty() {
                    entries.retain(|e| e.name.to_lowercase().starts_with(&q));
                } else {
                    entries.retain(|e| !e.name.starts_with('.') || e.name == "..");
                }
                list.extend(entries);
                list
            } else {
                let direct = resolve_path(&self.current_dir, trimmed);
                if direct.is_file() {
                    let name = direct.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                    let size = direct.metadata().map(|m| m.len()).unwrap_or(0);
                    vec![FileEntry {
                        path: direct,
                        name,
                        is_dir: false,
                        size,
                    }]
                } else {
                    Vec::new()
                }
            }
        } else if trimmed.is_empty() {
            self.entries.clone()
        } else {
            let q = trimmed.to_lowercase();
            self.entries
                .iter()
                .filter(|e| e.name.to_lowercase().contains(&q))
                .cloned()
                .collect()
        }
    }

    fn complete_tab(&mut self) {
        let trimmed = self.filter.trim().to_string();
        if trimmed.is_empty() {
            return;
        }

        if is_path_input(&trimmed) {
            let (dir_part, prefix) = match trimmed.rfind(|c| c == '/' || c == '\\') {
                Some(idx) => (&trimmed[..=idx], &trimmed[idx + 1..]),
                None => (trimmed.as_str(), ""),
            };

            let target_dir = resolve_path(&self.current_dir, dir_part);
            if target_dir.is_dir() {
                let entries = read_dir_entries(&target_dir);
                let q = prefix.to_lowercase();
                let matches: Vec<&FileEntry> = entries
                    .iter()
                    .filter(|e| e.name != ".." && e.name.to_lowercase().starts_with(&q))
                    .collect();

                if matches.len() == 1 {
                    let m = matches[0];
                    if m.is_dir {
                        self.filter = format!("{}{}/", dir_part, m.name);
                    } else {
                        self.filter = format!("{}{}", dir_part, m.name);
                    }
                    self.selected_idx = 0;
                } else if !matches.is_empty() {
                    let names: Vec<&str> = matches.iter().map(|e| e.name.as_str()).collect();
                    let common = longest_common_prefix(&names);
                    if common.len() > prefix.len() {
                        self.filter = format!("{}{}", dir_part, common);
                        self.selected_idx = 0;
                    }
                }
            }
        } else {
            let q = trimmed.to_lowercase();
            let matches: Vec<&FileEntry> = self
                .entries
                .iter()
                .filter(|e| e.name != ".." && e.name.to_lowercase().starts_with(&q))
                .collect();

            if matches.len() == 1 {
                let m = matches[0];
                if m.is_dir {
                    self.filter = format!("{}/", m.name);
                } else {
                    self.filter = m.name.clone();
                }
                self.selected_idx = 0;
            } else if !matches.is_empty() {
                let names: Vec<&str> = matches.iter().map(|e| e.name.as_str()).collect();
                let common = longest_common_prefix(&names);
                if common.len() > trimmed.len() {
                    self.filter = common;
                    self.selected_idx = 0;
                }
            }
        }
    }
}

pub fn is_path_input(s: &str) -> bool {
    let t = s.trim();
    t == "~"
        || t.starts_with("~/")
        || t.starts_with('/')
        || t.starts_with("./")
        || t.starts_with("../")
        || t.contains('/')
        || t.contains('\\')
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::Prefix(p) => out.push(p.as_os_str()),
            Component::RootDir => out.push("/"),
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            Component::Normal(n) => out.push(n),
        }
    }
    if out.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        out
    }
}

pub fn resolve_path(current_dir: &Path, input: &str) -> PathBuf {
    let s = input.trim();
    let raw = if s == "~" {
        std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("~"))
    } else if let Some(rest) = s.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join(rest)
        } else {
            PathBuf::from(s)
        }
    } else {
        let p = Path::new(s);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            current_dir.join(p)
        }
    };
    normalize_path(&raw)
}

pub fn format_file_size(size: u64) -> String {
    if size < 1024 {
        format!("{size} B")
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub fn longest_common_prefix(strings: &[&str]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    let first = strings[0];
    let mut len = first.len();
    for s in &strings[1..] {
        len = len.min(s.len());
        let mut match_len = 0;
        for (c1, c2) in first[..len].chars().zip(s.chars()) {
            if c1.to_lowercase().next() == c2.to_lowercase().next() {
                match_len += c1.len_utf8();
            } else {
                break;
            }
        }
        len = match_len;
    }
    first[..len].to_string()
}

pub fn read_dir_entries(dir: &Path) -> Vec<FileEntry> {
    let mut entries = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.filter_map(|e| e.ok()) {
            let path = e.path();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            let is_dir = path.is_dir();
            let size = e.metadata().map(|m| m.len()).unwrap_or(0);
            entries.push(FileEntry { path, name, is_dir, size });
        }
    }
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    entries
}

impl Dialog for FileDialog {
    fn handle_key(&mut self, key: Key, _model: &crate::model::UiModel) -> DialogResult {
        // Clipboard paste (Ctrl-V)
        if key.code == KeyCode::Char('v') && key.mods.contains(KeyModifiers::CONTROL) {
            if let Some(text) = crate::clipboard::get_text() {
                self.filter.push_str(text.trim());
                self.selected_idx = 0;
            }
            return DialogResult::KeepOpen;
        }

        let candidates = self.candidates();
        let count = candidates.len();

        match key.code {
            KeyCode::Esc => {
                if !self.filter.is_empty() {
                    self.filter.clear();
                    self.selected_idx = 0;
                    DialogResult::KeepOpen
                } else {
                    DialogResult::Close(false)
                }
            }
            KeyCode::Tab => {
                self.complete_tab();
                DialogResult::KeepOpen
            }
            KeyCode::Enter => {
                let trimmed = self.filter.trim();
                let direct = resolve_path(&self.current_dir, trimmed);
                if !trimmed.is_empty() && direct.is_file() {
                    return DialogResult::CloseValue(direct.to_string_lossy().to_string());
                }
                if is_path_input(trimmed) && direct.is_dir() {
                    self.current_dir = direct;
                    self.filter.clear();
                    self.refresh_entries();
                    return DialogResult::KeepOpen;
                }

                if let Some(entry) = candidates.get(self.selected_idx) {
                    if entry.is_dir {
                        self.current_dir = normalize_path(&entry.path);
                        self.filter.clear();
                        self.refresh_entries();
                        DialogResult::KeepOpen
                    } else {
                        DialogResult::CloseValue(entry.path.to_string_lossy().to_string())
                    }
                } else {
                    DialogResult::KeepOpen
                }
            }
            KeyCode::Backspace => {
                if !self.filter.is_empty() {
                    self.filter.pop();
                    self.selected_idx = 0;
                } else if let Some(parent) = self.current_dir.parent() {
                    self.current_dir = parent.to_path_buf();
                    self.refresh_entries();
                }
                DialogResult::KeepOpen
            }
            KeyCode::Left => {
                if self.filter.is_empty() {
                    if let Some(parent) = self.current_dir.parent() {
                        self.current_dir = parent.to_path_buf();
                        self.refresh_entries();
                    }
                } else {
                    self.filter.pop();
                    self.selected_idx = 0;
                }
                DialogResult::KeepOpen
            }
            KeyCode::Right => {
                if let Some(entry) = candidates.get(self.selected_idx) {
                    if entry.is_dir {
                        self.current_dir = normalize_path(&entry.path);
                        self.filter.clear();
                        self.refresh_entries();
                    } else {
                        return DialogResult::CloseValue(entry.path.to_string_lossy().to_string());
                    }
                }
                DialogResult::KeepOpen
            }
            KeyCode::Up | KeyCode::Char('p') if key.code == KeyCode::Up || key.mods.contains(KeyModifiers::CONTROL) => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                }
                DialogResult::KeepOpen
            }
            KeyCode::Down | KeyCode::Char('n') if key.code == KeyCode::Down || key.mods.contains(KeyModifiers::CONTROL) => {
                if self.selected_idx + 1 < count {
                    self.selected_idx += 1;
                }
                DialogResult::KeepOpen
            }
            KeyCode::PageUp => {
                self.selected_idx = self.selected_idx.saturating_sub(10);
                DialogResult::KeepOpen
            }
            KeyCode::PageDown => {
                self.selected_idx = (self.selected_idx + 10).min(count.saturating_sub(1));
                DialogResult::KeepOpen
            }
            KeyCode::Home => {
                self.selected_idx = 0;
                DialogResult::KeepOpen
            }
            KeyCode::End => {
                self.selected_idx = count.saturating_sub(1);
                DialogResult::KeepOpen
            }
            KeyCode::Char(c) if !key.mods.contains(KeyModifiers::CONTROL) && !key.mods.contains(KeyModifiers::ALT) => {
                self.filter.push(c);
                self.selected_idx = 0;
                DialogResult::KeepOpen
            }
            _ => DialogResult::KeepOpen,
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme, _img_cache: &mut crate::imgcache::ImageCache, _model: &crate::model::UiModel) {
        let popup_area = centered_rect(75, 70, area);
        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Send File [{}] ", self.current_dir.display()))
            .border_style(theme.focus_style);

        let inner = block.inner(popup_area);
        f.render_widget(block, popup_area);

        if inner.width < 4 || inner.height < 3 {
            return;
        }

        let inner_w = inner.width as usize;
        let mut lines = Vec::new();

        // Line 0: Search / Filter input
        let filter_display = if self.filter.is_empty() {
            Span::styled("Type filename or path (/ or ~), Tab to complete", Style::default().add_modifier(Modifier::DIM))
        } else {
            Span::styled(format!("{}_", self.filter), theme.focus_style)
        };
        lines.push(Line::from(vec![
            Span::styled(" Path/Filter: ", theme.list_name),
            filter_display,
        ]));

        // Line 1: Separator
        lines.push(Line::from(Span::styled("─".repeat(inner_w), theme.listborder)));

        let list_h = (inner.height as usize).saturating_sub(2);
        let candidates = self.candidates();

        if list_h > 0 {
            if self.selected_idx >= self.scroll_first + list_h {
                self.scroll_first = self.selected_idx + 1 - list_h;
            } else if self.selected_idx < self.scroll_first {
                self.scroll_first = self.selected_idx;
            }
        }

        if candidates.is_empty() {
            lines.push(Line::from("  No matching files or directories"));
        } else {
            for (idx, entry) in candidates.iter().enumerate().skip(self.scroll_first).take(list_h) {
                let is_sel = idx == self.selected_idx;
                let (prefix, right_part) = if entry.is_dir {
                    ("📁 ", "(dir)".to_string())
                } else {
                    ("📄 ", format_file_size(entry.size))
                };
                let left_name = format!("{prefix}{}", entry.name);
                let row_str = split_row(&left_name, &right_part, inner_w);

                let style = if is_sel {
                    theme.list_selected
                } else if entry.name.starts_with('.') && entry.name != ".." {
                    Style::default().add_modifier(Modifier::DIM)
                } else {
                    theme.list
                };

                lines.push(Line::from(Span::styled(row_str, style)));
            }
        }

        let p = Paragraph::new(lines);
        f.render_widget(p, inner);

        if candidates.len() > list_h {
            let list_area = Rect::new(inner.x, inner.y + 2, inner.width, list_h as u16);
            let mut state = ScrollbarState::new(candidates.len()).position(self.selected_idx);
            f.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight).begin_symbol(None).end_symbol(None),
                list_area,
                &mut state,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_dialog_lists_directory_and_handles_esc() {
        let mut d = FileDialog::new(Some(PathBuf::from("/tmp")));
        assert!(!d.entries.is_empty());
        let model = crate::model::UiModel::new();
        assert_eq!(d.handle_key(Key::ESC, &model), DialogResult::Close(false));
    }

    #[test]
    fn file_dialog_filters_by_filename() {
        let mut d = FileDialog::new(Some(PathBuf::from("/tmp")));
        let model = crate::model::UiModel::new();
        d.handle_key(Key { code: KeyCode::Char('z'), mods: KeyModifiers::NONE }, &model);
        d.handle_key(Key { code: KeyCode::Char('q'), mods: KeyModifiers::NONE }, &model);
        d.handle_key(Key { code: KeyCode::Char('x'), mods: KeyModifiers::NONE }, &model);
        assert_eq!(d.filter, "zqx");
    }

    #[test]
    fn resolves_relative_and_home_paths() {
        let cur = PathBuf::from("/tmp");
        let home = resolve_path(&cur, "~/myfile.txt");
        assert!(home.to_string_lossy().contains("myfile.txt"));

        let rel = resolve_path(&cur, "./abc/def.txt");
        assert_eq!(rel, PathBuf::from("/tmp/abc/def.txt"));

        let parent_rel = resolve_path(&cur, "../other");
        assert_eq!(parent_rel, PathBuf::from("/other"));
    }

    #[test]
    fn detects_path_input() {
        assert!(is_path_input("/var/log"));
        assert!(is_path_input("~/Downloads"));
        assert!(is_path_input("./foo"));
        assert!(is_path_input("../bar"));
        assert!(is_path_input("a/b"));
        assert!(!is_path_input("hello"));
    }

    #[test]
    fn tab_completion_completes_prefix() {
        let strings = vec!["download_file1.png", "download_file2.png"];
        assert_eq!(longest_common_prefix(&strings), "download_file");
    }

    #[test]
    fn esc_clears_filter_first_then_closes() {
        let mut d = FileDialog::new(Some(PathBuf::from("/tmp")));
        let model = crate::model::UiModel::new();
        d.filter = "something".into();
        assert_eq!(d.handle_key(Key::ESC, &model), DialogResult::KeepOpen);
        assert!(d.filter.is_empty());
        assert_eq!(d.handle_key(Key::ESC, &model), DialogResult::Close(false));
    }

    #[test]
    fn enter_direct_existing_file_selects_it() {
        let mut d = FileDialog::new(Some(PathBuf::from("/tmp")));
        let model = crate::model::UiModel::new();
        // Use a file guaranteed to exist on unix: /etc/hosts
        d.filter = "/etc/hosts".into();
        let res = d.handle_key(Key::ENTER, &model);
        assert_eq!(res, DialogResult::CloseValue("/etc/hosts".into()));
    }
}
