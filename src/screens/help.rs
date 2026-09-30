use super::{Builder, Doc};
use crate::app::App;
use crate::commands::COMMANDS;
use crate::text;

const KEYS: &[(&str, &str)] = &[
    ("Up Down  j k", "Move / scroll"),
    ("Enter  Right", "Select / open link"),
    ("Esc  Left  h", "Back"),
    ("PgUp PgDn", "Scroll by page"),
    ("Home End", "First / last"),
    ("1 - 7", "Jump to a section"),
    (":", "Command palette"),
    ("?", "This screen"),
    ("q  Ctrl-C", "Quit"),
];

pub fn build(app: &App, width: usize) -> Doc {
    let theme = &app.theme;
    let mut b = Builder::new(theme, width);

    b.heading("Keys");
    b.blank();
    let key_width = KEYS.iter().map(|(k, _)| text::width(k)).max().unwrap_or(0);
    for (keys, what) in KEYS {
        b.columns(
            &format!("{keys:<key_width$}"),
            theme.primary(),
            what,
            theme.text(),
        );
    }

    b.blank();
    b.heading("Commands  (press : first)");
    b.blank();
    let name_width = COMMANDS
        .iter()
        .map(|(n, _)| text::width(n))
        .max()
        .unwrap_or(0);
    for (name, what) in COMMANDS {
        b.columns(
            &format!("{name:<name_width$}"),
            theme.primary(),
            what,
            theme.text(),
        );
    }
    b.text("Commands can be abbreviated: :proj, :ex", theme.muted());

    b.blank();
    b.heading("This build");
    b.blank();
    b.text(
        &format!("portfolio {}", env!("CARGO_PKG_VERSION")),
        theme.text(),
    );
    let diagnostics = &app.diagnostics;
    let content = match &diagnostics.data_dir {
        Some(dir) => format!("Content: bundled, overridden from {}", dir.display()),
        None => "Content: bundled".to_owned(),
    };
    b.text(&content, theme.muted());
    match (&diagnostics.portrait_source, &diagnostics.portrait_note) {
        (Some(source), _) => b.text(&format!("Portrait: {source}"), theme.muted()),
        (None, Some(note)) => b.text(&format!("Portrait: {note}"), theme.muted()),
        (None, None) => {}
    }
    for warning in &diagnostics.warnings {
        b.text(&format!("Warning: {warning}"), theme.error());
    }
    b.finish()
}
