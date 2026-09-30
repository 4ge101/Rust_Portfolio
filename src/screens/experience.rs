use ratatui::text::Span;

use super::{Builder, Doc};
use crate::app::App;

pub fn build(app: &App, width: usize) -> Doc {
    let theme = &app.theme;
    let symbols = theme.symbols();
    let mut b = Builder::new(theme, width);

    if app.data.experience.is_empty() {
        b.empty_state("experience", "experience.toml");
        return b.finish();
    }

    let mut groups: Vec<(&str, Vec<_>)> = Vec::new();
    for entry in &app.data.experience {
        match groups.last_mut() {
            Some((period, entries)) if *period == entry.period => entries.push(entry),
            _ => groups.push((&entry.period, vec![entry])),
        }
    }

    for (index, (period, entries)) in groups.iter().enumerate() {
        if index > 0 {
            b.blank();
        }
        b.line(period.to_string(), theme.heading());
        b.line(symbols.vert, theme.muted());

        for (i, entry) in entries.iter().enumerate() {
            let is_last = i + 1 == entries.len();
            let branch = if is_last {
                symbols.last
            } else {
                symbols.branch
            };
            b.push(vec![
                Span::styled(format!("{branch} "), theme.muted()),
                Span::styled(b.fit(&entry.title, 4), theme.text().bold()),
            ]);

            let trunk = if is_last {
                "    ".to_owned()
            } else {
                format!("{}   ", symbols.vert)
            };
            let detail: Vec<&str> = [entry.organization.as_deref(), entry.description.as_deref()]
                .into_iter()
                .flatten()
                .collect();
            let wrap_width = b.width.saturating_sub(4 + 1);
            for paragraph in detail {
                for line in crate::text::wrap(paragraph, wrap_width) {
                    b.push(vec![
                        Span::styled(format!("{trunk} "), theme.muted()),
                        Span::styled(line, theme.muted()),
                    ]);
                }
            }
        }
    }
    b.finish()
}
