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
}
