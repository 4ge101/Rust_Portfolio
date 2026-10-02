use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Run,
    Help,
    Version,
    Check,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Cli {
    pub mode: Mode,
    pub data_dir: Option<PathBuf>,
    pub image: Option<PathBuf>,
}

pub const HELP: &str = "\
portfolio - an interactive terminal portfolio

USAGE:
    alixsami [OPTIONS]

OPTIONS:
    -h, --help           Show this help
    -v, -V, --version    Show the version
        --data <DIR>     Load content from DIR instead of the bundled data.
                         Any of profile.toml, projects.toml, skills.toml,
                         experience.toml, achievements.toml may be present;
                         missing files fall back to the bundled copy.
                         Default: ~/.config/alixsami (if it exists)
        --image <FILE>   Portrait to convert to ASCII (JPEG or PNG)
        --check          Validate content and portrait, then exit
                         (exit status 1 if anything needs attention)

KEYS:
    Up/Down  Navigate     Enter  Select     Esc  Back     q  Quit
    1-7      Jump         :      Commands   ?    Help
";

pub fn parse<I: IntoIterator<Item = String>>(args: I) -> Result<Cli, String> {
    let mut cli = Cli {
        mode: Mode::Run,
        data_dir: None,
        image: None,
    };
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        let (flag, inline) = match arg.split_once('=') {
            Some((flag, value)) if flag.starts_with("--") => {
                (flag.to_owned(), Some(value.to_owned()))
            }
            _ => (arg.clone(), None),
        };
        let mut value = |name: &str| {
            inline
                .clone()
                .or_else(|| args.next())
                .filter(|v| !v.is_empty())
                .ok_or_else(|| format!("{name} requires a value"))
        };
        match flag.as_str() {
            "-h" | "--help" => cli.mode = Mode::Help,
            "-v" | "-V" | "--version" => cli.mode = Mode::Version,
            "--check" => cli.mode = Mode::Check,
            "--data" => cli.data_dir = Some(PathBuf::from(value("--data")?)),
            "--image" => cli.image = Some(PathBuf::from(value("--image")?)),
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }
    Ok(cli)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(args: &[&str]) -> Result<Cli, String> {
        parse(args.iter().map(|a| (*a).to_owned()))
    }

    #[test]
    fn no_arguments_runs_the_tui() {
        assert_eq!(parse_args(&[]).unwrap().mode, Mode::Run);
    }

    #[test]
    fn help_and_version_flags() {
        assert_eq!(parse_args(&["--help"]).unwrap().mode, Mode::Help);
        assert_eq!(parse_args(&["-h"]).unwrap().mode, Mode::Help);
        assert_eq!(parse_args(&["--version"]).unwrap().mode, Mode::Version);
        assert_eq!(parse_args(&["-v"]).unwrap().mode, Mode::Version);
        assert_eq!(parse_args(&["-V"]).unwrap().mode, Mode::Version);
    }

    #[test]
    fn options_with_values() {
        let cli = parse_args(&["--data", "/tmp/x", "--image=me.jpg", "--check"]).unwrap();
        assert_eq!(cli.data_dir, Some(PathBuf::from("/tmp/x")));
        assert_eq!(cli.image, Some(PathBuf::from("me.jpg")));
        assert_eq!(cli.mode, Mode::Check);
    }

    #[test]
    fn errors_are_reported() {
        assert!(parse_args(&["--nope"]).unwrap_err().contains("--nope"));
        assert!(parse_args(&["--data"])
            .unwrap_err()
            .contains("requires a value"));
    }
}
