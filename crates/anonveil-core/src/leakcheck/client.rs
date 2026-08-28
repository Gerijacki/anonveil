//! Runs the Tor-reachability probe against a caller-supplied HTTP
//! fetcher, so the actual HTTP client (`reqwest`, wired up in
//! `anonveil-cli`) never has to be linked into this crate.

use crate::error::CoreResult;

use super::types::{interpret_check_response, LeakCheckResult};

/// Tor Project's own IP/Tor-status check endpoint.
pub const TOR_CHECK_URL: &str = "https://check.torproject.org/api/ip";

/// Anything capable of fetching a URL's body as text. Implemented for
/// the real `reqwest`-based client in `anonveil-cli`, and for a canned
/// in-memory responder in tests.
///
/// `async fn` in a trait means the returned future isn't guaranteed
/// `Send` — fine here, since AnonVeil only ever calls `run_check` inline
/// (never `tokio::spawn`s it onto another task), so a `Send` bound would
/// add nothing.
#[allow(async_fn_in_trait)]
pub trait HttpProbe {
    async fn get(&self, url: &str) -> CoreResult<String>;
}

/// Fetch [`TOR_CHECK_URL`] through `probe` and interpret the result.
///
/// Deliberately takes the fetch mechanism as a parameter rather than
/// owning it: whether that `GET` actually goes through Tor depends
/// entirely on the kill switch already being active, which is exactly
/// what this check is trying to demonstrate — this function only
/// interprets the response, it never decides how the request is routed.
pub async fn run_check<P: HttpProbe>(probe: &P) -> CoreResult<LeakCheckResult> {
    let body = probe.get(TOR_CHECK_URL).await?;
    interpret_check_response(&body)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CannedProbe(&'static str);

    impl HttpProbe for CannedProbe {
        async fn get(&self, url: &str) -> CoreResult<String> {
            assert_eq!(url, TOR_CHECK_URL);
            Ok(self.0.to_string())
        }
    }

    #[tokio::test]
    async fn run_check_reports_tor_reachable() {
        let probe = CannedProbe(r#"{"IsTor":true,"IP":"51.15.0.1"}"#);
        let result = run_check(&probe).await.unwrap();
        assert!(result.tor_reachable);
    }
}
