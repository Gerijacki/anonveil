//! `anonveil check` — Tor-reachability self-test. See [`types`] for the
//! honest scope note on what this can and can't verify about DNS leaks.

pub mod client;
pub mod types;

pub use client::{run_check, HttpProbe, TOR_CHECK_URL};
pub use types::{interpret_check_response, LeakCheckResult};
