//! Każde pismo w `przyklady/` i `tests/naprawy/` z plikiem `.oczekiwane` obok musi wydać dokładnie
//! to, co tam spisano (`tests/naprawy/` to błędy już naprawione, by nie wróciły).
//! Plik `.wejscie` (jeśli jest) idzie na wejście programu. Pisma w `tests/grzechy/` muszą zaś
//! zostać odrzucone, a komunikat skryby zawierać tekst z pliku `.grzech`. Pisma bez `.grzech`
//! to księgi, które tamte powołują.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

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

/// Puszcza program, lecz nie dłużej niż 20 sekund: zawieszony zwraca `Err`, nie wiesza testów.
fn run_limited(cmd: &mut Command) -> Result<Output, String> {
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let pipe = |r: Option<Box<dyn Read + Send>>| {
        let mut r = r.unwrap();
        std::thread::spawn(move || {
            let mut v = Vec::new();
            let _ = r.read_to_end(&mut v);
            v
        })
    };
    let out = pipe(child.stdout.take().map(|p| Box::new(p) as _));
    let err = pipe(child.stderr.take().map(|p| Box::new(p) as _));
    let start = Instant::now();
    let status = loop {
        if let Some(st) = child.try_wait().unwrap() {
            break st;
        }
        if start.elapsed() > Duration::from_secs(20) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("program zawiesił się (ponad 20 s)".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    Ok(Output {
        status,
        stdout: out.join().unwrap(),
        stderr: err.join().unwrap(),
    })
}

fn scratch(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("sarmata-testy-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d.join(name)
}

#[test]
fn przyklady_daja_oczekiwane_wyniki() {
    let checked = wyniki("przyklady");
    assert!(checked >= 8, "sprawdzono ledwie {checked} przykładów");
}

#[test]
fn naprawione_bledy_nie_wracaja() {
    let checked = wyniki("tests/naprawy");
    assert!(checked >= 6, "sprawdzono ledwie {checked} pism");
}

/// Kompiluje i puszcza każde pismo z `.oczekiwane` w `dir`; zwraca, ile sprawdzono.
fn wyniki(dir: &str) -> usize {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut list = Vec::new();
    pisma(&root.join(dir), &mut list);
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
        let run = match run_limited(Command::new(&exe).current_dir(dir).stdin(stdin)) {
            Ok(run) => run,
            Err(e) => {
                failures.push(format!("{}: {e}", src.display()));
                continue;
            }
        };
        // Programy mogą coś spisać do plików obok siebie; sprzątamy, by drzewo zostało czyste.
        let _ = std::fs::remove_file(dir.join("raport.txt"));
        let got = String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n");
        let want = std::fs::read_to_string(&expected).unwrap().replace("\r\n", "\n");
        if got != want {
            failures.push(format!(
                "{}:\n--- oczekiwano\n{want}\n--- otrzymano\n{got}",
                src.display()
            ));
        }
        checked += 1;
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
    checked
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
        // Na Windows piſmá z gita maią końce CRLF — przed zrównánym z kazániami
        // ſkryby (które mówią iednym «\n») ſprowádźmy ie do ſpołu.
        let want = want.replace("\r\n", "\n");
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
            let run = match run_limited(Command::new(&exe).stdin(Stdio::null())) {
                Ok(run) => run,
                Err(e) => {
                    failures.push(format!("{}: {e}", src.display()));
                    continue;
                }
            };
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

#[test]
fn pismo_bez_rozszerzenia_nie_bywa_nadpisane() {
    let src = scratch("bez-rozszerzenia");
    let tresc = "Obwieść „witay”.\nAmen.\n";
    std::fs::write(&src, tresc).unwrap();
    // Domyślne wyjście to miano pisma bez rozszerzenia w bieżącym katalogu, czyli tu — ono samo.
    let build = Command::new(sarmata())
        .current_dir(src.parent().unwrap())
        .arg("bez-rozszerzenia")
        .output()
        .unwrap();
    assert!(!build.status.success(), "skryba nadpisał pismo programem");
    assert_eq!(std::fs::read_to_string(&src).unwrap(), tresc);
    let build = Command::new(sarmata())
        .arg(&src)
        .arg("-o")
        .arg(&src)
        .output()
        .unwrap();
    assert!(
        !build.status.success(),
        "skryba nadpisał pismo programem (-o)"
    );
    assert_eq!(std::fs::read_to_string(&src).unwrap(), tresc);
}

#[test]
fn uruchom_obwieszcza_smierc_od_sygnalu() {
    let src = scratch("wieza.sar");
    std::fs::write(
        &src,
        "Uczynek wieża, przyimuiący liczbę:\n    Oddaj wieżę z liczby i ieden.\nObwieść wieżę z iednego.\nAmen.\n",
    )
    .unwrap();
    let run = Command::new(sarmata())
        .arg("uruchom")
        .arg(&src)
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&run.stderr);
    assert_eq!(run.status.code(), Some(128 + 11), "stderr: {err}");
    assert!(err.contains("SIGSEGV"), "program padł bez słowa: {err}");
}

#[test]
fn wyjscie_moze_stac_przed_pismem() {
    let src = scratch("przed.sar");
    std::fs::write(&src, "Obwieść „witay”.\nAmen.\n").unwrap();
    let exe = scratch("przed");
    let build = Command::new(sarmata())
        .arg("-o")
        .arg(&exe)
        .arg(&src)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let run = Command::new(&exe).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&run.stdout), "witay\n");
}

#[test]
fn rozmowa_konczy_sie_na_zakoncz() {
    use std::io::Write;
    let mut child = Command::new(sarmata())
        .arg("rozmowa")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Wejścia nie zamykamy: rozmowa ma skończyć się sama, na «Zakończ.», nie na końcu wejścia.
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all("Obwieść 1.\nZakończ.\n".as_bytes())
        .unwrap();
    stdin.flush().unwrap();
    let start = std::time::Instant::now();
    while child.try_wait().unwrap().is_none() {
        if start.elapsed() > std::time::Duration::from_secs(20) {
            let _ = child.kill();
            panic!("rozmowa trwa dalej po «Zakończ.»");
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n");
}
