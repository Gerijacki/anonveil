//! Error type for the privileged execution layer.

#[derive(Debug, thiserror::Error)]
pub enum PrivError {
    #[error("this command requires root — re-run with sudo")]
    NotRoot,

    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Core(#[from] anonveil_core::CoreError),

    #[error("required external command not found on PATH: {0} (is it installed?)")]
    CommandNotFound(String),

    #[error("`{command}` exited with status {status}: {stderr}")]
    CommandFailed {
        command: String,
        status: String,
        stderr: String,
    },

    #[error(
        "could not resolve the `tor` system user — is the `tor` package installed? \
         (tried `tor` and `debian-tor`)"
    )]
    TorUserNotFound,

    #[error(
        "AnonVeil's nftables table already exists — a previous `stop` may not have \
             completed. Run `anonveil stop --force` before `start`, or `anonveil panic` if \
             something looks wrong right now."
    )]
    TableAlreadyExists,

    #[error("timed out waiting for Tor to finish bootstrapping ({0}% after {1}s)")]
    BootstrapTimeout(u8, u64),

    #[error("no active AnonVeil session found in {0}")]
    NoActiveState(String),

    #[error(
        "another `anonveil` operation is already in progress (start/stop/restart/panic all \
         hold an exclusive lock while they run) — wait for it to finish and try again"
    )]
    AnotherOperationInProgress,
}

pub type PrivResult<T> = Result<T, PrivError>;
