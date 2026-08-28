//! Dashboard state, refreshed on a timer and on demand.

use anonveil_core::config::AnonveilConfig;
use anonveil_core::state::StateSnapshot;

pub struct App {
    pub state: StateSnapshot,
    pub tor_bootstrapped: Option<bool>,
    pub circuit_established: Option<bool>,
    pub control_error: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: StateSnapshot::default(),
            tor_bootstrapped: None,
            circuit_established: None,
            control_error: None,
            should_quit: false,
        }
    }

    /// Reload persisted state and, if active, query Tor's control port
    /// for live circuit info. Never fails the dashboard itself — any
    /// error (e.g. insufficient permissions to read the cookie file when
    /// not running as root) is surfaced as a status line, not a crash.
    pub async fn refresh(&mut self, config: &AnonveilConfig) {
        self.state = anonveil_priv::snapshot::load_state().unwrap_or_default();
        self.tor_bootstrapped = None;
        self.circuit_established = None;
        self.control_error = None;

        if !self.state.active {
            return;
        }

        match anonveil_priv::control_session::connect_and_authenticate(config.network.control_port)
            .await
        {
            Ok(mut client) => {
                match client
                    .get_info(&["status/bootstrap-phase", "status/circuit-established"])
                    .await
                {
                    Ok(info) => {
                        self.tor_bootstrapped = Some(
                            info.get("status/bootstrap-phase")
                                .map(|v| v.contains("PROGRESS=100"))
                                .unwrap_or(false),
                        );
                        self.circuit_established = Some(
                            info.get("status/circuit-established")
                                .map(|v| v == "1")
                                .unwrap_or(false),
                        );
                    }
                    Err(e) => self.control_error = Some(e.to_string()),
                }
            }
            Err(e) => self.control_error = Some(e.to_string()),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
