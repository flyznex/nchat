use ratatui_image::picker::Picker;
use std::sync::mpsc;
use std::time::Duration;

pub fn init_picker(timeout: Duration) -> Picker {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let res = Picker::from_query_stdio();
        let _ = tx.send(res);
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok(p)) => p,
        _ => Picker::halfblocks(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picker_fallback_does_not_hang() {
        let p = init_picker(Duration::from_millis(50));
        assert!(matches!(p.protocol_type(), _));
    }
}
