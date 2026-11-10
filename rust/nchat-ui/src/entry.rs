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

    pub fn required_height(&self, max_h: usize) -> u16 {
        let lines = self.text.split('\n').count();
        (lines as u16).clamp(1, max_h as u16)
    }

    pub fn cursor_coords(&self) -> (usize, usize) {
        let mut row = 0;
        let mut current_line_start = 0;
        let chars: Vec<char> = self.text.chars().collect();
        let cur_idx = self.cursor_char_idx.min(chars.len());

        for (i, &c) in chars[..cur_idx].iter().enumerate() {
            if c == '\n' {
                row += 1;
                current_line_start = i + 1;
            }
        }
        let col_str: String = chars[current_line_start..cur_idx].iter().collect();
        let col = UnicodeWidthStr::width(col_str.as_str());
        (row, col)
    }

    pub fn move_cursor_up(&mut self) {
        let chars: Vec<char> = self.text.chars().collect();
        let cur_idx = self.cursor_char_idx.min(chars.len());
        let mut prev_nl = None;
        for i in (0..cur_idx).rev() {
            if chars[i] == '\n' {
                prev_nl = Some(i);
                break;
            }
        }
        if let Some(nl) = prev_nl {
            let cur_line_start = nl + 1;
            let col = cur_idx - cur_line_start;
            let mut prev_line_start = 0;
            for i in (0..nl).rev() {
                if chars[i] == '\n' {
                    prev_line_start = i + 1;
                    break;
                }
            }
            let prev_line_len = nl - prev_line_start;
            self.cursor_char_idx = prev_line_start + col.min(prev_line_len);
        } else {
            self.cursor_char_idx = 0;
        }
    }

    pub fn move_cursor_down(&mut self) {
        let chars: Vec<char> = self.text.chars().collect();
        let cur_idx = self.cursor_char_idx.min(chars.len());
        let mut cur_line_start = 0;
        for i in (0..cur_idx).rev() {
            if chars[i] == '\n' {
                cur_line_start = i + 1;
                break;
            }
        }
        let col = cur_idx - cur_line_start;
        let mut next_nl = None;
        for (i, &c) in chars.iter().enumerate().skip(cur_idx) {
            if c == '\n' {
                next_nl = Some(i);
                break;
            }
        }
        if let Some(nl) = next_nl {
            let next_line_start = nl + 1;
            let mut next_line_end = chars.len();
            for (i, &c) in chars.iter().enumerate().skip(next_line_start) {
                if c == '\n' {
                    next_line_end = i;
                    break;
                }
            }
            let next_line_len = next_line_end - next_line_start;
            self.cursor_char_idx = next_line_start + col.min(next_line_len);
        } else {
            self.cursor_char_idx = chars.len();
        }
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

    #[test]
    fn paste_multiline_preserves_newlines_and_navigates_up_down() {
        let mut eb = EntryBuffer::new();
        eb.insert_str("line 1\nline 2 longer\nline 3");
        assert_eq!(eb.text(), "line 1\nline 2 longer\nline 3");

        // Cursor is at end of line 3
        let (row, col) = eb.cursor_coords();
        assert_eq!(row, 2);
        assert_eq!(col, 6);

        // Move up to line 2
        eb.move_cursor_up();
        let (row, col) = eb.cursor_coords();
        assert_eq!(row, 1);
        assert_eq!(col, 6);

        // Move up to line 1 (len 6, so stays at 6)
        eb.move_cursor_up();
        let (row, col) = eb.cursor_coords();
        assert_eq!(row, 0);
        assert_eq!(col, 6);

        // Move down back to line 2
        eb.move_cursor_down();
        let (row, col) = eb.cursor_coords();
        assert_eq!(row, 1);
        assert_eq!(col, 6);
    }

    #[test]
    fn required_height_scales_with_newlines() {
        let mut eb = EntryBuffer::new();
        assert_eq!(eb.required_height(8), 1);
        eb.insert_str("line 1\nline 2\nline 3");
        assert_eq!(eb.required_height(8), 3);
        eb.insert_str("\n4\n5\n6\n7\n8\n9\n10");
        assert_eq!(eb.required_height(8), 8); // clamped to max_h 8
    }
}
