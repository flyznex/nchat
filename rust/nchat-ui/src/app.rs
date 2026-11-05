use crate::colorconf::Theme;
use crate::layout::Layout;
use crate::model::UiModel;
use crate::nav::{Action, Focus, NavResolveResult, NavResolver};
use crate::protocol::Event;
use crate::render::{entry, help, history, list, status, top};
use crate::session::on_connect;
use crate::uiconf::UiConfig;
use crossterm::event::{self, Event as CEvent, MouseEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use nchat_core_sys::Core;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{stdout, Stdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct TerminalGuard {
    mouse_enabled: bool,
}

impl TerminalGuard {
    pub fn init(mouse_enabled: bool) -> std::io::Result<Self> {
        enable_raw_mode()?;
        let mut out = stdout();
        out.execute(EnterAlternateScreen)?;
        if mouse_enabled {
            out.execute(event::EnableMouseCapture)?;
        }

        // Install panic hook to ensure terminal restoration
        let orig_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = disable_raw_mode();
            let _ = stdout().execute(LeaveAlternateScreen);
            let _ = stdout().execute(event::DisableMouseCapture);
            orig_hook(info);
        }));

        Ok(Self { mouse_enabled })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
        if self.mouse_enabled {
            let _ = stdout().execute(event::DisableMouseCapture);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResizeDrag {
    ListWidth,
    EntryHeight,
}

fn clamp_list_width(column: u16, term_w: u16, is_right: bool) -> u16 {
    let raw = if is_right { term_w.saturating_sub(column + 1) } else { column };
    let upper = term_w.saturating_sub(25).max(10);
    raw.clamp(10, upper)
}

fn clamp_entry_height(row: u16, term_h: u16, help_h: u16) -> u16 {
    let raw = term_h.saturating_sub(row + help_h + 1);
    let upper = (term_h / 2).max(1);
    raw.clamp(1, upper)
}

pub struct App {
    pub model: UiModel,
    pub conf: UiConfig,
    pub theme: Theme,
    pub nav: NavResolver,
    pub picker: ratatui_image::picker::Picker,
    pub img_cache: crate::imgcache::ImageCache,
    pub active_dialog: Option<Box<dyn crate::dialog::Dialog>>,
    pub pending_delete_msg: Option<(String, String, String)>,
    list_width: u16,
    entry_height: u16,
    resize_drag: Option<ResizeDrag>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl App {
    pub fn new(conf: UiConfig, theme: Theme) -> std::io::Result<Self> {
        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)?;
        let vim_keys = conf.vim_keys;
        let list_width = conf.list_width;
        let entry_height = conf.entry_height;
        let picker = crate::picker::init_picker(Duration::from_secs(2));
        let img_cache = crate::imgcache::ImageCache::new();
        Ok(Self {
            model: UiModel::new(),
            conf,
            theme,
            nav: NavResolver::new(vim_keys),
            picker,
            img_cache,
            active_dialog: None,
            pending_delete_msg: None,
            list_width,
            entry_height,
            resize_drag: None,
            terminal,
        })
    }

    pub fn run(
        &mut self,
        rx: Receiver<String>,
        exit_flag: Arc<AtomicBool>,
        core: &mut Core,
    ) -> std::io::Result<()> {
        let frame_interval = Duration::from_millis(16); // ~60fps target
        let mut last_draw = Instant::now() - frame_interval;

        while !exit_flag.load(Ordering::Relaxed) {
            // Drain incoming bridge events (coalesce high-frequency events)
            while let Ok(raw) = rx.try_recv() {
                if let Ok(ev) = serde_json::from_str::<Event>(&raw) {
                    if let Event::Connect { profile, success: true } = &ev {
                        if let Some(info) = self.model.profiles.iter().find(|p| &p.id == profile) {
                            for request in on_connect(info) {
                                let _ = core.send(&request.to_json());
                            }
                        }
                    }
                    let prev_sel = self.model.selected_id.clone();
                    self.model.apply_event(&ev);
                    if let Some((prof, cid)) = &self.model.selected_id {
                        if prev_sel.as_ref() != Some(&(prof.clone(), cid.clone()))
                            || (matches!(&ev, Event::Chats { .. }) && self.model.messages(prof, cid).is_empty())
                        {
                            let req = crate::protocol::Request::GetMessages {
                                profile: prof.clone(),
                                chat_id: cid.clone(),
                                from_msg_id: String::new(),
                                limit: 50,
                            };
                            let _ = core.send(&req.to_json());
                            let status_req = crate::protocol::Request::GetStatus {
                                profile: prof.clone(),
                                user_id: cid.clone(),
                            };
                            let _ = core.send(&status_req.to_json());
                        }
                    }
                }
            }

            for read_req in self.model.take_pending_reads() {
                let _ = core.send(&read_req.to_json());
            }

            for dl_req in self.model.take_pending_downloads() {
                let _ = core.send(&dl_req.to_json());
            }

            // Poll crossterm events (timeout up to 10ms)
            if event::poll(Duration::from_millis(10))? {
                match event::read()? {
                    CEvent::Key(k) => {
                        if let Some(norm) = crate::keyconf::normalize_crossterm_key(k) {
                            if k.code == crossterm::event::KeyCode::Char('q')
                                && k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                            {
                                exit_flag.store(true, Ordering::Relaxed);
                                break;
                            }

                            if let Some(mut dialog) = self.active_dialog.take() {
                                match dialog.handle_key(norm) {
                                    crate::dialog::DialogResult::KeepOpen => {
                                        self.active_dialog = Some(dialog);
                                    }
                                    crate::dialog::DialogResult::Close(ok) => {
                                        if ok {
                                            if let Some((prof, cid, mid)) = self.pending_delete_msg.take() {
                                                let req = crate::protocol::Request::DeleteMessage {
                                                    profile: prof,
                                                    chat_id: cid,
                                                    msg_id: mid,
                                                };
                                                let _ = core.send(&req.to_json());
                                            }
                                        } else {
                                            self.pending_delete_msg = None;
                                        }
                                        self.active_dialog = None;
                                    }
                                    crate::dialog::DialogResult::CloseValue(val) => {
                                        if val.starts_with("[sticker_file_id:") && val.ends_with(']') {
                                            let id_str = &val[17..val.len() - 1];
                                            if let (Ok(sticker_file_id), Some((prof, cid))) =
                                                (id_str.parse::<i32>(), &self.model.selected_id)
                                            {
                                                let req = crate::protocol::Request::SendSticker {
                                                    profile: prof.clone(),
                                                    chat_id: cid.clone(),
                                                    sticker_file_id,
                                                };
                                                let _ = core.send(&req.to_json());
                                            }
                                        } else if val.starts_with('/') || val.contains('/') || val.contains('\\') {
                                            if let Some((prof, cid)) = &self.model.selected_id {
                                                let req = crate::protocol::Request::SendMessage {
                                                    profile: prof.clone(),
                                                    chat_id: cid.clone(),
                                                    text: String::new(),
                                                    quoted_id: String::new(),
                                                    file_path: val,
                                                };
                                                let _ = core.send(&req.to_json());
                                            }
                                        } else {
                                            self.model.entry_buffer.insert_str(&val);
                                        }
                                        self.active_dialog = None;
                                    }
                                }
                                self.model.dirty = true;
                                continue;
                            }

                            // Alt-u opens Emoji Dialog
                            if k.modifiers.contains(crossterm::event::KeyModifiers::ALT)
                                && k.code == crossterm::event::KeyCode::Char('u')
                            {
                                self.active_dialog = Some(Box::new(crate::dialog::emoji::EmojiDialog::new()));
                                self.model.dirty = true;
                                continue;
                            }

                            // Alt-g opens Sticker Dialog
                            if k.modifiers.contains(crossterm::event::KeyModifiers::ALT)
                                && k.code == crossterm::event::KeyCode::Char('g')
                            {
                                if let Some((prof, _)) = self.model.selected_id.clone() {
                                    let sets_req = crate::protocol::Request::GetStickerSets { profile: prof.clone() };
                                    let _ = core.send(&sets_req.to_json());
                                    let stickers_req = crate::protocol::Request::GetStickers {
                                        profile: prof.clone(),
                                        key: "recent".to_string(),
                                        set_id: 0,
                                        query: String::new(),
                                    };
                                    let _ = core.send(&stickers_req.to_json());
                                    self.active_dialog = Some(Box::new(crate::dialog::sticker::StickerDialog::new(prof)));
                                    self.model.dirty = true;
                                }
                                continue;
                            }

                            // Ctrl-O opens Send File Dialog
                            if k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                                && k.code == crossterm::event::KeyCode::Char('o')
                            {
                                self.active_dialog = Some(Box::new(crate::dialog::file::FileDialog::new(None)));
                                self.model.dirty = true;
                                continue;
                            }

                            if self.model.focus == Focus::Entry {
                                if k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                                    && k.code == crossterm::event::KeyCode::Char('v')
                                {
                                    if let Some(text) = crate::clipboard::get_text() {
                                        self.model.entry_buffer.insert_str(&text);
                                        self.model.dirty = true;
                                    }
                                    continue;
                                }

                                match k.code {
                                    crossterm::event::KeyCode::Esc => {
                                        if self.model.edit_message_active || self.model.reply_message_active {
                                            self.model.cancel_compose_mode();
                                        } else {
                                            self.model.focus = Focus::History;
                                        }
                                        self.model.dirty = true;
                                        continue;
                                    }
                                    crossterm::event::KeyCode::Enter => {
                                        if k.modifiers.contains(crossterm::event::KeyModifiers::ALT) {
                                            self.model.entry_buffer.insert_char('\n');
                                            self.model.dirty = true;
                                        } else {
                                            let text = self.model.entry_buffer.text().trim().to_string();
                                            if !text.is_empty() {
                                                if self.model.edit_message_active {
                                                    if let (Some((prof, cid)), Some(mid)) = (&self.model.selected_id, &self.model.edit_message_id) {
                                                        let req = crate::protocol::Request::EditMessage {
                                                            profile: prof.clone(),
                                                            chat_id: cid.clone(),
                                                            msg_id: mid.clone(),
                                                            text,
                                                        };
                                                        let _ = core.send(&req.to_json());
                                                        self.model.entry_buffer.clear();
                                                        self.model.cancel_compose_mode();
                                                        self.model.dirty = true;
                                                    }
                                                } else if let Some((prof, cid)) = &self.model.selected_id {
                                                    let quoted_id = if self.model.reply_message_active {
                                                        self.model.reply_message_id.clone().unwrap_or_default()
                                                    } else {
                                                        String::new()
                                                    };
                                                    let req = crate::protocol::Request::SendMessage {
                                                        profile: prof.clone(),
                                                        chat_id: cid.clone(),
                                                        text,
                                                        quoted_id,
                                                        file_path: String::new(),
                                                    };
                                                    let _ = core.send(&req.to_json());
                                                    self.model.entry_buffer.clear();
                                                    self.model.cancel_compose_mode();
                                                    self.model.dirty = true;
                                                }
                                            }
                                        }
                                        continue;
                                    }
                                    crossterm::event::KeyCode::Backspace => {
                                        self.model.entry_buffer.backspace();
                                        self.model.dirty = true;
                                        continue;
                                    }
                                    crossterm::event::KeyCode::Delete => {
                                        self.model.entry_buffer.delete();
                                        self.model.dirty = true;
                                        continue;
                                    }
                                    crossterm::event::KeyCode::Left => {
                                        self.model.entry_buffer.move_cursor_left();
                                        self.model.dirty = true;
                                        continue;
                                    }
                                    crossterm::event::KeyCode::Right => {
                                        self.model.entry_buffer.move_cursor_right();
                                        self.model.dirty = true;
                                        continue;
                                    }
                                    crossterm::event::KeyCode::Home => {
                                        self.model.entry_buffer.move_cursor_home();
                                        self.model.dirty = true;
                                        continue;
                                    }
                                    crossterm::event::KeyCode::End => {
                                        self.model.entry_buffer.move_cursor_end();
                                        self.model.dirty = true;
                                        continue;
                                    }
                                    crossterm::event::KeyCode::Char(c) if !k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                                        self.model.entry_buffer.insert_char(c);
                                        self.model.dirty = true;
                                        continue;
                                    }
                                    _ => {}
                                }
                            }

                            if self.model.filter_active {
                                match k.code {
                                    crossterm::event::KeyCode::Esc => {
                                        for req in self.model.filter_clear() {
                                            let _ = core.send(&req.to_json());
                                        }
                                        continue;
                                    }
                                    crossterm::event::KeyCode::Enter => {
                                        self.model.filter_active = false;
                                        self.model.dirty = true;
                                        continue;
                                    }
                                    crossterm::event::KeyCode::Backspace => {
                                        for req in self.model.filter_pop_char() {
                                            let _ = core.send(&req.to_json());
                                        }
                                        continue;
                                    }
                                    crossterm::event::KeyCode::Down => {
                                        for req in self.model.select_delta(1) {
                                            let _ = core.send(&req.to_json());
                                        }
                                        continue;
                                    }
                                    crossterm::event::KeyCode::Up => {
                                        for req in self.model.select_delta(-1) {
                                            let _ = core.send(&req.to_json());
                                        }
                                        continue;
                                    }
                                    crossterm::event::KeyCode::Char(c) if !k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                                        for req in self.model.filter_push_char(c) {
                                            let _ = core.send(&req.to_json());
                                        }
                                        continue;
                                    }
                                    _ => {}
                                }
                            }

                            if self.model.focus == Focus::List && k.code == crossterm::event::KeyCode::Char('/') {
                                self.model.filter_active = true;
                                self.model.filter_text.clear();
                                self.model.dirty = true;
                                continue;
                            }

                            if self.model.focus == Focus::List
                                && k.code == crossterm::event::KeyCode::Esc
                                && !self.model.filter_text.is_empty()
                            {
                                for req in self.model.filter_clear() {
                                    let _ = core.send(&req.to_json());
                                }
                                continue;
                            }

                            match self.nav.resolve(norm, self.model.focus) {
                                NavResolveResult::Matched(act) => self.handle_action(act, core),
                                NavResolveResult::Pending => {}
                                NavResolveResult::Ignored => {}
                            }
                        }
                    }
                    CEvent::Mouse(m) => {
                        let size = self.terminal.size()?;
                        let layout = Layout::compute(size.width, size.height, &self.effective_conf());

                        if self.conf.mouse_enabled {
                            if m.kind == MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                                let in_border = m.column >= layout.border.x
                                    && m.column < layout.border.x + layout.border.width
                                    && m.row >= layout.border.y
                                    && m.row < layout.border.y + layout.border.height;
                                let on_entry_top = m.row == layout.entry.y
                                    && m.column >= layout.entry.x
                                    && m.column < layout.entry.x + layout.entry.width;
                                if in_border {
                                    self.resize_drag = Some(ResizeDrag::ListWidth);
                                    continue;
                                }
                                if on_entry_top {
                                    self.resize_drag = Some(ResizeDrag::EntryHeight);
                                    continue;
                                }
                            }
                            if m.kind == MouseEventKind::Drag(crossterm::event::MouseButton::Left) {
                                match self.resize_drag {
                                    Some(ResizeDrag::ListWidth) => {
                                        let is_right = matches!(
                                            self.conf.list_position,
                                            crate::uiconf::ListPosition::Right
                                        );
                                        self.list_width = clamp_list_width(m.column, size.width, is_right);
                                        self.model.dirty = true;
                                        continue;
                                    }
                                    Some(ResizeDrag::EntryHeight) => {
                                        let help_h: u16 = if self.conf.help_enabled { 1 } else { 0 };
                                        self.entry_height = clamp_entry_height(m.row, size.height, help_h);
                                        self.model.dirty = true;
                                        continue;
                                    }
                                    None => {}
                                }
                            }
                            if matches!(m.kind, MouseEventKind::Up(_)) {
                                self.resize_drag = None;
                            }
                        }

                        if let Some(focus) = layout.pane_at(m.column, m.row) {
                            self.model.focus = focus;
                        }
                        if self.model.focus == Focus::History {
                            if m.kind == MouseEventKind::ScrollDown {
                                if let Some(req) = self.model.history_scroll(-3) {
                                    let _ = core.send(&req.to_json());
                                }
                            } else if m.kind == MouseEventKind::ScrollUp {
                                if let Some(req) = self.model.history_scroll(3) {
                                    let _ = core.send(&req.to_json());
                                }
                            }
                        } else {
                            if m.kind == MouseEventKind::ScrollDown {
                                for req in self.model.select_delta(1) {
                                    let _ = core.send(&req.to_json());
                                }
                            } else if m.kind == MouseEventKind::ScrollUp {
                                for req in self.model.select_delta(-1) {
                                    let _ = core.send(&req.to_json());
                                }
                            }
                        }
                    }
                    CEvent::Resize(_, _) => {
                        self.model.dirty = true;
                    }
                    _ => {}
                }
            }

            if self.img_cache.poll_completed(&self.picker) {
                self.model.dirty = true;
            }

            // Draw at most once per frame interval if dirty
            let now = Instant::now();
            if self.model.dirty && now.duration_since(last_draw) >= frame_interval {
                self.draw()?;
                last_draw = now;
                self.model.dirty = false;
            }
        }
        Ok(())
    }

    fn handle_action(&mut self, action: Action, core: &mut Core) {
        match action {
            Action::Down => {
                if self.model.focus == Focus::History {
                    if let Some(req) = self.model.history_scroll(-1) {
                        let _ = core.send(&req.to_json());
                    }
                } else {
                    for req in self.model.select_delta(1) {
                        let _ = core.send(&req.to_json());
                    }
                }
            }
            Action::Up => {
                if self.model.focus == Focus::History {
                    if let Some(req) = self.model.history_scroll(1) {
                        let _ = core.send(&req.to_json());
                    }
                } else {
                    for req in self.model.select_delta(-1) {
                        let _ = core.send(&req.to_json());
                    }
                }
            }
            Action::HalfDown => {
                if self.model.focus == Focus::History {
                    if let Some(req) = self.model.history_scroll(-5) {
                        let _ = core.send(&req.to_json());
                    }
                } else {
                    for req in self.model.select_delta(5) {
                        let _ = core.send(&req.to_json());
                    }
                }
            }
            Action::HalfUp => {
                if self.model.focus == Focus::History {
                    if let Some(req) = self.model.history_scroll(5) {
                        let _ = core.send(&req.to_json());
                    }
                } else {
                    for req in self.model.select_delta(-5) {
                        let _ = core.send(&req.to_json());
                    }
                }
            }
            Action::First => {
                if self.model.focus == Focus::History {
                    if let Some(req) = self.model.history_jump(true) {
                        let _ = core.send(&req.to_json());
                    }
                }
            }
            Action::Last => {
                if self.model.focus == Focus::History {
                    if let Some(req) = self.model.history_jump(false) {
                        let _ = core.send(&req.to_json());
                    }
                }
            }
            Action::FocusNext => {
                self.model.focus = match self.model.focus {
                    Focus::List => Focus::History,
                    Focus::History => Focus::Entry,
                    Focus::Entry => Focus::List,
                };
                self.model.dirty = true;
            }
            Action::FocusPrev => {
                self.model.focus = match self.model.focus {
                    Focus::List => Focus::Entry,
                    Focus::History => Focus::List,
                    Focus::Entry => Focus::History,
                };
                self.model.dirty = true;
            }
            Action::FocusLeft => {
                self.model.focus = Focus::List;
                self.model.dirty = true;
            }
            Action::FocusRight => {
                self.model.focus = Focus::History;
                self.model.dirty = true;
            }
            Action::Open => {
                self.model.focus = Focus::Entry;
                self.model.dirty = true;
            }
            Action::FocusEntry => {
                self.model.focus = Focus::Entry;
                self.model.dirty = true;
            }
            Action::Reply => {
                if self.model.focus == Focus::History {
                    self.model.start_reply();
                }
            }
            Action::Edit => {
                if self.model.focus == Focus::History {
                    self.model.start_edit();
                }
            }
            Action::Copy => {
                if self.model.focus == Focus::History {
                    if let Some(msg) = self.model.selected_message() {
                        crate::clipboard::set_text(&msg.text);
                    }
                }
            }
            Action::Delete => {
                if self.model.focus == Focus::History {
                    if let Some(msg) = self.model.selected_message() {
                        if let Some((prof, cid)) = &self.model.selected_id {
                            self.pending_delete_msg = Some((prof.clone(), cid.clone(), msg.id.clone()));
                            self.active_dialog = Some(Box::new(crate::dialog::message::MessageDialog::new(
                                "Confirmation",
                                "Delete selected message?",
                            )));
                            self.model.dirty = true;
                        }
                    }
                }
            }
            Action::Search => {
                if self.model.focus == Focus::List {
                    self.model.filter_active = true;
                    self.model.filter_text.clear();
                    self.model.dirty = true;
                }
            }
            Action::ToggleGroup => self.model.toggle_group(),
            Action::UnreadOnly => {
                self.model.unread_only = !self.model.unread_only;
                self.model.dirty = true;
            }
            Action::None => {}
            _ => {}
        }
    }

    fn effective_conf(&self) -> UiConfig {
        let mut conf = self.conf.clone();
        conf.list_width = self.list_width;
        conf.entry_height = self.entry_height;
        conf
    }

    fn draw(&mut self) -> std::io::Result<()> {
        let conf = self.effective_conf();
        let theme = self.theme.clone();
        let model = &mut self.model;
        let img_cache = &mut self.img_cache;

        let mut active_dialog = self.active_dialog.take();

        self.terminal.draw(|f| {
            let layout = Layout::compute(f.area().width, f.area().height, &conf);
            top::render_top(f, layout.top, model, &theme, &conf);
            status::render_status(f, layout.status, model, &theme, &conf);
            list::render_list(f, layout.list, model, &theme, &conf);
            history::render_history(f, layout.history, model, &theme, &conf, img_cache);
            entry::render_entry(f, layout.entry, model, &theme, &conf);
            help::render_help(f, layout.help, model, &theme, &conf);

            if let Some(dialog) = active_dialog.as_mut() {
                dialog.render(f, f.area(), &theme, img_cache, model);
            }
        })?;

        self.active_dialog = active_dialog;
        Ok(())
    }
}

#[cfg(test)]
mod resize_tests {
    use super::*;

    #[test]
    fn clamp_list_width_tracks_mouse_column_on_left() {
        assert_eq!(clamp_list_width(40, 120, false), 40);
        assert_eq!(clamp_list_width(2, 120, false), 10); // lower bound
        assert_eq!(clamp_list_width(200, 120, false), 95); // upper bound = 120-25
    }

    #[test]
    fn clamp_list_width_mirrors_for_right_position() {
        // list sits on the right; dragging to column 80 on a 120-wide terminal
        // means 120 - 80 - 1 = 39 columns of list width
        assert_eq!(clamp_list_width(80, 120, true), 39);
    }

    #[test]
    fn clamp_entry_height_tracks_mouse_row() {
        // dragging the entry's top border to row 30 on a 40-row terminal with a 1-row help bar
        // leaves 40 - 30 - 1 - 1 = 8 lines for the entry
        assert_eq!(clamp_entry_height(30, 40, 1), 8);
        assert_eq!(clamp_entry_height(39, 40, 1), 1); // lower bound
        assert_eq!(clamp_entry_height(0, 40, 0), 20); // upper bound = 40/2
    }
}
