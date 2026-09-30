use ratatui::text::Span;

use super::{Builder, Doc, Screen};
use crate::app::App;

pub fn build(app: &App, width: usize, height: usize) -> Doc {
    let theme = &app.theme;
    let profile = &app.data.profile;
    let mut b = Builder::new(theme, width);

    let compact = height < 14;

    b.line(profile.name.clone(), theme.title());
    b.text(&profile.title, theme.text());
    if let (Some(tagline), false) = (&profile.tagline, compact) {
        b.text(tagline, theme.muted());
    }
    b.blank();

    for (index, screen) in Screen::MENU.iter().enumerate() {
        b.item(app.sel, |b, selected| {
            let label_style = if selected {
                theme.selected()
            } else {
                theme.text()
            };
            b.push_marked(
                selected,
                vec![
                    Span::styled(format!("{}  ", index + 1), theme.muted()),
                    Span::styled(screen.title().to_owned(), label_style),
                ],
            );
        });
    }

    let mut doc = b.finish();
    let pad = width.saturating_sub(doc.max_width()) / 2;
    doc.indent(pad);
    if doc.lines.len() < height {
        doc.pad_top((height - doc.lines.len()) / 2);
    }
    doc
}
