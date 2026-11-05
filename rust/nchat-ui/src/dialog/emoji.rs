use super::{centered_rect, Dialog, DialogResult};
use crate::colorconf::Theme;
use crate::keyconf::Key;
use crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

const DEFAULT_EMOJIS: &[(&str, &str)] = &[
    ("😀", "grinning smile"),
    ("😃", "smiley face"),
    ("😄", "smile grin"),
    ("😁", "beam grin"),
    ("😆", "laughing squint"),
    ("😅", "sweat smile"),
    ("🤣", "rofl rolling"),
    ("😂", "joy tears"),
    ("🙂", "slightly smiling"),
    ("🙃", "upside down"),
    ("😉", "wink"),
    ("😊", "blush"),
    ("😇", "halo innocent"),
    ("🥰", "hearts love"),
    ("😍", "heart eyes"),
    ("🤩", "star struck"),
    ("😘", "kissing heart"),
    ("😗", "kissing"),
    ("😚", "kissing closed eyes"),
    ("😙", "kissing smiling eyes"),
    ("😋", "yum delicious"),
    ("😛", "tongue"),
    ("😜", "wink tongue"),
    ("🤪", "zany wild"),
    ("😝", "squint tongue"),
    ("🤑", "money face"),
    ("🤗", "hugs hugging"),
    ("🤭", "hand over mouth"),
    ("🤫", "shushing quiet"),
    ("🤔", "thinking"),
    ("🤐", "zipper mouth"),
    ("🤨", "raised eyebrow"),
    ("😐", "neutral face"),
    ("😑", "expressionless"),
    ("😶", "no mouth"),
    ("😏", "smirk"),
    ("😒", "unamused"),
    ("🙄", "rolling eyes"),
    ("😬", "grimacing"),
    ("🤥", "lying"),
    ("😌", "relieved"),
    ("😔", "pensive"),
    ("😪", "sleepy"),
    ("🤤", "drooling"),
    ("😴", "sleeping"),
    ("😷", "mask sick"),
    ("🤒", "thermometer fever"),
    ("🤕", "bandage hurt"),
    ("🤢", "nauseated vomit"),
    ("🤮", "vomiting puke"),
    ("🤧", "sneezing"),
    ("🥵", "hot sweating"),
    ("🥶", "cold freezing"),
    ("🥴", "woozy drunk"),
    ("😵", "dizzy dead"),
    ("🤯", "exploding head"),
    ("🤠", "cowboy"),
    ("🥳", "partying celebration"),
    ("😎", "sunglasses cool"),
    ("🤓", "nerd glasses"),
    ("🧐", "monocle"),
    ("👍", "thumbs up like"),
    ("👎", "thumbs down dislike"),
    ("👏", "clapping applause"),
    ("🙌", "raising hands celebrate"),
    ("🙏", "folded hands please pray thanks"),
    ("❤️", "red heart love"),
    ("🔥", "fire flame hot lit"),
    ("🎉", "party popper tada"),
    ("✨", "sparkles shiny"),
    ("💯", "hundred points perfect"),
];

pub struct EmojiDialog {
    pub filter: String,
    pub selected_idx: usize,
}

impl EmojiDialog {
    pub fn new() -> Self {
        Self {
            filter: String::new(),
            selected_idx: 0,
        }
    }

    pub fn filtered_emojis(&self) -> Vec<(&'static str, &'static str)> {
        if self.filter.is_empty() {
            DEFAULT_EMOJIS.to_vec()
        } else {
            let q = self.filter.to_lowercase();
            DEFAULT_EMOJIS
                .iter()
                .copied()
                .filter(|(emoji, desc)| emoji.contains(&q) || desc.contains(&q))
                .collect()
        }
    }
}

impl Default for EmojiDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl Dialog for EmojiDialog {
    fn handle_key(&mut self, key: Key) -> DialogResult {
        let emojis = self.filtered_emojis();
        let cols = 8;
        let count = emojis.len();

        match key.code {
            KeyCode::Esc => DialogResult::Close(false),
            KeyCode::Enter => {
                if let Some((emoji, _)) = emojis.get(self.selected_idx) {
                    DialogResult::CloseValue(emoji.to_string())
                } else {
                    DialogResult::Close(false)
                }
            }
            KeyCode::Left => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                }
                DialogResult::KeepOpen
            }
            KeyCode::Right => {
                if self.selected_idx + 1 < count {
                    self.selected_idx += 1;
                }
                DialogResult::KeepOpen
            }
            KeyCode::Up => {
                self.selected_idx = self.selected_idx.saturating_sub(cols);
                DialogResult::KeepOpen
            }
            KeyCode::Down => {
                if self.selected_idx + cols < count {
                    self.selected_idx += cols;
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
        let popup_area = centered_rect(65, 55, area);
        f.render_widget(Clear, popup_area);

        let title = if self.filter.is_empty() {
            " Emoji Picker (Alt-u) ".to_string()
        } else {
            format!(" Emoji Picker [/{}]: ", self.filter)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(theme.focus_style);

        let emojis = self.filtered_emojis();
        let cols = 8;
        let mut lines = Vec::new();

        lines.push(Line::from(format!(" Filter: {}_ (type to search)", self.filter)));
        lines.push(Line::from(""));

        for (row_idx, chunk) in emojis.chunks(cols).enumerate() {
            let mut spans = Vec::new();
            spans.push(Span::raw("  "));
            for (col_idx, (emoji, _)) in chunk.iter().enumerate() {
                let idx = row_idx * cols + col_idx;
                if idx == self.selected_idx {
                    spans.push(Span::styled(format!(" [{emoji}] "), theme.list_selected));
                } else {
                    spans.push(Span::raw(format!("  {emoji}  ")));
                }
            }
            lines.push(Line::from(spans));
            lines.push(Line::from(""));
        }

        let p = Paragraph::new(lines).block(block);
        f.render_widget(p, popup_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emoji_picker_filters_and_selects() {
        let mut d = EmojiDialog::new();
        d.filter = "fire".into();
        let emojis = d.filtered_emojis();
        assert!(!emojis.is_empty());
        assert_eq!(emojis[0].0, "🔥");

        let res = d.handle_key(Key::ENTER);
        assert_eq!(res, DialogResult::CloseValue("🔥".into()));
    }
}
