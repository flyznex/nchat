#[derive(Debug, Clone, PartialEq)]
pub struct FileInfo {
    pub file_status: u32,
    pub file_id: String,
    pub file_path: String,
    pub file_type: String,
}

impl FileInfo {
    pub fn is_downloaded(&self) -> bool {
        (self.file_status == 1 || std::path::Path::new(&self.file_path).is_file())
            && !self.file_path.is_empty()
            && !self.file_path.starts_with('[')
    }

    pub fn is_image(&self) -> bool {
        let lower = self.file_type.to_lowercase();
        let path_lower = self.file_path.to_lowercase();
        lower.starts_with("image/")
            || path_lower.ends_with(".jpg")
            || path_lower.ends_with(".jpeg")
            || path_lower.ends_with(".png")
            || path_lower.ends_with(".webp")
            || path_lower.ends_with(".webm")
            || lower.contains("webp")
            || lower.contains("webm")
    }
}

fn decode_hex_str(s: &str) -> String {
    let mut bytes = Vec::new();
    let s = s.trim();
    let chars: Vec<char> = s.chars().collect();
    for chunk in chars.chunks(2) {
        if chunk.len() == 2 {
            let hex_pair: String = chunk.iter().collect();
            if let Ok(b) = u8::from_str_radix(&hex_pair, 16) {
                bytes.push(b);
            }
        }
    }
    String::from_utf8(bytes).unwrap_or_default()
}

pub fn parse_file_info(s: &str) -> Option<FileInfo> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() < 4 {
        return None;
    }
    // StrUtil::NumToHex<int>(status) encodes the string representation of the number to hex,
    // so status 2 becomes StrToHex("2") = "32". We decode hex first, then parse.
    let status_decoded = decode_hex_str(parts[0]);
    let file_status = status_decoded
        .trim()
        .parse::<u32>()
        .or_else(|_| parts[0].trim().parse::<u32>())
        .ok()?;
    let file_id = decode_hex_str(parts[1]);
    let file_path = decode_hex_str(parts[2]);
    let file_type = decode_hex_str(parts[3]);

    Some(FileInfo {
        file_status,
        file_id,
        file_path,
        file_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex_encode(s: &str) -> String {
        s.bytes().map(|b| format!("{:02x}", b)).collect()
    }

    #[test]
    fn parses_comma_separated_hex_file_info() {
        let status_hex = hex_encode("1"); // C++ produces "31" for FileStatusDownloaded
        let id_hex = hex_encode("123");
        let path_hex = hex_encode("/path/to/img.jpg");
        let type_hex = hex_encode("image/jpeg");
        let encoded = format!("{status_hex},{id_hex},{path_hex},{type_hex}");

        let info = parse_file_info(&encoded).unwrap();
        assert_eq!(info.file_status, 1);
        assert_eq!(info.file_id, "123");
        assert_eq!(info.file_path, "/path/to/img.jpg");
        assert!(info.is_downloaded());
        assert!(info.is_image());
    }

    #[test]
    fn detects_non_downloaded_status() {
        let status_hex = hex_encode("0"); // not downloaded
        let id_hex = hex_encode("123");
        let path_hex = hex_encode("[Photo]");
        let type_hex = hex_encode("image/png");
        let encoded = format!("{status_hex},{id_hex},{path_hex},{type_hex}");

        let info = parse_file_info(&encoded).unwrap();
        assert_eq!(info.file_status, 0);
        assert!(!info.is_downloaded());
    }
}
