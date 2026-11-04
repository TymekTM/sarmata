//! Normalizacja pisowni i wyłuskiwanie rdzeni wyrazów.
//!
//! Dzięki temu `Ieſli`, `jeśli` i `JEŚLI` znaczą to samo, a `suma`, `sumę`
//! i `sumie` wskazują na tę samą rzecz.

/// Sprowadza wyraz do postaci bez ogonków i staropolskich osobliwości:
/// małe litery, `ſ`→`s`, `á`→`a`, `é`→`e`, `v`→`u`, `j`/`y`→`i`, polskie znaki bez kresek.
pub fn normalize(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    for ch in word.chars().flat_map(|c| c.to_lowercase()) {
        out.push(match ch {
            'ſ' | 'ś' => 's',
            'á' | 'ą' | 'à' => 'a',
            'é' | 'ę' | 'è' => 'e',
            'ó' | 'ò' => 'o',
            'ć' => 'c',
            'ź' | 'ż' => 'z',
            'ł' => 'l',
            'ń' => 'n',
            'v' => 'u',
            'j' | 'y' => 'i',
            c => c,
        });
    }
    out
}

/// Końcówki fleksyjne (już znormalizowane), dłuższe przed krótszymi.
const SUFFIXES: &[&str] = &[
    "iami", "ami", "iach", "ach", "iego", "ego", "iemu", "emu", "owie", "owi", "iom", "om", "iem",
    "em", "imi", "ich", "iei", "ei", "im", "ow", "ia", "ie", "ii", "io", "iu", "a", "e", "i", "o",
    "u",
];

/// Rdzeń znormalizowanego wyrazu: odcina jedną końcówkę, zostawiając co najmniej trzy litery.
pub fn stem(norm: &str) -> String {
    if norm.chars().count() <= 3 {
        return norm.to_string();
    }
    for s in SUFFIXES {
        if let Some(rest) = norm.strip_suffix(s)
            && rest.chars().count() >= 3
        {
            return rest.to_string();
        }
    }
    norm.to_string()
}

/// Miano zdatne na identyfikator w C (same litery łacińskie, cyfry i podkreślniki).
pub fn ascii_ident(s: &str) -> String {
    let n = normalize(s);
    let mut out = String::new();
    for c in n.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }
    out.trim_matches('_').chars().take(24).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_and_new_spelling_meet() {
        assert_eq!(normalize("Ieſli"), normalize("jeśli"));
        assert_eq!(normalize("Vczynek"), normalize("uczynek"));
        assert_eq!(normalize("mnieyſze"), normalize("mniejsze"));
        assert_eq!(normalize("Niechay"), normalize("niechaj"));
    }

    #[test]
    fn declensions_share_a_stem() {
        let s = |w: &str| stem(&normalize(w));
        assert_eq!(s("suma"), s("sumę"));
        assert_eq!(s("suma"), s("sumie"));
        assert_eq!(s("ocena"), s("ocen"));
        assert_eq!(s("ocena"), s("ocenami"));
        assert_eq!(s("student"), s("studenta"));
        assert_eq!(s("student"), s("studentów"));
        assert_eq!(s("średnia"), s("średniej"));
        assert_eq!(s("wynik"), s("wynikiem"));
        assert_eq!(s("największy"), s("największego"));
        assert_eq!(s("linia"), s("linii"));
        assert_eq!(s("linie"), s("linii"));
        assert_eq!(s("n"), "n");
    }
}
