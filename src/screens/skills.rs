use ratatui::text::Span;

use super::{Builder, Doc};
use crate::app::App;
use crate::text;

pub fn build(app: &App, width: usize) -> Doc {
    let theme = &app.theme;
    let mut b = Builder::new(theme, width);

    if app.data.skills.is_empty() {
        b.empty_state("skills", "skills.toml");
        return b.finish();
    }

    for (index, category) in app.data.skills.iter().enumerate() {
        if index > 0 {
            b.blank();
        }
        b.heading(&category.name);
        b.blank();

        let column = category
            .skills
            .iter()
            .map(|skill| text::width(skill))
            .max()
            .unwrap_or(0)
            + 3;
        let column = column.min(b.width);
        let columns = (b.width / column.max(1)).clamp(1, 4);
        for row in category.skills.chunks(columns) {
            let mut spans = Vec::new();
            for (i, skill) in row.iter().enumerate() {
                let cell = if i + 1 < row.len() {
                    b.fit_label(skill, column)
                } else {
                    b.fit(skill, 0)
                };
                spans.push(Span::styled(cell, theme.text()));
            }
            b.push(spans);
        }
    }
    b.finish()
}
