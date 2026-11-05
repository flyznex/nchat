use super::{centered_rect, Dialog, DialogResult};
use crate::colorconf::Theme;
use crate::keyconf::Key;
use crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub struct MentionDialog {
    pub members: Vec<String>,
    pub filter: String,
    pub selected_idx: usize,
}

impl MentionDialog {
    pub fn new(members: Vec<String>) -> Self {
        Self {
            members,
            filter: String::new(),
            selected_idx: 0,
        }
    }

    pub fn filtered_members(&self) -> Vec<&str> {
        let q = self.filter.to_lowercase();
        self.members
            .iter()
            .map(|s| s.as_str())
            .filter(|m| q.is_empty() || m.to_lowercase().contains(&q))
            .collect()
    }
}

impl Dialog for MentionDialog {
    fn handle_key(&mut self, key: Key, _model: &crate::model::UiModel) -> DialogResult {
        let filtered = self.filtered_members();
        let count = filtered.len();

        match key.code {
            KeyCode::Esc => DialogResult::Close(false),
            KeyCode::Enter => {
                if let Some(m) = filtered.get(self.selected_idx) {
                    let formatted = if m.contains(' ') {
                        format!("@[{m}] ")
                    } else {
                        format!("@{m} ")
                    };
                    DialogResult::CloseValue(formatted)
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
            KeyCode::Backspace => {
                self.filter.pop();
                self.selected_idx = 0;
                DialogResult::KeepOpen
            }
            KeyCode::Char(c) => {
                self.filter.push(c);
                self.selected_idx = 0;
                DialogResult::KeepOpen
            }
            _ => DialogResult::KeepOpen,
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme, _img_cache: &mut crate::imgcache::ImageCache, _model: &crate::model::UiModel) {
        let popup_area = centered_rect(50, 50, area);
        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Mention Member [/{}]: ", self.filter))
            .border_style(theme.focus_style);

        let filtered = self.filtered_members();
        let mut lines = Vec::new();

        if filtered.is_empty() {
            lines.push(Line::from("  No members found"));
        } else {
            for (idx, m) in filtered.iter().enumerate() {
                if idx == self.selected_idx {
                    lines.push(Line::from(Span::styled(format!(" > @{m} "), theme.list_selected)));
                } else {
                    lines.push(Line::from(format!("   @{m} ")));
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
    fn mention_dialog_selects_and_formats() {
        let mut d = MentionDialog::new(vec!["Alice Smith".into(), "Bob".into()]);
        d.filter = "alice".into();
        let filtered = d.filtered_members();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "Alice Smith");

        let model = crate::model::UiModel::new();
        let res = d.handle_key(Key::ENTER, &model);
        assert_eq!(res, DialogResult::CloseValue("@[Alice Smith] ".into()));
    }
}
