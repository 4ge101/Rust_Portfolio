mod app;
mod ascii;
mod cli;
mod commands;
mod data;
mod events;
mod opener;
mod screens;
mod text;
mod theme;
mod ui;

use std::io::{self, IsTerminal};
use std::process::ExitCode;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use ratatui::crossterm::event::{self, Event};

use app::{App, Effect};
use cli::Mode;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("alixsami: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = match cli::parse(std::env::args().skip(1)) {
        Ok(cli) => cli,
        Err(message) => {
            eprintln!("alixsami: {message}\nTry `alixsami --help`.");
            return Ok(ExitCode::from(2));
        }
    };

    match cli.mode {
        Mode::Help => {
            print!("{}", cli::HELP);
            return Ok(ExitCode::SUCCESS);
        }
        Mode::Version => {
            println!("alixsami {}", env!("CARGO_PKG_VERSION"));
            return Ok(ExitCode::SUCCESS);
        }
        Mode::Run | Mode::Check => {}
    }

    let data_dir = match &cli.data_dir {
        Some(dir) if !dir.is_dir() => bail!("--data: {} is not a directory", dir.display()),
        Some(dir) => Some(dir.clone()),
        None => data::default_config_dir(),
    };

    let loaded = data::load(data_dir.as_deref());
    let portrait = ascii::resolve(cli.image.as_deref(), data_dir.as_deref());

    if cli.mode == Mode::Check {
        return Ok(check(&loaded, &portrait, data_dir.as_deref()));
    }

    if !io::stdout().is_terminal() || !io::stdin().is_terminal() {
        bail!("this is an interactive program and needs a terminal (try `alixsami --check` in scripts)");
    }
    if std::env::var("TERM").is_ok_and(|term| term == "dumb") {
        bail!(
            "TERM=dumb is not supported; use a terminal emulator such as xterm, kitty or alacritty"
        );
    }

    let mut app = App::new(loaded, theme::Theme::detect(), portrait, data_dir);

    if cli.image.is_some() {
        if let Some(note) = app.diagnostics.portrait_note.clone() {
            app.set_status(app::StatusKind::Error, note);
        }
    }

    let mut terminal = ratatui::try_init().context("could not initialise the terminal")?;
    let result = event_loop(&mut terminal, &mut app);
    ratatui::restore();
    result?;
    Ok(ExitCode::SUCCESS)
}

fn event_loop(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> Result<()> {
    let (tx, rx) = mpsc::channel();
    let mut dirty = true;

    loop {
        if dirty {
            terminal.draw(|frame| ui::draw(frame, app))?;
            dirty = false;
        }
        if app.should_quit {
            return Ok(());
        }

        if event::poll(Duration::from_millis(200))? {
            loop {
                match event::read()? {
                    Event::Key(key) => {
                        if let Some(action) = events::map_key(key, app.in_command_mode()) {
                            if let Some(Effect::OpenUrl(url)) = app.update(action) {
                                opener::open(&url, tx.clone());
                            }
                        }
                    }
                    Event::Resize(..) => app.follow = true,
                    _ => {}
                }
                dirty = true;
                if app.should_quit || !event::poll(Duration::ZERO)? {
                    break;
                }
            }
        }

        while let Ok(result) = rx.try_recv() {
            app.on_open_result(result);
            dirty = true;
        }
        if app.expire_status(Instant::now()) {
            dirty = true;
        }
    }
}

fn check(
    loaded: &data::Loaded,
    portrait: &ascii::PortraitLoad,
    dir: Option<&std::path::Path>,
) -> ExitCode {
    let p = &loaded.portfolio;
    println!("alixsami {}", env!("CARGO_PKG_VERSION"));
    match dir {
        Some(dir) => println!("content   : bundled, overridden from {}", dir.display()),
        None => println!("content   : bundled"),
    }
    println!(
        "loaded    : {} projects, {} skill categories, {} experience entries, {} achievements",
        p.projects.len(),
        p.skills.len(),
        p.experience.len(),
        p.achievements.len()
    );
    match (&portrait.source, &portrait.note) {
        (Some(source), _) => println!("portrait  : {source}"),
        (None, note) => println!(
            "portrait  : none ({})",
            note.as_deref().unwrap_or("not found")
        ),
    }
    for warning in &loaded.warnings {
        println!("warning   : {warning}");
    }
    if loaded.warnings.is_empty() {
        println!("result    : ok");
        ExitCode::SUCCESS
    } else {
        println!("result    : {} warning(s)", loaded.warnings.len());
        ExitCode::FAILURE
    }
}