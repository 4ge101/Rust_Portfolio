# portfolio

An interactive terminal portfolio. A native Rust TUI (text UI) that runs
directly in your terminal — no browser, no server.

## Install

No Rust, no cloning — download a ready-to-run binary and use it straight
away.

**Linux / macOS**

```bash
curl -fsSL https://raw.githubusercontent.com/4ge101/Rust_Portfolio/main/install.sh | sh
```

**Windows (PowerShell)**

```powershell
irm https://raw.githubusercontent.com/4ge101/Rust_Portfolio/main/install.ps1 | iex
```

Both scripts detect your OS/CPU, download the matching binary from the
[Releases page](https://github.com/4ge101/Rust_Portfolio/releases), and put it
on your PATH. Prefer to do it by hand? Grab the archive for your platform
from Releases, unzip it, and run the binary directly — it works from any
folder, no installation step needed.

Already have Rust and want to build it yourself instead:

```bash
cargo install --git https://github.com/4ge101/Rust_Portfolio
```

## Run

```bash
alixsami                 # launch the TUI
alixsami --help          # usage and options
alixsami --version       # print the version
```

### Keys

`↑`/`↓` or `j`/`k` to move, `Enter` to select, `Esc` to go back, `1`-`7` to
jump to a section, `:` for a command palette, `?` for help, `q` to quit.

## Tech stack

- [Rust](https://www.rust-lang.org/)
- [ratatui](https://ratatui.rs/) + [crossterm](https://github.com/crossterm-rs/crossterm) — terminal UI
- [image](https://github.com/image-rs/image) — photo → ASCII art
- [serde](https://serde.rs/) + [toml](https://docs.rs/toml/) — content loading

## Folder structure

```
portfolio/
├── src/
│   ├── main.rs        entry point, terminal setup, event loop
│   ├── app.rs          app state and update logic
│   ├── events.rs        keyboard input -> actions
│   ├── ui.rs            drawing/rendering
│   ├── screens/          one file per section (about, projects, skills, ...)
│   ├── data.rs          loads and validates the TOML content
│   ├── ascii.rs          image -> ASCII art conversion
│   ├── theme.rs          colors and styling
│   ├── text.rs           text wrapping helpers
│   ├── commands.rs        the `:` command palette
│   └── opener.rs          opens links in the browser
├── data/                 your content (edit these, not the Rust code)
│   ├── profile.toml       name, bio, portrait settings, contact links
│   ├── projects.toml
│   ├── skills.toml
│   ├── experience.toml
│   └── achievements.toml
├── assets/
│   └── profile.png        your photo, compiled into the binary at build time
├── install.sh             Linux/macOS installer
├── install.ps1            Windows installer
└── Cargo.toml              package metadata and dependencies
```

## Customize the content

Edit the files in `data/` — everything on screen (bio, projects, skills,
links) comes from there, not from the Rust source. To change your photo,
replace `assets/profile.png` and rebuild.

## License

MIT — see [LICENSE](LICENSE).