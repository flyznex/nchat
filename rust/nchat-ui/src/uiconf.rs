use crate::conf::Conf;

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutMode {
    Sidebar,
    Classic,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListPosition {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct UiConfig {
    pub layout_mode: LayoutMode,
    pub list_enabled: bool,
    pub list_position: ListPosition,
    pub list_width: u16,
    pub entry_height: u16,
    pub top_enabled: bool,
    pub status_enabled: bool,
    pub help_enabled: bool,
    pub top_show_version: bool,
    pub mouse_enabled: bool,
    pub vim_keys: bool,
    pub unread_indicator: String,
    pub muted_indicate_unread: bool,
    pub muted_position_by_timestamp: bool,
    pub away_status_indication: bool,
    pub proxy_indicator: String,
    pub sent_align_right: bool,
    pub link_open_command: String,
    pub desktop_notify_enabled: bool,
    pub desktop_notify_active_current: bool,
    pub desktop_notify_active_noncurrent: bool,
    pub desktop_notify_inactive: bool,
    pub desktop_notify_connectivity: bool,
    pub desktop_notify_command: String,
    pub muted_notify_unread: bool,
    pub notify_every_unread: bool,
    pub terminal_bell_active: bool,
    pub terminal_bell_inactive: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            layout_mode: LayoutMode::Sidebar,
            list_enabled: true,
            list_position: ListPosition::Left,
            list_width: 30,
            entry_height: 4,
            top_enabled: true,
            status_enabled: true,
            help_enabled: true,
            top_show_version: false,
            mouse_enabled: false,
            vim_keys: true,
            unread_indicator: "*".into(),
            muted_indicate_unread: true,
            muted_position_by_timestamp: true,
            away_status_indication: false,
            proxy_indicator: "🔒".into(),
            sent_align_right: true,
            link_open_command: String::new(),
            desktop_notify_enabled: false,
            desktop_notify_active_current: false,
            desktop_notify_active_noncurrent: true,
            desktop_notify_inactive: true,
            desktop_notify_connectivity: true,
            desktop_notify_command: String::new(),
            muted_notify_unread: false,
            notify_every_unread: true,
            terminal_bell_active: false,
            terminal_bell_inactive: true,
        }
    }
}

impl UiConfig {
    pub fn from_conf(conf: &Conf) -> Self {
        let layout_mode = match conf.str_or("layout_mode", "sidebar") {
            "classic" => LayoutMode::Classic,
            _ => LayoutMode::Sidebar,
        };
        let list_position = match conf.str_or("list_position", "left") {
            "right" => ListPosition::Right,
            _ => ListPosition::Left,
        };

        Self {
            layout_mode,
            list_enabled: conf.bool_or("list_enabled", true),
            list_position,
            list_width: conf.num_or("list_width", 30),
            entry_height: conf.num_or("entry_height", 4),
            top_enabled: conf.bool_or("top_enabled", true),
            status_enabled: conf.bool_or("status_enabled", true),
            help_enabled: conf.bool_or("help_enabled", true),
            top_show_version: conf.bool_or("top_show_version", false),
            mouse_enabled: conf.bool_or("mouse_enabled", false),
            vim_keys: conf.bool_or("vim_keys", true),
            unread_indicator: conf.str_or("unread_indicator", "*").to_string(),
            muted_indicate_unread: conf.bool_or("muted_indicate_unread", true),
            muted_position_by_timestamp: conf.bool_or("muted_position_by_timestamp", true),
            away_status_indication: conf.bool_or("away_status_indication", false),
            proxy_indicator: conf.str_or("proxy_indicator", "🔒").to_string(),
            sent_align_right: conf.bool_or("sent_align_right", true),
            link_open_command: conf.str_or("link_open_command", "").to_string(),
            desktop_notify_enabled: conf.bool_or("desktop_notify_enabled", false),
            desktop_notify_active_current: conf.bool_or("desktop_notify_active_current", false),
            desktop_notify_active_noncurrent: conf.bool_or("desktop_notify_active_noncurrent", true),
            desktop_notify_inactive: conf.bool_or("desktop_notify_inactive", true),
            desktop_notify_connectivity: conf.bool_or("desktop_notify_connectivity", true),
            desktop_notify_command: conf.str_or("desktop_notify_command", "").to_string(),
            muted_notify_unread: conf.bool_or("muted_notify_unread", false),
            notify_every_unread: conf.bool_or("notify_every_unread", true),
            terminal_bell_active: conf.bool_or("terminal_bell_active", false),
            terminal_bell_inactive: conf.bool_or("terminal_bell_inactive", true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_uiconfig_values() {
        let conf = Conf::parse("layout_mode=classic\nlist_width=32\nvim_keys=0\n");
        let uic = UiConfig::from_conf(&conf);
        assert_eq!(uic.layout_mode, LayoutMode::Classic);
        assert_eq!(uic.list_width, 32);
        assert_eq!(uic.vim_keys, false);
        assert_eq!(uic.top_enabled, true); // default
    }

    #[test]
    fn parses_notification_configs_and_defaults() {
        // Test defaults
        let def_conf = Conf::default();
        let uic_def = UiConfig::from_conf(&def_conf);
        assert_eq!(uic_def.desktop_notify_enabled, false);
        assert_eq!(uic_def.desktop_notify_active_current, false);
        assert_eq!(uic_def.desktop_notify_active_noncurrent, true);
        assert_eq!(uic_def.desktop_notify_inactive, true);
        assert_eq!(uic_def.desktop_notify_connectivity, true);
        assert_eq!(uic_def.desktop_notify_command, "");
        assert_eq!(uic_def.muted_notify_unread, false);
        assert_eq!(uic_def.notify_every_unread, true);
        assert_eq!(uic_def.terminal_bell_active, false);
        assert_eq!(uic_def.terminal_bell_inactive, true);

        // Test custom values
        let conf = Conf::parse(
            "desktop_notify_enabled=1\n\
             desktop_notify_active_current=1\n\
             desktop_notify_active_noncurrent=0\n\
             desktop_notify_inactive=0\n\
             desktop_notify_connectivity=0\n\
             desktop_notify_command=notify-send 'MyChat' '%1: %2'\n\
             muted_notify_unread=1\n\
             notify_every_unread=0\n\
             terminal_bell_active=1\n\
             terminal_bell_inactive=0\n",
        );
        let uic = UiConfig::from_conf(&conf);
        assert_eq!(uic.desktop_notify_enabled, true);
        assert_eq!(uic.desktop_notify_active_current, true);
        assert_eq!(uic.desktop_notify_active_noncurrent, false);
        assert_eq!(uic.desktop_notify_inactive, false);
        assert_eq!(uic.desktop_notify_connectivity, false);
        assert_eq!(uic.desktop_notify_command, "notify-send 'MyChat' '%1: %2'");
        assert_eq!(uic.muted_notify_unread, true);
        assert_eq!(uic.notify_every_unread, false);
        assert_eq!(uic.terminal_bell_active, true);
        assert_eq!(uic.terminal_bell_inactive, false);
    }
}
