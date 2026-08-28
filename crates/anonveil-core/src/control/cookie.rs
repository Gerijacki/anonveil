//! Cookie-auth encoding for the Tor control port.
//!
//! AnonVeil uses `CookieAuthentication` exclusively in v0.1 (see
//! `torrc::generate` and `threat-model.md`); password auth is out of
//! scope. `anonveil-priv` reads the raw cookie file bytes (whose path is
//! learned from `PROTOCOLINFO`, never hardcoded — see `client.rs`) and
//! passes them here to build the `AUTHENTICATE` command.

/// Hex-encode a raw 32-byte control-auth cookie for use in an
/// `AUTHENTICATE <hex>` command, per control-spec.txt.
pub fn encode_cookie_hex(cookie: &[u8]) -> String {
    let mut s = String::with_capacity(cookie.len() * 2);
    for byte in cookie {
        s.push_str(&format!("{byte:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_empty() {
        assert_eq!(encode_cookie_hex(&[]), "");
    }

    #[test]
    fn encodes_known_bytes() {
        assert_eq!(encode_cookie_hex(&[0x00, 0xff, 0x0a, 0x9b]), "00ff0a9b");
    }
}
