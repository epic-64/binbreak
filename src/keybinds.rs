use crossterm::event::{KeyCode, KeyEvent};

pub(crate) fn is_up(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Up)
}

pub(crate) fn is_down(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Down)
}

pub(crate) fn is_left(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Left)
}

pub(crate) fn is_right(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Right)
}

pub(crate) fn is_select(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Enter)
}

pub(crate) fn is_exit(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Esc)
}
