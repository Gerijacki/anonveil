//! Thin wrapper over crossterm's blocking event poll, re-exported
//! through `ratatui::crossterm` so the version always matches whatever
//! backend ratatui itself bundles.

use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

/// Poll for up to `timeout`; returns the key pressed, if any (ignoring
/// key-release events, which some terminals emit).
pub fn poll_key(timeout: Duration) -> std::io::Result<Option<KeyEvent>> {
    if !event::poll(timeout)? {
        return Ok(None);
    }
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => Ok(Some(key)),
        _ => Ok(None),
    }
}

pub fn is_quit(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
}

pub fn is_refresh(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('r'))
}

/// Start if inactive, stop if active — the dashboard doesn't expose
/// separate start/stop keys, it toggles based on what `app.state.active`
/// already says (same as the icon/label it's showing).
pub fn is_toggle_start(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('s'))
}

pub fn is_panic(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('p'))
}

pub fn is_newnym(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('n'))
}
