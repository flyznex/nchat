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
        let _ = out.execute(crossterm::event::EnableFocusChange);
        let _ = out.execute(crossterm::event::EnableBracketedPaste);
        if mouse_enabled {
            out.execute(event::EnableMouseCapture)?;
        }

        // Install panic hook to ensure terminal restoration
        let orig_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = disable_raw_mode();
            let _ = stdout().execute(crossterm::event::DisableFocusChange);
            let _ = stdout().execute(crossterm::event::DisableBracketedPaste);
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
        let _ = stdout().execute(crossterm::event::DisableFocusChange);
        let _ = stdout().execute(crossterm::event::DisableBracketedPaste);
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
    pub keys: crate::keyconf::KeyConfig,
    pub nav: NavResolver,
    pub picker: ratatui_image::picker::Picker,
    pub img_cache: crate::imgcache::ImageCache,
    pub active_dialog: Option<Box<dyn crate::dialog::Dialog>>,
    pub pending_delete_msg: Option<(String, String, String)>,
    pub pending_poll_msg: Option<String>,
    pub terminal_active: bool,
    pub notify: crate::notify::NotifyService,
    last_esc: Option<Instant>,
    list_width: u16,
    entry_height: u16,
    resize_drag: Option<ResizeDrag>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl App {
    pub fn new(conf: UiConfig, theme: Theme, keys: crate::keyconf::KeyConfig) -> std::io::Result<Self> {
        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)?;
        let vim_keys = conf.vim_keys;
        let list_width = conf.list_width;
        let entry_height = conf.entry_height;
        let picker = crate::picker::init_picker(Duration::from_secs(2));
        let img_cache = crate::imgcache::ImageCache::new();
        let notify = crate::notify::NotifyService::new(&conf);
        Ok(Self {
            model: UiModel::new(),
            conf,
            theme,
            keys,
            nav: NavResolver::new(vim_keys),
            picker,
            img_cache,
            active_dialog: None,
            pending_delete_msg: None,
            pending_poll_msg: None,
            terminal_active: true,
            notify,
            last_esc: None,
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
                    let was_online = self.model.is_online;
                    self.model.apply_event(&ev);

                    if let Event::Connect { success, .. } = &ev {
                        if *success && !was_online && self.conf.desktop_notify_connectivity {
                            self.notify.send(crate::notify::NotifyJob::Desktop {
                                title: "Connection".into(),
                                text: "Online".into(),
                            });
                        }
                    }

                    if let Event::Messages { profile: msg_prof, chat_id: msg_cid, messages: msg_list, .. } = &ev {
                        let connect_time = self.model.connect_time(msg_prof);
                        let is_curr = self.model.selected_id.as_ref() == Some(&(msg_prof.clone(), msg_cid.clone()));
                        let chat_was_unread = self.model.chats.iter()
                            .find(|c| &c.profile_id == msg_prof && &c.chat_id == msg_cid)
                            .map(|c| c.is_unread)
                            .unwrap_or(false);
                        let is_muted = self.model.chats.iter()
                            .find(|c| &c.profile_id == msg_prof && &c.chat_id == msg_cid)
                            .map(|c| c.is_muted)
                            .unwrap_or(false);

                        for m in msg_list {
                            let decision = crate::notify::should_notify_message(
                                m.is_outgoing,
                                m.is_read,
                                m.time_sent,
                                connect_time,
                                is_muted,
                                m.has_mention,
                                chat_was_unread,
                                self.terminal_active,
                                is_curr,
                                &self.conf,
                            );

                            if decision.desktop || decision.bell {
                                let sender_name = self.model.contact_name(msg_prof, &m.sender_id);
                                let chat_name = self.model.chats.iter()
                                    .find(|c| &c.profile_id == msg_prof && &c.chat_id == msg_cid)
                                    .map(|c| c.name.clone())
                                    .unwrap_or_else(|| msg_cid.clone());

                                let mut title = if m.sender_id == *msg_cid {
                                    sender_name
                                } else {
                                    format!("{chat_name} - {sender_name}")
                                };
                                if self.model.profiles.len() > 1 {
                                    title.push_str(&format!(" ({msg_prof})"));
                                }

                                let job = match (decision.desktop, decision.bell) {
                                    (true, true) => crate::notify::NotifyJob::Both { title, text: m.text.clone() },
                                    (true, false) => crate::notify::NotifyJob::Desktop { title, text: m.text.clone() },
                                    (false, true) => crate::notify::NotifyJob::Bell,
                                    (false, false) => continue,
                                };
                                self.notify.send(job);
                            }
                        }
                    }

                    if let Some((prof, cid)) = &self.model.selected_id {
                        let prof_clone = prof.clone();
                        let cid_clone = cid.clone();
                        if prev_sel.as_ref() != Some(&(prof_clone.clone(), cid_clone.clone()))
                            || (matches!(&ev, Event::Chats { .. }) && self.model.messages(&prof_clone, &cid_clone).is_empty())
                        {
                            if let Some(req) = self.model.check_fetch_more_messages(&prof_clone, &cid_clone, 30) {
                                let _ = core.send(&req.to_json());
                            }
                            let status_req = crate::protocol::Request::GetStatus {
                                profile: prof_clone.clone(),
                                user_id: cid_clone.clone(),
                            };
                            let _ = core.send(&status_req.to_json());
                        } else if let Event::Messages { profile: msg_prof, chat_id: msg_cid, .. } = &ev {
                            if msg_prof == &prof_clone && msg_cid == &cid_clone {
                                if let Some(req) = self.model.check_fetch_more_messages(&prof_clone, &cid_clone, 30) {
                                    let _ = core.send(&req.to_json());
                                }
                            }
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
                            if self.keys.matches("quit", norm)
                                || (k.code == crossterm::event::KeyCode::Char('q')
                                    && k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL))
                            {
                                exit_flag.store(true, Ordering::Relaxed);
                                break;
                            }

                            if let Some(mut dialog) = self.active_dialog.take() {
                                match dialog.handle_key(norm, &self.model) {
                                    crate::dialog::DialogResult::KeepOpen => {
                                        self.active_dialog = Some(dialog);
                                    }
                                    crate::dialog::DialogResult::FetchStickers { profile, key, set_id } => {
                                        let req = crate::protocol::Request::GetStickers {
                                            profile,
                                            key,
                                            set_id,
                                            query: String::new(),
                                        };
                                        let _ = core.send(&req.to_json());
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
                                        } else if val.starts_with("[poll_vote:") && val.ends_with(']') {
                                            let ids_str = &val[11..val.len() - 1];
                                            let option_ids: Vec<i32> = if ids_str.is_empty() {
                                                Vec::new()
                                            } else {
                                                ids_str.split(',').filter_map(|s| s.parse::<i32>().ok()).collect()
                                            };
                                            if let (Some((prof, cid)), Some(mid)) = (&self.model.selected_id, self.pending_poll_msg.take()) {
                                                let req = crate::protocol::Request::SetPollAnswer {
                                                    profile: prof.clone(),
                                                    chat_id: cid.clone(),
                                                    msg_id: mid,
                                                    option_ids,
                                                };
                                                let _ = core.send(&req.to_json());
                                            }
                                        } else if val.starts_with('/') || val.contains('/') || val.contains('\\') || std::path::Path::new(&val).is_file() {
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

                            // Emoji Dialog: key.conf "select_emoji" (default Ctrl-S)
                            if self.keys.matches("select_emoji", norm)
                                || (k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                                    && k.code == crossterm::event::KeyCode::Char('s'))
                            {
                                self.active_dialog = Some(Box::new(crate::dialog::emoji::EmojiDialog::new()));
                                self.model.dirty = true;
                                continue;
                            }

                            // Open Link: key.conf "open_link" (default Alt-u)
                            if self.keys.matches("open_link", norm)
                                || (k.modifiers.contains(crossterm::event::KeyModifiers::ALT)
                                    && k.code == crossterm::event::KeyCode::Char('u'))
                            {
                                self.open_selected_or_latest_link();
                                continue;
                            }

                            // Sticker Dialog: key.conf "select_sticker" (default Alt-g)
                            if self.keys.matches("select_sticker", norm)
                                || (k.modifiers.contains(crossterm::event::KeyModifiers::ALT)
                                    && k.code == crossterm::event::KeyCode::Char('g'))
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

                            // Send File Dialog: key.conf "transfer" (default Ctrl-T) or Ctrl-O fallback
                            if self.keys.matches("transfer", norm)
                                || (k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                                    && (k.code == crossterm::event::KeyCode::Char('t')
                                        || k.code == crossterm::event::KeyCode::Char('o')))
                            {
                                self.active_dialog = Some(Box::new(crate::dialog::file::FileDialog::new(None)));
                                self.model.dirty = true;
                                continue;
                            }

                            if self.model.focus == Focus::Entry && !self.nav.has_pending() {
                                if k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                                    && k.code == crossterm::event::KeyCode::Char('v')
                                {
                                    if let Some(text) = crate::clipboard::get_text() {
                                        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
                                        self.model.entry_buffer.insert_str(&normalized);
                                        self.model.dirty = true;
                                    }
                                    continue;
                                }

                                if k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                                    && k.code == crossterm::event::KeyCode::Char('c')
                                {
                                    self.model.entry_buffer.clear();
                                    self.model.dirty = true;
                                    continue;
                                }

                                match k.code {
                                    crossterm::event::KeyCode::Esc => {
                                        self.handle_esc();
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
                                    crossterm::event::KeyCode::Up => {
                                        self.model.entry_buffer.move_cursor_up();
                                        self.model.dirty = true;
                                        continue;
                                    }
                                    crossterm::event::KeyCode::Down => {
                                        self.model.entry_buffer.move_cursor_down();
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

                            if self.model.focus == Focus::History && k.code == crossterm::event::KeyCode::Esc {
                                self.handle_esc();
                                continue;
                            }

                            // Folder tab navigation: key.conf "folder_next"/"folder_prev"
                            // (default Shift-Right/Shift-Left) or plain n/b fallback.
                            if self.model.focus == Focus::List {
                                let plain_n = crossterm::event::KeyCode::Char('n') == k.code
                                    && k.modifiers == crossterm::event::KeyModifiers::NONE;
                                let plain_b = crossterm::event::KeyCode::Char('b') == k.code
                                    && k.modifiers == crossterm::event::KeyModifiers::NONE;
                                if self.keys.matches("folder_next", norm) || plain_n {
                                    let reqs = self.model.cycle_folder(1);
                                    for r in reqs {
                                        let _ = core.send(&r.to_json());
                                    }
                                    continue;
                                }
                                if self.keys.matches("folder_prev", norm) || plain_b {
                                    let reqs = self.model.cycle_folder(-1);
                                    for r in reqs {
                                        let _ = core.send(&r.to_json());
                                    }
                                    continue;
                                }
                            }

                            if self.model.focus == Focus::History
                                && k.modifiers == crossterm::event::KeyModifiers::NONE
                                && k.code == crossterm::event::KeyCode::Char('v')
                            {
                                if let Some(msg) = self.model.selected_message() {
                                    if msg.file_info.starts_with("poll:") {
                                        if let Some(poll_data) = crate::dialog::poll::PollData::from_json(&msg.file_info[5..]) {
                                            self.pending_poll_msg = Some(msg.id.clone());
                                            self.active_dialog = Some(Box::new(crate::dialog::poll::PollVoteDialog::new(poll_data)));
                                            self.model.dirty = true;
                                            continue;
                                        }
                                    }
                                }
                            }

                            match self.nav.resolve(norm, self.model.focus) {
                                NavResolveResult::Matched(act) => self.handle_action(act, core),
                                NavResolveResult::Pending => {}
                                NavResolveResult::Ignored => {
                                    // In list or history focus, typing a character key automatically
                                    // switches focus to the message entry and inserts the character.
                                    if let crossterm::event::KeyCode::Char(c) = k.code {
                                        if !k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                                            && !k.modifiers.contains(crossterm::event::KeyModifiers::ALT)
                                        {
                                            self.model.focus = Focus::Entry;
                                            self.model.entry_buffer.insert_char(c);
                                            self.model.dirty = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    CEvent::Mouse(m) => {
                        let size = self.terminal.size()?;
                        let layout = Layout::compute(size.width, size.height, &self.effective_conf());

                        if self.conf.mouse_enabled {
                            if m.kind == MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                                if let Some(reqs) = self.folder_tab_click(&layout, m.column, m.row) {
                                    self.model.focus = Focus::List;
                                    self.model.select_message_active = false;
                                    for r in reqs {
                                        let _ = core.send(&r.to_json());
                                    }
                                    continue;
                                }
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
                            self.model.select_message_active = self.model.focus == Focus::History;
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
                    CEvent::Paste(s) => {
                        if self.model.focus == Focus::Entry {
                            let normalized = s.replace("\r\n", "\n").replace('\r', "\n");
                            self.model.entry_buffer.insert_str(&normalized);
                            self.model.dirty = true;
                        }
                    }
                    CEvent::Resize(_, _) => {
                        self.model.dirty = true;
                    }
                    CEvent::FocusGained => {
                        self.terminal_active = true;
                    }
                    CEvent::FocusLost => {
                        self.terminal_active = false;
                    }
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
                } else if self.model.focus == Focus::List {
                    for req in self.model.select_first() {
                        let _ = core.send(&req.to_json());
                    }
                }
            }
            Action::Last => {
                if self.model.focus == Focus::History {
                    if let Some(req) = self.model.history_jump(false) {
                        let _ = core.send(&req.to_json());
                    }
                } else if self.model.focus == Focus::List {
                    for req in self.model.select_last() {
                        let _ = core.send(&req.to_json());
                    }
                }
            }
            Action::ScreenHigh => {
                if self.model.focus == Focus::List {
                    let target = self.model.scroll_first;
                    for req in self.model.select_screen_row(target) {
                        let _ = core.send(&req.to_json());
                    }
                }
            }
            Action::ScreenMiddle => {
                if self.model.focus == Focus::List {
                    let target = self.model.scroll_first + 5;
                    for req in self.model.select_screen_row(target) {
                        let _ = core.send(&req.to_json());
                    }
                }
            }
            Action::ScreenLow => {
                if self.model.focus == Focus::List {
                    let target = self.model.scroll_first + 10;
                    for req in self.model.select_screen_row(target) {
                        let _ = core.send(&req.to_json());
                    }
                }
            }
            Action::FolderNext => {
                for req in self.model.cycle_folder(1) {
                    let _ = core.send(&req.to_json());
                }
            }
            Action::FolderPrev => {
                for req in self.model.cycle_folder(-1) {
                    let _ = core.send(&req.to_json());
                }
            }
            Action::FocusNext => {
                self.model.focus = match self.model.focus {
                    Focus::List => Focus::History,
                    Focus::History => Focus::Entry,
                    Focus::Entry => Focus::List,
                };
                self.model.select_message_active = self.model.focus == Focus::History;
                self.model.dirty = true;
            }
            Action::FocusPrev => {
                self.model.focus = match self.model.focus {
                    Focus::List => Focus::Entry,
                    Focus::History => Focus::List,
                    Focus::Entry => Focus::History,
                };
                self.model.select_message_active = self.model.focus == Focus::History;
                self.model.dirty = true;
            }
            Action::FocusLeft => {
                self.model.focus = Focus::List;
                self.model.select_message_active = false;
                self.model.dirty = true;
            }
            Action::FocusRight => {
                self.model.focus = Focus::History;
                self.model.select_message_active = true;
                self.model.dirty = true;
            }
            Action::Open => {
                self.model.focus = Focus::Entry;
                self.model.select_message_active = false;
                self.model.dirty = true;
            }
            Action::FocusEntry => {
                self.model.focus = Focus::Entry;
                self.model.select_message_active = false;
                self.model.dirty = true;
            }
            Action::Reply => {
                if self.model.focus == Focus::History {
                    if let Some(msg) = self.model.selected_message() {
                        if msg.file_info.starts_with("poll:") {
                            if let Some(poll_data) = crate::dialog::poll::PollData::from_json(&msg.file_info[5..]) {
                                self.pending_poll_msg = Some(msg.id.clone());
                                self.active_dialog = Some(Box::new(crate::dialog::poll::PollVoteDialog::new(poll_data)));
                                self.model.dirty = true;
                                return;
                            }
                        }
                    }
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
            Action::ToggleGroup => {
                self.model.toggle_group();
                for req in self.model.snap_selection_to_rows() {
                    let _ = core.send(&req.to_json());
                }
            }
            Action::UnreadOnly => {
                self.model.unread_only = !self.model.unread_only;
                self.model.dirty = true;
                for req in self.model.snap_selection_to_rows() {
                    let _ = core.send(&req.to_json());
                }
            }
            Action::None => {}
            _ => {}
        }
    }

    /// Hit-test a left-click against the folder tab bar rendered by
    /// `render::list` (first inner row of the list pane). Returns the
    /// requests produced by selecting the clicked folder, or `None` if the
    /// click was outside the tab bar / there are no folders to show.
    fn folder_tab_click(
        &mut self,
        layout: &Layout,
        column: u16,
        row: u16,
    ) -> Option<Vec<crate::protocol::Request>> {
        let list = layout.list;
        if list.width < 4 || list.height < 3 {
            return None;
        }
        // Mirrors Block::bordered().inner(list): border on all 4 sides.
        let inner_x = list.x + 1;
        let inner_y = list.y + 1;
        let inner_w = list.width - 2;
        let inner_h = list.height - 2;
        if row != inner_y || inner_h <= 2 {
            return None;
        }

        let folder_names = self.model.folder_names();
        if folder_names.is_empty() || column < inner_x || column >= inner_x + inner_w {
            return None;
        }

        let max_label_w = (inner_w as usize).saturating_sub(4).max(3);
        let raw_titles: Vec<String> = std::iter::once("All".to_string())
            .chain(folder_names.iter().map(|n| {
                if n.chars().count() > max_label_w {
                    crate::text::fit(n, max_label_w).trim_end().to_string()
                } else {
                    n.clone()
                }
            }))
            .collect();

        let widths: Vec<usize> = raw_titles.iter().map(|t| t.chars().count() + 2).collect();
        let sel_idx = if self.model.selected_folder.is_empty() {
            0
        } else {
            folder_names.iter().position(|n| n == &self.model.selected_folder).map(|i| i + 1).unwrap_or(0)
        };
        let avail = if sel_idx > 0 { (inner_w as usize).saturating_sub(2) } else { inner_w as usize };
        let first = crate::chatlist::first_tab(&widths, sel_idx, avail, 3);

        let mut x = inner_x;
        for (i, title) in raw_titles.iter().enumerate().skip(first) {
            let extra = if i == first && first > 0 { 2 } else { 0 };
            let width = (title.chars().count() + 2 + extra) as u16;
            if column >= x && column < x + width {
                return Some(if i == 0 {
                    self.model.set_selected_folder("")
                } else {
                    self.model.set_selected_folder(&folder_names[i - 1])
                });
            }
            x += width;
            if i + 1 < raw_titles.len() {
                x += 3; // " | " divider
            }
            if x >= inner_x + inner_w {
                break;
            }
        }
        None
    }

    fn open_selected_or_latest_link(&self) {
        let (prof, cid) = match &self.model.selected_id {
            Some(pair) => pair,
            None => return,
        };

        let msgs = self.model.messages(prof, cid);
        let target_msg = self.model.selected_message().or_else(|| msgs.last());

        if let Some(msg) = target_msg {
            let mut urls = crate::url::extract_urls(&msg.text);
            if urls.is_empty() && !msg.link_preview.is_empty() {
                urls = crate::url::extract_urls(&msg.link_preview);
            }

            let link_cmd = self.conf.link_open_command.clone();
            for u in urls {
                let _ = crate::url::open_url(&u, &link_cmd);
            }
        }
    }

    pub fn handle_esc(&mut self) {
        let now = Instant::now();
        let is_double_esc = self
            .last_esc
            .is_some_and(|t| now.duration_since(t) < Duration::from_millis(500));
        self.last_esc = if is_double_esc { None } else { Some(now) };

        if self.model.edit_message_active || self.model.reply_message_active {
            self.model.cancel_compose_mode();
        } else if is_double_esc {
            self.model.entry_buffer.clear();
            self.model.focus = Focus::Entry;
            self.model.select_message_active = false;
        } else if self.model.focus == Focus::Entry {
            self.model.focus = Focus::History;
            self.model.select_message_active = true;
        } else if self.model.focus == Focus::History {
            self.model.select_message_active = false;
            self.model.focus = Focus::Entry;
        }
        self.model.dirty = true;
    }

    fn effective_conf(&self) -> UiConfig {
        let mut conf = self.conf.clone();
        conf.list_width = self.list_width;
        let term_w = self.terminal.size().map(|s| s.width).unwrap_or(80);
        let entry_w = (term_w.saturating_sub(conf.list_width + 4) as usize).max(10);
        let (vis_lines, _, _) = self.model.entry_buffer.visual_layout(entry_w);
        let dyn_h = (vis_lines.len() as u16).clamp(1, 8);
        conf.entry_height = self.entry_height.max(dyn_h);
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

    #[test]
    fn folder_tab_click_handles_scrolled_tabs() {
        let mut app = App::new(
            crate::uiconf::UiConfig::default(),
            crate::colorconf::Theme::default(),
            crate::keyconf::KeyConfig::default(),
        )
        .unwrap();
        app.model.folders.insert(
            "P".to_string(),
            vec![
                (1, "Personal".to_string()),
                (2, "Work".to_string()),
                (3, "Home".to_string()),
            ],
        );
        app.model.selected_folder = "Home".to_string();

        let layout = Layout {
            top: ratatui::layout::Rect::default(),
            status: ratatui::layout::Rect::default(),
            border: ratatui::layout::Rect::default(),
            list: ratatui::layout::Rect::new(0, 0, 24, 20),
            history: ratatui::layout::Rect::new(24, 0, 56, 17),
            entry: ratatui::layout::Rect::new(24, 17, 56, 3),
            help: ratatui::layout::Rect::new(0, 20, 80, 1),
        };

        // When "Home" is selected in a 24-col pane, first tab shifts past "All"
        // Row 1 is the tab bar row (inner_y = list.y + 1 = 1)
        let res = app.folder_tab_click(&layout, 1, 1);
        assert!(res.is_some());
    }

    #[test]
    fn open_link_extracts_and_executes_for_selected_message() {
        let mut app = App::new(
            crate::uiconf::UiConfig::default(),
            crate::colorconf::Theme::default(),
            crate::keyconf::KeyConfig::default(),
        )
        .unwrap();

        let out_file = std::env::temp_dir().join(format!("test_open_link_{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&out_file);
        app.conf.link_open_command = format!("echo '%1' > {}", out_file.display());

        let prof = "P";
        let cid = "c1";
        app.model.apply_event(&crate::protocol::Event::Chats {
            profile: prof.into(),
            success: true,
            chats: vec![crate::protocol::Chat {
                id: cid.into(),
                is_unread: false,
                is_unread_mention: false,
                is_muted: false,
                is_pinned: false,
                is_archived: false,
                last_message_time: 100,
            }],
        });
        app.model.apply_event(&crate::protocol::Event::Messages {
            profile: prof.into(),
            success: true,
            chat_id: cid.into(),
            from_msg_id: "".into(),
            cached: false,
            sequence: true,
            messages: vec![crate::protocol::Message {
                id: "m1".into(),
                sender_id: "u1".into(),
                text: "link here: https://example.com/test".into(),
                quoted_id: "".into(),
                quoted_text: "".into(),
                quoted_sender: "".into(),
                file_info: "".into(),
                link_preview: "".into(),
                time_sent: 100,
                is_outgoing: false,
                is_read: true,
                is_delivered: false,
                is_edited: false,
                is_deleted: false,
                is_pinned: false,
                has_mention: false,
                buttons: Vec::new(),
            }],
        });

        app.model.selected_id = Some((prof.into(), cid.into()));
        app.open_selected_or_latest_link();

        // Give shell a tiny moment to write
        std::thread::sleep(std::time::Duration::from_millis(100));
        let content = std::fs::read_to_string(&out_file).unwrap_or_default();
        let _ = std::fs::remove_file(&out_file);
        assert_eq!(content.trim(), "https://example.com/test");
    }

    #[test]
    fn app_initializes_with_terminal_active_true() {
        let app = App::new(
            crate::uiconf::UiConfig::default(),
            crate::colorconf::Theme::default(),
            crate::keyconf::KeyConfig::default(),
        ).unwrap();
        assert_eq!(app.terminal_active, true);
    }

    #[test]
    fn double_esc_clears_entry_buffer() {
        let mut app = App::new(
            crate::uiconf::UiConfig::default(),
            crate::colorconf::Theme::default(),
            crate::keyconf::KeyConfig::default(),
        )
        .unwrap();
        app.model.focus = Focus::Entry;
        app.model.entry_buffer.insert_str("Hello draft");

        // First Esc: moves to History, keeps draft
        app.handle_esc();
        assert_eq!(app.model.focus, Focus::History);
        assert_eq!(app.model.entry_buffer.text(), "Hello draft");

        // Second Esc within 500ms: clears draft and returns to Entry
        app.handle_esc();
        assert_eq!(app.model.focus, Focus::Entry);
        assert_eq!(app.model.entry_buffer.text(), "");
    }

    #[test]
    fn single_esc_after_timeout_does_not_clear() {
        let mut app = App::new(
            crate::uiconf::UiConfig::default(),
            crate::colorconf::Theme::default(),
            crate::keyconf::KeyConfig::default(),
        )
        .unwrap();
        app.model.focus = Focus::Entry;
        app.model.entry_buffer.insert_str("Keep this draft");

        // First Esc: moves to History
        app.handle_esc();
        assert_eq!(app.model.focus, Focus::History);

        // Simulate 600ms passing
        app.last_esc = Some(Instant::now() - Duration::from_millis(600));

        // Second Esc after timeout: switches back to Entry, draft intact
        app.handle_esc();
        assert_eq!(app.model.focus, Focus::Entry);
        assert_eq!(app.model.entry_buffer.text(), "Keep this draft");
    }
}
