use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EntryBuffer {
    text: String,
    cursor_char_idx: usize,
}

impl EntryBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor_idx(&self) -> usize {
        self.cursor_char_idx
    }

    pub fn insert_char(&mut self, c: char) {
        let chars: Vec<char> = self.text.chars().collect();
        let idx = self.cursor_char_idx.min(chars.len());
        let mut new_chars = Vec::with_capacity(chars.len() + 1);
        new_chars.extend_from_slice(&chars[..idx]);
        new_chars.push(c);
        new_chars.extend_from_slice(&chars[idx..]);
        self.text = new_chars.into_iter().collect();
        self.cursor_char_idx = idx + 1;
    }

    pub fn insert_str(&mut self, s: &str) {
        for c in s.chars() {
            self.insert_char(c);
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_char_idx == 0 {
            return;
        }
        let chars: Vec<char> = self.text.chars().collect();
        let idx = self.cursor_char_idx.min(chars.len());
        let mut new_chars = Vec::with_capacity(chars.len().saturating_sub(1));
        new_chars.extend_from_slice(&chars[..idx - 1]);
        new_chars.extend_from_slice(&chars[idx..]);
        self.text = new_chars.into_iter().collect();
        self.cursor_char_idx -= 1;
    }

    pub fn delete(&mut self) {
        let chars: Vec<char> = self.text.chars().collect();
        let idx = self.cursor_char_idx;
        if idx >= chars.len() {
            return;
        }
        let mut new_chars = Vec::with_capacity(chars.len().saturating_sub(1));
        new_chars.extend_from_slice(&chars[..idx]);
        new_chars.extend_from_slice(&chars[idx + 1..]);
        self.text = new_chars.into_iter().collect();
    }

    pub fn delete_word_backward(&mut self) {
        if self.cursor_char_idx == 0 {
            return;
        }
        let chars: Vec<char> = self.text.chars().collect();
        let mut idx = self.cursor_char_idx.min(chars.len());

        // Skip trailing spaces
        while idx > 0 && chars[idx - 1].is_whitespace() {
            idx -= 1;
        }
        // Delete up to previous whitespace
        while idx > 0 && !chars[idx - 1].is_whitespace() {
            idx -= 1;
        }

        let mut new_chars = Vec::new();
        new_chars.extend_from_slice(&chars[..idx]);
        new_chars.extend_from_slice(&chars[self.cursor_char_idx.min(chars.len())..]);
        self.text = new_chars.into_iter().collect();
        self.cursor_char_idx = idx;
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_char_idx > 0 {
            self.cursor_char_idx -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        let len = self.text.chars().count();
        if self.cursor_char_idx < len {
            self.cursor_char_idx += 1;
        }
    }

    pub fn move_cursor_home(&mut self) {
        self.cursor_char_idx = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_char_idx = self.text.chars().count();
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_char_idx = 0;
    }

    pub fn lines(&self, max_width: usize) -> (Vec<String>, usize, usize) {
        if max_width == 0 {
            return (vec![String::new()], 0, 0);
        }

        let mut lines = Vec::new();
        let cursor_row = 0;
        let cursor_col = 0;

        for paragraph in self.text.split('\n') {
            let mut line = String::new();
            let mut line_w = 0;

            for word in paragraph.split(' ') {
                let word_w = UnicodeWidthStr::width(word);
                if line.is_empty() {
                    line.push_str(word);
                    line_w = word_w;
                } else if line_w + 1 + word_w <= max_width {
                    line.push(' ');
                    line.push_str(word);
                    line_w += 1 + word_w;
                } else {
                    lines.push(line);
                    line = word.to_string();
                    line_w = word_w;
                }
            }
            lines.push(line);
        }

        if lines.is_empty() {
            lines.push(String::new());
        }

        (lines, cursor_row, cursor_col)
    }

    pub fn required_height(&self, max_width: usize, max_h: usize) -> u16 {
        let (lines, _, _) = self.lines(max_width);
        // +1 for header title line
        let total = (lines.len() + 1).min(max_h).max(2);
        total as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_characters_and_tracks_cursor() {
        let mut eb = EntryBuffer::new();
        eb.insert_char('h');
        eb.insert_char('i');
        assert_eq!(eb.text(), "hi");
        assert_eq!(eb.cursor_idx(), 2);

        eb.move_cursor_left();
        assert_eq!(eb.cursor_idx(), 1);
        eb.insert_char('e');
        assert_eq!(eb.text(), "hei");
    }

    #[test]
    fn handles_backspace_and_delete() {
        let mut eb = EntryBuffer::new();
        eb.insert_str("hello");
        eb.backspace();
        assert_eq!(eb.text(), "hell");

        eb.move_cursor_home();
        eb.delete();
        assert_eq!(eb.text(), "ell");
    }

    #[test]
    fn deletes_word_backward() {
        let mut eb = EntryBuffer::new();
        eb.insert_str("hello world ");
        eb.delete_word_backward();
        assert_eq!(eb.text(), "hello ");
        eb.delete_word_backward();
        assert_eq!(eb.text(), "");
    }
}
