use super::{centered_rect, Dialog, DialogResult};
use crate::colorconf::Theme;
use crate::keyconf::Key;
use crate::model::UiModel;
use crate::protocol::StickerInfo;
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use ratatui_image::StatefulImage;

pub struct StickerDialog {
    pub profile: String,
    pub tab: usize, // 0 = Recent, 1..=set_ids.len() = that sticker set
    pub selected_idx: usize,
    cached: Vec<StickerInfo>,
    set_ids: Vec<i64>,
    set_titles: Vec<String>,
}

impl StickerDialog {
    pub fn new(profile: String) -> Self {
        Self {
            profile,
            tab: 0,
            selected_idx: 0,
            cached: Vec::new(),
            set_ids: Vec::new(),
            set_titles: Vec::new(),
        }
    }

    fn current_key(&self) -> String {
        if self.tab == 0 {
            "recent".to_string()
        } else {
            format!("set:{}", self.set_ids[self.tab - 1])
        }
    }

    fn current_set_id(&self) -> i64 {
        if self.tab == 0 {
            0
        } else {
            self.set_ids[self.tab - 1]
        }
    }

    fn tab_count(&self) -> usize {
        1 + self.set_ids.len()
    }

    fn switch_tab(&mut self, model: &UiModel) -> DialogResult {
        self.selected_idx = 0;
        let key = self.current_key();
        if model.stickers(&self.profile, &key).is_some() {
            DialogResult::KeepOpen
        } else {
            DialogResult::FetchStickers {
                profile: self.profile.clone(),
                key,
                set_id: self.current_set_id(),
            }
        }
    }
}

impl Dialog for StickerDialog {
    fn handle_key(&mut self, key: Key, model: &UiModel) -> DialogResult {
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
            KeyCode::Tab => {
                let tab_count = self.tab_count();
                self.tab = (self.tab + 1) % tab_count.max(1);
                self.switch_tab(model)
            }
            KeyCode::BackTab => {
                let tab_count = self.tab_count();
                self.tab = if self.tab == 0 { tab_count.saturating_sub(1) } else { self.tab - 1 };
                self.switch_tab(model)
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
        self.set_ids = model.sticker_sets(&self.profile).iter().map(|s| s.id).collect();
        self.set_titles = model.sticker_sets(&self.profile).iter().map(|s| s.title.clone()).collect();
        self.tab = self.tab.min(self.tab_count().saturating_sub(1));

        self.cached = model.stickers(&self.profile, &self.current_key()).map(|s| s.to_vec()).unwrap_or_default();
        if !self.cached.is_empty() {
            self.selected_idx = self.selected_idx.min(self.cached.len() - 1);
        }

        let popup_area = centered_rect(75, 65, area);
        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Send Sticker (Alt-g) [{}] ", self.cached.len()))
            .border_style(theme.focus_style);

        let inner_area = block.inner(popup_area);
        f.render_widget(block, popup_area);

        if inner_area.height < 2 {
            return;
        }
        let tab_area = Rect::new(inner_area.x, inner_area.y, inner_area.width, 1);
        let body_area = Rect::new(inner_area.x, inner_area.y + 1, inner_area.width, inner_area.height - 1);

        // Tab bar: "Recent" plus each installed sticker set's title
        let mut tab_spans = vec![Span::raw(" ")];
        for (idx, label) in std::iter::once("Recent").chain(self.set_titles.iter().map(|s| s.as_str())).enumerate() {
            let style = if idx == self.tab { theme.list_selected } else { theme.listborder };
            tab_spans.push(Span::styled(format!(" {label} "), style));
            tab_spans.push(Span::raw(" "));
        }
        tab_spans.push(Span::raw(" (Tab/Shift-Tab to switch)"));
        f.render_widget(Paragraph::new(Line::from(tab_spans)), tab_area);

        if self.cached.is_empty() {
            f.render_widget(
                Paragraph::new("\n  Loading stickers... (or this set/recent is empty)").style(theme.history_text_quoted),
                body_area,
            );
            return;
        }

        let columns = Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).split(body_area);
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
            .title(" Sticker Preview ")
            .border_style(theme.listborder);
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
    use crate::protocol::{Event, StickerSetInfo};

    #[test]
    fn sticker_dialog_starts_empty_until_model_supplies_data() {
        let d = StickerDialog::new("P1".into());
        assert_eq!(d.selected_idx, 0);
        assert_eq!(d.tab, 0);
        assert!(d.cached.is_empty());
    }

    #[test]
    fn tab_switches_to_other_sets_and_requests_missing_data() {
        let mut model = UiModel::new();
        model.apply_event(&Event::StickerSets {
            profile: "P1".into(),
            sticker_sets: vec![
                StickerSetInfo { id: 111, title: "Cats".into() },
                StickerSetInfo { id: 222, title: "Dogs".into() },
            ],
        });

        let mut d = StickerDialog::new("P1".into());
        d.set_ids = vec![111, 222];
        d.set_titles = vec!["Cats".into(), "Dogs".into()];

        // Tab from Recent (0) to the "Cats" set (1): not cached yet -> must request it.
        let key = Key { code: KeyCode::Tab, mods: crossterm::event::KeyModifiers::NONE };
        match d.handle_key(key, &model) {
            DialogResult::FetchStickers { profile, key, set_id } => {
                assert_eq!(profile, "P1");
                assert_eq!(key, "set:111");
                assert_eq!(set_id, 111);
            }
            other => panic!("expected FetchStickers, got {other:?}"),
        }
        assert_eq!(d.tab, 1);

        // Once the set's stickers are in the model, switching to it again must not re-request.
        model.apply_event(&Event::Stickers {
            profile: "P1".into(),
            key: "set:111".into(),
            stickers: vec![StickerInfo { file_id: 1, emoji: "🐱".into(), thumbnail_path: "".into() }],
        });
        d.tab = 0;
        let key = Key { code: KeyCode::Tab, mods: crossterm::event::KeyModifiers::NONE };
        assert_eq!(d.handle_key(key, &model), DialogResult::KeepOpen);
        assert_eq!(d.tab, 1);
    }
}
