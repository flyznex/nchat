use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct ListChat {
    pub profile_id: String,
    pub chat_id: String,
    pub name: String,
    pub group: String,
    pub is_unread: bool,
    pub is_muted: bool,
    pub is_pinned: bool,
    pub last_message_time: i64,
    pub preview: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListRow {
    Header { group: String, collapsed: bool },
    Chat(usize),
}

pub fn build_rows(
    chats: &[ListChat],
    filter_text: &str,
    unread_only: bool,
    collapsed: &HashSet<String>,
) -> Vec<ListRow> {
    let lower_filter = filter_text.to_lowercase();
    let visible_indices: Vec<usize> = chats
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            if unread_only && !c.is_unread {
                return false;
            }
            if !lower_filter.is_empty() && !c.name.to_lowercase().contains(&lower_filter) {
                return false;
            }
            true
        })
        .map(|(i, _)| i)
        .collect();

    if visible_indices.is_empty() {
        return Vec::new();
    }

    // Grouping applies when filter text is empty and there are >= 2 distinct groups
    let mut distinct_groups = HashSet::new();
    for &idx in &visible_indices {
        distinct_groups.insert(&chats[idx].group);
    }
    let grouped = filter_text.is_empty() && distinct_groups.len() >= 2;

    let mut rows = Vec::new();
    let mut current_group: Option<&str> = None;

    for &idx in &visible_indices {
        let chat = &chats[idx];
        if grouped {
            if current_group != Some(&chat.group) {
                current_group = Some(&chat.group);
                let is_collapsed = collapsed.contains(&chat.group);
                rows.push(ListRow::Header {
                    group: chat.group.clone(),
                    collapsed: is_collapsed,
                });
            }
            if collapsed.contains(&chat.group) {
                continue;
            }
        }
        rows.push(ListRow::Chat(idx));
    }

    rows
}

pub fn step(rows: &[ListRow], current_row: Option<usize>, dir: i32) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }
    if dir > 0 {
        let start = current_row.map(|c| c + 1).unwrap_or(0);
        for i in start..rows.len() {
            if matches!(rows[i], ListRow::Chat(_)) {
                return Some(i);
            }
        }
    } else if dir < 0 {
        let start = current_row.unwrap_or(rows.len());
        for i in (0..start).rev() {
            if matches!(rows[i], ListRow::Chat(_)) {
                return Some(i);
            }
        }
    }
    current_row
}

pub fn nearest_chat_row(rows: &[ListRow], target_chat_idx: usize) -> Option<usize> {
    // Exact or closest following chat row
    for (r_idx, r) in rows.iter().enumerate() {
        if let ListRow::Chat(c_idx) = r {
            if *c_idx >= target_chat_idx {
                return Some(r_idx);
            }
        }
    }
    // Otherwise the last chat row
    step(rows, None, -1)
}

pub fn scroll_first(lines: &[usize], mut first: usize, sel: Option<usize>, height: usize, free: bool) -> usize {
    if lines.is_empty() {
        return 0;
    }
    let last = lines.len() - 1;
    first = first.min(last);
    let span = |a: usize, b: usize| -> usize { lines[a..=b].iter().sum() };

    if free {
        while first > 0 && span(first - 1, last) <= height {
            first -= 1;
        }
    } else if let Some(s) = sel {
        if s < first {
            first = s;
        }
        while first < s && span(first, s) > height {
            first += 1;
        }
    }

    first
}

pub fn scroll_thumb(total: usize, first: usize, visible: usize, track: usize) -> Option<(usize, usize)> {
    if track == 0 || visible == 0 || total <= visible {
        return None;
    }
    let clamped_first = first.min(total - visible);
    let len = 1.max((track * visible + total / 2) / total).min(track);
    let pos = ((track * clamped_first + total / 2) / total).min(track - len);
    Some((pos, len))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_chat(id: &str, group: &str, unread: bool) -> ListChat {
        ListChat {
            profile_id: "P".into(),
            chat_id: id.into(),
            name: format!("Name {id}"),
            group: group.into(),
            is_unread: unread,
            is_muted: false,
            is_pinned: false,
            last_message_time: 100,
            preview: "hello".into(),
        }
    }

    #[test]
    fn groups_rows_when_multiple_distinct_groups() {
        let chats = vec![
            sample_chat("1", "Pinned", false),
            sample_chat("2", "Pinned", true),
            sample_chat("3", "Telegram", false),
        ];
        let collapsed = HashSet::new();
        let rows = build_rows(&chats, "", false, &collapsed);
        assert_eq!(rows.len(), 5); // Header Pinned + 2 chats + Header Telegram + 1 chat
        assert_eq!(rows[0], ListRow::Header { group: "Pinned".into(), collapsed: false });
        assert_eq!(rows[1], ListRow::Chat(0));
        assert_eq!(rows[2], ListRow::Chat(1));
        assert_eq!(rows[3], ListRow::Header { group: "Telegram".into(), collapsed: false });
        assert_eq!(rows[4], ListRow::Chat(2));
    }

    #[test]
    fn collapses_group_rows() {
        let chats = vec![
            sample_chat("1", "Pinned", false),
            sample_chat("2", "Telegram", false),
        ];
        let mut collapsed = HashSet::new();
        collapsed.insert("Pinned".into());
        let rows = build_rows(&chats, "", false, &collapsed);
        assert_eq!(rows.len(), 3); // Header Pinned (collapsed) + Header Telegram + Chat 2
        assert_eq!(rows[0], ListRow::Header { group: "Pinned".into(), collapsed: true });
        assert_eq!(rows[1], ListRow::Header { group: "Telegram".into(), collapsed: false });
        assert_eq!(rows[2], ListRow::Chat(1));
    }

    #[test]
    fn step_skips_header_rows() {
        let rows = vec![
            ListRow::Header { group: "H1".into(), collapsed: false },
            ListRow::Chat(0),
            ListRow::Header { group: "H2".into(), collapsed: false },
            ListRow::Chat(1),
        ];
        assert_eq!(step(&rows, None, 1), Some(1));
        assert_eq!(step(&rows, Some(1), 1), Some(3));
        assert_eq!(step(&rows, Some(3), -1), Some(1));
    }

    #[test]
    fn scroll_first_uniform_single_line_rows_tracks_selection() {
        let lines = vec![1usize; 10];
        // sel ahead of view: first advances until sel fits within height
        assert_eq!(scroll_first(&lines, 0, Some(5), 3, false), 3);
        // sel before first: snaps back to sel
        assert_eq!(scroll_first(&lines, 5, Some(1), 3, false), 1);
        // sel already in view: first unchanged
        assert_eq!(scroll_first(&lines, 2, Some(3), 3, false), 2);
        // no selection: first clamped to last index only
        assert_eq!(scroll_first(&lines, 2, None, 3, false), 2);
    }

    #[test]
    fn scroll_first_variable_two_line_rows_respects_row_heights() {
        // rows alternate between 1 and 2 lines tall
        let lines = vec![1usize, 2, 1, 2, 1, 2, 1, 2];
        // selecting row 5 (height 2) with viewport height 4 should advance first
        // until span(first, 5) <= 4
        let first = scroll_first(&lines, 0, Some(5), 4, false);
        let span: usize = lines[first..=5].iter().sum();
        assert!(span <= 4);
        assert!(first <= 5);

        // sel before first still snaps directly to sel regardless of row heights
        assert_eq!(scroll_first(&lines, 6, Some(2), 4, false), 2);
    }

    #[test]
    fn scroll_first_free_scroll_pulls_back_while_span_fits() {
        let lines = vec![2usize, 1, 1, 2, 1];
        // free scroll ignores selection and pulls first back as long as the
        // span from (first - 1) to the end still fits in height
        let first = scroll_first(&lines, 4, Some(0), 4, true);
        assert!(first > 0);
        let span: usize = lines[first..].iter().sum();
        assert!(span <= 4);
        // one further back would overflow height
        if first > 0 {
            let overflow: usize = lines[first - 1..].iter().sum();
            assert!(overflow > 4);
        }

        // when everything fits, free scroll pulls all the way to 0
        let all_fits = vec![1usize, 1, 1];
        assert_eq!(scroll_first(&all_fits, 2, None, 10, true), 0);
    }

    #[test]
    fn scroll_first_empty_lines_returns_zero() {
        let lines: Vec<usize> = Vec::new();
        assert_eq!(scroll_first(&lines, 3, Some(1), 5, false), 0);
        assert_eq!(scroll_first(&lines, 3, None, 5, true), 0);
    }

    #[test]
    fn calculates_scroll_thumb_position_and_len() {
        // total 100, visible 20, track 20, first 0
        let (pos, len) = scroll_thumb(100, 0, 20, 20).unwrap();
        assert_eq!(pos, 0);
        assert_eq!(len, 4);

        // scrolled to end: first = 80
        let (pos, len) = scroll_thumb(100, 80, 20, 20).unwrap();
        assert_eq!(pos + len, 20);
    }
}
