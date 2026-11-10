use crate::colorconf::Theme;
use crate::history::format_message;
use crate::imgcache::ImageCache;
use crate::markdown::StyleFlags;
use crate::model::UiModel;
use crate::nav::Focus;
use crate::uiconf::UiConfig;
use ratatui::layout::{Margin, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::Frame;
use ratatui_image::StatefulImage;

struct ImageMarker {
    line_idx: usize,
    rows: usize,
    x0: usize,
    w: usize,
    path: String,
}

pub fn render_history(
    f: &mut Frame,
    area: Rect,
    model: &UiModel,
    theme: &Theme,
    conf: &UiConfig,
    img_cache: &mut ImageCache,
) {
    if area.width < 4 || area.height < 3 {
        return;
    }

    let border_style = if model.focus == Focus::History {
        theme.focus_style
    } else {
        theme.listborder
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let (prof, cid) = match &model.selected_id {
        Some(pair) => pair,
        None => {
            let p = Paragraph::new(" No chat selected").style(theme.history_text_recv);
            f.render_widget(p, inner);
            return;
        }
    };

    let messages = model.messages(prof, cid);
    if messages.is_empty() {
        let p = Paragraph::new(" No messages in this chat").style(theme.history_text_recv);
        f.render_widget(p, inner);
        return;
    }

    let content_w = inner.width as usize;
    let height = inner.height as usize;
    let total_msgs = messages.len();

    let sel_idx = total_msgs.saturating_sub(1 + model.message_offset);
    let selection_active = model.focus == Focus::History || model.select_message_active;

    let mut all_lines: Vec<Line> = Vec::new();
    let mut image_markers: Vec<ImageMarker> = Vec::new();
    let mut msg_line_ranges: Vec<(usize, usize)> = Vec::with_capacity(total_msgs);

    for (idx, msg) in messages.iter().enumerate() {
        let msg_start_line = all_lines.len();

        let sender_name = if msg.is_outgoing {
            "You".to_string()
        } else {
            let mut name = model.contact_name(prof, &msg.sender_id);
            if name.is_empty() || name == msg.sender_id {
                if let Some(chat) = model.selected_chat() {
                    if !chat.chat_id.starts_with('-') || chat.chat_id == msg.sender_id || msg.sender_id.is_empty() {
                        if !chat.name.is_empty() {
                            name = chat.name.clone();
                        }
                    }
                }
            }
            name
        };

        let fmt = format_message(msg, &sender_name, content_w, conf.sent_align_right, conf);
        let is_selected = selection_active && idx == sel_idx;

        let name_style = if is_selected {
            theme.history_text_selected
        } else if fmt.is_outgoing {
            theme.history_name_sent
        } else {
            theme.history_name_recv
        };

        let text_style = if is_selected {
            theme.history_text_selected
        } else if fmt.is_outgoing {
            theme.history_text_sent
        } else {
            theme.history_text_recv
        };

        let pad_block = |text: &str| -> String {
            let w = crate::text::width(text);
            let pad = fmt.block_w.saturating_sub(w);
            format!("{text}{}", " ".repeat(pad))
        };

        let header_str = if is_selected {
            pad_block(&fmt.header)
        } else {
            fmt.header.clone()
        };
        if fmt.x0 > 0 {
            all_lines.push(Line::from(vec![
                Span::raw(" ".repeat(fmt.x0)),
                Span::styled(header_str, name_style),
            ]));
        } else {
            all_lines.push(Line::from(Span::styled(header_str, name_style)));
        }

        if let Some(q) = fmt.quote {
            let quote_str = if is_selected {
                pad_block(&q)
            } else {
                q
            };
            let quote_style = if is_selected {
                theme.history_text_selected
            } else {
                theme.history_text_quoted
            };
            if fmt.x0 > 0 {
                all_lines.push(Line::from(vec![
                    Span::raw(" ".repeat(fmt.x0)),
                    Span::styled(quote_str, quote_style),
                ]));
            } else {
                all_lines.push(Line::from(Span::styled(quote_str, quote_style)));
            }
        }

        if let Some(path) = fmt.image_path {
            let img_rows = fmt.image_rows.min(height / 2).max(1);
            let img_w = fmt.block_w.min(content_w).max(4);
            let img_start = all_lines.len();

            for r in 0..img_rows {
                if r == 0 {
                    let label = if is_selected {
                        pad_block("[Image]")
                    } else {
                        "[Image]".to_string()
                    };
                    let img_lbl_style = if is_selected {
                        theme.history_text_selected
                    } else {
                        theme.history_text_quoted
                    };
                    if fmt.x0 > 0 {
                        all_lines.push(Line::from(vec![
                            Span::raw(" ".repeat(fmt.x0)),
                            Span::styled(label, img_lbl_style),
                        ]));
                    } else {
                        all_lines.push(Line::from(Span::styled(label, img_lbl_style)));
                    }
                } else {
                    all_lines.push(Line::from(""));
                }
            }

            image_markers.push(ImageMarker {
                line_idx: img_start,
                rows: img_rows,
                x0: fmt.x0,
                w: img_w,
                path,
            });
        }

        for l in fmt.lines {
            let mut spans = Vec::new();
            if fmt.x0 > 0 {
                spans.push(Span::raw(" ".repeat(fmt.x0)));
            }
            for s in &l.spans {
                let mut style = text_style;
                if s.style.contains(StyleFlags::BOLD) {
                    style = style.add_modifier(ratatui::style::Modifier::BOLD);
                }
                if s.style.contains(StyleFlags::ITALIC) {
                    style = style.add_modifier(ratatui::style::Modifier::ITALIC);
                }
                if s.style.contains(StyleFlags::STRIKE) {
                    style = style.add_modifier(ratatui::style::Modifier::CROSSED_OUT);
                }
                if s.style.contains(StyleFlags::UNDERLINE) || s.style.contains(StyleFlags::LINK) {
                    style = style.add_modifier(ratatui::style::Modifier::UNDERLINED);
                }
                if s.style.contains(StyleFlags::CODE) {
                    style = style.add_modifier(ratatui::style::Modifier::REVERSED);
                }
                if s.style.contains(StyleFlags::CODE_BLOCK) {
                    style = theme.history_text_quoted.add_modifier(ratatui::style::Modifier::DIM);
                }
                let text = if s.style.contains(StyleFlags::SPOILER) && is_selected {
                    // Revealed by selecting the message: show the real text, dimmed/reversed.
                    style = style.add_modifier(ratatui::style::Modifier::DIM | ratatui::style::Modifier::REVERSED);
                    s.text.clone()
                } else if s.style.contains(StyleFlags::SPOILER) {
                    // Hidden until the message is selected.
                    style = style.add_modifier(ratatui::style::Modifier::REVERSED);
                    "[Spoiler]".to_string()
                } else {
                    s.text.clone()
                };
                spans.push(Span::styled(text, style));
            }
            if is_selected {
                let pad = fmt.block_w.saturating_sub(l.width);
                if pad > 0 {
                    spans.push(Span::styled(" ".repeat(pad), text_style));
                }
            }
            all_lines.push(Line::from(spans));
        }

        let msg_end_line = all_lines.len();
        msg_line_ranges.push((msg_start_line, msg_end_line));

        all_lines.push(Line::from(""));
    }

    let total_lines = all_lines.len();
    let visible_lines: Vec<Line>;
    let start_line_idx: usize;

    if total_lines <= height {
        let pad_count = height - total_lines;
        let mut padded = Vec::with_capacity(height);
        for _ in 0..pad_count {
            padded.push(Line::from(""));
        }
        padded.extend(all_lines);
        visible_lines = padded;
        start_line_idx = 0;
    } else {
        let max_scroll = total_lines - height;
        let target_start = if selection_active {
            if model.message_offset == 0 {
                max_scroll
            } else if let Some(&(sel_start, sel_end)) = msg_line_ranges.get(sel_idx) {
                if sel_end + 1 > height {
                    let bottom_anchor = (sel_end + 1).saturating_sub(height).min(max_scroll);
                    if sel_start < bottom_anchor {
                        sel_start.min(max_scroll)
                    } else {
                        bottom_anchor
                    }
                } else {
                    0
                }
            } else {
                max_scroll
            }
        } else {
            max_scroll
        };
        start_line_idx = target_start;
        let end_line_idx = (start_line_idx + height).min(total_lines);
        visible_lines = all_lines[start_line_idx..end_line_idx].to_vec();
    }

    let p = Paragraph::new(visible_lines);
    f.render_widget(p, inner);

    let pad_top = if total_lines <= height { height - total_lines } else { 0 };
    for marker in image_markers {
        if total_lines <= height {
            let screen_line = marker.line_idx + pad_top;
            if screen_line < height {
                let draw_rows = marker.rows.min(height - screen_line);
                let img_rect = Rect::new(
                    inner.x + marker.x0 as u16,
                    inner.y + screen_line as u16,
                    marker.w as u16,
                    draw_rows as u16,
                );
                if let Some(state) = img_cache.get_or_request(&marker.path) {
                    f.render_stateful_widget(StatefulImage::default(), img_rect, state);
                }
            }
        } else {
            let marker_end = marker.line_idx + marker.rows;
            if marker_end > start_line_idx && marker.line_idx < start_line_idx + height {
                let draw_start = marker.line_idx.max(start_line_idx);
                let draw_end = marker_end.min(start_line_idx + height);
                let screen_y = (draw_start - start_line_idx) as u16;
                let draw_rows = (draw_end - draw_start) as u16;
                let img_rect = Rect::new(
                    inner.x + marker.x0 as u16,
                    inner.y + screen_y,
                    marker.w as u16,
                    draw_rows,
                );
                if let Some(state) = img_cache.get_or_request(&marker.path) {
                    f.render_stateful_widget(StatefulImage::default(), img_rect, state);
                }
            }
        }
    }

    if total_lines > height {
        let max_scroll = total_lines - height;
        let mut state = ScrollbarState::new(max_scroll.max(1)).position(start_line_idx);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight).begin_symbol(None).end_symbol(None),
            area.inner(Margin { vertical: 1, horizontal: 0 }),
            &mut state,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colorconf::Theme;
    use crate::model::UiModel;
    use crate::protocol::{Chat, Message};
    use crate::uiconf::UiConfig;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::Terminal;

    #[test]
    fn renders_history_messages_bottom_up() {
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut model = UiModel::new();
        model.apply_event(&crate::protocol::Event::Chats {
            profile: "P1".into(), success: true,
            chats: vec![Chat {
                id: "c1".into(), is_unread: false, is_unread_mention: false,
                is_muted: false, is_pinned: false, is_archived: false, last_message_time: 10,
            }],
        });
        let msg = Message {
            id: "m1".into(), sender_id: "u1".into(), text: "Hello Ratatui".into(),
            quoted_id: "".into(), quoted_text: "".into(), quoted_sender: "".into(),
            file_info: "".into(), link_preview: "".into(), time_sent: 1700000000000,
            is_outgoing: false, is_read: true, is_delivered: true, is_edited: false,
            is_deleted: false, is_pinned: false, has_mention: false,
        };
        model.apply_event(&crate::protocol::Event::Messages {
            profile: "P1".into(), success: true, chat_id: "c1".into(),
            from_msg_id: "".into(), cached: false, sequence: false,
            messages: vec![msg],
        });

        let theme = Theme::default();
        let conf = UiConfig::default();
        let mut img_cache = ImageCache::new();

        terminal.draw(|f| {
            render_history(f, Rect::new(0, 0, 40, 10), &model, &theme, &conf, &mut img_cache);
        }).unwrap();

        let buf = terminal.backend().buffer();
        let rendered: String = (0..10).map(|y| {
            (0..40).map(|x| buf[(x, y)].symbol()).collect::<String>() + "\n"
        }).collect();

        assert!(rendered.contains("Hello Ratatui"));
    }

    #[test]
    fn renders_square_border() {
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut model = UiModel::new();
        model.apply_event(&crate::protocol::Event::Chats {
            profile: "P1".into(), success: true,
            chats: vec![Chat {
                id: "c1".into(), is_unread: false, is_unread_mention: false,
                is_muted: false, is_pinned: false, is_archived: false, last_message_time: 10,
            }],
        });
        let msg = Message {
            id: "m1".into(), sender_id: "u1".into(), text: "Hi".into(),
            quoted_id: "".into(), quoted_text: "".into(), quoted_sender: "".into(),
            file_info: "".into(), link_preview: "".into(), time_sent: 1700000000000,
            is_outgoing: false, is_read: true, is_delivered: true, is_edited: false,
            is_deleted: false, is_pinned: false, has_mention: false,
        };
        model.apply_event(&crate::protocol::Event::Messages {
            profile: "P1".into(), success: true, chat_id: "c1".into(),
            from_msg_id: "".into(), cached: false, sequence: false,
            messages: vec![msg],
        });

        let theme = Theme::default();
        let conf = UiConfig::default();
        let mut img_cache = ImageCache::new();

        terminal.draw(|f| {
            render_history(f, Rect::new(0, 0, 40, 10), &model, &theme, &conf, &mut img_cache);
        }).unwrap();

        let buf = terminal.backend().buffer();
        assert_eq!(buf[(0, 0)].symbol(), "┌");
        assert_eq!(buf[(39, 0)].symbol(), "┐");
    }

    #[test]
    fn highlights_selected_message_when_history_focused() {
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut model = UiModel::new();
        model.apply_event(&crate::protocol::Event::Chats {
            profile: "P1".into(), success: true,
            chats: vec![Chat {
                id: "c1".into(), is_unread: false, is_unread_mention: false,
                is_muted: false, is_pinned: false, is_archived: false, last_message_time: 10,
            }],
        });
        let msg = Message {
            id: "m1".into(), sender_id: "u1".into(), text: "HighlightMe".into(),
            quoted_id: "".into(), quoted_text: "".into(), quoted_sender: "".into(),
            file_info: "".into(), link_preview: "".into(), time_sent: 1700000000000,
            is_outgoing: false, is_read: true, is_delivered: true, is_edited: false,
            is_deleted: false, is_pinned: false, has_mention: false,
        };
        model.apply_event(&crate::protocol::Event::Messages {
            profile: "P1".into(), success: true, chat_id: "c1".into(),
            from_msg_id: "".into(), cached: false, sequence: false,
            messages: vec![msg],
        });

        // Set focus to History so selection is active
        model.focus = Focus::History;

        let theme = Theme::default();
        let conf = UiConfig::default();
        let mut img_cache = ImageCache::new();

        terminal.draw(|f| {
            render_history(f, Rect::new(0, 0, 40, 10), &model, &theme, &conf, &mut img_cache);
        }).unwrap();

        let buf = terminal.backend().buffer();
        // Find the cell with 'H' from 'HighlightMe'
        let mut found_reversed = false;
        for y in 0..10 {
            for x in 0..40 {
                let cell = &buf[(x, y)];
                if cell.symbol() == "H" && cell.modifier.contains(ratatui::style::Modifier::REVERSED) {
                    found_reversed = true;
                    break;
                }
            }
        }
        assert!(found_reversed, "expected selected message cell to be highlighted with REVERSED modifier");
    }
}
