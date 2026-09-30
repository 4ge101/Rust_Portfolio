use ratatui::text::Span;

use super::{Builder, Doc, ItemAction, Screen};
use crate::app::App;
use crate::text;

pub fn build_list(app: &App, width: usize) -> Doc {
    let theme = &app.theme;
    let symbols = theme.symbols();
    let mut b = Builder::new(theme, width);

    if app.data.projects.is_empty() {
        b.empty_state("projects", "projects.toml");
        return b.finish();
    }

    let last = app.data.projects.len() - 1;
    for (index, project) in app.data.projects.iter().enumerate() {
        b.item(app.sel, |b, selected| {
            let star_room = if project.featured { 2 } else { 0 };
            let mut head = vec![Span::styled(
                b.fit(&project.name, star_room),
                if selected {
                    theme.selected()
                } else {
                    theme.text().bold()
                },
            )];
            if project.featured {
                head.push(Span::styled(format!(" {}", symbols.star), theme.primary()));
            }
            b.push_marked(selected, head);

            let mut meta = project.technologies.join(&format!(" {} ", symbols.bullet));
            if let Some(status) = &project.status {
                if !meta.is_empty() {
                    meta.push_str(&format!("  {}  ", symbols.join));
                }
                meta.push_str(status);
            }
            if !meta.is_empty() {
                b.indented_text(2, &meta, theme.secondary());
            }

            if selected && !project.description.is_empty() {
                b.blank();
                let room = b.width.saturating_sub(2);
                let lines = text::wrap(&project.description, room);
                let shown = lines.len().min(3);
                for (i, line) in lines.iter().take(shown).enumerate() {
                    let mut line = line.clone();
                    if i + 1 == shown && lines.len() > shown {
                        line = text::truncate(&line, room.saturating_sub(1), "");
                        line.push_str(symbols.ellipsis);
                    }
                    b.push(vec![Span::raw("  "), Span::styled(line, theme.muted())]);
                }
            }
        });
        if index != last {
            b.rule();
        }
    }
    b.finish()
}

pub fn detail_actions(app: &App, index: usize) -> Vec<ItemAction> {
    let mut actions = Vec::new();
    if let Some(project) = app.data.projects.get(index) {
        actions.extend(project.github.clone().map(ItemAction::OpenUrl));
        actions.extend(project.website.clone().map(ItemAction::OpenUrl));
    }
    actions.push(ItemAction::Back);
    actions
}

pub fn build_detail(app: &App, width: usize, index: usize) -> Doc {
    let theme = &app.theme;
    let symbols = theme.symbols();
    let mut b = Builder::new(theme, width);

    let Some(project) = app.data.projects.get(index) else {
        b.line("Project not found.", theme.error());
        b.item(app.sel, |b, selected| {
            b.push_marked(
                selected,
                vec![Span::styled(
                    format!("{} Back", symbols.back),
                    theme.selected(),
                )],
            );
        });
        return b.finish();
    };

    b.text(&project.name, theme.title());
    let mut sub = Vec::new();
    if let Some(status) = &project.status {
        sub.push(status.clone());
    }
    if project.featured {
        sub.push(format!("{} featured", symbols.star));
    }
    if !sub.is_empty() {
        b.text(&sub.join(&format!("  {}  ", symbols.join)), theme.muted());
    }
    b.blank();

    if !project.description.is_empty() {
        b.text(&project.description, theme.text());
        b.blank();
    }

    if !project.technologies.is_empty() {
        b.heading("Technologies");
        b.text(
            &project.technologies.join(&format!("  {}  ", symbols.join)),
            theme.secondary(),
        );
        b.blank();
    }

    b.heading("Links");
    let links = [
        ("Repository", &project.github),
        ("Live demo", &project.website),
    ];
    let label_width = 12.min(b.width / 3);
    let mut any = false;
    for (label, url) in links {
        if let Some(url) = url {
            any = true;
            b.item(app.sel, |b, selected| {
                b.push_marked(
                    selected,
                    vec![
                        Span::styled(
                            b.fit_label(label, label_width),
                            if selected {
                                theme.selected()
                            } else {
                                theme.muted()
                            },
                        ),
                        Span::styled(
                            text::truncate(
                                url,
                                b.width.saturating_sub(label_width + 2),
                                symbols.ellipsis,
                            ),
                            theme.link(),
                        ),
                    ],
                );
            });
        }
    }
    if !any {
        b.line("No links configured.", theme.muted());
    }
    b.blank();

    b.item(app.sel, |b, selected| {
        b.push_marked(
            selected,
            vec![Span::styled(
                format!("{} Back", symbols.back),
                if selected {
                    theme.selected()
                } else {
                    theme.muted()
                },
            )],
        );
    });
    debug_assert_eq!(app.screen, Screen::ProjectDetail(index));
    b.finish()
}
