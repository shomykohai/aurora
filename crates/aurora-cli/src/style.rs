use std::fmt::Display;

use anstyle::{AnsiColor, Color, Style};

pub const TITLE: Style = Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Green)));
pub const LABEL: Style = Style::new().bold();
pub const VALUE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan)));
pub const DETAIL: Style = Style::new();
pub const WARN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow)));
pub const ERROR: Style = Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Red)));

pub fn title(text: impl Display) {
    anstream::println!("{TITLE}{text}{TITLE:#}");
}

pub fn field(label: &str, value: impl Display) {
    anstream::println!("  {LABEL}{label:<13}{LABEL:#} {VALUE}{value}{VALUE:#}");
}

pub fn detail(label: &str, value: impl Display, detail: impl Display) {
    anstream::println!(
        "  {LABEL}{label:<13}{LABEL:#} {VALUE}{value}{VALUE:#} {DETAIL}{detail}{DETAIL:#}"
    );
}

pub fn warn(text: impl Display) -> String {
    format!("{WARN}{text}{WARN:#}")
}

pub fn update(label: &str, now: impl Display, before: impl Display) {
    let (now, before) = (now.to_string(), before.to_string());

    if now == before {
        field(label, now);
    } else {
        detail(label, now, format_args!("(was: {before})"));
    }
}

pub const fn switch(on: bool) -> &'static str {
    if on { "enabled" } else { "disabled" }
}
