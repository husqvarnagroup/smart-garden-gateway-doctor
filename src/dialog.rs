use colored::Colorize;
use std::io::Write;

pub fn inform(msg: &str, mut out: impl Write) {
    writeln!(out, "{} {}", "!".red().bold(), msg).expect("Failed to inform");
}
