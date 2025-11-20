use crossterm::event::{KeyCode, KeyEvent};

pub(crate) fn is_up(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Up | KeyCode::Char('k'))
}

pub(crate) fn is_down(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Down | KeyCode::Char('j'))
}

pub(crate) fn is_left(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Left | KeyCode::Char('h'))
}

pub(crate) fn is_right(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Right | KeyCode::Char('l'))
}

pub(crate) fn is_select(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Enter)
}

pub(crate) fn is_exit(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Esc | KeyCode::Char('q' | 'Q'))
}
