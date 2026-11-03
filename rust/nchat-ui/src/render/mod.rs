pub mod list;
pub mod status;
pub mod time;
pub mod top;

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
    fn renders_top_and_status_in_test_backend() {
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut model = UiModel::new();
        model.is_online = true;
        let theme = Theme::default();
        let conf = UiConfig::default();

        terminal
            .draw(|f| {
                top::render_top(f, Rect::new(0, 0, 40, 1), &model, &theme, &conf);
                status::render_status(f, Rect::new(0, 9, 40, 1), &model, &theme, &conf);
            })
            .unwrap();

        let buf = terminal.backend().buffer();
        let top_line: String = (0..40).map(|x| buf[(x, 0)].symbol()).collect();
        assert!(top_line.contains("nchat"));
        assert!(top_line.contains("Online"));
    }
}
