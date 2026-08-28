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

/// Format a byte count the way `GETINFO traffic/read`/`traffic/written`
/// return it (a plain decimal string) into a human-readable size, for
/// `status`/the TUI's bandwidth counters.
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::human_bytes;

    #[test]
    fn formats_across_unit_boundaries() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(512), "512 B");
        assert_eq!(human_bytes(1024), "1.0 KB");
        assert_eq!(human_bytes(1_500_000), "1.4 MB");
        assert_eq!(human_bytes(5_000_000_000), "4.7 GB");
    }
}
