use super::{centered_rect, Dialog, DialogResult};
use crate::colorconf::Theme;
use crate::keyconf::Key;
use crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;
use std::path::PathBuf;

pub struct FileDialog {
    pub current_dir: PathBuf,
    pub entries: Vec<PathBuf>,
    pub selected_idx: usize,
}

impl FileDialog {
    pub fn new(start_dir: Option<PathBuf>) -> Self {
        let dir = start_dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let mut d = Self {
            current_dir: dir,
            entries: Vec::new(),
            selected_idx: 0,
        };
        d.refresh_entries();
        d
    }

    pub fn refresh_entries(&mut self) {
        self.entries.clear();
        self.selected_idx = 0;

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
    fn handle_key(&mut self, key: Key) -> DialogResult {
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
            KeyCode::Up => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                }
                DialogResult::KeepOpen
            }
            KeyCode::Down => {
                if self.selected_idx + 1 < count {
                    self.selected_idx += 1;
                }
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
            .border_type(BorderType::Rounded)
            .title(format!(" Send File [{}] ", self.current_dir.display()))
            .style(theme.focus_style);

        let mut lines = Vec::new();
        if self.entries.is_empty() {
            lines.push(Line::from("  Empty directory"));
        } else {
            for (idx, p) in self.entries.iter().enumerate().take(popup_area.height as usize - 2) {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_dialog_lists_directory_and_handles_esc() {
        let mut d = FileDialog::new(Some(PathBuf::from("/tmp")));
        assert!(!d.entries.is_empty());
        assert_eq!(d.handle_key(Key::ESC), DialogResult::Close(false));
    }
}

