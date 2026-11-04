/*
 * Bieg Sarmaty — wklejany przez skrybę na początek każdego programu.
 *
 * Wartości są dynamiczne (W), pamięci się nie zwalnia,
 * albowiem szlachta ziemi raz zdobytej nie oddaje.
 */
#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>
#include <math.h>
#include <time.h>
#include <unistd.h>

typedef enum { T_NIC = 0, T_LICZBA, T_ULAMEK, T_SAD, T_PISMO, T_REGESTR, T_ROD } Typ;

typedef struct Pismo { size_t len; size_t zn; char d[]; } Pismo;
typedef struct Regestr Regestr;
typedef struct Rod Rod;
typedef struct W {
    Typ t;
    union { long long i; double f; int b; Pismo *s; Regestr *r; Rod *o; } u;
} W;
struct Regestr { size_t len, cap; W *el; };
struct Rod { int rod; W *pola; };

#define SW_NIC ((W){0})

/*@@TABLICE@@*/

/* ─────────── grzechy ─────────── */

static void sw_grzech(int wiersz, const char *fmt, ...) {
    fflush(stdout);
    int kolor = isatty(2);
    if (wiersz >= SW_WIERSZE_PISMA)
        fprintf(stderr, "%sGrzech śmiertelny w „%s”, wierſzu %d:%s ", kolor ? "\x1b[1;31m" : "",
                sw_pisma[wiersz / SW_WIERSZE_PISMA], wiersz % SW_WIERSZE_PISMA,
                kolor ? "\x1b[0m" : "");
    else
        fprintf(stderr, "%sGrzech śmiertelny w wierſzu %d:%s ", kolor ? "\x1b[1;31m" : "", wiersz,
                kolor ? "\x1b[0m" : "");
    va_list ap;
    va_start(ap, fmt);
    vfprintf(stderr, fmt, ap);
    va_end(ap);
    fputc('\n', stderr);
    exit(1);
}

static void *sw_alloc(size_t n) {
    void *p = malloc(n ? n : 1);
    if (!p) {
        fputs("Pamięci zábrákło — ſkárbiec Rzeczypoſpolitey pusty.\n", stderr);
        exit(1);
    }
    return p;
}

static void *sw_realloc(void *p, size_t n) {
    p = realloc(p, n ? n : 1);
    if (!p) {
        fputs("Pamięci zábrákło — ſkárbiec Rzeczypoſpolitey pusty.\n", stderr);
        exit(1);
    }
    return p;
}

/* ─────────── wartości proste ─────────── */

static W sw_liczba(long long i) { W w; w.t = T_LICZBA; w.u.i = i; return w; }
static W sw_ulamek(double f) { W w; w.t = T_ULAMEK; w.u.f = f; return w; }
static W sw_sad(int b) { W w; w.t = T_SAD; w.u.b = b != 0; return w; }

static size_t sw_utf8_zn(const char *s, size_t n) {
    size_t k = 0;
    for (size_t i = 0; i < n; i++)
        if (((unsigned char)s[i] & 0xC0) != 0x80) k++;
    return k;
}

static W sw_pismo_n(const char *s, size_t n) {
    Pismo *p = sw_alloc(sizeof(Pismo) + n + 1);
    p->len = n;
    memcpy(p->d, s, n);
    p->d[n] = 0;
    p->zn = sw_utf8_zn(s, n);
    W w;
    w.t = T_PISMO;
    w.u.s = p;
    return w;
}

static W sw_regestr_nowy(void) {
    Regestr *r = sw_alloc(sizeof *r);
    r->len = r->cap = 0;
    r->el = NULL;
    W w;
    w.t = T_REGESTR;
    w.u.r = r;
    return w;
}

static void sw_wloz(W l, W v) {
    Regestr *r = l.u.r;
    if (r->len == r->cap) {
        r->cap = r->cap ? r->cap * 2 : 8;
        r->el = sw_realloc(r->el, r->cap * sizeof(W));
    }
    r->el[r->len++] = v;
}

static W sw_regestr_z(int n, W *el) {
    W w = sw_regestr_nowy();
    for (int i = 0; i < n; i++) sw_wloz(w, el[i]);
    return w;
}

static int sw_jest_liczba(W a) { return a.t == T_LICZBA || a.t == T_ULAMEK; }
static double sw_d(W a) { return a.t == T_LICZBA ? (double)a.u.i : a.u.f; }

static const char *sw_typ(W a) {
    switch (a.t) {
    case T_NIC: return "nicość";
    case T_LICZBA: return "liczbá";
    case T_ULAMEK: return "vłámek";
    case T_SAD: return "prawdá álbo fałſz";
    case T_PISMO: return "piſmo";
    case T_REGESTR: return "regeſtr";
    case T_ROD: return sw_rod_nazwy[a.u.o->rod];
    }
    return "?";
}

/* ─────────── bufor i obwieszczanie ─────────── */

typedef struct { char *d; size_t len, cap; } Buf;

static void buf_put(Buf *b, const char *s, size_t n) {
    if (b->len + n + 1 > b->cap) {
        size_t c = b->cap ? b->cap * 2 : 64;
        while (c < b->len + n + 1) c *= 2;
        b->d = sw_realloc(b->d, c);
        b->cap = c;
    }
    memcpy(b->d + b->len, s, n);
    b->len += n;
    b->d[b->len] = 0;
}

static void buf_str(Buf *b, const char *s) { buf_put(b, s, strlen(s)); }

static void sw_fmt_ulamek(Buf *b, double f) {
    if (isnan(f)) { buf_str(b, "niewiadoma"); return; }
    if (isinf(f)) { buf_str(b, f > 0 ? "nieſkończoność" : "minus nieſkończoność"); return; }
    char t[64];
    snprintf(t, sizeof t, "%.10g", f);
    int kropka = 0;
    for (char *c = t; *c; c++) {
        if (*c == '.') { *c = ','; kropka = 1; }
        if (*c == 'e' || *c == 'n' || *c == 'i') kropka = 1;
    }
    buf_str(b, t);
    if (!kropka) buf_str(b, ",0");
}

static int sw_slot(int rod, int pole);

static void sw_fmt(Buf *b, W a, int zagniezdzone, int glebia) {
    if (glebia > 40) { buf_str(b, "…"); return; }
    switch (a.t) {
    case T_NIC: buf_str(b, "nic"); break;
    case T_LICZBA: {
        char t[32];
        snprintf(t, sizeof t, "%lld", a.u.i);
        buf_str(b, t);
        break;
    }
    case T_ULAMEK: sw_fmt_ulamek(b, a.u.f); break;
    case T_SAD: buf_str(b, a.u.b ? "prawda" : "fałsz"); break;
    case T_PISMO:
        if (zagniezdzone) buf_str(b, "„");
        buf_put(b, a.u.s->d, a.u.s->len);
        if (zagniezdzone) buf_str(b, "”");
        break;
    case T_REGESTR:
        buf_str(b, "[");
        for (size_t i = 0; i < a.u.r->len; i++) {
            if (i) buf_str(b, "; ");
            sw_fmt(b, a.u.r->el[i], 1, glebia + 1);
        }
        buf_str(b, "]");
        break;
    case T_ROD: {
        Rod *o = a.u.o;
        buf_str(b, sw_rod_nazwy[o->rod]);
        buf_str(b, " { ");
        for (int k = 0; k < sw_rod_npol[o->rod]; k++) {
            if (k) buf_str(b, "; ");
            buf_str(b, sw_pole_nazwy[sw_rod_pola[o->rod][k]]);
            buf_str(b, ": ");
            sw_fmt(b, o->pola[k], 1, glebia + 1);
        }
        buf_str(b, " }");
        break;
    }
    }
}

static W sw_napis(W a) {
    if (a.t == T_PISMO) return a;
    Buf b = {0};
    sw_fmt(&b, a, 0, 0);
    W w = sw_pismo_n(b.d ? b.d : "", b.len);
    free(b.d);
    return w;
}

static W sw_zlacz(int n, W *el) {
    Buf b = {0};
    for (int i = 0; i < n; i++) sw_fmt(&b, el[i], 0, 0);
    W w = sw_pismo_n(b.d ? b.d : "", b.len);
    free(b.d);
    return w;
}

static void sw_obwiesc(int n, W *a, int nowy_wiersz) {
    Buf b = {0};
    for (int i = 0; i < n; i++) {
        if (i) buf_str(&b, " ");
        sw_fmt(&b, a[i], 0, 0);
    }
    if (nowy_wiersz) buf_str(&b, "\n");
    if (b.len) fwrite(b.d, 1, b.len, stdout);
    free(b.d);
}

/* ─────────── prawdziwość i liczby całe ─────────── */

static int sw_prawdziwe(W a, int L) {
    (void)L;
    switch (a.t) {
    case T_NIC: return 0;
    case T_LICZBA: return a.u.i != 0;
    case T_ULAMEK: return a.u.f != 0;
    case T_SAD: return a.u.b;
    case T_PISMO: return a.u.s->len > 0;
    case T_REGESTR: return a.u.r->len > 0;
    case T_ROD: return 1;
    }
    return 0;
}

static long long sw_calk(W a, int L) {
    if (a.t == T_LICZBA) return a.u.i;
    if (a.t == T_ULAMEK && a.u.f == floor(a.u.f) && fabs(a.u.f) < 9e18) return (long long)a.u.f;
    sw_grzech(L, "Spodziewáno ſię liczby cáłey, á dano: %s.", sw_typ(a));
    return 0;
}

/* ─────────── rachunki ─────────── */

static W sw_dodac(W a, W b, int L) {
    if (a.t == T_LICZBA && b.t == T_LICZBA) return sw_liczba(a.u.i + b.u.i);
    if (sw_jest_liczba(a) && sw_jest_liczba(b)) return sw_ulamek(sw_d(a) + sw_d(b));
    if (a.t == T_PISMO || b.t == T_PISMO) { W p[2] = {a, b}; return sw_zlacz(2, p); }
    if (a.t == T_REGESTR && b.t == T_REGESTR) {
        W w = sw_regestr_nowy();
        for (size_t i = 0; i < a.u.r->len; i++) sw_wloz(w, a.u.r->el[i]);
        for (size_t i = 0; i < b.u.r->len; i++) sw_wloz(w, b.u.r->el[i]);
        return w;
    }
    sw_grzech(L, "Nie godzi ſię dodáwáć rzeczy tákich: %s i %s.", sw_typ(a), sw_typ(b));
    return SW_NIC;
}

static W sw_odjac(W a, W b, int L) {
    if (a.t == T_LICZBA && b.t == T_LICZBA) return sw_liczba(a.u.i - b.u.i);
    if (sw_jest_liczba(a) && sw_jest_liczba(b)) return sw_ulamek(sw_d(a) - sw_d(b));
    sw_grzech(L, "Odeymowáć możná ieno liczby, á tu ſą: %s i %s.", sw_typ(a), sw_typ(b));
    return SW_NIC;
}

static W sw_powiel(W s, long long n, int L) {
    if (n < 0) sw_grzech(L, "Piſmá vieymną liczbę rázy powielić nie ſpoſób.");
    Buf b = {0};
    for (long long i = 0; i < n; i++) buf_put(&b, s.u.s->d, s.u.s->len);
    W w = sw_pismo_n(b.d ? b.d : "", b.len);
    free(b.d);
    return w;
}

static W sw_razy(W a, W b, int L) {
    if (a.t == T_LICZBA && b.t == T_LICZBA) return sw_liczba(a.u.i * b.u.i);
    if (sw_jest_liczba(a) && sw_jest_liczba(b)) return sw_ulamek(sw_d(a) * sw_d(b));
    if (a.t == T_PISMO && b.t == T_LICZBA) return sw_powiel(a, b.u.i, L);
    if (a.t == T_LICZBA && b.t == T_PISMO) return sw_powiel(b, a.u.i, L);
    sw_grzech(L, "Mnożyć możná ieno liczby, á tu ſą: %s i %s.", sw_typ(a), sw_typ(b));
    return SW_NIC;
}

static W sw_przez(W a, W b, int L) {
    if (!sw_jest_liczba(a) || !sw_jest_liczba(b))
        sw_grzech(L, "Dźielić możná ieno liczby, á tu ſą: %s i %s.", sw_typ(a), sw_typ(b));
    if (sw_d(b) == 0) sw_grzech(L, "Dźielić przez zero ſię ważyſz? Rzecz bezbożna.");
    if (a.t == T_LICZBA && b.t == T_LICZBA && a.u.i % b.u.i == 0) return sw_liczba(a.u.i / b.u.i);
    return sw_ulamek(sw_d(a) / sw_d(b));
}

static W sw_modulo(W a, W b, int L) {
    if (a.t == T_LICZBA && b.t == T_LICZBA) {
        if (b.u.i == 0) sw_grzech(L, "Reſzty z dźielenia przez zero nie dobędźieſz.");
        long long r = a.u.i % b.u.i;
        if (r != 0 && ((r < 0) != (b.u.i < 0))) r += b.u.i;
        return sw_liczba(r);
    }
    if (sw_jest_liczba(a) && sw_jest_liczba(b)) {
        if (sw_d(b) == 0) sw_grzech(L, "Reſzty z dźielenia przez zero nie dobędźieſz.");
        return sw_ulamek(fmod(sw_d(a), sw_d(b)));
    }
    sw_grzech(L, "Reſztę z dźielenia brác możná ieno z liczb, á tu ſą: %s i %s.", sw_typ(a), sw_typ(b));
    return SW_NIC;
}

static W sw_potega(W a, W b, int L) {
    if (a.t == T_LICZBA && b.t == T_LICZBA && b.u.i >= 0) {
        long long r = 1, x = a.u.i, e = b.u.i;
        while (e > 0) {
            if (e & 1) r *= x;
            x *= x;
            e >>= 1;
        }
        return sw_liczba(r);
    }
    if (sw_jest_liczba(a) && sw_jest_liczba(b)) return sw_ulamek(pow(sw_d(a), sw_d(b)));
    sw_grzech(L, "Do potęgi podnośić możná ieno liczby, á tu ſą: %s i %s.", sw_typ(a), sw_typ(b));
    return SW_NIC;
}

static W sw_przeciwne(W a, int L) {
    if (a.t == T_LICZBA) return sw_liczba(-a.u.i);
    if (a.t == T_ULAMEK) return sw_ulamek(-a.u.f);
    sw_grzech(L, "Przećiwieńſtwá ma ieno liczbá, nie %s.", sw_typ(a));
    return SW_NIC;
}

/* ─────────── porównania ─────────── */

static int sw_rowne_(W a, W b) {
    if (sw_jest_liczba(a) && sw_jest_liczba(b)) {
        if (a.t == T_LICZBA && b.t == T_LICZBA) return a.u.i == b.u.i;
        return sw_d(a) == sw_d(b);
    }
    if (a.t != b.t) return 0;
    switch (a.t) {
    case T_NIC: return 1;
    case T_SAD: return a.u.b == b.u.b;
    case T_PISMO: return a.u.s->len == b.u.s->len && memcmp(a.u.s->d, b.u.s->d, a.u.s->len) == 0;
    case T_REGESTR:
        if (a.u.r == b.u.r) return 1;
        if (a.u.r->len != b.u.r->len) return 0;
        for (size_t i = 0; i < a.u.r->len; i++)
            if (!sw_rowne_(a.u.r->el[i], b.u.r->el[i])) return 0;
        return 1;
    case T_ROD: return a.u.o == b.u.o;
    default: return 0;
    }
}

static int sw_porownaj(W a, W b, int L) {
    if (sw_jest_liczba(a) && sw_jest_liczba(b)) {
        if (a.t == T_LICZBA && b.t == T_LICZBA) return (a.u.i > b.u.i) - (a.u.i < b.u.i);
        double x = sw_d(a), y = sw_d(b);
        return (x > y) - (x < y);
    }
    if (a.t == T_PISMO && b.t == T_PISMO) {
        size_t n = a.u.s->len < b.u.s->len ? a.u.s->len : b.u.s->len;
        int c = memcmp(a.u.s->d, b.u.s->d, n);
        if (c) return c < 0 ? -1 : 1;
        return (a.u.s->len > b.u.s->len) - (a.u.s->len < b.u.s->len);
    }
    sw_grzech(L, "Nie dá ſię porównáć rzeczy tákich: %s i %s.", sw_typ(a), sw_typ(b));
    return 0;
}

static W sw_rowne(W a, W b, int L) { (void)L; return sw_sad(sw_rowne_(a, b)); }
static W sw_rozne(W a, W b, int L) { (void)L; return sw_sad(!sw_rowne_(a, b)); }
static W sw_mniejsze(W a, W b, int L) { return sw_sad(sw_porownaj(a, b, L) < 0); }
static W sw_wieksze(W a, W b, int L) { return sw_sad(sw_porownaj(a, b, L) > 0); }
static W sw_niewieksze(W a, W b, int L) { return sw_sad(sw_porownaj(a, b, L) <= 0); }
static W sw_niemniejsze(W a, W b, int L) { return sw_sad(sw_porownaj(a, b, L) >= 0); }

static W sw_podzielne(W a, W b, int L) {
    long long x = sw_calk(a, L), y = sw_calk(b, L);
    if (y == 0) sw_grzech(L, "Przez zero nic ſię nie dźieli — rzecz bezbożna.");
    return sw_sad(x % y == 0);
}

static W sw_parzyste(W a, int L) { return sw_sad(sw_calk(a, L) % 2 == 0); }
static W sw_nieparzyste(W a, int L) { return sw_sad(sw_calk(a, L) % 2 != 0); }

static W sw_dodatnie(W a, int L) {
    if (!sw_jest_liczba(a)) sw_grzech(L, "Dodátnia bywa ieno liczbá, nie %s.", sw_typ(a));
    return sw_sad(sw_d(a) > 0);
}

static W sw_ujemne(W a, int L) {
    if (!sw_jest_liczba(a)) sw_grzech(L, "Vieymna bywa ieno liczbá, nie %s.", sw_typ(a));
    return sw_sad(sw_d(a) < 0);
}

static W sw_puste(W a, int L) {
    switch (a.t) {
    case T_NIC: return sw_sad(1);
    case T_PISMO: return sw_sad(a.u.s->len == 0);
    case T_REGESTR: return sw_sad(a.u.r->len == 0);
    default: sw_grzech(L, "Puſtá bywa ieno rzecz, co coś mieśći, nie %s.", sw_typ(a));
    }
    return SW_NIC;
}

/* ─────────── piſmá ─────────── */

static size_t sw_off(Pismo *p, size_t k) {
    size_t i = 0, z = 0;
    while (i < p->len) {
        if (z == k) return i;
        i++;
        while (i < p->len && ((unsigned char)p->d[i] & 0xC0) == 0x80) i++;
        z++;
    }
    return p->len;
}

static W sw_znak(Pismo *p, size_t k) {
    size_t a = sw_off(p, k), b = sw_off(p, k + 1);
    return sw_pismo_n(p->d + a, b - a);
}

static long long sw_indeks(W i, long long dl, int L) {
    long long k = sw_calk(i, L);
    if (k == 0)
        sw_grzech(L, "Elementu zerowego nie mász — liczymy od iednego, iáko Pan Bóg przykazał.");
    if (k < 1 || k > dl) {
        if (dl == 0) sw_grzech(L, "Śięgaſz po element %lld, á tu pusto — ani iednego nie mász.", k);
        sw_grzech(L, "Śięgaſz po element %lld, á ieſt ich ledwie %lld.", k, dl);
    }
    return k - 1;
}

/* ─────────── regestry ─────────── */

static W sw_element(W c, W i, int L) {
    if (c.t == T_REGESTR) return c.u.r->el[sw_indeks(i, (long long)c.u.r->len, L)];
    if (c.t == T_PISMO) return sw_znak(c.u.s, (size_t)sw_indeks(i, (long long)c.u.s->zn, L));
    sw_grzech(L, "Z rzeczy tákiey (%s) elementu nie wyiąć.", sw_typ(c));
    return SW_NIC;
}

static void sw_ustaw(W c, W i, W v, int L) {
    if (c.t == T_PISMO) sw_grzech(L, "Piſmá zmieniáć nie lza — vtwórz nowe.");
    if (c.t != T_REGESTR) sw_grzech(L, "Z rzeczy tákiey (%s) elementu nie zmienić.", sw_typ(c));
    c.u.r->el[sw_indeks(i, (long long)c.u.r->len, L)] = v;
}

static W sw_ostatni(W c, int L) {
    if (c.t == T_REGESTR) {
        if (!c.u.r->len) sw_grzech(L, "Regeſtr puſty — oſtátniego elementu w nim nie ma.");
        return c.u.r->el[c.u.r->len - 1];
    }
    if (c.t == T_PISMO) {
        if (!c.u.s->zn) sw_grzech(L, "Piſmo puſte — oſtátniey litery w nim nie ma.");
        return sw_znak(c.u.s, c.u.s->zn - 1);
    }
    sw_grzech(L, "Z rzeczy tákiey (%s) elementu nie wyiąć.", sw_typ(c));
    return SW_NIC;
}

static void sw_ustaw_ostatni(W c, W v, int L) {
    if (c.t != T_REGESTR) sw_grzech(L, "Zmieniáć możná ieno elementy regeſtru.");
    if (!c.u.r->len) sw_grzech(L, "Regeſtr puſty — oſtátniego elementu w nim nie ma.");
    c.u.r->el[c.u.r->len - 1] = v;
}

static W sw_dlugosc(W c, int L) {
    if (c.t == T_REGESTR) return sw_liczba((long long)c.u.r->len);
    if (c.t == T_PISMO) return sw_liczba((long long)c.u.s->zn);
    sw_grzech(L, "Długośći nie ma rzecz táka: %s.", sw_typ(c));
    return SW_NIC;
}

static long long sw_iter_dl(W c, int L) {
    if (c.t == T_REGESTR) return (long long)c.u.r->len;
    if (c.t == T_PISMO) return (long long)c.u.s->zn;
    sw_grzech(L, "Po rzeczy tákiey (%s) chodźić element po elemenćie nie ſpoſób.", sw_typ(c));
    return 0;
}

static W sw_iter_el(W c, long long k, int L) {
    (void)L;
    if (c.t == T_REGESTR) return c.u.r->el[k];
    return sw_znak(c.u.s, (size_t)k);
}

static W sw_wspak(W a, int L) {
    if (a.t == T_REGESTR) {
        W w = sw_regestr_nowy();
        for (size_t i = a.u.r->len; i > 0; i--) sw_wloz(w, a.u.r->el[i - 1]);
        return w;
    }
    if (a.t == T_PISMO) {
        Pismo *p = a.u.s;
        char *t = sw_alloc(p->len + 1);
        size_t o = p->len, i = 0;
        while (i < p->len) {
            size_t j = i + 1;
            while (j < p->len && ((unsigned char)p->d[j] & 0xC0) == 0x80) j++;
            o -= j - i;
            memcpy(t + o, p->d + i, j - i);
            i = j;
        }
        W w = sw_pismo_n(t, p->len);
        free(t);
        return w;
    }
    sw_grzech(L, "Wſpak czytáć możná piſmo álbo regeſtr, nie %s.", sw_typ(a));
    return SW_NIC;
}

static W sw_zawiera(W c, W x, int L) {
    if (c.t == T_REGESTR) {
        for (size_t i = 0; i < c.u.r->len; i++)
            if (sw_rowne_(c.u.r->el[i], x)) return sw_sad(1);
        return sw_sad(0);
    }
    if (c.t == T_PISMO) {
        W s = sw_napis(x);
        return sw_sad(memmem(c.u.s->d, c.u.s->len, s.u.s->d, s.u.s->len) != NULL);
    }
    sw_grzech(L, "Rzecz táka (%s) niczego w ſobie nie mieśći.", sw_typ(c));
    return SW_NIC;
}

static W sw_dodaj(W cel, W v, int L) {
    if (cel.t == T_REGESTR) {
        sw_wloz(cel, v);
        return cel;
    }
    return sw_dodac(cel, v, L);
}

static void sw_usun(W c, W i, int L) {
    if (c.t != T_REGESTR) sw_grzech(L, "Uſuwáć możná ieno z regeſtru, nie z: %s.", sw_typ(c));
    Regestr *r = c.u.r;
    if (i.t == T_NIC) {
        if (!r->len) sw_grzech(L, "Regeſtr puſty — nie máſz czego uſuwáć.");
        r->len--;
        return;
    }
    long long k = sw_indeks(i, (long long)r->len, L);
    memmove(r->el + k, r->el + k + 1, (r->len - (size_t)k - 1) * sizeof(W));
    r->len--;
}

/* ─────────── rody ─────────── */

static int sw_slot(int rod, int pole) {
    for (int k = 0; k < sw_rod_npol[rod]; k++)
        if (sw_rod_pola[rod][k] == pole) return k;
    return -1;
}

static W sw_rod_nowy(int rod, int n, const int *pola, const W *wart, int L) {
    Rod *o = sw_alloc(sizeof *o);
    o->rod = rod;
    int np = sw_rod_npol[rod];
    o->pola = sw_alloc((size_t)(np ? np : 1) * sizeof(W));
    for (int k = 0; k < np; k++) o->pola[k] = SW_NIC;
    for (int i = 0; i < n; i++) {
        int s = sw_slot(rod, pola[i]);
        if (s < 0) sw_grzech(L, "Ród %s nie ma polá „%s”.", sw_rod_nazwy[rod], sw_pole_nazwy[pola[i]]);
        o->pola[s] = wart[i];
    }
    W w;
    w.t = T_ROD;
    w.u.o = o;
    return w;
}

static int sw_slot_w(W o, int pole, int L) {
    if (o.t != T_ROD)
        sw_grzech(L, "Pole „%s” máią ieno rody, á tu ieſt: %s.", sw_pole_nazwy[pole], sw_typ(o));
    int s = sw_slot(o.u.o->rod, pole);
    if (s < 0) sw_grzech(L, "Ród %s nie ma polá „%s”.", sw_rod_nazwy[o.u.o->rod], sw_pole_nazwy[pole]);
    return s;
}

static W sw_pole(W o, int pole, int L) { return o.u.o->pola[sw_slot_w(o, pole, L)]; }

static void sw_ustaw_pole(W o, int pole, W v, int L) { o.u.o->pola[sw_slot_w(o, pole, L)] = v; }

/* ─────────── sortowanie (stateczne, przez scalanie) ─────────── */

static int sw_sort_pole, sw_sort_L, sw_sort_mal;

static int sw_sort_cmp(W a, W b) {
    if (sw_sort_pole >= 0) {
        a = sw_pole(a, sw_sort_pole, sw_sort_L);
        b = sw_pole(b, sw_sort_pole, sw_sort_L);
    }
    int r = sw_porownaj(a, b, sw_sort_L);
    return sw_sort_mal ? -r : r;
}

static void sw_scal(W *a, W *tmp, size_t n) {
    if (n < 2) return;
    size_t m = n / 2;
    sw_scal(a, tmp, m);
    sw_scal(a + m, tmp, n - m);
    size_t i = 0, j = m, k = 0;
    while (i < m && j < n) tmp[k++] = sw_sort_cmp(a[j], a[i]) < 0 ? a[j++] : a[i++];
    while (i < m) tmp[k++] = a[i++];
    while (j < n) tmp[k++] = a[j++];
    memcpy(a, tmp, n * sizeof(W));
}

static void sw_sortuj(W c, int pole, int malejaco, int L) {
    if (c.t != T_REGESTR) sw_grzech(L, "Sortowáć możná ieno regeſtr, nie: %s.", sw_typ(c));
    sw_sort_pole = pole;
    sw_sort_L = L;
    sw_sort_mal = malejaco;
    W *tmp = sw_alloc(c.u.r->len * sizeof(W));
    sw_scal(c.u.r->el, tmp, c.u.r->len);
    free(tmp);
}

/* ─────────── wejście ─────────── */

/* Słowo z wejścia: liczba, jeśli wygląda na liczbę, inaczej pismo. */
static W sw_auto(const char *s, size_t n) {
    size_t a = 0, b = n;
    while (a < b && (s[a] == ' ' || s[a] == '\t')) a++;
    while (b > a && (s[b - 1] == ' ' || s[b - 1] == '\t')) b--;
    if (a < b && b - a < 60) {
        char t[64];
        memcpy(t, s + a, b - a);
        t[b - a] = 0;
        int cyfra = 0;
        for (char *c = t; *c; c++) {
            if (*c == ',') *c = '.';
            if (*c >= '0' && *c <= '9') cyfra = 1;
        }
        if (cyfra) {
            char *end;
            long long i = strtoll(t, &end, 10);
            if (*end == 0) return sw_liczba(i);
            double f = strtod(t, &end);
            if (*end == 0) return sw_ulamek(f);
        }
    }
    return sw_pismo_n(s, n);
}

static W sw_zapytaj(W pytanie) {
    if (pytanie.t != T_NIC) {
        W p = sw_napis(pytanie);
        fwrite(p.u.s->d, 1, p.u.s->len, stdout);
    }
    fflush(stdout);
    char *l = NULL;
    size_t cap = 0;
    ssize_t r = getline(&l, &cap, stdin);
    if (r < 0) {
        free(l);
        return SW_NIC;
    }
    while (r > 0 && (l[r - 1] == '\n' || l[r - 1] == '\r')) r--;
    W w = sw_auto(l, (size_t)r);
    free(l);
    return w;
}

static W sw_wczytaj(void) {
    fflush(stdout);
    int c;
    do c = getchar(); while (c == ' ' || c == '\t' || c == '\n' || c == '\r');
    if (c == EOF) return SW_NIC;
    Buf b = {0};
    while (c != EOF && c != ' ' && c != '\t' && c != '\n' && c != '\r') {
        char ch = (char)c;
        buf_put(&b, &ch, 1);
        c = getchar();
    }
    W w = sw_auto(b.d, b.len);
    free(b.d);
    return w;
}

/* ─────────── pliki ─────────── */

static W sw_b_tresc(W p, int L) {
    W path = sw_napis(p);
    FILE *f = fopen(path.u.s->d, "rb");
    if (!f) sw_grzech(L, "Pliku „%s” nie názleźiono.", path.u.s->d);
    Buf b = {0};
    char t[4096];
    size_t n;
    while ((n = fread(t, 1, sizeof t, f)) > 0) buf_put(&b, t, n);
    fclose(f);
    W w = sw_pismo_n(b.d ? b.d : "", b.len);
    free(b.d);
    return w;
}

static W sw_b_wiersze(W p, int L) {
    W t = sw_b_tresc(p, L);
    W w = sw_regestr_nowy();
    const char *s = t.u.s->d;
    size_t n = t.u.s->len, a = 0;
    for (size_t i = 0; i <= n; i++) {
        if (i == n || s[i] == '\n') {
            if (i == n && a == n) break;
            size_t e = i;
            if (e > a && s[e - 1] == '\r') e--;
            sw_wloz(w, sw_pismo_n(s + a, e - a));
            a = i + 1;
        }
    }
    return w;
}

static void sw_zapisz(W tresc, W p, int dopisz, int L) {
    W path = sw_napis(p);
    FILE *f = fopen(path.u.s->d, dopisz ? "a" : "w");
    if (!f) sw_grzech(L, "Do pliku „%s” piſáć nie podobná.", path.u.s->d);
    Buf b = {0};
    if (tresc.t == T_REGESTR) {
        for (size_t i = 0; i < tresc.u.r->len; i++) {
            sw_fmt(&b, tresc.u.r->el[i], 0, 0);
            buf_str(&b, "\n");
        }
    } else {
        sw_fmt(&b, tresc, 0, 0);
        buf_str(&b, "\n");
    }
    if (b.len) fwrite(b.d, 1, b.len, f);
    free(b.d);
    fclose(f);
}

/* ─────────── uczynki wbudowane ─────────── */

static W sw_b_pierwiastek(W x, int L) {
    if (!sw_jest_liczba(x)) sw_grzech(L, "Pierwiáſtek dobywá ſię ieno z liczby, nie z: %s.", sw_typ(x));
    if (sw_d(x) < 0) sw_grzech(L, "Pierwiáſtká z liczby vieymney nie dobędźieſz.");
    double r = sqrt(sw_d(x));
    if (x.t == T_LICZBA && r == floor(r)) return sw_liczba((long long)r);
    return sw_ulamek(r);
}

static W sw_b_abs(W x, int L) {
    if (x.t == T_LICZBA) return sw_liczba(x.u.i < 0 ? -x.u.i : x.u.i);
    if (x.t == T_ULAMEK) return sw_ulamek(fabs(x.u.f));
    sw_grzech(L, "Wártość bezwzględną ma ieno liczbá, nie %s.", sw_typ(x));
    return SW_NIC;
}

static W sw_b_zaokraglenie(W x, int L) {
    if (x.t == T_LICZBA) return x;
    if (x.t == T_ULAMEK) return sw_liczba(llround(x.u.f));
    sw_grzech(L, "Zaokrąglić możná ieno liczbę, nie %s.", sw_typ(x));
    return SW_NIC;
}

static W sw_b_calkowita(W x, int L) {
    if (x.t == T_LICZBA) return x;
    if (x.t == T_ULAMEK) return sw_liczba((long long)trunc(x.u.f));
    sw_grzech(L, "Część cáłkowitą ma ieno liczbá, nie %s.", sw_typ(x));
    return SW_NIC;
}

static W sw_b_liczbowa(W x, int L) {
    if (sw_jest_liczba(x)) return x;
    if (x.t == T_PISMO) {
        W w = sw_auto(x.u.s->d, x.u.s->len);
        if (sw_jest_liczba(w)) return w;
        sw_grzech(L, "„%s” liczbą nie ieſt, choćby ſię zárzekáło.", x.u.s->d);
    }
    sw_grzech(L, "Z rzeczy tákiey (%s) liczby nie wydobędźieſz.", sw_typ(x));
    return SW_NIC;
}

static W sw_b_losowa(W a, W b, int L) {
    long long x = sw_calk(a, L), y = sw_calk(b, L);
    if (x > y) { long long t = x; x = y; y = t; }
    unsigned long long span = (unsigned long long)(y - x) + 1;
    unsigned long long r = ((unsigned long long)random() << 31) ^ (unsigned long long)random();
    return sw_liczba(x + (long long)(r % span));
}

static int sw_dekoduj(const unsigned char *s, size_t n, size_t *dl) {
    if (s[0] < 0x80 || n < 2) { *dl = 1; return s[0]; }
    if ((s[0] & 0xE0) == 0xC0) { *dl = 2; return ((s[0] & 0x1F) << 6) | (s[1] & 0x3F); }
    if ((s[0] & 0xF0) == 0xE0 && n >= 3) {
        *dl = 3;
        return ((s[0] & 0x0F) << 12) | ((s[1] & 0x3F) << 6) | (s[2] & 0x3F);
    }
    *dl = 1;
    return s[0];
}

static void sw_koduj(Buf *b, int cp) {
    char t[4];
    if (cp < 0x80) { t[0] = (char)cp; buf_put(b, t, 1); }
    else if (cp < 0x800) { t[0] = (char)(0xC0 | (cp >> 6)); t[1] = (char)(0x80 | (cp & 0x3F)); buf_put(b, t, 2); }
    else {
        t[0] = (char)(0xE0 | (cp >> 12));
        t[1] = (char)(0x80 | ((cp >> 6) & 0x3F));
        t[2] = (char)(0x80 | (cp & 0x3F));
        buf_put(b, t, 3);
    }
}

static const int sw_male_l[] = {0x105, 0x107, 0x119, 0x142, 0x144, 0xF3, 0x15B, 0x17A, 0x17C, 0xE1, 0xE9};
static const int sw_wielkie_l[] = {0x104, 0x106, 0x118, 0x141, 0x143, 0xD3, 0x15A, 0x179, 0x17B, 0xC1, 0xC9};

static W sw_litery(W x, int wielkie, int L) {
    if (x.t != T_PISMO) sw_grzech(L, "Litery zmieniáć możná ieno w piśmie, nie w: %s.", sw_typ(x));
    Buf b = {0};
    const unsigned char *s = (const unsigned char *)x.u.s->d;
    size_t n = x.u.s->len, i = 0;
    while (i < n) {
        size_t dl;
        int cp = sw_dekoduj(s + i, n - i, &dl);
        if (cp < 0x80) cp = wielkie ? (cp >= 'a' && cp <= 'z' ? cp - 32 : cp) : (cp >= 'A' && cp <= 'Z' ? cp + 32 : cp);
        else
            for (int k = 0; k < 11; k++) {
                if (wielkie && cp == sw_male_l[k]) { cp = sw_wielkie_l[k]; break; }
                if (!wielkie && cp == sw_wielkie_l[k]) { cp = sw_male_l[k]; break; }
            }
        sw_koduj(&b, cp);
        i += dl;
    }
    W w = sw_pismo_n(b.d ? b.d : "", b.len);
    free(b.d);
    return w;
}

static W sw_b_wielkie(W x, int L) { return sw_litery(x, 1, L); }
static W sw_b_male(W x, int L) { return sw_litery(x, 0, L); }

static W sw_b_rozbior(W x, int L) {
    W s = sw_napis(x);
    (void)L;
    W w = sw_regestr_nowy();
    const char *d = s.u.s->d;
    size_t n = s.u.s->len, i = 0;
    while (i < n) {
        while (i < n && (d[i] == ' ' || d[i] == '\t' || d[i] == '\n' || d[i] == '\r')) i++;
        size_t a = i;
        while (i < n && !(d[i] == ' ' || d[i] == '\t' || d[i] == '\n' || d[i] == '\r')) i++;
        if (i > a) sw_wloz(w, sw_auto(d + a, i - a));
    }
    return w;
}

/* Kawałek pisma bez spacyi po bokach; liczby rozpoznaie sam. */
static void sw_wloz_kawalek(W w, const char *d, size_t a, size_t b) {
    while (a < b && (d[a] == ' ' || d[a] == '\t' || d[a] == '\r' || d[a] == '\n')) a++;
    while (b > a && (d[b - 1] == ' ' || d[b - 1] == '\t' || d[b - 1] == '\r' || d[b - 1] == '\n')) b--;
    sw_wloz(w, sw_auto(d + a, b - a));
}

static W sw_b_rozciecie(W x, W sep, int L) {
    W s = sw_napis(x), p = sw_napis(sep);
    const char *d = s.u.s->d, *q = p.u.s->d;
    size_t n = s.u.s->len, m = p.u.s->len, a = 0, i = 0;
    if (!m) sw_grzech(L, "Rozćinać przez puſte piſmo nie ſpoſób.");
    W w = sw_regestr_nowy();
    while (i + m <= n) {
        if (!memcmp(d + i, q, m)) {
            sw_wloz_kawalek(w, d, a, i);
            i += m;
            a = i;
        } else {
            i++;
        }
    }
    sw_wloz_kawalek(w, d, a, n);
    return w;
}

static W sw_skrajny(W x, int max, int L) {
    if (x.t != T_REGESTR) sw_grzech(L, "Skráynego elementu ſzuká ſię w regeſtrze, nie w: %s.", sw_typ(x));
    if (!x.u.r->len) sw_grzech(L, "Regeſtr puſty — nie máſz z czego wybieráć.");
    W m = x.u.r->el[0];
    for (size_t i = 1; i < x.u.r->len; i++) {
        int c = sw_porownaj(x.u.r->el[i], m, L);
        if (max ? c > 0 : c < 0) m = x.u.r->el[i];
    }
    return m;
}

static W sw_b_max(W x, int L) { return sw_skrajny(x, 1, L); }
static W sw_b_min(W x, int L) { return sw_skrajny(x, 0, L); }

static W sw_b_kopia(W x, int L) {
    (void)L;
    if (x.t == T_REGESTR) return sw_regestr_z((int)x.u.r->len, x.u.r->el);
    if (x.t == T_ROD) {
        int np = sw_rod_npol[x.u.o->rod];
        Rod *o = sw_alloc(sizeof *o);
        o->rod = x.u.o->rod;
        o->pola = sw_alloc((size_t)(np ? np : 1) * sizeof(W));
        memcpy(o->pola, x.u.o->pola, (size_t)np * sizeof(W));
        W w;
        w.t = T_ROD;
        w.u.o = o;
        return w;
    }
    return x;
}

static W sw_b_zlacz(W x, W sep, int L) {
    if (x.t != T_REGESTR) sw_grzech(L, "Złączáć możná ieno regeſtr, nie: %s.", sw_typ(x));
    Buf b = {0};
    W s = sw_napis(sep);
    for (size_t i = 0; i < x.u.r->len; i++) {
        if (i) buf_put(&b, s.u.s->d, s.u.s->len);
        sw_fmt(&b, x.u.r->el[i], 0, 0);
    }
    W w = sw_pismo_n(b.d ? b.d : "", b.len);
    free(b.d);
    return w;
}

/* ─────────── początek i koniec ─────────── */

static void sw_start(int argc, char **argv) {
    (void)argc;
    (void)argv;
    srandom((unsigned)time(NULL) ^ (unsigned)getpid());
}

static void sw_koniec(void) { fflush(stdout); }
