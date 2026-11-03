use crate::chatlist::{build_rows, step, ListChat, ListRow};
use crate::nav::Focus;
use crate::protocol::{Chat, Contact, Event, Message, ProfileInfo};
use std::collections::{HashMap, HashSet};

pub struct UiModel {
    pub profiles: Vec<ProfileInfo>,
    pub chats: Vec<ListChat>,
    contacts: HashMap<(String, String), Contact>,
    pub selected_id: Option<(String, String)>,
    pub focus: Focus,
    pub filter_text: String,
    pub filter_active: bool,
    pub unread_only: bool,
    pub collapsed_groups: HashSet<String>,
    pub is_online: bool,
    pub scroll_first: usize,
    pub dirty: bool,
}

impl UiModel {
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
            chats: Vec::new(),
            contacts: HashMap::new(),
            selected_id: None,
            focus: Focus::List,
            filter_text: String::new(),
            filter_active: false,
            unread_only: false,
            collapsed_groups: HashSet::new(),
            is_online: false,
            scroll_first: 0,
            dirty: true,
        }
    }

    pub fn set_profiles(&mut self, profiles: Vec<ProfileInfo>) {
        self.profiles = profiles;
        self.dirty = true;
    }

    pub fn apply_event(&mut self, event: &Event) {
        self.dirty = true;
        match event {
            Event::Connect { success, .. } => {
                if *success {
                    self.is_online = true;
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
                if !*success || messages.is_empty() {
                    return;
                }
                if let Some(c) = self.chats.iter_mut().find(|c| &c.profile_id == profile && &c.chat_id == chat_id) {
                    if let Some(last) = messages.iter().max_by_key(|m| m.time_sent) {
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
                if let Some(c) = self.chats.iter_mut().find(|c| &c.profile_id == profile && &c.chat_id == chat_id) {
                    c.preview = Self::format_preview(message);
                    c.last_message_time = c.last_message_time.max(message.time_sent);
                }
                self.sort_chats();
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
            });
        }
    }

    fn contact_name(&self, profile: &str, id: &str) -> String {
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
        build_rows(&self.chats, &self.filter_text, self.unread_only, &self.collapsed_groups)
    }

    pub fn select_delta(&mut self, delta: i32) {
        let rows = self.current_rows();
        let cur_row = self.selected_chat_row(&rows);
        if let Some(next_row) = step(&rows, cur_row, delta) {
            if let ListRow::Chat(c_idx) = rows[next_row] {
                let c = &self.chats[c_idx];
                self.selected_id = Some((c.profile_id.clone(), c.chat_id.clone()));
                self.dirty = true;
            }
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
