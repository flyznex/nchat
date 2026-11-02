// Drives the real C++ core with the Dummy protocol (always compiled into the default Rust build).
// The core keeps process-wide state, so this file must contain exactly one test.
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use nchat_core_sys::Core;
use nchat_ui::protocol::{Event, ProfileInfo, Request};
use nchat_ui::session::on_connect;

fn wait_for<T>(rx: &Receiver<String>, what: &str, mut pick: impl FnMut(Event) -> Option<T>) -> T {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let left = deadline
            .checked_duration_since(Instant::now())
            .unwrap_or_else(|| panic!("timeout waiting for {what}"));
        let raw = rx
            .recv_timeout(left)
            .unwrap_or_else(|_| panic!("timeout waiting for {what}"));
        let event: Event = serde_json::from_str(&raw).unwrap_or_else(|e| panic!("bad event {raw}: {e}"));
        if let Some(v) = pick(event) {
            return v;
        }
    }
}

#[test]
fn dummy_profile_end_to_end() {
    let dir = std::env::temp_dir().join(format!("nchat-e2e-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let dir_str = dir.to_str().unwrap();

    let (tx, rx) = mpsc::channel();
    let mut core = Core::init(dir_str, tx).expect("nc_init");
    std::fs::create_dir_all(dir.join("profiles/Dummy_1")).unwrap();

    let profiles: Vec<ProfileInfo> = serde_json::from_str(&core.start().expect("nc_start")).unwrap();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].id, "Dummy_1");
    assert!(profiles[0].auto_get_contacts);
    assert!(!profiles[0].auto_get_chats);

    wait_for(&rx, "connect", |e| matches!(e, Event::Connect { success: true, .. }).then_some(()));
    for request in on_connect(&profiles[0]) {
        core.send(&request.to_json()).expect("send request");
    }

    let chat_ids = wait_for(&rx, "chats", |e| match e {
        Event::Chats { chats, .. } if !chats.is_empty() => Some(chats.into_iter().map(|c| c.id).collect::<Vec<_>>()),
        _ => None,
    });
    for chat_id in &chat_ids {
        let request = Request::GetMessages {
            profile: "Dummy_1".into(),
            chat_id: chat_id.clone(),
            from_msg_id: String::new(),
            limit: 12,
        };
        core.send(&request.to_json()).expect("get_messages");
    }
    let (chat_id, count) = wait_for(&rx, "messages", |e| match e {
        Event::Messages { chat_id, messages, .. } if !messages.is_empty() => Some((chat_id, messages.len())),
        _ => None,
    });
    assert!(count > 0);

    let request = Request::SendMessage {
        profile: "Dummy_1".into(),
        chat_id: chat_id.clone(),
        text: "hello from rust".into(),
    };
    core.send(&request.to_json()).expect("send_message");
    let sent_text = wait_for(&rx, "sent", |e| match e {
        Event::Sent { message, success: true, .. } => Some(message.text),
        _ => None,
    });
    assert_eq!(sent_text, "hello from rust");

    assert_eq!(
        core.send(r#"{"type":"get_chats","profile":"nope"}"#),
        Err(nchat_core_sys::CoreError::Code(3))
    );
    assert_eq!(core.send("not json"), Err(nchat_core_sys::CoreError::Code(2)));

    core.stop();
    let _ = std::fs::remove_dir_all(&dir);
}
