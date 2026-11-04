use crate::chatlist::{scroll_first, scroll_thumb, ListRow};
use crate::colorconf::Theme;
use crate::model::UiModel;
use crate::nav::Focus;
use crate::render::time::format_list_time;
use crate::text::{fit, split_row};
use crate::uiconf::UiConfig;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render_list(f: &mut Frame, area: Rect, model: &mut UiModel, theme: &Theme, conf: &UiConfig) {
    if area.width < 2 || area.height == 0 {
        return;
    }

    // 1-cell inset padding
    let content_w = (area.width.saturating_sub(2)) as usize;
    let content_h = area.height as usize;
    if content_w == 0 || content_h == 0 {
        return;
    }

    let has_bar = (model.filter_active || !model.filter_text.is_empty() || model.unread_only) && content_h > 1;
    let list_h = if has_bar { content_h - 1 } else { content_h };

    let rows = model.current_rows();
    let sel_row = model.selected_chat_row(&rows);
    let two_lines = area.width >= 20 && list_h >= 4;

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
        let bar_line = fit(&text, content_w);
        lines.push(Line::from(Span::styled(bar_line, theme.focus_style)));
    }

    let end = (model.scroll_first + list_h).min(rows.len());

    for r_idx in model.scroll_first..end {
        let is_sel = sel_row == Some(r_idx);
        match &rows[r_idx] {
            ListRow::Header { group, collapsed } => {
                let prefix = if *collapsed { "> " } else { "v " };
                let text = format!("{prefix}{group}");
                let fitted = fit(&text, content_w);
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
                if chat.is_unread {
                    marks.push_str(&conf.unread_indicator);
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

                let row_str = split_row(&chat.name, &right_part, content_w);
                let style = if is_sel {
                    theme.list_selected
                } else if chat.is_unread {
                    theme.list_unread
                } else {
                    theme.list
                };
                lines.push(Line::from(Span::styled(row_str, style)));

                if two_lines && lines.len() < content_h {
                    let preview_str = fit(&format!(" {}", chat.preview), content_w);
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
    let inner_area = Rect::new(area.x + 1, area.y, area.width.saturating_sub(2), area.height);
    f.render_widget(p, inner_area);

    // Scrollbar in the rightmost cell
    let track = area.height as usize;
    let track_h = if has_bar { track.saturating_sub(1) } else { track };
    let bar_y_offset = if has_bar { 1 } else { 0 };
    if let Some((pos, len)) = scroll_thumb(rows.len(), model.scroll_first, list_h, track_h) {
        let bar_style = if model.focus == Focus::List {
            theme.focus_style
        } else {
            theme.listborder
        };
        for i in 0..len {
            let y = area.y + bar_y_offset + (pos + i) as u16;
            let bar_cell = Rect::new(area.x + area.width - 1, y, 1, 1);
            f.render_widget(Paragraph::new(" ").style(bar_style), bar_cell);
        }
    }
}
