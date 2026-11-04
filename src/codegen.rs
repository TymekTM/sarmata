//! Przekład drzewa na język C. Bieg (runtime) dokleja się na początku.

use std::collections::HashMap;
use std::fmt::Write;

use crate::ast::*;

const RUNTIME: &str = include_str!("../runtime/sarmata.c");
const TABLES_MARK: &str = "/*@@TABLICE@@*/";

/// `pisma` to miana pism (główne i powołane księgi) do obwieszczeń grzechów biegu.
pub fn generate(prog: &Program, pisma: &[String]) -> String {
    let mut g = Gen {
        prog,
        lits: Vec::new(),
        lit_ids: HashMap::new(),
        tmp: 0,
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
        pisma.iter().map(|p| c_string(p)).collect::<Vec<_>>().join(", ")
    );
    out.push_str(tail);

    out.push_str("\n/* ─────────── pismo Waszmości ─────────── */\n\n");
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
}

/// Miejsce rozłożone na: przygotowanie, odczyt i wzór zapisu (`@@` = nowa wartość).
struct PlaceCode {
    prelude: Vec<String>,
    get: String,
    set: String,
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
        let _ = writeln!(
            out,
            "static W {}({}) {{",
            sig.cname,
            params_decl(sig.arity, Some(&f.params))
        );
        for (c, display) in &f.locals {
            let _ = writeln!(out, "    W {c} = SW_NIC; /* {} */", comment(display));
        }
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
            Expr::Builtin(f, args, l) => format!("{f}({}, {l})", self.args(args)),
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
                let newv = format!("{f}({}, {tv}, {l})", pc.get);
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
                self.block(out, body, d + 1);
                line(out, "}");
            }
            Stmt::Forever(body) => {
                line(out, "for (;;) {");
                self.block(out, body, d + 1);
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
                line(
                    out,
                    &format!(
                        "  for (long long {k} = {a}; {st} > 0 ? {k} <= {b} : {k} >= {b}; {k} += {st}) {{"
                    ),
                );
                line(out, &format!("    {var} = sw_liczba({k});"));
                self.block(out, body, d + 1);
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
                self.block(out, body, d + 1);
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
                self.block(out, body, d + 1);
                line(out, "} }");
            }
            Stmt::Break => line(out, "break;"),
            Stmt::Continue => line(out, "continue;"),
            Stmt::Return(v) => {
                let v = v.as_ref().map_or("SW_NIC".to_string(), |e| self.expr(e));
                line(out, &format!("return {v};"));
            }
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
