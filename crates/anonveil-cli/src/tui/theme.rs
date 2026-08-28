//! The dashboard's color palette: neon-on-black, but chosen for
//! contrast/readability first — cyberpunk flavor, not a gimmick.

use ratatui::style::{Color, Modifier, Style};

pub const BG: Color = Color::Rgb(10, 10, 12);
pub const FG: Color = Color::Rgb(214, 255, 224);
pub const ACCENT: Color = Color::Rgb(0, 255, 156); // high-contrast neon green
pub const ACCENT_DIM: Color = Color::Rgb(0, 140, 90);
pub const WARN: Color = Color::Rgb(255, 209, 84);
pub const ERROR: Color = Color::Rgb(255, 90, 90);
pub const MUTED: Color = Color::Rgb(120, 130, 128);

pub fn title() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn border() -> Style {
    Style::default().fg(ACCENT_DIM)
}

pub fn ok() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn warn() -> Style {
    Style::default().fg(WARN).add_modifier(Modifier::BOLD)
}

pub fn error() -> Style {
    Style::default().fg(ERROR).add_modifier(Modifier::BOLD)
}

pub fn muted() -> Style {
    Style::default().fg(MUTED)
}

pub fn body() -> Style {
    Style::default().fg(FG).bg(BG)
}
