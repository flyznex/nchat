pub mod emoji;
pub mod file;
pub mod mention;
pub mod message;
pub mod sticker;

use crate::colorconf::Theme;
use crate::keyconf::Key;
use ratatui::layout::Rect;
use ratatui::Frame;

#[derive(Debug, Clone, PartialEq)]
pub enum DialogResult {
    KeepOpen,
    Close(bool),
    CloseValue(String),
}

pub trait Dialog {
    fn handle_key(&mut self, key: Key) -> DialogResult;
    fn render(
        &mut self,
        f: &mut Frame,
        area: Rect,
        theme: &Theme,
        img_cache: &mut crate::imgcache::ImageCache,
        model: &crate::model::UiModel,
    );
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_w = (r.width * percent_x) / 100;
    let popup_h = (r.height * percent_y) / 100;
    let x = r.x + (r.width.saturating_sub(popup_w)) / 2;
    let y = r.y + (r.height.saturating_sub(popup_h)) / 2;
    Rect::new(x, y, popup_w, popup_h)
}
