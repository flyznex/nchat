use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

pub fn fit(s: &str, target_width: usize) -> String {
    let current_width = width(s);
    if current_width == target_width {
        return s.to_string();
    }
    if current_width < target_width {
        let mut out = String::with_capacity(s.len() + (target_width - current_width));
        out.push_str(s);
        for _ in 0..(target_width - current_width) {
            out.push(' ');
        }
        return out;
    }

    // current_width > target_width
    if target_width == 0 {
        return String::new();
    }
    if target_width < 3 {
        // Target too narrow for ellipsis "..."
        let mut out = String::new();
        let mut w = 0;
        for c in s.chars() {
            let cw = c.width().unwrap_or(0);
            if w + cw > target_width {
                break;
            }
            out.push(c);
            w += cw;
        }
        while w < target_width {
            out.push(' ');
            w += 1;
        }
        return out;
    }

    let budget = target_width - 3;
    let mut out = String::new();
    let mut w = 0;
    for c in s.chars() {
        let cw = c.width().unwrap_or(0);
        if w + cw > budget {
            break;
        }
        out.push(c);
        w += cw;
    }
    out.push_str("...");
    w += 3;
    while w < target_width {
        out.push(' ');
        w += 1;
    }
    out
}

pub fn split_row(left: &str, right: &str, total_width: usize) -> String {
    let rw = width(right);
    if rw >= total_width {
        return fit(right, total_width);
    }
    let lw = total_width - rw;
    let fitted_left = fit(left, lw);
    format!("{fitted_left}{right}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fits_ascii_exact_and_shorter() {
        assert_eq!(fit("hello", 5), "hello");
        assert_eq!(fit("hi", 5), "hi   ");
    }

    #[test]
    fn fits_ascii_longer_with_ellipsis() {
        assert_eq!(fit("hello world", 8), "hello...");
        assert_eq!(fit("hello", 3), "...");
        assert_eq!(fit("hello", 2), "he");
        assert_eq!(fit("hello", 0), "");
    }

    #[test]
    fn fits_cjk_characters() {
        // "你好世界" = 4 chars, 8 cols width
        assert_eq!(width("你好"), 4);
        assert_eq!(fit("你好世界", 7), "你好...");
        assert_eq!(fit("你好世界", 8), "你好世界");
    }

    #[test]
    fn split_row_aligns_left_and_right() {
        assert_eq!(split_row("Alice", "12:00", 15), "Alice     12:00");
        assert_eq!(split_row("Alice", "*", 7), "Alice *");
    }
}
