use super::{centered_rect, Dialog, DialogResult};
use crate::colorconf::Theme;
use crate::keyconf::Key;
use crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

pub struct MessageDialog {
    pub title: String,
    pub message: String,
}

impl MessageDialog {
    pub fn new(title: &str, message: &str) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
        }
    }
}

impl Dialog for MessageDialog {
    fn handle_key(&mut self, key: Key) -> DialogResult {
        match key.code {
            KeyCode::Esc => DialogResult::Close(false),
            KeyCode::Enter => DialogResult::Close(true),
            _ => DialogResult::KeepOpen,
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme, _img_cache: &mut crate::imgcache::ImageCache, _model: &crate::model::UiModel) {
        let popup_area = centered_rect(60, 30, area);
        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(format!(" {} ", self.title))
            .style(theme.focus_style);

        let content = format!("\n  {}\n\n  [ Enter: OK | Esc: Cancel ]", self.message);
        let p = Paragraph::new(content).block(block);
        f.render_widget(p, popup_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_dialog_returns_close_on_enter_or_esc() {
        let mut d = MessageDialog::new("Confirm", "Are you sure?");
        assert_eq!(d.handle_key(Key::ENTER), DialogResult::Close(true));
        assert_eq!(d.handle_key(Key::ESC), DialogResult::Close(false));
    }
}
