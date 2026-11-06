use crate::uiconf::UiConfig;
use std::io::Write;
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum NotifyJob {
    Desktop { title: String, text: String },
    Bell,
    Both { title: String, text: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageNotifyDecision {
    pub desktop: bool,
    pub bell: bool,
}

pub fn sanitize_notification_text(input: &str) -> String {
    input.chars().filter(|c| *c != '\'' && *c != '"' && *c != '%').collect()
}

pub fn should_notify_message(
    is_outgoing: bool,
    is_read: bool,
    time_sent: i64,
    connect_time: i64,
    is_muted: bool,
    has_mention: bool,
    chat_was_unread: bool,
    terminal_active: bool,
    is_current_chat: bool,
    uic: &UiConfig,
) -> MessageNotifyDecision {
    if is_outgoing || is_read {
        return MessageNotifyDecision { desktop: false, bell: false };
    }

    if connect_time == 0 || time_sent <= connect_time {
        return MessageNotifyDecision { desktop: false, bell: false };
    }

    if is_muted && !has_mention && !uic.muted_notify_unread {
        return MessageNotifyDecision { desktop: false, bell: false };
    }

    if !uic.notify_every_unread && chat_was_unread {
        return MessageNotifyDecision { desktop: false, bell: false };
    }

    let bell = if terminal_active {
        uic.terminal_bell_active
    } else {
        uic.terminal_bell_inactive
    };

    let desktop = if !uic.desktop_notify_enabled {
        false
    } else if terminal_active {
        if is_current_chat {
            uic.desktop_notify_active_current
        } else {
            uic.desktop_notify_active_noncurrent
        }
    } else {
        uic.desktop_notify_inactive
    };

    MessageNotifyDecision { desktop, bell }
}

fn run_desktop_notify(custom_cmd: &str, title: &str, text: &str) {
    let title = sanitize_notification_text(title);
    let text = sanitize_notification_text(text);

    if !custom_cmd.is_empty() {
        let cmd = custom_cmd.replace("%1", &title).replace("%2", &text);
        let _ = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
        return;
    }

    #[cfg(target_os = "macos")]
    {
        let script = format!("display notification \"{title}: {text}\" with title \"nchat\"");
        let _ = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let msg = format!("{title}: {text}");
        let _ = Command::new("notify-send")
            .arg("nchat")
            .arg(&msg)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = (title, text);
    }
}

fn run_terminal_bell() {
    let mut out = std::io::stdout();
    let _ = out.write_all(b"\x07");
    let _ = out.flush();
}

pub struct NotifyService {
    sender: Sender<NotifyJob>,
}

impl NotifyService {
    pub fn new(uic: &UiConfig) -> Self {
        let (tx, rx) = mpsc::channel();
        let custom_cmd = uic.desktop_notify_command.clone();

        thread::Builder::new()
            .name("nchat-notify-worker".into())
            .spawn(move || {
                Self::worker_loop(rx, custom_cmd);
            })
            .expect("failed to spawn notify worker thread");

        Self { sender: tx }
    }

    pub fn send(&self, job: NotifyJob) {
        let _ = self.sender.send(job);
    }

    fn worker_loop(rx: Receiver<NotifyJob>, custom_cmd: String) {
        let mut last_desktop: Option<Instant> = None;
        let mut last_bell: Option<Instant> = None;
        let min_desktop_interval = Duration::from_millis(500);
        let min_bell_interval = Duration::from_millis(300);

        while let Ok(job) = rx.recv() {
            let now = Instant::now();
            match job {
                NotifyJob::Bell => {
                    if last_bell.map_or(true, |t| now.duration_since(t) >= min_bell_interval) {
                        run_terminal_bell();
                        last_bell = Some(now);
                    }
                }
                NotifyJob::Desktop { title, text } => {
                    if last_desktop.map_or(true, |t| now.duration_since(t) >= min_desktop_interval) {
                        run_desktop_notify(&custom_cmd, &title, &text);
                        last_desktop = Some(now);
                    }
                }
                NotifyJob::Both { title, text } => {
                    if last_bell.map_or(true, |t| now.duration_since(t) >= min_bell_interval) {
                        run_terminal_bell();
                        last_bell = Some(now);
                    }
                    if last_desktop.map_or(true, |t| now.duration_since(t) >= min_desktop_interval) {
                        run_desktop_notify(&custom_cmd, &title, &text);
                        last_desktop = Some(now);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitization() {
        let raw = "Alice's \"secret\" %100 test!";
        let sanitized = sanitize_notification_text(raw);
        assert_eq!(sanitized, "Alices secret 100 test!");
    }

    #[test]
    fn test_should_notify_message_rules() {
        let mut uic = crate::uiconf::UiConfig::default();
        uic.desktop_notify_enabled = true;

        // Outgoing message should never notify
        let d = should_notify_message(
            true,   // is_outgoing
            false,  // is_read
            100,    // time_sent
            50,     // connect_time
            false,  // is_muted
            false,  // has_mention
            false,  // was_unread
            true,   // terminal_active
            false,  // is_current_chat
            &uic,
        );
        assert_eq!(d.desktop, false);
        assert_eq!(d.bell, false);

        // Message sent before connect_time should never notify
        let d2 = should_notify_message(
            false,  // is_outgoing
            false,  // is_read
            40,     // time_sent <= connect_time (50)
            50,     // connect_time
            false,  // is_muted
            false,  // has_mention
            false,  // was_unread
            true,   // terminal_active
            false,  // is_current_chat
            &uic,
        );
        assert_eq!(d2.desktop, false);
        assert_eq!(d2.bell, false);

        // Muted chat without mention should NOT notify
        let d3 = should_notify_message(
            false, false, 100, 50,
            true,   // is_muted
            false,  // has_mention
            false, true, false, &uic,
        );
        assert_eq!(d3.desktop, false);
        assert_eq!(d3.bell, false);

        // Muted chat WITH mention SHOULD notify
        let d4 = should_notify_message(
            false, false, 100, 50,
            true,   // is_muted
            true,   // has_mention
            false, true, false, &uic,
        );
        assert_eq!(d4.desktop, true);

        // Active terminal on current chat: desktop_notify_active_current is false by default
        let d5 = should_notify_message(
            false, false, 100, 50,
            false, false, false,
            true,   // terminal_active
            true,   // is_current_chat
            &uic,
        );
        assert_eq!(d5.desktop, false);

        // Inactive terminal: desktop_notify_inactive is true, terminal_bell_inactive is true
        let d6 = should_notify_message(
            false, false, 100, 50,
            false, false, false,
            false,  // terminal_active
            false,
            &uic,
        );
        assert_eq!(d6.desktop, true);
        assert_eq!(d6.bell, true);
    }
}
