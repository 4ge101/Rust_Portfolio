use std::io;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;

pub type OpenResult = Result<String, String>;

pub fn is_allowed_url(url: &str) -> bool {
    let has_scheme = ["https://", "http://", "mailto:"]
        .iter()
        .any(|scheme| url.len() > scheme.len() && url.to_ascii_lowercase().starts_with(scheme));
    has_scheme && url.len() <= 2048 && !url.chars().any(|c| c.is_whitespace() || c.is_control())
}

fn opener_program() -> Option<&'static str> {
    if cfg!(target_os = "macos") {
        Some("open")
    } else if cfg!(all(unix, not(target_os = "macos"))) {
        Some("xdg-open")
    } else {
        None
    }
}

pub fn open(url: &str, tx: Sender<OpenResult>) {
    if !is_allowed_url(url) {
        let _ = tx.send(Err(format!("refusing to open unsupported link: {url}")));
        return;
    }
    let Some(program) = opener_program() else {
        let _ = tx.send(Err("opening links is not supported on this OS".to_owned()));
        return;
    };

    let spawned = Command::new(program)
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    let mut child = match spawned {
        Ok(child) => child,
        Err(err) => {
            let reason = if err.kind() == io::ErrorKind::NotFound {
                format!("`{program}` not found")
            } else {
                format!("could not run `{program}`: {err}")
            };
            let _ = tx.send(Err(format!("{reason} - link: {url}")));
            return;
        }
    };

    let url = url.to_owned();
    thread::spawn(move || {
        let message = match child.wait() {
            Ok(status) if status.success() => Ok(url),
            Ok(status) => Err(format!("`{program}` failed ({status}) - link: {url}")),
            Err(err) => Err(format!("`{program}` failed ({err}) - link: {url}")),
        };
        let _ = tx.send(message);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_web_and_mail_links() {
        assert!(is_allowed_url("https://github.com/rust-lang/rust"));
        assert!(is_allowed_url("http://example.com"));
        assert!(is_allowed_url("HTTPS://EXAMPLE.COM"));
        assert!(is_allowed_url("mailto:someone@example.com"));
    }

    #[test]
    fn rejects_everything_else() {
        assert!(!is_allowed_url(""));
        assert!(!is_allowed_url("https://"));
        assert!(!is_allowed_url("--help"));
        assert!(!is_allowed_url("-x https://example.com"));
        assert!(!is_allowed_url("file:///etc/passwd"));
        assert!(!is_allowed_url("javascript:alert(1)"));
        assert!(!is_allowed_url("https://exa mple.com"));
        assert!(!is_allowed_url("https://example.com\n--evil"));
    }

    #[test]
    fn unsupported_links_report_an_error_instead_of_running() {
        let (tx, rx) = std::sync::mpsc::channel();
        open("file:///etc/passwd", tx);
        assert!(rx.recv().unwrap().is_err());
    }
}
