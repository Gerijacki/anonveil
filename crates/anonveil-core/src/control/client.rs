//! The Tor control-port client itself, generic over its transport so it
//! can be exercised in tests against an in-memory duplex stream instead
//! of a real socket. `anonveil-priv` supplies the real transport (a
//! `tokio::net::TcpStream` to `127.0.0.1:<control_port>`).

use std::collections::HashMap;

use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

use crate::error::{CoreError, CoreResult};

use super::cookie::encode_cookie_hex;
use super::protocol::{assemble_reply, encode_command, parse_keyword_value, Reply};

/// What `PROTOCOLINFO` told us about how to authenticate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolInfo {
    pub auth_methods: Vec<String>,
    /// Path to the cookie-auth file, if `COOKIE`/`SAFECOOKIE` is offered.
    /// Always learned from the daemon's own reply — never assumed from a
    /// hardcoded path — since it depends on `DataDirectory`.
    pub cookie_file: Option<String>,
}

/// Split a control-spec `KEY=value KEY="quoted value"` token list on
/// spaces, *except* spaces inside a `"..."` `QuotedString` (control-spec.txt
/// §2.1) — a naive `str::split(' ')` truncates `COOKIEFILE="..."` at the
/// first space once `DataDirectory` (and hence the cookie file's path)
/// contains one, silently handing back a truncated, wrong path.
fn split_respecting_quotes(s: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut in_quotes = false;
    let mut start = 0;
    for (i, c) in s.char_indices() {
        match c {
            '"' => in_quotes = !in_quotes,
            ' ' if !in_quotes => {
                if i > start {
                    tokens.push(&s[start..i]);
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        tokens.push(&s[start..]);
    }
    tokens
}

/// A connected-but-not-yet-authenticated (or authenticated) control-port
/// session. `T` is anything that looks like a duplex byte stream.
pub struct ControlClient<T> {
    io: BufReader<T>,
}

impl<T: AsyncRead + AsyncWrite + Unpin> ControlClient<T> {
    /// Wrap an already-connected transport. Does not authenticate —
    /// call [`Self::authenticate`] (after [`Self::protocol_info`] if the
    /// cookie file path isn't already known) before issuing any other
    /// command.
    pub fn new(transport: T) -> Self {
        Self {
            io: BufReader::new(transport),
        }
    }

    async fn send_command(&mut self, command: &str) -> CoreResult<Reply> {
        self.io.write_all(&encode_command(command)).await?;
        self.io.flush().await?;
        self.read_reply().await
    }

    async fn read_reply(&mut self) -> CoreResult<Reply> {
        let mut raw_lines: Vec<String> = Vec::new();
        loop {
            let mut line = String::new();
            let n = self.io.read_line(&mut line).await?;
            if n == 0 {
                return Err(CoreError::Io(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "control port closed connection mid-reply",
                )));
            }
            let line = line.trim_end_matches(['\r', '\n']).to_string();
            let is_final = line.as_bytes().get(3).map(|b| *b == b' ').unwrap_or(false);
            let is_continuation = line.as_bytes().get(3).map(|b| *b == b'-').unwrap_or(false);
            raw_lines.push(line);
            if is_final || !is_continuation {
                // Either a proper final line, or a malformed/short line —
                // in both cases stop reading and let assemble_reply()
                // report a precise parse error rather than looping
                // forever waiting for a terminator that will never come.
                break;
            }
        }
        let refs: Vec<&str> = raw_lines.iter().map(String::as_str).collect();
        assemble_reply(&refs)
    }

    /// `PROTOCOLINFO` — safe to call before authenticating.
    pub async fn protocol_info(&mut self) -> CoreResult<ProtocolInfo> {
        let reply = self.send_command("PROTOCOLINFO 1").await?;
        if !reply.is_success() {
            return Err(CoreError::ControlProtocol(format!(
                "PROTOCOLINFO failed: {}",
                reply.last_line()
            )));
        }
        let mut auth_methods = Vec::new();
        let mut cookie_file = None;
        for line in &reply.lines {
            if let Some(rest) = line.strip_prefix("AUTH ") {
                for token in split_respecting_quotes(rest) {
                    if let Some(methods) = token.strip_prefix("METHODS=") {
                        auth_methods = methods.split(',').map(str::to_string).collect();
                    } else if let Some(path) = token.strip_prefix("COOKIEFILE=") {
                        cookie_file = Some(path.trim_matches('"').to_string());
                    }
                }
            }
        }
        Ok(ProtocolInfo {
            auth_methods,
            cookie_file,
        })
    }

    /// `AUTHENTICATE <hex(cookie)>`. v0.1 only supports cookie auth (see
    /// module docs on [`super`]).
    pub async fn authenticate(&mut self, cookie: &[u8]) -> CoreResult<()> {
        let hex = encode_cookie_hex(cookie);
        let reply = self.send_command(&format!("AUTHENTICATE {hex}")).await?;
        if !reply.is_success() {
            return Err(CoreError::AuthFailed(reply.last_line().to_string()));
        }
        Ok(())
    }

    /// `SIGNAL NEWNYM` — request a fresh circuit/identity.
    pub async fn signal_newnym(&mut self) -> CoreResult<()> {
        let reply = self.send_command("SIGNAL NEWNYM").await?;
        if !reply.is_success() {
            return Err(CoreError::ControlProtocol(format!(
                "SIGNAL NEWNYM failed: {}",
                reply.last_line()
            )));
        }
        Ok(())
    }

    /// `GETINFO <keys...>` — returns whatever `key=value` pairs came
    /// back. Keys Tor doesn't recognize simply won't appear in the map;
    /// callers should treat a missing key as "unknown", not an error.
    pub async fn get_info(&mut self, keys: &[&str]) -> CoreResult<HashMap<String, String>> {
        let reply = self
            .send_command(&format!("GETINFO {}", keys.join(" ")))
            .await?;
        if !reply.is_success() {
            return Err(CoreError::ControlProtocol(format!(
                "GETINFO failed: {}",
                reply.last_line()
            )));
        }
        let mut map = HashMap::new();
        for line in &reply.lines {
            if let Some((k, v)) = parse_keyword_value(line) {
                map.insert(k, v);
            }
        }
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    /// Spawn a fake control-port "server" task over one end of an
    /// in-memory duplex stream that just echoes back a canned reply for
    /// whatever single line it reads, and hand back a `ControlClient`
    /// wired to the other end.
    fn client_with_scripted_reply(reply: &'static str) -> ControlClient<tokio::io::DuplexStream> {
        let (client_end, mut server_end) = duplex(4096);
        tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];
            use tokio::io::AsyncReadExt;
            let _ = server_end.read(&mut buf).await; // drain the request
            let _ = server_end.write_all(reply.as_bytes()).await;
        });
        ControlClient::new(client_end)
    }

    #[tokio::test]
    async fn authenticate_success() {
        let mut client = client_with_scripted_reply("250 OK\r\n");
        client.authenticate(&[0x01, 0x02]).await.unwrap();
    }

    #[tokio::test]
    async fn authenticate_failure_surfaces_reason() {
        let mut client =
            client_with_scripted_reply("515 Authentication failed: Wrong length on cookie.\r\n");
        let err = client.authenticate(&[0x01]).await.unwrap_err();
        match err {
            CoreError::AuthFailed(msg) => assert!(msg.contains("Wrong length")),
            other => panic!("expected AuthFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn signal_newnym_success() {
        let mut client = client_with_scripted_reply("250 OK\r\n");
        client.signal_newnym().await.unwrap();
    }

    #[tokio::test]
    async fn get_info_parses_multiline_reply() {
        let mut client = client_with_scripted_reply(
            "250-status/bootstrap-phase=NOTICE BOOTSTRAP PROGRESS=100 TAG=done\r\n250 OK\r\n",
        );
        let info = client.get_info(&["status/bootstrap-phase"]).await.unwrap();
        assert_eq!(
            info.get("status/bootstrap-phase").unwrap(),
            "NOTICE BOOTSTRAP PROGRESS=100 TAG=done"
        );
    }

    #[tokio::test]
    async fn protocol_info_extracts_cookie_file() {
        let mut client = client_with_scripted_reply(
            "250-PROTOCOLINFO 1\r\n\
             250-AUTH METHODS=COOKIE,SAFECOOKIE COOKIEFILE=\"/var/lib/tor/control_auth_cookie\"\r\n\
             250-VERSION Tor=\"0.4.8.9\"\r\n\
             250 OK\r\n",
        );
        let info = client.protocol_info().await.unwrap();
        assert_eq!(
            info.cookie_file.as_deref(),
            Some("/var/lib/tor/control_auth_cookie")
        );
        assert!(info.auth_methods.contains(&"COOKIE".to_string()));
    }

    #[tokio::test]
    async fn protocol_info_extracts_cookie_file_containing_a_space() {
        // A DataDirectory (and hence COOKIEFILE) with a space in it is
        // unusual but legal — control-spec.txt quotes it for exactly this
        // reason. A naive split(' ') would truncate the path here.
        let mut client = client_with_scripted_reply(
            "250-AUTH METHODS=COOKIE COOKIEFILE=\"/var/lib/my tor dir/control_auth_cookie\"\r\n\
             250 OK\r\n",
        );
        let info = client.protocol_info().await.unwrap();
        assert_eq!(
            info.cookie_file.as_deref(),
            Some("/var/lib/my tor dir/control_auth_cookie")
        );
    }

    #[test]
    fn quote_aware_split_keeps_spaces_inside_quotes_together() {
        let tokens = split_respecting_quotes(
            r#"METHODS=COOKIE,SAFECOOKIE COOKIEFILE="/var/lib/my tor dir/cookie""#,
        );
        assert_eq!(
            tokens,
            vec![
                "METHODS=COOKIE,SAFECOOKIE",
                r#"COOKIEFILE="/var/lib/my tor dir/cookie""#,
            ]
        );
    }

    #[tokio::test]
    async fn unexpected_disconnect_is_reported_as_io_error() {
        let (client_end, server_end) = duplex(4096);
        drop(server_end); // hang up immediately
        let mut client = ControlClient::new(client_end);
        let err = client.signal_newnym().await.unwrap_err();
        assert!(matches!(err, CoreError::Io(_)));
    }
}
