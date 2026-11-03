pub fn format_list_time(time_ms: i64, now_sec: i64) -> String {
    if time_ms <= 0 {
        return String::new();
    }
    let msg_sec = time_ms / 1000;

    let mut msg_tm = std::mem::MaybeUninit::<libc::tm>::uninit();
    let mut now_tm = std::mem::MaybeUninit::<libc::tm>::uninit();

    unsafe {
        libc::localtime_r(&msg_sec, msg_tm.as_mut_ptr());
        libc::localtime_r(&now_sec, now_tm.as_mut_ptr());
        let m = msg_tm.assume_init();
        let n = now_tm.assume_init();

        if m.tm_year == n.tm_year && m.tm_yday == n.tm_yday {
            format!("{:02}:{:02}", m.tm_hour, m.tm_min)
        } else {
            format!("{:02}-{:02}", m.tm_mon + 1, m.tm_mday)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_today_as_hh_mm() {
        let now = 1700000000;
        let formatted = format_list_time(now * 1000, now);
        assert_eq!(formatted.len(), 5);
        assert!(formatted.contains(':'));
    }

    #[test]
    fn formats_past_day_as_mm_dd() {
        let now = 1700000000;
        let past = now - 86400 * 2;
        let formatted = format_list_time(past * 1000, now);
        assert_eq!(formatted.len(), 5);
        assert!(formatted.contains('-'));
    }
}
