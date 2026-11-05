use crate::colorconf::Theme;
use crate::model::UiModel;
use crate::nav::Focus;
use crate::uiconf::UiConfig;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render_status(f: &mut Frame, area: Rect, model: &UiModel, theme: &Theme, _conf: &UiConfig) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    let left = match model.selected_chat() {
        Some(c) => {
            let status_str = model.chat_status(&c.profile_id, &c.chat_id);
            let status_part = if !status_str.is_empty() {
                format!(" ({status_str})")
            } else {
                String::new()
            };
            if model.profiles.len() > 1 {
                format!(" {}{status_part} @ {}", c.name, c.profile_id)
            } else {
                format!(" {}{status_part}", c.name)
            }
        }
        None => String::new(),
    };

    let tag = match model.focus {
        Focus::List => " LIST ",
        Focus::History => " HISTORY ",
        Focus::Entry => " ENTRY ",
    };

    let tag_len = tag.len();
    let left_w = (area.width as usize).saturating_sub(tag_len);
    let left_str = crate::text::fit(&left, left_w);
    let left_span = Span::styled(left_str, theme.status);
    let tag_span = Span::styled(tag, theme.focus_style);
    let line = Line::from(vec![left_span, tag_span]);
    let p = Paragraph::new(line);
    f.render_widget(p, area);
}
