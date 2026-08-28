//! `anonveil` — the CLI/TUI binary.

mod banner;
mod commands;
mod config_paths;
mod logging;
mod style;
mod tui;

use std::path::PathBuf;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};

/// AnonVeil — a system-wide Tor kill switch, for Arch and Debian.
#[derive(Parser)]
#[command(name = "anonveil", version, about, long_about = None)]
struct Cli {
    /// Path to config.toml (default: /etc/anonveil/config.toml).
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Activate the kill switch: redirect all traffic through Tor.
    Start,
    /// Deactivate the kill switch and restore normal networking.
    Stop {
        /// Restore even if AnonVeil doesn't think it's active — use
        /// after a crashed/interrupted session.
        #[arg(long)]
        force: bool,
    },
    /// Reapply the kill switch and Tor configuration from scratch.
    Restart,
    /// Request a new Tor circuit/identity without touching the firewall.
    Newnym,
    /// Show current protection status.
    Status,
    /// Instantly cut all network traffic except loopback.
    Panic,
    /// Run a Tor-reachability self-test.
    Check,
    /// MAC address spoofing.
    Mac {
        #[command(subcommand)]
        action: MacAction,
    },
    /// Launch the live-status dashboard (also the default with no
    /// subcommand).
    Dashboard,
    /// Check every precondition `start` depends on at once (commands on
    /// PATH, the tor service, config validity, port availability, ...).
    Doctor,
    /// Print the exact nftables ruleset `start` would load, without
    /// touching the system or requiring root.
    AuditRuleset,
    /// Print a shell completion script to stdout.
    Completions { shell: clap_complete::Shell },
    /// Print the man page (troff source) to stdout.
    Man,
    /// Resume a session that was active before a reboot, if any. Used by
    /// the optional `anonveil.service` boot unit — not intended to be run
    /// by hand (use `start` for that). A no-op if AnonVeil wasn't active
    /// when the system last shut down.
    #[command(hide = true)]
    BootResume,
}

#[derive(Subcommand)]
enum MacAction {
    /// Randomize the MAC address of a network interface.
    Randomize {
        /// Interface to randomize (default: the default-route interface).
        #[arg(long)]
        interface: Option<String>,
    },
    /// Restore a network interface's original MAC address.
    Restore {
        /// Interface to restore (default: the default-route interface).
        #[arg(long)]
        interface: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // These two are pure stdout generators with no config/logging/system
    // dependency at all — handled before any of that is set up.
    match &cli.command {
        Some(Commands::Completions { shell }) => {
            commands::completions::run(*shell, Cli::command());
            return Ok(());
        }
        Some(Commands::Man) => return commands::man::run(Cli::command()),
        _ => {}
    }

    // `doctor` deliberately loads config itself, leniently — a broken
    // config.toml must show up as one of its findings, not a hard crash
    // before doctor gets a chance to report anything at all.
    if matches!(cli.command, Some(Commands::Doctor)) {
        let result = commands::doctor::run(cli.config.as_ref());
        if let Err(e) = &result {
            style::error(&format!("{e}"));
        }
        return result;
    }

    let config = config_paths::load(cli.config.as_ref())?;

    let _log_guard = logging::init(&config.logging.level);

    let result = match cli.command {
        Some(Commands::Start) => {
            banner::print_banner();
            commands::start::run(&config, false).await
        }
        Some(Commands::BootResume) => commands::start::run(&config, true).await,
        Some(Commands::Stop { force }) => commands::stop::run(force),
        Some(Commands::Restart) => commands::restart::run(&config).await,
        Some(Commands::Newnym) => commands::newnym::run(&config).await,
        Some(Commands::Status) => commands::status::run(&config).await,
        Some(Commands::Panic) => commands::panic::run(),
        Some(Commands::Check) => commands::check::run().await,
        Some(Commands::Mac { action }) => match action {
            MacAction::Randomize { interface } => commands::mac::randomize(interface),
            MacAction::Restore { interface } => commands::mac::restore(interface),
        },
        Some(Commands::AuditRuleset) => commands::audit_ruleset::run(&config),
        Some(Commands::Doctor) | Some(Commands::Completions { .. }) | Some(Commands::Man) => {
            unreachable!("handled above before config/logging were set up")
        }
        Some(Commands::Dashboard) | None => tui::run(&config).await,
    };

    if let Err(e) = &result {
        style::error(&format!("{e}"));
    }
    result
}
