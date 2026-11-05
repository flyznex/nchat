use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Chat {
    pub id: String,
    pub is_unread: bool,
    pub is_unread_mention: bool,
    pub is_muted: bool,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub last_message_time: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub phone: String,
    pub is_self: bool,
    pub is_alias: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Message {
    pub id: String,
    pub sender_id: String,
    pub text: String,
    pub quoted_id: String,
    pub quoted_text: String,
    pub quoted_sender: String,
    pub file_info: String,
    pub link_preview: String,
    pub time_sent: i64,
    pub is_outgoing: bool,
    pub is_read: bool,
    pub is_delivered: bool,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_pinned: bool,
    pub has_mention: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct StickerSetInfo {
    pub id: i64,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct StickerInfo {
    pub file_id: i32,
    pub emoji: String,
    pub thumbnail_path: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ChatFolderInfo {
    pub id: i32,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Connect { profile: String, success: bool },
    Chats { profile: String, success: bool, chats: Vec<Chat> },
    Contacts { profile: String, full_sync: bool, contacts: Vec<Contact> },
    Messages {
        profile: String,
        success: bool,
        chat_id: String,
        from_msg_id: String,
        cached: bool,
        sequence: bool,
        messages: Vec<Message>,
    },
    Sent { profile: String, success: bool, chat_id: String, message: Message },
    File { profile: String, chat_id: String, msg_id: String, file_info: String },
    UserStatus { profile: String, user_id: String, is_online: bool, time_seen: i64 },
    StickerSets { profile: String, sticker_sets: Vec<StickerSetInfo> },
    Stickers { profile: String, key: String, stickers: Vec<StickerInfo> },
    ChatFolders { profile: String, folders: Vec<ChatFolderInfo> },
    UpdateChatFolder { profile: String, folder_id: i32, is_member: bool, chat_ids: Vec<String> },
    Other { profile: String, kind: i32 },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    GetChats { profile: String },
    GetContacts { profile: String },
    GetMessages { profile: String, chat_id: String, from_msg_id: String, limit: i32 },
    SendMessage {
        profile: String,
        chat_id: String,
        text: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        quoted_id: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        file_path: String,
    },
    EditMessage { profile: String, chat_id: String, msg_id: String, text: String },
    DeleteMessage { profile: String, chat_id: String, msg_id: String },
    SendReaction {
        profile: String,
        chat_id: String,
        msg_id: String,
        emoji: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        prev_emoji: String,
    },
    DownloadFile { profile: String, chat_id: String, msg_id: String, file_id: String },
    MarkRead { profile: String, chat_id: String, msg_id: String },
    GetStatus { profile: String, user_id: String },
    GetStickerSets { profile: String },
    GetStickers { profile: String, key: String, set_id: i64, query: String },
    SendSticker { profile: String, chat_id: String, sticker_file_id: i32 },
}

impl Request {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Request serializes")
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ProfileInfo {
    pub id: String,
    pub display_name: String,
    pub auto_get_chats: bool,
    pub auto_get_contacts: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Same literal as s_MessageJson in tests/test018.cpp; keep in sync.
    const MESSAGE_JSON: &str = r#"{"id":"m1","sender_id":"u1","text":"hi","quoted_id":"","quoted_text":"","quoted_sender":"","file_info":"","link_preview":"","time_sent":5,"is_outgoing":false,"is_read":true,"is_delivered":false,"is_edited":false,"is_deleted":false,"is_pinned":false,"has_mention":false}"#;

    fn message() -> Message {
        Message {
            id: "m1".into(),
            sender_id: "u1".into(),
            text: "hi".into(),
            quoted_id: String::new(),
            quoted_text: String::new(),
            quoted_sender: String::new(),
            file_info: String::new(),
            link_preview: String::new(),
            time_sent: 5,
            is_outgoing: false,
            is_read: true,
            is_delivered: false,
            is_edited: false,
            is_deleted: false,
            is_pinned: false,
            has_mention: false,
        }
    }

    #[test]
    fn parses_connect() {
        let e: Event = serde_json::from_str(r#"{"type":"connect","profile":"Dummy_1","success":true}"#).unwrap();
        assert_eq!(e, Event::Connect { profile: "Dummy_1".into(), success: true });
    }

    #[test]
    fn parses_chats() {
        let e: Event = serde_json::from_str(r#"{"type":"chats","profile":"P","success":true,"chats":[{"id":"c1","is_unread":true,"is_unread_mention":false,"is_muted":false,"is_pinned":true,"is_archived":false,"last_message_time":1700000000123}]}"#).unwrap();
        match e {
            Event::Chats { profile, success, chats } => {
                assert_eq!(profile, "P");
                assert!(success);
                assert_eq!(chats.len(), 1);
                assert_eq!(chats[0].id, "c1");
                assert!(chats[0].is_unread && chats[0].is_pinned);
                assert_eq!(chats[0].last_message_time, 1700000000123);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn parses_contacts_with_escapes() {
        let e: Event = serde_json::from_str(r#"{"type":"contacts","profile":"P","full_sync":true,"contacts":[{"id":"u1","name":"Ann \"A\"\n","phone":"123","is_self":true,"is_alias":false}]}"#).unwrap();
        match e {
            Event::Contacts { full_sync, contacts, .. } => {
                assert!(full_sync);
                assert_eq!(contacts[0].name, "Ann \"A\"\n");
                assert!(contacts[0].is_self);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn parses_messages_and_sent() {
        let json = format!(
            r#"{{"type":"messages","profile":"P","success":true,"chat_id":"c1","from_msg_id":"","cached":false,"sequence":true,"messages":[{MESSAGE_JSON}]}}"#
        );
        let e: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(
            e,
            Event::Messages {
                profile: "P".into(),
                success: true,
                chat_id: "c1".into(),
                from_msg_id: String::new(),
                cached: false,
                sequence: true,
                messages: vec![message()],
            }
        );

        let json = format!(
            r#"{{"type":"sent","profile":"P","success":true,"chat_id":"c1","message":{MESSAGE_JSON}}}"#
        );
        let e: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(
            e,
            Event::Sent { profile: "P".into(), success: true, chat_id: "c1".into(), message: message() }
        );
        let e: Event = serde_json::from_str(r#"{"type":"file","profile":"P","chat_id":"c1","msg_id":"m1","file_info":"abc"}"#).unwrap();
        assert_eq!(e, Event::File { profile: "P".into(), chat_id: "c1".into(), msg_id: "m1".into(), file_info: "abc".into() });
    }

    #[test]
    fn parses_sticker_events() {
        let e: Event = serde_json::from_str(
            r#"{"type":"sticker_sets","profile":"P","sticker_sets":[{"id":123456789012,"title":"Cats"}]}"#,
        )
        .unwrap();
        assert_eq!(
            e,
            Event::StickerSets {
                profile: "P".into(),
                sticker_sets: vec![StickerSetInfo { id: 123456789012, title: "Cats".into() }],
            }
        );

        let e: Event = serde_json::from_str(
            r#"{"type":"stickers","profile":"P","key":"recent","stickers":[{"file_id":42,"emoji":"😀","thumbnail_path":"/tmp/t.webp"}]}"#,
        )
        .unwrap();
        assert_eq!(
            e,
            Event::Stickers {
                profile: "P".into(),
                key: "recent".into(),
                stickers: vec![StickerInfo { file_id: 42, emoji: "😀".into(), thumbnail_path: "/tmp/t.webp".into() }],
            }
        );
    }

    #[test]
    fn deserializes_chat_folders_event() {
        let json = r#"{"type":"chat_folders","profile":"P1","folders":[{"id":1,"title":"Work"},{"id":2,"title":"Family"}]}"#;
        let event: Event = serde_json::from_str(json).unwrap();
        match event {
            Event::ChatFolders { profile, folders } => {
                assert_eq!(profile, "P1");
                assert_eq!(folders.len(), 2);
                assert_eq!(folders[0].id, 1);
                assert_eq!(folders[0].title, "Work");
                assert_eq!(folders[1].id, 2);
                assert_eq!(folders[1].title, "Family");
            }
            other => panic!("expected ChatFolders event, got {:?}", other),
        }
    }

    #[test]
    fn deserializes_update_chat_folder_event() {
        let json = r#"{"type":"update_chat_folder","profile":"P1","folder_id":1,"is_member":true,"chat_ids":["c1","c2"]}"#;
        let event: Event = serde_json::from_str(json).unwrap();
        match event {
            Event::UpdateChatFolder { profile, folder_id, is_member, chat_ids } => {
                assert_eq!(profile, "P1");
                assert_eq!(folder_id, 1);
                assert!(is_member);
                assert_eq!(chat_ids, vec!["c1", "c2"]);
            }
            other => panic!("expected UpdateChatFolder event, got {:?}", other),
        }
    }

    #[test]
    fn parses_other() {
        let e: Event = serde_json::from_str(r#"{"type":"other","profile":"P","kind":42}"#).unwrap();
        assert_eq!(e, Event::Other { profile: "P".into(), kind: 42 });
    }

    #[test]
    fn serializes_requests() {
        let r = Request::GetChats { profile: "P".into() };
        assert_eq!(r.to_json(), r#"{"type":"get_chats","profile":"P"}"#);
        let r = Request::GetMessages { profile: "P".into(), chat_id: "c1".into(), from_msg_id: String::new(), limit: 12 };
        assert_eq!(
            r.to_json(),
            r#"{"type":"get_messages","profile":"P","chat_id":"c1","from_msg_id":"","limit":12}"#
        );
        let r = Request::DownloadFile { profile: "P".into(), chat_id: "c1".into(), msg_id: "m1".into(), file_id: "f1".into() };
        assert_eq!(
            r.to_json(),
            r#"{"type":"download_file","profile":"P","chat_id":"c1","msg_id":"m1","file_id":"f1"}"#
        );
        let r = Request::SendMessage {
            profile: "P".into(),
            chat_id: "c1".into(),
            text: "hej \"du\"\n".into(),
            quoted_id: String::new(),
            file_path: String::new(),
        };
        assert_eq!(
            r.to_json(),
            r#"{"type":"send_message","profile":"P","chat_id":"c1","text":"hej \"du\"\n"}"#
        );

        let r = Request::GetStickerSets { profile: "P".into() };
        assert_eq!(r.to_json(), r#"{"type":"get_sticker_sets","profile":"P"}"#);

        let r = Request::GetStickers { profile: "P".into(), key: "recent".into(), set_id: 0, query: String::new() };
        assert_eq!(
            r.to_json(),
            r#"{"type":"get_stickers","profile":"P","key":"recent","set_id":0,"query":""}"#
        );

        let r = Request::SendSticker { profile: "P".into(), chat_id: "c1".into(), sticker_file_id: 42 };
        assert_eq!(
            r.to_json(),
            r#"{"type":"send_sticker","profile":"P","chat_id":"c1","sticker_file_id":42}"#
        );
    }

    #[test]
    fn parses_profiles() {
        let p: Vec<ProfileInfo> = serde_json::from_str(
            r#"[{"id":"Dummy_1","display_name":"","auto_get_chats":false,"auto_get_contacts":true}]"#,
        )
        .unwrap();
        assert_eq!(p[0].id, "Dummy_1");
        assert!(!p[0].auto_get_chats && p[0].auto_get_contacts);
    }
}
