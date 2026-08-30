//! The dashboard's single live-status screen.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::app::App;
use super::theme;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(6),
            Constraint::Length(1),
        ])
        .split(area);

    draw_title(frame, chunks[0]);
    draw_status(frame, chunks[1], app);
    draw_footer(frame, chunks[2]);
}

fn draw_title(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(Span::styled(
            "  ▄▀█ █▄░█ █▀█ █▄░█ █░█ █▀▀ █ █░░",
            theme::title(),
        )),
        Line::from(Span::styled(
            "  █▀█ █░▀█ █▄█ █░▀█ ▀▄▀ ██▄ █ █▄▄",
            theme::title(),
        )),
        Line::from(Span::styled(
            "  system-wide Tor kill switch — Arch & Debian",
            theme::muted(),
        )),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}

fn draw_status(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines = Vec::new();

    if app.state.panic_active {
        lines.push(Line::from(Span::styled(
            "⚠ PANIC MODE — all traffic blocked except loopback",
            theme::error(),
        )));
        lines.push(Line::from(""));
    }

    if app.state.active {
        lines.push(Line::from(vec![
            Span::styled("● ", theme::ok()),
            Span::styled("ACTIVE", theme::ok()),
            Span::styled("  — traffic is routed through Tor", theme::muted()),
        ]));
        if let Some(since) = &app.state.activated_at {
            lines.push(Line::from(Span::styled(
                format!("  since {since}"),
                theme::muted(),
            )));
        }
        if !app.kill_switch_actually_loaded {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "⚠ STATE MISMATCH: kill switch is NOT actually loaded (reboot?) — traffic is",
                theme::error(),
            )));
            lines.push(Line::from(Span::styled(
                "  UNPROTECTED right now. Run `anonveil start` to reapply it.",
                theme::error(),
            )));
        }
        lines.push(Line::from(""));
        lines.push(status_line("tor bootstrapped", app.tor_bootstrapped));
        lines.push(status_line("circuit established", app.circuit_established));
        if let Some((read, written)) = app.traffic {
            lines.push(Line::from(vec![
                Span::styled("  traffic: ", theme::muted()),
                Span::styled(
                    format!(
                        "{} down / {} up",
                        crate::style::human_bytes(read),
                        crate::style::human_bytes(written)
                    ),
                    theme::muted(),
                ),
            ]));
        }
        if let Some(when) = &app.state.last_ip_rotation {
            lines.push(Line::from(Span::styled(
                format!("  last IP rotation: {when}"),
                theme::muted(),
            )));
        }
        if let Some(when) = &app.state.last_mac_rotation {
            lines.push(Line::from(Span::styled(
                format!("  last MAC rotation: {when}"),
                theme::muted(),
            )));
        }
        if let Some(err) = &app.control_error {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("could not query tor control port: {err}"),
                theme::warn(),
            )));
        }
    } else {
        lines.push(Line::from(vec![
            Span::styled("○ ", theme::muted()),
            Span::styled("INACTIVE", theme::muted()),
            Span::styled(
                "  — run `anonveil start` (as root) to activate",
                theme::muted(),
            ),
        ]));
    }

    let block = Block::default()
        .title(" status ")
        .borders(Borders::ALL)
        .border_style(theme::border());
    frame.render_widget(
        Paragraph::new(lines).style(theme::body()).block(block),
        area,
    );
}

fn status_line(label: &str, value: Option<bool>) -> Line<'static> {
    let (text, style) = match value {
        Some(true) => ("yes".to_string(), theme::ok()),
        Some(false) => ("no".to_string(), theme::warn()),
        None => ("unknown".to_string(), theme::muted()),
    };
    Line::from(vec![
        Span::styled(format!("  {label}: "), theme::muted()),
        Span::styled(text, style),
    ])
}

fn draw_footer(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(" [q] ", theme::title()),
        Span::styled("quit   ", theme::muted()),
        Span::styled("[r] ", theme::title()),
        Span::styled("refresh   ", theme::muted()),
        Span::styled("[s] ", theme::title()),
        Span::styled("start/stop   ", theme::muted()),
        Span::styled("[p] ", theme::title()),
        Span::styled("panic   ", theme::muted()),
        Span::styled("[n] ", theme::title()),
        Span::styled("newnym", theme::muted()),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}
