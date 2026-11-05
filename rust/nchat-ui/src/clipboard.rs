use std::io::Write;
use std::process::{Command, Stdio};

pub fn set_text(text: &str) {
    #[cfg(target_os = "macos")]
    {
        if let Ok(mut child) = Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
        }
    }
}

pub fn get_text() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("pbpaste").output().ok()?;
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).to_string();
            if !s.is_empty() {
                return Some(s);
            }
        }
    }
    None
}

pub fn has_image() -> bool {
    #[cfg(target_os = "macos")]
    {
        // Check if clipboard contains image data using osascript
        let script = "try\nget the clipboard as «class PNGf»\nreturn true\non error\nreturn false\nend try";
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output();

        if let Ok(out) = output {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_lowercase();
            return s == "true";
        }
    }
    false
}

pub fn save_image_to_temp() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        static mut COUNTER: u32 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        let temp_path = format!("/tmp/nchat-paste-{id}.png");

        // Save clipboard PNG using osascript
        let script = format!(
            "set f to open for access POSIX file \"{temp_path}\" with write permission\nwrite (the clipboard as «class PNGf») to f\nclose access f"
        );
        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output();

        if let Ok(out) = output {
            if out.status.success() && std::path::Path::new(&temp_path).is_file() {
                return Some(temp_path);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_handles_gracefully() {
        // Just verify functions do not panic
        let _ = get_text();
        let _ = has_image();
    }
}
