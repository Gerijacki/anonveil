//! MAC address randomization (`anonveil mac randomize` / `mac restore`).
//!
//! Off by default (see `AnonveilConfig::mac`). Uses `ip link` (iproute2,
//! present by default on both target distros) rather than depending on
//! `macchanger` for one small piece of functionality.

use std::fs;
use std::path::Path;

use rand::RngCore;

use crate::error::{PrivError, PrivResult};
use crate::exec::run;

const BACKUP_FILE: &str = "/var/lib/anonveil/mac-backup.txt";

/// Find the interface the default route goes through, for the common
/// case of `anonveil mac randomize` with no `--interface` given.
pub fn default_interface() -> PrivResult<String> {
    let output = run("ip", &["route", "show", "default"])?;
    output
        .split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .find(|w| w[0] == "dev")
        .map(|w| w[1].to_string())
        .ok_or_else(|| {
            PrivError::Core(anonveil_core::CoreError::InvalidFirewallConfig(
                "could not determine the default network interface; pass --interface explicitly"
                    .to_string(),
            ))
        })
}

fn current_mac(iface: &str) -> PrivResult<String> {
    let output = run("ip", &["-o", "link", "show", iface])?;
    output
        .split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .find(|w| w[0] == "link/ether")
        .map(|w| w[1].to_string())
        .ok_or_else(|| {
            PrivError::Core(anonveil_core::CoreError::InvalidFirewallConfig(format!(
                "could not read the current MAC address of {iface}"
            )))
        })
}

/// A random, valid unicast, locally-administered MAC address (the same
/// convention `macchanger -r` and NetworkManager's MAC-randomization use).
fn random_mac() -> String {
    let mut bytes = [0u8; 6];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes[0] = (bytes[0] & 0xFE) | 0x02; // unicast, locally administered
    bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(":")
}

fn set_mac(iface: &str, mac: &str) -> PrivResult<()> {
    run("ip", &["link", "set", "dev", iface, "down"])?;
    let result = run("ip", &["link", "set", "dev", iface, "address", mac]);
    // Always try to bring the interface back up, even if setting the
    // address failed, so a failure here doesn't leave networking dead.
    let up_result = run("ip", &["link", "set", "dev", iface, "up"]);
    result?;
    up_result?;
    Ok(())
}

fn read_backup_map() -> PrivResult<Vec<(String, String)>> {
    if !Path::new(BACKUP_FILE).exists() {
        return Ok(Vec::new());
    }
    let contents = fs::read_to_string(BACKUP_FILE)?;
    Ok(contents
        .lines()
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect())
}

fn write_backup_map(entries: &[(String, String)]) -> PrivResult<()> {
    if let Some(parent) = Path::new(BACKUP_FILE).parent() {
        fs::create_dir_all(parent)?;
    }
    let text = entries
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(BACKUP_FILE, text)?;
    Ok(())
}

/// Randomize `iface`'s MAC address, remembering the original (only the
/// *first* time — calling this repeatedly without an intervening
/// `restore` must not overwrite the real original with an
/// already-randomized value).
pub fn randomize(iface: &str) -> PrivResult<String> {
    let mut backups = read_backup_map()?;
    if !backups.iter().any(|(k, _)| k == iface) {
        let original = current_mac(iface)?;
        backups.push((iface.to_string(), original));
        write_backup_map(&backups)?;
    }
    let new_mac = random_mac();
    set_mac(iface, &new_mac)?;
    Ok(new_mac)
}

/// Restore `iface`'s original MAC address, if AnonVeil has one recorded.
pub fn restore(iface: &str) -> PrivResult<()> {
    let mut backups = read_backup_map()?;
    let Some(pos) = backups.iter().position(|(k, _)| k == iface) else {
        return Ok(()); // nothing to restore — not an error, just a no-op
    };
    let (_, original) = backups.remove(pos);
    set_mac(iface, &original)?;
    write_backup_map(&backups)?;
    Ok(())
}
