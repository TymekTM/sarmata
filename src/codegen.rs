//! Przekład drzewa na język C. Bieg (runtime) dokleja się na początku.

use std::collections::HashMap;
use std::fmt::Write;

use crate::ast::*;

const RUNTIME: &str = include_str!("../runtime/sarmata.c");
const TABLES_MARK: &str = "/*@@TABLICE@@*/";

/// Początek pisma Waszmości w wydanym C — po nim nie ma już biegu.
pub const PROGRAM_MARK: &str = "\n/* ─────────── pismo Waszmości ─────────── */\n";

/// `pisma` to miana pism (główne i powołane księgi) do obwieszczeń grzechów biegu.
/// `zrodla`, jeśli podane, to treści tych pism: wtedy nad każdym zdaniem w C staje
/// w komentarzu zdanie z pisma (przekład).
pub fn generate(prog: &Program, pisma: &[String], zrodla: Option<&[String]>) -> String {
    let mut g = Gen {
        prog,
        lits: Vec::new(),
        lit_ids: HashMap::new(),
        tmp: 0,
        proby: 0,
        petle: Vec::new(),
        zrodla: zrodla.map(|z| z.iter().map(|s| s.lines().collect()).collect()),
    };

    let mut funcs = String::new();
    for f in &prog.funcs {
        g.function(&mut funcs, f);
    }
    let mut main = String::new();
    g.block(&mut main, &prog.main, 1);

    let mut out = String::new();
    let (head, tail) = RUNTIME
        .split_once(TABLES_MARK)
        .expect("znacznik tablic w biegu");
    out.push_str(head);
    out.push_str(&g.tables());
    let _ = writeln!(
        out,
        "#define SW_WIERSZE_PISMA {}\nstatic const char *const sw_pisma[] = {{{}}};",
        crate::error::WIERSZE_PISMA,
        pisma
            .iter()
            .map(|p| c_string(p))
            .collect::<Vec<_>>()
            .join(", ")
    );
    out.push_str(tail);

    out.push_str(PROGRAM_MARK);
    out.push('\n');
    let _ = writeln!(out, "static W sw_lit[{}];", g.lits.len().max(1));
    for (c, display) in &prog.globals {
        let _ = writeln!(out, "static W {c}; /* {} */", comment(display));
    }
    for s in &prog.sigs {
        let _ = writeln!(out, "static W {}({});", s.cname, params_decl(s.arity, None));
    }
    out.push('\n');
    out.push_str(&funcs);
    out.push_str("int main(int argc, char **argv) {\n    sw_start(argc, argv);\n");
    for (i, s) in g.lits.iter().enumerate() {
        let _ = writeln!(
            out,
            "    sw_lit[{i}] = sw_pismo_n({}, {});",
            c_string(s),
            s.len()
        );
    }
    out.push_str(&main);
    out.push_str("    sw_koniec();\n    return 0;\n}\n");
    out
}

/// Nagłówek C pętli `od a do b co st` (bez klamry). Krok czyni się tylko, gdy zostało doń
/// miejsca, by u granicy liczby („do 9223372036854775807”) licznik nie przewinął się na
/// ujemne i pętla nie kręciła się bez końca.
fn range_for(k: &str, ok: &str, a: &str, b: &str, st: &str) -> String {
    let u = "(unsigned long long)";
    format!(
        "for (long long {k} = {a}, {ok} = {st} > 0 ? {a} <= {b} : {a} >= {b}; {ok}; \
         {ok} = {st} > 0 ? {u}{b} - {u}{k} >= {u}{st} : {u}{k} - {u}{b} >= 0ULL - {u}{st}, \
         {k} = {ok} ? {k} + {st} : {k})"
    )
}

fn comment(s: &str) -> String {
    s.replace("*/", "* /")
}

fn params_decl(arity: usize, names: Option<&[String]>) -> String {
    if arity == 0 {
        return "void".to_string();
    }
    (0..arity)
        .map(|i| match names {
            Some(n) => format!("W {}", n[i]),
            None => "W".to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn c_string(s: &str) -> String {
    let mut out = String::from("\"");
    for b in s.bytes() {
        match b {
            b'"' => out.push_str("\\\""),
            b'\\' => out.push_str("\\\\"),
            b'?' => out.push_str("\\?"),
            0x20..=0x7e => out.push(b as char),
            _ => {
                let _ = write!(out, "\\{:03o}", b);
            }
        }
    }
    out.push('"');
    out
}

struct Gen<'a> {
    prog: &'a Program,
    lits: Vec<String>,
    lit_ids: HashMap<String, usize>,
    tmp: usize,
    /// Ile «Spróbuj» otwartych w bieżącym uczynku — tyle trzeba zdjąć przy «Zwróć».
    proby: usize,
    /// Głębokość prób przy wejściu w każdą otwartą pętlę (dla «Przerwij»/«Dalej»).
    petle: Vec<usize>,
    zrodla: Option<Vec<Vec<&'a str>>>,
}

/// Miejsce rozłożone na: przygotowanie, odczyt i wzór zapisu (`@@` = nowa wartość).
struct PlaceCode {
    prelude: Vec<String>,
    get: String,
    set: String,
}

fn has_try(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|s| match s {
        Stmt::Try { .. } => true,
        Stmt::If(arms, els, _) => {
            arms.iter().any(|(_, b)| has_try(b)) || els.as_deref().is_some_and(has_try)
        }
        Stmt::While(_, b, _) | Stmt::Forever(b) | Stmt::Repeat(_, b, _) => has_try(b),
        Stmt::ForRange { body, .. } | Stmt::ForEach { body, .. } => has_try(body),
        _ => false,
    })
}

impl Gen<'_> {
    fn tables(&self) -> String {
        let p = self.prog;
        let mut s = String::from("/* rody i pola */\n");
        for (i, r) in p.rods.iter().enumerate() {
            let ids: Vec<String> = r.fields.iter().map(|f| f.to_string()).collect();
            let _ = writeln!(
                s,
                "static const int sw_rod_pola_{i}[] = {{{}}};",
                ids.join(", ")
            );
        }
        let n = p.rods.len();
        let npol: Vec<String> = p.rods.iter().map(|r| r.fields.len().to_string()).collect();
        let pola: Vec<String> = (0..n).map(|i| format!("sw_rod_pola_{i}")).collect();
        let nazwy: Vec<String> = p.rods.iter().map(|r| c_string(&r.display)).collect();
        let pole_nazwy: Vec<String> = p.fields.iter().map(|f| c_string(f)).collect();
        let or0 = |v: Vec<String>, zero: &str| {
            if v.is_empty() {
                zero.to_string()
            } else {
                v.join(", ")
            }
        };
        let _ = writeln!(
            s,
            "static const int sw_rod_npol[] = {{{}}};",
            or0(npol, "0")
        );
        let _ = writeln!(
            s,
            "static const int *const sw_rod_pola[] = {{{}}};",
            or0(pola, "0")
        );
        let _ = writeln!(
            s,
            "static const char *const sw_rod_nazwy[] = {{{}}};",
            or0(nazwy, "\"\"")
        );
        let _ = writeln!(
            s,
            "static const char *const sw_pole_nazwy[] = {{{}}};",
            or0(pole_nazwy, "\"\"")
        );
        s
    }

    fn lit(&mut self, s: &str) -> String {
        let id = match self.lit_ids.get(s) {
            Some(&i) => i,
            None => {
                self.lits.push(s.to_string());
                self.lit_ids.insert(s.to_string(), self.lits.len() - 1);
                self.lits.len() - 1
            }
        };
        format!("sw_lit[{id}]")
    }

    fn fresh(&mut self) -> String {
        self.tmp += 1;
        format!("_t{}", self.tmp)
    }

    fn function(&mut self, out: &mut String, f: &FuncDef) {
        let sig = &self.prog.sigs[f.index];
        let _ = writeln!(out, "/* uczynek „{}” */", comment(&sig.display));
        // Po longjmp zmienne nie-volatile mogą mieć wartości z sufitu.
        let vol = if has_try(&f.body) { "volatile " } else { "" };
        let params: Vec<String> = f.params.iter().map(|p| format!("{vol}W {p}")).collect();
        let _ = writeln!(
            out,
            "static W {}({}) {{",
            sig.cname,
            if params.is_empty() {
                "void".to_string()
            } else {
                params.join(", ")
            }
        );
        for (c, display) in &f.locals {
            let _ = writeln!(out, "    {vol}W {c} = SW_NIC; /* {} */", comment(display));
        }
        self.proby = 0;
        self.block(out, &f.body, 1);
        out.push_str("    return SW_NIC;\n}\n\n");
    }

    fn block(&mut self, out: &mut String, stmts: &[Stmt], depth: usize) {
        for s in stmts {
            self.stmt(out, s, depth);
        }
    }

    fn args(&mut self, items: &[Expr]) -> String {
        items
            .iter()
            .map(|e| self.expr(e))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn array(&mut self, items: &[Expr]) -> String {
        if items.is_empty() {
            "NULL".to_string()
        } else {
            format!("(W[]){{{}}}", self.args(items))
        }
    }

    fn expr(&mut self, e: &Expr) -> String {
        match e {
            Expr::Int(n) => format!("sw_liczba({n}LL)"),
            Expr::Float(f) => format!("sw_ulamek({f:?})"),
            Expr::Str(s) => self.lit(s),
            Expr::Interp(parts) => format!("sw_zlacz({}, {})", parts.len(), self.array(parts)),
            Expr::Bool(b) => format!("sw_sad({})", *b as i32),
            Expr::Nic => "SW_NIC".to_string(),
            Expr::Var(c) => c.clone(),
            Expr::Bin(op, a, b, l) => {
                let f = match op {
                    BinOp::Add => "sw_dodac",
                    BinOp::Sub => "sw_odjac",
                    BinOp::Mul => "sw_razy",
                    BinOp::Div => "sw_przez",
                    BinOp::Mod => "sw_modulo",
                    BinOp::Pow => "sw_potega",
                    BinOp::Eq => "sw_rowne",
                    BinOp::Ne => "sw_rozne",
                    BinOp::Lt => "sw_mniejsze",
                    BinOp::Gt => "sw_wieksze",
                    BinOp::Le => "sw_niewieksze",
                    BinOp::Ge => "sw_niemniejsze",
                    BinOp::Divisible => "sw_podzielne",
                    BinOp::Contains => "sw_zawiera",
                };
                let (a, b) = (self.expr(a), self.expr(b));
                format!("{f}({a}, {b}, {l})")
            }
            Expr::Un(op, a, l) => {
                let f = match op {
                    UnOp::Neg => "sw_przeciwne",
                    UnOp::Even => "sw_parzyste",
                    UnOp::Odd => "sw_nieparzyste",
                    UnOp::Empty => "sw_puste",
                    UnOp::Positive => "sw_dodatnie",
                    UnOp::Negative => "sw_ujemne",
                    UnOp::Reverse => "sw_wspak",
                    UnOp::Len => "sw_dlugosc",
                    UnOp::Roman => "sw_rzymskie",
                    UnOp::Words => "sw_slownie",
                };
                format!("{f}({}, {l})", self.expr(a))
            }
            Expr::Not(a, l) => format!("sw_sad(!sw_prawdziwe({}, {l}))", self.expr(a)),
            Expr::And(a, b, l) => {
                let (a, b) = (self.expr(a), self.expr(b));
                format!("sw_sad(sw_prawdziwe({a}, {l}) && sw_prawdziwe({b}, {l}))")
            }
            Expr::Or(a, b, l) => {
                let (a, b) = (self.expr(a), self.expr(b));
                format!("sw_sad(sw_prawdziwe({a}, {l}) || sw_prawdziwe({b}, {l}))")
            }
            Expr::Call(i, args) => format!("{}({})", self.prog.sigs[*i].cname, self.args(args)),
            Expr::Builtin(f, args, l) if args.is_empty() => format!("{f}({l})"),
            Expr::Builtin(f, args, l) => format!("{f}({}, {l})", self.args(args)),
            Expr::IsType(a, code, l) => format!("sw_jest_typu({}, {code}, {l})", self.expr(a)),
            Expr::Dict(pairs, l) => {
                if pairs.is_empty() {
                    "sw_spis_nowy()".to_string()
                } else {
                    let flat: Vec<Expr> = pairs
                        .iter()
                        .flat_map(|(k, v)| [k.clone(), v.clone()])
                        .collect();
                    format!("sw_spis_z({}, {}, {l})", pairs.len(), self.array(&flat))
                }
            }
            Expr::Comp {
                var,
                src,
                map,
                filter,
                line: l,
            } => self.comprehension(var, src, map.as_deref(), filter.as_deref(), *l),
            Expr::Index(c, i, l) => {
                let (c, i) = (self.expr(c), self.expr(i));
                format!("sw_element({c}, {i}, {l})")
            }
            Expr::Last(c, l) => format!("sw_ostatni({}, {l})", self.expr(c)),
            Expr::Field(o, f, l) => format!("sw_pole({}, {f}, {l})", self.expr(o)),
            Expr::List(items) => {
                if items.is_empty() {
                    "sw_regestr_nowy()".to_string()
                } else {
                    format!("sw_regestr_z({}, {})", items.len(), self.array(items))
                }
            }
            Expr::Fill(n, x, l) => {
                let (r, k, cnt) = (self.fresh(), self.fresh(), self.fresh());
                let n = self.expr(n);
                let x = self.expr(x);
                format!(
                    "({{ W {r} = sw_regestr_nowy(); long long {cnt} = sw_calk({n}, {l}); \
                     for (long long {k} = 0; {k} < {cnt}; {k}++) sw_wloz({r}, {x}); {r}; }})"
                )
            }
            Expr::New(rod, fields, l) => {
                if fields.is_empty() {
                    format!("sw_rod_nowy({rod}, 0, NULL, NULL, {l})")
                } else {
                    let ids: Vec<String> = fields.iter().map(|(f, _)| f.to_string()).collect();
                    let vals: Vec<Expr> = fields.iter().map(|(_, e)| e.clone()).collect();
                    format!(
                        "sw_rod_nowy({rod}, {}, (int[]){{{}}}, {}, {l})",
                        fields.len(),
                        ids.join(", "),
                        self.array(&vals)
                    )
                }
            }
        }
    }

    fn comprehension(
        &mut self,
        var: &str,
        src: &CompSrc,
        map: Option<&Expr>,
        filter: Option<&Expr>,
        l: usize,
    ) -> String {
        let r = self.fresh();
        let head = match src {
            CompSrc::Each(coll) => {
                let (c, k) = (self.fresh(), self.fresh());
                let ce = self.expr(coll);
                format!(
                    "W {c} = {ce}; for (long long {k} = 0; {k} < sw_iter_dl({c}, {l}); {k}++) {{ {var} = sw_iter_el({c}, {k}, {l});"
                )
            }
            CompSrc::Range(from, to, step) => {
                let (a, b, st, k) = (self.fresh(), self.fresh(), self.fresh(), self.fresh());
                let fe = self.expr(from);
                let te = self.expr(to);
                let se = match step {
                    Some(s) => format!("sw_calk({}, {l})", self.expr(s)),
                    None => match (&**from, &**to) {
                        (Expr::Int(x), Expr::Int(y)) if x > y => "-1".to_string(),
                        _ => "1".to_string(),
                    },
                };
                format!(
                    "long long {a} = sw_calk({fe}, {l}), {b} = sw_calk({te}, {l}), {st} = {se}; \
                     if ({st} == 0) sw_grzech({l}, \"Krok równy zeru — regestr nie miałby końcá.\"); \
                     {} {{ {var} = sw_liczba({k});",
                    range_for(&k, &self.fresh(), &a, &b, &st)
                )
            }
        };
        let item = map.map_or_else(|| var.to_string(), |m| self.expr(m));
        let put = match filter {
            Some(f) => format!(
                "if (sw_prawdziwe({}, {l})) sw_wloz({r}, {item});",
                self.expr(f)
            ),
            None => format!("sw_wloz({r}, {item});"),
        };
        format!("({{ W {r} = sw_regestr_nowy(); {head} {put} }} {r}; }})")
    }

    /// Zdanie z pisma w komentarzu nad jego przekładem.
    fn mark(&self, out: &mut String, ind: &str, first: usize, last: usize) {
        let Some(zrodla) = &self.zrodla else {
            return;
        };
        let w = crate::error::WIERSZE_PISMA;
        let (id, a, b) = (first / w, first % w, last % w);
        let Some(lines) = zrodla.get(id) else {
            return;
        };
        let text: Vec<&str> = (a.max(1)..=b.max(a))
            .filter_map(|i| lines.get(i - 1))
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        if !text.is_empty() {
            let _ = writeln!(out, "{ind}/* {} */", comment(&text.join(" ")));
        }
    }

    /// Ile prób zdjąć przy wyjściu z pętli przez «Przerwij» albo «Dalej».
    fn unwind_loop(&self) -> String {
        let k = self.proby - self.petle.last().copied().unwrap_or(0);
        if k > 0 {
            format!("sw_proby_n -= {k}; ")
        } else {
            String::new()
        }
    }

    fn loop_body(&mut self, out: &mut String, body: &[Stmt], d: usize) {
        self.petle.push(self.proby);
        self.block(out, body, d);
        self.petle.pop();
    }

    fn place(&mut self, p: &Place) -> PlaceCode {
        match p {
            Place::Var(c) => PlaceCode {
                prelude: vec![],
                get: c.clone(),
                set: format!("{c} = @@;"),
            },
            Place::Index(c, i, l) => {
                let (tc, ti) = (self.fresh(), self.fresh());
                let (ce, ie) = (self.expr(c), self.expr(i));
                PlaceCode {
                    prelude: vec![format!("W {tc} = {ce};"), format!("W {ti} = {ie};")],
                    get: format!("sw_element({tc}, {ti}, {l})"),
                    set: format!("sw_ustaw({tc}, {ti}, @@, {l});"),
                }
            }
            Place::Last(c, l) => {
                let tc = self.fresh();
                let ce = self.expr(c);
                PlaceCode {
                    prelude: vec![format!("W {tc} = {ce};")],
                    get: format!("sw_ostatni({tc}, {l})"),
                    set: format!("sw_ustaw_ostatni({tc}, @@, {l});"),
                }
            }
            Place::Field(o, f, l) => {
                let to = self.fresh();
                let oe = self.expr(o);
                PlaceCode {
                    prelude: vec![format!("W {to} = {oe};")],
                    get: format!("sw_pole({to}, {f}, {l})"),
                    set: format!("sw_ustaw_pole({to}, {f}, @@, {l});"),
                }
            }
        }
    }

    fn stmt(&mut self, out: &mut String, s: &Stmt, d: usize) {
        let ind = "    ".repeat(d);
        let line = |out: &mut String, text: &str| {
            out.push_str(&ind);
            out.push_str(text);
            out.push('\n');
        };
        match s {
            Stmt::Assign(p, v) => {
                let pc = self.place(p);
                let v = self.expr(v);
                if pc.prelude.is_empty() {
                    line(out, &pc.set.replace("@@", &v));
                } else {
                    let tv = self.fresh();
                    line(
                        out,
                        &format!(
                            "{{ {} W {tv} = {v}; {} }}",
                            pc.prelude.join(" "),
                            pc.set.replace("@@", &tv)
                        ),
                    );
                }
            }
            Stmt::Compound(p, op, v, l) => {
                let f = match op {
                    CompoundOp::Add => "sw_dodac",
                    CompoundOp::Sub => "sw_odjac",
                    CompoundOp::Mul => "sw_razy",
                    CompoundOp::Div => "sw_przez",
                    CompoundOp::Put => "sw_dodaj",
                };
                let pc = self.place(p);
                let v = self.expr(v);
                let tv = self.fresh();
                // Klucza w spisie może jeszcze nie być: wtedy zaczyna się od zera.
                let get = if matches!(p, Place::Index(..)) {
                    format!("sw_zaczatek({}, {tv}, {})", pc.get, *op as i32)
                } else {
                    pc.get
                };
                let newv = format!("{f}({get}, {tv}, {l})");
                line(
                    out,
                    &format!(
                        "{{ {} W {tv} = {v}; {} }}",
                        pc.prelude.join(" "),
                        pc.set.replace("@@", &newv)
                    ),
                );
            }
            Stmt::Print(args, nl) => {
                let arr = self.array(args);
                line(
                    out,
                    &format!("sw_obwiesc({}, {arr}, {});", args.len(), *nl as i32),
                );
            }
            Stmt::ReadLine(p, prompt) => {
                let pr = prompt
                    .as_ref()
                    .map_or("SW_NIC".to_string(), |e| self.expr(e));
                let pc = self.place(p);
                let tv = self.fresh();
                line(
                    out,
                    &format!(
                        "{{ {} W {tv} = sw_zapytaj({pr}); {} }}",
                        pc.prelude.join(" "),
                        pc.set.replace("@@", &tv)
                    ),
                );
            }
            Stmt::ReadToken(p) => {
                let pc = self.place(p);
                let tv = self.fresh();
                line(
                    out,
                    &format!(
                        "{{ {} W {tv} = sw_wczytaj(); {} }}",
                        pc.prelude.join(" "),
                        pc.set.replace("@@", &tv)
                    ),
                );
            }
            Stmt::If(arms, els, l) => {
                for (i, (cond, body)) in arms.iter().enumerate() {
                    let c = self.expr(cond);
                    let kw = if i == 0 { "if" } else { "} else if" };
                    line(out, &format!("{kw} (sw_prawdziwe({c}, {l})) {{"));
                    self.block(out, body, d + 1);
                }
                if let Some(body) = els {
                    line(out, "} else {");
                    self.block(out, body, d + 1);
                }
                line(out, "}");
            }
            Stmt::While(cond, body, l) => {
                let c = self.expr(cond);
                line(out, &format!("while (sw_prawdziwe({c}, {l})) {{"));
                self.loop_body(out, body, d + 1);
                line(out, "}");
            }
            Stmt::Forever(body) => {
                line(out, "for (;;) {");
                self.loop_body(out, body, d + 1);
                line(out, "}");
            }
            Stmt::ForRange {
                var,
                from,
                to,
                step,
                body,
                line: l,
            } => {
                let (a, b, st, k) = (self.fresh(), self.fresh(), self.fresh(), self.fresh());
                let fe = self.expr(from);
                let te = self.expr(to);
                let se = match step {
                    Some(s) => format!("sw_calk({}, {l})", self.expr(s)),
                    // „od dziesięciu do jednego” liczy wstecz; przy granicach liczonych w biegu
                    // krok jest zawsze +1, by „od 1 do n” przy n = 0 nie czyniło nic.
                    None => match (from, to) {
                        (Expr::Int(x), Expr::Int(y)) if x > y => "-1".to_string(),
                        _ => "1".to_string(),
                    },
                };
                line(
                    out,
                    &format!(
                        "{{ long long {a} = sw_calk({fe}, {l}), {b} = sw_calk({te}, {l}); long long {st} = {se};"
                    ),
                );
                line(
                    out,
                    &format!(
                        "  if ({st} == 0) sw_grzech({l}, \"Krok pętli równy zeru — chodziłbyś w kółko do sądnego dniá.\");"
                    ),
                );
                let ok = self.fresh();
                line(out, &format!("  {} {{", range_for(&k, &ok, &a, &b, &st)));
                line(out, &format!("    {var} = sw_liczba({k});"));
                self.loop_body(out, body, d + 1);
                line(out, "} }");
            }
            Stmt::ForEach {
                var,
                coll,
                body,
                line: l,
            } => {
                let (c, k) = (self.fresh(), self.fresh());
                let ce = self.expr(coll);
                line(out, &format!("{{ W {c} = {ce};"));
                line(
                    out,
                    &format!("  for (long long {k} = 0; {k} < sw_iter_dl({c}, {l}); {k}++) {{"),
                );
                line(out, &format!("    {var} = sw_iter_el({c}, {k}, {l});"));
                self.loop_body(out, body, d + 1);
                line(out, "} }");
            }
            Stmt::Repeat(n, body, l) => {
                let (cnt, k) = (self.fresh(), self.fresh());
                let ne = self.expr(n);
                line(out, &format!("{{ long long {cnt} = sw_calk({ne}, {l});"));
                line(
                    out,
                    &format!("  for (long long {k} = 0; {k} < {cnt}; {k}++) {{"),
                );
                self.loop_body(out, body, d + 1);
                line(out, "} }");
            }
            Stmt::Break => line(out, &format!("{}break;", self.unwind_loop())),
            Stmt::Continue => line(out, &format!("{}continue;", self.unwind_loop())),
            Stmt::Return(v) => {
                let v = v.as_ref().map_or("SW_NIC".to_string(), |e| self.expr(e));
                if self.proby > 0 {
                    let t = self.fresh();
                    line(
                        out,
                        &format!(
                            "{{ W {t} = {v}; sw_proby_n -= {}; return {t}; }}",
                            self.proby
                        ),
                    );
                } else {
                    line(out, &format!("return {v};"));
                }
            }
            Stmt::Try {
                body,
                catch_var,
                handler,
                ..
            } => {
                line(out, "if (setjmp(*sw_proba_wejdz()) == 0) {");
                self.proby += 1;
                self.block(out, body, d + 1);
                self.proby -= 1;
                line(out, "    sw_proba_wyjdz();");
                line(out, "} else {");
                if let Some(v) = catch_var {
                    line(out, &format!("    {v} = sw_bieda;"));
                }
                if let Some(h) = handler {
                    self.block(out, h, d + 1);
                }
                line(out, "}");
            }
            Stmt::Throw(v, l) => {
                let v = self.expr(v);
                line(out, &format!("sw_rzuc({v}, {l});"));
            }
            Stmt::Exit(v, l) => {
                let v = self.expr(v);
                line(out, &format!("sw_zakoncz({v}, {l});"));
            }
            Stmt::Mark(first, last) => self.mark(out, &ind, *first, *last),
            Stmt::Expr(e) => {
                let e = self.expr(e);
                line(out, &format!("(void){e};"));
            }
            Stmt::Sort {
                coll,
                field,
                desc,
                line: l,
            } => {
                let c = self.expr(coll);
                let f = field.map_or(-1, |f| f as i64);
                line(out, &format!("sw_sortuj({c}, {f}, {}, {l});", *desc as i32));
            }
            Stmt::Remove {
                coll,
                index,
                line: l,
            } => {
                let c = self.expr(coll);
                let i = index
                    .as_ref()
                    .map_or("SW_NIC".to_string(), |e| self.expr(e));
                line(out, &format!("sw_usun({c}, {i}, {l});"));
            }
            Stmt::WriteFile {
                content,
                path,
                append,
                line: l,
            } => {
                let c = self.expr(content);
                let p = self.expr(path);
                line(
                    out,
                    &format!("sw_zapisz({c}, {p}, {}, {l});", *append as i32),
                );
            }
        }
    }
}
