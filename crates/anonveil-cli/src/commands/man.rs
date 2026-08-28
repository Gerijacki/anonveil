//! `anonveil man` — print the man page (troff source) to stdout.
//! Packaging generates this at build time and installs it to
//! `/usr/share/man/man1/anonveil.1` — see `commands::completions` for why
//! this is runtime-generated rather than a `build.rs`.

use anyhow::Result;
use clap::Command;
use clap_mangen::Man;

pub fn run(command: Command) -> Result<()> {
    let man = Man::new(command);
    man.render(&mut std::io::stdout())?;
    Ok(())
}
