use image::DynamicImage;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::mpsc;

pub struct ImageCache {
    tx: mpsc::Sender<String>,
    rx: mpsc::Receiver<(String, Result<DynamicImage, String>)>,
    in_flight: HashSet<String>,
    failed: HashSet<String>,
    loaded: HashMap<String, StatefulProtocol>,
}

impl ImageCache {
    pub fn new() -> Self {
        let (req_tx, req_rx) = mpsc::channel::<String>();
        let (done_tx, done_rx) = mpsc::channel::<(String, Result<DynamicImage, String>)>();

        std::thread::spawn(move || {
            while let Ok(path) = req_rx.recv() {
                let res = if Path::new(&path).is_file() {
                    match image::open(&path) {
                        Ok(img) => Ok(img),
                        Err(_) => {
                            // Video stickers (.webm) or formats image crate can't decode directly:
                            // convert frame 1 to temporary png using ffmpeg / sips
                            use std::hash::{Hash, Hasher};
                            let mut hasher = std::collections::hash_map::DefaultHasher::new();
                            path.hash(&mut hasher);
                            let tmp_png = format!("/tmp/nchat-conv-{:016x}.png", hasher.finish());

                            let mut conv_ok = false;
                            if path.ends_with(".webm") {
                                let ffmpeg_bin = if Path::new("/opt/homebrew/bin/ffmpeg").is_file() {
                                    "/opt/homebrew/bin/ffmpeg"
                                } else {
                                    "ffmpeg"
                                };
                                if let Ok(status) = std::process::Command::new(ffmpeg_bin)
                                    .args(["-v", "error", "-y", "-i", &path, "-frames:v", "1", &tmp_png])
                                    .status()
                                {
                                    if status.success() && Path::new(&tmp_png).is_file() {
                                        conv_ok = true;
                                    }
                                }
                            }
                            if !conv_ok {
                                if let Ok(status) = std::process::Command::new("sips")
                                    .args(["-s", "format", "png", "-Z", "400", &path, "--out", &tmp_png])
                                    .status()
                                {
                                    if status.success() && Path::new(&tmp_png).is_file() {
                                        conv_ok = true;
                                    }
                                }
                            }

                            if conv_ok {
                                image::open(&tmp_png).map_err(|e| e.to_string())
                            } else {
                                Err("failed to decode image or convert sticker".into())
                            }
                        }
                    }
                } else {
                    Err("not a file".into())
                };
                let _ = done_tx.send((path, res));
            }
        });

        Self {
            tx: req_tx,
            rx: done_rx,
            in_flight: HashSet::new(),
            failed: HashSet::new(),
            loaded: HashMap::new(),
        }
    }

    pub fn poll_completed(&mut self, picker: &Picker) -> bool {
        let mut changed = false;
        while let Ok((path, res)) = self.rx.try_recv() {
            self.in_flight.remove(&path);
            match res {
                Ok(dyn_img) => {
                    let state = picker.new_resize_protocol(dyn_img);
                    self.loaded.insert(path, state);
                    changed = true;
                }
                Err(_) => {
                    self.failed.insert(path);
                }
            }
        }
        changed
    }

    pub fn get_or_request(&mut self, path: &str) -> Option<&mut StatefulProtocol> {
        if self.loaded.contains_key(path) {
            return self.loaded.get_mut(path);
        }
        if !self.in_flight.contains(path) && !self.failed.contains(path) {
            self.in_flight.insert(path.to_string());
            let _ = self.tx.send(path.to_string());
        }
        None
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn cache_requests_and_handles_missing_file() {
        let mut cache = ImageCache::new();
        let picker = Picker::halfblocks();
        assert!(cache.get_or_request("/non/existent/img.jpg").is_none());

        // Wait brief moment for background worker
        std::thread::sleep(Duration::from_millis(50));
        cache.poll_completed(&picker);
        assert!(cache.failed.contains("/non/existent/img.jpg"));
    }
}
