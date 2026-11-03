use crate::nav::Focus;
use crate::uiconf::{LayoutMode, ListPosition, UiConfig};
use ratatui::layout::Rect;

pub struct Layout {
    pub top: Rect,
    pub status: Rect,
    pub help: Rect,
    pub list: Rect,
    pub border: Rect,
    pub history: Rect,
    pub entry: Rect,
}

impl Layout {
    pub fn compute(term_w: u16, term_h: u16, conf: &UiConfig) -> Self {
        let top_h = if conf.top_enabled { 1 } else { 0 };
        let help_h = if conf.help_enabled { 1 } else { 0 };
        let status_h = if conf.status_enabled { 1 } else { 0 };

        // Entry content height calculation: max(1, min(entry_height, term_h / 2)) + 1 border line
        let entry_lines = (conf.entry_height as u16).min(term_h / 2).max(1);
        let entry_h = entry_lines + 1; // including border

        let mid_h = term_h.saturating_sub(top_h + help_h + entry_h + status_h);
        let body_h = term_h.saturating_sub(top_h + help_h);

        let list_w = if conf.list_enabled { conf.list_width.min(term_w) } else { 0 };
        let border_w = if conf.list_enabled && list_w > 0 && list_w < term_w { 1 } else { 0 };
        let col_w = term_w.saturating_sub(list_w + border_w);

        let is_right = conf.list_position == ListPosition::Right;
        let col_x = if is_right { 0 } else { list_w + border_w };
        let border_x = if is_right { col_w } else { list_w };
        let list_x = if is_right { col_w + border_w } else { 0 };

        let (low_x, low_w, side_h) = match conf.layout_mode {
            LayoutMode::Sidebar => (col_x, col_w, body_h),
            LayoutMode::Classic => (0, term_w, mid_h),
        };

        let top = Rect::new(0, 0, if conf.top_enabled { term_w } else { 0 }, top_h);
        let help = Rect::new(
            0,
            term_h.saturating_sub(help_h),
            if conf.help_enabled { term_w } else { 0 },
            help_h,
        );

        let entry_y = term_h.saturating_sub(help_h + entry_h);
        let status_y = term_h.saturating_sub(help_h + entry_h + status_h);

        let entry = Rect::new(low_x, entry_y, low_w, entry_h);
        let status = Rect::new(
            low_x,
            status_y,
            if conf.status_enabled { low_w } else { 0 },
            status_h,
        );

        let list = Rect::new(list_x, top_h, list_w, side_h);
        let border = Rect::new(border_x, top_h, border_w, side_h);
        let history = Rect::new(col_x, top_h, col_w, mid_h);

        Layout {
            top,
            status,
            help,
            list,
            border,
            history,
            entry,
        }
    }

    pub fn pane_at(&self, x: u16, y: u16) -> Option<Focus> {
        let inside = |r: &Rect| x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height;

        if inside(&self.list) {
            return Some(Focus::List);
        }
        if inside(&self.history) {
            return Some(Focus::History);
        }
        if inside(&self.entry) {
            return Some(Focus::Entry);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_sidebar_layout_rects() {
        let conf = UiConfig {
            layout_mode: LayoutMode::Sidebar,
            list_enabled: true,
            list_position: ListPosition::Left,
            list_width: 30,
            entry_height: 4,
            top_enabled: true,
            status_enabled: true,
            help_enabled: true,
            ..Default::default()
        };

        let l = Layout::compute(100, 40, &conf);
        assert_eq!(l.top, Rect::new(0, 0, 100, 1));
        assert_eq!(l.help, Rect::new(0, 39, 100, 1));
        assert_eq!(l.list, Rect::new(0, 1, 30, 38)); // height = 40 - 1 - 1
        assert_eq!(l.border, Rect::new(30, 1, 1, 38));
        // history & entry on right column (x = 31, w = 69)
        assert_eq!(l.entry.x, 31);
        assert_eq!(l.entry.width, 69);
        assert_eq!(l.status.x, 31);
        assert_eq!(l.history.x, 31);
    }

    #[test]
    fn hit_tests_correct_pane() {
        let conf = UiConfig::default();
        let l = Layout::compute(100, 40, &conf);
        assert_eq!(l.pane_at(10, 10), Some(Focus::List));
        assert_eq!(l.pane_at(50, 10), Some(Focus::History));
    }
}
