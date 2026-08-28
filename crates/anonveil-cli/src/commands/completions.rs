//! `anonveil completions <shell>` — print a shell completion script to
//! stdout. Packaging (`packaging/arch/PKGBUILD`,
//! `.github/workflows/release.yml`'s `build-deb` job) runs this at build
//! time and installs the result to the shell's standard completion
//! directory; running it by hand and redirecting to a file works too.

use clap::Command;
use clap_complete::{generate, Shell};

pub fn run(shell: Shell, mut command: Command) {
    let name = command.get_name().to_string();
    generate(shell, &mut command, name, &mut std::io::stdout());
}
