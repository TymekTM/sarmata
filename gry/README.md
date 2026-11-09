# Sejm 1652 — Symulator Liberum Veto

Pełnoprawna gra tekstowa w Sarmacie: siedem dni obrad, ósmego dnia głosowanie,
á liberum veto czai się w ławie poselskiey. Kronikarz opowiada wierszem,
skryba kompiluje, a Rzeczpospolita czeka na ustawę o podatku na obronę kresów.

Sam kod gra czyta się iak kronika wierszem: rozdziały rymowane parami,
uczynki anaforyczne („okazanie wierności", „okazanie łaski", „okazanie ławy",
„wieść nocna"), glossy na marginesach — a liczniki głosowania to para
„zgoda" i „swara", bo tak gra godzi zgodę ze swarą od poczęcia.

## Animacje

Gra otwiera się tytułowym ekranem: chorągwie banera zjawiaią się linia po
lini, wiersz zapraszaiaiący pisze się litera po literze iak maszyna do
pisania, po ENTERZE cztery paski ładowania (Zbieranie szlachty z karczem,
Podkuwanie husarskich koni, Grzanie węgrzyna na zapas, Odkurzanie liberum
veto) gotują obrady. W grze: noc siada gwiazdami, kropki napięcia przed
czytaniem głosów, cisza z muchą koło ucha przed «NIE POZWALAM!», dzwony
zwycięstwa y świeca dogasaiąca w epilogu. Między scenami gąbka wymazuje
tablicę: rozkaz „Wymaż ekran" czyści terminal przed tytułem, nocą, dniem
ósmym y finałem uchwały (w rurze á pliku gąbka rozsądnie milcze). Sarmata
nie ma zegara, więc czas mierzą same pętle: uczynek „zwłoka" kręci się
milionami obrotów, á każdy obrót to gruchot pustej sakwy.

## Jak grać

```sh
./target/release/sarmata uruchom gry/sejm.sar
```

## Zasady

* Jesteś mówcą ustawy. Masz sto dukatów, łaskę króla (z dziesięciu) i siedem dni.
* Każdy z ośmiu posłów ma **wierność** (0-10), **chciwość** (cenę przekupstwa)
  i **upór**. Wierność co najmniej sześć daje głos ZA, nie więcej niż trzy — PRZECIW;
  cztery i pięć to wahanie się i rzut kością.
* Upór co najmniej osiem czyni posła **wetującym**: jeśli w dniu głosowania
  będzie przeciw, wstanie z ławy z okrzykiem «NIE POZWOLĘ!». Weto można
  odkupić za podwójną chciwość — honor to inna księga.
* Działania: rozmowa (+1 wierności, Fredro +2, szansa obrazy −1),
  przekupstwo (+3, koszt = chciwość; Czarniecki łapówek nie bierze —
  skandal kosztuje łaskę u króla), uczta (20 dukatów, wszyscy +1),
  audiencja u króla (+1 łaski, raz dziennie), zapis obrady do skrzyni.
* Co noc wieść losowa: Tatarzy, złodziej w skarbcu, jarmark, anonimowy list
  albo traktat Fredry o wecie. Siciński co noc utwierdza się w uporze.
* Zapis gry mieści się w skrzyni Skarbu (pliki `sejm1652.kronika` obok
  binarki); przy starcie można wczytać przerwaną obradę.
* Wynik zależy od większości: uchwała, upadek wetem albo — najnudniejszy —
  upadek większością. Rachunek chwały liczony z dukatów, łaski i głosów
  nada ci tytuł: *Ociec Oyczyzny*, *Mąż Stanu*, *Poseł Iak Większość*.

## OBSŁUGA

Gracz wybiera działania liczbami 0-6, posłów numerami 1-8, pytania tak/nie.
Grafika: czysty terminal UTF-8; banery w `gry/wyglad/`, teksty wierszem.
