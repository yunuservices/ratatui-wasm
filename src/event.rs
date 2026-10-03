use crate::wit::{Event, KeyCode, KeyEvent, ResizeEvent};

/// A key press with a modifier bitmask (shift=1, ctrl=2, alt=4, super=8).
pub const fn key(code: KeyCode, modifiers: u8) -> Event {
    Event::Key(KeyEvent { code, modifiers })
}

pub const fn resize(cols: u16, rows: u16) -> Event {
    Event::Resize(ResizeEvent { cols, rows })
}

pub fn char_key(character: char) -> KeyCode {
    KeyCode::Codepoint(character.to_string())
}

pub const fn f_key(number: u8) -> KeyCode {
    KeyCode::Function(number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resize_builds_event() {
        let event = resize(80, 24);
        assert!(
            matches!(event, Event::Resize(ResizeEvent { cols: 80, rows: 24 })),
            "got {event:?}"
        );
    }

    #[test]
    fn f_key_builds_function_code() {
        let code = f_key(5);
        assert!(matches!(code, KeyCode::Function(5)), "got {code:?}");
    }

    #[test]
    fn key_builds_key_event() {
        let code = char_key('x');
        let event = key(code, 1);
        assert!(
            matches!(event, Event::Key(KeyEvent { code: KeyCode::Codepoint(ref s), modifiers: 1 }) if s == "x"),
            "got {event:?}"
        );
    }
}
