//! Dashboard-triggered actions (`[s]` start/stop, `[p]` panic, `[n]`
//! newnym). Deliberately does **not** try to run these quietly inside the
//! TUI's alternate screen and summarize the result in a toast: `start`/
//! `stop` are multi-step, can take up to 90s waiting for Tor to
//! bootstrap, and already have well-designed step-by-step
//! `style::step`/`ok`/`warn`/`error` output — hiding that behind a
//! cramped status line would be a worse experience, not a better one, and
//! would mean either duplicating that reporting or teaching the command
//! modules to return structured progress instead of printing it (a much
//! bigger change for no real benefit). Instead: leave the alternate
//! screen, run the action exactly as the plain CLI subcommand would
//! (real output, real terminal), wait for the user to acknowledge it,
//! then re-enter the dashboard and refresh.

use anonveil_core::config::AnonveilConfig;
use ratatui::crossterm::event::KeyEvent;

use crate::commands;
use crate::style;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    ToggleStart,
    Panic,
    Newnym,
}

impl Action {
    pub fn for_key(key: &KeyEvent) -> Option<Self> {
        if super::event::is_toggle_start(key) {
            Some(Self::ToggleStart)
        } else if super::event::is_panic(key) {
            Some(Self::Panic)
        } else if super::event::is_newnym(key) {
            Some(Self::Newnym)
        } else {
            None
        }
    }
}

/// Run `action` with the terminal already restored to normal mode. Always
/// prints *something* (the action's own output, or an error) and always
/// waits for the user before the caller re-enters the dashboard — so the
/// screen never flickers back before there was anything to read.
pub async fn run(action: Action, config: &AnonveilConfig, currently_active: bool) {
    println!();
    let result = match action {
        Action::ToggleStart if currently_active => commands::stop::run(false),
        Action::ToggleStart => commands::start::run(config, false).await,
        Action::Panic => commands::panic::run(),
        Action::Newnym => commands::newnym::run(config).await,
    };
    if let Err(e) = result {
        style::error(&format!("{e}"));
    }
    println!();
    print!("Press Enter to return to the dashboard...");
    use std::io::Write;
    let _ = std::io::stdout().flush();
    let mut discard = String::new();
    let _ = std::io::stdin().read_line(&mut discard);
}
