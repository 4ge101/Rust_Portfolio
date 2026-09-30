use ratatui::text::Span;

use super::{Builder, Doc, ItemAction};
use crate::app::App;
use crate::text;

pub fn actions(app: &App) -> Vec<ItemAction> {
    app.data
        .profile
        .contact
        .iter()
        .filter_map(|link| link.target().map(ItemAction::OpenUrl))
        .collect()
}

pub fn build(app: &App, width: usize) -> Doc {
    let theme = &app.theme;
    let mut b = Builder::new(theme, width);
    let links = &app.data.profile.contact;

    if links.is_empty() {
        b.empty_state("contact methods", "the [[contact]] entries in profile.toml");
        return b.finish();
    }

    let label_width = links
        .iter()
        .map(|link| text::width(&link.label))
        .max()
        .unwrap_or(0)
        .saturating_add(3)
        .min(b.width / 2);
    let value_width = b.width.saturating_sub(label_width);

    for link in links {
        let value = text::truncate(&link.value, value_width, theme.symbols().ellipsis);
        if link.target().is_some() {
            b.item(app.sel, |b, selected| {
                b.push_marked(
                    selected,
                    vec![
                        Span::styled(
                            b.fit_label(&link.label, label_width),
                            if selected {
                                theme.selected()
                            } else {
                                theme.heading()
                            },
                        ),
                        Span::styled(value.clone(), theme.link()),
                    ],
                );
            });
        } else {
            b.push(vec![
                Span::styled(b.fit_label(&link.label, label_width), theme.heading()),
                Span::styled(value, theme.text()),
            ]);
        }
    }
    b.finish()
}
