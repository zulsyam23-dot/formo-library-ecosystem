use std::env;
use std::io::{self, IsTerminal};

pub fn print_error(message: &str) {
    eprintln!(
        "{}: {message}",
        style("error", "31", stderr_supports_color())
    );
}

pub fn print_warn(message: &str) {
    eprintln!(
        "{}: {message}",
        style("warn", "33", stderr_supports_color())
    );
}

pub fn print_watch(message: &str) {
    println!("{}", style(message, "36", stdout_supports_color()));
}

pub fn print_ok(message: &str) {
    println!("{}", style(message, "32", stdout_supports_color()));
}

fn style(text: &str, color_code: &str, enabled: bool) -> String {
    if enabled {
        format!("\x1b[{color_code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

fn stderr_supports_color() -> bool {
    color_enabled() && io::stderr().is_terminal()
}

fn stdout_supports_color() -> bool {
    color_enabled() && io::stdout().is_terminal()
}

fn color_enabled() -> bool {
    if env::var_os("NO_COLOR").is_some() {
        return false;
    }

    if matches!(env::var("FORCE_COLOR").ok().as_deref(), Some("1")) {
        return true;
    }

    true
}
