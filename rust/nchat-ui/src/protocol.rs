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
    Other { profile: String, kind: i32 },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    GetChats { profile: String },
    GetContacts { profile: String },
    GetMessages { profile: String, chat_id: String, from_msg_id: String, limit: i32 },
    SendMessage { profile: String, chat_id: String, text: String },
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
        let r = Request::SendMessage { profile: "P".into(), chat_id: "c1".into(), text: "hej \"du\"\n".into() };
        assert_eq!(
            r.to_json(),
            r#"{"type":"send_message","profile":"P","chat_id":"c1","text":"hej \"du\"\n"}"#
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
