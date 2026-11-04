# Statut języka Sarmata

Pełny opis języka: najpierw krótki kurs, potem ścisła ściąga i lista pułapek.
Jeśli piszesz w Sarmacie po raz pierwszy, przeczytaj całość. Jeśli wracasz, zajrzyj
do [ściągi](#7-ściąga) i [pułapek](#8-pułapki).

Spis treści:
1. [Pierwszy program](#1-pierwszy-program)
2. [Jak skryba czyta tekst](#2-jak-skryba-czyta-tekst)
3. [Miana, czyli nazwy](#3-miana-czyli-nazwy)
4. [Wartości](#4-wartości)
5. [Rozkazy](#5-rozkazy)
6. [Uczynki, rody i księgi](#6-uczynki-rody-i-księgi)
7. [Ściąga](#7-ściąga)
8. [Pułapki](#8-pułapki)
9. [Styl, czyli jak pisać kronikę, a nie kod](#9-styl-czyli-jak-pisać-kronikę-a-nie-kod)
10. [Pełny przykład](#10-pełny-przykład)

---

## 1. Pierwszy program

```
Anno Domini MMXXVI.

Obwieść „Witay, świecie!”.

Amen.
```

Zapisz to jako `witay.sar` i uruchom:

```sh
sarmata uruchom witay.sar      # skompiluj i uruchom
sarmata witay.sar              # tylko skompiluj do pliku ./witay
sarmata c witay.sar            # pokaż wygenerowany kod C
```

Pierwszy wiersz to przypis (komentarz), bo zaczyna się od słowa *Anno*. Drugi to rozkaz
wypisania. `Amen.` kończy program i jest **obowiązkowe**.

---

## 2. Jak skryba czyta tekst

### Zdania

Program to ciąg zdań. **Każde zdanie to rozkaz** i kończy się kropką (`!`, `?` i `;` też
kończą zdanie). Zdanie może ciągnąć się przez kilka wierszy, bo dla skryby liczy się
tylko znak końca zdania:

```
Obwieść „Naywiętſzy wſpólny dzielnik wynosi”,
    naywiętſzy wſpólny dzielnik z czterdziestu i sześćdziesięciu.
```

Kropka wewnątrz cudzysłowu na samym końcu wiersza też kończy zdanie:
`Obwieść „Gotowe.”` jest poprawne.

### Bloki

Zdanie zakończone **dwukropkiem** i następujące po nim **wcięte** zdania tworzą blok,
podobnie jak w Pythonie:

```
Dla każdey liczby od iednego do trzech:
    Obwieść liczbę.
    Obwieść „—”.
Obwieść „koniec”.
```

Wcięcia mają znaczenie tylko na początku zdania, więc wiersz, który jest dalszym ciągiem
zdania, może być wcięty dowolnie.

Krótkie ciało można zapisać w tym samym zdaniu, po przecinku albo dwukropku:

```
Ieśli liczba iest parzysta, obwieść „parzysta”.
Powtórz trzy razy: obwieść „hey”.
```

### Kilka rozkazów w jednym zdaniu

Rozkazy można łączyć przecinkiem, „i”, „á potem”, „po czym”:

```
Zwiększ licznik, á potem obwieść licznik.
Dodaj trzy do zbioru i dodaj ieden do zbioru.
```

### Pisownia

Pisać można po staremu albo po nowemu. Przed porównaniem każde słowo jest sprowadzane
do wspólnej postaci: małe litery, `ſ`→s, `á`→a, `é`→e, `y`→i, `j`→i, `v`→u,
a `ą ę ó ś ć ź ż ł ń` tracą znaki diakrytyczne.

Zatem `Ieſli`, `Jeśli` i `ieśli` to jedno słowo, podobnie `niechay` i `niechaj` albo
`mnieysza` i `mniejsza`. Wielkość liter nie ma znaczenia, z jednym wyjątkiem: liczby
rzymskie muszą być pisane wielkimi literami.

### Przypisy (komentarze)

Wiersz zaczynający się od jednego ze słów *Glossa, Glosa, Przypis, Rozdział, Księga,
Pieśń, Anno, Przedmowa, Posłowie, Tytuł* jest przypisem i skryba go pomija.

Przypis, który nie kończy się kropką (ani `!`, `?`), obejmuje też kolejne wiersze,
o ile zaczynają się **małą literą**. Pusty wiersz zawsze kończy przypis.

```
Przedmowa. Proiekt semestralny z przedmiotu „Podstawy programowania”,
spisany ku chwale Rzeczypospolitey.

Rozdział wtóry, o przyimowaniu żaków.
```

### Słowa-wypełniacze

Skryba pomija słowa, które dodają tylko kolorytu, więc można ich używać do woli:

> tedy, wtedy, to, zatem, przeto, oto, otóż, wszak, wszakże, albowiem, bowiem, nuże,
> ninie, teraz, więc, się, waść, waćpan, waszmość, waszeć, mości, panie, łaskawie, proszę,
> uprzejmie, zaś, lecz, natenczas, wówczas, onego, owego, też, także, jeszcze, toż, ano,
> ach, och, hej, zaprawdę, jakoż

Wyrażenia *co następuje* i *jako następuje* też znikają.

`Ieśli liczba iest parzysta, to obwieść ją tedy.` znaczy dokładnie to samo co
`Ieśli liczba iest parzysta, obwieść ją.`

Uwaga: przez to te słowa **nie mogą** być mianami.

---

## 3. Miana, czyli nazwy

### Chrzest

Zmienną tworzy się rozkazem `Niechay`:

```
Niechay liczba będzie pięć.
Niechay będzie liczba równa pięć.
Niechay pierwsza liczba będzie dziesięć.
Niechay naywiększy wspólny dzielnik będzie zero.
```

Miano może mieć kilka słów, pisanych **osobno**, ze spacjami. Jeśli zaczyna się od
liczebnika porządkowego (`pierwsza liczba`, `druga liczba`), to też jest miano.
Podkreślnik (`miano_wpisu`) jest zakazany: skryba go odrzuci i podpowie, jak zapisać to
osobnymi słowami. Każde słowo miana odmienia się wtedy samo (`cały utarg` → `całego utargu`).

### Rozpoznawanie po rdzeniu

Skryba rozpoznaje miana **po rdzeniu**, więc odmiana działa sama:

```
Niechay liczba będzie pięć.
Zwiększ liczbę o dwa.
Obwieść silnię z liczby.
```

`liczba`, `liczbę`, `liczby`, `liczbą` mają rdzeń `liczb`, więc to ta sama rzecz.

Rdzeń powstaje przez odcięcie **jednej** typowej końcówki (np. *-ami, -ach, -ego, -em, -om,
-ów, -ie, -ia, -ii, -a, -e, -i, -o, -u, -y*), przy czym zostają co najmniej trzy litery.
Działa to dobrze dla regularnie odmienianych rzeczowników: *liczba, ocena, suma, nota,
lista, wynik, miano, nazwisko, licznik, numer, wiersz, słowo, żak*.

Zawodzi przy wymianach głosek i „e ruchomym”:

| Mianownik | Forma | Działa? |
|---|---|---|
| imię | imienia | ✗ |
| skarbiec | skarbca, skarbcem | ✗ |
| żak | żacy | ✗ (ale *żaki, żaków, żakami* działają) |
| dzień | dnia | ✗ |
| posłaniec | posłańca (ale *posłańcy, posłańców* to inna rzecz!) | ✗ |
| liczba | liczbę, liczby, liczbą | ✓ |
| autor, kot, rząd, most | o autorze, o kocie, w rządzie, na moście | ✓ (miejscownik skryba zna) |

**Rada:** wybieraj słowa regularne. Zamiast *imię* pisz *miano*, zamiast *skarbiec* pisz *skarb*.

### Niechay: nowe czy stare?

`Niechay X będzie …` najpierw szuka widocznej rzeczy o tym samym rdzeniu. Jeśli ją
znajdzie, zmienia jej wartość. Jeśli nie, tworzy nową:
- w uczynku jest to zmienna lokalna,
- poza uczynkiem jest to zmienna globalna.

`Odtąd X iest …` tylko zmienia wartość już istniejącej rzeczy.

`Zapytaj o X`, `Wczytaj X` i zmienna pętli `Dla każdego X …` też chrzczą nową rzecz,
jeśli takiej jeszcze nie ma. Nie trzeba jej wcześniej tworzyć przez `Niechay`.

### Zasięg

- Zmienne globalne (ochrzczone poza uczynkami) są widoczne **i zmienialne** we wszystkich
  uczynkach.
- Rzeczy, które uczynek przyjmuje, i zmienne ochrzczone w uczynku są lokalne.
- Zmienna pętli `Dla każdego X …` żyje tylko w tej pętli i przesłania inne rzeczy
  o tym samym rdzeniu.

### Rozstrzyganie

Jeśli w danym miejscu pasuje kilka mian, wygrywa:
1. dłuższe (więcej słów),
2. dokładna forma przed dopasowaniem po rdzeniu,
3. zmienna z bliższego zasięgu,
4. kolejno: zmienna, uczynek własny, uczynek wbudowany, pole.

Ród zawsze ustępuje wszystkiemu innemu. Można więc mieć `Ród Towar` i pisać
`Dla każdego towaru z kramu, obwieść towar.`, a `towar` będzie zmienną pętli.

Gdy dwie rzeczy remisują, skryba zgłasza dwuznaczność i prosi o miana o różnych rdzeniach.

---

## 4. Wartości

### Liczby

- Cyfry: `5`, `-3`, `1572`.
- Słownie, w dowolnym przypadku i złożone: `pięć`, `pięciu`, `dwieście trzydzieści pięć`,
  `tysiąc pięćset siedemdziesiąt dwa`, `dwa tysiące dwadzieścia sześć`.
- Rzymskie, **wielkimi** literami: `MDLXXII`, `XIV`.

### Ułamki

`3,14`, `3.14`, `dwa i pół`, `pół`, `półtora`.

Wypisywane są z przecinkiem: `4,75`, do 10 cyfr znaczących. Liczba całkowita dzielona
przez całkowitą daje całkowitą, jeśli dzieli się bez reszty (`osiem podzielone przez dwa` → `4`),
a w przeciwnym razie ułamek (`siedem podzielone przez dwa` → `3,5`).

### Pisma (napisy)

```
„Witay”      "Witay"      “Witay”      «Witay»
```

- Wstawki: `„Silnia z {n} wynosi {silnia z n}.”`. W klamrach może stać dowolne wyrażenie.
- Znaki specjalne: `\n` (nowy wiersz), `\t` (tabulator), `\„` (dosłowny znak).
- Puste pismo: `puste pismo`.
- Litery liczy się po znakach Unicode, więc `długość „żółć”` to 4.
- Cytat może ciągnąć się przez kilka wierszy. Skryba skleja je jedną spacją, a wcięcia
  kolejnych wierszy pomija. Pusty wiersz w środku cytatu to błąd.

```
Obwieść „Baszty do końca strzegł {strażnik}, á gdy przyszła odsiecz,
    dostał od Króla wieś pod Krakowem.”
```

### Prawda, fałsz, nic

- Prawda: `prawda`, `zaiste`.
- Fałsz: `fałsz`, `bynajmniej`, `nieprawda`, `łgarstwo`.
- Brak wartości: `nic`, `nicość`. Nieochrzczone pola rodu mają tę wartość, `Zapytaj`
  zwraca ją na końcu wejścia.

W warunku za fałsz uchodzą: `fałsz`, `nic`, `0`, puste pismo i pusty regestr.
Wszystko inne jest prawdą.

### Regestry (listy)

```
regestr z pięciu, trzech i ośmiu
regestr pusty        pusty regestr        nowy regestr
regestr o długości trzech pełen zer
```

Indeksuje się **od jedynki**:

```
element trzy z ocen
element numer ocen
pierwszy element ocen
drugi element ocen
ostatni element ocen
długość ocen
oceny wspak
```

Liczebniki porządkowe działają od *pierwszy* do *dziesiąty*, a *ostatni* oznacza ostatni
element. Te same sposoby działają na pismach i dają pojedyncze litery.

---

## 5. Rozkazy

### Nadawanie wartości

```
Niechay x będzie pięć.                    Niechay będzie x równe pięć.
Niechay x = pięć.                         Odtąd x iest sześć.
Odtąd a iest b, á b iest reszta.          (kolejne nadania bez powtarzania „Odtąd”)
Odtąd drugi element ocen iest pięć.       Odtąd średnia studenta iest cztery.
```

### Zmiana wartości

```
Zwiększ licznik.                Zwiększ licznik o trzy.
Zmniejsz licznik.               Zmniejsz licznik o dwa.
Pomnóż kapitał przez dwa.       Podziel tort przez osiem.
Odejmij dwa od skarbu.
Dodaj ocenę do ocen.            (do regestru: dokłada na koniec)
Dodaj „ab” do napisu.           (do pisma: dokleja; do liczby: dolicza)
```

### Wypisywanie

```
Obwieść „Witay”.
Obwieść „Wynik:”, wynik, „punktów”.        (rzeczy oddziela się spacją)
Obwieść liczbę nie kończąc wiersza.
Obwieść.                                    (pusty wiersz)
```

Rzeczy wypisywane rozdziela się przecinkiem, „i” albo „oraz”. Regestry wypisują się jako
`[1; 2; 3]`, pisma wewnątrz regestru w cudzysłowie, rody jako `Żak { miano: „Jan”; … }`.

**Mowa opowieści.** Do wypisywania służą też czasowniki w czasie przeszłym: *rzekł, rzekła,
rzekli, rzecze, odrzekł, powiedział, odpowiedział, przemówił, zawołał, krzyknął,
zakrzyknęli, szepnął, westchnął, zaśpiewał, ozwał, prawił, opowiada, zapisano*. Wszystko
między takim czasownikiem a **dwukropkiem** to narracja, którą skryba pomija. Wypisuje
się to, co stoi po dwukropku:

```
Rzekł tedy rotmistrz Zbigniew, gładząc wąsa: „— Mości panowie, chleba ledwie co.”
Westchnął rotmistrz: „— Na ciebie wypadło, {wysłannik}. Z Bogiem!”
Gdy sucharów iest zero, opowiada kronikarz: „Ziedli ostatnie suchary.”
```

Bez dwukropka taki czasownik działa jak zwykłe `Obwieść`.

### Wczytywanie

```
Zapytaj „Ile lat? ” o wiek.      (cały wiersz; pytanie wypisuje się bez nowego wiersza)
Zapytaj o wiek.
Wczytaj liczbę.                  (jedno słowo oddzielone odstępami)
```

Jeśli wczytany tekst wygląda jak liczba (`17`, `3,5`), staje się liczbą, a w przeciwnym
razie pismem. Na końcu wejścia zwracane jest `nic`.

### Warunki

```
Ieśli wiek iest mnieyszy niż osiemnaście:
    Obwieść „Za młody”.
A ieśli wiek iest większy albo równy sześćdziesiąt:
    Obwieść „Senior”.
Inaczey:
    Obwieść „W sam raz”.
```

- *Jeśli* ma też formy: `jeżeli`, `gdy`, `gdyby`, `kiedy`, `skoro`.
- Kolejne warunki: `A ieśli …` albo `Azali …`.
- Ostatnia gałąź: `Inaczey …`, `A inaczey …`, `W przeciwnym razie …`, `A ieśli nie, …`.
- Gałęzie mogą stać w tym samym zdaniu (po przecinku) albo w kolejnych zdaniach.

```
Ieśli x iest siedem, obwieść „7”, w przeciwnym razie obwieść „nie 7”.
Ieśli x iest sześć, obwieść „sześć”. A ieśli nie, obwieść „nie sześć”.
```

Jeśli kolejny warunek dotyczy tej samej rzeczy co pierwszy, podmiot można pominąć:

```
Ieśli liczba iest podzielna przez piętnaście, obwieść „FizzBuzz”,
á ieśli podzielna przez trzy, obwieść „Fizz”,
inaczey obwieść liczbę.
```

### Porównania i logika

| Zapis | Znaczenie |
|---|---|
| `x iest pięć`, `x iest równy pięć`, `x = 5` | równość |
| `x nie iest pięć`, `x iest różny od pięciu`, `x != 5` | nierówność |
| `x iest większy niż y`, `x iest więcey niż y`, `większy od y`, `x przewyższa y`, `x > y` | większe |
| `x iest mnieyszy niż y`, `x iest mniey niż y`, `x < y` | mniejsze |
| `x iest większy albo równy y`, `x iest co naymniey y`, `x >= y` | ≥ |
| `x iest mnieyszy albo równy y`, `x iest co naywyżey y`, `x <= y` | ≤ |
| `x iest podzielny przez y` | dzieli się bez reszty |
| `x iest parzysty`, `nieparzysty`, `dodatni`, `ujemny` | własności liczby |
| `x iest pusty` | puste pismo albo regestr |
| `zbiór zawiera x`, `x iest w zbiorze` | zawieranie (regestr albo fragment pisma) |
| `a i b`, `a oraz b`, `a albo b`, `a lub b`, `nie a` | logika |

Słowa *większy, mniejszy, równy, parzysty…* działają w każdym rodzaju i przypadku
(*większa, większe, równej…*). Pisma porównuje się alfabetycznie, bajt po bajcie.

### Pętle

```
Dla każdego n od iednego do dziesięciu: …
Dla każdey liczby od dwóch do stu, czyń: …
Dla każdego k od dziesięciu do iednego co minus trzy: …
Dla każdego żaka z żaków: …                 (także: „ze”, „w”, „wśród”)
Póki dzielnik nie iest równy zeru, czyń: …
Póki x iest większe niż dwa: …
Powtórz trzy razy: …
Zawsze czyń: …                               (albo: „Bez końca czyń: …”)
```

- `Dla każdego … od A do B` liczy włącznie z `B`.
- Kierunek wybiera się sam tylko wtedy, gdy obie granice są stałymi liczbami
  („od dziesięciu do iednego” liczy w dół). W każdym innym razie krok wynosi +1, a przedział
  pusty (np. od 1 do 0) nie wykona się ani razu. Krok podaje się przez `co`.
- Pętla `Dla każdego … z regestru` przechodzi po elementach regestru albo po literach pisma.
- `Przerwij.` (też: `dość`, `przestań`) kończy pętlę.
- `Pomiń.` (też: `dalej`, `opuść`) przechodzi do następnego obrotu.

### Działania

| Zapis | Znaczenie |
|---|---|
| `a plus b`, `a dodać b`, `a + b` | dodawanie; pisma się skleja, regestry łączy |
| `a minus b`, `a bez b`, `a odjąć b`, `a - b` | odejmowanie |
| `a razy b`, `a pomnożone przez b`, `a * b` | mnożenie; `„ab” razy trzy` powiela pismo |
| `a podzielone przez b`, `a / b` | dzielenie |
| `a modulo b`, `reszta z dzielenia a przez b` | reszta; zawsze ma znak dzielnika, jak w Pythonie |
| `a do potęgi b`, `a do kwadratu`, `a do sześcianu` | potęgowanie |
| `minus a` | liczba przeciwna |
| `(…)` | nawiasy |

Mnożenie i dzielenie wiążą mocniej niż dodawanie. Porównania wiążą słabiej niż działania,
a „i”/„albo” najsłabiej.

### Sortowanie i usuwanie

```
Posortuj liczby.                             (rosnąco)
Posortuj liczby malejąco.
Posortuj żaki według średniey malejąco.      (regestr rodów według pola)
Usuń ostatni element z listy.
Usuń pierwszy element z listy.
Usuń element trzy z listy.
```

Sortowanie jest stabilne. Zmienia regestr w miejscu.

### Pliki

```
Zapisz raport do pliku „raport.txt”.         (nadpisuje)
Dopisz wiersz do pliku „dziennik.txt”.       (dopisuje na końcu)
Niechay tekst będzie treść pliku „dane.txt”.
Niechay wiersze będą wiersze pliku „dane.txt”.
```

Zapisanie regestru daje osobny wiersz dla każdego elementu, a zapisanie innej rzeczy
daje jeden wiersz. `wiersze pliku` zwraca regestr pism bez znaków końca wiersza.

---

## 6. Uczynki, rody i księgi

### Uczynki (funkcje)

```
Uczynek silnia, przyimuiący liczbę:
    Ieśli liczba iest mnieysza niż dwa, oddaj ieden.
    Oddaj liczbę razy silnię z liczby bez iednego.

Uczynek naywiększy wspólny dzielnik, przyimuiący pierwszą liczbę i drugą liczbę:
    …

Uczynek utworzenie żaka z wiersza:
    …
```

Nagłówek ma postać `Uczynek <miano>[,] [przyimuiący|biorący|z|o|dla|od|na] <rzecz>[, <rzecz> i <rzecz>]:`.
Uczynek bez rzeczy: `Uczynek powitanie:`.

Przyimek albo `przyimuiący` pisze się **raz**: `Uczynek utworzenie tomu z wiersza:` albo
`Uczynek utworzenie tomu, przyimuiący wiersz:`, ale nie oba naraz.

- `Oddaj X.` zwraca wartość, a samo `Oddaj.` kończy uczynek bez wartości.
- Uczynki spisuje się tylko na wierzchu programu, nie wewnątrz bloków. Mogą stać
  przed miejscem użycia albo po nim.
- Rekurencja działa.

**Wywołanie:** miano uczynku w dowolnym przypadku, potem opcjonalny przyimek
(`z`, `dla`, `od`, `o`, `na`) i rzeczy oddzielone przecinkiem, „i” albo „oraz”:

```
Obwieść silnię z pięciu.
Niechay d będzie naywiększy wspólny dzielnik z czterdziestu i sześćdziesięciu.
Uczyń powitanie.
Uczyń przywitanie gościa z „Jana”.
silnia z trzech.                  (wywołanie jako osobne zdanie, wynik przepada)
```

Skryba zna liczbę rzeczy, które uczynek przyjmuje, więc sam wie, gdzie kończy się
wywołanie, i nawiasy nie są potrzebne. Każda rzecz jest wyrażeniem z działaniami, ale bez
porównań: w `silnia z n minus ieden` rzeczą jest `n minus ieden`.

### Uczynki wbudowane

| Miano | Przykład | Wynik |
|---|---|---|
| pierwiastek | `pierwiastek z dwóch` | `1,414213562` (z kwadratu liczby całkowitej wychodzi liczba całkowita) |
| wartość bezwzględna | `wartość bezwzględna z minus trzy` | `3` |
| zaokrąglenie | `zaokrąglenie z 2,6` | `3` |
| część całkowita | `część całkowita z 2,6` | `2` |
| wartość liczbowa | `wartość liczbowa z „42”` | `42` |
| losowa liczba | `losowa liczba od iednego do sześciu` | liczba od 1 do 6 włącznie |
| wielkie litery | `wielkie litery z „żółć”` | `ŻÓŁĆ` |
| małe litery | `małe litery z „ŻÓŁĆ”` | `żółć` |
| rozbiór | `rozbiór „Jan Rej 5 4”` | `[„Jan”; „Rej”; 5; 4]` (dzieli po spacjach, liczby rozpoznaje) |
| rozcięcie | `rozcięcie „Psałterz; Kochanowski; 1579” przez „;”` | `[„Psałterz”; „Kochanowski”; 1579]` (tnie po wskazanym znaku, spacje po bokach odrzuca, liczby rozpoznaje) |
| treść pliku | `treść pliku „a.txt”` | całe pismo |
| wiersze pliku | `wiersze pliku „a.txt”` | regestr wierszy |
| największy | `największy z liczb` | największy element |
| najmniejszy | `najmniejszy z liczb` | najmniejszy element |
| kopia | `kopia listy` | płytka kopia regestru albo rodu |
| złączenie | `złączenie liczb przez „, ”` | `1, 2, 3` |

Regestry i rody przekazuje się **przez odniesienie**: zmiana wewnątrz uczynku jest
widoczna na zewnątrz. Niezależną kopię daje `kopia`.

### Rody (struktury)

```
Ród Żak ma miano, nazwisko, stopnie i średnią.
```

Ród spisuje się na wierzchu programu. Jego pola wylicza się po `ma` (albo po dwukropku),
oddzielając je przecinkami albo „i”.

**Tworzenie:** `nowy <Ród> o <pole> <wartość>, <pole> <wartość> i <pole> <wartość>`.
Po miano pola można wstawić „równy” albo „jako”. Pola pominięte mają wartość `nic`.

```
Niechay student będzie nowym Żakiem o mianie „Jan”, nazwisku „Kochanowski”
    i średniey równey pięć.
```

**Pola** czyta się, stawiając miano pola **przed** rzeczą, jak w polszczyźnie:

```
Obwieść miano studenta.
Odtąd średnia studenta iest cztery.
Obwieść „{miano żaka} {nazwisko żaka}”.
```

### Księgi (wiele plików)

Większy program można rozpisać na kilka plików. Plik z uczynkami i rodami to **księga**.
Pismo główne powołuje ją jednym zdaniem:

```
Powołując się na „żacy.sar”.
```

Od tej chwili uczynki i rody z księgi działają tak, jakby spisano je w piśmie głównym.
Woła się je po mianie, z całą odmianą.

- Księga to zwykły plik `.sar` zakończony `Amen.`, ale zawiera **tylko** uczynki, rody
  i przypisy. Rozkaz na wierzchu księgi (np. `Obwieść …`) to błąd, bo rozkazy należą do
  pisma głównego.
- Ścieżkę liczy się od pliku, który powołuje. Końcówkę `.sar` wolno pominąć:
  `Powołując się na „księgi/rachunki”.`
- Kilka ksiąg naraz: `Powołując się na „żacy.sar”, „rachunki.sar” i „druk.sar”.`
- Księga może powoływać inne księgi. Ta sama księga powołana dwa razy wczytuje się raz.
  Dwie księgi, które powołują siebie nawzajem, to *błędne koło powołań*, i skryba to wytknie.
- Powołanie stoi na wierzchu pisma (nie w bloku), najlepiej na początku.
- Grzechy wskazują właściwy plik, także w trakcie działania:
  `Grzech śmiertelny w „księgi/rachunki.sar”, wierſzu 11: …`.

Przykład: `przyklady/dziennik/dziennik.sar` powołuje `przyklady/dziennik/żacy.sar`.

---

## 7. Ściąga

```
PROGRAM     := { powołanie } { zdanie | uczynek | ród } "Amen."
KSIĘGA      := { powołanie } { uczynek | ród } "Amen."
powołanie   := "Powołując się na" CYTAT { ("," | "i") CYTAT } "."
zdanie      := rozkaz { ("," | "i" | "á potem" | "po czym") rozkaz } "."
             | złożone
złożone     := nagłówek ":" WCIĘCIE { zdanie } WYCIĘCIE
             | nagłówek ("," | ":") rozkaz { łącznik rozkaz } "."
nagłówek    := "Ieśli" warunek | "Póki" warunek ["czyń"] | "Zawsze" ["czyń"]
             | "Bez końca" ["czyń"] | "Powtórz" wyr "razy"
             | "Dla każdego" miano "od" wyr "do" wyr ["co" wyr] ["czyń"]
             | "Dla każdego" miano ("z"|"w") wyr ["czyń"]
uczynek     := "Uczynek" miano [","] [("przyimuiący"|"z"|"o"|"dla"|"od"|"na") miano {("," | "i") miano}] ":" ciało
ród         := "Ród" miano ("ma" | ":") pole {("," | "i") pole} "."
```

| Chcę… | Piszę |
|---|---|
| zmienna | `Niechay x będzie 5.` |
| zmiana | `Odtąd x iest 6.` / `Zwiększ x o 2.` |
| wypisać | `Obwieść „x =”, x.` |
| wczytać | `Zapytaj „Podaj: ” o x.` |
| if/elif/else | `Ieśli … : … / A ieśli … : … / Inaczey: …` |
| for range | `Dla każdego i od 1 do n: …` *(uwaga: `i` jest zakazane, użyj `k`, `numer`…)* |
| for each | `Dla każdego el z listy: …` |
| while | `Póki x iest większe niż 0: …` |
| funkcja | `Uczynek f, przyimuiący a i b: … Oddaj …` |
| wywołanie | `f z 1 i 2` |
| lista | `regestr z 1, 2 i 3`, `regestr pusty` |
| append | `Dodaj x do listy.` |
| indeks | `element k z listy`, `pierwszy element listy`, `ostatni element listy` |
| długość | `długość listy` |
| struktura | `Ród Punkt ma szerokość i wysokość.` / `nowy Punkt o szerokości 1 i wysokości 2` |
| pole | `szerokość punktu` |
| sortowanie | `Posortuj listę według pola malejąco.` |
| plik | `Zapisz x do pliku „a.txt”.`, `wiersze pliku „a.txt”` |
| komentarz | `Przypis: …` / `Rozdział …` na początku wiersza |
| import | `Powołując się na „biblioteka.sar”.` |
| koniec | `Amen.` |

---

## 8. Pułapki

Najczęstsze błędy, zwłaszcza przy pisaniu przez model językowy:

1. **Brak `Amen.`** na końcu programu.
2. **Słowa skryby jako miana.** Mianem nie może być żadne słowo kluczowe ani jego
   odmiana. Najczęściej wpadają:
   - `i`, `co`, `do`, `od`, `z`, `w`, `o`, `na`, `po`, `ma`, `raz`, `bez`, `nad`
   - `pusty/pusta`, `nowy/nowa`, `pełny`, `każdy`, `wszystkie`, `równy`, `różny`
   - `większy`, `mniejszy`, `parzysty`, `dodatni`, `ujemny`
   - `pierwszy`…`dziesiąty` i `ostatni` (same, bez rzeczownika za nimi)
   - `element`, `wyraz`, `długość`, `rozmiar`, `regestr`/`rejestr`, `plik`, `wspak`
   - `prawda`, `fałsz`, `nic`, `zero` i wszystkie liczebniki słowne
   - `gdy`, `kiedy`, `skoro`, `dalej` (to „pomiń”!), `dość`, `weź`, `pobierz`, `przyjmij`, `potem`, `następnie`
   - wszystkie wypełniacze z rozdziału 2 (`to`, `teraz`, `więc`, `też`, `jeszcze`…)

   Skryba wytknie taki błąd wprost: *„Słowo „pusty” ieſt ſłowem ſkryby i miánem rzeczy być nie może.”*
3. **Dwie rzeczy o tym samym rdzeniu**, np. `liczba` i `liczby` jako osobne zmienne. To
   jedna rzecz. Dla regestru i jego elementu wybieraj różne słowa: `oceny` i `nota`,
   `żaki` i `uczeń`, `liczby` i `kandydat`.
4. **Rzeczowniki nieregularne** (`imię`, `skarbiec`, `dzień`, `żacy`). Zobacz tabelę w rozdziale 3.
5. **Indeks od zera.** Pierwszy element to `element ieden`; `element zero` to błąd.
6. **Zmienna pętli zakryta przez pole albo ród o tym samym rdzeniu.** Pole o rdzeniu
   zmiennej może przejąć słowo, jeśli stoi za nim coś, co wygląda jak rzecz. Nadawaj polom
   inne słowa niż zmiennym.
7. **`Oddaj` poza uczynkiem, `Przerwij`/`Pomiń` poza pętlą.** To błędy.
8. **Uczynek albo ród wewnątrz bloku.** Spisuje się je tylko na wierzchu.
9. **Porównanie jako rzecz uczynku.** `f z x iest 5` czyta się jako `(f z x) iest 5`.
   Żeby przekazać prawdę albo fałsz, wylicz ją najpierw do zmiennej.
10. **Działanie po wywołaniu wchodzi do rzeczy uczynku.** `silnia z n minus ieden` to
    `silnia(n - 1)`, a nie `silnia(n) - 1`. Żeby działać na wyniku, weź wywołanie w nawias:
    `(silnia z n) minus ieden`.
11. **Podkreślniki w mianach** (`moja_lista`). To błąd; pisz `moia lista`.
12. **Kropka w środku zdania.** Ułamki pisz z przecinkiem albo kropką bez spacji (`3,14`).
    Kropka ze spacją kończy zdanie.

---

## 9. Styl, czyli jak pisać kronikę, a nie kod

Skryba przyjmie i suchy, techniczny tekst, ale wtedy cały urok języka przepada.
Sarmata ma się czytać jak kronika. Kilka zasad:

| Zamiast (nawyk programisty) | Pisz (kronikarz) |
|---|---|
| `Odtąd utarg iest utarg plus wpłata.` | `Dodaj wpłatę do utargu.` |
| `Odtąd ilość towaru iest (ilość towaru) minus popyt.` | `Zmniejsz ilość towaru o popyt.` |
| `Niechay nowy wpis będzie nowym Towarem o …` + `Dodaj nowy wpis do kramu.` | `Dodaj nowy Towar o nazwie …, cenie … i ilości … do kramu.` |
| `Niechay licznik braków będzie zero.` | `Niechay braków będzie zero.` albo `Niechay liczba braków będzie zero.` |
| `Ieśli zakup iest nic: Przerwij.` (blok na jedną linię) | `Ieśli zakup iest nicością, przerwij.` |
| `Obwieść „[BRAK] Nie masz towaru:”, nazwa, „!”.` | `Obwieść „Nie masz w składzie towaru «{nazwa}»!”.` (wstawka zamiast sklejania) |
| `ilosc`, `sciezka`, `dochod` | `ilość`, `ścieżka`, `dochód` (polskie litery są w porządku) |
| `wczytanie_zapasow` | `wczytanie zapasów` |

- **Miana to rzeczowniki z epoki:** *kram, skład, żak, kronika, skarb, utarg, należność,
  godność* (imię), *miano*. Uczynki to rzeczowniki odczasownikowe: *wyliczenie średniey,
  odszukanie towaru, spisanie raportu*. Wtedy wywołanie czyta się samo: *„Niechay zakup
  będzie odszukaniem towaru z kramu…”*.
- **Nawiasów prawie nigdy nie trzeba.** Pole stoi przed rzeczą (`cena towaru`), działania
  mają pierwszeństwo jak w matematyce. Nawias jest potrzebny tylko po wywołaniu uczynku
  (pułapka 10).
- **Rozdziały jako przypisy:** *„Rozdział wtóry, w którym kupiec towar wydaie.”*
  Porządkują program i bawią czytelnika.
- **Wypełniacze dodają rytmu:** *tedy, zatem, przeto, oto, waszmość*.
- **Łącz krótkie rozkazy w jedno zdanie:** *„Zmniejsz ilość towaru o popyt, á potem dodaj
  należność do utargu.”*
- **Dialog pisz czasownikiem opowieści:** *„Rzekł tedy rotmistrz, gładząc wąsa: „— …””*
  zamiast `Obwieść „…”`. Zob. „Mowa opowieści” w rozdziale 5.
- **Licz rzeczy, nie długości:** *„Gdy gąb iest więcey niż sucharów…”*.
- **Pomijaj podmiot w kolejnych warunkach:** *„Ieśli liczba iest podzielna przez trzy,
  …, á ieśli podzielna przez pięć, …”*.
- **Staropolska pisownia jest opcjonalna**, ale *ieśli, niechay, mnieyszy, á* robią klimat.

---

## 10. Pełny przykład

Fragment `przyklady/dziennik/dziennik.sar`, typowego projektu semestralnego:

```
Ród Żak ma miano, nazwisko, stopnie i średnią.

Uczynek wyliczenie średniey, przyimuiący noty:
    Ieśli noty są puste, oddaj zero.
    Niechay suma będzie zerem.
    Dla każdey noty z not, dodaj notę do sumy.
    Oddaj sumę podzieloną przez długość not.

Uczynek utworzenie żaka z wiersza:
    Niechay słowa będą rozbiorem wiersza.
    Niechay noty będą regestrem pustym.
    Dla każdego numeru od trzech do długości słów, dodaj element numer ze słów do not.
    Oddaj nowego Żaka o mianie pierwszym ze słów, nazwisku drugim ze słów,
        stopniach równych notom i średniey równey wyliczeniu średniey z not.

Uczynek okazanie rankingu, przyimuiący spis:
    Posortuj spis według średniey malejąco.
    Niechay mieysce będzie ieden.
    Dla każdego żaka ze spisu:
        Obwieść „{mieysce}. {miano żaka} {nazwisko żaka} — średnia {średnia żaka}”.
        Zwiększ mieysce.

Niechay żaki będą regestrem pustym.
Dla każdego wiersza z wierszy pliku „oceny.txt”, dodaj utworzenie żaka z wiersza do żaków.
Uczyń okazanie rankingu z żaków.

Amen.
```

Więcej przykładów jest w katalogu `przyklady/`.
