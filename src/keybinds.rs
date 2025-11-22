use crossterm::event::{KeyCode, KeyEvent};

pub const fn is_up(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Up | KeyCode::Char('k'))
}

pub const fn is_down(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Down | KeyCode::Char('j'))
}

pub const fn is_left(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Left | KeyCode::Char('h'))
}

pub const fn is_right(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Right | KeyCode::Char('l'))
}

pub const fn is_select(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Enter)
}

pub const fn is_exit(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Esc | KeyCode::Char('q' | 'Q'))
}
