use crate::conf::Conf;
use ratatui::style::{Color, Modifier, Style};

pub fn parse_color(val: &str) -> Option<Color> {
    let s = val.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("usercolor") {
        return None;
    }
    match s.to_ascii_lowercase().as_str() {
        "black" => Some(Color::Indexed(0)),
        "red" => Some(Color::Indexed(1)),
        "green" => Some(Color::Indexed(2)),
        "yellow" => Some(Color::Indexed(3)),
        "blue" => Some(Color::Indexed(4)),
        "magenta" => Some(Color::Indexed(5)),
        "cyan" => Some(Color::Indexed(6)),
        "white" => Some(Color::Indexed(7)),
        "gray" | "bright_black" => Some(Color::Indexed(8)),
        "bright_red" => Some(Color::Indexed(9)),
        "bright_green" => Some(Color::Indexed(10)),
        "bright_yellow" => Some(Color::Indexed(11)),
        "bright_blue" => Some(Color::Indexed(12)),
        "bright_magenta" => Some(Color::Indexed(13)),
        "bright_cyan" => Some(Color::Indexed(14)),
        "bright_white" => Some(Color::Indexed(15)),
        other => {
            if let Some(hex) = other.strip_prefix("0x").or_else(|| other.strip_prefix('#')) {
                if hex.len() == 6 {
                    if let Ok(rgb) = u32::from_str_radix(hex, 16) {
                        return Some(Color::Rgb(
                            ((rgb >> 16) & 0xff) as u8,
                            ((rgb >> 8) & 0xff) as u8,
                            (rgb & 0xff) as u8,
                        ));
                    }
                }
            }
            if let Ok(idx) = other.parse::<u8>() {
                return Some(Color::Indexed(idx));
            }
            None
        }
    }
}

pub fn parse_modifier(val: &str) -> Modifier {
    let mut modifier = Modifier::empty();
    for token in val.split(|c| c == ',' || c == ' ') {
        match token.trim().to_ascii_lowercase().as_str() {
            "bold" => modifier.insert(Modifier::BOLD),
            "dim" => modifier.insert(Modifier::DIM),
            "italic" => modifier.insert(Modifier::ITALIC),
            "underline" => modifier.insert(Modifier::UNDERLINED),
            "reverse" => modifier.insert(Modifier::REVERSED),
            _ => {}
        }
    }
    modifier
}

const CONTACT_PALETTE: [Color; 10] = [
    Color::Rgb(230, 126, 34),
    Color::Rgb(46, 204, 113),
    Color::Rgb(52, 152, 219),
    Color::Rgb(155, 89, 182),
    Color::Rgb(241, 196, 15),
    Color::Rgb(231, 76, 60),
    Color::Rgb(26, 188, 156),
    Color::Rgb(243, 156, 18),
    Color::Rgb(93, 173, 226),
    Color::Rgb(175, 122, 197),
];

pub fn contact_color(key: &str) -> Color {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in key.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    CONTACT_PALETTE[(hash as usize) % CONTACT_PALETTE.len()]
}

fn make_style(conf: &Conf, prefix: &str) -> Style {
    let mut style = Style::default();
    let fg_key = format!("{prefix}_color_fg");
    let bg_key = format!("{prefix}_color_bg");
    let attr_key = format!("{prefix}_attr");

    if let Some(fg) = conf.get(&fg_key).and_then(parse_color) {
        style = style.fg(fg);
    }
    if let Some(bg) = conf.get(&bg_key).and_then(parse_color) {
        style = style.bg(bg);
    }
    let m = parse_modifier(conf.str_or(&attr_key, ""));
    if !m.is_empty() {
        style = style.add_modifier(m);
    }
    style
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub top: Style,
    pub status: Style,
    pub list: Style,
    pub list_selected: Style,
    pub list_name: Style,
    pub list_preview: Style,
    pub list_unread: Style,
    pub listborder: Style,
    pub dim_modifier: Modifier,
    pub focus_style: Style,
    pub history_text_sent: Style,
    pub history_text_recv: Style,
    pub history_name_sent: Style,
    pub history_name_recv: Style,
    pub history_text_quoted: Style,
    pub history_text_selected: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            top: Style::default().add_modifier(Modifier::REVERSED),
            status: Style::default().add_modifier(Modifier::REVERSED),
            list: Style::default(),
            list_selected: Style::default().add_modifier(Modifier::REVERSED),
            list_name: Style::default().add_modifier(Modifier::BOLD),
            list_preview: Style::default().fg(Color::Indexed(8)),
            list_unread: Style::default(),
            listborder: Style::default(),
            dim_modifier: Modifier::DIM,
            focus_style: Style::default().add_modifier(Modifier::REVERSED),
            history_text_sent: Style::default(),
            history_text_recv: Style::default(),
            history_name_sent: Style::default().add_modifier(Modifier::BOLD),
            history_name_recv: Style::default().add_modifier(Modifier::BOLD),
            history_text_quoted: Style::default().fg(Color::Indexed(8)),
            history_text_selected: Style::default().add_modifier(Modifier::REVERSED),
        }
    }
}

impl Theme {
    pub fn from_conf(conf: &Conf) -> Self {
        let mut t = Self::default();
        t.top = make_style(conf, "top");
        if t.top == Style::default() {
            t.top = t.top.add_modifier(Modifier::REVERSED);
        }
        t.status = make_style(conf, "status");
        if t.status == Style::default() {
            t.status = t.status.add_modifier(Modifier::REVERSED);
        }
        t.list = make_style(conf, "list");
        t.list_selected = t.list;
        if let Some(fg) = conf
            .get("list_color_selected_fg")
            .and_then(parse_color)
        {
            t.list_selected = t.list_selected.fg(fg);
        }
        if let Some(bg) = conf
            .get("list_color_selected_bg")
            .and_then(parse_color)
        {
            t.list_selected = t.list_selected.bg(bg);
        }
        let selected_modifier = parse_modifier(conf.str_or("list_attr_selected", "reverse"));
        if !selected_modifier.is_empty() {
            t.list_selected = t.list_selected.add_modifier(selected_modifier);
        }
        t.list_name = make_style(conf, "list_name");
        if t.list_name == Style::default() {
            t.list_name = t.list_name.add_modifier(Modifier::BOLD);
        }
        t.list_preview = make_style(conf, "list_preview");
        if t.list_preview.fg.is_none() {
            t.list_preview = t.list_preview.fg(Color::Indexed(8));
        }
        t.list_unread = Style::default();
        if let Some(fg) = conf.get("list_color_unread_fg").and_then(parse_color) {
            t.list_unread = t.list_unread.fg(fg);
        }
        if let Some(bg) = conf.get("list_color_unread_bg").and_then(parse_color) {
            t.list_unread = t.list_unread.bg(bg);
        }
        let unread_attr = conf
            .get("list_color_unread_attr")
            .or_else(|| conf.get("list_attr_unread"))
            .unwrap_or("");
        let unread_modifier = parse_modifier(unread_attr);
        if !unread_modifier.is_empty() {
            t.list_unread = t.list_unread.add_modifier(unread_modifier);
        }
        t.listborder = make_style(conf, "listborder");
        t.dim_modifier = parse_modifier(conf.str_or("dim_attr", "dim"));
        t.focus_style = make_style(conf, "focus");
        if t.focus_style == Style::default() {
            t.focus_style = t.focus_style.add_modifier(Modifier::REVERSED);
        }

        t.history_text_sent = make_style(conf, "history_text_sent");
        t.history_text_recv = make_style(conf, "history_text_recv");
        t.history_name_sent = make_style(conf, "history_name_sent");
        if t.history_name_sent.add_modifier.is_empty() {
            t.history_name_sent = t.history_name_sent.add_modifier(Modifier::BOLD);
        }
        t.history_name_recv = make_style(conf, "history_name_recv");
        if t.history_name_recv.add_modifier.is_empty() {
            t.history_name_recv = t.history_name_recv.add_modifier(Modifier::BOLD);
        }
        t.history_text_quoted = make_style(conf, "history_text_quoted");
        if t.history_text_quoted.fg.is_none() {
            t.history_text_quoted = t.history_text_quoted.fg(Color::Indexed(8));
        }
        t.history_text_selected = make_style(conf, "history_text");
        if let Some(fg) = conf
            .get("history_text_color_selected_fg")
            .or_else(|| conf.get("history_text_selected_color_fg"))
            .and_then(parse_color)
        {
            t.history_text_selected = t.history_text_selected.fg(fg);
        }
        if let Some(bg) = conf
            .get("history_text_color_selected_bg")
            .or_else(|| conf.get("history_text_selected_color_bg"))
            .and_then(parse_color)
        {
            t.history_text_selected = t.history_text_selected.bg(bg);
        }
        let sel_attr = parse_modifier(conf.str_or("history_text_attr_selected", "reverse"));
        if !sel_attr.is_empty() {
            t.history_text_selected = t.history_text_selected.add_modifier(sel_attr);
        }

        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_color_names() {
        assert_eq!(parse_color("black"), Some(Color::Indexed(0)));
        assert_eq!(parse_color("red"), Some(Color::Indexed(1)));
        assert_eq!(parse_color("gray"), Some(Color::Indexed(8)));
        assert_eq!(parse_color("bright_black"), Some(Color::Indexed(8)));
        assert_eq!(parse_color("white"), Some(Color::Indexed(7)));
        assert_eq!(parse_color(""), None);
    }

    #[test]
    fn parses_hex_and_indexed_colors() {
        assert_eq!(parse_color("0x112233"), Some(Color::Rgb(0x11, 0x22, 0x33)));
        assert_eq!(parse_color("240"), Some(Color::Indexed(240)));
    }

    #[test]
    fn parses_modifiers() {
        assert_eq!(parse_modifier("reverse"), Modifier::REVERSED);
        assert_eq!(parse_modifier("bold,underline"), Modifier::BOLD | Modifier::UNDERLINED);
    }

    #[test]
    fn loads_theme_styles() {
        let conf = Conf::parse("
            top_attr=reverse
            list_name_attr=bold
            list_preview_color_fg=gray
        ");
        let theme = Theme::from_conf(&conf);
        assert_eq!(theme.top.add_modifier, Modifier::REVERSED);
        assert_eq!(theme.list_name.add_modifier, Modifier::BOLD);
        assert_eq!(theme.list_preview.fg, Some(Color::Indexed(8)));
    }

    #[test]
    fn list_attr_selected_sets_selected_modifier() {
        let conf = Conf::parse("list_attr_selected=reverse");
        let theme = Theme::from_conf(&conf);
        assert_eq!(theme.list_selected.add_modifier, Modifier::REVERSED);
    }

    #[test]
    fn list_color_unread_fg_sets_unread_fg() {
        let conf = Conf::parse("list_color_unread_fg=red");
        let theme = Theme::from_conf(&conf);
        assert_eq!(theme.list_unread.fg, Some(Color::Indexed(1)));
    }

    #[test]
    fn contact_color_is_deterministic() {
        assert_eq!(contact_color("chat_123"), contact_color("chat_123"));
    }

    #[test]
    fn contact_color_spreads_across_palette() {
        let colors: std::collections::HashSet<Color> =
            (0..30).map(|i| contact_color(&format!("chat_{i}"))).collect();
        assert!(colors.len() > 1, "expected more than one distinct color across 30 keys");
    }
}
