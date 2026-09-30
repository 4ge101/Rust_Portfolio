use super::{Builder, Doc};
use crate::app::App;

pub fn build(app: &App, width: usize) -> Doc {
    let theme = &app.theme;
    let profile = &app.data.profile;
    let mut b = Builder::new(theme, width);

    b.line(profile.name.clone(), theme.title());
    b.text(&profile.title, theme.muted());
    if let Some(location) = &profile.location {
        b.text(location, theme.muted());
    }
    b.blank();

    for paragraph in &profile.bio {
        b.text(paragraph, theme.text());
        b.blank();
    }

    if !profile.focus.is_empty() {
        b.heading("Focus");
        b.bullets(&profile.focus);
        b.blank();
    }
    if !profile.interests.is_empty() {
        b.heading("Currently exploring");
        b.bullets(&profile.interests);
        b.blank();
    }
    if let Some(philosophy) = &profile.philosophy {
        b.heading("Philosophy");
        b.text(philosophy, theme.text());
        b.blank();
    }

    if profile.bio.is_empty() && profile.focus.is_empty() && profile.interests.is_empty() {
        b.text(
            "Add a bio, focus and interests to profile.toml and they will appear here.",
            theme.muted(),
        );
    }
    b.finish()
}
