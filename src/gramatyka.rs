//! Gramatyka TextMate dla edytorów (VS Code i krewni), wywiedziona wprost ze słownika
//! skryby, by kolorowanie nie rozjechało się z tym, co skryba naprawdę rozumie.

use std::collections::BTreeMap;

use crate::parser::BUILTINS;
use crate::text::normalize;
use crate::words::{COMMENT_STARTERS, KEYWORDS, Kw, NUMBERS};

/// Litera znormalizowana → wszystkie pisownie, z których mogła powstać.
fn letter(c: char) -> String {
    match c {
        's' => "[sſś]".into(),
        'a' => "[aáąà]".into(),
        'e' => "[eéęè]".into(),
        'o' => "[oóò]".into(),
        'c' => "[cć]".into(),
        'z' => "[zźż]".into(),
        'l' => "[lł]".into(),
        'n' => "[nń]".into(),
        'u' => "[uv]".into(),
        'i' => "[ijy]".into(),
        ' ' => "\\s+".into(),
        c => regex_escape(c),
    }
}

fn regex_escape(c: char) -> String {
    if c.is_alphanumeric() {
        c.to_string()
    } else {
        format!("\\{c}")
    }
}

fn pattern(norm: &str) -> String {
    norm.chars().map(letter).collect()
}

/// Wzór na całe wyrazy: najdłuższe najpierw, by „pomnożone” nie stało się „pomnoż” + „one”.
fn words(mut forms: Vec<String>) -> String {
    forms.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()).then(a.cmp(b)));
    forms.dedup();
    let alts: Vec<String> = forms.iter().map(|f| pattern(f)).collect();
    format!(
        "(?i)(?<![\\p{{L}}\\p{{N}}])(?:{})(?![\\p{{L}}\\p{{N}}])",
        alts.join("|")
    )
}

fn scope(kw: Kw) -> &'static str {
    use Kw::*;
    match kw {
        Jesli | Azali | Inaczej | Przeciwnym | Razie | Poki | Czyn | Zawsze | Dla | Kazdy
        | Powtorz | Razy | Przerwij | Pomin | Oddaj | Sprobuj | Biada | Zwany | Rzuc | Zakoncz
        | Potem | Po | Czym | Amen => "keyword.control.sarmata",
        Uczynek | Przyjmujacy | Rod | Ma | Nowy | Niechaj | Bedzie | Odtad => {
            "storage.type.sarmata"
        }
        Prawda | Falsz | Nic | Liczebnik | Pol | Poltora | Ordinal(_) => {
            "constant.language.sarmata"
        }
        Nie | I | Oraz | Albo | Plus | Minus | Podzielone | Pomnozone | Modulo | Wiekszy
        | Mniejszy | Rowny | Rozny | Podzielny | Parzysty | Nieparzysty | Pusty | Dodatni
        | Ujemny | Zawiera | Przewyzsza | Niz | Najmniej | Najwyzej | Iest | Iako => {
            "keyword.operator.sarmata"
        }
        Obwiesc | Rzekl | Zapytaj | Wczytaj | Uczyn | Zwieksz | Zmniejsz | Dodaj | Odejmij
        | Pomnoz | Podziel | Posortuj | Usun | Zapisz | Dopisz | Utworz | Przenies => {
            "support.function.verb.sarmata"
        }
        _ => "keyword.other.sarmata",
    }
}

/// Plik `sarmata.tmLanguage.json`.
pub fn tm_language() -> String {
    let mut groups: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for (form, kw) in KEYWORDS.iter() {
        groups.entry(scope(*kw)).or_default().push(form.clone());
    }
    groups
        .entry("constant.numeric.words.sarmata")
        .or_default()
        .extend(NUMBERS.keys().cloned());
    let builtins: Vec<String> = BUILTINS.iter().map(|b| normalize(b.0)).collect();
    let comments: Vec<String> = COMMENT_STARTERS.iter().cloned().collect();

    let mut patterns = vec![
        format!(
            r#"{{ "name": "comment.line.sarmata", "match": {} }}"#,
            json(&format!(
                "(?i)^\\s*{}.*$",
                words(comments).trim_start_matches("(?i)")
            ))
        ),
        r##"{ "include": "#cytat" }"##.to_string(),
        format!(
            r#"{{ "name": "support.function.builtin.sarmata", "match": {} }}"#,
            json(&words(builtins))
        ),
    ];
    for (name, forms) in groups {
        patterns.push(format!(
            r#"{{ "name": {}, "match": {} }}"#,
            json(name),
            json(&words(forms))
        ));
    }
    patterns.push(format!(
        r#"{{ "name": "constant.numeric.roman.sarmata", "match": {} }}"#,
        json(r"(?<![\p{L}\p{N}])M{0,3}(?:CM|CD|D?C{0,3})(?:XC|XL|L?X{0,3})(?:IX|IV|V?I{0,3})(?![\p{L}\p{N}])(?<=[MDCLXVI])")
    ));
    patterns.push(format!(
        r#"{{ "name": "constant.numeric.sarmata", "match": {} }}"#,
        json(r"(?<![\p{L}\p{N}])\d+(?:[.,]\d+)?(?![\p{L}\p{N}])")
    ));

    let quote = |open: &str, close: &str| {
        format!(
            r##"{{ "name": "string.quoted.sarmata", "begin": {}, "end": {}, "patterns": [ {{ "include": "#wstawka" }} ] }}"##,
            json(open),
            json(close)
        )
    };
    let cytaty = [
        quote("„", "[”“\"]"),
        quote("“", "[”\"]"),
        quote("«", "»"),
        quote("\"", "\""),
    ];

    format!(
        r#"{{
  "$schema": "https://raw.githubusercontent.com/martinring/tmlanguage/master/tmlanguage.json",
  "name": "Sarmata",
  "scopeName": "source.sarmata",
  "fileTypes": ["sar"],
  "patterns": [
    {}
  ],
  "repository": {{
    "cytat": {{ "patterns": [
      {}
    ] }},
    "wstawka": {{
      "name": "meta.interpolation.sarmata",
      "begin": "\\{{",
      "end": "\\}}",
      "beginCaptures": {{ "0": {{ "name": "punctuation.section.interpolation.begin.sarmata" }} }},
      "endCaptures": {{ "0": {{ "name": "punctuation.section.interpolation.end.sarmata" }} }},
      "patterns": [ {{ "include": "$self" }} ]
    }}
  }}
}}
"#,
        patterns.join(",\n    "),
        cytaty.join(",\n      ")
    )
}

fn json(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn edytor_ma_aktualna_gramatyke() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/edytory/vscode/syntaxes/sarmata.tmLanguage.json"
        );
        let saved = std::fs::read_to_string(path).unwrap_or_default();
        assert!(
            saved == super::tm_language(),
            "Gramatyka w edytory/vscode jest nieaktualna. Odśwież ją: \
             cargo run -- gramatyka > edytory/vscode/syntaxes/sarmata.tmLanguage.json"
        );
    }
}
