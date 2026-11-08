//! Lekser: tnie pismo na słowa, liczby, cytaty i znaki, a wcięcia zamienia
//! na otwarcia i zamknięcia bloków.
//!
//! Wcięcie liczy się tylko na początku zdania (po `.`, `!`, `?`, `;` lub `:`),
//! więc jedno zdanie wolno rozpisać na wiele wierszy jak strofę.

use crate::error::CompileError;
use crate::text::{is_mark, normalize};
use crate::words::{COMMENT_STARTERS, FILLERS, Kw, keyword};

#[derive(Clone, Debug, PartialEq)]
pub enum StrPart {
    Lit(String),
    /// Wstawka `{…}` wewnątrz cytatu: tekst, wiersz, kolumna.
    Expr(String, usize, usize),
}

#[derive(Clone, Debug, PartialEq)]
pub enum K {
    Word,
    Int(i64),
    Float(f64),
    Str(Vec<StrPart>),
    P(&'static str),
    Indent,
    Dedent,
    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub k: K,
    pub raw: String,
    pub norm: String,
    pub kw: Option<Kw>,
    pub line: usize,
    pub col: usize,
}

impl Token {
    fn new(k: K, raw: String, line: usize, col: usize) -> Token {
        let (norm, kw) = if k == K::Word {
            let n = normalize(&raw);
            let kw = keyword(&n);
            (n, kw)
        } else {
            (String::new(), None)
        };
        Token {
            k,
            raw,
            norm,
            kw,
            line,
            col,
        }
    }

    pub fn width(&self) -> usize {
        self.raw.chars().count().max(1)
    }
}

type R<T> = Result<T, CompileError>;

const TERMINATORS: &[&str] = &[".", "!", "?", ";", ":"];
const PUNCT2: &[&str] = &["<=", ">=", "!=", "=="];
const PUNCT1: &[&str] = &[
    ".", ",", ":", ";", "!", "?", "(", ")", "+", "-", "*", "/", "=", "<", ">",
];

/// `base` to przesunięcie numerów wierszy (zob. `error::WIERSZE_PISMA`).
pub fn lex(src: &str, base: usize) -> R<Vec<Token>> {
    let src = src.strip_prefix('\u{feff}').unwrap_or(src);
    let mut out: Vec<Token> = Vec::new();
    let mut indents = vec![0usize];
    let mut boundary = true;
    let mut last_line = base + 1;
    // Przypis nie zakończony kropką ciągnie się na wiersze, co zaczynają się małą literą.
    let mut comment_open = false;

    let lines: Vec<&str> = src.lines().collect();
    let mut li = 0;
    while li < lines.len() {
        let line = lines[li];
        let lineno = base + li + 1;
        last_line = lineno;
        li += 1;
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            comment_open = false;
            continue;
        }
        let first: String = trimmed
            .chars()
            .take_while(|&c| c.is_alphabetic() || is_mark(c))
            .collect();
        let continues = comment_open && trimmed.starts_with(char::is_lowercase);
        if continues || (!first.is_empty() && COMMENT_STARTERS.contains(&normalize(&first))) {
            comment_open = !trimmed.trim_end().ends_with(['.', '!', '?']);
            continue;
        }
        comment_open = false;
        if boundary {
            let width = indent_width(line);
            let top = *indents.last().unwrap();
            if width > top {
                indents.push(width);
                out.push(Token::new(K::Indent, String::new(), lineno, 1));
            } else if width < top {
                while width < *indents.last().unwrap() {
                    indents.pop();
                    out.push(Token::new(K::Dedent, String::new(), lineno, 1));
                }
                if width != *indents.last().unwrap() {
                    return Err(CompileError::new(
                        lineno,
                        1,
                        width.max(1),
                        "Wcięćie krzywe iáko płot u ſąśiádá — nie páſuie do żadnego z wyżſzych bloków.",
                    ));
                }
            }
        }
        // Cytat nie zamknięty u końca wiersza ciągnie się w następnych, aż do pustego wiersza.
        // `szwy` pamiętają, gdzie w sklejonym tekście zaczyna się każdy dalszy wiersz (znak,
        // numer wiersza, wcięcie), by słowa i grzechy za szwem wskazywały prawdziwe miejsce.
        let mut text = line.to_string();
        let mut szwy: Vec<(usize, usize, usize)> = Vec::new();
        while quote_open(&text) && li < lines.len() && !lines[li].trim().is_empty() {
            text.push(' ');
            let next = lines[li];
            let indent = next.chars().count() - next.trim_start().chars().count();
            szwy.push((text.chars().count(), base + li + 1, indent));
            text.push_str(next.trim());
            li += 1;
            last_line = base + li;
        }
        let place = |col: usize| match szwy.iter().rev().find(|s| col > s.0) {
            Some(&(start, l, indent)) => (l, col - start + indent),
            None => (lineno, col),
        };
        let before = out.len();
        let chars: Vec<char> = text.chars().collect();
        lex_chars(&chars, lineno, 0, &mut out).map_err(|mut e| {
            (e.line, e.col) = place(e.col);
            e
        })?;
        if out.len() > before {
            // „…wynosi pięć.” — kropka w cudzysłowie u końca wiersza też kończy zdanie.
            if let K::Str(parts) = &out.last().unwrap().k
                && let Some(StrPart::Lit(s)) = parts.last()
                && s.trim_end().ends_with(['.', '!', '?'])
            {
                out.push(Token::new(
                    K::P("."),
                    String::new(),
                    lineno,
                    chars.len() + 1,
                ));
            }
            boundary = matches!(&out.last().unwrap().k, K::P(p) if TERMINATORS.contains(p));
        }
        if !szwy.is_empty() {
            for t in &mut out[before..] {
                (t.line, t.col) = place(t.col);
                if let K::Str(parts) = &mut t.k {
                    for p in parts {
                        if let StrPart::Expr(_, l, c) = p {
                            (*l, *c) = place(*c);
                        }
                    }
                }
            }
        }
    }
    while indents.len() > 1 {
        indents.pop();
        out.push(Token::new(K::Dedent, String::new(), last_line + 1, 1));
    }
    out.push(Token::new(K::Eof, String::new(), last_line + 1, 1));
    Ok(drop_phrase_fillers(out))
}

/// Lekser dla wstawki `{…}` w cytacie: bez wcięć, zakończony `Eof`.
pub fn lex_fragment(text: &str, line: usize, col: usize) -> R<Vec<Token>> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    lex_chars(&chars, line, col.saturating_sub(1), &mut out)?;
    out.push(Token::new(K::Eof, String::new(), line, col + chars.len()));
    Ok(drop_phrase_fillers(out))
}

fn closers(open: char) -> Option<&'static [char]> {
    match open {
        '„' => Some(&['”', '“', '"']),
        '“' => Some(&['”', '"']),
        '«' => Some(&['»']),
        '"' => Some(&['"']),
        _ => None,
    }
}

/// Czy wiersz kończy się wewnątrz cytatu.
fn quote_open(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let mut inside: Option<&[char]> = None;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match inside {
            None => inside = closers(c),
            Some(_) if c == '\\' => i += 1,
            Some(_) if c == '{' => match interp_end(&chars, i + 1) {
                Some(j) => i = j,
                None => return true,
            },
            Some(cl) if cl.contains(&c) => inside = None,
            Some(_) => {}
        }
        i += 1;
    }
    inside.is_some()
}

/// Miejsce `}`, co zamyka wstawkę zaczętą tuż przed `from`. Wstawka może mieć własne cytaty
/// („… {złączenie x przez „, ”} …”), a w nich i klamrę: „{złączenie x przez „}”}”.
fn interp_end(chars: &[char], from: usize) -> Option<usize> {
    let mut inner: Option<&[char]> = None;
    let mut j = from;
    while j < chars.len() {
        let d = chars[j];
        match inner {
            None if d == '}' => return Some(j),
            None => inner = closers(d),
            Some(_) if d == '\\' => j += 1,
            Some(cl) if cl.contains(&d) => inner = None,
            Some(_) => {}
        }
        j += 1;
    }
    None
}

fn indent_width(line: &str) -> usize {
    let mut w = 0;
    for c in line.chars() {
        match c {
            ' ' => w += 1,
            '\t' => w += 4,
            _ => break,
        }
    }
    w
}

/// „co następuje” i „jako następuje” to ozdobniki dwusłowne.
fn drop_phrase_fillers(toks: Vec<Token>) -> Vec<Token> {
    let mut out: Vec<Token> = Vec::with_capacity(toks.len());
    for t in toks {
        if t.k == K::Word && t.norm == "nastepuie" {
            if let Some(prev) = out.last()
                && prev.k == K::Word
                && (prev.norm == "co" || prev.norm == "iako")
            {
                out.pop();
            }
            continue;
        }
        out.push(t);
    }
    out
}

fn lex_chars(chars: &[char], line: usize, col_off: usize, out: &mut Vec<Token>) -> R<()> {
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let col = col_off + i + 1;
        if c.is_whitespace() || matches!(c, '—' | '–' | '…') {
            i += 1;
            continue;
        }
        // „_miano” też wchodzi tu, by dostać upomnienie o podkreślniku, nie o obcym znaku.
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len()
                && (chars[i].is_alphanumeric() || chars[i] == '_' || is_mark(chars[i]))
            {
                i += 1;
            }
            let raw: String = chars[start..i].iter().collect();
            if raw.contains('_') {
                return Err(CompileError::new(
                    line,
                    col,
                    raw.chars().count(),
                    format!(
                        "Podkreślnik w „{raw}” to wynalazek bezbożny. Pisz miano osobnymi słowami: «{}» — wtedy i odmiana zadziała.",
                        raw.split('_')
                            .filter(|s| !s.is_empty())
                            .collect::<Vec<_>>()
                            .join(" ")
                    ),
                ));
            }
            if !FILLERS.contains(&normalize(&raw)) {
                out.push(Token::new(K::Word, raw, line, col));
            }
            continue;
        }
        if c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let mut is_float = false;
            if i + 1 < chars.len()
                && (chars[i] == '.' || chars[i] == ',')
                && chars[i + 1].is_ascii_digit()
            {
                is_float = true;
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let raw: String = chars[start..i].iter().collect();
            let too_big = || {
                CompileError::new(
                    line,
                    col,
                    raw.chars().count(),
                    "Liczbá to większa niż wſzytkie ſkárby Rzeczypoſpolitey.",
                )
            };
            let k = if is_float {
                // Ułamek ponad miarę f64 Rust czyta jako nieskończoność, a tej C nie zapisze.
                match raw.replace(',', ".").parse::<f64>() {
                    Ok(f) if f.is_finite() => K::Float(f),
                    _ => return Err(too_big()),
                }
            } else {
                match raw.parse::<i64>() {
                    Ok(n) => K::Int(n),
                    Err(_) => return Err(too_big()),
                }
            };
            out.push(Token::new(k, raw, line, col));
            continue;
        }
        if matches!(c, '„' | '"' | '“' | '«') {
            let (tok, next) = lex_string(chars, i, line, col_off)?;
            out.push(tok);
            i = next;
            continue;
        }
        let rest: String = chars[i..chars.len().min(i + 2)].iter().collect();
        if let Some(p) = PUNCT2.iter().find(|p| rest.starts_with(**p)) {
            out.push(Token::new(K::P(p), p.to_string(), line, col));
            i += 2;
            continue;
        }
        if let Some(p) = PUNCT1.iter().find(|p| p.starts_with(c)) {
            out.push(Token::new(K::P(p), p.to_string(), line, col));
            i += 1;
            continue;
        }
        return Err(CompileError::new(
            line,
            col,
            1,
            format!("Znák „{c}” obcy ieſt ſkrybie i w piſmie ſię nie godzi."),
        ));
    }
    Ok(())
}

fn lex_string(chars: &[char], start: usize, line: usize, col_off: usize) -> R<(Token, usize)> {
    let closers = closers(chars[start]).unwrap();
    let mut parts = Vec::new();
    let mut buf = String::new();
    let mut i = start + 1;
    loop {
        let Some(&c) = chars.get(i) else {
            return Err(CompileError::new(
                line,
                col_off + start + 1,
                1,
                "Cytát nie zámknięty — brák cudzyſłowu zámykáiącego przed puſtym wierſzem.",
            ));
        };
        if closers.contains(&c) {
            i += 1;
            break;
        }
        match c {
            '\\' => {
                let next = chars.get(i + 1).copied().unwrap_or('\\');
                buf.push(match next {
                    'n' => '\n',
                    't' => '\t',
                    other => other,
                });
                i += 2;
            }
            '{' => {
                if !buf.is_empty() {
                    parts.push(StrPart::Lit(std::mem::take(&mut buf)));
                }
                let inner_start = i + 1;
                let Some(j) = interp_end(chars, inner_start) else {
                    return Err(CompileError::new(
                        line,
                        col_off + i + 1,
                        1,
                        "Wſtáwká „{” otwártá, á „}” iey nie zámyka.",
                    ));
                };
                let text: String = chars[inner_start..j].iter().collect();
                parts.push(StrPart::Expr(text, line, col_off + inner_start + 1));
                i = j + 1;
            }
            _ => {
                buf.push(c);
                i += 1;
            }
        }
    }
    if !buf.is_empty() || parts.is_empty() {
        parts.push(StrPart::Lit(buf));
    }
    let raw: String = chars[start..i].iter().collect();
    Ok((Token::new(K::Str(parts), raw, line, col_off + start + 1), i))
}
