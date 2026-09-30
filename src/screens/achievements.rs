use ratatui::text::Span;

use super::{Builder, Doc, ItemAction};
use crate::app::App;
use crate::data::Achievement;
use crate::text;

fn grouped(all: &[Achievement]) -> Vec<(&str, Vec<&Achievement>)> {
    let mut groups: Vec<(&str, Vec<&Achievement>)> = Vec::new();
    for achievement in all {
        let category = achievement.category.as_deref().unwrap_or("Other");
        match groups.iter_mut().find(|(name, _)| *name == category) {
            Some((_, entries)) => entries.push(achievement),
            None => groups.push((category, vec![achievement])),
        }
    }
    groups
}

pub fn actions(app: &App) -> Vec<ItemAction> {
    grouped(&app.data.achievements)
        .into_iter()
        .flat_map(|(_, entries)| entries)
        .filter_map(|achievement| achievement.url.clone().map(ItemAction::OpenUrl))
        .collect()
}

pub fn build(app: &App, width: usize) -> Doc {
    let theme = &app.theme;
    let mut b = Builder::new(theme, width);

    if app.data.achievements.is_empty() {
        b.empty_state("achievements", "achievements.toml");
        return b.finish();
    }

    for (index, (category, entries)) in grouped(&app.data.achievements).iter().enumerate() {
        if index > 0 {
            b.blank();
        }
        b.heading(category);
        b.blank();

        for achievement in entries {
            let body = |b: &mut Builder, selected: Option<bool>| {
                let date_room = achievement.date.as_ref().map_or(0, |d| text::width(d) + 4);
                let mut head = vec![Span::styled(
                    b.fit(&achievement.title, date_room),
                    if selected == Some(true) {
                        theme.selected()
                    } else {
                        theme.text().bold()
                    },
                )];
                if let Some(date) = &achievement.date {
                    head.push(Span::styled(
                        format!("  {} {}", theme.symbols().join, b.fit(date, 4)),
                        theme.muted(),
                    ));
                }
                match selected {
                    Some(selected) => b.push_marked(selected, head),
                    None => b.push(head),
                }
                if let Some(description) = &achievement.description {
                    b.text(description, theme.muted());
                }
                if let Some(url) = &achievement.url {
                    b.line(
                        text::truncate(url, b.width, theme.symbols().ellipsis),
                        theme.link(),
                    );
                }
            };

            if achievement.url.is_some() {
                b.item(app.sel, |b, selected| body(b, Some(selected)));
            } else {
                body(&mut b, None);
            }
        }
    }
    b.finish()
}
