//! `anonveil check` — Tor-reachability self-test. See
//! `anonveil_core::leakcheck` for the honest scope note on what this
//! does and doesn't verify.

use anonveil_core::leakcheck::{run_check, HttpProbe};
use anonveil_core::{CoreError, CoreResult};
use anyhow::Result;

use crate::style;

struct ReqwestProbe(reqwest::Client);

impl HttpProbe for ReqwestProbe {
    async fn get(&self, url: &str) -> CoreResult<String> {
        let response = self
            .0
            .get(url)
            .send()
            .await
            .map_err(|e| CoreError::ControlProtocol(format!("request to {url} failed: {e}")))?;
        response
            .text()
            .await
            .map_err(|e| CoreError::ControlProtocol(format!("reading response body: {e}")))
    }
}

pub async fn run() -> Result<()> {
    style::step("checking Tor reachability via check.torproject.org...");
    let probe = ReqwestProbe(reqwest::Client::builder().build()?);
    let result = run_check(&probe).await?;

    if result.tor_reachable {
        style::ok("Traffic from this host is exiting through Tor.");
        if let Some(ip) = &result.exit_ip {
            style::dim(&format!("  exit relay IP: {ip}"));
        }
    } else {
        style::error("This connection is NOT going through Tor.");
        style::dim("  Run `anonveil start` first, or check `anonveil status`.");
    }

    style::dim(
        "  note: this confirms Tor reachability, not fingerprinting resistance — see \
         threat-model.md. Use Tor Browser for anything where fingerprinting matters.",
    );
    Ok(())
}
