use crate::colorconf::Theme;
use crate::model::UiModel;
use crate::text::split_row;
use crate::uiconf::UiConfig;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render_top(f: &mut Frame, area: Rect, model: &UiModel, theme: &Theme, conf: &UiConfig) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    let mut title = " nchat".to_string();
    if conf.top_show_version {
        title.push_str(" 0.1.0");
    }

    let status = if model.is_online { "Online" } else { "Offline" };
    let right = format!("{status} ");
    let line = split_row(&title, &right, area.width as usize);

    let p = Paragraph::new(line).style(theme.top);
    f.render_widget(p, area);
}
