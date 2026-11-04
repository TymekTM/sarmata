//! Skryba — kompilator języka Sarmata.

mod ast;
mod codegen;
mod error;
mod gramatyka;
mod lexer;
mod parser;
mod powolania;
mod text;
mod words;

use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

const POMOC: &str = "\
Sarmata — ięzyk progrämowániá ſtárą polſzczyzną piſány.

Vżycie:
  sarmata <piſmo.sar> [-o <wyiśćie>]   ſkompiluy do pliku wykonywálnego
  sarmata uruchom <piſmo.sar> [ſłowá]  ſkompiluy i od rázu vruchom
  sarmata przekład <piſmo.sar>         pokaż przekład ná C, zdánie po zdániu
  sarmata c <piſmo.sar>                pokaż cáły C, z biegiem włącznie
  sarmata rozmowa                      rozmawiay ze ſkrybą wierſz po wierſzu
  sarmata gramatyka                    wypiſz gramátykę TextMate dla edytorów
  sarmata pomoc                        to obwieſzczenie

Kompilátor C bierze ſię ze zmienney CC (domyślnie «cc»).
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(msg) => {
            eprint!("{msg}");
            if !msg.ends_with('\n') {
                eprintln!();
            }
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let src_arg = || {
        args.get(1)
            .map(Path::new)
            .ok_or("Którego piſmá? Podaj ścieżkę.")
    };
    match args.first().map(String::as_str) {
        None | Some("pomoc" | "-h" | "--help" | "--pomoc") => {
            print!("{POMOC}");
            Ok(ExitCode::SUCCESS)
        }
        Some("c") => {
            print!("{}", translate_file(src_arg()?, false)?);
            Ok(ExitCode::SUCCESS)
        }
        Some("przekład" | "przeklad" | "przekłád") => {
            let c = translate_file(src_arg()?, true)?;
            let (_, program) = c
                .split_once(codegen::PROGRAM_MARK)
                .expect("znacznik piſmá w przekłádzie");
            print!("{}", program.trim_start());
            Ok(ExitCode::SUCCESS)
        }
        Some("gramatyka" | "gramatyka-vscode") => {
            print!("{}", gramatyka::tm_language());
            Ok(ExitCode::SUCCESS)
        }
        Some("rozmowa") => rozmowa(),
        Some("uruchom") => {
            let c = translate_file(src_arg()?, false)?;
            let exe = temp_path("");
            cc(&c, &exe, "-O2")?;
            let status = Command::new(&exe)
                .args(&args[2..])
                .status()
                .map_err(|e| format!("Nie vdáło ſię vruchomić prográmu: {e}"))?;
            let _ = std::fs::remove_file(&exe);
            Ok(ExitCode::from(status.code().unwrap_or(1) as u8))
        }
        Some(src) => {
            let out = match args.iter().position(|a| a == "-o") {
                Some(i) => PathBuf::from(
                    args.get(i + 1)
                        .ok_or("Po «-o» podaj miano pliku wyiśćiowego.")?,
                ),
                None => PathBuf::from(Path::new(src).file_stem().ok_or("Dźiwne miano piſmá.")?),
            };
            cc(&translate_file(Path::new(src), false)?, &out, "-O2")?;
            eprintln!("Skrybá ſkończył. Prográm ſpiſány do „{}”.", out.display());
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn translate_file(path: &Path, przeklad: bool) -> Result<String, String> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| format!("Piſmá „{}” odczytáć nie ſpoſób: {e}", path.display()))?;
    translate(path, path.display().to_string(), src, przeklad)
}

/// `przeklad` dokłada nad każdym zdaniem w C to zdanie z pisma.
fn translate(path: &Path, shown: String, src: String, przeklad: bool) -> Result<String, String> {
    let (toks, pisma) = powolania::zbierz(path, shown, src)?;
    let prog = parser::Parser::new(toks)
        .parse_program()
        .map_err(|e| error::render(&e, &pisma))?;
    let (miana, zrodla): (Vec<String>, Vec<String>) =
        pisma.into_iter().map(|p| (p.shown, p.src)).unzip();
    Ok(codegen::generate(
        &prog,
        &miana,
        przeklad.then_some(zrodla.as_slice()),
    ))
}

fn temp_path(ext: &str) -> PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("sarmata_{}_{n}{ext}", std::process::id()))
}

/// Oddaje C kowalowi (kompilatorowi C) i każe wykuć program `out`.
fn cc(c: &str, out: &Path, opt: &str) -> Result<(), String> {
    let cfile = temp_path(".c");
    std::fs::write(&cfile, c).map_err(|e| format!("Nie vdáło ſię zápiſáć brudnopiſu: {e}"))?;
    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
    let result = Command::new(&cc)
        .args(["-std=gnu11", opt, "-w", "-o"])
        .arg(out)
        .arg(&cfile)
        .arg("-lm")
        .output();
    let _ = std::fs::remove_file(&cfile);
    let output = result.map_err(|e| {
        format!("Kowálá (kompilátorá C, «{cc}») nie názleźiono: {e}\nZáinſtáluy gcc álbo clang.")
    })?;
    if !output.status.success() {
        return Err(format!(
            "Kowál (kompilátor C) odmówił roboty. To grzech ſkryby, nie Wáſzmośći — zgłoś go, proſzę.\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

/// Rozmowa ze skrybą: każde zdanie dopisuje się do pisma, całość kompiluje się
/// od nowa i pokazuje się tylko to, co przybyło. Zdanie, które zgrzeszyło, przepada.
fn rozmowa() -> Result<ExitCode, String> {
    let tty = std::io::stdin().is_terminal();
    if tty {
        eprintln!(
            "Skrybá ſłucha. Piſz zdánia; blok po dwukropku kończy pusty wierſz. «Amen.» kończy rozmowę."
        );
    }
    let cwd = std::env::current_dir()
        .unwrap_or_default()
        .join("rozmowa.sar");
    let exe = temp_path("");
    let mut history = String::new();
    let mut shown = 0usize;
    let mut chunk = String::new();
    let mut block = false;
    let mut lines = std::io::stdin().lock().lines();
    loop {
        if tty {
            eprint!("{}", if chunk.is_empty() { "✠ " } else { "… " });
            let _ = std::io::stderr().flush();
        }
        let Some(Ok(line)) = lines.next() else { break };
        let t = line.trim();
        if chunk.is_empty() {
            if t.is_empty() {
                continue;
            }
            if words::keyword(&text::normalize(t.trim_end_matches(['.', '!'])))
                == Some(words::Kw::Amen)
            {
                break;
            }
        }
        if !t.is_empty() {
            chunk.push_str(&line);
            chunk.push('\n');
            if t.ends_with(':') {
                block = true;
            }
            if block {
                continue;
            }
        }
        block = false;
        if chunk.trim().is_empty() {
            chunk.clear();
            continue;
        }
        let pismo = format!("{history}{chunk}\nAmen.\n");
        let built =
            translate(&cwd, "rozmowa".into(), pismo, false).and_then(|c| cc(&c, &exe, "-O0"));
        if let Err(msg) = built {
            eprint!("{msg}");
            if !msg.ends_with('\n') {
                eprintln!();
            }
            chunk.clear();
            continue;
        }
        let out = Command::new(&exe)
            .stdin(Stdio::null())
            .output()
            .map_err(|e| format!("Nie vdáło ſię vruchomić prográmu: {e}"))?;
        let stdout = std::io::stdout();
        let mut so = stdout.lock();
        let _ = so.write_all(out.stdout.get(shown..).unwrap_or(&[]));
        let _ = so.flush();
        if out.status.success() {
            history.push_str(&chunk);
            shown = out.stdout.len();
        } else {
            let _ = std::io::stderr().write_all(&out.stderr);
            if tty {
                eprintln!("(To zdánie przepádá — piſmo zoſtáie, iákie było.)");
            }
        }
        chunk.clear();
    }
    let _ = std::fs::remove_file(&exe);
    Ok(ExitCode::SUCCESS)
}
