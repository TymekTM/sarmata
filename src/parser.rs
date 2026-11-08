//! Parser: czyta prozę zdanie po zdaniu i składa z niej drzewo.
//!
//! Miana rzeczy rozpoznaje po rdzeniu (więc „sumę” to „suma”), a ile rzeczy
//! wziąć do uczynku, wie z jego nagłówka. Stąd „silnia z n bez jednego”
//! nie potrzebuje nawiasów.

use std::collections::HashMap;

use crate::ast::*;
use crate::error::CompileError;
use crate::lexer::{self, K, StrPart, Token};
use crate::text::{ascii_ident, normalize, stem};
use crate::words::{Kw, NumClass, number, roman};

type R<T> = Result<T, CompileError>;

/// Uczynki wbudowane: miano, ile rzeczy, funkcja biegu, dodatkowe przegrody między rzeczami.
pub const BUILTINS: &[(&str, usize, &str, &[Kw])] = &[
    ("pierwiastek", 1, "sw_b_pierwiastek", &[]),
    ("wartość bezwzględna", 1, "sw_b_abs", &[]),
    ("zaokrąglenie", 1, "sw_b_zaokraglenie", &[]),
    ("część całkowita", 1, "sw_b_calkowita", &[]),
    ("wartość liczbowa", 1, "sw_b_liczbowa", &[]),
    ("losowa liczba", 2, "sw_b_losowa", &[Kw::Do]),
    ("wielkie litery", 1, "sw_b_wielkie", &[]),
    ("małe litery", 1, "sw_b_male", &[]),
    ("rozbiór", 1, "sw_b_rozbior", &[]),
    ("treść pliku", 1, "sw_b_tresc", &[]),
    ("wiersze pliku", 1, "sw_b_wiersze", &[]),
    ("największy", 1, "sw_b_max", &[]),
    ("najmniejszy", 1, "sw_b_min", &[]),
    ("kopia", 1, "sw_b_kopia", &[]),
    ("złączenie", 2, "sw_b_zlacz", &[Kw::Przez]),
    ("rozcięcie", 2, "sw_b_rozciecie", &[Kw::Przez]),
    ("rodzaj", 1, "sw_b_rodzaj", &[]),
    ("słowa wywołania", 0, "sw_b_slowa_wywolania", &[]),
    ("zawartość katalogu", 1, "sw_b_zawartosc", &[]),
];

const SIMPLE_VERBS: &[Kw] = &[
    Kw::Niechaj,
    Kw::Odtad,
    Kw::Obwiesc,
    Kw::Rzekl,
    Kw::Zapytaj,
    Kw::Wczytaj,
    Kw::Zwieksz,
    Kw::Zmniejsz,
    Kw::Pomnoz,
    Kw::Podziel,
    Kw::Dodaj,
    Kw::Odejmij,
    Kw::Przerwij,
    Kw::Pomin,
    Kw::Oddaj,
    Kw::Uczyn,
    Kw::Posortuj,
    Kw::Usun,
    Kw::Zapisz,
    Kw::Dopisz,
    Kw::Rzuc,
    Kw::Zakoncz,
    Kw::Utworz,
    Kw::Przenies,
];

struct VarEntry {
    words: Vec<String>,
    stems: Vec<String>,
    cname: String,
}

struct FuncInfo {
    words: Vec<String>,
    stems: Vec<String>,
    display: String,
    arity: usize,
    cname: String,
    builtin: bool,
    seps: Vec<Kw>,
}

struct NamedInfo {
    words: Vec<String>,
    stems: Vec<String>,
    display: String,
}

#[derive(Clone, Debug)]
enum NK {
    Var(String),
    Func(usize),
    Rod(usize),
    Field(usize),
}

#[derive(Clone, Debug)]
struct Found {
    len: usize,
    exact: bool,
    rank: usize,
    kind: NK,
}

pub struct Parser {
    toks: Vec<Token>,
    pos: usize,
    scopes: Vec<Vec<VarEntry>>,
    fn_base: Option<usize>,
    fn_locals: Vec<(String, String)>,
    loop_depth: usize,
    funcs: Vec<FuncInfo>,
    func_at: HashMap<usize, usize>,
    rods: Vec<NamedInfo>,
    rod_fields: Vec<Vec<usize>>,
    fields: Vec<NamedInfo>,
    globals: Vec<(String, String)>,
    counter: usize,
    func_defs: Vec<FuncDef>,
    /// Miejsca „dla każdego”, których regestr składany w locie właśnie się czyta.
    comp_active: Vec<usize>,
}

fn is_name_word(t: &Token) -> bool {
    t.k == K::Word && t.kw.is_none()
}

/// Słowo miana; liczebnik porządkowy też, jeśli dalej idzie zwykłe słowo („pierwsza liczba”).
fn name_word_at(toks: &[Token], i: usize) -> bool {
    let tok = |i: usize| &toks[i.min(toks.len() - 1)];
    is_name_word(tok(i)) || (matches!(tok(i).kw, Some(Kw::Ordinal(_))) && is_name_word(tok(i + 1)))
}

fn words_of(toks: &[Token]) -> (Vec<String>, Vec<String>, String) {
    let words: Vec<String> = toks.iter().map(|t| t.norm.clone()).collect();
    let stems = words.iter().map(|w| stem(w)).collect();
    let display = toks
        .iter()
        .map(|t| t.raw.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    (words, stems, display)
}

fn split_name(name: &str) -> (Vec<String>, Vec<String>) {
    let words: Vec<String> = name.split_whitespace().map(normalize).collect();
    let stems = words.iter().map(|w| stem(w)).collect();
    (words, stems)
}

fn err_at(t: &Token, msg: impl Into<String>) -> CompileError {
    CompileError::new(t.line, t.col, t.width(), msg)
}

fn describe(t: &Token) -> String {
    match &t.k {
        K::Eof => "koniec piſmá".to_string(),
        K::Indent => "wcięćie".to_string(),
        K::Dedent => "koniec bloku".to_string(),
        _ => format!("„{}”", t.raw),
    }
}

/// Podmiot warunku, by „á ieśli podzielna przez trzy” wiedziało, o czym mowa.
fn subject_of(e: &Expr) -> Option<Expr> {
    match e {
        Expr::Bin(_, l, _, _) | Expr::Un(_, l, _) => Some((**l).clone()),
        // „Ieśli x iest większe niż ieden, a mnieysze niż dziesięć: … A ieśli podzielne przez trzy”
        Expr::Not(inner, _) | Expr::And(inner, _, _) | Expr::Or(inner, _, _) => subject_of(inner),
        _ => None,
    }
}

/// Miejscownik z obocznością spółgłosek: „o autorze” → autor, „o kocie” → kot,
/// „w rządzie” → rząd, „na moście” → most.
fn locative_of(form: &str, base: &str) -> bool {
    [("rze", "r"), ("dzie", "d"), ("scie", "st"), ("cie", "t")]
        .iter()
        .any(|(loc, nom)| {
            form.strip_suffix(loc)
                .is_some_and(|r| base.strip_prefix(r) == Some(nom))
        })
}

/// Nagłówek uczynku: miano, rzeczy (parametry) i pozycja dwukropka.
fn parse_header(toks: &[Token], start: usize) -> R<(Vec<Token>, Vec<Vec<Token>>, usize)> {
    let tok = |i: usize| &toks[i.min(toks.len() - 1)];
    let mut i = start + 1;
    let mut name = Vec::new();
    while name_word_at(toks, i) {
        name.push(tok(i).clone());
        i += 1;
    }
    if name.is_empty() {
        return Err(err_at(
            tok(i),
            "Vczynek bez miána? Nápiſz: «Vczynek ſilniá, przyimuiący n:».",
        ));
    }
    if tok(i).k == K::P(",") {
        i += 1;
    }
    let mut params = Vec::new();
    if matches!(
        tok(i).kw,
        Some(Kw::Przyjmujacy | Kw::Z | Kw::O | Kw::Dla | Kw::Od | Kw::Na)
    ) {
        let intro = tok(i).raw.clone();
        i += 1;
        loop {
            let mut p = Vec::new();
            while name_word_at(toks, i) {
                p.push(tok(i).clone());
                i += 1;
            }
            if p.is_empty() && !params.is_empty() && tok(i).kw == Some(Kw::Przyjmujacy) {
                let name: Vec<&str> = name.iter().map(|t| t.raw.as_str()).collect();
                return Err(err_at(
                    tok(i),
                    format!(
                        "Rzeczy przyimowáne iuż wymieniono po «{intro}», drugie «{}» zbędne. \
                         Nápiſz álbo «Vczynek {} {intro} …:», álbo «Vczynek {}, przyimuiący …:».",
                        tok(i).raw,
                        name.join(" "),
                        name.join(" ")
                    ),
                ));
            }
            if p.is_empty() {
                return Err(err_at(
                    tok(i),
                    format!(
                        "Spodziewáłem ſię miána rzeczy, którą vczynek przyimuie, á ieſt {}.",
                        describe(tok(i))
                    ),
                ));
            }
            params.push(p);
            if tok(i).k == K::P(",") || matches!(tok(i).kw, Some(Kw::I | Kw::Oraz)) {
                i += 1;
                continue;
            }
            break;
        }
    }
    if tok(i).k != K::P(":") {
        return Err(err_at(
            tok(i),
            format!(
                "Po nágłówku vczynku przyſtoi dwukropek, á ieſt {}.",
                describe(tok(i))
            ),
        ));
    }
    Ok((name, params, i))
}

impl Parser {
    pub fn new(toks: Vec<Token>) -> Parser {
        let mut p = Parser {
            toks,
            pos: 0,
            scopes: vec![Vec::new()],
            fn_base: None,
            fn_locals: Vec::new(),
            loop_depth: 0,
            funcs: Vec::new(),
            func_at: HashMap::new(),
            rods: Vec::new(),
            rod_fields: Vec::new(),
            fields: Vec::new(),
            globals: Vec::new(),
            counter: 0,
            func_defs: Vec::new(),
            comp_active: Vec::new(),
        };
        for (name, arity, cfn, seps) in BUILTINS {
            let (words, stems) = split_name(name);
            p.funcs.push(FuncInfo {
                words,
                stems,
                display: name.to_string(),
                arity: *arity,
                cname: cfn.to_string(),
                builtin: true,
                seps: seps.to_vec(),
            });
        }
        p
    }

    // ───────────────────────── narzędzia ─────────────────────────

    fn tok(&self, p: usize) -> &Token {
        &self.toks[p.min(self.toks.len() - 1)]
    }
    fn peek(&self) -> &Token {
        self.tok(self.pos)
    }
    fn kw_at(&self, off: usize) -> Option<Kw> {
        self.tok(self.pos + off).kw
    }
    fn is_kw(&self, k: Kw) -> bool {
        self.peek().kw == Some(k)
    }
    fn eat_kw(&mut self, k: Kw) -> bool {
        if self.is_kw(k) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn is_p(&self, s: &str) -> bool {
        matches!(&self.peek().k, K::P(p) if *p == s)
    }
    fn eat_p(&mut self, s: &str) -> bool {
        if self.is_p(s) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn norm_at(&self, off: usize) -> &str {
        let t = self.tok(self.pos + off);
        if t.k == K::Word { &t.norm } else { "" }
    }
    fn err_here(&self, msg: impl Into<String>) -> CompileError {
        err_at(self.peek(), msg)
    }
    fn expected(&self, what: &str) -> CompileError {
        self.err_here(format!(
            "Spodziewáłem ſię {what}, á zástáłem {}.",
            describe(self.peek())
        ))
    }
    fn expect_kw(&mut self, k: Kw, what: &str) -> R<()> {
        if self.eat_kw(k) {
            Ok(())
        } else {
            Err(self.expected(what))
        }
    }
    fn is_terminator(&self) -> bool {
        self.is_p(".") || self.is_p("!") || self.is_p("?") || self.is_p(";")
    }
    fn expect_terminator(&mut self) -> R<()> {
        if self.is_terminator() {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.expected("końcá zdánia (kropki)"))
        }
    }
    fn line(&self) -> usize {
        self.peek().line
    }

    // ───────────────────────── miana ─────────────────────────

    fn match_at(
        &self,
        p: usize,
        words: &[String],
        stems: &[String],
        allow_kw: bool,
    ) -> Option<bool> {
        let mut exact = true;
        for (i, (w, s)) in words.iter().zip(stems).enumerate() {
            let t = self.tok(p + i);
            if t.k != K::Word
                || (!allow_kw && t.kw.is_some() && !matches!(t.kw, Some(Kw::Ordinal(_))))
            {
                return None;
            }
            if t.norm == *w {
                continue;
            }
            if stem(&t.norm) == *s || locative_of(&t.norm, w) {
                exact = false;
                continue;
            }
            return None;
        }
        Some(exact)
    }

    fn visible_frames(&self) -> Vec<usize> {
        let n = self.scopes.len();
        match self.fn_base {
            Some(b) => (b..n).rev().chain(std::iter::once(0)).collect(),
            None => (0..n).rev().collect(),
        }
    }

    fn find_names(&self, p: usize) -> Vec<Found> {
        let mut found = Vec::new();
        for (rank, fi) in self.visible_frames().into_iter().enumerate() {
            for v in &self.scopes[fi] {
                if let Some(exact) = self.match_at(p, &v.words, &v.stems, false) {
                    found.push(Found {
                        len: v.words.len(),
                        exact,
                        rank,
                        kind: NK::Var(v.cname.clone()),
                    });
                }
            }
        }
        for (i, f) in self.funcs.iter().enumerate() {
            if let Some(exact) = self.match_at(p, &f.words, &f.stems, f.builtin) {
                let rank = if f.builtin { 1500 } else { 1000 };
                found.push(Found {
                    len: f.words.len(),
                    exact,
                    rank,
                    kind: NK::Func(i),
                });
            }
        }
        for (i, f) in self.fields.iter().enumerate() {
            if let Some(exact) = self.match_at(p, &f.words, &f.stems, false) {
                found.push(Found {
                    len: f.words.len(),
                    exact,
                    rank: 2000,
                    kind: NK::Field(i),
                });
            }
        }
        for (i, r) in self.rods.iter().enumerate() {
            if let Some(exact) = self.match_at(p, &r.words, &r.stems, false) {
                found.push(Found {
                    len: r.words.len(),
                    exact,
                    rank: 3000,
                    kind: NK::Rod(i),
                });
            }
        }
        found
    }

    fn kind_name(&self, k: &NK) -> String {
        match k {
            NK::Var(c) => {
                for frame in &self.scopes {
                    if let Some(v) = frame.iter().find(|v| &v.cname == c) {
                        return format!("rzecz „{}”", v.words.join(" "));
                    }
                }
                "rzecz".to_string()
            }
            NK::Func(i) => format!("vczynek „{}”", self.funcs[*i].display),
            NK::Rod(i) => format!("ród „{}”", self.rods[*i].display),
            NK::Field(i) => format!("pole „{}”", self.fields[*i].display),
        }
    }

    fn best(&self, mut list: Vec<Found>, p: usize) -> R<Option<Found>> {
        // Ród w środku wyrażenia nic nie znaczy (tworzy się go słowem «nowy»), więc ustępuje
        // każdej innej rzeczy, nawet gdy jego miano pasuje dokładniej.
        list.sort_by_key(|f| {
            (
                std::cmp::Reverse(f.len),
                matches!(f.kind, NK::Rod(_)),
                !f.exact,
                f.rank,
            )
        });
        if list.len() >= 2 {
            let (a, b) = (&list[0], &list[1]);
            if a.len == b.len && a.exact == b.exact && a.rank == b.rank {
                return Err(err_at(
                    self.tok(p),
                    format!(
                        "Słowo „{}” dwuznáczne: może znáczyć {} álbo {}. Dayże rzeczom miána o różnych rdzeniach.",
                        self.tok(p).raw,
                        self.kind_name(&a.kind),
                        self.kind_name(&b.kind)
                    ),
                ));
            }
        }
        Ok(list.into_iter().next())
    }

    /// Najlepsze dopasowanie spośród rzeczy, uczynków i rodów oraz osobno spośród pól.
    fn resolve(&self, p: usize) -> R<(Option<Found>, Option<Found>)> {
        let (fields, other): (Vec<Found>, Vec<Found>) = self
            .find_names(p)
            .into_iter()
            .partition(|f| matches!(f.kind, NK::Field(_)));
        Ok((self.best(other, p)?, self.best(fields, p)?))
    }

    /// Czy od pozycji `p` może zacząć się wyrażenie.
    fn starts_operand(&self, p: usize) -> bool {
        let t = self.tok(p);
        match &t.k {
            K::Int(_) | K::Float(_) | K::Str(_) => true,
            K::P(s) => *s == "(" || *s == "-",
            K::Word => match t.kw {
                None => true,
                Some(k) => matches!(
                    k,
                    Kw::Liczebnik
                        | Kw::Pol
                        | Kw::Poltora
                        | Kw::Prawda
                        | Kw::Falsz
                        | Kw::Nic
                        | Kw::Element
                        | Kw::Ordinal(_)
                        | Kw::Dlugosc
                        | Kw::Regestr
                        | Kw::Pusty
                        | Kw::Nowy
                        | Kw::Nie
                        | Kw::Minus
                        | Kw::Spis
                        | Kw::Kazdy
                        | Kw::Plik
                ),
            },
            _ => false,
        }
    }

    /// Jak wyżej, lecz bez „nie” i „minus” — dla zestawień typu „ocena studenta”.
    fn starts_operand_strict(&self, p: usize) -> bool {
        let t = self.tok(p);
        if matches!(t.kw, Some(Kw::Nie | Kw::Minus)) || t.k == K::P("-") {
            return false;
        }
        self.starts_operand(p)
    }

    fn is_func_at(&self, p: usize) -> bool {
        if !is_name_word(self.tok(p)) && !self.funcs.iter().any(|f| f.builtin) {
            return false;
        }
        matches!(
            self.resolve(p),
            Ok((
                Some(Found {
                    kind: NK::Func(_),
                    ..
                }),
                _
            ))
        )
    }

    /// Czasownik-rozkaz; w wyliczeniach po przecinku uczynek bywa raczej kolejną rzeczą.
    fn starts_verb(&self, p: usize) -> bool {
        self.tok(p).kw.is_some_and(|k| SIMPLE_VERBS.contains(&k))
    }

    fn starts_simple(&self, p: usize) -> bool {
        let t = self.tok(p);
        match t.kw {
            Some(k) => SIMPLE_VERBS.contains(&k),
            None => t.k == K::Word && self.is_func_at(p),
        }
    }

    fn starts_compound(&self, p: usize) -> bool {
        let t = self.tok(p);
        matches!(
            t.kw,
            Some(Kw::Jesli | Kw::Poki | Kw::Zawsze | Kw::Dla | Kw::Powtorz | Kw::Sprobuj)
        ) || (t.kw == Some(Kw::Minus) && t.norm == "bez" && self.tok(p + 1).norm == "konca")
    }

    fn new_cname(&mut self, display: &str) -> String {
        self.counter += 1;
        format!("v{}_{}", self.counter, ascii_ident(display))
    }

    fn hoist(&mut self, cname: &str, display: &str) {
        if self.fn_base.is_some() {
            self.fn_locals
                .push((cname.to_string(), display.to_string()));
        } else {
            self.globals.push((cname.to_string(), display.to_string()));
        }
    }

    fn declare(&mut self, toks: &[Token]) -> String {
        let (words, stems, display) = words_of(toks);
        let cname = self.new_cname(&display);
        self.hoist(&cname, &display);
        let frame = self.fn_base.unwrap_or(0);
        self.scopes[frame].push(VarEntry {
            words,
            stems,
            cname: cname.clone(),
        });
        cname
    }

    fn lookup_var(&self, toks: &[Token]) -> Option<String> {
        let (words, stems, _) = words_of(toks);
        for fi in self.visible_frames() {
            for v in &self.scopes[fi] {
                if v.words.len() == words.len()
                    && v.words
                        .iter()
                        .zip(&words)
                        .zip(v.stems.iter().zip(&stems))
                        .all(|((a, b), (sa, sb))| a == b || sa == sb)
                {
                    return Some(v.cname.clone());
                }
            }
        }
        None
    }

    fn gather_name_words(&mut self) -> Vec<Token> {
        let mut out = Vec::new();
        while name_word_at(&self.toks, self.pos) {
            out.push(self.peek().clone());
            self.pos += 1;
        }
        out
    }

    fn name_error(&self, what: &str) -> CompileError {
        let t = self.peek();
        if t.k == K::Word && t.norm == "i" {
            return err_at(
                t,
                "Słowo „i” ieſt ſpóynikiem, nie miánem. Názwiy rzecz inácżey, np. «licznik» álbo «numer».",
            );
        }
        if t.kw.is_some() {
            return err_at(
                t,
                format!(
                    "Słowo „{}” ieſt ſłowem ſkryby i miánem rzeczy być nie może. Názwiy {what} inácżey.",
                    t.raw
                ),
            );
        }
        self.expected(&format!("miána {what}"))
    }

    // ───────────────────────── przedbieg ─────────────────────────

    /// Zbiera nagłówki uczynków i rody, by można ich używać przed miejscem, gdzie spisane.
    fn prepass(&mut self) -> R<()> {
        let mut i = 0;
        while i < self.toks.len() {
            let at_start = i == 0
                || matches!(&self.toks[i - 1].k, K::P(p) if [".", "!", "?", ";", ":"].contains(p))
                || matches!(self.toks[i - 1].k, K::Indent | K::Dedent);
            if at_start && self.toks[i].kw == Some(Kw::Uczynek) {
                let (name, params, _) = parse_header(&self.toks, i)?;
                let (words, stems, display) = words_of(&name);
                if let Some(f) = self.funcs.iter().find(|f| !f.builtin && f.stems == stems) {
                    return Err(err_at(
                        &name[0],
                        format!(
                            "Dwa vczynki o iednym miániu „{}” — ſkrybá nie wie, który ieſt który.",
                            f.display
                        ),
                    ));
                }
                let idx = self.funcs.len();
                self.funcs.push(FuncInfo {
                    words,
                    stems,
                    cname: format!("u{}_{}", idx, ascii_ident(&display)),
                    display,
                    arity: params.len(),
                    builtin: false,
                    seps: Vec::new(),
                });
                self.func_at.insert(i, idx);
            }
            if at_start && self.toks[i].kw == Some(Kw::Rod) {
                self.prepass_rod(i)?;
            }
            i += 1;
        }
        Ok(())
    }

    fn prepass_rod(&mut self, start: usize) -> R<()> {
        let mut i = start + 1;
        let mut name = Vec::new();
        while is_name_word(self.tok(i)) {
            name.push(self.tok(i).clone());
            i += 1;
        }
        if name.is_empty() {
            return Err(err_at(
                self.tok(i),
                "Ród bez miána? Nápiſz: «Ród Student ma miano i ocenę.»",
            ));
        }
        if self.tok(i).kw == Some(Kw::Ma) || self.tok(i).k == K::P(":") {
            i += 1;
        } else {
            return Err(err_at(
                self.tok(i),
                format!(
                    "Po miániu rodu przyſtoi «ma» álbo dwukropek, á ieſt {}.",
                    describe(self.tok(i))
                ),
            ));
        }
        let mut field_ids = Vec::new();
        loop {
            let mut f = Vec::new();
            while is_name_word(self.tok(i)) {
                f.push(self.tok(i).clone());
                i += 1;
            }
            if f.is_empty() {
                return Err(err_at(
                    self.tok(i),
                    format!(
                        "Spodziewáłem ſię miána polá, á ieſt {}.",
                        describe(self.tok(i))
                    ),
                ));
            }
            let (words, stems, display) = words_of(&f);
            let id = match self.fields.iter().position(|x| x.stems == stems) {
                Some(id) => id,
                None => {
                    self.fields.push(NamedInfo {
                        words,
                        stems,
                        display,
                    });
                    self.fields.len() - 1
                }
            };
            field_ids.push(id);
            if self.tok(i).k == K::P(",") || matches!(self.tok(i).kw, Some(Kw::I | Kw::Oraz)) {
                i += 1;
                continue;
            }
            break;
        }
        let (words, stems, display) = words_of(&name);
        self.rods.push(NamedInfo {
            words,
            stems,
            display,
        });
        self.rod_fields.push(field_ids);
        Ok(())
    }

    // ───────────────────────── program ─────────────────────────

    pub fn parse_program(mut self) -> R<Program> {
        self.prepass()?;
        let mut main = Vec::new();
        loop {
            let t = self.peek().clone();
            match (&t.k, t.kw) {
                (K::Eof, _) => {
                    return Err(err_at(
                        &t,
                        "Modlitwá niedokończona — brák «Amen.» ná końcu piſmá.",
                    ));
                }
                (_, Some(Kw::Amen)) => {
                    self.pos += 1;
                    self.is_terminator().then(|| self.pos += 1);
                    if self.peek().k != K::Eof {
                        return Err(self.err_here("Po «Amen.» nic iuż piſáć nie przyſtoi."));
                    }
                    break;
                }
                (K::Indent, _) => {
                    return Err(self.err_here(
                        "Wcięćie bez przyczyny — żadne zdánie z dwukropkiem tu bloku nie zácżynáło.",
                    ));
                }
                (_, Some(Kw::Uczynek)) => self.parse_function()?,
                (_, Some(Kw::Rod)) => self.skip_rod()?,
                _ => self.parse_sentence_into(&mut main)?,
            }
        }
        let sigs = self
            .funcs
            .iter()
            .filter(|f| !f.builtin)
            .map(|f| FuncSig {
                display: f.display.clone(),
                cname: f.cname.clone(),
                arity: f.arity,
            })
            .collect();
        let builtin_count = BUILTINS.len();
        let mut funcs = self.func_defs;
        for f in &mut funcs {
            f.index -= builtin_count;
        }
        Ok(Program {
            sigs,
            funcs,
            globals: self.globals,
            main,
            rods: self
                .rods
                .iter()
                .zip(&self.rod_fields)
                .map(|(r, f)| RodDef {
                    display: r.display.clone(),
                    fields: f.clone(),
                })
                .collect(),
            fields: self.fields.iter().map(|f| f.display.clone()).collect(),
        })
    }

    fn skip_rod(&mut self) -> R<()> {
        while !self.is_terminator() && self.peek().k != K::Eof {
            self.pos += 1;
        }
        self.expect_terminator()
    }

    fn parse_function(&mut self) -> R<()> {
        let start = self.pos;
        if self.fn_base.is_some() {
            return Err(self.err_here("Vczynku w vczynku piſáć nie godzi ſię."));
        }
        let fidx = self.func_at[&start];
        let (_, params, colon) = parse_header(&self.toks, start)?;
        self.pos = colon;
        let base = self.scopes.len();
        self.fn_base = Some(base);
        self.fn_locals.clear();
        let mut frame = Vec::new();
        let mut cparams = Vec::new();
        for p in &params {
            let (words, stems, display) = words_of(p);
            let cname = self.new_cname(&display);
            cparams.push(cname.clone());
            frame.push(VarEntry {
                words,
                stems,
                cname,
            });
        }
        self.scopes.push(frame);
        let saved_loop = std::mem::replace(&mut self.loop_depth, 0);
        let (body, block) = self.parse_body()?;
        if !block {
            self.expect_terminator()?;
        }
        self.loop_depth = saved_loop;
        self.scopes.truncate(base);
        self.fn_base = None;
        self.func_defs.push(FuncDef {
            index: fidx,
            params: cparams,
            locals: std::mem::take(&mut self.fn_locals),
            body,
        });
        Ok(())
    }

    // ───────────────────────── zdania ─────────────────────────

    fn parse_block(&mut self) -> R<Vec<Stmt>> {
        let mut out = Vec::new();
        loop {
            match self.peek().k {
                K::Dedent => {
                    self.pos += 1;
                    break;
                }
                K::Eof => break,
                K::Indent => {
                    return Err(
                        self.err_here("Wcięćie bez przyczyny — nikt tu nowego bloku nie zácżynał.")
                    );
                }
                _ => {}
            }
            if self.is_kw(Kw::Amen) {
                return Err(self.err_here(
                    "«Amen.» przyſtoi ieno ná ſámym końcu piſmá, nie we wnętrzu bloku.",
                ));
            }
            self.parse_sentence_into(&mut out)?;
        }
        Ok(out)
    }

    /// Ciało po nagłówku: blok wcięty po dwukropku albo zdanie w tym samym wierszu.
    /// Zwraca też, czy skończyło się blokiem (wtedy kropki nie trzeba).
    fn parse_body(&mut self) -> R<(Vec<Stmt>, bool)> {
        self.eat_p(",");
        let colon = self.eat_p(":");
        if colon && self.peek().k == K::Indent {
            self.pos += 1;
            return Ok((self.parse_block()?, true));
        }
        if self.starts_compound(self.pos) {
            let (s, block) = self.parse_compound()?;
            return Ok((vec![s], block));
        }
        if !self.starts_simple(self.pos) {
            return Err(self.expected(if colon {
                "wciętego bloku álbo rozkázu po dwukropku"
            } else {
                "rozkázu (np. «obwieść», «niechay», «zwiększ»)"
            }));
        }
        Ok((self.parse_inline_chain()?, false))
    }

    fn parse_inline_chain(&mut self) -> R<Vec<Stmt>> {
        let mut last = self.peek().kw;
        let mut out = vec![self.parse_simple()?];
        loop {
            if self.try_connector() {
                last = self.peek().kw;
                out.push(self.parse_simple()?);
            } else if let Some(verb) = last.filter(|k| matches!(k, Kw::Niechaj | Kw::Odtad))
                && self.try_implied_assign()
            {
                let t = self.peek().clone();
                out.push(self.parse_verb(verb, &t, t.line)?);
            } else {
                return Ok(out);
            }
        }
    }

    /// „Odtąd a iest b, á b iest c” — drugie nadanie bez powtórzonego rozkazu.
    fn try_implied_assign(&mut self) -> bool {
        let save = self.pos;
        let comma = self.eat_p(",");
        let conn = (self.norm_at(0) == "a" || self.is_kw(Kw::I)) && {
            self.pos += 1;
            true
        };
        let mut i = self.pos;
        let named = (comma || conn)
            && (name_word_at(&self.toks, i)
                || matches!(self.tok(i).kw, Some(Kw::Element | Kw::Ordinal(_))));
        if named {
            while !matches!(self.tok(i).k, K::P(_) | K::Eof | K::Indent | K::Dedent) {
                if matches!(self.tok(i).kw, Some(Kw::Iest | Kw::Bedzie)) {
                    return true;
                }
                if self.tok(i).kw.is_some_and(|k| SIMPLE_VERBS.contains(&k)) {
                    break;
                }
                i += 1;
            }
        }
        self.pos = save;
        false
    }

    /// Spójniki między rozkazami w jednym zdaniu: „, a potem”, „po czym”, „i”.
    fn try_connector(&mut self) -> bool {
        let save = self.pos;
        let comma = self.eat_p(",");
        let mut conn = false;
        if self.norm_at(0) == "a" || self.is_kw(Kw::I) {
            self.pos += 1;
            conn = true;
        }
        if self.eat_kw(Kw::Potem) {
            conn = true;
        } else if self.is_kw(Kw::Po) && self.kw_at(1) == Some(Kw::Czym) {
            self.pos += 2;
            conn = true;
        }
        if (comma || conn) && self.starts_simple(self.pos) {
            true
        } else {
            self.pos = save;
            false
        }
    }

    fn is_elif_start(&self) -> bool {
        if self.is_kw(Kw::Azali) {
            return true;
        }
        self.norm_at(0) == "a"
            && self.kw_at(1) == Some(Kw::Jesli)
            && !self.is_else_start()
            && !self.is_catch_start()
    }

    /// „A w razie biedy”, „A jeśli bieda”, „Na wypadek biedy”, „Biada…”
    fn is_catch_start(&self) -> bool {
        let o = usize::from(self.norm_at(0) == "a");
        match self.kw_at(o) {
            Some(Kw::Jesli) => self.kw_at(o + 1) == Some(Kw::Biada),
            Some(Kw::W) => {
                self.kw_at(o + 1) == Some(Kw::Razie) && self.kw_at(o + 2) == Some(Kw::Biada)
            }
            Some(Kw::Na) => {
                self.norm_at(o + 1) == "wipadek" && self.kw_at(o + 2) == Some(Kw::Biada)
            }
            Some(Kw::Biada) => true,
            _ => false,
        }
    }

    /// Biada po próbie: miano biedy po «zwanej» i ciało, co ją opłakuje.
    fn parse_catch(&mut self, stmt: &mut Stmt) -> R<bool> {
        let Stmt::Try {
            catch_var, handler, ..
        } = stmt
        else {
            unreachable!()
        };
        if handler.is_some() {
            return Err(self.err_here("Tá próbá ma iuż ſwoię biedę; drugiey mieć nie może."));
        }
        while !self.is_kw(Kw::Biada) {
            self.pos += 1;
        }
        self.pos += 1;
        // „A w razie biedy wielkiej, zwanej nieszczęściem: …” — reszta to opowieść.
        let mut name = Vec::new();
        loop {
            if self.is_p(",") && self.kw_at(1) == Some(Kw::Zwany) {
                self.pos += 1;
            }
            if self.eat_kw(Kw::Zwany) {
                name = self.gather_name_words();
                if name.is_empty() {
                    return Err(self.name_error("biedy"));
                }
                continue;
            }
            if self.is_p(":")
                || self.is_p(",")
                || self.is_terminator()
                || matches!(self.peek().k, K::Indent | K::Dedent | K::Eof)
                || self.starts_simple(self.pos)
                || self.starts_compound(self.pos)
            {
                break;
            }
            self.pos += 1;
        }
        let (body, block) = if name.is_empty() {
            self.parse_body()?
        } else {
            let (words, stems, display) = words_of(&name);
            let cname = self.new_cname(&display);
            self.hoist(&cname, &display);
            *catch_var = Some(cname.clone());
            self.with_scope(
                Some(VarEntry {
                    words,
                    stems,
                    cname,
                }),
                |p| p.parse_body(),
            )?
        };
        *handler = Some(body);
        Ok(block)
    }

    fn with_scope<T>(
        &mut self,
        entry: Option<VarEntry>,
        f: impl FnOnce(&mut Self) -> R<T>,
    ) -> R<T> {
        self.scopes.push(entry.into_iter().collect());
        let r = f(self);
        self.scopes.pop();
        r
    }

    fn is_else_start(&self) -> bool {
        if self.is_kw(Kw::Inaczej) {
            return true;
        }
        if self.norm_at(0) == "a" && self.kw_at(1) == Some(Kw::Inaczej) {
            return true;
        }
        if self.is_kw(Kw::W) && self.kw_at(1) == Some(Kw::Przeciwnym) {
            return true;
        }
        // „A jeśli nie, …”
        if self.norm_at(0) == "a"
            && self.kw_at(1) == Some(Kw::Jesli)
            && self.kw_at(2) == Some(Kw::Nie)
        {
            let t = self.tok(self.pos + 3);
            return t.k == K::P(",") || t.k == K::P(":") || self.starts_simple(self.pos + 3);
        }
        false
    }

    fn eat_else(&mut self) {
        if self.norm_at(0) == "a" {
            self.pos += 1;
        }
        if self.eat_kw(Kw::Inaczej) {
            return;
        }
        if self.eat_kw(Kw::W) {
            self.pos += 1; // przeciwnym
            self.eat_kw(Kw::Razie);
            return;
        }
        self.pos += 2; // jeśli nie
    }

    fn eat_elif(&mut self) {
        if !self.eat_kw(Kw::Azali) {
            self.pos += 2; // a jeśli
        }
    }

    fn parse_sentence_into(&mut self, out: &mut Vec<Stmt>) -> R<()> {
        if self.is_catch_start() {
            let t = self.peek().clone();
            let Some(mut stmt @ Stmt::Try { .. }) = out.pop() else {
                return Err(err_at(
                    &t,
                    "Biedá bez wcześnieyſzego «Spróbuy» — czego tu ſię lękáć?",
                ));
            };
            let block = self.parse_catch(&mut stmt)?;
            if !block {
                self.expect_terminator()?;
            }
            out.push(stmt);
            return Ok(());
        }
        if self.is_elif_start() || self.is_else_start() {
            let t = self.peek().clone();
            let Some(Stmt::If(arms, els, line)) = out.pop() else {
                return Err(err_at(
                    &t,
                    "«Inácżey» álbo «A ieſli» bez wcześnieyſzego «Ieſli» — do czego to przypiąć?",
                ));
            };
            if els.is_some() {
                return Err(err_at(
                    &t,
                    "To «Ieſli» ma iuż ſwoie «inácżey»; drugiego mieć nie może.",
                ));
            }
            let mut stmt = Stmt::If(arms, None, line);
            let block = self.parse_if_tail(&mut stmt, false)?;
            if !block {
                self.expect_terminator()?;
            }
            out.push(stmt);
            return Ok(());
        }
        if self.is_kw(Kw::Uczynek) {
            return Err(
                self.err_here("Vczynek ſpiſuie ſię ieno ná wierzchu piſmá, nie we wnętrzu bloku.")
            );
        }
        if self.is_kw(Kw::Rod) {
            return Err(
                self.err_here("Ród ſpiſuie ſię ieno ná wierzchu piſmá, nie we wnętrzu bloku.")
            );
        }
        let (at, start, first) = (out.len(), self.pos, self.line());
        let last = if self.starts_compound(self.pos) {
            let (s, block) = self.parse_compound()?;
            out.push(s);
            if !block {
                self.expect_terminator()?;
            }
            self.header_end(start)
        } else {
            out.extend(self.parse_inline_chain()?);
            self.expect_terminator()?;
            self.tok(self.pos - 1).line
        };
        out.insert(at, Stmt::Mark(first, last));
        Ok(())
    }

    /// Wiersz, w którym kończy się nagłówek zdania (do dwukropka albo kropki).
    fn header_end(&self, start: usize) -> usize {
        let mut i = start;
        while !matches!(
            &self.tok(i).k,
            K::P(":" | "." | "!" | "?" | ";") | K::Indent | K::Eof
        ) {
            i += 1;
        }
        self.tok(i).line
    }

    fn parse_compound(&mut self) -> R<(Stmt, bool)> {
        let line = self.line();
        let t = self.peek().clone();
        match t.kw {
            Some(Kw::Jesli) => {
                self.pos += 1;
                let cond = self.parse_cond()?;
                let (body, block) = self.parse_body()?;
                let mut stmt = Stmt::If(vec![(cond, body)], None, line);
                let block = if block {
                    true
                } else {
                    self.parse_if_tail(&mut stmt, true)?
                };
                Ok((stmt, block))
            }
            Some(Kw::Poki) => {
                self.pos += 1;
                let cond = self.parse_cond()?;
                self.eat_p(",");
                self.eat_kw(Kw::Czyn);
                let (body, block) = self.loop_body()?;
                Ok((Stmt::While(cond, body, line), block))
            }
            Some(Kw::Zawsze) | Some(Kw::Minus) => {
                self.pos += if t.kw == Some(Kw::Zawsze) { 1 } else { 2 };
                self.eat_p(",");
                self.eat_kw(Kw::Czyn);
                let (body, block) = self.loop_body()?;
                Ok((Stmt::Forever(body), block))
            }
            Some(Kw::Powtorz) => {
                self.pos += 1;
                let n = self.parse_add()?;
                self.expect_kw(Kw::Razy, "ſłowá «rázy» (np. «Powtórz trzy rázy:»)")?;
                self.eat_p(",");
                self.eat_kw(Kw::Czyn);
                let (body, block) = self.loop_body()?;
                Ok((Stmt::Repeat(n, body, line), block))
            }
            Some(Kw::Dla) => self.parse_for(line),
            Some(Kw::Sprobuj) => {
                self.pos += 1;
                let (body, block) = self.parse_body()?;
                let mut stmt = Stmt::Try {
                    body,
                    catch_var: None,
                    handler: None,
                };
                if block {
                    return Ok((stmt, true));
                }
                // „Spróbuj …, a w razie biedy …” w jednym zdaniu.
                let save = self.pos;
                if self.eat_p(",") && self.is_catch_start() {
                    let block = self.parse_catch(&mut stmt)?;
                    return Ok((stmt, block));
                }
                self.pos = save;
                Ok((stmt, false))
            }
            _ => Err(self.expected("rozkázu")),
        }
    }

    fn loop_body(&mut self) -> R<(Vec<Stmt>, bool)> {
        self.loop_depth += 1;
        let r = self.parse_body();
        self.loop_depth -= 1;
        r
    }

    /// Dalsze „a jeśli …” i „inaczej …” — w tym samym zdaniu albo w następnym.
    /// Warunek w nagłówku: tu „a” (też „, a”) znaczy „i” — „Ieśli belka iest spisem,
    /// a belka nie iest pusta”. Poza nagłówkami „a” zaczyna nowy rozkaz.
    fn parse_cond(&mut self) -> R<Expr> {
        let mut cond = self.parse_expr()?;
        // „Ieśli x iest większe niż trzy, a mnieysze niż dziesięć”
        let subject = subject_of(&cond);
        loop {
            let save = self.pos;
            let line = self.line();
            self.eat_p(",");
            if self.norm_at(0) != "a"
                || self.is_elif_start()
                || self.is_else_start()
                || self.is_catch_start()
                || !(self.starts_operand(self.pos + 1) || self.is_cmp_tail(1))
                || self.starts_verb(self.pos + 1)
                || self.starts_simple(self.pos + 1)
            {
                self.pos = save;
                return Ok(cond);
            }
            self.pos += 1;
            let next = match &subject {
                Some(l) if self.starts_bare_cmp() => self.parse_cmp_with(l.clone())?,
                _ => self.parse_expr()?,
            };
            cond = Expr::And(Box::new(cond), Box::new(next), line);
        }
    }

    fn parse_if_tail(&mut self, stmt: &mut Stmt, same_sentence: bool) -> R<bool> {
        let Stmt::If(arms, els, _) = stmt else {
            unreachable!()
        };
        let mut first = !same_sentence;
        loop {
            let save = self.pos;
            if !first && !self.eat_p(",") {
                return Ok(false);
            }
            first = false;
            if self.is_else_start() {
                self.eat_else();
                let (body, block) = self.parse_body()?;
                *els = Some(body);
                return Ok(block);
            }
            if self.is_elif_start() {
                self.eat_elif();
                let subject = if self.starts_bare_cmp() {
                    subject_of(&arms[0].0)
                } else {
                    None
                };
                let cond = match subject {
                    Some(l) => self.parse_cmp_with(l)?,
                    None => self.parse_cond()?,
                };
                let (body, block) = self.parse_body()?;
                arms.push((cond, body));
                if block {
                    return Ok(true);
                }
                continue;
            }
            self.pos = save;
            return Ok(false);
        }
    }

    fn parse_for(&mut self, line: usize) -> R<(Stmt, bool)> {
        self.pos += 1;
        self.eat_kw(Kw::Kazdy);
        let name = self.gather_name_words();
        if name.is_empty() {
            return Err(self.name_error("rzeczy, co po kolei przybiera wártości"));
        }
        enum Kind {
            Range(Expr, Expr, Option<Expr>),
            Each(Expr),
        }
        let kind = if self.eat_kw(Kw::Od) {
            let from = self.parse_add()?;
            self.expect_kw(Kw::Do, "ſłowá «do» (np. «od iednego do dzieśięćiu»)")?;
            let to = self.parse_add()?;
            let step = if self.eat_kw(Kw::Co) {
                Some(self.parse_add()?)
            } else {
                None
            };
            Kind::Range(from, to, step)
        } else if self.eat_kw(Kw::Z) || self.eat_kw(Kw::W) {
            Kind::Each(self.parse_add()?)
        } else {
            return Err(self.expected("«od … do …» álbo «z …» po miániu rzeczy w pętli"));
        };
        self.eat_p(",");
        self.eat_kw(Kw::Czyn);
        let (words, stems, display) = words_of(&name);
        let cname = self.new_cname(&display);
        self.hoist(&cname, &display);
        self.scopes.push(vec![VarEntry {
            words,
            stems,
            cname: cname.clone(),
        }]);
        let r = self.loop_body();
        self.scopes.pop();
        let (body, block) = r?;
        let stmt = match kind {
            Kind::Range(from, to, step) => Stmt::ForRange {
                var: cname,
                from,
                to,
                step,
                body,
                line,
            },
            Kind::Each(coll) => Stmt::ForEach {
                var: cname,
                coll,
                body,
                line,
            },
        };
        Ok((stmt, block))
    }

    fn parse_simple(&mut self) -> R<Stmt> {
        let line = self.line();
        let t = self.peek().clone();
        let Some(kw) = t.kw else {
            if t.k == K::Word && self.is_func_at(self.pos) {
                let e = self.parse_postfix()?;
                return Ok(Stmt::Expr(e));
            }
            if t.k == K::Word {
                return Err(err_at(
                    &t,
                    format!(
                        "Nie wiem, co czynić z „{}”. Zdánie winno ſię zácżynáć rozkázem, np. «Niechay», «Obwieść», «Zwiększ».",
                        t.raw
                    ),
                ));
            }
            return Err(self.expected("rozkázu"));
        };
        self.pos += 1;
        self.parse_verb(kw, &t, line)
    }

    /// Rozkaz po słowie `kw` (już zjedzonym albo domyślnym, jak w „…, á b iest c”).
    fn parse_verb(&mut self, kw: Kw, t: &Token, line: usize) -> R<Stmt> {
        match kw {
            Kw::Niechaj => {
                self.eat_kw(Kw::Bedzie);
                let place = self.parse_target()?;
                let mut sep = false;
                if self.eat_kw(Kw::Bedzie) {
                    sep = true;
                    let _ = self.eat_kw(Kw::Rowny) || self.eat_kw(Kw::Iako);
                } else if self.eat_kw(Kw::Rowny)
                    || self.eat_kw(Kw::Iako)
                    || self.eat_kw(Kw::Iest)
                    || self.eat_p("=")
                {
                    sep = true;
                }
                if !sep && !(self.starts_operand(self.pos) && !is_name_word(self.peek())) {
                    return Err(self.expected("«będźie», «równe» álbo «=» po miániu rzeczy"));
                }
                let v = self.parse_expr()?;
                Ok(Stmt::Assign(place, v))
            }
            Kw::Odtad => {
                let place = self.parse_place()?;
                if !(self.eat_kw(Kw::Iest) || self.eat_kw(Kw::Bedzie) || self.eat_p("="))
                    && !self.is_kw(Kw::Rowny)
                {
                    return Err(self.expected("«ieſt» álbo «będźie»"));
                }
                self.eat_kw(Kw::Rowny);
                let v = self.parse_expr()?;
                Ok(Stmt::Assign(place, v))
            }
            Kw::Obwiesc => self.parse_print(),
            Kw::Rzekl => {
                self.skip_narration();
                self.parse_print()
            }
            Kw::Zapytaj => {
                let prompt = if matches!(self.peek().k, K::Str(_)) {
                    Some(self.parse_primary()?)
                } else {
                    None
                };
                self.eat_kw(Kw::O);
                let place = self.parse_target()?;
                Ok(Stmt::ReadLine(place, prompt))
            }
            Kw::Wczytaj => {
                let place = self.parse_target()?;
                Ok(Stmt::ReadToken(place))
            }
            Kw::Zwieksz | Kw::Zmniejsz => {
                let place = self.parse_place()?;
                let amount = if self.eat_kw(Kw::O) {
                    self.parse_add()?
                } else {
                    Expr::Int(1)
                };
                let op = if kw == Kw::Zwieksz {
                    CompoundOp::Add
                } else {
                    CompoundOp::Sub
                };
                Ok(Stmt::Compound(place, op, amount, line))
            }
            Kw::Pomnoz | Kw::Podziel => {
                let place = self.parse_place()?;
                self.expect_kw(Kw::Przez, "ſłowá «przez»")?;
                let amount = self.parse_add()?;
                let op = if kw == Kw::Pomnoz {
                    CompoundOp::Mul
                } else {
                    CompoundOp::Div
                };
                Ok(Stmt::Compound(place, op, amount, line))
            }
            Kw::Dodaj => {
                let v = self.parse_add()?;
                self.expect_kw(Kw::Do, "ſłowá «do» (np. «Dodáy ocenę do ocen»)")?;
                let place = self.parse_place()?;
                Ok(Stmt::Compound(place, CompoundOp::Put, v, line))
            }
            Kw::Odejmij => {
                let v = self.parse_add()?;
                self.expect_kw(Kw::Od, "ſłowá «od»")?;
                let place = self.parse_place()?;
                Ok(Stmt::Compound(place, CompoundOp::Sub, v, line))
            }
            Kw::Przerwij | Kw::Pomin => {
                if self.loop_depth == 0 {
                    return Err(err_at(
                        t,
                        format!("«{}» przyſtoi ieno we wnętrzu pętli.", t.raw),
                    ));
                }
                Ok(if kw == Kw::Przerwij {
                    Stmt::Break
                } else {
                    Stmt::Continue
                })
            }
            Kw::Oddaj => {
                if self.fn_base.is_none() {
                    return Err(err_at(t, "«Oddáy» przyſtoi ieno we wnętrzu vczynku."));
                }
                if self.is_terminator() || self.is_p(",") || self.peek().k == K::Dedent {
                    Ok(Stmt::Return(None))
                } else {
                    Ok(Stmt::Return(Some(self.parse_expr()?)))
                }
            }
            Kw::Uczyn => {
                let e = self.parse_postfix()?;
                if !matches!(e, Expr::Call(..) | Expr::Builtin(..)) {
                    return Err(err_at(t, "Po «Vczyń» przyſtoi miano vczynku."));
                }
                Ok(Stmt::Expr(e))
            }
            Kw::Posortuj => {
                let coll = self.parse_postfix()?;
                let mut field = None;
                if self.eat_kw(Kw::Wedlug) {
                    let (_, f) = self.resolve(self.pos)?;
                    let Some(Found {
                        kind: NK::Field(id),
                        len,
                        ..
                    }) = f
                    else {
                        return Err(self.expected("miána polá, według którego ſortowáć"));
                    };
                    self.pos += len;
                    field = Some(id);
                }
                let desc = self.eat_kw(Kw::Malejaco);
                self.eat_kw(Kw::Rosnaco);
                Ok(Stmt::Sort {
                    coll,
                    field,
                    desc,
                    line,
                })
            }
            Kw::Usun => {
                if self.eat_kw(Kw::Plik) {
                    let path = self.parse_add()?;
                    return Ok(Stmt::Expr(Expr::Builtin(
                        "sw_b_usun_plik",
                        vec![path],
                        line,
                    )));
                }
                if self.is_kw(Kw::Element) && matches!(self.kw_at(1), Some(Kw::Ordinal(_))) {
                    self.pos += 1;
                }
                let index = match self.peek().kw {
                    Some(Kw::Ordinal(n)) => {
                        self.pos += 1;
                        if n < 0 { None } else { Some(Expr::Int(n)) }
                    }
                    Some(Kw::Element) => {
                        self.pos += 1;
                        Some(self.parse_add()?)
                    }
                    // „Usuń „chleb” z cen” — klucz ze spisu albo wartość z regestru.
                    _ => {
                        let v = self.parse_add()?;
                        if !(self.eat_kw(Kw::Z) || self.eat_kw(Kw::W)) {
                            return Err(self.expected("«z» (np. «Uſuń „chleb” z cen»)"));
                        }
                        let coll = self.parse_postfix()?;
                        return Ok(Stmt::Expr(Expr::Builtin(
                            "sw_usun_wartosc",
                            vec![coll, v],
                            line,
                        )));
                    }
                };
                self.eat_kw(Kw::Element);
                let _ = self.eat_kw(Kw::Z) || self.eat_kw(Kw::W);
                let coll = self.parse_postfix()?;
                Ok(Stmt::Remove { coll, index, line })
            }
            Kw::Zapisz | Kw::Dopisz => {
                let content = self.parse_add()?;
                self.expect_kw(
                    Kw::Do,
                    "ſłowá «do» (np. «Zápiſz rápórt do pliku „wynik.txt”»)",
                )?;
                self.expect_kw(Kw::Plik, "ſłowá «pliku»")?;
                let path = self.parse_add()?;
                Ok(Stmt::WriteFile {
                    content,
                    path,
                    append: kw == Kw::Dopisz,
                    line,
                })
            }
            Kw::Rzuc => {
                // „Rzuć klątwę „Chleba brak!”.”
                if self.norm_at(0).starts_with("klatw") {
                    self.pos += 1;
                }
                let v = if self.is_terminator()
                    || self.is_p(",")
                    || matches!(self.peek().k, K::Dedent | K::Eof)
                {
                    Expr::Str("klątwa".to_string())
                } else {
                    self.parse_expr()?
                };
                Ok(Stmt::Throw(v, line))
            }
            Kw::Zakoncz => {
                // „Zakończ opowieść.”, „Zakończ z grzechem.”, „Zakończ z kodem trzy.” Słowa przed
                // «z» to sama ozdoba, nawet gdy „program” bywa też mianem zmiennej.
                while is_name_word(self.peek()) {
                    self.pos += 1;
                }
                let code = if self.is_kw(Kw::Z) && self.norm_at(1).starts_with("grzech") {
                    self.pos += 2;
                    Expr::Int(1)
                } else if self.is_kw(Kw::Z) && self.norm_at(1) == "kodem" {
                    self.pos += 2;
                    self.parse_add()?
                } else {
                    Expr::Int(0)
                };
                Ok(Stmt::Exit(code, line))
            }
            Kw::Utworz => {
                if !self.norm_at(0).starts_with("katalog") {
                    return Err(self.expected("«kátálog» (np. «Vtwórz kátálog „kroniki”»)"));
                }
                self.pos += 1;
                let path = self.parse_add()?;
                Ok(Stmt::Expr(Expr::Builtin("sw_b_katalog", vec![path], line)))
            }
            Kw::Przenies => {
                self.eat_kw(Kw::Plik);
                let from = self.parse_add()?;
                self.expect_kw(
                    Kw::Do,
                    "ſłowá «do» (np. «Przenieś plik „a.txt” do „b.txt”»)",
                )?;
                self.eat_kw(Kw::Plik);
                let to = self.parse_add()?;
                Ok(Stmt::Expr(Expr::Builtin(
                    "sw_b_przenies",
                    vec![from, to],
                    line,
                )))
            }
            _ => Err(err_at(
                t,
                format!("Słowo „{}” nie może zácżynáć zdánia.", t.raw),
            )),
        }
    }

    fn at_no_newline(&self) -> bool {
        self.is_kw(Kw::Nie) && self.norm_at(1) == "konczac" && self.norm_at(2) == "wiersza"
    }

    fn parse_print(&mut self) -> R<Stmt> {
        let mut args = Vec::new();
        let mut newline = true;
        if self.is_terminator() || self.peek().k == K::Dedent || self.peek().k == K::Eof {
            return Ok(Stmt::Print(args, true));
        }
        loop {
            if self.at_no_newline() {
                self.pos += 3;
                newline = false;
                break;
            }
            let arg = match self.try_comprehension()? {
                Some(e) => e,
                None => self.parse_not()?,
            };
            args.push(arg);
            let save = self.pos;
            let sep = self.eat_p(",") || self.eat_kw(Kw::I) || self.eat_kw(Kw::Oraz);
            if self.at_no_newline() {
                continue;
            }
            if !sep {
                break;
            }
            if self.starts_verb(self.pos)
                || self.norm_at(0) == "a"
                || self.is_kw(Kw::Potem)
                || self.is_kw(Kw::Po)
                || self.is_else_start()
                || !self.starts_operand(self.pos)
            {
                self.pos = save;
                break;
            }
        }
        Ok(Stmt::Print(args, newline))
    }

    /// „Rzekł tedy stary rotmistrz, gładząc wąsa: …” — wszystko do dwukropka to opowieść,
    /// którą skryba pomija. Bez dwukropka w zdaniu nie ma czego pomijać.
    fn skip_narration(&mut self) {
        let mut p = self.pos;
        loop {
            match &self.tok(p).k {
                K::P(":") => {
                    self.pos = p + 1;
                    return;
                }
                K::P("." | "!" | "?" | ";") | K::Indent | K::Dedent | K::Eof => return,
                _ => p += 1,
            }
        }
    }

    /// Miejsce, któremu nadaje się wartość. Nowe miano zostaje tu ochrzczone.
    fn parse_target(&mut self) -> R<Place> {
        let t = self.peek().clone();
        if !name_word_at(&self.toks, self.pos) {
            if matches!(t.kw, Some(Kw::Element | Kw::Ordinal(_))) {
                return self.parse_place();
            }
            return Err(self.name_error("rzeczy"));
        }
        let (other, field) = self.resolve(self.pos)?;
        if let Some(f) = &field {
            let other_len = other.as_ref().map_or(0, |o| o.len);
            let after = self.pos + f.len;
            if f.len >= other_len
                && self.starts_operand_strict(after)
                && (!is_name_word(self.tok(after)) || !self.find_names(after).is_empty())
            {
                return self.parse_place();
            }
        }
        if let Some(Found {
            kind: NK::Var(_),
            len,
            ..
        }) = &other
            && self.tok(self.pos + len).kw == Some(Kw::Pod)
        {
            return self.parse_place();
        }
        let words = self.gather_name_words();
        if let Some(c) = self.lookup_var(&words) {
            return Ok(Place::Var(c));
        }
        Ok(Place::Var(self.declare(&words)))
    }

    fn parse_place(&mut self) -> R<Place> {
        let t = self.peek().clone();
        let e = self.parse_postfix()?;
        match e {
            Expr::Var(c) => Ok(Place::Var(c)),
            Expr::Index(c, i, l) => Ok(Place::Index(*c, *i, l)),
            Expr::Last(c, l) => Ok(Place::Last(*c, l)),
            Expr::Field(o, f, l) => Ok(Place::Field(*o, f, l)),
            _ => Err(err_at(
                &t,
                "Tu potrzebá rzeczy, którey można nádáć wártość — miána, elementu álbo polá.",
            )),
        }
    }

    // ───────────────────────── wyrażenia ─────────────────────────

    pub fn parse_expr(&mut self) -> R<Expr> {
        if let Some(e) = self.try_comprehension()? {
            return Ok(e);
        }
        self.parse_or()
    }

    /// „dla każdego” dalej w tym samym zdaniu, nie po przecinku (tam zaczyna się pętla).
    fn find_comp_dla(&self) -> Option<usize> {
        let mut depth = 0i32;
        let mut i = self.pos;
        loop {
            let t = self.tok(i);
            match &t.k {
                K::Eof | K::Indent | K::Dedent => return None,
                K::P("(") => depth += 1,
                K::P(")") => {
                    depth -= 1;
                    if depth < 0 {
                        return None;
                    }
                }
                K::P("." | "!" | "?" | ";" | ":") if depth == 0 => return None,
                K::Word
                    if depth == 0
                        && i > self.pos
                        && t.kw == Some(Kw::Dla)
                        && self.tok(i + 1).kw == Some(Kw::Kazdy)
                        && !matches!(self.tok(i - 1).k, K::P(",") | K::P(":")) =>
                {
                    return (!self.comp_active.contains(&i)).then_some(i);
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// „x razy x dla każdego x od jednego do dziesięciu, które są parzyste” — regestr
    /// składany w locie. Głowa stoi przed rzeczą, którą nazywa, więc czyta się ją
    /// z mianem już ochrzczonym; jeśli się nie uda, to nie był regestr składany.
    fn try_comprehension(&mut self) -> R<Option<Expr>> {
        let Some(d) = self.find_comp_dla() else {
            return Ok(None);
        };
        let (start, line) = (self.pos, self.line());
        let mut after = d + 2;
        let mut name = Vec::new();
        while name_word_at(&self.toks, after) {
            name.push(self.tok(after).clone());
            after += 1;
        }
        if name.is_empty() {
            return Ok(None);
        }
        let (words, stems, display) = words_of(&name);
        let cname = self.new_cname(&display);
        let entry = || VarEntry {
            words: words.clone(),
            stems: stems.clone(),
            cname: cname.clone(),
        };
        self.comp_active.push(d);
        let head = self.with_scope(Some(entry()), |p| p.parse_or());
        self.comp_active.pop();
        let head = match head {
            Ok(h) if self.pos == d => h,
            _ => {
                self.pos = start;
                return Ok(None);
            }
        };
        self.hoist(&cname, &display);
        self.pos = after;
        let src = self.parse_comp_src()?;
        let el = Expr::Var(cname.clone());
        let filter = self.with_scope(Some(entry()), |p| p.parse_comp_filter(&el))?;
        Ok(Some(Expr::Comp {
            var: cname,
            src,
            map: Some(Box::new(head)),
            filter: filter.map(Box::new),
            line,
        }))
    }

    fn parse_comp_src(&mut self) -> R<CompSrc> {
        if self.eat_kw(Kw::Od) {
            let from = self.parse_add()?;
            self.expect_kw(Kw::Do, "ſłowá «do» (np. «od iednego do dzieśięćiu»)")?;
            let to = self.parse_add()?;
            let step = if self.eat_kw(Kw::Co) {
                Some(Box::new(self.parse_add()?))
            } else {
                None
            };
            return Ok(CompSrc::Range(Box::new(from), Box::new(to), step));
        }
        if self.eat_kw(Kw::Z) || self.eat_kw(Kw::W) {
            return Ok(CompSrc::Each(Box::new(self.parse_add()?)));
        }
        Err(self.expected("«od … do …» álbo «z …»"))
    }

    /// „wszystkie oceny, które są większe niż trzy”, „wszyscy żacy z listy, których
    /// średnia jest większa niż cztery”, „wszystkie liczby od 1 do 50 podzielne…”.
    fn parse_every(&mut self, line: usize) -> R<Expr> {
        let save = self.pos;
        let name = self.gather_name_words();
        let (src, entry_name) = if matches!(self.peek().kw, Some(Kw::Z | Kw::W | Kw::Od)) {
            (self.parse_comp_src()?, name)
        } else {
            self.pos = save;
            (CompSrc::Each(Box::new(self.parse_postfix()?)), Vec::new())
        };
        let (words, stems, display) = words_of(&entry_name);
        let cname = self.new_cname(if display.is_empty() { "ten" } else { &display });
        self.hoist(&cname, &display);
        let entry = (!words.is_empty()).then(|| VarEntry {
            words,
            stems,
            cname: cname.clone(),
        });
        let el = Expr::Var(cname.clone());
        let filter = self.with_scope(entry, |p| p.parse_comp_filter(&el))?;
        Ok(Expr::Comp {
            var: cname,
            src,
            map: None,
            filter: filter.map(Box::new),
            line,
        })
    }

    /// Warunek po przecinku: „które są …”, „których średnia …”, „gdzie …”, złączone „i”/„albo”.
    fn parse_comp_filter(&mut self, el: &Expr) -> R<Option<Expr>> {
        let save = self.pos;
        self.eat_p(",");
        let Some(mut cond) = self.parse_pred_clause(el)? else {
            self.pos = save;
            return Ok(None);
        };
        loop {
            let line = self.line();
            let or = self.is_kw(Kw::Albo);
            let next_ok = self.starts_operand(self.pos + 1)
                || matches!(self.kw_at(1), Some(Kw::Iest))
                || self.is_cmp_tail(1);
            if !(or || self.is_kw(Kw::I) || self.is_kw(Kw::Oraz))
                || !next_ok
                || self.starts_simple(self.pos + 1)
                || self.norm_at(1) == "a"
            {
                break;
            }
            self.pos += 1;
            let next = match self.parse_pred_clause(el)? {
                Some(c) => c,
                None if self.starts_bare_cmp() => self.parse_cmp_with(el.clone())?,
                None => self.parse_not()?,
            };
            cond = if or {
                Expr::Or(Box::new(cond), Box::new(next), line)
            } else {
                Expr::And(Box::new(cond), Box::new(next), line)
            };
        }
        Ok(Some(cond))
    }

    fn parse_pred_clause(&mut self, el: &Expr) -> R<Option<Expr>> {
        let line = self.line();
        let subject = match self.norm_at(0) {
            "ktori" | "ktora" | "ktore" | "ktorzi" => {
                self.pos += 1;
                el.clone()
            }
            "co" if matches!(self.kw_at(1), Some(Kw::Iest | Kw::Nie)) || self.is_cmp_tail(1) => {
                self.pos += 1;
                el.clone()
            }
            "ktorego" | "ktorei" | "ktorich" => {
                self.pos += 1;
                if self.eat_kw(Kw::Dlugosc) {
                    Expr::Un(UnOp::Len, Box::new(el.clone()), line)
                } else {
                    let (_, f) = self.resolve(self.pos)?;
                    let Some(Found {
                        kind: NK::Field(fid),
                        len,
                        ..
                    }) = f
                    else {
                        return Err(self.expected("miána polá (np. «których średniá ieſt …»)"));
                    };
                    self.pos += len;
                    Expr::Field(Box::new(el.clone()), fid, line)
                }
            }
            "gdzie" => {
                self.pos += 1;
                return Ok(Some(self.parse_not()?));
            }
            _ if self.is_kw(Kw::Jesli) => {
                self.pos += 1;
                return Ok(Some(self.parse_not()?));
            }
            _ => return Ok(None),
        };
        let before = self.pos;
        let e = self.parse_cmp_with(subject)?;
        if self.pos == before {
            return Err(self.expected("porównánia (np. «które ſą większe niż trzy»)"));
        }
        Ok(Some(e))
    }

    fn parse_or(&mut self) -> R<Expr> {
        let mut l = self.parse_and()?;
        while self.is_kw(Kw::Albo) && self.starts_operand(self.pos + 1) {
            let line = self.line();
            self.pos += 1;
            let r = self.parse_and()?;
            l = Expr::Or(Box::new(l), Box::new(r), line);
        }
        Ok(l)
    }

    fn parse_and(&mut self) -> R<Expr> {
        let mut l = self.parse_not()?;
        while (self.is_kw(Kw::I) || self.is_kw(Kw::Oraz))
            && self.starts_operand(self.pos + 1)
            && !self.starts_simple(self.pos + 1)
        {
            let line = self.line();
            self.pos += 1;
            let r = self.parse_not()?;
            l = Expr::And(Box::new(l), Box::new(r), line);
        }
        Ok(l)
    }

    fn parse_not(&mut self) -> R<Expr> {
        if self.is_kw(Kw::Nie) && self.kw_at(1) != Some(Kw::Iest) {
            let line = self.line();
            self.pos += 1;
            let e = self.parse_not()?;
            return Ok(Expr::Not(Box::new(e), line));
        }
        self.parse_cmp()
    }

    fn is_cmp_tail(&self, off: usize) -> bool {
        let t = self.tok(self.pos + off);
        if let K::P(p) = t.k {
            return ["=", "==", "!=", "<", ">", "<=", ">="].contains(&p);
        }
        matches!(
            t.kw,
            Some(
                Kw::Wiekszy
                    | Kw::Mniejszy
                    | Kw::Rowny
                    | Kw::Rozny
                    | Kw::Podzielny
                    | Kw::Parzysty
                    | Kw::Nieparzysty
                    | Kw::Dodatni
                    | Kw::Ujemny
                    | Kw::Zawiera
                    | Kw::Przewyzsza
                    | Kw::Ma
            )
        )
    }

    fn parse_cmp(&mut self) -> R<Expr> {
        let l = self.parse_add()?;
        self.parse_cmp_with(l)
    }

    /// Czy tu zaczyna się porównanie bez podmiotu — „á ieśli podzielna przez trzy”.
    fn starts_bare_cmp(&self) -> bool {
        self.is_kw(Kw::Iest)
            || (self.is_kw(Kw::Nie) && (self.kw_at(1) == Some(Kw::Iest) || self.is_cmp_tail(1)))
            || (self.is_cmp_tail(0) && !matches!(self.peek().k, K::P(_)))
    }

    fn parse_cmp_with(&mut self, l: Expr) -> R<Expr> {
        let line = self.line();
        let mut neg = false;
        let after_is;
        if self.is_kw(Kw::Iest) {
            self.pos += 1;
            neg = self.eat_kw(Kw::Nie);
            after_is = true;
        } else if self.is_kw(Kw::Nie) && self.kw_at(1) == Some(Kw::Iest) {
            self.pos += 2;
            neg = true;
            after_is = true;
        } else if self.is_kw(Kw::Nie) && self.is_cmp_tail(1) {
            self.pos += 1;
            neg = true;
            after_is = false;
        } else if self.is_cmp_tail(0) {
            after_is = false;
        } else {
            return Ok(l);
        }
        let e = self.cmp_tail(l, after_is, line)?;
        Ok(if neg { Expr::Not(Box::new(e), line) } else { e })
    }

    /// „iest pismem”, „iest liczbą całkowitą”, „iest Studentem” — rodzaj rzeczy.
    fn type_word(&mut self) -> Option<i32> {
        let (code, len) = match self.norm_at(0) {
            "pismem" | "napisem" | "tekstem" => (3, 1),
            "ulamkiem" => (2, 1),
            "regestrem" | "reiestrem" => (4, 1),
            "spisem" | "slownikiem" => (5, 1),
            "nicoscia" => (0, 1),
            "liczba" if self.norm_at(1) == "calkowita" => (7, 2),
            // „liczbą” — chyba że Waszmość ma rzecz zwaną dokładnie tak
            "liczba"
                if !self
                    .find_names(self.pos)
                    .iter()
                    .any(|f| f.exact && matches!(f.kind, NK::Var(_))) =>
            {
                (1, 1)
            }
            _ if self.is_kw(Kw::Prawda)
                && self.kw_at(1) == Some(Kw::Albo)
                && self.kw_at(2) == Some(Kw::Falsz) =>
            {
                (6, 3)
            }
            _ => match self.resolve(self.pos) {
                Ok((
                    Some(Found {
                        kind: NK::Rod(rid),
                        len,
                        ..
                    }),
                    _,
                )) => (100 + rid as i32, len),
                _ => return None,
            },
        };
        self.pos += len;
        Some(code)
    }

    fn cmp_tail(&mut self, l: Expr, after_is: bool, line: usize) -> R<Expr> {
        if after_is && let Some(code) = self.type_word() {
            return Ok(Expr::IsType(Box::new(l), code, line));
        }
        let bin = |op, l: Expr, r: Expr| Expr::Bin(op, Box::new(l), Box::new(r), line);
        let un = |op, l: Expr| Expr::Un(op, Box::new(l), line);
        if let K::P(p) = self.peek().k {
            let op = match p {
                "=" | "==" => Some(BinOp::Eq),
                "!=" => Some(BinOp::Ne),
                "<" => Some(BinOp::Lt),
                ">" => Some(BinOp::Gt),
                "<=" => Some(BinOp::Le),
                ">=" => Some(BinOp::Ge),
                _ => None,
            };
            if let Some(op) = op {
                self.pos += 1;
                let r = self.parse_add()?;
                return Ok(bin(op, l, r));
            }
        }
        let kw = self.peek().kw;
        match kw {
            Some(Kw::Wiekszy) | Some(Kw::Mniejszy) => {
                self.pos += 1;
                let or_equal = if self.is_kw(Kw::Albo) && self.kw_at(1) == Some(Kw::Rowny) {
                    self.pos += 2;
                    true
                } else {
                    false
                };
                // „większy albo równy dziesięć” — po „równy” samo „niż” bywa zbędne.
                if !(self.eat_kw(Kw::Niz) || self.eat_kw(Kw::Od) || or_equal) {
                    return Err(self.expected("«niż», «niźli» álbo «od»"));
                }
                let r = self.parse_add()?;
                let op = match (kw, or_equal) {
                    (Some(Kw::Wiekszy), false) => BinOp::Gt,
                    (Some(Kw::Wiekszy), true) => BinOp::Ge,
                    (_, false) => BinOp::Lt,
                    (_, true) => BinOp::Le,
                };
                Ok(bin(op, l, r))
            }
            Some(Kw::Przewyzsza) => {
                self.pos += 1;
                let r = self.parse_add()?;
                Ok(bin(BinOp::Gt, l, r))
            }
            Some(Kw::Rowny) => {
                self.pos += 1;
                self.eat_kw(Kw::Z);
                let r = self.parse_add()?;
                Ok(bin(BinOp::Eq, l, r))
            }
            Some(Kw::Rozny) => {
                self.pos += 1;
                self.eat_kw(Kw::Od);
                let r = self.parse_add()?;
                Ok(bin(BinOp::Ne, l, r))
            }
            Some(Kw::Co) if matches!(self.kw_at(1), Some(Kw::Najmniej | Kw::Najwyzej)) => {
                let op = if self.kw_at(1) == Some(Kw::Najmniej) {
                    BinOp::Ge
                } else {
                    BinOp::Le
                };
                self.pos += 2;
                let r = self.parse_add()?;
                Ok(bin(op, l, r))
            }
            Some(Kw::Podzielny) => {
                self.pos += 1;
                self.expect_kw(Kw::Przez, "ſłowá «przez»")?;
                let r = self.parse_add()?;
                Ok(bin(BinOp::Divisible, l, r))
            }
            Some(Kw::Parzysty) => {
                self.pos += 1;
                Ok(un(UnOp::Even, l))
            }
            Some(Kw::Nieparzysty) => {
                self.pos += 1;
                Ok(un(UnOp::Odd, l))
            }
            Some(Kw::Dodatni) => {
                self.pos += 1;
                Ok(un(UnOp::Positive, l))
            }
            Some(Kw::Ujemny) => {
                self.pos += 1;
                Ok(un(UnOp::Negative, l))
            }
            Some(Kw::Pusty) if after_is => {
                self.pos += 1;
                self.eat_kw(Kw::Regestr);
                Ok(un(UnOp::Empty, l))
            }
            Some(Kw::Zawiera) | Some(Kw::Ma) => {
                self.pos += 1;
                let r = self.parse_add()?;
                Ok(bin(BinOp::Contains, l, r))
            }
            Some(Kw::W) if after_is => {
                self.pos += 1;
                let r = self.parse_add()?;
                Ok(bin(BinOp::Contains, r, l))
            }
            _ if after_is => {
                let r = self.parse_add()?;
                Ok(bin(BinOp::Eq, l, r))
            }
            _ => Err(self.expected("porównánia")),
        }
    }

    fn parse_add(&mut self) -> R<Expr> {
        let mut l = self.parse_mul()?;
        loop {
            let op = if self.is_kw(Kw::Plus) || self.is_p("+") {
                BinOp::Add
            } else if self.is_kw(Kw::Minus) || self.is_p("-") {
                BinOp::Sub
            } else {
                break;
            };
            if !self.starts_operand(self.pos + 1) {
                break;
            }
            let line = self.line();
            self.pos += 1;
            let r = self.parse_mul()?;
            l = Expr::Bin(op, Box::new(l), Box::new(r), line);
        }
        // „rok rzymskimi”, „po rzymsku”, „suma słownie”
        loop {
            let line = self.line();
            let (op, len) = if self.norm_at(0) == "rzimskimi" {
                (UnOp::Roman, 1)
            } else if (self.norm_at(0) == "ciframi" && self.norm_at(1) == "rzimskimi")
                || (self.is_kw(Kw::Po) && self.norm_at(1) == "rzimsku")
            {
                (UnOp::Roman, 2)
            } else if self.norm_at(0) == "slownie" {
                (UnOp::Words, 1)
            } else {
                break;
            };
            self.pos += len;
            l = Expr::Un(op, Box::new(l), line);
        }
        Ok(l)
    }

    fn parse_mul(&mut self) -> R<Expr> {
        let mut l = self.parse_unary()?;
        loop {
            let line = self.line();
            let op =
                if (self.is_kw(Kw::Razy) || self.is_p("*")) && self.starts_operand(self.pos + 1) {
                    self.pos += 1;
                    BinOp::Mul
                } else if self.is_kw(Kw::Pomnozone) {
                    self.pos += 1;
                    self.expect_kw(Kw::Przez, "ſłowá «przez»")?;
                    BinOp::Mul
                } else if self.is_kw(Kw::Podzielone) {
                    self.pos += 1;
                    self.expect_kw(Kw::Przez, "ſłowá «przez»")?;
                    BinOp::Div
                } else if self.is_p("/") {
                    self.pos += 1;
                    BinOp::Div
                } else if self.is_kw(Kw::Modulo) || self.is_p("%") {
                    self.pos += 1;
                    BinOp::Mod
                } else if self.is_kw(Kw::Do) && self.norm_at(1) == "potegi" {
                    self.pos += 2;
                    BinOp::Pow
                } else if self.is_kw(Kw::Do)
                    && (self.norm_at(1) == "kwadratu" || self.norm_at(1) == "szescianu")
                {
                    let n = if self.norm_at(1) == "kwadratu" { 2 } else { 3 };
                    self.pos += 2;
                    l = Expr::Bin(BinOp::Pow, Box::new(l), Box::new(Expr::Int(n)), line);
                    continue;
                } else {
                    break;
                };
            let r = self.parse_unary()?;
            l = Expr::Bin(op, Box::new(l), Box::new(r), line);
        }
        Ok(l)
    }

    fn parse_unary(&mut self) -> R<Expr> {
        if (self.is_kw(Kw::Minus) || self.is_p("-")) && self.starts_operand(self.pos + 1) {
            let line = self.line();
            self.pos += 1;
            let e = self.parse_unary()?;
            return Ok(Expr::Un(UnOp::Neg, Box::new(e), line));
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> R<Expr> {
        let mut e = self.parse_primary()?;
        loop {
            let line = self.line();
            if self.is_kw(Kw::Wspak) {
                self.pos += 1;
            } else if (self.norm_at(0).starts_with("czitan") || self.is_kw(Kw::Na))
                && self.kw_at(1) == Some(Kw::Wspak)
            {
                // „czytane wspak”, „na wspak”
                self.pos += 2;
            } else if self.is_kw(Kw::Pod) && self.starts_operand_strict(self.pos + 1) {
                // „ceny pod „chleb”” — wartość ze spisu
                self.pos += 1;
                let key = self.parse_primary()?;
                e = Expr::Index(Box::new(e), Box::new(key), line);
                continue;
            } else {
                break;
            }
            e = Expr::Un(UnOp::Reverse, Box::new(e), line);
        }
        Ok(e)
    }

    fn parse_primary(&mut self) -> R<Expr> {
        let t = self.peek().clone();
        match &t.k {
            K::Int(n) => {
                self.pos += 1;
                Ok(Expr::Int(*n))
            }
            K::Float(f) => {
                self.pos += 1;
                Ok(Expr::Float(*f))
            }
            K::Str(parts) => {
                self.pos += 1;
                self.string_expr(parts)
            }
            K::P("(") => {
                self.pos += 1;
                let e = self.parse_expr()?;
                if !self.eat_p(")") {
                    return Err(self.expected("náwiáſu zámykáiącego „)”"));
                }
                Ok(e)
            }
            K::Word => self.parse_word_primary(),
            _ => Err(self.expected("wártości")),
        }
    }

    fn string_expr(&mut self, parts: &[StrPart]) -> R<Expr> {
        if let [StrPart::Lit(s)] = parts {
            return Ok(Expr::Str(s.clone()));
        }
        let mut out = Vec::new();
        for part in parts {
            match part {
                StrPart::Lit(s) => out.push(Expr::Str(s.clone())),
                StrPart::Expr(text, line, col) => {
                    let toks = lexer::lex_fragment(text, *line, *col)?;
                    let saved = std::mem::replace(&mut self.toks, toks);
                    let saved_pos = std::mem::replace(&mut self.pos, 0);
                    // Miejsca w `comp_active` liczą się w słowach zewnętrznych, nie we wstawce.
                    let saved_comp = std::mem::take(&mut self.comp_active);
                    let r = self.parse_expr().and_then(|e| {
                        if self.peek().k == K::Eof {
                            Ok(e)
                        } else {
                            Err(self.expected("końcá wſtáwki „}”"))
                        }
                    });
                    self.toks = saved;
                    self.pos = saved_pos;
                    self.comp_active = saved_comp;
                    out.push(r?);
                }
            }
        }
        Ok(Expr::Interp(out))
    }

    fn parse_number_words(&mut self) -> R<Expr> {
        let first = self.peek().clone();
        let too_big = || {
            err_at(
                &first,
                "Liczbá to większa niż wſzytkie ſkárby Rzeczypoſpolitey.",
            )
        };
        let mut total: i64 = 0;
        let mut group: i64 = 0;
        let mut stage = 4; // 3 setki, 2 dziesiątki, 1 jedności
        let mut power = i64::MAX; // ostatni rząd (tysiąc, milion…); następny musi być mniejszy
        // Czy przed „tysiącem” padła liczba: samo „tysiąc” to 1000, lecz „zero tysięcy” to 0.
        let mut named = false;
        while self.is_kw(Kw::Liczebnik) {
            let (v, class) = number(&self.peek().norm).unwrap();
            let ok = match class {
                NumClass::Hundred => stage > 3,
                NumClass::Ten => stage > 2,
                NumClass::Teen => stage > 2,
                NumClass::Unit => stage > 1,
                NumClass::Power => v < power,
            };
            if !ok {
                break;
            }
            match class {
                NumClass::Hundred => {
                    group += v;
                    stage = 3;
                    named = true;
                }
                NumClass::Ten => {
                    group += v;
                    stage = 2;
                    named = true;
                }
                NumClass::Teen | NumClass::Unit => {
                    group += v;
                    stage = 1;
                    named = true;
                }
                NumClass::Power => {
                    total = (if named { group } else { 1 })
                        .checked_mul(v)
                        .and_then(|x| x.checked_add(total))
                        .ok_or_else(too_big)?;
                    group = 0;
                    named = false;
                    stage = 4;
                    power = v;
                }
            }
            self.pos += 1;
        }
        let n = total.checked_add(group).ok_or_else(too_big)?;
        if self.is_kw(Kw::I) && self.kw_at(1) == Some(Kw::Pol) {
            self.pos += 2;
            return Ok(Expr::Float(n as f64 + 0.5));
        }
        Ok(Expr::Int(n))
    }

    fn parse_word_primary(&mut self) -> R<Expr> {
        let t = self.peek().clone();
        let line = t.line;
        // Wersalikowe „I” w miejscu wartości to rzymska jedynka, nie spójnik „i”.
        if t.kw == Some(Kw::I) && t.raw == "I" {
            self.pos += 1;
            return Ok(Expr::Int(1));
        }
        // „pierwsza liczba” bywa mianem rzeczy, nie zaś pierwszym elementem regestru „liczba”.
        let ordinal_name = matches!(t.kw, Some(Kw::Ordinal(_)))
            && matches!(
                self.resolve(self.pos),
                Ok((Some(Found { len: 2.., .. }), _))
            );
        if let Some(kw) = t.kw.filter(|_| !ordinal_name) {
            return match kw {
                Kw::Liczebnik => self.parse_number_words(),
                Kw::Pol => {
                    self.pos += 1;
                    Ok(Expr::Float(0.5))
                }
                Kw::Poltora => {
                    self.pos += 1;
                    Ok(Expr::Float(1.5))
                }
                Kw::Prawda => {
                    self.pos += 1;
                    Ok(Expr::Bool(true))
                }
                Kw::Falsz => {
                    self.pos += 1;
                    Ok(Expr::Bool(false))
                }
                Kw::Nic => {
                    self.pos += 1;
                    Ok(Expr::Nic)
                }
                Kw::Element => {
                    self.pos += 1;
                    // „element drugi z ocen” — to samo, co „drugi element ocen”
                    if matches!(self.peek().kw, Some(Kw::Ordinal(_))) {
                        return self.parse_primary();
                    }
                    let idx = self.parse_add()?;
                    let _ = self.eat_kw(Kw::Z) || self.eat_kw(Kw::W);
                    let coll = self.parse_postfix()?;
                    Ok(Expr::Index(Box::new(coll), Box::new(idx), line))
                }
                Kw::Ordinal(n) => {
                    self.pos += 1;
                    self.eat_kw(Kw::Element);
                    let _ = self.eat_kw(Kw::Z) || self.eat_kw(Kw::W);
                    let coll = self.parse_postfix()?;
                    Ok(if n < 0 {
                        Expr::Last(Box::new(coll), line)
                    } else {
                        Expr::Index(Box::new(coll), Box::new(Expr::Int(n)), line)
                    })
                }
                Kw::Dlugosc => {
                    self.pos += 1;
                    self.eat_kw(Kw::Z);
                    let e = self.parse_postfix()?;
                    Ok(Expr::Un(UnOp::Len, Box::new(e), line))
                }
                Kw::Regestr => {
                    self.pos += 1;
                    self.parse_list_rest(line)
                }
                Kw::Spis => {
                    self.pos += 1;
                    self.parse_dict_rest(line)
                }
                Kw::Kazdy => {
                    self.pos += 1;
                    self.parse_every(line)
                }
                Kw::Plik => {
                    // „plik „kronika.txt” istnieje”
                    self.pos += 1;
                    let path = self.parse_postfix()?;
                    let neg = self.is_kw(Kw::Nie) && self.norm_at(1) == "istnieie";
                    if !neg && self.norm_at(0) != "istnieie" {
                        return Err(self
                            .expected("«iſtnieie» po miániu pliku (np. «plik „a.txt” iſtnieie»)"));
                    }
                    self.pos += 1 + usize::from(neg);
                    let e = Expr::Builtin("sw_b_istnieje", vec![path], line);
                    Ok(if neg { Expr::Not(Box::new(e), line) } else { e })
                }
                Kw::Pusty => {
                    self.pos += 1;
                    if self.eat_kw(Kw::Regestr) {
                        return Ok(Expr::List(Vec::new()));
                    }
                    if self.eat_kw(Kw::Spis) {
                        return Ok(Expr::Dict(Vec::new(), line));
                    }
                    if self.norm_at(0).starts_with("pism") {
                        self.pos += 1;
                        return Ok(Expr::Str(String::new()));
                    }
                    Err(self.expected("«regeſtr» álbo «piſmo» po «puſty»"))
                }
                Kw::Nowy => {
                    self.pos += 1;
                    if self.eat_kw(Kw::Regestr) {
                        return Ok(Expr::List(Vec::new()));
                    }
                    if self.eat_kw(Kw::Spis) {
                        return Ok(Expr::Dict(Vec::new(), line));
                    }
                    self.parse_new(line)
                }
                _ => Err(err_at(
                    &t,
                    format!(
                        "Słowo „{}” tu nie przyſtoi — ſpodziewáłem ſię wártości.",
                        t.raw
                    ),
                )),
            };
        }

        // „istnieje plik „kronika.txt””
        if t.norm == "istnieie" && self.kw_at(1) == Some(Kw::Plik) {
            self.pos += 2;
            let path = self.parse_postfix()?;
            return Ok(Expr::Builtin("sw_b_istnieje", vec![path], line));
        }
        // „klucze z cen”, „wartości z cen” — o ile Waszmość nie ma rzeczy tak zwanej
        if (t.norm == "klucze" || t.norm == "wartosci")
            && matches!(self.kw_at(1), Some(Kw::Z | Kw::W))
            && !matches!(
                self.resolve(self.pos),
                Ok((
                    Some(Found {
                        kind: NK::Var(_),
                        exact: true,
                        ..
                    }),
                    _
                ))
            )
        {
            self.pos += 2;
            let x = self.parse_postfix()?;
            let f = if t.norm == "klucze" {
                "sw_b_klucze"
            } else {
                "sw_b_wartosci"
            };
            return Ok(Expr::Builtin(f, vec![x], line));
        }
        // „reszta z dzielenia a przez b”
        if t.norm.starts_with("reszt")
            && self.kw_at(1) == Some(Kw::Z)
            && self.norm_at(2) == "dzielenia"
        {
            self.pos += 3;
            let a = self.parse_unary()?;
            self.expect_kw(Kw::Przez, "ſłowá «przez»")?;
            let b = self.parse_unary()?;
            return Ok(Expr::Bin(BinOp::Mod, Box::new(a), Box::new(b), line));
        }

        let (other, field) = self.resolve(self.pos)?;
        if let Some(f) = &field {
            let other_len = other.as_ref().map_or(0, |o| o.len);
            if f.len >= other_len && self.starts_operand_strict(self.pos + f.len) {
                let NK::Field(fid) = f.kind else {
                    unreachable!()
                };
                self.pos += f.len;
                let obj = self.parse_postfix()?;
                return Ok(Expr::Field(Box::new(obj), fid, line));
            }
        }
        if let Some(o) = other {
            return match o.kind {
                NK::Var(c) => {
                    self.pos += o.len;
                    Ok(Expr::Var(c))
                }
                NK::Func(i) => self.parse_call(i, o.len, line),
                NK::Rod(_) => Err(err_at(
                    &t,
                    "Ród ſtwárza ſię ſłowem «nowy», np. «nowy Student o miániu „Jan”».",
                )),
                NK::Field(_) => unreachable!(),
            };
        }
        if field.is_some() {
            return Err(err_at(
                &t,
                format!(
                    "Pole „{}” — ále czyie? Po miániu polá przyſtoi rzecz, np. «ocená ſtudentá».",
                    t.raw
                ),
            ));
        }
        if let Some(n) = roman(&t.raw) {
            self.pos += 1;
            return Ok(Expr::Int(n));
        }
        Err(err_at(
            &t,
            format!(
                "Słowo „{}” obce ieſt ſkrybie. Nie ochrzcono go przez «Niechay będźie …» ani nie ieſt to vczynek.",
                t.raw
            ),
        ))
    }

    fn parse_call(&mut self, fidx: usize, name_len: usize, line: usize) -> R<Expr> {
        let name_tok = self.peek().clone();
        self.pos += name_len;
        let arity = self.funcs[fidx].arity;
        let mut args = Vec::new();
        if arity > 0 {
            let prep = matches!(
                self.peek().kw,
                Some(Kw::Z | Kw::Dla | Kw::Od | Kw::O | Kw::Na)
            );
            if prep {
                self.pos += 1;
            } else if !self.starts_operand_strict(self.pos) {
                return Err(err_at(
                    &name_tok,
                    format!(
                        "Vczynek „{}” żąda {} rzeczy, á nie dano mu żadney (np. «{} z n»).",
                        self.funcs[fidx].display, arity, self.funcs[fidx].display
                    ),
                ));
            }
            for i in 0..arity {
                if i > 0 {
                    let sep = self.eat_p(",")
                        || self.eat_kw(Kw::I)
                        || self.eat_kw(Kw::Oraz)
                        || self.funcs[fidx]
                            .seps
                            .clone()
                            .into_iter()
                            .any(|k| self.eat_kw(k));
                    if !sep {
                        return Err(err_at(
                            &name_tok,
                            format!(
                                "Vczynek „{}” żąda {} rzeczy, á dano mu ledwie {}.",
                                self.funcs[fidx].display, arity, i
                            ),
                        ));
                    }
                }
                // „złączenie miano człeka dla każdego człeka z gości przez „, ””
                let arg = match self.try_comprehension()? {
                    Some(e) => e,
                    None => self.parse_add()?,
                };
                args.push(arg);
            }
        }
        let f = &self.funcs[fidx];
        if f.builtin {
            let cfn = BUILTINS.iter().find(|b| b.2 == f.cname).unwrap().2;
            Ok(Expr::Builtin(cfn, args, line))
        } else {
            Ok(Expr::Call(fidx - BUILTINS.len(), args))
        }
    }

    fn parse_list_rest(&mut self, line: usize) -> R<Expr> {
        if self.is_kw(Kw::O) && self.kw_at(1) == Some(Kw::Dlugosc) {
            self.pos += 2;
            let n = self.parse_add()?;
            let _ = self.eat_p(",");
            if !(self.eat_kw(Kw::Pelen) || self.eat_kw(Kw::Z)) {
                return Err(self.expected("«pełen» (np. «regeſtr o długośći pięćiu pełen zer»)"));
            }
            let x = self.parse_add()?;
            return Ok(Expr::Fill(Box::new(n), Box::new(x), line));
        }
        if self.eat_kw(Kw::Pusty) || !self.eat_kw(Kw::Z) {
            return Ok(Expr::List(Vec::new()));
        }
        let mut items = vec![self.parse_add()?];
        loop {
            let save = self.pos;
            if !(self.eat_p(",") || self.eat_kw(Kw::I) || self.eat_kw(Kw::Oraz)) {
                break;
            }
            if self.starts_verb(self.pos)
                || !self.starts_operand(self.pos)
                || self.norm_at(0) == "a"
            {
                self.pos = save;
                break;
            }
            items.push(self.parse_add()?);
        }
        Ok(Expr::List(items))
    }

    /// „spis, gdzie „chleb” ma trzy, a „miód” ma pięć” albo sam „spis” (pusty).
    fn parse_dict_rest(&mut self, line: usize) -> R<Expr> {
        if self.eat_kw(Kw::Pusty) {
            return Ok(Expr::Dict(Vec::new(), line));
        }
        let save = self.pos;
        self.eat_p(",");
        let intro = if self.norm_at(0) == "gdzie" || self.is_kw(Kw::Z) {
            1
        } else if self.is_kw(Kw::W) && self.norm_at(1) == "ktorim" {
            2
        } else {
            0
        };
        if intro == 0 {
            self.pos = save;
            return Ok(Expr::Dict(Vec::new(), line));
        }
        self.pos += intro;
        let mut pairs = Vec::new();
        loop {
            let k = self.parse_add()?;
            if !(self.eat_kw(Kw::Ma) || self.eat_kw(Kw::Rowny)) {
                return Err(
                    self.expected("«ma» między kluczem á wártośćią (np. «„chleb” ma trzy»)")
                );
            }
            let v = self.parse_add()?;
            pairs.push((k, v));
            // „…, „miód” ma pięć, a „wino” ma dwanaście”
            let save = self.pos;
            let comma = self.eat_p(",");
            let conj = self.eat_kw(Kw::I) || self.eat_kw(Kw::Oraz);
            if !conj && self.norm_at(0) == "a" {
                self.pos += 1;
            } else if !comma && !conj {
                break;
            }
            if self.starts_verb(self.pos) || !self.starts_operand(self.pos) {
                self.pos = save;
                break;
            }
        }
        Ok(Expr::Dict(pairs, line))
    }

    fn parse_new(&mut self, line: usize) -> R<Expr> {
        let found = self.find_names(self.pos);
        let Some(r) = found
            .into_iter()
            .filter(|f| matches!(f.kind, NK::Rod(_)))
            .max_by_key(|f| f.len)
        else {
            return Err(self.expected("miána rodu po «nowy»"));
        };
        let NK::Rod(rid) = r.kind else { unreachable!() };
        self.pos += r.len;
        let _ = self.eat_kw(Kw::O) || self.eat_kw(Kw::Z) || self.eat_p(",");
        let mut fields = Vec::new();
        loop {
            let save = self.pos;
            let matched = self.rod_fields[rid]
                .iter()
                .filter_map(|&fid| {
                    let f = &self.fields[fid];
                    self.match_at(self.pos, &f.words, &f.stems, false)
                        .map(|_| (fid, f.words.len()))
                })
                .max_by_key(|x| x.1);
            let Some((fid, len)) = matched else {
                if fields.is_empty() && self.stray_field_word() {
                    return Err(self.unknown_field(rid));
                }
                self.pos = save;
                break;
            };
            self.pos += len;
            let _ = self.eat_kw(Kw::Rowny) || self.eat_kw(Kw::Iako);
            fields.push((fid, self.parse_add()?));
            let save = self.pos;
            if !(self.eat_p(",") || self.eat_kw(Kw::I) || self.eat_kw(Kw::Oraz)) {
                break;
            }
            let next_is_field = self.rod_fields[rid].iter().any(|&fid| {
                let f = &self.fields[fid];
                self.match_at(self.pos, &f.words, &f.stems, false).is_some()
            });
            if !next_is_field {
                if self.stray_field_word() {
                    return Err(self.unknown_field(rid));
                }
                self.pos = save;
                break;
            }
        }
        Ok(Expr::New(rid, fields, line))
    }

    /// Słowo, które wygląda na pole, a nie jest niczym znanym skrybie.
    fn stray_field_word(&self) -> bool {
        let t = self.peek();
        is_name_word(t) && t.norm != "a" && self.find_names(self.pos).is_empty()
    }

    fn unknown_field(&self, rid: usize) -> CompileError {
        let pola: Vec<&str> = self.rod_fields[rid]
            .iter()
            .map(|&f| self.fields[f].display.as_str())
            .collect();
        self.err_here(format!(
            "Ród {} nie ma polá „{}”. Ma te: {}. Iżby odmiáná zádziáłáłá, pole i iego formá muſzą mieć wſpólny rdzeń.",
            self.rods[rid].display,
            self.peek().raw,
            pola.join(", ")
        ))
    }
}
