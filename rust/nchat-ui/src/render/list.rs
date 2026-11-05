use crate::chatlist::{scroll_first, ListRow};
use crate::colorconf::Theme;
use crate::model::UiModel;
use crate::nav::Focus;
use crate::render::time::format_list_time;
use crate::text::{fit, split_row};
use crate::uiconf::UiConfig;
use ratatui::layout::{Margin, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::Frame;

pub fn render_list(f: &mut Frame, area: Rect, model: &mut UiModel, theme: &Theme, conf: &UiConfig) {
    if area.width < 4 || area.height < 3 {
        return;
    }

    let border_style = if model.focus == Focus::List {
        theme.focus_style
    } else {
        theme.listborder
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(" Chats ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let full_w = inner.width as usize;
    let content_w = full_w;
    let content_h = inner.height as usize;
    if full_w == 0 || content_h == 0 {
        return;
    }

    let has_bar = (model.filter_active || !model.filter_text.is_empty() || model.unread_only) && content_h > 1;
    let list_h = if has_bar { content_h - 1 } else { content_h };

    let rows = model.current_rows();
    let sel_row = model.selected_chat_row(&rows);
    let two_lines = inner.width >= 20 && list_h >= 4;

    let row_lines: Vec<usize> = rows
        .iter()
        .map(|r| match r {
            ListRow::Header { .. } => 1,
            ListRow::Chat(_) => {
                if two_lines {
                    2
                } else {
                    1
                }
            }
        })
        .collect();

    model.scroll_first = scroll_first(&row_lines, model.scroll_first, sel_row, list_h, false);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let mut lines = Vec::new();

    if has_bar {
        let text = format!(
            "/{}{}{}",
            model.filter_text,
            if model.filter_active { "_" } else { "" },
            if model.unread_only { " [unread]" } else { "" }
        );
        let bar_line = fit(&text, full_w);
        lines.push(Line::from(Span::styled(bar_line, theme.focus_style)));
    }

    let end = (model.scroll_first + list_h).min(rows.len());

    for r_idx in model.scroll_first..end {
        let is_sel = sel_row == Some(r_idx);
        match &rows[r_idx] {
            ListRow::Header { group, collapsed } => {
                let prefix = if *collapsed { "> " } else { "v " };
                let text = format!("{prefix}{group}");
                let fitted = fit(&text, full_w);
                lines.push(Line::from(Span::styled(fitted, theme.list_name)));
            }
            ListRow::Chat(c_idx) => {
                let chat = &model.chats[*c_idx];
                let mut marks = String::new();
                if chat.is_pinned {
                    marks.push('^');
                }
                if chat.is_muted {
                    marks.push('~');
                }

                let time_str = if two_lines {
                    format_list_time(chat.last_message_time, now)
                } else {
                    String::new()
                };

                let right_part = if !time_str.is_empty() && !marks.is_empty() {
                    format!("{time_str} {marks}")
                } else if !time_str.is_empty() {
                    time_str
                } else {
                    marks
                };

                let marker_w = if chat.is_unread { conf.unread_indicator.chars().count() + 1 } else { 0 };
                let name_w = content_w.saturating_sub(marker_w);
                let row_str = split_row(&chat.name, &right_part, name_w);
                let style = if is_sel {
                    theme.list_selected
                } else if chat.is_unread {
                    theme.list_unread
                } else {
                    theme.list
                };

                let mut spans = vec![Span::styled(row_str, style)];
                if chat.is_unread {
                    let unread_style = if theme.list_unread.fg.is_some() {
                        theme.list_unread
                    } else {
                        Style::default().fg(Color::Rgb(255, 99, 71))
                    };
                    spans.push(Span::styled(format!(" {}", conf.unread_indicator), unread_style));
                }
                lines.push(Line::from(spans));

                if two_lines && lines.len() < content_h {
                    let preview_str = fit(&format!("  {}", chat.preview), content_w);
                    let preview_style = if is_sel {
                        theme.list_selected
                    } else {
                        theme.list_preview
                    };
                    lines.push(Line::from(Span::styled(preview_str, preview_style)));
                }
            }
        }
    }

    let p = Paragraph::new(lines).style(theme.list);
    f.render_widget(p, inner);

    if rows.len() > list_h {
        let mut sb_area = area.inner(Margin { vertical: 1, horizontal: 0 });
        if has_bar {
            sb_area.y += 1;
            sb_area.height = sb_area.height.saturating_sub(1);
        }
        let mut state = ScrollbarState::new(rows.len()).position(model.scroll_first);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight).begin_symbol(None).end_symbol(None),
            sb_area,
            &mut state,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Chat;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn renders_square_border_and_colored_unread_badge() {
        let backend = TestBackend::new(30, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut model = UiModel::new();
        model.apply_event(&crate::protocol::Event::Chats {
            profile: "P1".into(),
            success: true,
            chats: vec![Chat {
                id: "c1".into(),
                is_unread: true,
                is_unread_mention: false,
                is_muted: false,
                is_pinned: false,
                is_archived: false,
                last_message_time: 10,
            }],
        });

        let theme = Theme::default();
        let conf = UiConfig::default();

        terminal
            .draw(|f| {
                render_list(f, Rect::new(0, 0, 30, 10), &mut model, &theme, &conf);
            })
            .unwrap();

        let buf = terminal.backend().buffer();
        assert_eq!(buf[(0, 0)].symbol(), "┌");
        let row1: String = (0..30).map(|x| buf[(x, 1)].symbol()).collect();
        assert!(!row1.contains('▎'), "chat row should not contain avatar bar glyph");
        assert!(row1.contains('*'), "expected default unread indicator, got: {row1:?}");
    }
}
