//! The ASCII banner shown on `--help`, on `status`, and as the TUI
//! splash. Kept intentionally restrained: readable, high-contrast, no
//! gimmicks — the "professional" half of "cyberpunk but professional".

const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[38;5;46m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";

pub fn print_banner() {
    println!("{GREEN}{BOLD}   ▄▀█ █▄░█ █▀█ █▄░█ █░█ █▀▀ █ █░░{RESET}");
    println!("{GREEN}{BOLD}   █▀█ █░▀█ █▄█ █░▀█ ▀▄▀ ██▄ █ █▄▄{RESET}");
    println!("{DIM}   system-wide Tor kill switch — Arch & Debian{RESET}");
    println!();
}
