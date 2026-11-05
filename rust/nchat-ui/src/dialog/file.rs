use super::{centered_rect, Dialog, DialogResult};
use crate::colorconf::Theme;
use crate::keyconf::Key;
use crossterm::event::KeyCode;
use ratatui::layout::{Margin, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::Frame;
use std::path::PathBuf;

pub struct FileDialog {
    pub current_dir: PathBuf,
    pub entries: Vec<PathBuf>,
    pub selected_idx: usize,
    pub scroll_first: usize,
}

impl FileDialog {
    pub fn new(start_dir: Option<PathBuf>) -> Self {
        let dir = start_dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let mut d = Self {
            current_dir: dir,
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
            self.entries.push(parent.to_path_buf());
        }

        if let Ok(rd) = std::fs::read_dir(&self.current_dir) {
            let mut list: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
            list.sort();
            self.entries.extend(list);
        }
    }
}

impl Dialog for FileDialog {
    fn handle_key(&mut self, key: Key, _model: &crate::model::UiModel) -> DialogResult {
        let count = self.entries.len();
        match key.code {
            KeyCode::Esc => DialogResult::Close(false),
            KeyCode::Enter => {
                if let Some(path) = self.entries.get(self.selected_idx) {
                    if path.is_dir() {
                        self.current_dir = path.clone();
                        self.refresh_entries();
                        DialogResult::KeepOpen
                    } else {
                        DialogResult::CloseValue(path.to_string_lossy().to_string())
                    }
                } else {
                    DialogResult::Close(false)
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                }
                DialogResult::KeepOpen
            }
            KeyCode::Down | KeyCode::Char('j') => {
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
            _ => DialogResult::KeepOpen,
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme, _img_cache: &mut crate::imgcache::ImageCache, _model: &crate::model::UiModel) {
        let popup_area = centered_rect(70, 60, area);
        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Send File [{}] ", self.current_dir.display()))
            .border_style(theme.focus_style);

        let visible_h = (popup_area.height as usize).saturating_sub(2);
        if visible_h > 0 {
            if self.selected_idx >= self.scroll_first + visible_h {
                self.scroll_first = self.selected_idx + 1 - visible_h;
            } else if self.selected_idx < self.scroll_first {
                self.scroll_first = self.selected_idx;
            }
        }

        let mut lines = Vec::new();
        if self.entries.is_empty() {
            lines.push(Line::from("  Empty directory"));
        } else {
            for (idx, p) in self.entries.iter().enumerate().skip(self.scroll_first).take(visible_h) {
                let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "..".into());
                let prefix = if p.is_dir() { "📁 " } else { "📄 " };
                let display = format!("{prefix}{name}");

                if idx == self.selected_idx {
                    lines.push(Line::from(Span::styled(format!(" > {display} "), theme.list_selected)));
                } else {
                    lines.push(Line::from(format!("   {display} ")));
                }
            }
        }

        let p = Paragraph::new(lines).block(block);
        f.render_widget(p, popup_area);

        if self.entries.len() > visible_h {
            let mut state = ScrollbarState::new(self.entries.len()).position(self.selected_idx);
            f.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight).begin_symbol(None).end_symbol(None),
                popup_area.inner(Margin { vertical: 1, horizontal: 0 }),
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
    fn file_dialog_navigates_with_arrows_and_jk() {
        let mut d = FileDialog::new(Some(PathBuf::from("/tmp")));
        let model = crate::model::UiModel::new();
        let count = d.entries.len();
        if count > 1 {
            let k_down = Key { code: KeyCode::Char('j'), mods: crossterm::event::KeyModifiers::NONE };
            d.handle_key(k_down, &model);
            assert_eq!(d.selected_idx, 1);

            let k_up = Key { code: KeyCode::Char('k'), mods: crossterm::event::KeyModifiers::NONE };
            d.handle_key(k_up, &model);
            assert_eq!(d.selected_idx, 0);
        }
    }
}

