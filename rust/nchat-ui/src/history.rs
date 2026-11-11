use crate::protocol::Message;
use crate::render::time::format_list_time;
use crate::text::{fit, width};
use crate::uiconf::UiConfig;

pub use crate::markdown::{FormattedLine, FormattedSpan};

#[derive(Debug, Clone, PartialEq)]
pub struct FormattedMessage {
    pub id: String,
    pub header: String,
    pub lines: Vec<FormattedLine>,
    pub quote: Option<String>,
    pub image_path: Option<String>,
    pub image_rows: usize,
    pub block_w: usize,
    pub x0: usize,
    pub is_outgoing: bool,
}

pub fn format_message(
    msg: &Message,
    sender_name: &str,
    max_pane_w: usize,
    align_sent: bool,
    _conf: &UiConfig,
) -> FormattedMessage {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let time_str = format_list_time(msg.time_sent, now);
    let mut header = if !time_str.is_empty() {
        format!("{sender_name} ({time_str})")
    } else {
        sender_name.to_string()
    };

    if msg.is_outgoing {
        if msg.is_read {
            header.push_str(" ✓✓");
        } else {
            header.push_str(" ✓");
        }
    }
    if msg.is_edited {
        header.push_str(" ✎");
    }
    if msg.is_pinned {
        header.push_str(" 📌");
    }

    let max_w = if align_sent {
        (max_pane_w * 3 / 4).max(1)
    } else {
        max_pane_w
    };

    let quote = if !msg.quoted_text.is_empty() {
        let first_line = msg.quoted_text.lines().next().unwrap_or("");
        let truncated = fit(first_line, max_w.saturating_sub(4));
        Some(format!("> {truncated}"))
    } else {
        None
    };

    let (image_path, image_rows, display_text) = if !msg.file_info.is_empty() {
        if let Some(info) = crate::fileinfo::parse_file_info(&msg.file_info) {
            if info.is_downloaded() && info.is_image() {
                (Some(info.file_path), 10, msg.text.as_str())
            } else {
                (None, 0, msg.text.as_str())
            }
        } else {
            (None, 0, msg.text.as_str())
        }
    } else if msg.text.starts_with("[file:") && msg.text.ends_with(']') {
        let path = &msg.text[6..msg.text.len() - 1];
        if std::path::Path::new(path).is_file() {
            let lower = path.to_lowercase();
            if lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".png") || lower.ends_with(".webp") || lower.ends_with(".webm") {
                (Some(path.to_string()), 10, "")
            } else {
                (None, 0, msg.text.as_str())
            }
        } else {
            (None, 0, msg.text.as_str())
        }
    } else {
        (None, 0, msg.text.as_str())
    };

    let lines = if display_text.is_empty() {
        Vec::new()
    } else {
        crate::markdown::wrap_markdown(display_text, max_w)
    };

    let mut block_w = width(&header);
    if let Some(q) = &quote {
        block_w = block_w.max(width(q));
    }
    for l in &lines {
        block_w = block_w.max(l.width);
    }
    if image_path.is_some() {
        block_w = block_w.max(24.min(max_w));
    }
    block_w = block_w.min(max_w).max(1);

    let x0 = if align_sent && msg.is_outgoing {
        max_pane_w.saturating_sub(block_w)
    } else {
        0
    };

    FormattedMessage {
        id: msg.id.clone(),
        header,
        lines,
        quote,
        image_path,
        image_rows,
        block_w,
        x0,
        is_outgoing: msg.is_outgoing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_message_produces_formatted_spans_for_markdown() {
        let msg = Message {
            id: "m_md".into(), sender_id: "alice".into(), text: "Hello *world* and `code`".into(),
            quoted_id: "".into(), quoted_text: "".into(), quoted_sender: "".into(),
            file_info: "".into(), link_preview: "".into(), time_sent: 1700000000000,
            is_outgoing: false, is_read: true, is_delivered: true, is_edited: false,
            is_deleted: false, is_pinned: false, has_mention: false,
            buttons: Vec::new(),
        };
        let conf = UiConfig::default();
        let fmt = format_message(&msg, "Alice", 40, false, &conf);
        assert!(!fmt.lines.is_empty());
        let spans = &fmt.lines[0].spans;
        assert_eq!(spans[0].text, "Hello ");
        assert_eq!(spans[1].text, "world");
        assert!(spans[1].style.contains(crate::markdown::StyleFlags::BOLD));
    }

    #[test]
    fn formats_outgoing_message_read_and_unread_indicators() {
        let mut msg = Message {
            id: "m1".into(), sender_id: "me".into(), text: "hello".into(),
            quoted_id: "".into(), quoted_text: "".into(), quoted_sender: "".into(),
            file_info: "".into(), link_preview: "".into(), time_sent: 1700000000000,
            is_outgoing: true, is_read: false, is_delivered: true, is_edited: false,
            is_deleted: false, is_pinned: false, has_mention: false,
            buttons: Vec::new(),
        };
        let conf = UiConfig::default();
        let unread_fmt = format_message(&msg, "Me", 40, true, &conf);
        assert!(unread_fmt.header.contains("✓"));
        assert!(!unread_fmt.header.contains("✓✓"));

        msg.is_read = true;
        let read_fmt = format_message(&msg, "Me", 40, true, &conf);
        assert!(read_fmt.header.contains("✓✓"));
    }

    #[test]
    fn formats_incoming_message_aligned_left() {
        let msg = Message {
            id: "m1".into(), sender_id: "alice".into(), text: "hello world".into(),
            quoted_id: "".into(), quoted_text: "".into(), quoted_sender: "".into(),
            file_info: "".into(), link_preview: "".into(), time_sent: 1700000000000,
            is_outgoing: false, is_read: true, is_delivered: true, is_edited: false,
            is_deleted: false, is_pinned: false, has_mention: false,
            buttons: Vec::new(),
        };
        let conf = UiConfig::default();
        let fmt = format_message(&msg, "Alice", 40, true, &conf);
        assert_eq!(fmt.x0, 0); // left aligned
        assert!(!fmt.is_outgoing);
        assert!(fmt.header.contains("Alice"));
    }

    #[test]
    fn formats_outgoing_message_aligned_right() {
        let msg = Message {
            id: "m2".into(), sender_id: "me".into(), text: "short".into(),
            quoted_id: "".into(), quoted_text: "".into(), quoted_sender: "".into(),
            file_info: "".into(), link_preview: "".into(), time_sent: 1700000000000,
            is_outgoing: true, is_read: true, is_delivered: true, is_edited: false,
            is_deleted: false, is_pinned: false, has_mention: false,
            buttons: Vec::new(),
        };
        let conf = UiConfig::default();
        let fmt = format_message(&msg, "Me", 40, true, &conf);
        assert!(fmt.is_outgoing);
        assert!(fmt.x0 > 0); // pushed right
        assert_eq!(fmt.x0 + fmt.block_w, 40);
        assert!(fmt.header.contains("✓")); // delivery indicator
    }

    #[test]
    fn wraps_long_text_and_clamps_max_width() {
        let msg = Message {
            id: "m3".into(), sender_id: "me".into(),
            text: "This is a very long line of text that should wrap across several lines nicely".into(),
            quoted_id: "".into(), quoted_text: "".into(), quoted_sender: "".into(),
            file_info: "".into(), link_preview: "".into(), time_sent: 1700000000000,
            is_outgoing: true, is_read: false, is_delivered: true, is_edited: true,
            is_deleted: false, is_pinned: false, has_mention: false,
            buttons: Vec::new(),
        };
        let conf = UiConfig::default();
        let fmt = format_message(&msg, "Me", 40, true, &conf);
        assert!(fmt.lines.len() > 1);
        assert!(fmt.block_w <= 30); // 75% of 40 = 30
        assert!(fmt.header.contains("✎")); // edited indicator
    }
}
