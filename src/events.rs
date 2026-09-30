use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::screens::Screen;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    PageUp,
    PageDown,
    First,
    Last,
    Select,
    Back,
    Quit,
    Goto(Screen),
    Help,
    OpenCommand,
    Type(char),
    Backspace,
    Submit,
    Cancel,
}

pub fn map_key(key: KeyEvent, command_mode: bool) -> Option<Action> {
    if key.kind == KeyEventKind::Release {
        return None;
    }
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    if ctrl && matches!(key.code, KeyCode::Char('c' | 'd')) {
        return Some(Action::Quit);
    }

    if command_mode {
        return match key.code {
            KeyCode::Enter => Some(Action::Submit),
            KeyCode::Esc => Some(Action::Cancel),
            KeyCode::Backspace => Some(Action::Backspace),
            KeyCode::Char('u') if ctrl => Some(Action::Cancel),
            KeyCode::Char(c) if !ctrl => Some(Action::Type(c)),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => Some(Action::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::Down),
        KeyCode::PageUp => Some(Action::PageUp),
        KeyCode::PageDown | KeyCode::Char(' ') => Some(Action::PageDown),
        KeyCode::Home | KeyCode::Char('g') => Some(Action::First),
        KeyCode::End | KeyCode::Char('G') => Some(Action::Last),
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => Some(Action::Select),
        KeyCode::Esc | KeyCode::Left | KeyCode::Backspace | KeyCode::Char('h') => {
            Some(Action::Back)
        }
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char(':') => Some(Action::OpenCommand),
        KeyCode::Char('?') => Some(Action::Help),
        KeyCode::Char(c @ '1'..='7') => {
            let index = usize::from(c as u8 - b'1');
            Some(Action::Goto(Screen::MENU[index]))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn required_navigation_keys() {
        assert_eq!(map_key(key(KeyCode::Up), false), Some(Action::Up));
        assert_eq!(map_key(key(KeyCode::Down), false), Some(Action::Down));
        assert_eq!(map_key(key(KeyCode::Enter), false), Some(Action::Select));
        assert_eq!(map_key(key(KeyCode::Esc), false), Some(Action::Back));
        assert_eq!(map_key(key(KeyCode::Char('q')), false), Some(Action::Quit));
    }

    #[test]
    fn number_shortcuts_cover_the_menu() {
        for (i, screen) in Screen::MENU.iter().enumerate() {
            let c = char::from(b'1' + i as u8);
            assert_eq!(
                map_key(key(KeyCode::Char(c)), false),
                Some(Action::Goto(*screen))
            );
        }
        assert_eq!(map_key(key(KeyCode::Char('8')), false), None);
    }

    #[test]
    fn command_mode_treats_keys_as_text() {
        assert_eq!(
            map_key(key(KeyCode::Char('q')), true),
            Some(Action::Type('q'))
        );
        assert_eq!(
            map_key(key(KeyCode::Char('1')), true),
            Some(Action::Type('1'))
        );
        assert_eq!(map_key(key(KeyCode::Esc), true), Some(Action::Cancel));
        assert_eq!(map_key(key(KeyCode::Enter), true), Some(Action::Submit));
    }

    #[test]
    fn ctrl_c_always_quits() {
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(map_key(ctrl_c, false), Some(Action::Quit));
        assert_eq!(map_key(ctrl_c, true), Some(Action::Quit));
    }

    #[test]
    fn releases_are_ignored() {
        let mut release = key(KeyCode::Down);
        release.kind = KeyEventKind::Release;
        assert_eq!(map_key(release, false), None);
    }
}
