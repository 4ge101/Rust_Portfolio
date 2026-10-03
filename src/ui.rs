use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::Frame;

use crate::app::{App, Input, StatusKind};
use crate::ascii::Options;
use crate::screens::{self, Doc, Screen};
use crate::text;
use crate::theme::Theme;

pub const MIN_WIDTH: u16 = 40;
pub const MIN_HEIGHT: u16 = 12;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        draw_too_small(frame, area, &app.theme);
        return;
    }
    let theme = app.theme.clone();

    let block = Block::bordered()
        .border_set(theme.border_set())
        .border_style(theme.border())
        .title_top(title_line(app, area.width))
        .title_top(status_line(app, area.width).right_aligned());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [content, separator, footer] = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    let symbols = theme.symbols();
    let rule = symbols
        .rule
        .repeat(usize::from(area.width).saturating_sub(2));
    frame.render_widget(
        Paragraph::new(format!("{}{rule}{}", symbols.sep_left, symbols.sep_right))
            .style(theme.border()),
        Rect {
            x: area.x,
            y: separator.y,
            width: area.width,
            height: 1,
        },
    );

    let margin = if area.width >= 64 { 2 } else { 1 };
    let top_pad = u16::from(content.height >= 16);
    let body = Rect {
        x: content.x + margin,
        y: content.y + top_pad,
        width: content.width.saturating_sub(margin * 2),
        height: content.height.saturating_sub(top_pad),
    };

    if app.screen == Screen::Home {
        draw_home(frame, app, body);
    } else {
        let doc = screens::build(
            app,
            usize::from(body.width.saturating_sub(1)),
            usize::from(body.height),
        );
        draw_doc(frame, app, body, &doc);
    }

    draw_footer(frame, app, footer, area.width);
}

fn draw_too_small(frame: &mut Frame, area: Rect, theme: &Theme) {
    let text = vec![
        Line::styled("Terminal window too small.", theme.error()),
        Line::default(),
        Line::styled("Please resize your terminal.", theme.text()),
        Line::default(),
        Line::styled(
            format!(
                "{}x{} now, need at least {}x{}",
                area.width, area.height, MIN_WIDTH, MIN_HEIGHT
            ),
            theme.muted(),
        ),
    ];
    frame.render_widget(
        Paragraph::new(text).wrap(ratatui::widgets::Wrap { trim: true }),
        area,
    );
}

fn title_line(app: &App, width: u16) -> Line<'static> {
    let theme = &app.theme;
    let crumb = theme.symbols().crumb;
    let budget = usize::from(width).saturating_sub(8);

    let name = text::truncate(
        &app.data.profile.name.to_uppercase(),
        budget.min(28),
        theme.symbols().ellipsis,
    );
    let mut spans = vec![Span::raw(" "), Span::styled(name, theme.title())];

    let mut trail = Vec::new();
    match app.screen {
        Screen::Home => {}
        Screen::ProjectDetail(index) => {
            trail.push("Projects".to_owned());
            if let Some(project) = app.data.projects.get(index) {
                trail.push(project.name.clone());
            }
        }
        other => trail.push(other.title().to_owned()),
    }
    let used = text::width(&spans[1].content);
    let mut remaining = budget.saturating_sub(used);
    for part in trail {
        let label = format!(" {crumb} {part}");
        if text::width(&label) > remaining {
            break;
        }
        remaining -= text::width(&label);
        spans.push(Span::styled(label, theme.muted()));
    }
    spans.push(Span::raw(" "));
    Line::from(spans)
}

fn status_line(app: &App, width: u16) -> Line<'static> {
    let Some(status) = app
        .data
        .profile
        .status
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    else {
        return Line::default();
    };

    if usize::from(width) < 60 {
        return Line::default();
    }
    let theme = &app.theme;
    Line::from(vec![
        Span::raw(" "),
        Span::styled(theme.symbols().dot, theme.secondary()),
        Span::styled(
            format!(" {} ", text::truncate(status, 24, theme.symbols().ellipsis)),
            theme.text(),
        ),
    ])
}

fn draw_home(frame: &mut Frame, app: &mut App, area: Rect) {
    let wide = app.portrait.is_some() && area.width >= 84 && area.height >= 18;
    let (portrait_area, menu_area) = if wide {
        let wanted = app.data.profile.portrait.width.saturating_add(2);
        let left = wanted.min(area.width.saturating_sub(40));
        (
            Some(Rect {
                width: left,
                ..area
            }),
            Rect {
                x: area.x + left,
                width: area.width - left,
                ..area
            },
        )
    } else {
        (None, area)
    };

    if let Some(rect) = portrait_area {
        draw_portrait(frame, app, rect);
    }
    let doc = screens::build(
        app,
        usize::from(menu_area.width.saturating_sub(1)),
        usize::from(menu_area.height),
    );
    draw_doc(frame, app, menu_area, &doc);
}

fn draw_portrait(frame: &mut Frame, app: &mut App, area: Rect) {
    let style = app.theme.portrait();
    let options = Options {
        cols: area
            .width
            .saturating_sub(2)
            .min(app.data.profile.portrait.width),
        rows: area.height,
        invert: app.invert,
    };
    let Some(portrait) = app.portrait.as_mut() else {
        return;
    };
    let lines: Vec<Line> = portrait
        .lines(options)
        .iter()
        .map(|line| Line::styled(line.clone(), style))
        .collect();
    let height = u16::try_from(lines.len())
        .unwrap_or(area.height)
        .min(area.height);
    let rect = Rect {
        y: area.y + (area.height - height) / 2,
        height,
        ..area
    };
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), rect);
}

fn draw_doc(frame: &mut Frame, app: &mut App, area: Rect, doc: &Doc) {
    let height = usize::from(area.height);
    let total = doc.lines.len();
    let max_scroll = total.saturating_sub(height);
    app.viewport_rows = height;

    if app.follow {
        if let Some(&(start, end)) = doc.items.get(app.sel) {
            if app.sel == 0 {
                app.scroll = 0;
            }
            if end - start >= height || start < app.scroll {
                app.scroll = start;
            } else if end > app.scroll + height {
                app.scroll = end - height;
            }
        }
        app.follow = false;
    }
    app.scroll = app.scroll.min(max_scroll);

    let visible: Vec<Line> = doc
        .lines
        .iter()
        .skip(app.scroll)
        .take(height)
        .cloned()
        .collect();
    frame.render_widget(
        Paragraph::new(visible),
        Rect {
            width: area.width.saturating_sub(1),
            ..area
        },
    );

    if max_scroll > 0 {
        let symbols = app.theme.symbols();
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some(symbols.bar_track))
            .thumb_symbol(symbols.bar_thumb)
            .track_style(app.theme.border())
            .thumb_style(app.theme.muted());
        let mut state = ScrollbarState::new(max_scroll)
            .position(app.scroll)
            .viewport_content_length(height);
        frame.render_stateful_widget(scrollbar, area, &mut state);
    }
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect, frame_width: u16) {
    let theme = &app.theme;
    let area = Rect {
        x: area.x + 1,
        width: area.width.saturating_sub(2),
        ..area
    };

    if let Input::Command(buffer) = &app.input {
        let line = Line::from(vec![
            Span::styled(":", theme.primary()),
            Span::styled(buffer.clone(), theme.text()),
        ]);
        frame.render_widget(Paragraph::new(line), area);
        let x = area.x + 1 + u16::try_from(text::width(buffer)).unwrap_or(0);
        frame.set_cursor_position((x.min(area.right().saturating_sub(1)), area.y));
        return;
    }

    let hints = hints(app);
    let mut spans = Vec::new();
    let mut used = 0;
    for (key, label) in &hints {
        let piece = text::width(key) + 1 + text::width(label) + 3;
        if used + piece > usize::from(area.width) {
            break;
        }
        spans.push(Span::styled(key.clone(), theme.primary()));
        spans.push(Span::styled(format!(" {label}   "), theme.muted()));
        used += piece;
    }

    if let Some(status) = &app.status {
        let style = match status.kind {
            StatusKind::Info => theme.secondary(),
            StatusKind::Error => theme.error(),
        };
        let room = usize::from(area.width).saturating_sub(used);
        let wide_enough = text::width(&status.text) + 2 <= room;
        if wide_enough {
            frame.render_widget(Paragraph::new(Line::from(spans)), area);
            frame.render_widget(
                Paragraph::new(Line::styled(status.text.clone(), style))
                    .alignment(Alignment::Right),
                area,
            );
        } else {
            let shown = text::truncate(
                &status.text,
                usize::from(area.width),
                theme.symbols().ellipsis,
            );
            frame.render_widget(Paragraph::new(Line::styled(shown, style)), area);
        }
        return;
    }
    let _ = frame_width;
    frame.render_widget(Paragraph::new(Line::from(spans)).style(Style::new()), area);
}

fn hints(app: &App) -> Vec<(String, &'static str)> {
    let arrows = app.theme.symbols().arrows.to_owned();
    let has_items = !screens::actions(app).is_empty();
    let mut hints: Vec<(String, &'static str)> = Vec::new();

    match app.screen {
        Screen::Home => {
            hints.push((arrows, "Navigate"));
            hints.push(("Enter".into(), "Select"));
            hints.push(("q".into(), "Quit"));
            hints.push(("1-7".into(), "Jump"));
            hints.push((":".into(), "Command"));
            hints.push(("?".into(), "Help"));
        }
        screen => {
            hints.push((arrows, if has_items { "Navigate" } else { "Scroll" }));
            if has_items {
                let label = match screen {
                    Screen::Projects => "Open",
                    _ => "Open link",
                };
                hints.push(("Enter".into(), label));
            }
            hints.push(("Esc".into(), "Back"));
            hints.push(("q".into(), "Quit"));
            hints.push((":".into(), "Command"));
            hints.push(("?".into(), "Help"));
        }
    }
    hints
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::ascii::{Portrait, PortraitLoad};
    use crate::data;
    use crate::events::Action;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn sample_app(with_portrait: bool) -> App {
        let mut loaded = data::load(None);
        let p = &mut loaded.portfolio;
        p.projects[0].github = Some("https://github.com/x/robot".into());
        p.projects[0].website = Some("https://example.com".into());
        p.projects[0].description = "A long description. ".repeat(30);
        p.profile.status = Some("Available".into());
        p.profile.location = Some("Somewhere".into());
        p.profile.philosophy = Some("Build small, sharp tools.".into());
        p.profile.github = Some(data::Github {
            username: "octocat".into(),
            profile_url: None,
            repos: vec![data::Repo {
                name: "hello".into(),
                description: Some("d".into()),
                url: None,
            }],
        });
        p.profile.contact = vec![
            data::ContactLink {
                label: "Email".into(),
                value: "me@example.com".into(),
                url: None,
            },
            data::ContactLink {
                label: "Location".into(),
                value: "plain text".into(),
                url: None,
            },
        ];
        p.achievements = vec![
            data::Achievement {
                title: "Won a hackathon".into(),
                category: Some("Hackathon".into()),
                date: Some("2026".into()),
                description: Some("Built a thing.".into()),
                url: Some("https://example.com/win".into()),
            },
            data::Achievement {
                title: "Certificate".into(),
                category: None,
                date: None,
                description: None,
                url: None,
            },
        ];
        let portrait = if with_portrait {
            let gray = image::GrayImage::from_fn(200, 260, |x, y| {
                let (dx, dy) = (x as f32 - 100.0, y as f32 - 130.0);
                image::Luma([(255.0 - (dx * dx + dy * dy).sqrt() * 1.6).clamp(0.0, 255.0) as u8])
            });
            let mut file = std::env::temp_dir();
            file.push(format!("portfolio-ui-test-{}.png", std::process::id()));
            gray.save(&file).unwrap();
            let loaded_portrait = Portrait::from_path(&file).unwrap();
            std::fs::remove_file(&file).unwrap();
            PortraitLoad {
                portrait: Some(loaded_portrait),
                note: None,
                source: Some("test".into()),
            }
        } else {
            PortraitLoad {
                portrait: None,
                note: None,
                source: None,
            }
        };
        App::new(
            loaded,
            Theme::from_env(Some("truecolor"), None, None, None),
            portrait,
            None,
        )
    }

    fn render(app: &mut App, w: u16, h: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        terminal.draw(|f| draw(f, app)).unwrap();
        let buffer = terminal.backend().buffer().clone();
        (0..h)
            .map(|y| (0..w).map(|x| buffer[(x, y)].symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn every_screen() -> Vec<Screen> {
        let mut screens = vec![
            Screen::Home,
            Screen::Projects,
            Screen::ProjectDetail(0),
            Screen::Help,
        ];
        screens.extend(Screen::MENU);
        screens
    }

    #[test]
    fn item_ranges_match_actions_on_every_screen() {
        let mut app = sample_app(false);
        for screen in every_screen() {
            app.screen = screen;
            let doc = screens::build(&app, 60, 20);
            assert_eq!(doc.items.len(), screens::actions(&app).len(), "{screen:?}");
        }
    }

    #[test]
    fn no_line_is_wider_than_its_viewport() {
        let mut app = sample_app(false);

        for width in [20usize, 35, 60, 120] {
            for screen in every_screen() {
                app.screen = screen;
                let doc = screens::build(&app, width, 20);
                for line in &doc.lines {
                    assert!(line.width() <= width, "{screen:?} @ {width}: {line:?}");
                }
            }
        }
    }

    #[test]
    fn every_screen_renders_at_every_size_without_panicking() {
        for with_portrait in [false, true] {
            let mut app = sample_app(with_portrait);
            for screen in every_screen() {
                for (w, h) in [
                    (1, 1),
                    (39, 30),
                    (40, 11),
                    (40, 12),
                    (52, 14),
                    (80, 24),
                    (84, 20),
                    (120, 40),
                    (200, 60),
                ] {
                    app.screen = screen;
                    for step in 0..6 {
                        render(&mut app, w, h);
                        app.update(if step % 2 == 0 {
                            Action::Down
                        } else {
                            Action::PageDown
                        });
                    }
                }
            }
        }
    }

    #[test]
    fn too_small_terminal_shows_the_resize_message() {
        let mut app = sample_app(false);
        for (w, h) in [(39, 24), (80, 11)] {
            let screen = render(&mut app, w, h);
            assert!(screen.contains("Terminal window too small."), "{w}x{h}");
            assert!(screen.contains("Please resize your terminal."), "{w}x{h}");
        }

        assert!(render(&mut app, 20, 6).contains("Terminal window"));
        assert!(!render(&mut app, 40, 12).contains("too small"));
    }

    #[test]
    fn home_lists_every_menu_entry_and_the_marker() {
        let mut app = sample_app(false);
        let screen = render(&mut app, 80, 24);
        for entry in [
            "About",
            "Projects",
            "Skills",
            "Experience",
            "Achievements",
            "GitHub",
            "Contact",
        ] {
            assert!(screen.contains(entry), "missing {entry}");
        }
        assert!(screen.contains("❯"));
        assert!(screen.contains("● Available"));
    }

    #[test]
    fn wide_home_shows_the_portrait() {
        let mut app = sample_app(true);
        let wide = render(&mut app, 120, 34);
        assert!(
            wide.contains('@') || wide.contains('#'),
            "portrait glyphs expected"
        );

        let narrow = render(&mut app, 60, 24);
        assert!(!narrow.contains('@'));
        assert!(narrow.contains("Contact"));
    }

    #[test]
    fn small_home_keeps_the_selected_row_visible() {
        let mut app = sample_app(false);
        for _ in 0..6 {
            app.update(Action::Down);
        }
        let screen = render(&mut app, 40, 12);
        assert!(screen.contains("Contact"), "{screen}");
    }

    #[test]
    fn long_documents_scroll_to_the_end() {
        let mut app = sample_app(false);
        app.update(Action::Goto(Screen::Help));
        app.update(Action::Last);
        let screen = render(&mut app, 60, 14);
        assert!(
            screen.contains("Warning")
                || screen.contains("alixsami 0.1.0")
                || screen.contains("Content"),
            "{screen}"
        );
    }

    #[test]
    fn command_prompt_is_visible() {
        let mut app = sample_app(false);
        app.update(Action::OpenCommand);
        app.update(Action::Type('p'));
        assert!(render(&mut app, 80, 24).contains(":p"));
    }

    #[test]
    fn ascii_fallback_has_no_box_drawing() {
        let mut app = sample_app(false);
        app.theme = Theme::from_env(None, None, Some("1"), Some("C"));
        app.data.profile.title = "Rust developer".into();
        let screen = render(&mut app, 80, 24);
        assert!(screen.is_ascii(), "{screen}");
    }
}