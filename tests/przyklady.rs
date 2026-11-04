//! Każde pismo w `przyklady/` z plikiem `.oczekiwane` obok musi wydać dokładnie to, co tam spisano.
//! Plik `.wejscie` (jeśli jest) idzie na wejście programu. Pisma w `tests/grzechy/` muszą zaś
//! zostać odrzucone, a komunikat skryby zawierać tekst z pliku `.grzech`. Pisma bez `.grzech`
//! to księgi, które tamte powołują.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn sarmata() -> &'static str {
    env!("CARGO_BIN_EXE_sarmata")
}

fn pisma(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            pisma(&p, out);
        } else if p.extension().is_some_and(|x| x == "sar") {
            out.push(p);
        }
    }
}

fn scratch(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("sarmata-testy-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d.join(name)
}

#[test]
fn przyklady_daja_oczekiwane_wyniki() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut list = Vec::new();
    pisma(&root.join("przyklady"), &mut list);
    list.sort();
    let mut checked = 0;
    let mut failures = Vec::new();
    for src in list {
        let expected = src.with_extension("oczekiwane");
        if !expected.exists() {
            continue;
        }
        let stem = src.file_stem().unwrap().to_string_lossy().to_string();
        let exe = scratch(&stem);
        let build = Command::new(sarmata())
            .arg(&src)
            .arg("-o")
            .arg(&exe)
            .output()
            .unwrap();
        if !build.status.success() {
            failures.push(format!(
                "{}: nie skompilowano\n{}",
                src.display(),
                String::from_utf8_lossy(&build.stderr)
            ));
            continue;
        }
        let input = src.with_extension("wejscie");
        let stdin = if input.exists() {
            Stdio::from(std::fs::File::open(&input).unwrap())
        } else {
            Stdio::null()
        };
        let dir = src.parent().unwrap();
        let run = Command::new(&exe)
            .current_dir(dir)
            .stdin(stdin)
            .output()
            .unwrap();
        // Programy mogą coś spisać do plików obok siebie; sprzątamy, by drzewo zostało czyste.
        let _ = std::fs::remove_file(dir.join("raport.txt"));
        let got = String::from_utf8_lossy(&run.stdout);
        let want = std::fs::read_to_string(&expected).unwrap();
        if got != want {
            failures.push(format!(
                "{}:\n--- oczekiwano\n{want}\n--- otrzymano\n{got}",
                src.display()
            ));
        }
        checked += 1;
    }
    assert!(checked >= 8, "sprawdzono ledwie {checked} przykładów");
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn grzechy_sa_wytykane() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut list = Vec::new();
    pisma(&root.join("tests/grzechy"), &mut list);
    list.sort();
    let mut failures = Vec::new();
    for src in &list {
        let Ok(want) = std::fs::read_to_string(src.with_extension("grzech")) else {
            continue;
        };
        let want = want.trim();
        let exe = scratch("grzech");
        let build = Command::new(sarmata())
            .arg(src)
            .arg("-o")
            .arg(&exe)
            .output()
            .unwrap();
        let msg = if build.status.success() {
            // Grzech biegu: program się kompiluje, lecz pada przy wykonaniu.
            let run = Command::new(&exe).stdin(Stdio::null()).output().unwrap();
            if run.status.success() {
                failures.push(format!("{}: grzech przeszedł niezauważony", src.display()));
                continue;
            }
            String::from_utf8_lossy(&run.stderr).to_string()
        } else {
            String::from_utf8_lossy(&build.stderr).to_string()
        };
        if !msg.contains(want) {
            failures.push(format!(
                "{}: oczekiwano „{want}”, a skryba rzekł:\n{msg}",
                src.display()
            ));
        }
    }
    assert!(!list.is_empty());
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
