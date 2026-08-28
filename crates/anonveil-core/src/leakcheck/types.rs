//! Result types for `anonveil check`.
//!
//! Scope note: classic "DNS leak" prevention in AnonVeil is a property
//! of the firewall ruleset itself (`firewall::nft` drops all non-loopback
//! port-53 traffic by construction — see its module docs), not something
//! an application-level HTTP probe can fully verify after the fact. What
//! this module *does* verify honestly is: "does traffic from this host
//! actually egress through the Tor network right now" — by asking Tor
//! Project's own check endpoint, reached the same way any other app's
//! traffic would be (transparently, through the kill switch), which
//! exercises both the DNS and TCP redirection paths for real.

use serde::Deserialize;

use crate::error::{CoreError, CoreResult};

/// Response shape of `https://check.torproject.org/api/ip`.
#[derive(Debug, Deserialize)]
struct TorCheckResponse {
    #[serde(rename = "IsTor")]
    is_tor: bool,
    #[serde(rename = "IP")]
    ip: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeakCheckResult {
    /// Whether Tor Project's check service confirms this connection came
    /// from a Tor exit relay.
    pub tor_reachable: bool,
    /// The apparent public IP the check service saw (a Tor exit relay's
    /// IP when `tor_reachable` is true).
    pub exit_ip: Option<String>,
}

/// Parse the JSON body of a `check.torproject.org/api/ip` response.
/// Pure, so it can be unit-tested with canned response bodies with no
/// network access at all.
pub fn interpret_check_response(body: &str) -> CoreResult<LeakCheckResult> {
    let parsed: TorCheckResponse = serde_json::from_str(body)
        .map_err(|e| CoreError::ControlProtocol(format!("malformed tor-check response: {e}")))?;
    Ok(LeakCheckResult {
        tor_reachable: parsed.is_tor,
        exit_ip: parsed.ip,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_positive_response() {
        let body = r#"{"IsTor":true,"IP":"51.15.0.1"}"#;
        let result = interpret_check_response(body).unwrap();
        assert!(result.tor_reachable);
        assert_eq!(result.exit_ip.as_deref(), Some("51.15.0.1"));
    }

    #[test]
    fn parses_negative_response() {
        let body = r#"{"IsTor":false,"IP":"203.0.113.9"}"#;
        let result = interpret_check_response(body).unwrap();
        assert!(!result.tor_reachable);
    }

    #[test]
    fn rejects_malformed_body() {
        let err = interpret_check_response("not json").unwrap_err();
        assert!(matches!(err, CoreError::ControlProtocol(_)));
    }
}
