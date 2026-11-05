use crate::colorconf::Theme;
use crate::model::UiModel;
use crate::nav::Focus;
use crate::uiconf::UiConfig;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render_help(
    f: &mut Frame,
    area: Rect,
    model: &UiModel,
    theme: &Theme,
    conf: &UiConfig,
) {
    if area.height == 0 || area.width == 0 || !conf.help_enabled {
        return;
    }

    let items = match model.focus {
        Focus::List => {
            if conf.vim_keys {
                " j/k Move | gg/G Ends | / Filter | u Unread | z Fold | l Open | Tab Focus | Ctrl-Q Quit"
            } else {
                " Down/Up Move | Home/End Ends | / Filter | u Unread | Enter Open | Tab Focus | Ctrl-Q Quit"
            }
        }
        Focus::History => {
            if conf.vim_keys {
                " j/k Move | Ctrl-d/u Scroll | G/gg Ends | / Find | Tab Focus | Ctrl-Q Quit"
            } else {
                " Down/Up Move | PgDn/PgUp Scroll | End/Home Ends | Tab Focus | Ctrl-Q Quit"
            }
        }
        Focus::Entry => {
            " Enter Send | Alt-Enter Newline | Alt-u Emoji | Alt-g Sticker | Ctrl-V Paste | Tab Focus"
        }
    };

    let p = Paragraph::new(items).style(theme.status);
    f.render_widget(p, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colorconf::Theme;
    use crate::model::UiModel;
    use crate::uiconf::UiConfig;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::Terminal;

    #[test]
    fn renders_help_bar_for_list_focus() {
        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        let model = UiModel::new();
        let theme = Theme::default();
        let conf = UiConfig::default();

        terminal
            .draw(|f| {
                render_help(f, Rect::new(0, 0, 80, 1), &model, &theme, &conf);
            })
            .unwrap();

        let buf = terminal.backend().buffer();
        let line: String = (0..80).map(|x| buf[(x, 0)].symbol()).collect();
        assert!(line.contains("Move"));
        assert!(line.contains("Focus"));
    }
}
