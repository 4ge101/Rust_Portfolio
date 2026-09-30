use crate::screens::Screen;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Goto(Screen),
    Help,
    Quit,
}

pub const COMMANDS: &[(&str, &str)] = &[
    ("about", "Open About"),
    ("projects", "Open Projects"),
    ("skills", "Open Skills"),
    ("experience", "Open Experience"),
    ("achievements", "Open Achievements"),
    ("github", "Open GitHub"),
    ("contact", "Open Contact"),
    ("home", "Back to the main menu"),
    ("help", "Show keys and commands"),
    ("quit", "Exit"),
];

const NAMES: &[&str] = &[
    "about",
    "projects",
    "skills",
    "experience",
    "achievements",
    "github",
    "contact",
    "home",
    "help",
    "quit",
];

pub fn parse(input: &str) -> Result<Command, String> {
    let mut words = input.split_whitespace();
    let Some(word) = words.next() else {
        return Err("type a command, e.g. :projects (:help lists them)".to_owned());
    };
    let argument = words.next();
    if words.next().is_some() {
        return Err("too many arguments".to_owned());
    }

    let name = resolve(&word.to_ascii_lowercase())?;
    let no_argument = |command: Command| match argument {
        None => Ok(command),
        Some(_) => Err(format!(":{name} takes no argument")),
    };

    match name {
        "about" => no_argument(Command::Goto(Screen::About)),
        "projects" => no_argument(Command::Goto(Screen::Projects)),
        "skills" => no_argument(Command::Goto(Screen::Skills)),
        "experience" => no_argument(Command::Goto(Screen::Experience)),
        "achievements" => no_argument(Command::Goto(Screen::Achievements)),
        "github" => no_argument(Command::Goto(Screen::GitHub)),
        "contact" => no_argument(Command::Goto(Screen::Contact)),
        "home" => no_argument(Command::Goto(Screen::Home)),
        "help" => no_argument(Command::Help),
        "quit" => no_argument(Command::Quit),
        _ => unreachable!("resolve() only returns known names"),
    }
}

fn resolve(word: &str) -> Result<&'static str, String> {
    match word {
        "q" | "exit" => return Ok("quit"),
        "?" => return Ok("help"),
        _ => {}
    }
    if let Some(exact) = NAMES.iter().find(|name| **name == word) {
        return Ok(exact);
    }
    let matches: Vec<&&str> = NAMES.iter().filter(|name| name.starts_with(word)).collect();
    match matches.as_slice() {
        [only] => Ok(only),
        [] => Err(format!("unknown command: {word}")),
        many => Err(format!(
            "ambiguous: {}",
            many.iter()
                .map(|m| format!(":{m}"))
                .collect::<Vec<_>>()
                .join(" ")
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_listed_command() {
        for (usage, _) in COMMANDS {
            let word = usage.split_whitespace().next().unwrap();
            let input = word;
            assert!(parse(input).is_ok(), "{input}");
        }
    }

    #[test]
    fn accepts_unambiguous_prefixes_and_aliases() {
        assert_eq!(parse("proj"), Ok(Command::Goto(Screen::Projects)));
        assert_eq!(parse("ex"), Ok(Command::Goto(Screen::Experience)));
        assert_eq!(parse("q"), Ok(Command::Quit));
        assert_eq!(parse("?"), Ok(Command::Help));
        assert_eq!(parse("  ABOUT  "), Ok(Command::Goto(Screen::About)));
    }

    #[test]
    fn rejects_ambiguous_and_unknown() {
        assert!(parse("a").unwrap_err().starts_with("ambiguous"));
        assert!(parse("h").unwrap_err().starts_with("ambiguous"));
        assert!(parse("nope").unwrap_err().contains("unknown command"));
        assert!(parse("").is_err());
    }

    #[test]
    fn validates_arguments() {
        assert!(parse("about now").is_err());
        assert!(parse("help extra").is_err());
    }
}
