use crate::colorconf::Theme;
use crate::model::UiModel;
use crate::nav::Focus;
use crate::uiconf::UiConfig;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

pub fn render_entry(
    f: &mut Frame,
    area: Rect,
    model: &UiModel,
    theme: &Theme,
    _conf: &UiConfig,
) {
    if area.width < 4 || area.height < 3 {
        return;
    }

    let is_focused = model.focus == Focus::Entry;
    let border_style = if is_focused { theme.focus_style } else { theme.listborder };

    let title = if model.edit_message_active {
        " Editing message | Enter save | Esc cancel".to_string()
    } else if model.reply_message_active {
        if let Some(msg) = model.selected_message() {
            let who = if msg.is_outgoing { "yourself" } else { &msg.sender_id };
            format!(" Replying to {who} | Enter send | Esc cancel")
        } else {
            " Replying to message | Enter send | Esc cancel".to_string()
        }
    } else {
        " Message | Enter send | Alt-Enter newline".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(border_style)
        .title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let text = model.entry_buffer.text();
    let text_lines: Vec<Line> = if text.is_empty() {
        vec![Line::from("")]
    } else {
        text.lines().map(Line::from).collect()
    };
    f.render_widget(Paragraph::new(text_lines), inner);

    if is_focused {
        let cx = inner.x + (model.entry_buffer.cursor_idx() as u16).min(inner.width.saturating_sub(1));
        let cy = inner.y;
        f.set_cursor_position((cx, cy));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colorconf::Theme;
    use crate::model::UiModel;
    use crate::uiconf::UiConfig;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::Terminal;

    #[test]
    fn renders_entry_title_and_content() {
        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut model = UiModel::new();
        model.entry_buffer.insert_str("Hello");
        let theme = Theme::default();
        let conf = UiConfig::default();

        terminal
            .draw(|f| {
                render_entry(f, Rect::new(0, 0, 40, 5), &model, &theme, &conf);
            })
            .unwrap();

        let buf = terminal.backend().buffer();
        let title_line: String = (0..40).map(|x| buf[(x, 0)].symbol()).collect();
        let content_line: String = (0..40).map(|x| buf[(x, 1)].symbol()).collect();

        assert!(title_line.contains("Message"));
        assert_eq!(buf[(0, 0)].symbol(), "╭");
        assert!(content_line.contains("Hello"));
    }
}
