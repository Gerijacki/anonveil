//! Shared error type for `anonveil-core`.

/// Errors that can occur while computing rulesets, speaking the Tor
/// control protocol over a caller-supplied transport, or parsing config.
///
/// This type deliberately contains no variant that implies a file was
/// opened or a process was spawned directly by this crate — those
/// failures are reported by `anonveil-priv` using its own error type,
/// which may *wrap* a [`CoreError`] returned from a pure computation.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("i/o error on control-port transport: {0}")]
    Io(#[from] std::io::Error),

    #[error("tor control-port authentication failed: {0}")]
    AuthFailed(String),

    #[error("unexpected reply from tor control port: {0}")]
    ControlProtocol(String),

    #[error("failed to parse config: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("failed to serialize config: {0}")]
    ConfigSerialize(#[from] toml::ser::Error),

    #[error("failed to (de)serialize state snapshot: {0}")]
    StateSerde(#[from] serde_json::Error),

    #[error("invalid firewall configuration: {0}")]
    InvalidFirewallConfig(String),
}

pub type CoreResult<T> = Result<T, CoreError>;
