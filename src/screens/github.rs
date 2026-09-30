use ratatui::text::Span;

use super::{Builder, Doc, ItemAction};
use crate::app::App;
use crate::text;

pub fn actions(app: &App) -> Vec<ItemAction> {
    let Some(github) = &app.data.profile.github else {
        return Vec::new();
    };
    let mut actions = vec![ItemAction::OpenUrl(github.profile_link())];
    actions.extend(
        github
            .repos
            .iter()
            .map(|repo| ItemAction::OpenUrl(repo.link(github))),
    );
    actions
}

pub fn build(app: &App, width: usize) -> Doc {
    let theme = &app.theme;
    let ellipsis = theme.symbols().ellipsis;
    let mut b = Builder::new(theme, width);

    let Some(github) = &app.data.profile.github else {
        b.empty_state("GitHub account", "the [github] table in profile.toml");
        return b.finish();
    };

    b.line(format!("@{}", github.username), theme.title());
    b.blank();

    let profile_url = github.profile_link();
    b.item(app.sel, |b, selected| {
        b.push_marked(
            selected,
            vec![
                Span::styled(
                    b.fit_label("Profile", 10.min(b.width / 3)),
                    if selected {
                        theme.selected()
                    } else {
                        theme.muted()
                    },
                ),
                Span::styled(
                    text::truncate(
                        &profile_url,
                        b.width.saturating_sub(10.min(b.width / 3)),
                        ellipsis,
                    ),
                    theme.link(),
                ),
            ],
        );
    });

    if !github.repos.is_empty() {
        b.blank();
        b.heading("Selected repositories");
        b.blank();
        for repo in &github.repos {
            let url = repo.link(github);
            b.item(app.sel, |b, selected| {
                b.push_marked(
                    selected,
                    vec![Span::styled(
                        b.fit(&repo.name, 0),
                        if selected {
                            theme.selected()
                        } else {
                            theme.text().bold()
                        },
                    )],
                );
                if let Some(description) = &repo.description {
                    b.text(description, theme.muted());
                }
                b.line(text::truncate(&url, b.width, ellipsis), theme.link());
            });
            b.blank();
        }
    }
    b.finish()
}
