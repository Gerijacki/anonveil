//! The AnonVeil dashboard: one live-status screen, launched when
//! `anonveil` is run with no subcommand (or via `anonveil dashboard`
//! explicitly).

pub mod actions;
pub mod app;
pub mod event;
pub mod theme;
pub mod ui;

use std::time::{Duration, Instant};

use anonveil_core::config::AnonveilConfig;
use anyhow::Result;

use app::App;

const REFRESH_INTERVAL: Duration = Duration::from_secs(3);
const POLL_INTERVAL: Duration = Duration::from_millis(200);

pub async fn run(config: &AnonveilConfig) -> Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new();
    app.refresh(config).await;
    let mut last_refresh = Instant::now();

    let result = loop {
        if let Err(e) = terminal.draw(|frame| ui::draw(frame, &app)) {
            break Err(e.into());
        }

        match event::poll_key(POLL_INTERVAL) {
            Ok(Some(key)) => {
                if event::is_quit(&key) {
                    break Ok(());
                }
                if event::is_refresh(&key) {
                    app.refresh(config).await;
                    last_refresh = Instant::now();
                }
                if let Some(action) = actions::Action::for_key(&key) {
                    // Leave the dashboard's alternate screen entirely for
                    // the duration of the action — see actions.rs for why.
                    ratatui::restore();
                    actions::run(action, config, app.state.active).await;
                    terminal = ratatui::init();
                    app.refresh(config).await;
                    last_refresh = Instant::now();
                }
            }
            Ok(None) => {}
            Err(e) => break Err(e.into()),
        }

        if last_refresh.elapsed() >= REFRESH_INTERVAL {
            app.refresh(config).await;
            last_refresh = Instant::now();
        }

        if app.should_quit {
            break Ok(());
        }
    };

    ratatui::restore();
    result
}
