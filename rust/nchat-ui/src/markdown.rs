use unicode_width::UnicodeWidthChar;

/// Bitmask of inline/block text styles. Implemented as a plain wrapper over
/// `u16` rather than pulling in the `bitflags` crate (no new dependency needed
/// for eight flags).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StyleFlags(u16);

impl StyleFlags {
    pub const NONE: StyleFlags = StyleFlags(0);
    pub const BOLD: StyleFlags = StyleFlags(1 << 0);
    pub const ITALIC: StyleFlags = StyleFlags(1 << 1);
    pub const CODE: StyleFlags = StyleFlags(1 << 2);
    pub const CODE_BLOCK: StyleFlags = StyleFlags(1 << 3);
    pub const STRIKE: StyleFlags = StyleFlags(1 << 4);
    pub const UNDERLINE: StyleFlags = StyleFlags(1 << 5);
    pub const SPOILER: StyleFlags = StyleFlags(1 << 6);
    pub const LINK: StyleFlags = StyleFlags(1 << 7);

    pub fn contains(self, other: StyleFlags) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::ops::BitOr for StyleFlags {
    type Output = StyleFlags;
    fn bitor(self, rhs: StyleFlags) -> StyleFlags {
        StyleFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for StyleFlags {
    fn bitor_assign(&mut self, rhs: StyleFlags) {
        self.0 |= rhs.0;
    }
}

/// A run of text sharing a single style mask.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormattedSpan {
    pub text: String,
    pub style: StyleFlags,
}

/// A single wrapped output line, made up of styled spans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormattedLine {
    pub spans: Vec<FormattedSpan>,
    pub width: usize,
}

/// Parses Markdown V2 style text and word-wraps it to `max_width` columns,
/// preserving style flags (and raw code block content) across wrapped lines.
pub fn wrap_markdown(text: &str, max_width: usize) -> Vec<FormattedLine> {
    let mut lines = Vec::new();
    let mut rest = text;

    while let Some(fence_start) = rest.find("```") {
        if fence_start > 0 {
            lines.extend(wrap_plain_text(&rest[..fence_start], max_width));
        }
        let (code_lines, remainder) = extract_code_block(&rest[fence_start..]);
        for code_line in code_lines {
            let width = line_width(&code_line);
            lines.push(FormattedLine {
                spans: vec![FormattedSpan { text: code_line, style: StyleFlags::CODE_BLOCK }],
                width,
            });
        }
        rest = remainder;
    }
    lines.extend(wrap_plain_text(rest, max_width));

    lines
}

/// Splits off one fenced code block starting at `s` (which must begin with
/// "```"). Returns the verbatim content lines (indentation and newlines
/// preserved, fence lines dropped) plus whatever text follows the block.
fn extract_code_block(s: &str) -> (Vec<String>, &str) {
    let after_open = match s.find('\n') {
        Some(idx) => &s[idx + 1..],
        None => return (Vec::new(), ""),
    };
    match after_open.find("```") {
        Some(close_pos) => {
            let content = after_open[..close_pos].strip_suffix('\n').unwrap_or(&after_open[..close_pos]);
            let lines = content.split('\n').map(|l| l.to_string()).collect();
            let after_close = &after_open[close_pos + 3..];
            let remainder = match after_close.find('\n') {
                Some(idx) => &after_close[idx + 1..],
                None => "",
            };
            (lines, remainder)
        }
        None => (after_open.split('\n').map(|l| l.to_string()).collect(), ""),
    }
}

/// Parses and word-wraps plain (non-code-block) text, one source line at a
/// time so manual newlines in the input are preserved as line breaks.
fn wrap_plain_text(text: &str, max_width: usize) -> Vec<FormattedLine> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(FormattedLine { spans: Vec::new(), width: 0 });
            continue;
        }
        let spans = parse_inline(paragraph, StyleFlags::NONE);
        lines.extend(word_wrap_spans(&spans, max_width));
    }
    lines
}

/// Recursively parses inline Markdown V2 styling into flat styled spans.
/// `base` is the style already in effect from an enclosing delimiter, so
/// nested styles (e.g. `*_both_*`) accumulate into a single combined mask.
fn parse_inline(input: &str, base: StyleFlags) -> Vec<FormattedSpan> {
    let chars: Vec<char> = input.chars().collect();
    let n = chars.len();
    let mut spans = Vec::new();
    let mut buf = String::new();
    let mut i = 0;

    macro_rules! flush_buf {
        () => {
            if !buf.is_empty() {
                spans.push(FormattedSpan { text: std::mem::take(&mut buf), style: base });
            }
        };
    }

    while i < n {
        let c = chars[i];

        if c == '\\' {
            // Only the markdown markup chars are escapable (matches mdstyle.cpp:62);
            // anything else (e.g. `C:\new\name`, `\d+`) keeps the backslash literal.
            if i + 1 < n && is_escapable(chars[i + 1]) {
                buf.push(chars[i + 1]);
                i += 2;
            } else {
                buf.push(c);
                i += 1;
            }
            continue;
        }

        if starts_with_at(&chars, i, "http://") || starts_with_at(&chars, i, "https://") {
            flush_buf!();
            let start = i;
            let mut j = i;
            while j < n && !chars[j].is_whitespace() {
                j += 1;
            }
            spans.push(FormattedSpan { text: chars[start..j].iter().collect(), style: base | StyleFlags::LINK });
            i = j;
            continue;
        }

        match c {
            '*' => {
                if is_valid_opener(&chars, i, 1) {
                    if let Some(end) = find_single(&chars, i + 1, '*') {
                        flush_buf!();
                        let inner: String = chars[i + 1..end].iter().collect();
                        spans.extend(parse_inline(&inner, base | StyleFlags::BOLD));
                        i = end + 1;
                        continue;
                    }
                }
            }
            '_' => {
                if i + 1 < n && chars[i + 1] == '_' {
                    if is_valid_opener(&chars, i, 2) {
                        if let Some(end) = find_double(&chars, i + 2, '_') {
                            flush_buf!();
                            let inner: String = chars[i + 2..end].iter().collect();
                            spans.extend(parse_inline(&inner, base | StyleFlags::UNDERLINE));
                            i = end + 2;
                            continue;
                        }
                    }
                } else if is_valid_opener(&chars, i, 1) {
                    if let Some(end) = find_single(&chars, i + 1, '_') {
                        flush_buf!();
                        let inner: String = chars[i + 1..end].iter().collect();
                        spans.extend(parse_inline(&inner, base | StyleFlags::ITALIC));
                        i = end + 1;
                        continue;
                    }
                }
            }
            '~' => {
                if is_valid_opener(&chars, i, 1) {
                    if let Some(end) = find_single(&chars, i + 1, '~') {
                        flush_buf!();
                        let inner: String = chars[i + 1..end].iter().collect();
                        spans.extend(parse_inline(&inner, base | StyleFlags::STRIKE));
                        i = end + 1;
                        continue;
                    }
                }
            }
            '`' => {
                if is_valid_opener(&chars, i, 1) {
                    if let Some(end) = find_single(&chars, i + 1, '`') {
                        flush_buf!();
                        // Code spans are verbatim: no nested markdown parsing inside.
                        spans.push(FormattedSpan {
                            text: chars[i + 1..end].iter().collect(),
                            style: base | StyleFlags::CODE,
                        });
                        i = end + 1;
                        continue;
                    }
                }
            }
            '|' => {
                if i + 1 < n && chars[i + 1] == '|' && is_valid_opener(&chars, i, 2) {
                    if let Some(end) = find_double(&chars, i + 2, '|') {
                        flush_buf!();
                        let inner: String = chars[i + 2..end].iter().collect();
                        spans.extend(parse_inline(&inner, base | StyleFlags::SPOILER));
                        i = end + 2;
                        continue;
                    }
                }
            }
            '[' => {
                if let Some(close_bracket) = find_single(&chars, i + 1, ']') {
                    if close_bracket + 1 < n && chars[close_bracket + 1] == '(' {
                        if let Some(close_paren) = find_single(&chars, close_bracket + 2, ')') {
                            flush_buf!();
                            let label: String = chars[i + 1..close_bracket].iter().collect();
                            spans.extend(parse_inline(&label, base | StyleFlags::LINK));
                            i = close_paren + 1;
                            continue;
                        }
                    }
                }
            }
            _ => {}
        }

        buf.push(c);
        i += 1;
    }

    flush_buf!();
    spans
}

/// Markup chars that `\` is allowed to escape (matches mdstyle.cpp:62). A
/// backslash before anything else (e.g. `C:\new\name`, `\d+`) stays literal.
const ESCAPABLE: [char; 8] = ['*', '_', '~', '`', '[', ']', '|', '\\'];

fn is_escapable(c: char) -> bool {
    ESCAPABLE.contains(&c)
}

/// Word-boundary guard for a delimiter opening at `open` (spanning
/// `delim_len` chars, e.g. 2 for `__`/`||`): must not be followed by
/// whitespace, and must not be preceded by an alphanumeric char — so
/// `5*3=15` and `my_file_name.txt` don't get treated as markup
/// (matches mdstyle.cpp FindClose's opener check).
fn is_valid_opener(chars: &[char], open: usize, delim_len: usize) -> bool {
    let n = chars.len();
    let after = open + delim_len;
    if after >= n || chars[after].is_whitespace() {
        return false;
    }
    if open > 0 && chars[open - 1].is_alphanumeric() {
        return false;
    }
    true
}

/// Finds the next unescaped occurrence of `delim` at or after `start` that is
/// a valid closer: not immediately after the opener, not preceded by
/// whitespace, and not followed by an alphanumeric char (matches mdstyle.cpp
/// FindClose's closer check). `start` is not itself examined as a closer.
fn find_single(chars: &[char], start: usize, delim: char) -> Option<usize> {
    let n = chars.len();
    let mut i = start;
    while i < n {
        if chars[i] == '\\' && i + 1 < n && is_escapable(chars[i + 1]) {
            i += 2;
            continue;
        }
        if chars[i] == delim
            && i > start
            && !chars[i - 1].is_whitespace()
            && (i + 1 >= n || !chars[i + 1].is_alphanumeric())
        {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Same as `find_single`, but the closing delimiter is `delim` repeated twice.
fn find_double(chars: &[char], start: usize, delim: char) -> Option<usize> {
    let n = chars.len();
    let mut i = start;
    while i + 1 < n {
        if chars[i] == '\\' && i + 1 < n && is_escapable(chars[i + 1]) {
            i += 2;
            continue;
        }
        if chars[i] == delim
            && chars[i + 1] == delim
            && i > start
            && !chars[i - 1].is_whitespace()
            && (i + 2 >= n || !chars[i + 2].is_alphanumeric())
        {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn starts_with_at(chars: &[char], i: usize, pat: &str) -> bool {
    let pat: Vec<char> = pat.chars().collect();
    i + pat.len() <= chars.len() && chars[i..i + pat.len()] == pat[..]
}

fn char_width(c: char) -> usize {
    UnicodeWidthChar::width(c).unwrap_or(0)
}

fn line_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

/// Greedy word-wraps a sequence of styled spans to `max_width` columns,
/// re-grouping each output line's characters back into style-runs.
fn word_wrap_spans(spans: &[FormattedSpan], max_width: usize) -> Vec<FormattedLine> {
    let atoms: Vec<(char, StyleFlags)> =
        spans.iter().flat_map(|s| s.text.chars().map(move |c| (c, s.style))).collect();

    if atoms.is_empty() {
        return vec![FormattedLine { spans: Vec::new(), width: 0 }];
    }

    let max_width = max_width.max(1);
    let mut lines = Vec::new();
    let mut current: Vec<(char, StyleFlags)> = Vec::new();
    let mut current_width = 0usize;
    let mut idx = 0;

    while idx < atoms.len() {
        if atoms[idx].0.is_whitespace() {
            let w = char_width(atoms[idx].0);
            if current_width == 0 {
                idx += 1; // drop leading whitespace on a new line
                continue;
            }
            if current_width + w <= max_width {
                current.push(atoms[idx]);
                current_width += w;
            } else {
                lines.push(finalize_line(std::mem::take(&mut current)));
                current_width = 0; // the space that caused the wrap is dropped
            }
            idx += 1;
            continue;
        }

        let start = idx;
        while idx < atoms.len() && !atoms[idx].0.is_whitespace() {
            idx += 1;
        }
        let word = &atoms[start..idx];
        let word_width: usize = word.iter().map(|(c, _)| char_width(*c)).sum();

        if current_width > 0 && current_width + word_width > max_width {
            lines.push(finalize_line(std::mem::take(&mut current)));
            current_width = 0;
        }
        current.extend_from_slice(word);
        current_width += word_width;
    }

    if !current.is_empty() {
        lines.push(finalize_line(current));
    }
    if lines.is_empty() {
        lines.push(FormattedLine { spans: Vec::new(), width: 0 });
    }
    lines
}

fn finalize_line(atoms: Vec<(char, StyleFlags)>) -> FormattedLine {
    let mut spans: Vec<FormattedSpan> = Vec::new();
    let mut width = 0usize;
    for (c, style) in atoms {
        width += char_width(c);
        match spans.last_mut() {
            Some(last) if last.style == style => last.text.push(c),
            _ => spans.push(FormattedSpan { text: c.to_string(), style }),
        }
    }
    FormattedLine { spans, width }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_inline_markdown_styles() {
        let lines = wrap_markdown("*bold* _italic_ ~strike~ `code`", 80);
        assert_eq!(lines.len(), 1);
        let spans = &lines[0].spans;
        assert_eq!(spans[0].text, "bold");
        assert!(spans[0].style.contains(StyleFlags::BOLD));
        assert_eq!(spans[1].text, " ");
        assert_eq!(spans[2].text, "italic");
        assert!(spans[2].style.contains(StyleFlags::ITALIC));
        assert_eq!(spans[4].text, "strike");
        assert!(spans[4].style.contains(StyleFlags::STRIKE));
        assert_eq!(spans[6].text, "code");
        assert!(spans[6].style.contains(StyleFlags::CODE));
    }

    #[test]
    fn parses_nested_bold_and_italic() {
        let lines = wrap_markdown("*_both_*", 80);
        assert_eq!(lines[0].spans[0].text, "both");
        assert!(lines[0].spans[0].style.contains(StyleFlags::BOLD | StyleFlags::ITALIC));
    }

    #[test]
    fn preserves_code_blocks_verbatim() {
        let md = "```rust\nfn main() {\n    println!(\"hi\");\n}\n```";
        let lines = wrap_markdown(md, 80);
        assert!(lines.len() >= 3);
        assert!(lines[0].spans[0].style.contains(StyleFlags::CODE_BLOCK));
        assert_eq!(lines[1].spans[0].text, "    println!(\"hi\");");
    }

    #[test]
    fn handles_escaped_characters() {
        let lines = wrap_markdown(r"\*not bold\* and \_not italic\_", 80);
        assert_eq!(lines[0].spans[0].text, "*not bold* and _not italic_");
        assert_eq!(lines[0].spans[0].style, StyleFlags::NONE);
    }

    #[test]
    fn wraps_long_styled_lines_preserving_style() {
        let text = "*this is a very long bold sentence that needs to wrap across lines*";
        let lines = wrap_markdown(text, 20);
        assert!(lines.len() > 1);
        for line in &lines {
            for span in &line.spans {
                assert!(span.style.contains(StyleFlags::BOLD));
            }
        }
    }

    #[test]
    fn parses_underline_and_spoiler() {
        let lines = wrap_markdown("__under__ ||secret||", 80);
        let spans = &lines[0].spans;
        assert_eq!(spans[0].text, "under");
        assert!(spans[0].style.contains(StyleFlags::UNDERLINE));
        assert_eq!(spans[2].text, "secret");
        assert!(spans[2].style.contains(StyleFlags::SPOILER));
    }

    #[test]
    fn parses_link_and_preserves_raw_url() {
        let lines = wrap_markdown("[nchat](https://example.com) https://example.org/path_with_underscore", 80);
        let spans = &lines[0].spans;
        assert_eq!(spans[0].text, "nchat");
        assert!(spans[0].style.contains(StyleFlags::LINK));
        assert_eq!(spans[2].text, "https://example.org/path_with_underscore");
        assert!(spans[2].style.contains(StyleFlags::LINK));
    }

    #[test]
    fn preserves_manual_newlines_outside_code_blocks() {
        let lines = wrap_markdown("first line\nsecond line", 80);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].spans[0].text, "first line");
        assert_eq!(lines[1].spans[0].text, "second line");
    }

    #[test]
    fn asterisk_between_digits_is_not_bold() {
        let lines = wrap_markdown("5*3=15 and 2*6=12", 80);
        assert_eq!(lines[0].spans.len(), 1);
        assert_eq!(lines[0].spans[0].text, "5*3=15 and 2*6=12");
        assert_eq!(lines[0].spans[0].style, StyleFlags::NONE);
    }

    #[test]
    fn underscore_in_filename_is_not_italic() {
        let lines = wrap_markdown("my_file_name.txt here", 80);
        assert_eq!(lines[0].spans.len(), 1);
        assert_eq!(lines[0].spans[0].text, "my_file_name.txt here");
        assert_eq!(lines[0].spans[0].style, StyleFlags::NONE);
    }

    #[test]
    fn backslash_before_non_markup_char_stays_literal() {
        let lines = wrap_markdown(r"C:\new\name", 80);
        assert_eq!(lines[0].spans.len(), 1);
        assert_eq!(lines[0].spans[0].text, r"C:\new\name");
        assert_eq!(lines[0].spans[0].style, StyleFlags::NONE);

        let lines = wrap_markdown(r"\d+ is a regex", 80);
        assert_eq!(lines[0].spans[0].text, r"\d+ is a regex");
    }

    #[test]
    fn valid_markup_still_parses_with_boundary_guards() {
        let lines = wrap_markdown("*bold* and _italic_", 80);
        let spans = &lines[0].spans;
        assert_eq!(spans[0].text, "bold");
        assert!(spans[0].style.contains(StyleFlags::BOLD));
        assert_eq!(spans[2].text, "italic");
        assert!(spans[2].style.contains(StyleFlags::ITALIC));
    }
}
