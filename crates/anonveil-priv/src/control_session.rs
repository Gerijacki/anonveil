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

/// How long to keep retrying the *initial* TCP connect before giving up.
/// `reload-or-restart` returning success doesn't guarantee the control
/// port's listener is already accepting connections — a HUP reload can
/// briefly close and reopen its sockets — so a bare `connect` right after
/// is a real, if usually brief, race, not just a theoretical one.
const CONNECT_RETRY_BUDGET: Duration = Duration::from_secs(10);
const CONNECT_RETRY_INTERVAL: Duration = Duration::from_millis(250);

/// Connect to `127.0.0.1:<control_port>`, learn the cookie file path via
/// `PROTOCOLINFO`, read it, and authenticate. Returns a ready-to-use,
/// authenticated client.
pub async fn connect_and_authenticate(control_port: u16) -> PrivResult<LiveControlClient> {
    let stream = connect_with_retry(control_port).await?;
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

/// Retry a plain TCP connect to the control port for up to
/// [`CONNECT_RETRY_BUDGET`], treating connection-refused as transient.
/// Any other error (e.g. a DNS/address error, which can't happen for a
/// literal loopback address, but the type doesn't know that) is returned
/// immediately rather than retried.
async fn connect_with_retry(control_port: u16) -> PrivResult<TcpStream> {
    let start = std::time::Instant::now();
    loop {
        match TcpStream::connect(("127.0.0.1", control_port)).await {
            Ok(stream) => return Ok(stream),
            Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => {
                if start.elapsed() >= CONNECT_RETRY_BUDGET {
                    return Err(e.into());
                }
                sleep(CONNECT_RETRY_INTERVAL).await;
            }
            Err(e) => return Err(e.into()),
        }
    }
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
