# Rozmówki, czyli Sarmata dla programisty

Ściąga dla człowieka, który myśli po angielsku albo w Pythonie, a chce pisać po sarmacku.
Najpierw jest słowniczek pojęć, potem tabele „to, co znasz → jak to napisać”, a na końcu
pełna lista słów, które skryba rozumie, wraz z synonimami i słowami, które pomija.

Pełny opis języka jest w [statucie](statut.md). Tu są tylko tłumaczenia.

---

## Zanim zaczniesz: pisownia

Skryba sprowadza każde słowo do wspólnej postaci, zanim je porówna. Dlatego w tabelach
niżej słowa stoją we współczesnej pisowni, ale każda z tych form działa tak samo:

| Piszesz | Skryba czyta | Przykład: to samo słowo |
|---|---|---|
| `j`, `y`, `i` | `i` | `jeśli` = `ieśli` = `yeśli`, `niechaj` = `niechay` |
| `v`, `u` | `u` | `uczynek` = `vczynek` |
| `ſ`, `ś`, `s` | `s` | `jeśli` = `ieſli` |
| `á`, `ą`, `a` | `a` | `będą` = `będá` |
| `é`, `ę`, `e` | `e` | `będzie` = `bedzie` |
| `ó`, `o` / `ć`, `c` / `ź`, `ż`, `z` / `ł`, `l` / `ń`, `n` | bez kresek | `większy` = `wiekszy` |
| wielkie i małe litery | małe | `JEŚLI` = `jeśli` |

Wyjątek: liczby rzymskie (`MDCX`) muszą być pisane wielkimi literami.

Odmiana też działa sama: `liczba`, `liczbę`, `liczby`, `liczbą` to jedna zmienna.

---

## Słowniczek pojęć

| Po angielsku | Po sarmacku | Uwagi |
|---|---|---|
| compiler | **skryba** | program `sarmata` |
| program, source file | **pismo** | plik `.sar` |
| string | **pismo** | to samo słowo co plik źródłowy, z kontekstu wiadomo, o co chodzi |
| variable name, identifier | **miano** | może mieć kilka słów: `naywiększy wspólny dzielnik` |
| declaring a variable | **chrzest** | `Niechay x będzie 5.` |
| value, argument | **rzecz** | |
| statement | **rozkaz** / **zdanie** | kończy się kropką |
| block | **blok** | dwukropek + wcięcie, jak w Pythonie |
| comment | **przypis** / **glosa** | cały wiersz, patrz niżej |
| list / array | **regestr** | liczony od jedynki |
| dictionary / map | **spis** | |
| struct / class | **ród** | same pola, bez metod |
| field | **pole** | |
| object / instance | **nowy Żak**, **nowy Punkt** | `nowy` + miano rodu |
| function | **uczynek** | |
| function parameter | rzecz, którą uczynek **przyjmuje** | |
| return | **oddaj** | |
| builtin function | **uczynek wbudowany** | |
| null / None | **nic**, **nicość** | |
| boolean | **prawda albo fałsz** | |
| float | **ułamek** | |
| integer | **liczba całkowita** | |
| type | **rodzaj** | |
| import / include | **powołanie** | `Powołując się na „księga.sar”.` |
| module, library | **księga** | |
| exception | **klątwa** | rzucasz ją rozkazem `Rzuć` |
| caught exception | **bieda** | |
| try | **próba** | `Spróbuj:` |
| compile error | **grzech** (śmiertelny) | |
| runtime | **bieg** | `runtime/sarmata.c` |
| print | **obwieścić** | `Obwieść …` |
| end of program | **Amen.** | obowiązkowe na końcu |

---

## Tabele tłumaczeń

W kolumnie „Sarmata” stoi forma współczesna. Po staremu (`Niechay`, `Ieśli`, `iest`) działa
tak samo.

### Podstawy

| Python | Sarmata |
|---|---|
| `# komentarz` | `Przypis: komentarz.` (wiersz zaczynający się od *Przypis*, *Rozdział*, *Glossa*…) |
| koniec programu | `Amen.` |
| `x = 5` (nowa zmienna) | `Niechaj x będzie pięć.` / `Niechaj x będzie 5.` / `Niechaj x = 5.` |
| `x = 6` (zmiana) | `Odtąd x jest sześć.` (`Niechaj` też zmienia, jeśli rzecz już istnieje) |
| `a = b; b = 7` | `Odtąd a jest b, a b jest siedem.` (kolejne nadania bez powtarzania „Odtąd”; liczone po kolei, nie jednocześnie jak `a, b = …`) |
| `x += 1` | `Zwiększ x.` |
| `x += 3` | `Zwiększ x o trzy.` |
| `x -= 1` | `Zmniejsz x.` |
| `x -= 2` | `Zmniejsz x o dwa.` / `Odejmij dwa od x.` |
| `x *= 2` | `Pomnóż x przez dwa.` |
| `x /= 8` | `Podziel x przez osiem.` |
| `"Ala"`, `f"x = {x}"` | `„Ala”`, `„x = {x}”` (też `"…"`, `«…»`) |
| `"\n"` | `„\n”` |
| `None`, `True`, `False` | `nic`, `prawda`, `fałsz` |
| `3.14` | `3,14` albo `3.14` (bez spacji po kropce!) |
| `0.5`, `1.5`, `2.5` | `pół`, `półtora`, `dwa i pół` |

### Wejście i wyjście

| Python | Sarmata |
|---|---|
| `print(x)` | `Obwieść x.` (też `Wypisz x.`, `Powiedz x.`) |
| `print("Wynik:", x)` | `Obwieść „Wynik:”, x.` |
| `print(x, end="")` | `Obwieść x nie kończąc wiersza.` |
| `print()` | `Obwieść.` |
| `x = input("Imię? ")` | `Zapytaj „Imię? ” o x.` |
| `x = input()` | `Zapytaj o x.` |
| `x = int(input())` | `Zapytaj o x.` (liczba sama staje się liczbą) |
| jedno słowo, jak `cin >> x` | `Wczytaj x.` |
| koniec wejścia (EOF) | `x` dostaje `nic`: `Póki x nie jest nicością: …` |
| `print` w opowieści | `Rzekł tedy rotmistrz, gładząc wąsa: „— Witajcie!”` (narracja przed dwukropkiem jest pomijana) |

### Działania

| Python | Sarmata |
|---|---|
| `a + b` | `a plus b`, `a dodać b`, `a + b` |
| `a - b` | `a minus b`, `a bez b`, `a odjąć b`, `a - b` |
| `a * b` | `a razy b`, `a pomnożone przez b`, `a * b` |
| `a / b` | `a podzielone przez b`, `a / b` (samo `a przez b` nie działa) |
| `a % b` | `a modulo b`, `reszta z dzielenia a przez b` |
| `a ** b` | `a do potęgi b` |
| `a ** 2`, `a ** 3` | `a do kwadratu`, `a do sześcianu` |
| `-a` | `minus a` |
| `(a + b) * c` | `(a plus b) razy c` |
| `"ab" * 3` | `„ab” razy trzy` |

### Porównania i logika

| Python | Sarmata |
|---|---|
| `x == 5` | `x jest pięć`, `x jest równy pięć`, `x = 5` |
| `x != 5` | `x nie jest pięć`, `x jest różny od pięciu`, `x != 5` |
| `x > y` | `x jest większy niż y`, `x jest więcej niż y`, `x przewyższa y`, `x > y` |
| `x < y` | `x jest mniejszy niż y`, `x jest mniej niż y`, `x < y` |
| `x >= y` | `x jest co najmniej y`, `x jest większy albo równy y` |
| `x <= y` | `x jest co najwyżej y`, `x jest mniejszy albo równy y` |
| `x % 2 == 0` | `x jest parzysty` |
| `x % 2 == 1` | `x jest nieparzysty` |
| `x % 3 == 0` | `x jest podzielny przez trzy` |
| `x > 0`, `x < 0` | `x jest dodatni`, `x jest ujemny` |
| `len(x) == 0` | `x jest pusty` |
| `v in lista` | `liczby zawierają v`, `v jest w liczbach`, `spis ma v` |
| `a and b` | `a i b`, `a oraz b` (w nagłówku `Jeśli`/`Póki` także `a`) |
| `a or b` | `a albo b`, `a lub b` |
| `not a` | `nie a` |

Przymiotniki odmieniają się dowolnie: `większy`, `większa`, `większe`, `większej`…

### Warunki

| Python | Sarmata |
|---|---|
| `if x > 3:` | `Jeśli x jest większe niż trzy:` (też `jeżeli`, `gdy`, `kiedy`, `skoro`) |
| `elif x > 1:` | `A jeśli x jest większe niż jeden:` / `Azali x jest większe niż jeden:` |
| `else:` | `Inaczej:` / `A inaczej:` / `W przeciwnym razie:` / `A jeśli nie:` |
| `if x: print(1)` w jednej linii | `Jeśli x jest prawdą, obwieść 1.` |
| `if a > 3 and a < 10:` | `Jeśli a jest większe niż trzy, a mniejsze niż dziesięć:` |

Ciąg `if/elif/else` w jednym zdaniu:

```
Jeśli liczba jest podzielna przez piętnaście, obwieść „FizzBuzz”,
a jeśli podzielna przez trzy, obwieść „Fizz”,
inaczej obwieść liczbę.
```

### Pętle

| Python | Sarmata |
|---|---|
| `for i in range(1, 11):` | `Dla każdego numeru od jednego do dziesięciu:` (**włącznie** z 10) |
| `for i in range(10, 0, -1):` | `Dla każdego numeru od dziesięciu do jednego:` (stałe granice: kierunek sam) |
| `for i in range(0, 100, 5):` | `Dla każdego numeru od zera do stu co pięć:` |
| `for gosc in goscie:` | `Dla każdego gościa z gości:` (też `ze`, `w`, `wśród`; miano pętli nie może być słowem skryby, np. `element`) |
| `for c in "abc":` | `Dla każdej litery z „abc”:` |
| `for k in slownik:` | `Dla każdego klucza ze spisu:` (po kluczach) |
| `for _ in range(3):` | `Powtórz trzy razy:` |
| `while x > 0:` | `Póki x jest większe niż zero:` (też `dopóki`, opcjonalnie `, czyń`) |
| `while True:` | `Zawsze czyń:` / `Bez końca czyń:` |
| `break` | `Przerwij.` (też `dość`, `przestań`, `zaniechaj`) |
| `continue` | `Pomiń.` (też `dalej`, `opuść`) |

Uwaga: `i` jest spójnikiem, więc nie może być mianem. Zamiast `i` pisz `numer`, `licznik`, `k`.

### Funkcje

| Python | Sarmata |
|---|---|
| `def powitanie():` | `Uczynek powitanie:` |
| `def silnia(n):` | `Uczynek silnia, przyjmujący liczbę:` / `Uczynek silnia z liczby:` |
| `def nwd(a, b):` | `Uczynek nwd, przyjmujący pierwszą liczbę i drugą liczbę:` |
| `return x` | `Oddaj x.` (też `zwróć`) |
| `return` | `Oddaj.` |
| `y = silnia(5)` | `Niechaj y będzie silnią z pięciu.` |
| `nwd(40, 60)` | `nwd z czterdziestu i sześćdziesięciu` |
| `powitanie()` jako osobne zdanie | `Uczyń powitanie.` (też `wykonaj`, `wezwij`, `wywołaj`) |
| `f(n) - 1` | `(f z n) minus jeden` (bez nawiasu `f z n minus jeden` to `f(n - 1)`) |

Przyimek przy wywołaniu jest dowolny: `z`, `dla`, `od`, `o`, `na`, albo żaden.

### Listy (regestry)

| Python | Sarmata |
|---|---|
| `[]` | `regestr pusty`, `pusty regestr`, `nowy regestr` |
| `[5, 3, 8]` | `regestr z pięciu, trzech i ośmiu` |
| `[0] * 5` | `regestr o długości pięciu pełen zer` |
| `lista[0]` | `pierwszy element listy`, `element pierwszy z listy`, `element jeden z listy` |
| `lista[k-1]` | `element k z listy` (liczy się **od jedynki**) |
| `lista[-1]` | `ostatni element listy` |
| `lista[1] = 9` | `Odtąd drugi element listy jest dziewięć.` |
| `len(lista)` | `długość listy` (też `rozmiar`) |
| `lista.append(x)` | `Dodaj x do listy.` |
| `lista.pop()` | `Usuń ostatni element z listy.` |
| `del lista[2]` | `Usuń element trzy z listy.` (trzeci element!) |
| `lista.remove(v)` | `Usuń v z listy.` |
| `lista.sort()` | `Posortuj listę.` |
| `lista.sort(reverse=True)` | `Posortuj listę malejąco.` |
| `lista.sort(key=lambda z: z.srednia)` | `Posortuj listę według średniej.` |
| `lista[::-1]` | `lista wspak` |
| `max(lista)`, `min(lista)` | `największy z listy`, `najmniejszy z listy` |
| `lista.copy()` | `kopia listy` |
| `", ".join(lista)` | `złączenie listy przez „, ”` |
| `a + b` (sklejenie list) | `a plus b` |

### List comprehension i filter

| Python | Sarmata |
|---|---|
| `[x*x for x in range(1, 6)]` | `x razy x dla każdego x od jednego do pięciu` |
| `[n+1 for n in liczby]` | `n plus jeden dla każdego n z liczb` |
| `[x for x in liczby if x > 4]` | `wszystkie liczby, które są większe niż cztery` |
| `[x for x in range(1, 51) if x % 3 == 0]` | `wszystkie liczby od 1 do 50, które są podzielne przez trzy` |
| `[z for z in zaki if z.srednia > 4]` | `wszyscy z żaków, których średnia jest większa niż cztery` |
| `[s for s in slowa if len(s) > 4]` | `wszystkie słowa, których długość jest większa niż cztery` |
| `[z.miano for z in zaki]` | `miano żaka dla każdego żaka z żaków` |
| `[x for x in liczby if x % 2]` (dowolny warunek) | `wszystkie x z liczb, gdzie x jest nieparzyste` |

Warunki łączysz `i` albo `albo`: `…, które są parzyste i większe niż dwa`.

### Słowniki (spisy)

| Python | Sarmata |
|---|---|
| `{}` | `spis pusty`, `pusty spis`, `nowy spis` (też `słownik`) |
| `{"chleb": 3, "miód": 5}` | `spis, gdzie „chleb” ma trzy, a „miód” ma pięć` |
| `d["chleb"]` | `ceny pod „chleb”` (gdy klucza nie ma: `nic`) |
| `d["sól"] = 1` | `Odtąd ceny pod „sól” jest jeden.` |
| `d["chleb"] += 2` | `Zwiększ ceny pod „chleb” o dwa.` (brakujący klucz zaczyna od zera) |
| `d[k].append(x)` (gdy pod `k` leży regestr) | `Dodaj x do gości pod „karczma”.` |
| `"chleb" in d` | `ceny mają „chleb”`, `ceny zawierają „chleb”` |
| `del d["chleb"]` | `Usuń „chleb” z cen.` |
| `list(d.keys())`, `list(d.values())` | `klucze z cen`, `wartości z cen` |
| `len(d)` | `długość cen` |

### Napisy (pisma)

| Python | Sarmata |
|---|---|
| `""` | `puste pismo` |
| `len(s)` | `długość s` (po literach, `„żółć”` ma 4) |
| `s[0]` | `pierwszy element s` (pojedyncza litera) |
| `s[::-1]` | `s wspak`, `s czytane wspak` |
| `s.upper()`, `s.lower()` | `wielkie litery z s`, `małe litery z s` |
| `s.split()` | `rozbiór s` (liczby same stają się liczbami) |
| `s.split(";")` | `rozcięcie s przez „;”` |
| `int(s)`, `float(s)` | `wartość liczbowa z s` |
| `"ab" in s` | `s zawiera „ab”` |
| `str(x)` | `„{x}”` |

### Liczby

| Python | Sarmata |
|---|---|
| `math.sqrt(x)` | `pierwiastek z x` |
| `abs(x)` | `wartość bezwzględna z x` |
| `round(x)` | `zaokrąglenie z x` |
| `int(x)` (obcięcie) | `część całkowita z x` |
| `random.randint(1, 6)` | `losowa liczba od jednego do sześciu` |
| liczba rzymska | `rok rzymskimi`, `rok po rzymsku`, `rok cyframi rzymskimi` → `MDCX` |
| liczba słownie | `suma słownie` → `dwa tysiące dwadzieścia sześć` (po staropolsku) |

### Rodzaje (typy)

| Python | Sarmata |
|---|---|
| `type(x).__name__` | `rodzaj z x` |
| `isinstance(x, (int, float))` | `x jest liczbą` |
| `isinstance(x, int)` | `x jest liczbą całkowitą` |
| `isinstance(x, float)` | `x jest ułamkiem` |
| `isinstance(x, str)` | `x jest pismem` |
| `isinstance(x, list)` | `x jest regestrem` |
| `isinstance(x, dict)` | `x jest spisem` |
| `isinstance(x, bool)` | `x jest prawdą albo fałszem` |
| `x is None` | `x jest nicością` |
| `isinstance(x, Zak)` | `x jest Żakiem` |

### Struktury (rody)

| Python | Sarmata |
|---|---|
| `class Zak:` z polami | `Ród Żak ma miano, nazwisko i średnią.` |
| `Zak(miano="Jan", srednia=5)` | `nowy Żak o mianie „Jan” i średniej równej pięć` |
| `z.miano` | `miano żaka` (pole stoi **przed** rzeczą) |
| `z.srednia = 4` | `Odtąd średnia żaka jest cztery.` |
| `z.kiesa -= 2` | `Zmniejsz kiesę gościa o dwa.` |
| `copy.copy(z)` | `kopia żaka` |

Regestry, spisy i rody przekazuje się przez odniesienie, jak w Pythonie.

### Wyjątki (klątwy)

| Python | Sarmata |
|---|---|
| `try:` | `Spróbuj:` (też `próbuj`) |
| `except:` | `A w razie biedy:` / `A jeśli bieda:` / `Na wypadek klęski:` / `Biada:` |
| `except Exception as e:` | `A w razie biedy, zwanej e:` (też `nazwanej`) |
| `raise Exception("…")` | `Rzuć klątwę „…”.` (też `Rzuć „…”`, `ciśnij`, `miotnij`) |
| try/except w jednej linii | `Spróbuj obwieść x, a w razie biedy obwieść „nie wyszło”.` |
| `ZeroDivisionError`, `IndexError` | też są łapane, bieda to wtedy pismo z komunikatem |

### Pliki

| Python | Sarmata |
|---|---|
| `open(p, "w").write(x)` | `Zapisz x do pliku „raport.txt”.` |
| `open(p, "a").write(x)` | `Dopisz x do pliku „raport.txt”.` |
| `open(p).read()` | `treść pliku „raport.txt”` |
| `open(p).read().splitlines()` | `wiersze pliku „raport.txt”` |
| `os.path.exists(p)` | `plik „raport.txt” istnieje`, `istnieje plik „raport.txt”` |
| `not os.path.exists(p)` | `plik „raport.txt” nie istnieje` |
| `os.remove(p)` | `Usuń plik „raport.txt”.` |
| `os.makedirs(p, exist_ok=True)` | `Utwórz katalog „kronika/stara”.` |
| `os.rename(a, b)` | `Przenieś plik „a.txt” do „b.txt”.` |
| `sorted(os.listdir(p))` | `zawartość katalogu „kronika”` |

### Program jako całość

| Python | Sarmata |
|---|---|
| `import modul` | `Powołując się na „księga.sar”.` |
| `sys.argv[1:]` | `słowa wywołania` |
| `sys.exit(0)` | `Zakończ.` |
| `sys.exit(3)` | `Zakończ z kodem trzy.` |
| `sys.exit(1)` | `Zakończ z grzechem.` |

---

## Wszystkie słowa skryby

Każda grupa to jedno słowo dla skryby: synonimy są wymienne. Słowa podane z myślnikiem
(`każd-`) mają wszystkie końcówki przymiotnika (`każdy`, `każda`, `każdego`, `każdej`…).

**Żadnego z tych słów nie można użyć jako miana zmiennej.**

### Nadawanie i stan

| Rola | Słowa |
|---|---|
| nowa zmienna | `niechaj`, `niech`, `niechże`, `niechajże` |
| „będzie” | `będzie`, `będą` |
| „równy”, `=` | `równ-`, `wynosi`, `wynoszą` |
| „jako” | `jako` |
| zmiana wartości | `odtąd`, `odtądże`, `odtychmiast` |
| „jest”, `==` | `jest`, `są` |

### Mowa i słuch

| Rola | Słowa |
|---|---|
| print | `obwieść`, `obwieśćże`, `ogłoś`, `głoś`, `rzeknij`, `rzeknijże`, `wypisz`, `napisz`, `oznajmij`, `powiedz`, `wyrzeknij`, `zakrzyknij`, `wołaj` |
| print z narracją | `rzekł`, `rzekła`, `rzekli`, `rzekły`, `rzecze`, `odrzekł`, `odrzekła`, `odrzekli`, `powiedział`, `powiedziała`, `powiedzieli`, `odpowiedział`, `odpowiedziała`, `przemówił`, `przemówiła`, `zawołał`, `zawołała`, `zawołali`, `krzyknął`, `krzyknęła`, `krzyknęli`, `zakrzyknął`, `zakrzyknęła`, `zakrzyknęli`, `szepnął`, `szepnęła`, `westchnął`, `westchnęła`, `zaśpiewał`, `zaśpiewała`, `zaśpiewali`, `ozwał`, `ozwała`, `prawił`, `opowiada`, `zapisano` |
| input, cały wiersz | `zapytaj`, `spytaj`, `pytaj`, `zapytajże` |
| input, jedno słowo | `wczytaj`, `pobierz`, `weź`, `przyjmij` |
| bez nowego wiersza | `nie kończąc wiersza` |

Czasowniki z narracją: wszystko między takim czasownikiem a dwukropkiem skryba pomija.
Bez dwukropka działają jak `obwieść`.

### Warunki

| Rola | Słowa |
|---|---|
| if | `jeśli`, `jeżeli`, `jeśliby`, `jeżeliby`, `gdy`, `gdyby`, `kiedy`, `skoro`, `jeśliże` |
| elif | `azali`, `azaliż`, albo `a` + słowo z wiersza „if” |
| else | `inaczej`, `inakszej`, `inaksze`, `w przeciwnym razie` |

### Pętle

| Rola | Słowa |
|---|---|
| while | `póki`, `dopóki`, `dokąd`, `pókiż` |
| „czyń” (ozdobne po nagłówku) | `czyń`, `czyńże`, `rób`, `powtarzaj`, `działaj` |
| forever | `zawsze`, `wiecznie`, `wiekuiście`, `nieustannie`, `bez końca` |
| for | `dla` + `każd-` / `wszelk-` / `wszystk-` / `wszytk-` / `wszyscy` / `wszytcy` |
| repeat | `powtórz`, `powtórzże` … `razy` / `raz` |
| break | `przerwij`, `dość`, `przestań`, `zaniechaj` |
| continue | `pomiń`, `dalej`, `opuść` |

### Próby i klątwy

| Rola | Słowa |
|---|---|
| try | `spróbuj`, `spróbujże`, `próbuj` |
| except | `biada`, `bieda`, `biedy`, `biedzie`, `biedę`, `klęska`, `klęski`, `klęsce`, `klęskę` |
| as | `zwan-`, `nazwan-`, `nazywan-` |
| raise | `rzuć`, `rzućże`, `ciśnij`, `miotnij` (+ opcjonalnie `klątwę`) |
| exit | `zakończ`, `zakończże`, `skończ` |

### Przyimki

| Słowo | Do czego |
|---|---|
| `od`, `ode` | początek zakresu, odejmowanie (`odejmij dwa od x`) |
| `do` | koniec zakresu, cel (`dodaj x do listy`, `zapisz do pliku`) |
| `co` | krok pętli (`co pięć`), `co najmniej` |
| `z`, `ze`, `spośród`, `spomiędzy` | źródło: element z regestru, rzecz uczynku |
| `w`, `we`, `wśród`, `pośród` | jak `z` |
| `o` | o ile (`zwiększ o trzy`), pytanie (`zapytaj o x`), pola nowego rodu |
| `na` | przyimek przy wywołaniu |
| `przez` | mnożenie, dzielenie, separator (`złączenie … przez „, ”`) |
| `pod` | klucz spisu (`ceny pod „chleb”`) |

### Uczynki i rody

| Rola | Słowa |
|---|---|
| def | `uczynek`, `powinność` |
| parametry | `przyjmując-`, `biorąc-`, `żądając-`, `przyjmuje`, `bierze` |
| return | `oddaj`, `oddajże`, `zwróć` |
| wywołanie jako zdanie | `uczyń`, `wykonaj`, `wezwij`, `przywołaj`, `wywołaj` |
| class | `ród` |
| „ma” (pola, klucze, zawieranie) | `ma`, `mają`, `posiada`, `posiadają` |
| new | `now-` |

### Czynności na rzeczach

| Rola | Słowa |
|---|---|
| `+= ` | `zwiększ`, `powiększ` |
| `-=` | `zmniejsz`, `pomniejsz`, `umniejsz` |
| append / dolicz | `dodaj`, `dołóż`, `przydaj`, `dorzuć` |
| odejmij | `odejmij`, `ujmij` |
| `*=` | `pomnóż`, `rozmnóż` |
| `/=` | `podziel` |
| sort | `posortuj`, `uporządkuj`, `ułóż` |
| key= | `według`, `wedle`, `podług` |
| kierunek | `rosnąco`, `malejąco` |
| delete | `usuń`, `wyrzuć` |
| clear screen | `wymaż`, `zmaż` (+ `ekran`) |
| zapis do pliku | `zapisz`, `spisz` (nadpisuje), `dopisz` (na koniec) |
| plik | `plik`, `pliku`, `plikiem`, `pliki`, `plików` |
| mkdir | `utwórz`, `stwórz`, `załóż` (+ `katalog`) |
| rename | `przenieś`, `przesuń` |

### Logika i działania

| Rola | Słowa |
|---|---|
| not | `nie` |
| and | `i`, `oraz`, `tudzież` |
| or | `albo`, `lub`, `bądź` |
| `+` | `plus`, `dodać` |
| `-` | `minus`, `odjąć`, `bez` |
| `/` | `podzielon-`, `dzielon-` (+ `przez`) |
| `*` | `pomnożon-`, `mnożon-` (+ `przez`), `razy` |
| `%` | `modulo`, `reszta z dzielenia … przez …` |
| `**` | `do potęgi`, `do kwadratu`, `do sześcianu` |

### Porównania

| Rola | Słowa |
|---|---|
| `>` | `większ-`, `więtsz-`, `więcej`, `przewyższa`, `przewyższają` |
| `<` | `mniejsz-`, `mniej` |
| `!=` | `różn-` |
| „niż” | `niż`, `niźli`, `aniżeli`, `nad` |
| `>=`, `<=` | `co najmniej`, `co najwyżej` |
| podzielność | `podzieln-`, `dzieli` |
| parzystość | `parzyst-`, `nieparzyst-` |
| `len == 0` | `pust-` |
| znak | `dodatn-`, `ujemn-` |
| in | `zawiera`, `zawierają`, `mieści`, `ma`, `mają` |

### Wartości

| Rola | Słowa |
|---|---|
| True | `prawda`, `prawdą`, `prawdę`, `prawdy`, `zaiste` |
| False | `fałsz`, `fałszem`, `fałszu`, `bynajmniej`, `nieprawda`, `łgarstwo` |
| None | `nic`, `niczego`, `niczym`, `nicość`, `nicości`, `nicością` |
| 0,5 i 1,5 | `pół`, `półtora` |

### Regestry, spisy, napisy

| Rola | Słowa |
|---|---|
| element | `element`, `elementu`, `elementem`, `elemencie`, `elementy`, `elementów`, `wyraz`, `wyrazu`, `wyrazem` |
| indeksy 1–10 | `pierwsz-`, `drug-`/`wtór-`, `trzec-`, `czwart-`, `piąt-`, `szóst-`, `siódm-`, `ósm-`, `dziewiąt-`, `dziesiąt-` |
| `[-1]` | `ostatn-` |
| len | `długość`, `długości`, `długością`, `rozmiar`, `rozmiaru`, `rozmiarem` |
| list | `regestr`, `rejestr` (w każdym przypadku) |
| `[x] * n` | `pełen`, `pełn-`, `wypełnion-` |
| reverse | `wspak` |
| dict | `spis`, `słownik` (w każdym przypadku) |

### Łączenie zdań

| Rola | Słowa |
|---|---|
| „a potem” | `potem`, `następnie`, `wreszcie`, `później`, `po czym` |
| koniec programu | `amen` |

### Liczebniki

Liczby można pisać słownie, w dowolnym przypadku, i składać je
(`dwieście trzydzieści pięć`, `dwa tysiące dwadzieścia sześć`):

| | Formy |
|---|---|
| 0–9 | `zero`, `jeden`/`jedna`/`jedno`…, `dwa`/`dwie`/`dwóch`…, `trzy`, `cztery`, `pięć`, `sześć`, `siedem`, `osiem`, `dziewięć` |
| 10–19 | `dziesięć`, `jedenaście`, `dwanaście`, `trzynaście`, `czternaście`, `piętnaście`, `szesnaście`, `siedemnaście`, `osiemnaście`, `dziewiętnaście` |
| dziesiątki | `dwadzieścia`, `trzydzieści`, `czterdzieści`, `pięćdziesiąt`, …, `dziewięćdziesiąt` |
| setki | `sto`, `dwieście`, `trzysta`, `czterysta`, `pięćset`, …, `dziewięćset` |
| więcej | `tysiąc`/`tysiące`/`tysięcy`, `milion`/`miliony`/`milionów` |
| rzymskie | `MDCX`, `XIV` (tylko wielkimi literami) |

Wszystkie te słowa są zajęte i nie mogą być mianami.

---

## Słowa, które skryba pomija (wypełniacze)

Można ich używać do woli, dla kolorytu. Skryba ich nie widzi, więc też **nie mogą być mianami**.

| Słowo | Znaczy mniej więcej |
|---|---|
| `tedy`, `wtedy`, `natenczas`, `wówczas` | then, at that time |
| `to`, `zatem`, `przeto`, `więc` | so, therefore |
| `oto`, `otóż` | here, behold |
| `wszak`, `wszakże` | after all |
| `albowiem`, `bowiem` | for, because |
| `nuże` | come on! |
| `ninie`, `teraz` | now |
| `się` | (zaimek zwrotny) |
| `waść`, `waćpan`, `waszmość`, `waszeć`, `mości`, `panie` | Your Grace, sir |
| `łaskawie`, `proszę`, `uprzejmie` | kindly, please |
| `zaś`, `lecz` | while, but |
| `onego`, `owego` | that (one) |
| `też`, `także`, `jeszcze`, `toż` | also, still |
| `ano`, `ach`, `och`, `hej`, `zaprawdę`, `jakoż` | wykrzykniki |
| `co następuje`, `jako następuje` | as follows (całe wyrażenie znika) |

`Jeśli liczba jest parzysta, to obwieść liczbę tedy.` znaczy to samo co
`Jeśli liczba jest parzysta, obwieść ją.`

Poza tym skryba pomija:
- narrację między czasownikiem mowy a dwukropkiem: `Rzekł tedy stary rotmistrz, gładząc wąsa: „…”`,
- narrację po biedzie: `A w razie biedy wielkiej, zwanej nieszczęściem, co spadła na karczmę: …`,
- myślniki `—`, `–` i wielokropki `…` między słowami.

---

## Przypisy (komentarze)

Wiersz zaczynający się od jednego z tych słów jest przypisem:

> `Glossa`, `Glosa`, `Przypis`, `Rozdział`, `Księga`, `Pieśń`, `Anno`, `Przedmowa`, `Posłowie`, `Tytuł`

Przypis bez kropki na końcu obejmuje też kolejne wiersze, które zaczynają się małą literą.
Pusty wiersz zawsze go kończy.

```
Rozdział pierwszy, w którym liczymy.
Przypis: to zwykły komentarz,
który ciągnie się przez dwa wiersze
```

---

## Najczęstsze potknięcia

| Napisałeś | Popraw na | Dlaczego |
|---|---|---|
| `Dla każdego i od 1 do 5` | `Dla każdego numeru od 1 do 5` | `i` to spójnik |
| `x przez y` | `x podzielone przez y` | `przez` samo nie dzieli |
| `lista[0]` w myślach | `pierwszy element listy` | liczy się od jedynki |
| `Niechaj pusty będzie …` | inne miano | `pusty` to słowo skryby |
| `moja_lista` | `moja lista` | podkreślnik jest zakazany |
| brak `Amen.` | `Amen.` na końcu | obowiązkowe |
| `3. 14` | `3,14` | kropka ze spacją kończy zdanie |
| `liczba` i `liczby` jako dwie zmienne | `liczby` i `kandydat` | ten sam rdzeń, czyli jedna rzecz |
| `Niechaj y będzie …`, `Niechaj w …`, `Niechaj ż …` | `Niechaj q będzie …` | `y` = `i`, `w` i `ż` = `z` to słowa skryby |
| `Niechaj spis będzie …` | `Niechaj ceny będą …` | `spis`, `regestr`, `plik` są zajęte |
| `v jest w liście` (dla zmiennej `lista`) | `lista zawiera v` | odmiana z inną głoską (`lista` → `liście`) gubi rdzeń |
| `obwieść ją` | `obwieść liczbę` | skryba nie zna zaimków |
| `obwieść co następuje: „…”` | `obwieść co następuje „…”` | dwukropek po wypełniaczu nie działa (działa tylko po narracji) |
