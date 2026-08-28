//! Opens the *real* Tor control-port connection and drives
//! `anonveil_core::control::ControlClient` over it. Everything about
//! the wire protocol itself lives in `anonveil-core`; this module only
//! supplies the live TCP socket and the cookie bytes read from disk.

use std::time::Duration;

use anonveil_core::control::ControlClient;
use tokio::net::TcpStream;
use tokio::time::sleep;
use tracing::debug;

use crate::error::{PrivError, PrivResult};

pub type LiveControlClient = ControlClient<TcpStream>;

/// Connect to `127.0.0.1:<control_port>`, learn the cookie file path via
/// `PROTOCOLINFO`, read it, and authenticate. Returns a ready-to-use,
/// authenticated client.
pub async fn connect_and_authenticate(control_port: u16) -> PrivResult<LiveControlClient> {
    let stream = TcpStream::connect(("127.0.0.1", control_port)).await?;
    let mut client = ControlClient::new(stream);

    let info = client.protocol_info().await?;
    let cookie_path = info.cookie_file.ok_or_else(|| {
        PrivError::Core(anonveil_core::CoreError::ControlProtocol(
            "tor did not report a COOKIEFILE — is CookieAuthentication enabled? \
             (see the AnonVeil-managed torrc fragment)"
                .to_string(),
        ))
    })?;
    let cookie = std::fs::read(&cookie_path)?;
    client.authenticate(&cookie).await?;
    Ok(client)
}

/// Extract the integer after `PROGRESS=` in a
/// `status/bootstrap-phase` GETINFO value, without pulling in a regex
/// dependency for one field.
fn parse_bootstrap_progress(value: &str) -> Option<u8> {
    let after = value.split("PROGRESS=").nth(1)?;
    let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

/// Poll `GETINFO status/bootstrap-phase` until Tor reports 100% or
/// `timeout` elapses.
pub async fn wait_for_bootstrap(
    client: &mut LiveControlClient,
    timeout: Duration,
) -> PrivResult<()> {
    let start = std::time::Instant::now();
    loop {
        let info = client.get_info(&["status/bootstrap-phase"]).await?;
        let progress = info
            .get("status/bootstrap-phase")
            .and_then(|v| parse_bootstrap_progress(v))
            .unwrap_or(0);
        debug!(progress, "tor bootstrap");
        if progress >= 100 {
            return Ok(());
        }
        if start.elapsed() >= timeout {
            return Err(PrivError::BootstrapTimeout(progress, timeout.as_secs()));
        }
        sleep(Duration::from_millis(750)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bootstrap_progress() {
        assert_eq!(
            parse_bootstrap_progress("NOTICE BOOTSTRAP PROGRESS=100 TAG=done SUMMARY=Done"),
            Some(100)
        );
        assert_eq!(
            parse_bootstrap_progress("NOTICE BOOTSTRAP PROGRESS=45 TAG=conn_dir"),
            Some(45)
        );
        assert_eq!(parse_bootstrap_progress("garbage"), None);
    }
}
