//! Grzechy (błędy) kompilacji i ich obwieszczanie.

use std::io::IsTerminal;

#[derive(Debug, Clone)]
pub struct CompileError {
    pub line: usize,
    pub col: usize,
    pub len: usize,
    pub msg: String,
}

impl CompileError {
    pub fn new(line: usize, col: usize, len: usize, msg: impl Into<String>) -> Self {
        CompileError {
            line,
            col,
            len: len.max(1),
            msg: msg.into(),
        }
    }
}

/// Numer wiersza niesie też numer pisma: `pismo * WIERSZE_PISMA + wiersz`.
/// Pismo główne ma numer zero, więc jego wiersze zostają, jakie są.
pub const WIERSZE_PISMA: usize = 1_000_000;

/// Pismo, z którego skryba czyta: miano do obwieszczeń i treść.
pub struct Pismo {
    pub shown: String,
    pub src: String,
}

/// Składa obwieszczenie grzechu z fragmentem pisma i daszkiem pod winnym słowem.
pub fn render(err: &CompileError, pisma: &[Pismo]) -> String {
    let (src, path) = match pisma.get(err.line / WIERSZE_PISMA) {
        Some(p) => (p.src.as_str(), p.shown.as_str()),
        None => ("", "?"),
    };
    let line = err.line % WIERSZE_PISMA;
    let color = std::io::stderr().is_terminal();
    let (red, bold, dim, reset) = if color {
        ("\x1b[1;31m", "\x1b[1m", "\x1b[2m", "\x1b[0m")
    } else {
        ("", "", "", "")
    };
    let mut out = format!(
        "{red}Grzech śmiertelny{reset} w „{path}”, wierſz {}:\n",
        line
    );
    if let Some(text) = src.lines().nth(line.saturating_sub(1)) {
        let num = line.to_string();
        let pad = " ".repeat(num.len());
        let caret_pad: String = text
            .chars()
            .take(err.col.saturating_sub(1))
            .map(|c| if c == '\t' { '\t' } else { ' ' })
            .collect();
        out.push_str(&format!("{dim} {num} │{reset} {text}\n"));
        out.push_str(&format!(
            "{dim} {pad} │{reset} {caret_pad}{red}{}{reset}\n",
            "^".repeat(err.len)
        ));
    }
    out.push_str(&format!("{bold}{}{reset}\n", err.msg));
    out
}
