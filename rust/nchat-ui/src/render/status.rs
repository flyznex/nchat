use crate::colorconf::Theme;
use crate::model::UiModel;
use crate::nav::Focus;
use crate::text::split_row;
use crate::uiconf::UiConfig;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render_status(f: &mut Frame, area: Rect, model: &UiModel, theme: &Theme, _conf: &UiConfig) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    let left = match model.selected_chat() {
        Some(c) => format!(" {}", c.name),
        None => String::new(),
    };

    let tag = match model.focus {
        Focus::List => " LIST ",
        Focus::History => " HISTORY ",
        Focus::Entry => " ENTRY ",
    };

    let line = split_row(&left, tag, area.width as usize);
    let p = Paragraph::new(line).style(theme.status);
    f.render_widget(p, area);
}
