use super::{centered_rect, Dialog, DialogResult};
use crate::colorconf::Theme;
use crate::keyconf::Key;
use crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PollOptionData {
    pub id: i32,
    pub text: String,
    #[serde(default)]
    pub chosen: bool,
    #[serde(default)]
    pub votes: i32,
    #[serde(default)]
    pub pct: i32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PollData {
    pub id: String,
    pub question: String,
    #[serde(default)]
    pub closed: bool,
    #[serde(default)]
    pub multiple: bool,
    #[serde(default)]
    pub total_votes: i32,
    #[serde(default)]
    pub options: Vec<PollOptionData>,
}

impl PollData {
    pub fn from_json(json_str: &str) -> Option<Self> {
        serde_json::from_str(json_str).ok()
    }
}

pub struct PollVoteDialog {
    pub poll: PollData,
    pub cursor: usize,
    pub selected_ids: HashSet<i32>,
    pub has_voted: bool,
}

impl PollVoteDialog {
    pub fn new(poll: PollData) -> Self {
        let mut selected_ids = HashSet::new();
        let mut has_voted = false;
        for opt in &poll.options {
            if opt.chosen {
                selected_ids.insert(opt.id);
                has_voted = true;
            }
        }
        Self {
            poll,
            cursor: 0,
            selected_ids,
            has_voted,
        }
    }

    fn total_items(&self) -> usize {
        self.poll.options.len() + if self.has_voted { 1 } else { 0 }
    }
}

impl Dialog for PollVoteDialog {
    fn handle_key(&mut self, key: Key, _model: &crate::model::UiModel) -> DialogResult {
        if self.poll.closed {
            return match key.code {
                KeyCode::Esc | KeyCode::Enter => DialogResult::Close(false),
                _ => DialogResult::KeepOpen,
            };
        }

        let total = self.total_items();
        if total == 0 {
            return DialogResult::Close(false);
        }

        match key.code {
            KeyCode::Esc => DialogResult::Close(false),
            KeyCode::Up | KeyCode::Char('k') => {
                self.cursor = if self.cursor == 0 {
                    total.saturating_sub(1)
                } else {
                    self.cursor - 1
                };
                DialogResult::KeepOpen
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.cursor = if self.cursor + 1 >= total {
                    0
                } else {
                    self.cursor + 1
                };
                DialogResult::KeepOpen
            }
            KeyCode::Char('0') => {
                if self.has_voted {
                    DialogResult::CloseValue("[poll_vote:]".to_string())
                } else {
                    DialogResult::KeepOpen
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let digit = c.to_digit(10).unwrap_or(0) as usize;
                if digit >= 1 && digit <= self.poll.options.len() {
                    let opt_idx = digit - 1;
                    let opt_id = self.poll.options[opt_idx].id;
                    if self.poll.multiple {
                        if self.selected_ids.contains(&opt_id) {
                            self.selected_ids.remove(&opt_id);
                        } else {
                            self.selected_ids.insert(opt_id);
                        }
                        self.cursor = opt_idx;
                        DialogResult::KeepOpen
                    } else {
                        DialogResult::CloseValue(format!("[poll_vote:{opt_id}]"))
                    }
                } else {
                    DialogResult::KeepOpen
                }
            }
            KeyCode::Char(' ') => {
                if self.cursor < self.poll.options.len() {
                    let opt_id = self.poll.options[self.cursor].id;
                    if self.poll.multiple {
                        if self.selected_ids.contains(&opt_id) {
                            self.selected_ids.remove(&opt_id);
                        } else {
                            self.selected_ids.insert(opt_id);
                        }
                    } else {
                        self.selected_ids.clear();
                        self.selected_ids.insert(opt_id);
                    }
                }
                DialogResult::KeepOpen
            }
            KeyCode::Enter => {
                if self.cursor == self.poll.options.len() {
                    // Retract vote option selected
                    DialogResult::CloseValue("[poll_vote:]".to_string())
                } else if self.poll.multiple {
                    let mut ids: Vec<i32> = self.selected_ids.iter().copied().collect();
                    ids.sort_unstable();
                    let s = ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");
                    DialogResult::CloseValue(format!("[poll_vote:{s}]"))
                } else {
                    let opt_id = self.poll.options[self.cursor].id;
                    DialogResult::CloseValue(format!("[poll_vote:{opt_id}]"))
                }
            }
            _ => DialogResult::KeepOpen,
        }
    }

    fn render(
        &mut self,
        f: &mut Frame,
        area: Rect,
        theme: &Theme,
        _img_cache: &mut crate::imgcache::ImageCache,
        _model: &crate::model::UiModel,
    ) {
        let total = self.total_items();
        let height_needed = ((total + 6) as u16).min(area.height.saturating_sub(4)).max(8);
        let width_needed = (area.width * 70 / 100).max(40).min(area.width.saturating_sub(4));
        let x = area.x + (area.width.saturating_sub(width_needed)) / 2;
        let y = area.y + (area.height.saturating_sub(height_needed)) / 2;
        let popup_area = Rect::new(x, y, width_needed, height_needed);

        f.render_widget(Clear, popup_area);

        let mut title = format!(" 📊 Poll: {} ", self.poll.question);
        if self.poll.closed {
            title.push_str("[Closed] ");
        } else if self.poll.multiple {
            title.push_str("[Multiple Choice] ");
        } else {
            title.push_str("[Single Choice] ");
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(theme.focus_style);

        let mut lines = Vec::new();
        lines.push(String::new());

        if self.poll.closed {
            lines.push("  [Poll is closed. Voting is disabled.]".to_string());
            lines.push(String::new());
            lines.push("  [ Esc: Close ]".to_string());
        } else {
            for (i, opt) in self.poll.options.iter().enumerate() {
                let is_cursor = self.cursor == i;
                let checked = self.selected_ids.contains(&opt.id);
                let check_mark = if checked { "[x]" } else { "[ ]" };
                let prefix = if is_cursor { ">" } else { " " };
                let votes_str = if opt.votes > 0 || self.poll.total_votes > 0 {
                    format!(" ({}%, {} votes)", opt.pct, opt.votes)
                } else {
                    String::new()
                };
                lines.push(format!("  {prefix} [{}] {} {}{}", i + 1, check_mark, opt.text, votes_str));
            }

            if self.has_voted {
                let is_cursor = self.cursor == self.poll.options.len();
                let prefix = if is_cursor { ">" } else { " " };
                lines.push(String::new());
                lines.push(format!("  {prefix} [0] Retract vote"));
            }

            lines.push(String::new());
            if self.poll.multiple {
                lines.push("  [ ↑/↓/jk: Navigate | 1-N/Space: Toggle | Enter: Submit | Esc: Cancel ]".to_string());
            } else {
                lines.push("  [ ↑/↓/jk: Navigate | 1-N/Enter: Vote | Esc: Cancel ]".to_string());
            }
        }

        let text = lines.join("\n");
        let p = Paragraph::new(text).block(block);
        f.render_widget(p, popup_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    fn sample_poll_json() -> &'static str {
        r#"{"id":"101","question":"Favorite color?","closed":false,"multiple":false,"total_votes":10,"options":[{"id":0,"text":"Red","chosen":false,"votes":4,"pct":40},{"id":1,"text":"Blue","chosen":true,"votes":6,"pct":60}]}"#
    }

    fn sample_multiple_poll_json() -> &'static str {
        r#"{"id":"102","question":"Languages?","closed":false,"multiple":true,"total_votes":5,"options":[{"id":0,"text":"Rust","chosen":true,"votes":5,"pct":100},{"id":1,"text":"C++","chosen":false,"votes":3,"pct":60}]}"#
    }

    #[test]
    fn parses_poll_data_from_json() {
        let p = PollData::from_json(sample_poll_json()).expect("parse success");
        assert_eq!(p.question, "Favorite color?");
        assert_eq!(p.options.len(), 2);
        assert!(!p.closed);
        assert!(!p.multiple);
        assert_eq!(p.options[1].chosen, true);
    }

    #[test]
    fn single_choice_number_key_votes_immediately() {
        let p = PollData::from_json(sample_poll_json()).unwrap();
        let mut d = PollVoteDialog::new(p);
        let model = crate::model::UiModel::new();
        let res = d.handle_key(Key { code: KeyCode::Char('1'), mods: KeyModifiers::NONE }, &model);
        assert_eq!(res, DialogResult::CloseValue("[poll_vote:0]".to_string()));
    }

    #[test]
    fn retract_vote_key_returns_empty_ids() {
        let p = PollData::from_json(sample_poll_json()).unwrap();
        let mut d = PollVoteDialog::new(p);
        let model = crate::model::UiModel::new();
        let res = d.handle_key(Key { code: KeyCode::Char('0'), mods: KeyModifiers::NONE }, &model);
        assert_eq!(res, DialogResult::CloseValue("[poll_vote:]".to_string()));
    }

    #[test]
    fn multi_choice_toggling_and_submit() {
        let p = PollData::from_json(sample_multiple_poll_json()).unwrap();
        let mut d = PollVoteDialog::new(p);
        let model = crate::model::UiModel::new();
        // Toggle option 2 (id 1)
        let _ = d.handle_key(Key { code: KeyCode::Char('2'), mods: KeyModifiers::NONE }, &model);
        assert!(d.selected_ids.contains(&0));
        assert!(d.selected_ids.contains(&1));

        // Submit with Enter
        let res = d.handle_key(Key { code: KeyCode::Enter, mods: KeyModifiers::NONE }, &model);
        assert_eq!(res, DialogResult::CloseValue("[poll_vote:0,1]".to_string()));
    }

    #[test]
    fn closed_poll_only_allows_esc_or_enter_close() {
        let mut p = PollData::from_json(sample_poll_json()).unwrap();
        p.closed = true;
        let mut d = PollVoteDialog::new(p);
        let model = crate::model::UiModel::new();
        let res1 = d.handle_key(Key { code: KeyCode::Char('1'), mods: KeyModifiers::NONE }, &model);
        assert_eq!(res1, DialogResult::KeepOpen);
        let res2 = d.handle_key(Key { code: KeyCode::Esc, mods: KeyModifiers::NONE }, &model);
        assert_eq!(res2, DialogResult::Close(false));
    }
}
