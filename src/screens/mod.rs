use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::app::App;
use crate::text;
use crate::theme::Theme;

pub mod about;
pub mod achievements;
pub mod contact;
pub mod experience;
pub mod github;
pub mod help;
pub mod home;
pub mod projects;
pub mod skills;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    About,
    Projects,
    ProjectDetail(usize),
    Skills,
    Experience,
    Achievements,
    GitHub,
    Contact,
    Help,
}

impl Screen {
    pub const MENU: [Screen; 7] = [
        Screen::About,
        Screen::Projects,
        Screen::Skills,
        Screen::Experience,
        Screen::Achievements,
        Screen::GitHub,
        Screen::Contact,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Screen::Home => "Home",
            Screen::About => "About",
            Screen::Projects | Screen::ProjectDetail(_) => "Projects",
            Screen::Skills => "Skills",
            Screen::Experience => "Experience",
            Screen::Achievements => "Achievements",
            Screen::GitHub => "GitHub",
            Screen::Contact => "Contact",
            Screen::Help => "Help",
        }
    }

    pub fn menu_index(self) -> Option<usize> {
        Self::MENU.iter().position(|screen| *screen == self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemAction {
    Goto(Screen),
    OpenProject(usize),
    OpenUrl(String),
    Back,
}

pub struct Doc {
    pub lines: Vec<Line<'static>>,
    pub items: Vec<(usize, usize)>,
}

impl Doc {
    pub fn indent(&mut self, cells: usize) {
        if cells == 0 {
            return;
        }
        for line in &mut self.lines {
            line.spans.insert(0, Span::raw(" ".repeat(cells)));
        }
    }

    pub fn pad_top(&mut self, rows: usize) {
        let blank = vec![Line::default(); rows];
        self.lines.splice(0..0, blank);
        for (start, end) in &mut self.items {
            *start += rows;
            *end += rows;
        }
    }

    pub fn max_width(&self) -> usize {
        self.lines.iter().map(Line::width).max().unwrap_or(0)
    }
}

pub fn actions(app: &App) -> Vec<ItemAction> {
    match app.screen {
        Screen::Home => Screen::MENU.iter().copied().map(ItemAction::Goto).collect(),
        Screen::Projects => (0..app.data.projects.len())
            .map(ItemAction::OpenProject)
            .collect(),
        Screen::ProjectDetail(index) => projects::detail_actions(app, index),
        Screen::Achievements => achievements::actions(app),
        Screen::GitHub => github::actions(app),
        Screen::Contact => contact::actions(app),
        Screen::About | Screen::Skills | Screen::Experience | Screen::Help => Vec::new(),
    }
}

pub fn build(app: &App, width: usize, height: usize) -> Doc {
    match app.screen {
        Screen::Home => home::build(app, width, height),
        Screen::About => about::build(app, width),
        Screen::Projects => projects::build_list(app, width),
        Screen::ProjectDetail(index) => projects::build_detail(app, width, index),
        Screen::Skills => skills::build(app, width),
        Screen::Experience => experience::build(app, width),
        Screen::Achievements => achievements::build(app, width),
        Screen::GitHub => github::build(app, width),
        Screen::Contact => contact::build(app, width),
        Screen::Help => help::build(app, width),
    }
}

pub struct Builder<'a> {
    pub theme: &'a Theme,
    pub width: usize,
    lines: Vec<Line<'static>>,
    items: Vec<(usize, usize)>,
}

const GUTTER: &str = "  ";

impl<'a> Builder<'a> {
    pub fn new(theme: &'a Theme, width: usize) -> Self {
        Self {
            theme,
            width: width.saturating_sub(GUTTER.len()).max(1),
            lines: Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn blank(&mut self) {
        self.lines.push(Line::default());
    }

    pub fn push(&mut self, spans: Vec<Span<'static>>) {
        let mut all = vec![Span::raw(GUTTER)];
        all.extend(spans);
        self.lines.push(Line::from(all));
    }

    pub fn push_marked(&mut self, selected: bool, spans: Vec<Span<'static>>) {
        let gutter = if selected {
            Span::styled(
                format!("{} ", self.theme.symbols().marker),
                self.theme.selected(),
            )
        } else {
            Span::raw(GUTTER)
        };
        let mut all = vec![gutter];
        all.extend(spans);
        self.lines.push(Line::from(all));
    }

    pub fn line(&mut self, text: impl Into<String>, style: Style) {
        let fitted = text::truncate(&text.into(), self.width, self.theme.symbols().ellipsis);
        self.push(vec![Span::styled(fitted, style)]);
    }

    pub fn columns(&mut self, key: &str, key_style: Style, description: &str, desc_style: Style) {
        let column = (text::width(key) + 3).min(self.width / 2);
        let label = self.fit_label(key, column);
        let room = self.width.saturating_sub(column).max(1);
        for (i, line) in text::wrap(description, room).into_iter().enumerate() {
            let lead = if i == 0 {
                label.clone()
            } else {
                " ".repeat(column)
            };
            self.push(vec![
                Span::styled(lead, key_style),
                Span::styled(line, desc_style),
            ]);
        }
    }

    pub fn fit_label(&self, label: &str, column: usize) -> String {
        text::pad_right(
            &text::truncate(
                label,
                column.saturating_sub(1),
                self.theme.symbols().ellipsis,
            ),
            column,
        )
    }

    pub fn fit(&self, text: &str, reserved: usize) -> String {
        text::truncate(
            text,
            self.width.saturating_sub(reserved),
            self.theme.symbols().ellipsis,
        )
    }

    pub fn text(&mut self, text: &str, style: Style) {
        self.indented_text(0, text, style);
    }

    pub fn indented_text(&mut self, indent: usize, text: &str, style: Style) {
        let pad = " ".repeat(indent);
        for line in text::wrap(text, self.width.saturating_sub(indent)) {
            self.push(vec![Span::raw(pad.clone()), Span::styled(line, style)]);
        }
    }

    pub fn heading(&mut self, label: &str) {
        self.line(label.to_uppercase(), self.theme.heading());
    }

    pub fn bullets(&mut self, entries: &[String]) {
        let bullet = self.theme.symbols().bullet;
        for entry in entries {
            for (i, line) in text::wrap(entry, self.width.saturating_sub(2))
                .into_iter()
                .enumerate()
            {
                let lead = if i == 0 {
                    Span::styled(format!("{bullet} "), self.theme.muted())
                } else {
                    Span::raw("  ")
                };
                self.push(vec![lead, Span::styled(line, self.theme.text())]);
            }
        }
    }

    pub fn rule(&mut self) {
        let rule = self.theme.symbols().rule.repeat(self.width);
        self.line(rule, self.theme.muted());
    }

    pub fn item(&mut self, selection: usize, f: impl FnOnce(&mut Self, bool)) {
        let selected = self.items.len() == selection;
        let start = self.lines.len();
        f(self, selected);
        self.items.push((start, self.lines.len()));
    }

    pub fn finish(self) -> Doc {
        Doc {
            lines: self.lines,
            items: self.items,
        }
    }

    pub fn empty_state(&mut self, what: &str, file: &str) {
        self.line(format!("No {what} configured yet."), self.theme.muted());
        self.blank();
        self.text(
            &format!("Add entries to {file} and they will appear here."),
            self.theme.muted(),
        );
    }
}
