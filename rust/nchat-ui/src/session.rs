use crate::protocol::{Event, ProfileInfo, Request};

pub fn on_connect(profile: &ProfileInfo) -> Vec<Request> {
    let mut requests = Vec::new();
    if !profile.auto_get_chats {
        requests.push(Request::GetChats { profile: profile.id.clone() });
    }
    if !profile.auto_get_contacts {
        requests.push(Request::GetContacts { profile: profile.id.clone() });
    }
    requests
}

pub fn describe(event: &Event) -> String {
    match event {
        Event::Connect { profile, success } => format!("{profile} connect success={success}"),
        Event::Chats { profile, chats, .. } => format!("{profile} chats n={}", chats.len()),
        Event::Contacts { profile, contacts, .. } => format!("{profile} contacts n={}", contacts.len()),
        Event::Messages { profile, chat_id, messages, .. } => match messages.last() {
            Some(m) => format!(
                "{profile} messages chat={chat_id} n={} last=\"{}\" from={}",
                messages.len(),
                m.text.replace('\n', " "),
                m.sender_id
            ),
            None => format!("{profile} messages chat={chat_id} n=0"),
        },
        Event::Sent { profile, chat_id, success, .. } => format!("{profile} sent chat={chat_id} success={success}"),
        Event::File { profile, chat_id, msg_id, .. } => format!("{profile} file chat={chat_id} msg={msg_id}"),
        Event::UserStatus { profile, user_id, is_online, .. } => format!("{profile} status user={user_id} online={is_online}"),
        Event::StickerSets { profile, sticker_sets } => format!("{profile} sticker_sets n={}", sticker_sets.len()),
        Event::Stickers { profile, key, stickers } => format!("{profile} stickers key={key} n={}", stickers.len()),
        Event::ChatFolders { profile, folders } => format!("{profile} chat_folders n={}", folders.len()),
        Event::UpdateChatFolder { profile, folder_id, is_member, .. } => {
            format!("{profile} update_chat_folder id={folder_id} is_member={is_member}")
        }
        Event::Other { profile, kind } => format!("{profile} other kind={kind}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{Message, ProfileInfo, Request};

    fn profile(auto_chats: bool, auto_contacts: bool) -> ProfileInfo {
        ProfileInfo {
            id: "P".into(),
            display_name: String::new(),
            auto_get_chats: auto_chats,
            auto_get_contacts: auto_contacts,
        }
    }

    #[test]
    fn on_connect_requests_what_the_protocol_does_not_push() {
        assert_eq!(on_connect(&profile(false, true)), vec![Request::GetChats { profile: "P".into() }]);
        assert_eq!(on_connect(&profile(true, false)), vec![Request::GetContacts { profile: "P".into() }]);
        assert_eq!(
            on_connect(&profile(false, false)),
            vec![Request::GetChats { profile: "P".into() }, Request::GetContacts { profile: "P".into() }]
        );
        assert!(on_connect(&profile(true, true)).is_empty());
    }

    #[test]
    fn describes_incoming_message_on_one_line() {
        let m = Message {
            id: "m1".into(), sender_id: "u1".into(), text: "hello\nworld".into(),
            quoted_id: String::new(), quoted_text: String::new(), quoted_sender: String::new(),
            file_info: String::new(), link_preview: String::new(), time_sent: 5,
            is_outgoing: false, is_read: false, is_delivered: false, is_edited: false,
            is_deleted: false, is_pinned: false, has_mention: false,
        };
        let e = Event::Messages {
            profile: "P".into(), success: true, chat_id: "c1".into(), from_msg_id: String::new(),
            cached: false, sequence: false, messages: vec![m],
        };
        assert_eq!(describe(&e), "P messages chat=c1 n=1 last=\"hello world\" from=u1");
    }

    #[test]
    fn describes_connect_and_chats() {
        assert_eq!(describe(&Event::Connect { profile: "P".into(), success: true }), "P connect success=true");
        let e = Event::Chats { profile: "P".into(), success: true, chats: vec![] };
        assert_eq!(describe(&e), "P chats n=0");
        let f = Event::File { profile: "P".into(), chat_id: "c1".into(), msg_id: "m1".into(), file_info: "".into() };
        assert_eq!(describe(&f), "P file chat=c1 msg=m1");
    }
}
