use crate::colorconf::Theme;
use crate::layout::Layout;
use crate::model::UiModel;
use crate::nav::{Action, Focus, NavResolveResult, NavResolver};
use crate::protocol::Event;
use crate::render::{list, status, top};
use crate::uiconf::UiConfig;
use crossterm::event::{self, Event as CEvent, MouseEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{stdout, Stdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct TerminalGuard {
    mouse_enabled: bool,
}

impl TerminalGuard {
    pub fn init(mouse_enabled: bool) -> std::io::Result<Self> {
        enable_raw_mode()?;
        let mut out = stdout();
        out.execute(EnterAlternateScreen)?;
        if mouse_enabled {
            out.execute(event::EnableMouseCapture)?;
        }

        // Install panic hook to ensure terminal restoration
        let orig_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = disable_raw_mode();
            let _ = stdout().execute(LeaveAlternateScreen);
            let _ = stdout().execute(event::DisableMouseCapture);
            orig_hook(info);
        }));

        Ok(Self { mouse_enabled })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
        if self.mouse_enabled {
            let _ = stdout().execute(event::DisableMouseCapture);
        }
    }
}

pub struct App {
    pub model: UiModel,
    pub conf: UiConfig,
    pub theme: Theme,
    pub nav: NavResolver,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl App {
    pub fn new(conf: UiConfig, theme: Theme) -> std::io::Result<Self> {
        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)?;
        let vim_keys = conf.vim_keys;
        Ok(Self {
            model: UiModel::new(),
            conf,
            theme,
            nav: NavResolver::new(vim_keys),
            terminal,
        })
    }

    pub fn run(
        &mut self,
        rx: Receiver<String>,
        exit_flag: Arc<AtomicBool>,
    ) -> std::io::Result<()> {
        let mut last_draw = Instant::now();
        let frame_interval = Duration::from_millis(16); // ~60fps target

        while !exit_flag.load(Ordering::Relaxed) {
            // Drain incoming bridge events (coalesce high-frequency events)
            while let Ok(raw) = rx.try_recv() {
                if let Ok(ev) = serde_json::from_str::<Event>(&raw) {
                    self.model.apply_event(&ev);
                }
            }

            // Poll crossterm events (timeout up to 10ms)
            if event::poll(Duration::from_millis(10))? {
                match event::read()? {
                    CEvent::Key(k) => {
                        if let Some(norm) = crate::keyconf::normalize_crossterm_key(k) {
                            if norm == crate::keyconf::Key::ESC && self.model.filter_active {
                                self.model.filter_text.clear();
                                self.model.filter_active = false;
                                self.model.dirty = true;
                                continue;
                            }
                            if k.code == crossterm::event::KeyCode::Char('q')
                                && k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                            {
                                exit_flag.store(true, Ordering::Relaxed);
                                break;
                            }
                            match self.nav.resolve(norm, self.model.focus) {
                                NavResolveResult::Matched(act) => self.handle_action(act),
                                NavResolveResult::Pending => {}
                                NavResolveResult::Ignored => {}
                            }
                        }
                    }
                    CEvent::Mouse(m) => {
                        let size = self.terminal.size()?;
                        let layout = Layout::compute(size.width, size.height, &self.conf);
                        if let Some(focus) = layout.pane_at(m.column, m.row) {
                            self.model.focus = focus;
                        }
                        if m.kind == MouseEventKind::ScrollDown {
                            self.model.select_delta(1);
                        } else if m.kind == MouseEventKind::ScrollUp {
                            self.model.select_delta(-1);
                        }
                    }
                    CEvent::Resize(_, _) => {
                        self.model.dirty = true;
                    }
                    _ => {}
                }
            }

            // Draw at most once per frame interval if dirty
            let now = Instant::now();
            if self.model.dirty && now.duration_since(last_draw) >= frame_interval {
                self.draw()?;
                last_draw = now;
                self.model.dirty = false;
            }
        }
        Ok(())
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Down => self.model.select_delta(1),
            Action::Up => self.model.select_delta(-1),
            Action::FocusNext => {
                self.model.focus = match self.model.focus {
                    Focus::List => Focus::History,
                    Focus::History => Focus::Entry,
                    Focus::Entry => Focus::List,
                };
                self.model.dirty = true;
            }
            Action::FocusPrev => {
                self.model.focus = match self.model.focus {
                    Focus::List => Focus::Entry,
                    Focus::History => Focus::List,
                    Focus::Entry => Focus::History,
                };
                self.model.dirty = true;
            }
            Action::FocusLeft => {
                self.model.focus = Focus::List;
                self.model.dirty = true;
            }
            Action::FocusRight => {
                self.model.focus = Focus::History;
                self.model.dirty = true;
            }
            Action::ToggleGroup => self.model.toggle_group(),
            Action::UnreadOnly => {
                self.model.unread_only = !self.model.unread_only;
                self.model.dirty = true;
            }
            Action::None => {}
            _ => {}
        }
    }

    fn draw(&mut self) -> std::io::Result<()> {
        let conf = self.conf.clone();
        let theme = self.theme.clone();
        let model = &mut self.model;

        self.terminal.draw(|f| {
            let layout = Layout::compute(f.area().width, f.area().height, &conf);
            top::render_top(f, layout.top, model, &theme, &conf);
            status::render_status(f, layout.status, model, &theme, &conf);
            list::render_list(f, layout.list, model, &theme, &conf);
        })?;
        Ok(())
    }
}
