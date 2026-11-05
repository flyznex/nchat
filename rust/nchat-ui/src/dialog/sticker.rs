use super::{centered_rect, Dialog, DialogResult};
use crate::colorconf::Theme;
use crate::keyconf::Key;
use crate::model::UiModel;
use crate::protocol::StickerInfo;
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;
use ratatui_image::StatefulImage;

// ponytail: only the "recent" (favorites+recent) key is shown; per-set tab
// browsing needs mid-dialog request round-trips, add when a user asks for it.
const STICKER_KEY: &str = "recent";

pub struct StickerDialog {
    pub profile: String,
    pub selected_idx: usize,
    cached: Vec<StickerInfo>,
}

impl StickerDialog {
    pub fn new(profile: String) -> Self {
        Self { profile, selected_idx: 0, cached: Vec::new() }
    }
}

impl Dialog for StickerDialog {
    fn handle_key(&mut self, key: Key) -> DialogResult {
        let count = self.cached.len();
        let cols = 4;
        match key.code {
            KeyCode::Esc => DialogResult::Close(false),
            KeyCode::Enter => {
                if let Some(s) = self.cached.get(self.selected_idx) {
                    DialogResult::CloseValue(format!("[sticker_file_id:{}]", s.file_id))
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
                if count > 0 && self.selected_idx + 1 < count {
                    self.selected_idx += 1;
                }
                DialogResult::KeepOpen
            }
            KeyCode::Up => {
                self.selected_idx = self.selected_idx.saturating_sub(cols);
                DialogResult::KeepOpen
            }
            KeyCode::Down => {
                if count > 0 && self.selected_idx + cols < count {
                    self.selected_idx += cols;
                }
                DialogResult::KeepOpen
            }
            _ => DialogResult::KeepOpen,
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme, img_cache: &mut crate::imgcache::ImageCache, model: &UiModel) {
        self.cached = model.stickers(&self.profile, STICKER_KEY).map(|s| s.to_vec()).unwrap_or_default();
        if !self.cached.is_empty() {
            self.selected_idx = self.selected_idx.min(self.cached.len() - 1);
        }

        let popup_area = centered_rect(75, 65, area);
        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(format!(" Send Sticker (Alt-g) - Recent [{}] ", self.cached.len()))
            .style(theme.focus_style);

        let inner_area = block.inner(popup_area);
        f.render_widget(block, popup_area);

        if self.cached.is_empty() {
            f.render_widget(
                Paragraph::new("\n  Loading stickers... (or no recent stickers yet)").style(theme.history_text_quoted),
                inner_area,
            );
            return;
        }

        let columns = Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).split(inner_area);
        let left_pane = columns[0];
        let right_pane = columns[1];

        let cols = 4;
        let rows_per_page = (left_pane.height.saturating_sub(2) / 2).max(1) as usize;
        let page_size = cols * rows_per_page;
        let page_start = (self.selected_idx / page_size.max(1)) * page_size.max(1);

        let mut lines = Vec::new();
        lines.push(Line::from("  Arrow keys to choose, Enter to send:"));
        lines.push(Line::from(""));

        let chunk_vec: Vec<(usize, &StickerInfo)> =
            self.cached.iter().enumerate().skip(page_start).take(page_size.max(cols)).collect();

        for chunk in chunk_vec.chunks(cols) {
            let mut spans = Vec::new();
            spans.push(Span::raw("  "));
            for &(idx, sticker) in chunk {
                let label = if sticker.emoji.is_empty() { "🎭" } else { &sticker.emoji };
                if idx == self.selected_idx {
                    spans.push(Span::styled(format!(" [{label}] "), theme.list_selected));
                } else {
                    spans.push(Span::raw(format!("  {label}  ")));
                }
            }
            lines.push(Line::from(spans));
        }

        f.render_widget(Paragraph::new(lines), left_pane);

        let preview_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Sticker Preview ")
            .style(theme.listborder);
        let preview_inner = preview_block.inner(right_pane);
        f.render_widget(preview_block, right_pane);

        if let Some(sticker) = self.cached.get(self.selected_idx) {
            if !sticker.thumbnail_path.is_empty() {
                if let Some(state) = img_cache.get_or_request(&sticker.thumbnail_path) {
                    f.render_stateful_widget(StatefulImage::default(), preview_inner, state);
                } else {
                    f.render_widget(
                        Paragraph::new("  [Loading preview...]").style(theme.history_text_quoted),
                        preview_inner,
                    );
                }
            } else {
                f.render_widget(
                    Paragraph::new(format!("\n\n   {}", sticker.emoji)).style(theme.history_text_recv),
                    preview_inner,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sticker_dialog_starts_empty_until_model_supplies_data() {
        let d = StickerDialog::new("P1".into());
        assert_eq!(d.selected_idx, 0);
        assert!(d.cached.is_empty());
    }
}
