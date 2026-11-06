use std::process::Command;

/// Extracts all HTTP and HTTPS URLs from a text string.
pub fn extract_urls(text: &str) -> Vec<String> {
    let mut urls = Vec::new();
    for word in text.split_whitespace() {
        let trimmed = word.trim();
        let candidate = if trimmed.starts_with('(') && trimmed.ends_with(')') && trimmed.len() > 2 {
            &trimmed[1..trimmed.len() - 1]
        } else if trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed.len() > 2 {
            &trimmed[1..trimmed.len() - 1]
        } else if trimmed.starts_with('<') && trimmed.ends_with('>') && trimmed.len() > 2 {
            &trimmed[1..trimmed.len() - 1]
        } else if let Some(stripped) = trimmed.strip_prefix('(') {
            stripped
        } else {
            trimmed
        };

        if let Some(pos) = candidate.find("http://").or_else(|| candidate.find("https://")) {
            let mut u = &candidate[pos..];
            // Strip trailing punctuation common in sentences
            while u.ends_with('.')
                || u.ends_with(',')
                || u.ends_with(';')
                || u.ends_with('!')
                || u.ends_with('?')
                || u.ends_with(')')
                || u.ends_with(']')
                || u.ends_with('"')
                || u.ends_with('\'')
                || u.ends_with('>')
            {
                u = &u[..u.len() - 1];
            }
            if u.len() > 8 && !urls.contains(&u.to_string()) {
                urls.push(u.to_string());
            }
        }
    }
    urls
}

/// Opens a URL using the configured command (with `%1` replacement) or the platform default.
pub fn open_url(url: &str, custom_cmd: &str) -> std::io::Result<()> {
    if !custom_cmd.is_empty() {
        let cmd_str = custom_cmd.replace("%1", url);
        Command::new("sh")
            .arg("-c")
            .arg(&cmd_str)
            .spawn()?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/c", "start", "", url])
            .spawn()?;
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Command::new("xdg-open")
            .arg(url)
            .spawn()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_urls_from_sample_text() {
        let text = "@kand07 meeting note với anh Tuấn nhé ạ: https://notexapp.com/share/6aec53a8080c9918c7827943";
        let urls = extract_urls(text);
        assert_eq!(urls, vec!["https://notexapp.com/share/6aec53a8080c9918c7827943"]);
    }

    #[test]
    fn extracts_multiple_urls_and_strips_punctuation() {
        let text = "Check (https://google.com/search?q=rust) and http://example.org/test, also <https://crates.io>.";
        let urls = extract_urls(text);
        assert_eq!(
            urls,
            vec![
                "https://google.com/search?q=rust",
                "http://example.org/test",
                "https://crates.io",
            ]
        );
    }

    #[test]
    fn handles_text_without_urls() {
        let text = "Hello world, no link here!";
        assert!(extract_urls(text).is_empty());
    }
}
