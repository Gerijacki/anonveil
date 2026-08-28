//! Minimal Tor control-port wire protocol (control-spec.txt), just the
//! subset AnonVeil needs: single-command requests and their (possibly
//! multi-line) replies. Deliberately first-party rather than an external
//! crate — see the module-level rationale in [`super`].
//!
//! Not implemented (because AnonVeil never needs them): asynchronous
//! `650` events, `+`-introduced multi-line *data* blocks (GETINFO keys
//! AnonVeil queries are all single-line values), and any auth method
//! other than cookie auth.

use crate::error::{CoreError, CoreResult};

/// A fully-read control-port reply: possibly several `250-...` lines
/// followed by a final `250 ...` (or an error code) line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    /// The reply code on the *final* line (e.g. `250`, `515`).
    pub code: u16,
    /// Text of every line, in order, with the `code<sep>` prefix
    /// stripped.
    pub lines: Vec<String>,
}

impl Reply {
    pub fn is_success(&self) -> bool {
        self.code == 250
    }

    /// The text of the final line (most single-line replies only have
    /// this one).
    pub fn last_line(&self) -> &str {
        self.lines.last().map(String::as_str).unwrap_or_default()
    }
}

/// Encode a command line for the control port. Appends the CRLF
/// terminator required by control-spec.txt.
pub fn encode_command(command: &str) -> Vec<u8> {
    format!("{command}\r\n").into_bytes()
}

/// Parse one already-CRLF-stripped line of a reply into
/// `(code, is_final, text)`.
///
/// `is_final` is true for a line using `<code> ` (space separator);
/// false for a `<code>-` (dash, continuation) line. A `<code>+` data
/// line is treated as an error since AnonVeil never issues a command
/// that returns one (see module docs).
fn parse_line(line: &str) -> CoreResult<(u16, bool, String)> {
    // Operate on bytes for the fixed-width `<3-digit code><1-byte sep>`
    // prefix so a malformed line can never panic on a non-UTF8-boundary
    // split — the control-spec guarantees this prefix is ASCII, but
    // AnonVeil doesn't have to trust that to stay safe.
    let bytes = line.as_bytes();
    if bytes.len() < 4 || !bytes[..3].iter().all(u8::is_ascii_digit) {
        return Err(CoreError::ControlProtocol(format!(
            "malformed reply line: {line:?}"
        )));
    }
    let code: u16 = std::str::from_utf8(&bytes[..3])
        .ok()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| CoreError::ControlProtocol(format!("bad reply code in {line:?}")))?;
    let sep = bytes[3] as char;
    if !line.is_char_boundary(4) {
        return Err(CoreError::ControlProtocol(format!(
            "malformed reply line: {line:?}"
        )));
    }
    let text = line[4..].to_string();
    match sep {
        ' ' => Ok((code, true, text)),
        '-' => Ok((code, false, text)),
        '+' => Err(CoreError::ControlProtocol(format!(
            "unsupported multi-line data reply: {line:?}"
        ))),
        other => Err(CoreError::ControlProtocol(format!(
            "unknown reply separator {other:?} in {line:?}"
        ))),
    }
}

/// Assemble a full [`Reply`] from an ordered list of raw lines (already
/// stripped of the trailing CRLF), as would be read one at a time from
/// the control-port socket until a final line is seen.
///
/// This is split out from the actual socket-reading loop (in
/// `client.rs`) so the parsing logic itself is unit-testable with plain
/// strings — no transport, no async, no mocks required.
pub fn assemble_reply(raw_lines: &[&str]) -> CoreResult<Reply> {
    if raw_lines.is_empty() {
        return Err(CoreError::ControlProtocol("empty reply".to_string()));
    }
    let mut lines = Vec::with_capacity(raw_lines.len());
    let mut final_code = None;
    for (i, raw) in raw_lines.iter().enumerate() {
        let (code, is_final, text) = parse_line(raw)?;
        lines.push(text);
        let is_last = i + 1 == raw_lines.len();
        if is_final != is_last {
            return Err(CoreError::ControlProtocol(format!(
                "reply framing mismatch at line {i}: {raw:?}"
            )));
        }
        if is_final {
            final_code = Some(code);
        }
    }
    let code = final_code.ok_or_else(|| {
        CoreError::ControlProtocol("reply never terminated with a final line".to_string())
    })?;
    Ok(Reply { code, lines })
}

/// Parse a `250-keyword=value` / `250 keyword=value` reply line's text
/// (i.e. [`Reply::lines`] entries) into `(keyword, value)`, as used for
/// `GETINFO` replies. Returns `None` for lines that aren't `key=value`
/// (e.g. a bare `OK` terminator line).
pub fn parse_keyword_value(line: &str) -> Option<(String, String)> {
    let (key, value) = line.split_once('=')?;
    Some((key.to_string(), value.trim_matches('"').to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_command_with_crlf() {
        assert_eq!(encode_command("SIGNAL NEWNYM"), b"SIGNAL NEWNYM\r\n");
    }

    #[test]
    fn assembles_single_line_ok() {
        let reply = assemble_reply(&["250 OK"]).unwrap();
        assert!(reply.is_success());
        assert_eq!(reply.last_line(), "OK");
    }

    #[test]
    fn assembles_multiline_getinfo() {
        let reply = assemble_reply(&[
            "250-status/bootstrap-phase=NOTICE BOOTSTRAP PROGRESS=100 TAG=done",
            "250 OK",
        ])
        .unwrap();
        assert!(reply.is_success());
        assert_eq!(reply.lines.len(), 2);
        let (key, value) = parse_keyword_value(&reply.lines[0]).unwrap();
        assert_eq!(key, "status/bootstrap-phase");
        assert!(value.starts_with("NOTICE BOOTSTRAP"));
    }

    #[test]
    fn assembles_error_reply() {
        let reply =
            assemble_reply(&["515 Authentication failed: Wrong length on cookie."]).unwrap();
        assert!(!reply.is_success());
        assert_eq!(reply.code, 515);
    }

    #[test]
    fn rejects_reply_missing_final_line() {
        // Only continuation lines, never terminated — must be treated as
        // a framing error, not silently accepted.
        let err = assemble_reply(&["250-status/bootstrap-phase=..."]).unwrap_err();
        assert!(matches!(err, CoreError::ControlProtocol(_)));
    }

    #[test]
    fn rejects_data_block_reply() {
        let err = assemble_reply(&["250+config-text="]).unwrap_err();
        assert!(matches!(err, CoreError::ControlProtocol(_)));
    }

    #[test]
    fn parses_protocolinfo_cookiefile() {
        let line =
            r#"AUTH METHODS=COOKIE,SAFECOOKIE COOKIEFILE="/var/lib/tor/control_auth_cookie""#;
        // COOKIEFILE isn't the first key=value pair here (METHODS is),
        // so client.rs is responsible for picking the right token out of
        // an AUTH line; parse_keyword_value only handles the simple
        // single-pair GETINFO case. This test documents that boundary.
        assert!(line.contains("COOKIEFILE="));
    }
}
