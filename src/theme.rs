use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::border;

#[derive(Debug, Clone, Copy)]
pub struct Symbols {
    pub marker: &'static str,
    pub bullet: &'static str,
    pub dot: &'static str,
    pub star: &'static str,
    pub ellipsis: &'static str,
    pub back: &'static str,
    pub arrows: &'static str,
    pub crumb: &'static str,
    pub join: &'static str,
    pub branch: &'static str,
    pub last: &'static str,
    pub vert: &'static str,
    pub rule: &'static str,
    pub sep_left: &'static str,
    pub sep_right: &'static str,
    pub bar_track: &'static str,
    pub bar_thumb: &'static str,
}

const UNICODE: Symbols = Symbols {
    marker: "❯",
    bullet: "•",
    dot: "●",
    star: "★",
    ellipsis: "…",
    back: "←",
    arrows: "↑↓",
    crumb: "›",
    join: "·",
    branch: "├──",
    last: "└──",
    vert: "│",
    rule: "─",
    sep_left: "├",
    sep_right: "┤",
    bar_track: "│",
    bar_thumb: "█",
};

const ASCII: Symbols = Symbols {
    marker: ">",
    bullet: "*",
    dot: "*",
    star: "*",
    ellipsis: "~",
    back: "<-",
    arrows: "Up/Down",
    crumb: ">",
    join: "-",
    branch: "|--",
    last: "`--",
    vert: "|",
    rule: "-",
    sep_left: "+",
    sep_right: "+",
    bar_track: "|",
    bar_thumb: "#",
};

const ASCII_BORDER: border::Set<'static> = border::Set {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColorDepth {
    True,
    Indexed,
    Basic,
    None,
}

#[derive(Debug, Clone)]
pub struct Theme {
    primary: Color,
    secondary: Color,
    muted: Color,
    error: Color,
    depth: ColorDepth,
    unicode: bool,
}

impl Theme {
    pub fn detect() -> Self {
        let var = |name: &str| std::env::var(name).ok();
        Self::from_env(
            var("COLORTERM").as_deref(),
            var("TERM").as_deref(),
            var("NO_COLOR").as_deref(),
            [var("LC_ALL"), var("LC_CTYPE"), var("LANG")]
                .into_iter()
                .flatten()
                .find(|value| !value.is_empty())
                .as_deref(),
        )
    }

    pub fn from_env(
        colorterm: Option<&str>,
        term: Option<&str>,
        no_color: Option<&str>,
        locale: Option<&str>,
    ) -> Self {
        let depth = if no_color.is_some_and(|value| !value.is_empty()) {
            ColorDepth::None
        } else if colorterm.is_some_and(|c| c.contains("truecolor") || c.contains("24bit")) {
            ColorDepth::True
        } else if term.is_some_and(|t| t.contains("256color")) {
            ColorDepth::Indexed
        } else {
            ColorDepth::Basic
        };

        let unicode = locale.is_none_or(|l| {
            let l = l.to_ascii_lowercase();
            l.contains("utf-8") || l.contains("utf8")
        });

        let (primary, secondary, muted, error) = match depth {
            ColorDepth::True => (
                Color::Rgb(232, 132, 76),
                Color::Rgb(122, 162, 190),
                Color::Rgb(122, 130, 142),
                Color::Rgb(224, 108, 117),
            ),
            ColorDepth::Indexed => (
                Color::Indexed(173),
                Color::Indexed(110),
                Color::Indexed(244),
                Color::Indexed(167),
            ),
            ColorDepth::Basic => (Color::Yellow, Color::Cyan, Color::DarkGray, Color::Red),
            ColorDepth::None => (Color::Reset, Color::Reset, Color::Reset, Color::Reset),
        };

        Self {
            primary,
            secondary,
            muted,
            error,
            depth,
            unicode,
        }
    }

    pub fn symbols(&self) -> &'static Symbols {
        if self.unicode {
            &UNICODE
        } else {
            &ASCII
        }
    }

    pub fn border_set(&self) -> border::Set<'static> {
        if self.unicode {
            border::ROUNDED
        } else {
            ASCII_BORDER
        }
    }

    fn tint(&self, color: Color) -> Style {
        Style::new().fg(color)
    }

    pub fn text(&self) -> Style {
        Style::new()
    }

    pub fn muted(&self) -> Style {
        if self.depth == ColorDepth::None {
            Style::new().add_modifier(Modifier::DIM)
        } else {
            self.tint(self.muted)
        }
    }

    pub fn primary(&self) -> Style {
        self.tint(self.primary)
    }

    pub fn secondary(&self) -> Style {
        self.tint(self.secondary)
    }

    pub fn error(&self) -> Style {
        self.tint(self.error).add_modifier(Modifier::BOLD)
    }

    pub fn title(&self) -> Style {
        self.primary().add_modifier(Modifier::BOLD)
    }

    pub fn heading(&self) -> Style {
        self.secondary().add_modifier(Modifier::BOLD)
    }

    pub fn selected(&self) -> Style {
        self.primary().add_modifier(Modifier::BOLD)
    }

    pub fn border(&self) -> Style {
        self.muted()
    }

    pub fn link(&self) -> Style {
        self.secondary().add_modifier(Modifier::UNDERLINED)
    }

    pub fn portrait(&self) -> Style {
        self.text()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme(colorterm: Option<&str>, term: Option<&str>, no_color: Option<&str>) -> Theme {
        Theme::from_env(colorterm, term, no_color, Some("en_US.UTF-8"))
    }

    #[test]
    fn detects_colour_depth() {
        assert_eq!(theme(Some("truecolor"), None, None).depth, ColorDepth::True);
        assert_eq!(theme(Some("24bit"), None, None).depth, ColorDepth::True);
        assert_eq!(
            theme(None, Some("xterm-256color"), None).depth,
            ColorDepth::Indexed
        );
        assert_eq!(theme(None, Some("xterm"), None).depth, ColorDepth::Basic);
        assert_eq!(theme(None, None, None).depth, ColorDepth::Basic);
    }

    #[test]
    fn no_color_wins_over_everything() {
        let t = theme(Some("truecolor"), Some("xterm-256color"), Some("1"));
        assert_eq!(t.depth, ColorDepth::None);
        assert_eq!(t.primary().fg, Some(Color::Reset));

        assert_ne!(
            theme(Some("truecolor"), None, Some("")).depth,
            ColorDepth::None
        );
    }

    #[test]
    fn falls_back_to_ascii_without_utf8() {
        let t = Theme::from_env(None, None, None, Some("C"));
        assert_eq!(t.symbols().marker, ">");
        let t = Theme::from_env(None, None, None, Some("en_GB.utf8"));
        assert_eq!(t.symbols().marker, "❯");
        let t = Theme::from_env(None, None, None, None);
        assert_eq!(t.symbols().marker, "❯");
    }
}
