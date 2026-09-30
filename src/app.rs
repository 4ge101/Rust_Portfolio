use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::ascii::{Portrait, PortraitLoad};
use crate::commands::{self, Command};
use crate::data::{Loaded, Portfolio};
use crate::events::Action;
use crate::opener::OpenResult;
use crate::screens::{self, ItemAction, Screen};
use crate::theme::Theme;

const INFO_TTL: Duration = Duration::from_secs(4);
const ERROR_TTL: Duration = Duration::from_secs(8);
const MAX_COMMAND_LEN: usize = 48;

#[derive(Debug, PartialEq, Eq)]
pub enum Effect {
    OpenUrl(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Normal,
    Command(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Info,
    Error,
}

#[derive(Debug, Clone)]
pub struct Status {
    pub text: String,
    pub kind: StatusKind,
    expires: Instant,
}

#[derive(Debug, Default)]
pub struct Diagnostics {
    pub warnings: Vec<String>,
    pub portrait_source: Option<String>,
    pub portrait_note: Option<String>,
    pub data_dir: Option<PathBuf>,
}

pub struct App {
    pub data: Portfolio,
    pub theme: Theme,
    pub screen: Screen,
    pub sel: usize,
    pub scroll: usize,
    pub follow: bool,
    pub viewport_rows: usize,
    pub input: Input,
    pub status: Option<Status>,
    pub portrait: Option<Portrait>,
    pub invert: bool,
    pub diagnostics: Diagnostics,
    pub should_quit: bool,
    help_return: Screen,
}

impl App {
    pub fn new(
        loaded: Loaded,
        theme: Theme,
        portrait: PortraitLoad,
        data_dir: Option<PathBuf>,
    ) -> Self {
        let config = loaded.portfolio.profile.portrait.clone();
        let mut app = Self {
            data: loaded.portfolio,
            theme,
            screen: Screen::Home,
            sel: 0,
            scroll: 0,
            follow: true,
            viewport_rows: 10,
            input: Input::Normal,
            status: None,
            portrait: portrait.portrait,
            invert: config.invert,
            diagnostics: Diagnostics {
                warnings: loaded.warnings,
                portrait_source: portrait.source,
                portrait_note: portrait.note,
                data_dir,
            },
            should_quit: false,
            help_return: Screen::Home,
        };

        if let Some(first) = app.diagnostics.warnings.first() {
            let extra = app.diagnostics.warnings.len() - 1;
            let suffix = if extra > 0 {
                format!(" (+{extra} more)")
            } else {
                String::new()
            };
            let text = format!("config: {first}{suffix}");
            app.set_status(StatusKind::Error, text);
        }
        app
    }

    pub fn in_command_mode(&self) -> bool {
        matches!(self.input, Input::Command(_))
    }

    pub fn set_status(&mut self, kind: StatusKind, text: impl Into<String>) {
        let ttl = match kind {
            StatusKind::Info => INFO_TTL,
            StatusKind::Error => ERROR_TTL,
        };
        self.status = Some(Status {
            text: text.into(),
            kind,
            expires: Instant::now() + ttl,
        });
    }

    pub fn expire_status(&mut self, now: Instant) -> bool {
        if self.status.as_ref().is_some_and(|s| now >= s.expires) {
            self.status = None;
            return true;
        }
        false
    }

    pub fn on_open_result(&mut self, result: OpenResult) {
        match result {
            Ok(url) => self.set_status(StatusKind::Info, format!("Opened {url}")),
            Err(message) => self.set_status(StatusKind::Error, message),
        }
    }

    pub fn update(&mut self, action: Action) -> Option<Effect> {
        if let Input::Command(buffer) = &mut self.input {
            match action {
                Action::Type(c) if !c.is_control() && buffer.chars().count() < MAX_COMMAND_LEN => {
                    buffer.push(c);
                }
                Action::Backspace => {
                    if buffer.pop().is_none() {
                        self.input = Input::Normal;
                    }
                }
                Action::Cancel => self.input = Input::Normal,
                Action::Submit => {
                    let text = std::mem::take(buffer);
                    self.input = Input::Normal;
                    return self.run_command(&text);
                }
                Action::Quit => self.should_quit = true,
                _ => {}
            }
            return None;
        }

        match action {
            Action::Up => self.step(-1),
            Action::Down => self.step(1),
            Action::PageUp => self.page(false),
            Action::PageDown => self.page(true),
            Action::First => self.jump(false),
            Action::Last => self.jump(true),
            Action::Select => return self.select(),
            Action::Back => self.back(),
            Action::Quit => self.should_quit = true,
            Action::Goto(screen) => self.navigate(screen),
            Action::Help => self.navigate(Screen::Help),
            Action::OpenCommand => {
                self.status = None;
                self.input = Input::Command(String::new());
            }
            Action::Type(_) | Action::Backspace | Action::Submit | Action::Cancel => {}
        }
        None
    }

    fn run_command(&mut self, input: &str) -> Option<Effect> {
        match commands::parse(input) {
            Ok(Command::Goto(screen)) => self.navigate(screen),
            Ok(Command::Help) => self.navigate(Screen::Help),
            Ok(Command::Quit) => self.should_quit = true,
            Err(message) => self.set_status(StatusKind::Error, message),
        }
        None
    }

    fn navigate(&mut self, screen: Screen) {
        if screen == Screen::Help && self.screen != Screen::Help {
            self.help_return = self.screen;
        }
        self.screen = screen;
        self.sel = 0;
        self.scroll = 0;
        self.follow = true;
    }

    fn back(&mut self) {
        match self.screen {
            Screen::Home => {}
            Screen::ProjectDetail(index) => {
                self.navigate(Screen::Projects);
                self.sel = index.min(self.data.projects.len().saturating_sub(1));
            }
            Screen::Help => {
                let target = self.help_return;
                self.navigate(target);
            }
            other => {
                self.navigate(Screen::Home);
                self.sel = other.menu_index().unwrap_or(0);
            }
        }
    }

    fn item_count(&self) -> usize {
        screens::actions(self).len()
    }

    fn step(&mut self, delta: isize) {
        let count = self.item_count();
        if count > 0 {
            self.sel = self.sel.saturating_add_signed(delta).min(count - 1);
            self.follow = true;
        } else {
            self.scroll = self.scroll.saturating_add_signed(delta);
        }
    }

    fn page(&mut self, down: bool) {
        let page = self.viewport_rows.saturating_sub(1).max(1);
        self.scroll = if down {
            self.scroll.saturating_add(page)
        } else {
            self.scroll.saturating_sub(page)
        };
        self.follow = false;
    }

    fn jump(&mut self, to_end: bool) {
        let count = self.item_count();
        if count > 0 {
            self.sel = if to_end { count - 1 } else { 0 };
            self.follow = true;
        } else {
            self.scroll = if to_end { usize::MAX } else { 0 };
        }
    }

    fn select(&mut self) -> Option<Effect> {
        let action = screens::actions(self).into_iter().nth(self.sel)?;
        match action {
            ItemAction::Goto(screen) => self.navigate(screen),
            ItemAction::OpenProject(index) => self.navigate(Screen::ProjectDetail(index)),
            ItemAction::Back => self.back(),
            ItemAction::OpenUrl(url) => {
                self.set_status(StatusKind::Info, format!("Opening {url} ..."));
                return Some(Effect::OpenUrl(url));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data;

    fn app_with(edit: impl FnOnce(&mut Portfolio)) -> App {
        let mut loaded = data::load(None);
        edit(&mut loaded.portfolio);
        App::new(
            loaded,
            Theme::from_env(None, None, None, None),
            PortraitLoad {
                portrait: None,
                note: None,
                source: None,
            },
            None,
        )
    }

    fn linked() -> App {
        app_with(|p| {
            p.projects[0].github = Some("https://github.com/x/robot".to_owned());
            p.projects[0].website = Some("https://example.com".to_owned());
        })
    }

    #[test]
    fn menu_selection_opens_each_section() {
        for (i, screen) in Screen::MENU.iter().enumerate() {
            let mut app = linked();
            for _ in 0..i {
                app.update(Action::Down);
            }
            assert_eq!(app.update(Action::Select), None);
            assert_eq!(app.screen, *screen);
        }
    }

    #[test]
    fn number_keys_jump_and_esc_returns_to_the_same_menu_row() {
        let mut app = linked();
        app.update(Action::Goto(Screen::Skills));
        assert_eq!(app.screen, Screen::Skills);
        app.update(Action::Back);
        assert_eq!(app.screen, Screen::Home);
        assert_eq!(Screen::MENU[app.sel], Screen::Skills);
    }

    #[test]
    fn selection_is_clamped_at_both_ends() {
        let mut app = linked();
        app.update(Action::Up);
        assert_eq!(app.sel, 0);
        for _ in 0..50 {
            app.update(Action::Down);
        }
        assert_eq!(app.sel, Screen::MENU.len() - 1);
    }

    #[test]
    fn project_detail_flow_and_links() {
        let mut app = linked();
        app.update(Action::Goto(Screen::Projects));
        app.update(Action::Select);
        assert_eq!(app.screen, Screen::ProjectDetail(0));

        assert_eq!(
            app.update(Action::Select),
            Some(Effect::OpenUrl("https://github.com/x/robot".to_owned()))
        );
        app.update(Action::Down);
        assert_eq!(
            app.update(Action::Select),
            Some(Effect::OpenUrl("https://example.com".to_owned()))
        );

        app.update(Action::Last);
        app.update(Action::Select);
        assert_eq!(app.screen, Screen::Projects);
        assert_eq!(app.sel, 0);
    }

    #[test]
    fn project_without_links_only_offers_back() {
        let mut app = app_with(|p| {
            p.projects[0].github = None;
            p.projects[0].website = None;
        });
        app.update(Action::Goto(Screen::ProjectDetail(0)));
        assert_eq!(screens::actions(&app), vec![ItemAction::Back]);
    }

    #[test]
    fn help_returns_to_where_it_was_opened() {
        let mut app = linked();
        app.update(Action::Goto(Screen::Contact));
        app.update(Action::Help);
        assert_eq!(app.screen, Screen::Help);
        app.update(Action::Back);
        assert_eq!(app.screen, Screen::Contact);
    }

    #[test]
    fn esc_on_home_does_nothing_and_q_quits() {
        let mut app = linked();
        app.update(Action::Back);
        assert_eq!(app.screen, Screen::Home);
        assert!(!app.should_quit);
        app.update(Action::Quit);
        assert!(app.should_quit);
    }

    #[test]
    fn command_palette_runs_commands() {
        let mut app = linked();
        app.update(Action::OpenCommand);
        assert!(app.in_command_mode());
        for c in "proj".chars() {
            app.update(Action::Type(c));
        }
        app.update(Action::Submit);
        assert!(!app.in_command_mode());
        assert_eq!(app.screen, Screen::Projects);

        app.update(Action::OpenCommand);
        for c in "nonsense".chars() {
            app.update(Action::Type(c));
        }
        app.update(Action::Submit);
        assert_eq!(app.status.as_ref().unwrap().kind, StatusKind::Error);
        assert_eq!(app.screen, Screen::Projects);

        app.update(Action::OpenCommand);
        for c in "quit".chars() {
            app.update(Action::Type(c));
        }
        app.update(Action::Submit);
        assert!(app.should_quit);
    }

    #[test]
    fn command_prompt_cancel_and_backspace() {
        let mut app = linked();
        app.update(Action::OpenCommand);
        app.update(Action::Type('x'));
        app.update(Action::Cancel);
        assert!(!app.in_command_mode());

        app.update(Action::OpenCommand);
        app.update(Action::Backspace);
        assert!(
            !app.in_command_mode(),
            "backspace on empty prompt closes it"
        );
    }

    #[test]
    fn digits_are_text_inside_the_prompt() {
        let mut app = linked();
        app.update(Action::OpenCommand);
        app.update(Action::Type('1'));
        assert_eq!(app.screen, Screen::Home);
        assert_eq!(app.input, Input::Command("1".to_owned()));
    }

    #[test]
    fn scrolling_screens_scroll_and_never_underflow() {
        let mut app = linked();
        app.update(Action::Goto(Screen::About));
        app.update(Action::Up);
        assert_eq!(app.scroll, 0);
        app.update(Action::Down);
        assert_eq!(app.scroll, 1);
        app.update(Action::Last);
        assert_eq!(app.scroll, usize::MAX);
        app.update(Action::First);
        assert_eq!(app.scroll, 0);
    }

    #[test]
    fn status_messages_expire() {
        let mut app = linked();
        app.set_status(StatusKind::Info, "hello");
        assert!(!app.expire_status(Instant::now()));
        assert!(app.expire_status(Instant::now() + Duration::from_secs(60)));
        assert!(app.status.is_none());
    }

    #[test]
    fn configuration_warnings_surface_at_startup() {
        let mut loaded = data::load(None);
        loaded.warnings.push("skills.toml: broken".to_owned());
        let app = App::new(
            loaded,
            Theme::from_env(None, None, None, None),
            PortraitLoad {
                portrait: None,
                note: None,
                source: None,
            },
            None,
        );
        assert!(app.status.unwrap().text.contains("skills.toml"));
    }
}
