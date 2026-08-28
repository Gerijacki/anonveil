//! `anonveil mac randomize|restore` — MAC address spoofing.

use anyhow::Result;

use crate::style;

fn resolve_interface(interface: Option<String>) -> Result<String> {
    match interface {
        Some(i) => Ok(i),
        None => Ok(anonveil_priv::mac::default_interface()?),
    }
}

pub fn randomize(interface: Option<String>) -> Result<()> {
    anonveil_priv::privilege::require_root()?;
    let iface = resolve_interface(interface)?;
    let new_mac = anonveil_priv::mac::randomize(&iface)?;
    style::ok(&format!("{iface}: MAC address randomized to {new_mac}"));
    Ok(())
}

pub fn restore(interface: Option<String>) -> Result<()> {
    anonveil_priv::privilege::require_root()?;
    let iface = resolve_interface(interface)?;
    anonveil_priv::mac::restore(&iface)?;
    style::ok(&format!(
        "{iface}: MAC address restored to its original value."
    ));
    Ok(())
}
