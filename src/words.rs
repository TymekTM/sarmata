//! Słownik skryby: słowa kluczowe, liczebniki, ozdobniki i początki glos.
//!
//! Wszystkie formy zapisane są tu współczesną polszczyzną i normalizowane przy
//! starcie, więc staropolskie `Ieſli` trafia w to samo miejsce co `jeśli`.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use crate::text::normalize;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kw {
    // przypisanie
    Niechaj,
    Bedzie,
    Rowny,
    Iako,
    Odtad,
    Iest,
    // mowa i słuch
    Obwiesc,
    /// „Rzekł tedy rotmistrz: …” — obwieszczenie w czasie przeszłym, z narracją przed dwukropkiem.
    Rzekl,
    Zapytaj,
    Wczytaj,
    // rozgałęzienia
    Jesli,
    Azali,
    Inaczej,
    Przeciwnym,
    Razie,
    // pętle
    Poki,
    Czyn,
    Zawsze,
    Dla,
    Kazdy,
    Powtorz,
    Razy,
    Przerwij,
    Pomin,
    // próby i klątwy
    Sprobuj,
    Biada,
    Zwany,
    Rzuc,
    Zakoncz,
    // przyimki
    Od,
    Do,
    Co,
    Z,
    W,
    O,
    Na,
    Przez,
    Pod,
    // uczynki i rody
    Uczynek,
    Przyjmujacy,
    Oddaj,
    Uczyn,
    Rod,
    Ma,
    Nowy,
    // czynności na rzeczach
    Zwieksz,
    Zmniejsz,
    Dodaj,
    Odejmij,
    Pomnoz,
    Podziel,
    Posortuj,
    Wedlug,
    Rosnaco,
    Malejaco,
    Usun,
    Zapisz,
    Dopisz,
    Plik,
    Utworz,
    Przenies,
    // logika
    Nie,
    I,
    Oraz,
    Albo,
    // rachunki
    Plus,
    Minus,
    Podzielone,
    Pomnozone,
    Modulo,
    // porównania
    Wiekszy,
    Mniejszy,
    Rozny,
    Podzielny,
    Parzysty,
    Nieparzysty,
    Pusty,
    Dodatni,
    Ujemny,
    Zawiera,
    Przewyzsza,
    Niz,
    Najmniej,
    Najwyzej,
    // wartości
    Prawda,
    Falsz,
    Nic,
    Liczebnik,
    Pol,
    Poltora,
    // regestry
    Element,
    /// Liczebnik porządkowy; -1 znaczy „ostatni”.
    Ordinal(i64),
    Dlugosc,
    Regestr,
    Pelen,
    Wspak,
    Spis,
    // spójniki zdań
    Potem,
    Po,
    Czym,
    Amen,
}

/// Rząd wielkości liczebnika, potrzebny do składania „dwustu trzydziestu pięciu”.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NumClass {
    Unit,
    Teen,
    Ten,
    Hundred,
    Thousand,
    Million,
}

const ADJ_ENDINGS: &[&str] = &[
    "y", "a", "e", "ego", "ej", "emu", "ym", "ą", "i", "ych", "ymi", "ia", "ie", "iego", "iej",
    "iemu", "im", "ią", "ich", "imi",
];

/// Wszystkie formy przymiotnika o danym temacie (z nadmiarem, co nie szkodzi).
fn adj(stem: &str) -> Vec<String> {
    ADJ_ENDINGS.iter().map(|e| format!("{stem}{e}")).collect()
}

fn add(m: &mut HashMap<String, Kw>, forms: &[&str], kw: Kw) {
    for f in forms {
        m.insert(normalize(f), kw);
    }
}

fn add_adj(m: &mut HashMap<String, Kw>, stems: &[&str], kw: Kw) {
    for s in stems {
        for f in adj(s) {
            m.insert(normalize(&f), kw);
        }
    }
}

pub static NUMBERS: LazyLock<HashMap<String, (i64, NumClass)>> = LazyLock::new(|| {
    use NumClass::*;
    let table: &[(i64, NumClass, &[&str])] = &[
        (0, Unit, &["zero", "zera", "zeru", "zerem", "zer"]),
        (
            1,
            Unit,
            &[
                "jeden", "jedna", "jedno", "jednego", "jednej", "jednemu", "jednym", "jedną",
            ],
        ),
        (
            2,
            Unit,
            &["dwa", "dwie", "dwóch", "dwu", "dwoma", "dwom", "dwiema"],
        ),
        (3, Unit, &["trzy", "trzech", "trzem", "trzema"]),
        (4, Unit, &["cztery", "czterech", "czterem", "czterema"]),
        (5, Unit, &["pięć", "pięciu", "pięcioma"]),
        (6, Unit, &["sześć", "sześciu", "sześcioma"]),
        (7, Unit, &["siedem", "siedmiu", "siedmioma"]),
        (8, Unit, &["osiem", "ośmiu", "ośmioma"]),
        (9, Unit, &["dziewięć", "dziewięciu", "dziewięcioma"]),
        (10, Teen, &["dziesięć", "dziesięciu", "dziesięcioma"]),
        (11, Teen, &["jedenaście", "jedenastu", "jedenastoma"]),
        (12, Teen, &["dwanaście", "dwunastu", "dwunastoma"]),
        (13, Teen, &["trzynaście", "trzynastu", "trzynastoma"]),
        (14, Teen, &["czternaście", "czternastu", "czternastoma"]),
        (15, Teen, &["piętnaście", "piętnastu", "piętnastoma"]),
        (16, Teen, &["szesnaście", "szesnastu", "szesnastoma"]),
        (17, Teen, &["siedemnaście", "siedemnastu", "siedemnastoma"]),
        (18, Teen, &["osiemnaście", "osiemnastu", "osiemnastoma"]),
        (
            19,
            Teen,
            &["dziewiętnaście", "dziewiętnastu", "dziewiętnastoma"],
        ),
        (20, Ten, &["dwadzieścia", "dwudziestu", "dwudziestoma"]),
        (30, Ten, &["trzydzieści", "trzydziestu", "trzydziestoma"]),
        (40, Ten, &["czterdzieści", "czterdziestu", "czterdziestoma"]),
        (50, Ten, &["pięćdziesiąt", "pięćdziesięciu"]),
        (60, Ten, &["sześćdziesiąt", "sześćdziesięciu"]),
        (70, Ten, &["siedemdziesiąt", "siedemdziesięciu"]),
        (80, Ten, &["osiemdziesiąt", "osiemdziesięciu"]),
        (90, Ten, &["dziewięćdziesiąt", "dziewięćdziesięciu"]),
        (100, Hundred, &["sto", "stu", "stoma"]),
        (200, Hundred, &["dwieście", "dwustu"]),
        (300, Hundred, &["trzysta", "trzystu"]),
        (400, Hundred, &["czterysta", "czterystu"]),
        (500, Hundred, &["pięćset", "pięciuset"]),
        (600, Hundred, &["sześćset", "sześciuset"]),
        (700, Hundred, &["siedemset", "siedmiuset"]),
        (800, Hundred, &["osiemset", "ośmiuset"]),
        (900, Hundred, &["dziewięćset", "dziewięciuset"]),
        (
            1000,
            Thousand,
            &[
                "tysiąc",
                "tysiące",
                "tysięcy",
                "tysiąca",
                "tysiącu",
                "tysiącem",
            ],
        ),
        (
            1_000_000,
            Million,
            &["milion", "miliony", "milionów", "miliona", "milionem"],
        ),
    ];
    let mut m = HashMap::new();
    for (v, c, forms) in table {
        for f in *forms {
            m.insert(normalize(f), (*v, *c));
        }
    }
    m
});

pub static KEYWORDS: LazyLock<HashMap<String, Kw>> = LazyLock::new(|| {
    use Kw::*;
    let mut m = HashMap::new();

    add(
        &mut m,
        &["niechaj", "niech", "niechże", "niechajże"],
        Niechaj,
    );
    add(&mut m, &["będzie", "będą"], Bedzie);
    add_adj(&mut m, &["równ"], Rowny);
    add(&mut m, &["wynosi", "wynoszą"], Rowny);
    add(&mut m, &["jako"], Iako);
    add(&mut m, &["odtąd", "odtądże", "odtychmiast"], Odtad);
    add(&mut m, &["jest", "są"], Iest);

    add(
        &mut m,
        &[
            "obwieść",
            "obwieśćże",
            "ogłoś",
            "głoś",
            "rzeknij",
            "rzeknijże",
            "wypisz",
            "napisz",
            "oznajmij",
            "powiedz",
            "wyrzeknij",
            "zakrzyknij",
            "wołaj",
        ],
        Obwiesc,
    );
    add(
        &mut m,
        &[
            "rzekł",
            "rzekła",
            "rzekli",
            "rzekły",
            "rzecze",
            "odrzekł",
            "odrzekła",
            "odrzekli",
            "powiedział",
            "powiedziała",
            "powiedzieli",
            "odpowiedział",
            "odpowiedziała",
            "przemówił",
            "przemówiła",
            "zawołał",
            "zawołała",
            "zawołali",
            "krzyknął",
            "krzyknęła",
            "krzyknęli",
            "zakrzyknął",
            "zakrzyknęła",
            "zakrzyknęli",
            "szepnął",
            "szepnęła",
            "westchnął",
            "westchnęła",
            "zaśpiewał",
            "zaśpiewała",
            "zaśpiewali",
            "ozwał",
            "ozwała",
            "prawił",
            "opowiada",
            "zapisano",
        ],
        Rzekl,
    );
    add(
        &mut m,
        &["zapytaj", "spytaj", "pytaj", "zapytajże"],
        Zapytaj,
    );
    add(&mut m, &["wczytaj", "pobierz", "weź", "przyjmij"], Wczytaj);

    add(
        &mut m,
        &[
            "jeśli",
            "jeżeli",
            "jeśliby",
            "jeżeliby",
            "gdy",
            "gdyby",
            "kiedy",
            "skoro",
            "jeśliże",
        ],
        Jesli,
    );
    add(&mut m, &["azali", "azaliż"], Azali);
    add(&mut m, &["inaczej", "inakszej", "inaksze"], Inaczej);
    add(&mut m, &["przeciwnym"], Przeciwnym);
    add(&mut m, &["razie"], Razie);

    add(&mut m, &["póki", "dopóki", "dokąd", "pókiż"], Poki);
    add(
        &mut m,
        &["czyń", "czyńże", "rób", "powtarzaj", "działaj"],
        Czyn,
    );
    add(
        &mut m,
        &["zawsze", "wiecznie", "wiekuiście", "nieustannie"],
        Zawsze,
    );
    add(&mut m, &["dla"], Dla);
    add_adj(&mut m, &["każd", "wszelk", "wszystk", "wszytk"], Kazdy);
    add(&mut m, &["wszyscy", "wszytcy"], Kazdy);
    add(&mut m, &["powtórz", "powtórzże"], Powtorz);
    add(&mut m, &["razy", "raz"], Razy);
    add(
        &mut m,
        &["przerwij", "dość", "przestań", "zaniechaj"],
        Przerwij,
    );
    add(&mut m, &["pomiń", "dalej", "opuść"], Pomin);

    add(&mut m, &["spróbuj", "spróbujże", "próbuj"], Sprobuj);
    add(
        &mut m,
        &[
            "biada", "bieda", "biedy", "biedzie", "biedę", "klęska", "klęski", "klęsce", "klęskę",
        ],
        Biada,
    );
    add_adj(&mut m, &["zwan", "nazwan", "nazywan"], Zwany);
    add(&mut m, &["rzuć", "rzućże", "ciśnij", "miotnij"], Rzuc);
    add(&mut m, &["zakończ", "zakończże", "skończ"], Zakoncz);

    add(&mut m, &["od", "ode"], Od);
    add(&mut m, &["do"], Do);
    add(&mut m, &["co"], Co);
    add(&mut m, &["z", "ze", "spośród", "spomiędzy"], Z);
    add(&mut m, &["w", "we", "wśród", "pośród"], W);
    add(&mut m, &["o"], O);
    add(&mut m, &["na"], Na);
    add(&mut m, &["przez"], Przez);
    add(&mut m, &["pod"], Pod);

    add(&mut m, &["uczynek", "powinność"], Uczynek);
    add_adj(&mut m, &["przyjmując", "biorąc", "żądając"], Przyjmujacy);
    add(&mut m, &["przyjmuje", "bierze"], Przyjmujacy);
    add(&mut m, &["oddaj", "oddajże", "zwróć"], Oddaj);
    add(
        &mut m,
        &["uczyń", "wykonaj", "wezwij", "przywołaj", "wywołaj"],
        Uczyn,
    );
    add(&mut m, &["ród"], Rod);
    add(&mut m, &["ma", "mają", "posiada", "posiadają"], Ma);
    add_adj(&mut m, &["now"], Nowy);

    add(&mut m, &["zwiększ", "powiększ"], Zwieksz);
    add(&mut m, &["zmniejsz", "pomniejsz", "umniejsz"], Zmniejsz);
    add(&mut m, &["dodaj", "dołóż", "przydaj", "dorzuć"], Dodaj);
    add(&mut m, &["odejmij", "ujmij"], Odejmij);
    add(&mut m, &["pomnóż", "rozmnóż"], Pomnoz);
    add(&mut m, &["podziel"], Podziel);
    add(&mut m, &["posortuj", "uporządkuj", "ułóż"], Posortuj);
    add(&mut m, &["według", "wedle", "podług"], Wedlug);
    add(&mut m, &["rosnąco"], Rosnaco);
    add(&mut m, &["malejąco"], Malejaco);
    add(&mut m, &["usuń", "wyrzuć", "wymaż"], Usun);
    add(&mut m, &["zapisz", "spisz"], Zapisz);
    add(&mut m, &["dopisz"], Dopisz);
    add(
        &mut m,
        &["plik", "pliku", "plikiem", "pliki", "plików"],
        Plik,
    );
    add(&mut m, &["utwórz", "stwórz", "załóż"], Utworz);
    add(&mut m, &["przenieś", "przesuń"], Przenies);

    add(&mut m, &["nie"], Nie);
    add(&mut m, &["i"], I);
    add(&mut m, &["oraz", "tudzież"], Oraz);
    add(&mut m, &["albo", "lub", "bądź"], Albo);

    add(&mut m, &["plus", "dodać"], Plus);
    add(&mut m, &["minus", "odjąć", "bez"], Minus);
    add_adj(&mut m, &["podzielon", "dzielon"], Podzielone);
    add_adj(&mut m, &["pomnożon", "mnożon"], Pomnozone);
    add(&mut m, &["modulo"], Modulo);

    add_adj(&mut m, &["większ", "więtsz"], Wiekszy);
    add_adj(&mut m, &["mniejsz"], Mniejszy);
    add(&mut m, &["więcej"], Wiekszy);
    add(&mut m, &["mniej"], Mniejszy);
    add_adj(&mut m, &["różn"], Rozny);
    add_adj(&mut m, &["podzieln"], Podzielny);
    // „dzielą” byłoby po normalizacji tym samym co „dzieła” (dzieło), więc go nie ma.
    add(&mut m, &["dzieli"], Podzielny);
    add_adj(&mut m, &["parzyst"], Parzysty);
    add_adj(&mut m, &["nieparzyst"], Nieparzysty);
    add_adj(&mut m, &["pust"], Pusty);
    add_adj(&mut m, &["dodatn"], Dodatni);
    add_adj(&mut m, &["ujemn"], Ujemny);
    add(&mut m, &["zawiera", "zawierają", "mieści"], Zawiera);
    add(&mut m, &["przewyższa", "przewyższają"], Przewyzsza);
    add(&mut m, &["niż", "niźli", "aniżeli", "nad"], Niz);
    add(&mut m, &["najmniej"], Najmniej);
    add(&mut m, &["najwyżej"], Najwyzej);

    add(
        &mut m,
        &["prawda", "prawdą", "prawdę", "prawdy", "zaiste"],
        Prawda,
    );
    add(
        &mut m,
        &[
            "fałsz",
            "fałszem",
            "fałszu",
            "bynajmniej",
            "nieprawda",
            "łgarstwo",
        ],
        Falsz,
    );
    add(
        &mut m,
        &["nic", "niczego", "niczym", "nicość", "nicości", "nicością"],
        Nic,
    );
    add(&mut m, &["pół"], Pol);
    add(&mut m, &["półtora"], Poltora);

    add(
        &mut m,
        &[
            "element",
            "elementu",
            "elementem",
            "elemencie",
            "elementy",
            "elementów",
            "wyraz",
            "wyrazu",
            "wyrazem",
        ],
        Element,
    );
    let ordinals: &[(i64, &[&str])] = &[
        (1, &["pierwsz"]),
        (2, &["drug", "wtór"]),
        (3, &["trzec"]),
        (4, &["czwart"]),
        (5, &["piąt"]),
        (6, &["szóst"]),
        (7, &["siódm"]),
        (8, &["ósm"]),
        (9, &["dziewiąt"]),
        (10, &["dziesiąt"]),
        (-1, &["ostatn"]),
    ];
    for (n, stems) in ordinals {
        add_adj(&mut m, stems, Ordinal(*n));
    }
    add(
        &mut m,
        &[
            "długość",
            "długości",
            "długością",
            "rozmiar",
            "rozmiaru",
            "rozmiarem",
        ],
        Dlugosc,
    );
    add(
        &mut m,
        &[
            "regestr",
            "regestru",
            "regestrem",
            "regestrze",
            "regestry",
            "regestrów",
            "rejestr",
            "rejestru",
            "rejestrem",
            "rejestrze",
            "rejestry",
            "rejestrów",
        ],
        Regestr,
    );
    add(&mut m, &["pełen"], Pelen);
    add_adj(&mut m, &["pełn", "wypełnion"], Pelen);
    add(&mut m, &["wspak"], Wspak);
    add(
        &mut m,
        &[
            "spis",
            "spisu",
            "spisem",
            "spisie",
            "spisy",
            "spisów",
            "słownik",
            "słownika",
            "słownikiem",
            "słowniku",
            "słowniki",
        ],
        Spis,
    );

    add(
        &mut m,
        &["potem", "następnie", "wreszcie", "później"],
        Potem,
    );
    add(&mut m, &["po"], Po);
    add(&mut m, &["czym"], Czym);
    add(&mut m, &["amen"], Amen);

    for k in NUMBERS.keys() {
        m.insert(k.clone(), Liczebnik);
    }
    m
});

/// Słowa ozdobne, które skryba pomija, by proza płynęła.
pub static FILLERS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    [
        "tedy",
        "wtedy",
        "to",
        "zatem",
        "przeto",
        "oto",
        "otóż",
        "wszak",
        "wszakże",
        "albowiem",
        "bowiem",
        "nuże",
        "ninie",
        "teraz",
        "więc",
        "się",
        "waść",
        "waćpan",
        "waszmość",
        "waszeć",
        "mości",
        "panie",
        "łaskawie",
        "proszę",
        "uprzejmie",
        "zaś",
        "lecz",
        "natenczas",
        "wówczas",
        "onego",
        "owego",
        "też",
        "także",
        "jeszcze",
        "toż",
        "ano",
        "ach",
        "och",
        "hej",
        "zaprawdę",
        "jakoż",
    ]
    .iter()
    .map(|s| normalize(s))
    .collect()
});

/// Wiersz zaczynający się od takiego słowa jest glosą (komentarzem) lub nagłówkiem.
pub static COMMENT_STARTERS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    [
        "glossa",
        "glosa",
        "przypis",
        "rozdział",
        "księga",
        "pieśń",
        "anno",
        "przedmowa",
        "posłowie",
        "tytuł",
    ]
    .iter()
    .map(|s| normalize(s))
    .collect()
});

pub fn keyword(norm: &str) -> Option<Kw> {
    KEYWORDS.get(norm).copied()
}

pub fn number(norm: &str) -> Option<(i64, NumClass)> {
    NUMBERS.get(norm).copied()
}

/// Liczba rzymska pisana wersalikami, np. `MMXXVI`.
pub fn roman(raw: &str) -> Option<i64> {
    if raw.is_empty() || !raw.chars().all(|c| "IVXLCDM".contains(c)) {
        return None;
    }
    let val = |c: char| match c {
        'I' => 1,
        'V' => 5,
        'X' => 10,
        'L' => 50,
        'C' => 100,
        'D' => 500,
        _ => 1000,
    };
    let cs: Vec<i64> = raw.chars().map(val).collect();
    let mut total = 0;
    for i in 0..cs.len() {
        if i + 1 < cs.len() && cs[i] < cs[i + 1] {
            total -= cs[i];
        } else {
            total += cs[i];
        }
    }
    (to_roman(total) == raw).then_some(total)
}

fn to_roman(mut n: i64) -> String {
    let table = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut s = String::new();
    for (v, r) in table {
        while n >= v {
            s.push_str(r);
            n -= v;
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_accept_old_spelling() {
        assert_eq!(keyword(&normalize("Ieſli")), Some(Kw::Jesli));
        assert_eq!(keyword(&normalize("mnieyſze")), Some(Kw::Mniejszy));
        assert_eq!(keyword(&normalize("więtſzy")), Some(Kw::Wiekszy));
        assert_eq!(keyword(&normalize("Oddáy")), Some(Kw::Oddaj));
        assert_eq!(keyword(&normalize("wtóry")), Some(Kw::Ordinal(2)));
    }

    #[test]
    fn roman_numerals() {
        assert_eq!(roman("MMXXVI"), Some(2026));
        assert_eq!(roman("IIII"), None);
        assert_eq!(roman("Ala"), None);
    }
}
