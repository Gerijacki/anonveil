//! Locating and loading `config.toml`.

use std::path::PathBuf;
use std::str::FromStr;

use anonveil_core::config::AnonveilConfig;
use anyhow::{Context, Result};

pub const DEFAULT_CONFIG_PATH: &str = "/etc/anonveil/config.toml";

/// Load the config from `override_path` if given, else
/// [`DEFAULT_CONFIG_PATH`] if it exists, else compiled-in defaults (with
/// a note printed to stderr so the user knows nothing was customized).
pub fn load(override_path: Option<&PathBuf>) -> Result<AnonveilConfig> {
    let path = override_path
        .cloned()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH));

    if !path.exists() {
        if override_path.is_some() {
            anyhow::bail!("config file not found: {}", path.display());
        }
        eprintln!(
            "note: {} not found, using built-in defaults \
             (copy config/config.example.toml there to customize)",
            path.display()
        );
        return Ok(AnonveilConfig::default());
    }

    let contents =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    AnonveilConfig::from_str(&contents).with_context(|| format!("parsing {}", path.display()))
}
