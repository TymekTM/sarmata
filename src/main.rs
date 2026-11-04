//! Skryba — kompilator języka Sarmata.

mod ast;
mod codegen;
mod error;
mod lexer;
mod parser;
mod powolania;
mod text;
mod words;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const POMOC: &str = "\
Sarmata — ięzyk progrämowániá ſtárą polſzczyzną piſány.

Vżycie:
  sarmata <piſmo.sar> [-o <wyiśćie>]   ſkompiluy do pliku wykonywálnego
  sarmata uruchom <piſmo.sar>          ſkompiluy i od rázu vruchom
  sarmata c <piſmo.sar>                pokaż, co ſkrybá piſze w ięzyku C
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
    match args.first().map(String::as_str) {
        None | Some("pomoc" | "-h" | "--help" | "--pomoc") => {
            print!("{POMOC}");
            Ok(ExitCode::SUCCESS)
        }
        Some("c") => {
            let src = args.get(1).ok_or("Którego piſmá? Podaj ścieżkę.")?;
            print!("{}", translate(Path::new(src))?);
            Ok(ExitCode::SUCCESS)
        }
        Some("uruchom") => {
            let src = args.get(1).ok_or("Którego piſmá? Podaj ścieżkę.")?;
            let exe = std::env::temp_dir().join(format!("sarmata_{}", std::process::id()));
            build(Path::new(src), &exe)?;
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
            build(Path::new(src), &out)?;
            eprintln!("Skrybá ſkończył. Prográm ſpiſány do „{}”.", out.display());
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn translate(path: &Path) -> Result<String, String> {
    let (toks, pisma) = powolania::zbierz(path)?;
    let prog = parser::Parser::new(toks)
        .parse_program()
        .map_err(|e| error::render(&e, &pisma))?;
    let miana: Vec<String> = pisma.into_iter().map(|p| p.shown).collect();
    Ok(codegen::generate(&prog, &miana))
}

fn build(src: &Path, out: &Path) -> Result<(), String> {
    let c = translate(src)?;
    let cfile = std::env::temp_dir().join(format!("sarmata_{}.c", std::process::id()));
    std::fs::write(&cfile, c).map_err(|e| format!("Nie vdáło ſię zápiſáć brudnopiſu: {e}"))?;
    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
    let result = Command::new(&cc)
        .args(["-std=gnu11", "-O2", "-w", "-o"])
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
