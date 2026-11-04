//! Powołania: «Powołując się na „księga.sar”.» wplata w pismo uczynki i rody z innego pliku.
//!
//! Księga to zwykłe pismo zakończone «Amen.», w którym stoją same uczynki i rody.
//! Jej słowa wstawia się w miejsce powołania, nim parser zacznie czytać, a numery
//! wierszy niosą numer pisma (zob. `error::WIERSZE_PISMA`), by grzechy wskazywały
//! właściwy plik.

use std::path::{Path, PathBuf};

use crate::error::{self, CompileError, Pismo, WIERSZE_PISMA};
use crate::lexer::{self, K, StrPart, Token};
use crate::words::Kw;

type R<T> = Result<T, CompileError>;

const TERMINATORS: &[&str] = &[".", "!", "?", ";", ":"];
const POWOLANIE: &str = "Powołanie piſze ſię ták: «Powołuiąc ſię ná „kſięgá.sar”.»";

/// Czyta pismo główne wraz ze wszystkimi powołanymi księgami. Zwraca słowa gotowe
/// dla parsera i miana pism (do obwieszczeń grzechów biegu).
pub fn zbierz(path: &Path) -> Result<(Vec<Token>, Vec<Pismo>), String> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| format!("Piſmá „{}” odczytáć nie ſpoſób: {e}", path.display()))?;
    let mut z = Zbieracz::default();
    match z.pismo(path, path.display().to_string(), src, false) {
        Ok(toks) => Ok((toks, z.pisma)),
        Err(e) => Err(error::render(&e, &z.pisma)),
    }
}

#[derive(Default)]
struct Zbieracz {
    pisma: Vec<Pismo>,
    kanony: Vec<PathBuf>,
    /// Pisma właśnie czytane, od głównego w głąb — do wykrywania błędnego koła.
    stos: Vec<usize>,
}

impl Zbieracz {
    fn pismo(&mut self, path: &Path, shown: String, src: String, ksiega: bool) -> R<Vec<Token>> {
        let id = self.pisma.len();
        self.kanony.push(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()));
        self.pisma.push(Pismo { shown, src });
        self.stos.push(id);
        let toks = lexer::lex(&self.pisma[id].src, id * WIERSZE_PISMA)?;
        let dir = path.parent().unwrap_or(Path::new("")).to_path_buf();

        let mut out = Vec::with_capacity(toks.len());
        let mut depth = 0usize;
        let mut start = true;
        let mut i = 0;
        while i < toks.len() {
            let t = &toks[i];
            if start && depth == 0 && t.k == K::Word {
                if t.norm == "powoluiac" {
                    let (names, next) = powolanie(&toks, i)?;
                    for name in names {
                        out.extend(self.ksiega(&dir, name)?);
                    }
                    i = next;
                    continue;
                }
                if ksiega {
                    match t.kw {
                        Some(Kw::Uczynek | Kw::Rod) => {}
                        Some(Kw::Amen) => {
                            // Reszty (kropki i końca) nie bierzemy: parser zobaczy «Amen.»
                            // dopiero na końcu pisma głównego.
                            if toks.get(i + 2).is_some_and(|n| n.k != K::Eof) {
                                return Err(at(&toks[i + 2], "Po «Amen.» nic iuż piſáć nie przyſtoi."));
                            }
                            self.stos.pop();
                            return Ok(out);
                        }
                        _ => {
                            return Err(at(
                                t,
                                format!(
                                    "W kſiędze „{}” przyſtoi ſpiſywáć ieno vczynki i rody. Rozkázy należą do piſmá głównego.",
                                    self.pisma[id].shown
                                ),
                            ));
                        }
                    }
                }
            }
            match t.k {
                K::Indent => depth += 1,
                K::Dedent => depth -= 1,
                K::Eof if ksiega => {
                    return Err(at(t, "Kſięgá też kończy ſię modlitwą — brák «Amen.» ná końcu."));
                }
                _ => {}
            }
            start = matches!(&t.k, K::P(p) if TERMINATORS.contains(p))
                || matches!(t.k, K::Indent | K::Dedent);
            out.push(t.clone());
            i += 1;
        }
        self.stos.pop();
        Ok(out)
    }

    /// Wczytuje księgę, o której mówi cytat `name`. Księgę już wczytaną pomija.
    fn ksiega(&mut self, dir: &Path, name: &Token) -> R<Vec<Token>> {
        let K::Str(parts) = &name.k else { unreachable!() };
        let StrPart::Lit(rel) = &parts[0] else { unreachable!() };
        let mut path = dir.join(rel);
        if path.extension().is_none() {
            path.set_extension("sar");
        }
        let kanon = path.canonicalize().unwrap_or_else(|_| path.clone());
        if let Some(id) = self.kanony.iter().position(|k| *k == kanon) {
            if let Some(pos) = self.stos.iter().position(|&s| s == id) {
                let kolo: Vec<&str> = self.stos[pos..]
                    .iter()
                    .chain([&id])
                    .map(|&s| self.pisma[s].shown.as_str())
                    .collect();
                return Err(at(
                    name,
                    format!("Błędne koło powołań: {}. Kſięgá nie może powoływáć ſámey śiebie.", kolo.join(" → ")),
                ));
            }
            return Ok(Vec::new());
        }
        let shown = path.display().to_string();
        let src = std::fs::read_to_string(&path)
            .map_err(|e| {
                let why = match e.kind() {
                    std::io::ErrorKind::NotFound => "tákiey kſięgi w tym mieyſcu nie mász".to_string(),
                    _ => e.to_string(),
                };
                at(name, format!("Kſięgi „{shown}” odczytáć nie ſpoſób: {why}. Ścieżkę liczy ſię od piſmá, co powołuie."))
            })?;
        self.pismo(&path, shown, src, true)
    }
}

/// «Powołując się na „a.sar”, „b.sar” i „c.sar”.» — zwraca cytaty i miejsce po zdaniu.
fn powolanie(toks: &[Token], i: usize) -> R<(Vec<&Token>, usize)> {
    let mut j = i + 1;
    if toks.get(j).is_some_and(|t| t.norm == "na") {
        j += 1;
    } else {
        return Err(at(&toks[j.min(toks.len() - 1)], POWOLANIE));
    }
    let mut names = Vec::new();
    loop {
        let t = &toks[j];
        match &t.k {
            K::Str(parts) if matches!(parts.as_slice(), [StrPart::Lit(s)] if !s.trim().is_empty()) => {
                names.push(t)
            }
            _ => return Err(at(t, POWOLANIE)),
        }
        j += 1;
        let sep = &toks[j];
        match &sep.k {
            K::P(".") | K::P(";") | K::P("!") => return Ok((names, j + 1)),
            K::P(",") => j += 1,
            K::Word if sep.norm == "i" || sep.norm == "oraz" => j += 1,
            _ => return Err(at(sep, POWOLANIE)),
        }
    }
}

fn at(t: &Token, msg: impl Into<String>) -> CompileError {
    CompileError::new(t.line, t.col, t.width(), msg)
}
