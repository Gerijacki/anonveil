//! Small, consistent CLI output helpers. Kept dependency-free (raw ANSI)
//! since the palette used is tiny and fixed.

const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[38;5;46m";
const CYAN: &str = "\x1b[38;5;51m";
const YELLOW: &str = "\x1b[38;5;220m";
const RED: &str = "\x1b[38;5;196m";
const DIM: &str = "\x1b[2m";

pub fn step(msg: &str) {
    println!("{CYAN}::{RESET} {msg}");
}

pub fn ok(msg: &str) {
    println!("{GREEN}✔{RESET} {msg}");
}

pub fn warn(msg: &str) {
    println!("{YELLOW}⚠{RESET} {msg}");
}

pub fn error(msg: &str) {
    eprintln!("{RED}✘{RESET} {msg}");
}

pub fn dim(msg: &str) {
    println!("{DIM}{msg}{RESET}");
}
