use crate::chatlist::{build_rows, step, ListChat, ListRow};
use crate::nav::Focus;
use crate::protocol::{Chat, Contact, Event, Message, ProfileInfo, Request, StickerInfo, StickerSetInfo};
use std::collections::{HashMap, HashSet};

pub struct UiModel {
    pub profiles: Vec<ProfileInfo>,
    pub chats: Vec<ListChat>,
    contacts: HashMap<(String, String), Contact>,
    messages: HashMap<(String, String), Vec<Message>>,
    download_requested: HashSet<(String, String)>,
    pub selected_id: Option<(String, String)>,
    pub focus: Focus,
    pub filter_text: String,
    pub filter_active: bool,
    pub unread_only: bool,
    pub collapsed_groups: HashSet<String>,
    pub folders: HashMap<String, Vec<(i32, String)>>,
    pub chat_folder_ids: HashMap<(String, String), HashSet<i32>>,
    pub selected_folder: String,
    pub is_online: bool,
    pub scroll_first: usize,
    pub message_offset: usize,
    pub select_message_active: bool,
    pub entry_buffer: crate::entry::EntryBuffer,
    pub edit_message_active: bool,
    pub edit_message_id: Option<String>,
    pub reply_message_active: bool,
    pub reply_message_id: Option<String>,
    user_status: HashMap<(String, String), (bool, i64)>,
    sticker_sets: HashMap<String, Vec<StickerSetInfo>>,
    stickers: HashMap<(String, String), Vec<StickerInfo>>,
    pub requested_from_msg_ids: HashSet<(String, String, String)>,
    pub exhausted_history: HashSet<(String, String)>,
    pub connect_times: HashMap<String, i64>,
    pub last_read_outbox: HashMap<(String, String), u64>,
    pub dirty: bool,
}

impl UiModel {
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
            chats: Vec::new(),
            contacts: HashMap::new(),
            messages: HashMap::new(),
            download_requested: HashSet::new(),
            requested_from_msg_ids: HashSet::new(),
            exhausted_history: HashSet::new(),
            connect_times: HashMap::new(),
            last_read_outbox: HashMap::new(),
            selected_id: None,
            focus: Focus::List,
            filter_text: String::new(),
            filter_active: false,
            unread_only: false,
            collapsed_groups: HashSet::new(),
            folders: HashMap::new(),
            chat_folder_ids: HashMap::new(),
            selected_folder: String::new(),
            is_online: false,
            scroll_first: 0,
            message_offset: 0,
            select_message_active: false,
            entry_buffer: crate::entry::EntryBuffer::new(),
            edit_message_active: false,
            edit_message_id: None,
            reply_message_active: false,
            reply_message_id: None,
            user_status: HashMap::new(),
            sticker_sets: HashMap::new(),
            stickers: HashMap::new(),
            dirty: true,
        }
    }

    pub fn set_profiles(&mut self, profiles: Vec<ProfileInfo>) {
        self.profiles = profiles;
        self.dirty = true;
    }

    pub fn connect_time(&self, profile: &str) -> i64 {
        self.connect_times.get(profile).copied().unwrap_or(0)
    }

    pub fn apply_event(&mut self, event: &Event) {
        self.dirty = true;
        match event {
            Event::Connect { profile, success } => {
                if *success {
                    self.is_online = true;
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as i64)
                        .unwrap_or(0);
                    self.connect_times.insert(profile.clone(), now_ms);
                }
            }
            Event::Chats { profile, success, chats } => {
                if !*success {
                    return;
                }
                for c in chats {
                    if c.is_archived {
                        continue;
                    }
                    self.upsert_chat(profile, c);
                }
                self.sort_chats();
            }
            Event::Contacts { profile, contacts, .. } => {
                for ct in contacts {
                    self.contacts.insert((profile.clone(), ct.id.clone()), ct.clone());
                }
                self.refresh_chat_names();
            }
            Event::Messages { profile, success, chat_id, messages, .. } => {
                if !*success {
                    return;
                }
                if messages.is_empty() {
                    self.exhausted_history.insert((profile.clone(), chat_id.clone()));
                    return;
                }
                let key = (profile.clone(), chat_id.clone());
                let entry = self.messages.entry(key.clone()).or_default();
                for m in messages {
                    if m.id.is_empty() {
                        continue;
                    }
                    if let Some(existing) = entry.iter_mut().find(|x| x.id == m.id) {
                        *existing = m.clone();
                    } else {
                        entry.push(m.clone());
                    }
                }
                entry.sort_by_key(|m| m.time_sent);

                if let Some(watermark) = self.last_read_outbox.get(&key).copied() {
                    for m in entry.iter_mut() {
                        if m.is_outgoing && !m.is_read {
                            if let Ok(m_num) = u64::from_str_radix(&m.id, 16) {
                                if m_num <= watermark {
                                    m.is_read = true;
                                }
                            }
                        }
                    }
                }

                if let Some(c) = self.chats.iter_mut().find(|c| &c.profile_id == profile && &c.chat_id == chat_id) {
                    if let Some(last) = entry.last() {
                        c.preview = Self::format_preview(last);
                        c.last_message_time = c.last_message_time.max(last.time_sent);
                    }
                }
                self.sort_chats();
            }
            Event::Sent { profile, success, chat_id, message } => {
                if !*success {
                    return;
                }
                if !message.id.is_empty() {
                    let key = (profile.clone(), chat_id.clone());
                    let entry = self.messages.entry(key).or_default();
                    if let Some(existing) = entry.iter_mut().find(|x| x.id == message.id) {
                        *existing = message.clone();
                    } else {
                        entry.push(message.clone());
                    }
                    entry.sort_by_key(|m| m.time_sent);
                }

                if let Some(c) = self.chats.iter_mut().find(|c| &c.profile_id == profile && &c.chat_id == chat_id) {
                    if !message.text.is_empty() || !message.file_info.is_empty() {
                        c.preview = Self::format_preview(message);
                    }
                    if message.time_sent > 0 {
                        c.last_message_time = c.last_message_time.max(message.time_sent);
                    }
                }
                self.sort_chats();
            }
            Event::File { profile, chat_id, msg_id, file_info } => {
                let key = (profile.clone(), chat_id.clone());
                if let Some(entry) = self.messages.get_mut(&key) {
                    if let Some(m) = entry.iter_mut().find(|m| &m.id == msg_id) {
                        m.file_info = file_info.clone();
                    }
                }
                self.dirty = true;
            }
            Event::UserStatus { profile, user_id, is_online, time_seen } => {
                self.user_status.insert((profile.clone(), user_id.clone()), (*is_online, *time_seen));
                self.dirty = true;
            }
            Event::StickerSets { profile, sticker_sets } => {
                self.sticker_sets.insert(profile.clone(), sticker_sets.clone());
                self.dirty = true;
            }
            Event::Stickers { profile, key, stickers } => {
                self.stickers.insert((profile.clone(), key.clone()), stickers.clone());
                self.dirty = true;
            }
            Event::ChatFolders { profile, folders } => {
                let list = folders.iter().map(|f| (f.id, f.title.clone())).collect();
                self.folders.insert(profile.clone(), list);
                let names = self.folder_names();
                if !self.selected_folder.is_empty() && !names.contains(&self.selected_folder) {
                    self.selected_folder.clear();
                }
                self.refresh_chat_folders();
                self.snap_selection_to_rows();
                self.dirty = true;
            }
            Event::UpdateChatFolder { profile, folder_id, is_member, chat_ids } => {
                for cid in chat_ids {
                    let entry = self.chat_folder_ids.entry((profile.clone(), cid.clone())).or_default();
                    if *is_member {
                        entry.insert(*folder_id);
                    } else {
                        entry.remove(folder_id);
                    }
                }
                self.refresh_chat_folders();
                self.snap_selection_to_rows();
                self.dirty = true;
            }
            Event::MessageStatus { profile, chat_id, msg_id, is_read, is_delivered } => {
                let key = (profile.clone(), chat_id.clone());
                let target_num = u64::from_str_radix(msg_id, 16).ok();
                if let Some(target) = target_num {
                    if *is_read {
                        let cur = self.last_read_outbox.entry(key.clone()).or_insert(0);
                        *cur = (*cur).max(target);
                    }
                }
                if let Some(entry) = self.messages.get_mut(&key) {
                    for m in entry.iter_mut() {
                        let is_target = &m.id == msg_id;
                        let is_prior_tg = if let (Some(t), Ok(m_num)) = (target_num, u64::from_str_radix(&m.id, 16)) {
                            m.is_outgoing && m_num <= t
                        } else {
                            false
                        };
                        if is_target || (*is_read && is_prior_tg) {
                            if *is_read {
                                m.is_read = true;
                            }
                            if is_target && *is_delivered {
                                m.is_delivered = true;
                            }
                        }
                    }
                }
                self.dirty = true;
            }
            Event::Other { .. } => {}
        }
    }

    fn format_preview(m: &Message) -> String {
        let t = m.text.replace(['\n', '\r', '\t'], " ");
        if !t.is_empty() {
            return t;
        }
        if !m.file_info.is_empty() {
            return "[attachment]".into();
        }
        String::new()
    }

    fn upsert_chat(&mut self, profile: &str, c: &Chat) {
        let group = if c.is_pinned {
            "Pinned".to_string()
        } else {
            profile.to_string()
        };

        let name = self.contact_name(profile, &c.id);

        if let Some(existing) = self.chats.iter_mut().find(|e| e.profile_id == profile && e.chat_id == c.id) {
            existing.is_unread = c.is_unread;
            existing.is_muted = c.is_muted;
            existing.is_pinned = c.is_pinned;
            existing.group = group;
            if c.last_message_time > 0 {
                existing.last_message_time = c.last_message_time;
            }
        } else {
            self.chats.push(ListChat {
                profile_id: profile.to_string(),
                chat_id: c.id.clone(),
                name,
                group,
                is_unread: c.is_unread,
                is_muted: c.is_muted,
                is_pinned: c.is_pinned,
                last_message_time: c.last_message_time,
                preview: String::new(),
                folders: HashSet::new(),
            });
        }
        self.refresh_chat_folders();
    }

    fn refresh_chat_folders(&mut self) {
        for chat in &mut self.chats {
            chat.folders.clear();
            let parent_id = chat.chat_id.split(':').next().unwrap_or(&chat.chat_id);
            if let Some(folder_ids) = self.chat_folder_ids.get(&(chat.profile_id.clone(), parent_id.to_string())) {
                if let Some(folders) = self.folders.get(&chat.profile_id) {
                    for (fid, title) in folders {
                        if folder_ids.contains(fid) {
                            chat.folders.insert(title.clone());
                        }
                    }
                }
            }
        }
    }

    pub fn contact_name(&self, profile: &str, id: &str) -> String {
        if let Some(c) = self.contacts.get(&(profile.to_string(), id.to_string())) {
            if c.is_self {
                return "Saved Messages".into();
            }
            if !c.name.is_empty() {
                return c.name.clone();
            }
        }
        id.to_string()
    }

    fn refresh_chat_names(&mut self) {
        for i in 0..self.chats.len() {
            let name = self.contact_name(&self.chats[i].profile_id, &self.chats[i].chat_id);
            self.chats[i].name = name;
        }
    }

    pub fn sort_chats(&mut self) {
        // Pinned first, then non-muted, then last_message_time desc, then chat_id asc
        self.chats.sort_by(|a, b| {
            b.is_pinned.cmp(&a.is_pinned)
                .then_with(|| a.is_muted.cmp(&b.is_muted))
                .then_with(|| b.last_message_time.cmp(&a.last_message_time))
                .then_with(|| a.chat_id.cmp(&b.chat_id))
        });

        // Ensure selection exists
        if self.selected_id.is_none() && !self.chats.is_empty() {
            self.selected_id = Some((self.chats[0].profile_id.clone(), self.chats[0].chat_id.clone()));
        }
    }

    pub fn selected_chat(&self) -> Option<&ListChat> {
        let (prof, cid) = self.selected_id.as_ref()?;
        self.chats.iter().find(|c| &c.profile_id == prof && &c.chat_id == cid)
    }

    pub fn current_rows(&self) -> Vec<ListRow> {
        build_rows(&self.chats, &self.filter_text, self.unread_only, &self.selected_folder, &self.collapsed_groups)
    }

    pub fn folder_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        if !self.profiles.is_empty() {
            for profile in &self.profiles {
                if let Some(list) = self.folders.get(&profile.id) {
                    for (_, title) in list {
                        if !title.is_empty() && !names.contains(title) {
                            names.push(title.clone());
                        }
                    }
                }
            }
        } else {
            let mut keys: Vec<&String> = self.folders.keys().collect();
            keys.sort();
            for k in keys {
                if let Some(list) = self.folders.get(k) {
                    for (_, title) in list {
                        if !title.is_empty() && !names.contains(title) {
                            names.push(title.clone());
                        }
                    }
                }
            }
        }
        names
    }

    pub fn cycle_folder(&mut self, delta: i32) -> Vec<Request> {
        let names = self.folder_names();
        if names.is_empty() {
            return Vec::new();
        }
        let total = names.len() + 1; // 0 is All
        let current_idx = if self.selected_folder.is_empty() {
            0
        } else {
            names.iter().position(|n| n == &self.selected_folder).map(|i| i + 1).unwrap_or(0)
        };
        let next_idx = (((current_idx as isize + delta as isize) % total as isize + total as isize) % total as isize) as usize;
        self.selected_folder = if next_idx == 0 {
            String::new()
        } else {
            names[next_idx - 1].clone()
        };
        self.dirty = true;
        self.snap_selection_to_rows()
    }

    pub fn set_selected_folder(&mut self, folder: &str) -> Vec<Request> {
        if self.selected_folder != folder {
            self.selected_folder = folder.to_string();
            self.dirty = true;
            return self.snap_selection_to_rows();
        }
        Vec::new()
    }

    pub fn messages(&self, profile: &str, chat_id: &str) -> &[Message] {
        self.messages.get(&(profile.to_string(), chat_id.to_string()))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn current_messages(&self) -> &[Message] {
        if let Some((prof, cid)) = &self.selected_id {
            self.messages(prof, cid)
        } else {
            &[]
        }
    }

    pub fn selected_message(&self) -> Option<&Message> {
        let (prof, cid) = self.selected_id.as_ref()?;
        let msgs = self.messages(prof, cid);
        if msgs.is_empty() {
            return None;
        }
        let total = msgs.len();
        let idx = total.saturating_sub(1 + self.message_offset);
        msgs.get(idx)
    }

    pub fn start_reply(&mut self) {
        let msg_id = self.selected_message().map(|m| m.id.clone());
        if let Some(id) = msg_id {
            self.reply_message_active = true;
            self.reply_message_id = Some(id);
            self.edit_message_active = false;
            self.edit_message_id = None;
            self.focus = Focus::Entry;
            self.dirty = true;
        }
    }

    pub fn start_edit(&mut self) -> bool {
        let outgoing_msg = self.selected_message().and_then(|m| {
            if m.is_outgoing {
                Some((m.id.clone(), m.text.clone()))
            } else {
                None
            }
        });

        if let Some((id, text)) = outgoing_msg {
            self.edit_message_active = true;
            self.edit_message_id = Some(id);
            self.reply_message_active = false;
            self.reply_message_id = None;
            self.entry_buffer.clear();
            self.entry_buffer.insert_str(&text);
            self.focus = Focus::Entry;
            self.dirty = true;
            return true;
        }
        false
    }

    pub fn cancel_compose_mode(&mut self) {
        self.reply_message_active = false;
        self.reply_message_id = None;
        self.edit_message_active = false;
        self.edit_message_id = None;
        self.dirty = true;
    }

    pub fn select_chat(&mut self, profile: &str, chat_id: &str) -> Vec<Request> {
        let changed = match &self.selected_id {
            Some((p, c)) => p != profile || c != chat_id,
            None => true,
        };
        self.selected_id = Some((profile.to_string(), chat_id.to_string()));
        self.message_offset = 0;
        self.select_message_active = false;
        self.dirty = true;

        let mut reqs = Vec::new();
        if changed {
            if let Some(req) = self.check_fetch_more_messages(profile, chat_id, 30) {
                reqs.push(req);
            }
            reqs.push(Request::GetStatus {
                profile: profile.to_string(),
                user_id: chat_id.to_string(),
            });
        }
        reqs
    }

    /// Checks whether more older messages should be fetched to fill the history pane
    /// (up to `target_count` messages, typically 30-50).
    pub fn check_fetch_more_messages(
        &mut self,
        profile: &str,
        chat_id: &str,
        target_count: usize,
    ) -> Option<Request> {
        let key = (profile.to_string(), chat_id.to_string());
        if self.exhausted_history.contains(&key) {
            return None;
        }

        let entry = self.messages.get(&key);
        let count = entry.map(|v| v.len()).unwrap_or(0);

        if count >= target_count {
            return None;
        }

        let from_msg_id = if count == 0 {
            String::new()
        } else {
            entry.and_then(|v| v.first()).map(|m| m.id.clone()).unwrap_or_default()
        };

        if from_msg_id.is_empty() && count > 0 {
            return None;
        }

        let req_key = (profile.to_string(), chat_id.to_string(), from_msg_id.clone());
        if self.requested_from_msg_ids.contains(&req_key) {
            return None;
        }

        self.requested_from_msg_ids.insert(req_key);
        Some(Request::GetMessages {
            profile: profile.to_string(),
            chat_id: chat_id.to_string(),
            from_msg_id,
            limit: 50,
        })
    }

    pub fn sticker_sets(&self, profile: &str) -> &[StickerSetInfo] {
        self.sticker_sets.get(profile).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn stickers(&self, profile: &str, key: &str) -> Option<&[StickerInfo]> {
        self.stickers.get(&(profile.to_string(), key.to_string())).map(|v| v.as_slice())
    }

    pub fn chat_status(&self, profile: &str, chat_id: &str) -> String {
        if let Some(&(is_online, time_seen)) = self.user_status.get(&(profile.to_string(), chat_id.to_string())) {
            if is_online {
                "online".to_string()
            } else if time_seen > 0 {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                let time_str = crate::render::time::format_list_time(time_seen, now);
                format!("seen {time_str}")
            } else {
                "away".to_string()
            }
        } else {
            String::new()
        }
    }

    pub fn take_pending_reads(&mut self) -> Vec<Request> {
        let mut reqs = Vec::new();
        if let Some((prof, cid)) = &self.selected_id {
            if let Some(msgs) = self.messages.get_mut(&(prof.clone(), cid.clone())) {
                for m in msgs.iter_mut() {
                    if !m.is_outgoing && !m.is_read {
                        m.is_read = true;
                        reqs.push(Request::MarkRead {
                            profile: prof.clone(),
                            chat_id: cid.clone(),
                            msg_id: m.id.clone(),
                        });
                    }
                }
            }
            if !reqs.is_empty() {
                if let Some(c) = self.chats.iter_mut().find(|c| &c.profile_id == prof && &c.chat_id == cid) {
                    c.is_unread = false;
                }
                self.dirty = true;
            }
        }
        reqs
    }

    pub fn select_delta(&mut self, delta: i32) -> Vec<Request> {
        let rows = self.current_rows();
        let cur_row = self.selected_chat_row(&rows);
        if let Some(next_row) = step(&rows, cur_row, delta) {
            if let ListRow::Chat(c_idx) = rows[next_row] {
                let prof = self.chats[c_idx].profile_id.clone();
                let cid = self.chats[c_idx].chat_id.clone();
                return self.select_chat(&prof, &cid);
            }
        }
        Vec::new()
    }

    pub fn select_first(&mut self) -> Vec<Request> {
        let rows = self.current_rows();
        if let Some(first_row) = step(&rows, None, 1) {
            if let ListRow::Chat(c_idx) = rows[first_row] {
                let prof = self.chats[c_idx].profile_id.clone();
                let cid = self.chats[c_idx].chat_id.clone();
                return self.select_chat(&prof, &cid);
            }
        }
        Vec::new()
    }

    pub fn select_last(&mut self) -> Vec<Request> {
        let rows = self.current_rows();
        if let Some(last_row) = step(&rows, None, -1) {
            if let ListRow::Chat(c_idx) = rows[last_row] {
                let prof = self.chats[c_idx].profile_id.clone();
                let cid = self.chats[c_idx].chat_id.clone();
                return self.select_chat(&prof, &cid);
            }
        }
        Vec::new()
    }

    pub fn select_screen_row(&mut self, target_row: usize) -> Vec<Request> {
        let rows = self.current_rows();
        if rows.is_empty() {
            return Vec::new();
        }
        let clamped = target_row.min(rows.len().saturating_sub(1));
        let row_idx = if matches!(rows[clamped], ListRow::Chat(_)) {
            clamped
        } else if let Some(next) = step(&rows, Some(clamped), 1) {
            next
        } else if let Some(prev) = step(&rows, Some(clamped), -1) {
            prev
        } else {
            return Vec::new();
        };
        if let ListRow::Chat(c_idx) = rows[row_idx] {
            let prof = self.chats[c_idx].profile_id.clone();
            let cid = self.chats[c_idx].chat_id.clone();
            return self.select_chat(&prof, &cid);
        }
        Vec::new()
    }

    pub fn take_pending_downloads(&mut self) -> Vec<Request> {
        let mut reqs = Vec::new();
        if let Some((prof, cid)) = &self.selected_id {
            if let Some(msgs) = self.messages.get(&(prof.clone(), cid.clone())) {
                for m in msgs {
                    if !m.file_info.is_empty() {
                        if let Some(info) = crate::fileinfo::parse_file_info(&m.file_info) {
                            if !info.is_downloaded() && info.is_image() && !info.file_id.is_empty() {
                                let key = (prof.clone(), info.file_id.clone());
                                if !self.download_requested.contains(&key) {
                                    self.download_requested.insert(key);
                                    reqs.push(Request::DownloadFile {
                                        profile: prof.clone(),
                                        chat_id: cid.clone(),
                                        msg_id: m.id.clone(),
                                        file_id: info.file_id,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        reqs
    }

    pub fn history_scroll(&mut self, delta: i32) -> Option<Request> {
        let (prof, cid) = self.selected_id.as_ref()?;
        let total = self.messages(prof, cid).len();
        if total == 0 {
            return None;
        }

        let max_offset = total.saturating_sub(1);
        if delta > 0 {
            self.message_offset = self.message_offset.saturating_add(delta as usize).min(max_offset);
        } else {
            self.message_offset = self.message_offset.saturating_sub((-delta) as usize);
        }
        self.dirty = true;

        if delta > 0 || self.message_offset + 10 >= total {
            let prof_clone = prof.clone();
            let cid_clone = cid.clone();
            let target = (total + 30).max(50);
            return self.check_fetch_more_messages(&prof_clone, &cid_clone, target);
        }
        None
    }

    pub fn history_jump(&mut self, oldest: bool) -> Option<Request> {
        let (prof, cid) = self.selected_id.as_ref()?;
        let total = self.messages(prof, cid).len();
        if total == 0 {
            return None;
        }
        self.message_offset = if oldest { total.saturating_sub(1) } else { 0 };
        self.dirty = true;

        if oldest {
            let prof_clone = prof.clone();
            let cid_clone = cid.clone();
            let target = total + 50;
            return self.check_fetch_more_messages(&prof_clone, &cid_clone, target);
        }
        None
    }

    pub fn selected_chat_row(&self, rows: &[ListRow]) -> Option<usize> {
        let (prof, cid) = self.selected_id.as_ref()?;
        for (r_idx, r) in rows.iter().enumerate() {
            if let ListRow::Chat(c_idx) = r {
                let c = &self.chats[*c_idx];
                if &c.profile_id == prof && &c.chat_id == cid {
                    return Some(r_idx);
                }
            }
        }
        None
    }

    pub fn toggle_group(&mut self) {
        let (prof, cid) = match &self.selected_id {
            Some(s) => s,
            None => return,
        };
        let group = match self.chats.iter().find(|c| &c.profile_id == prof && &c.chat_id == cid) {
            Some(c) => c.group.clone(),
            None => return,
        };

        if self.collapsed_groups.is_empty() {
            self.collapsed_groups.insert(group);
        } else {
            // Expand all if anything collapsed
            self.collapsed_groups.clear();
        }
        self.dirty = true;
    }

    pub fn snap_selection_to_rows(&mut self) -> Vec<Request> {
        let rows = self.current_rows();
        if self.selected_chat_row(&rows).is_none() {
            if let Some(first_chat_idx) = rows.iter().find_map(|r| match r {
                ListRow::Chat(idx) => Some(*idx),
                _ => None,
            }) {
                let prof = self.chats[first_chat_idx].profile_id.clone();
                let cid = self.chats[first_chat_idx].chat_id.clone();
                return self.select_chat(&prof, &cid);
            }
        }
        Vec::new()
    }

    pub fn filter_push_char(&mut self, c: char) -> Vec<Request> {
        self.filter_text.push(c);
        self.dirty = true;
        let rows = self.current_rows();
        if let Some(first_chat_idx) = rows.iter().find_map(|r| match r {
            ListRow::Chat(idx) => Some(*idx),
            _ => None,
        }) {
            let prof = self.chats[first_chat_idx].profile_id.clone();
            let cid = self.chats[first_chat_idx].chat_id.clone();
            return self.select_chat(&prof, &cid);
        }
        Vec::new()
    }

    pub fn filter_pop_char(&mut self) -> Vec<Request> {
        self.filter_text.pop();
        self.dirty = true;
        let rows = self.current_rows();
        if let Some(first_chat_idx) = rows.iter().find_map(|r| match r {
            ListRow::Chat(idx) => Some(*idx),
            _ => None,
        }) {
            let prof = self.chats[first_chat_idx].profile_id.clone();
            let cid = self.chats[first_chat_idx].chat_id.clone();
            return self.select_chat(&prof, &cid);
        }
        Vec::new()
    }

    pub fn filter_clear(&mut self) -> Vec<Request> {
        self.filter_text.clear();
        self.filter_active = false;
        self.dirty = true;
        self.snap_selection_to_rows()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::ChatFolderInfo;

    fn sample_message(id: &str, time_sent: i64) -> Message {
        Message {
            id: id.into(),
            sender_id: "u1".into(),
            text: "hi".into(),
            quoted_id: String::new(),
            quoted_text: String::new(),
            quoted_sender: String::new(),
            file_info: String::new(),
            link_preview: String::new(),
            time_sent,
            is_outgoing: false,
            is_read: false,
            is_delivered: false,
            is_edited: false,
            is_deleted: false,
            is_pinned: false,
            has_mention: false,
        }
    }

    #[test]
    fn message_status_event_updates_is_read_and_is_delivered() {
        let mut model = UiModel::new();
        let key = ("P".to_string(), "c1".to_string());
        let mut msg = sample_message("m1", 100);
        msg.is_outgoing = true;
        msg.is_read = false;
        msg.is_delivered = false;
        model.messages.insert(key, vec![msg]);

        model.apply_event(&Event::MessageStatus {
            profile: "P".into(),
            chat_id: "c1".into(),
            msg_id: "m1".into(),
            is_read: true,
            is_delivered: true,
        });

        let updated = &model.messages[&("P".to_string(), "c1".to_string())][0];
        assert!(updated.is_read);
        assert!(updated.is_delivered);
    }

    #[test]
    fn message_status_watermark_marks_earlier_outgoing_messages_read() {
        let mut model = UiModel::new();
        let key = ("P".to_string(), "c1".to_string());
        // In Telegram, IDs are hex strings like "10" (16), "20" (32)
        let mut msg1 = sample_message("10", 100);
        msg1.is_outgoing = true;
        msg1.is_read = false;
        let mut msg2 = sample_message("20", 200);
        msg2.is_outgoing = true;
        msg2.is_read = false;
        model.messages.insert(key, vec![msg1, msg2]);

        model.apply_event(&Event::MessageStatus {
            profile: "P".into(),
            chat_id: "c1".into(),
            msg_id: "20".into(),
            is_read: true,
            is_delivered: false,
        });

        let msgs = &model.messages[&("P".to_string(), "c1".to_string())];
        assert!(msgs[0].is_read, "msg1 with id 0x10 <= 0x20 should be marked read");
        assert!(msgs[1].is_read, "msg2 with id 0x20 should be marked read");
    }

    #[test]
    fn records_connect_time_on_event_connect() {
        let mut m = UiModel::new();
        assert_eq!(m.connect_time("P1"), 0);

        m.apply_event(&Event::Connect { profile: "P1".into(), success: true });
        assert!(m.connect_time("P1") > 0);
    }

    #[test]
    fn applies_connect_and_chats_events() {
        let mut m = UiModel::new();
        m.apply_event(&Event::Connect { profile: "P1".into(), success: true });
        assert!(m.is_online);

        let chat = Chat {
            id: "c1".into(),
            is_unread: true,
            is_unread_mention: false,
            is_muted: false,
            is_pinned: true,
            is_archived: false,
            last_message_time: 200,
        };
        m.apply_event(&Event::Chats { profile: "P1".into(), success: true, chats: vec![chat] });
        assert_eq!(m.chats.len(), 1);
        assert_eq!(m.chats[0].chat_id, "c1");
        assert_eq!(m.chats[0].group, "Pinned");
        assert_eq!(m.selected_id, Some(("P1".into(), "c1".into())));
    }

    #[test]
    fn contact_name_updates_chat_name() {
        let mut m = UiModel::new();
        m.apply_event(&Event::Chats {
            profile: "P1".into(),
            success: true,
            chats: vec![Chat {
                id: "u1".into(),
                is_unread: false,
                is_unread_mention: false,
                is_muted: false,
                is_pinned: false,
                is_archived: false,
                last_message_time: 10,
            }],
        });
        assert_eq!(m.chats[0].name, "u1");

        m.apply_event(&Event::Contacts {
            profile: "P1".into(),
            full_sync: false,
            contacts: vec![Contact {
                id: "u1".into(),
                name: "Alice".into(),
                phone: "".into(),
                is_self: false,
                is_alias: false,
            }],
        });
        assert_eq!(m.chats[0].name, "Alice");
    }

    #[test]
    fn incoming_message_updates_preview_and_timestamp() {
        let mut m = UiModel::new();
        m.apply_event(&Event::Chats {
            profile: "P1".into(),
            success: true,
            chats: vec![Chat {
                id: "c1".into(),
                is_unread: false,
                is_unread_mention: false,
                is_muted: false,
                is_pinned: false,
                is_archived: false,
                last_message_time: 10,
            }],
        });
        let msg = Message {
            id: "m1".into(),
            sender_id: "u1".into(),
            text: "new message".into(),
            quoted_id: "".into(),
            quoted_text: "".into(),
            quoted_sender: "".into(),
            file_info: "".into(),
            link_preview: "".into(),
            time_sent: 50,
            is_outgoing: false,
            is_read: false,
            is_delivered: false,
            is_edited: false,
            is_deleted: false,
            is_pinned: false,
            has_mention: false,
        };
        m.apply_event(&Event::Messages {
            profile: "P1".into(),
            success: true,
            chat_id: "c1".into(),
            from_msg_id: "".into(),
            cached: false,
            sequence: false,
            messages: vec![msg],
        });
        assert_eq!(m.chats[0].preview, "new message");
        assert_eq!(m.chats[0].last_message_time, 50);
    }

    #[test]
    fn stores_messages_and_requests_on_chat_selection() {
        let mut m = UiModel::new();
        m.apply_event(&Event::Chats {
            profile: "P1".into(),
            success: true,
            chats: vec![Chat {
                id: "c1".into(),
                is_unread: false,
                is_unread_mention: false,
                is_muted: false,
                is_pinned: false,
                is_archived: false,
                last_message_time: 10,
            }],
        });

        // Chats event auto-selects the lone chat; clear so select_chat sees a real change.
        m.selected_id = None;
        let reqs = m.select_chat("P1", "c1");
        assert_eq!(reqs.len(), 2);
        assert_eq!(
            reqs[0],
            Request::GetMessages {
                profile: "P1".into(),
                chat_id: "c1".into(),
                from_msg_id: "".into(),
                limit: 50,
            }
        );
        assert_eq!(
            reqs[1],
            Request::GetStatus {
                profile: "P1".into(),
                user_id: "c1".into(),
            }
        );
    }

    #[test]
    fn merges_and_sorts_incoming_messages() {
        let mut m = UiModel::new();
        let msg1 = Message {
            id: "m1".into(), sender_id: "u1".into(), text: "first".into(),
            quoted_id: "".into(), quoted_text: "".into(), quoted_sender: "".into(),
            file_info: "".into(), link_preview: "".into(), time_sent: 100,
            is_outgoing: false, is_read: true, is_delivered: false, is_edited: false,
            is_deleted: false, is_pinned: false, has_mention: false,
        };
        let msg2 = Message {
            id: "m2".into(), sender_id: "u1".into(), text: "second".into(),
            quoted_id: "".into(), quoted_text: "".into(), quoted_sender: "".into(),
            file_info: "".into(), link_preview: "".into(), time_sent: 200,
            is_outgoing: true, is_read: false, is_delivered: true, is_edited: false,
            is_deleted: false, is_pinned: false, has_mention: false,
        };

        m.apply_event(&Event::Messages {
            profile: "P1".into(),
            success: true,
            chat_id: "c1".into(),
            from_msg_id: "".into(),
            cached: false,
            sequence: false,
            messages: vec![msg2.clone(), msg1.clone()],
        });

        let msgs = m.messages("P1", "c1");
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].id, "m1"); // sorted ascending by time_sent
        assert_eq!(msgs[1].id, "m2");
    }

    #[test]
    fn history_scroll_bounds_and_requests_older_messages_at_top() {
        let mut m = UiModel::new();
        m.selected_id = Some(("P1".into(), "c1".into()));
        assert_eq!(m.history_scroll(5), None);
        assert_eq!(m.message_offset, 0);
    }

    #[test]
    fn filters_chats_and_snaps_selection() {
        let mut m = UiModel::new();
        m.apply_event(&Event::Chats {
            profile: "P1".into(),
            success: true,
            chats: vec![
                Chat {
                    id: "u1".into(),
                    is_unread: false,
                    is_unread_mention: false,
                    is_muted: false,
                    is_pinned: false,
                    is_archived: false,
                    last_message_time: 20,
                },
                Chat {
                    id: "u2".into(),
                    is_unread: false,
                    is_unread_mention: false,
                    is_muted: false,
                    is_pinned: false,
                    is_archived: false,
                    last_message_time: 10,
                },
            ],
        });
        m.apply_event(&Event::Contacts {
            profile: "P1".into(),
            full_sync: false,
            contacts: vec![
                Contact {
                    id: "u1".into(),
                    name: "Bob".into(),
                    phone: "".into(),
                    is_self: false,
                    is_alias: false,
                },
                Contact {
                    id: "u2".into(),
                    name: "Alice".into(),
                    phone: "".into(),
                    is_self: false,
                    is_alias: false,
                },
            ],
        });

        // Filter for "ali"
        m.filter_push_char('a');
        m.filter_push_char('l');
        m.filter_push_char('i');
        assert_eq!(m.selected_id, Some(("P1".into(), "u2".into())));
        let rows = m.current_rows();
        assert_eq!(rows.len(), 1);

        // Clear filter
        m.filter_clear();
        assert_eq!(m.current_rows().len(), 2);
    }

    #[test]
    fn does_not_duplicate_sent_messages() {
        let mut m = UiModel::new();
        m.apply_event(&Event::Chats {
            profile: "P1".into(),
            success: true,
            chats: vec![Chat {
                id: "c1".into(),
                is_unread: false,
                is_unread_mention: false,
                is_muted: false,
                is_pinned: false,
                is_archived: false,
                last_message_time: 10,
            }],
        });

        // Backend acknowledges sending message (id is empty, time_sent is 0)
        let dummy_sent = Message {
            id: "".into(),
            sender_id: "".into(),
            text: "hello world".into(),
            quoted_id: "".into(),
            quoted_text: "".into(),
            quoted_sender: "".into(),
            file_info: "".into(),
            link_preview: "".into(),
            time_sent: 0,
            is_outgoing: true,
            is_read: false,
            is_delivered: false,
            is_edited: false,
            is_deleted: false,
            is_pinned: false,
            has_mention: false,
        };
        m.apply_event(&Event::Sent {
            profile: "P1".into(),
            success: true,
            chat_id: "c1".into(),
            message: dummy_sent,
        });

        // Real message arrives from network via Event::Messages
        let real_msg = Message {
            id: "m_real_123".into(),
            sender_id: "u_self".into(),
            text: "hello world".into(),
            quoted_id: "".into(),
            quoted_text: "".into(),
            quoted_sender: "".into(),
            file_info: "".into(),
            link_preview: "".into(),
            time_sent: 1700000000000,
            is_outgoing: true,
            is_read: false,
            is_delivered: true,
            is_edited: false,
            is_deleted: false,
            is_pinned: false,
            has_mention: false,
        };
        m.apply_event(&Event::Messages {
            profile: "P1".into(),
            success: true,
            chat_id: "c1".into(),
            from_msg_id: "".into(),
            cached: false,
            sequence: false,
            messages: vec![real_msg],
        });

        let msgs = m.messages("P1", "c1");
        assert_eq!(msgs.len(), 1, "Must not contain duplicate ghost message");
        assert_eq!(msgs[0].id, "m_real_123");
        assert_eq!(msgs[0].time_sent, 1700000000000);
    }

    #[test]
    fn applies_chat_folders_events_and_cycles() {
        let mut m = UiModel::new();
        m.apply_event(&Event::Chats {
            profile: "P1".into(), success: true,
            chats: vec![
                Chat { id: "c1".into(), is_unread: false, is_unread_mention: false, is_muted: false, is_pinned: false, is_archived: false, last_message_time: 10 },
                Chat { id: "c2".into(), is_unread: false, is_unread_mention: false, is_muted: false, is_pinned: false, is_archived: false, last_message_time: 20 },
            ],
        });

        m.apply_event(&Event::ChatFolders {
            profile: "P1".into(),
            folders: vec![
                ChatFolderInfo { id: 10, title: "Work".into() },
                ChatFolderInfo { id: 20, title: "Personal".into() },
            ],
        });

        m.apply_event(&Event::UpdateChatFolder {
            profile: "P1".into(),
            folder_id: 10,
            is_member: true,
            chat_ids: vec!["c1".into()],
        });

        assert_eq!(m.folder_names(), vec!["Work", "Personal"]);
        assert_eq!(m.selected_folder, "");

        // Cycle forward: "" -> "Work"
        m.cycle_folder(1);
        assert_eq!(m.selected_folder, "Work");
        let rows = m.current_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(m.chats[match rows[0] { ListRow::Chat(i) => i, _ => 99 }].chat_id, "c1");

        // Cycle forward: "Work" -> "Personal"
        m.cycle_folder(1);
        assert_eq!(m.selected_folder, "Personal");

        // Cycle forward: "Personal" -> "" (All)
        m.cycle_folder(1);
        assert_eq!(m.selected_folder, "");

        // Cycle backward: "" -> "Personal"
        m.cycle_folder(-1);
        assert_eq!(m.selected_folder, "Personal");
    }

    #[test]
    fn select_first_last_and_screen_row() {
        let mut m = UiModel::new();
        m.apply_event(&Event::Chats {
            profile: "P1".into(),
            success: true,
            chats: vec![
                Chat { id: "c1".into(), is_unread: false, is_unread_mention: false, is_muted: false, is_pinned: false, is_archived: false, last_message_time: 30 },
                Chat { id: "c2".into(), is_unread: false, is_unread_mention: false, is_muted: false, is_pinned: false, is_archived: false, last_message_time: 20 },
                Chat { id: "c3".into(), is_unread: false, is_unread_mention: false, is_muted: false, is_pinned: false, is_archived: false, last_message_time: 10 },
            ],
        });

        // select_first selects the top chat (c1)
        m.select_first();
        assert_eq!(m.selected_id, Some(("P1".into(), "c1".into())));

        // select_last selects the bottom chat (c3)
        m.select_last();
        assert_eq!(m.selected_id, Some(("P1".into(), "c3".into())));

        // select_screen_row selects row at index
        m.select_screen_row(1);
        let selected_cid = m.selected_id.as_ref().unwrap().1.clone();
        assert!(selected_cid == "c2" || selected_cid == "c1" || selected_cid == "c3");
    }

    #[test]
    fn auto_fetches_older_messages_to_fill_history() {
        let mut m = UiModel::new();
        let prof = "Telegram";
        let cid = "12345";

        // Initial fetch with empty history
        let req1 = m.check_fetch_more_messages(prof, cid, 30).unwrap();
        assert_eq!(
            req1,
            Request::GetMessages {
                profile: prof.into(),
                chat_id: cid.into(),
                from_msg_id: "".into(),
                limit: 50,
            }
        );

        // Same request is not duplicated while in flight
        assert_eq!(m.check_fetch_more_messages(prof, cid, 30), None);

        // First message arrives (like Image #3 single message)
        let msg1 = Message {
            id: "m_first".into(), sender_id: "u1".into(), text: "hello".into(),
            quoted_id: "".into(), quoted_text: "".into(), quoted_sender: "".into(),
            file_info: "".into(), link_preview: "".into(), time_sent: 1000,
            is_outgoing: false, is_read: true, is_delivered: false, is_edited: false,
            is_deleted: false, is_pinned: false, has_mention: false,
        };
        m.apply_event(&Event::Messages {
            profile: prof.into(),
            success: true,
            chat_id: cid.into(),
            from_msg_id: "".into(),
            cached: true,
            sequence: true,
            messages: vec![msg1],
        });

        // Chat only has 1 message (< 30): auto-fetches older messages using m_first
        let req2 = m.check_fetch_more_messages(prof, cid, 30).unwrap();
        assert_eq!(
            req2,
            Request::GetMessages {
                profile: prof.into(),
                chat_id: cid.into(),
                from_msg_id: "m_first".into(),
                limit: 50,
            }
        );

        // Older batch arrives from server
        let mut batch = Vec::new();
        for i in 1..=30 {
            batch.push(Message {
                id: format!("m_older_{i}"), sender_id: "u1".into(), text: format!("msg {i}"),
                quoted_id: "".into(), quoted_text: "".into(), quoted_sender: "".into(),
                file_info: "".into(), link_preview: "".into(), time_sent: 500 + i as i64,
                is_outgoing: false, is_read: true, is_delivered: false, is_edited: false,
                is_deleted: false, is_pinned: false, has_mention: false,
            });
        }
        m.apply_event(&Event::Messages {
            profile: prof.into(),
            success: true,
            chat_id: cid.into(),
            from_msg_id: "m_first".into(),
            cached: false,
            sequence: true,
            messages: batch,
        });

        // Now chat has 31 messages (>= 30): no further auto-fetch needed
        assert_eq!(m.check_fetch_more_messages(prof, cid, 30), None);

        // Empty messages event marks chat history as exhausted (start of chat reached)
        m.apply_event(&Event::Messages {
            profile: prof.into(),
            success: true,
            chat_id: cid.into(),
            from_msg_id: "m_older_1".into(),
            cached: false,
            sequence: true,
            messages: vec![],
        });
        assert!(m.exhausted_history.contains(&(prof.into(), cid.into())));
        assert_eq!(m.check_fetch_more_messages(prof, cid, 50), None);
    }
}
