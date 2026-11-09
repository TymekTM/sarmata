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
#include <setjmp.h>
#include <errno.h>
#include <limits.h>
#include <dirent.h>
#include <sys/stat.h>

/* ─────────── Windows (MinGW) — braki POSIX-u ─────────── */
#if defined(_WIN32)
#include <direct.h>
#include <io.h>
#include <fcntl.h>

__declspec(dllimport) int __stdcall SetConsoleOutputCP(unsigned int);
__declspec(dllimport) int __stdcall SetConsoleCP(unsigned int);
/* Konsola po ludzku: UTF-8 á końce wierſzów po iednym «\n», iáko na Linuxie. */
static void sw_konsola_utf8(void) {
    _setmode(_fileno(stdout), _O_BINARY);
    _setmode(_fileno(stderr), _O_BINARY);
    SetConsoleOutputCP(65001);
    SetConsoleCP(65001);
}

#define mkdir(path, mode) _mkdir(path)

/* vasprintf: rozmiar mierzy _vscprintf, potem zwykły vsnprintf. */
static int sw_vasprintf(char **out, const char *fmt, va_list ap) {
    int n = _vscprintf(fmt, ap);
    if (n < 0) { *out = NULL; return -1; }
    *out = (char *)malloc((size_t)n + 1);
    if (!*out) return -1;
    return vsnprintf(*out, (size_t)n + 1, fmt, ap);
}
#define vasprintf sw_vasprintf

/* memmem: igła w stogu siana. */
static void *sw_memmem(const void *h, size_t hl, const void *n, size_t nl) {
    const char *p = (const char *)h, *q = (const char *)n;
    if (nl == 0) return (void *)p;
    if (nl > hl) return NULL;
    for (size_t i = 0; i + nl <= hl; i++)
        if (p[i] == q[0] && memcmp(p + i, q, nl) == 0) return (void *)(p + i);
    return NULL;
}
#define memmem sw_memmem

/* getline: wiersz z pliku, razem z końcem, powiększając bufor. */
static ssize_t sw_getline(char **l, size_t *cap, FILE *f) {
    size_t n = 0;
    int c;
    if (*l == NULL) {
        *cap = 128;
        *l = (char *)malloc(*cap);
        if (!*l) return -1;
    }
    for (;;) {
        c = fgetc(f);
        if (c == EOF) { if (n == 0) return -1; break; }
        if (n + 2 > *cap) {
            *cap *= 2;
            char *t = (char *)realloc(*l, *cap);
            if (!t) return -1;
            *l = t;
        }
        (*l)[n++] = (char)c;
        if (c == '\n') break;
    }
    (*l)[n] = 0;
    return (ssize_t)n;
}
#define getline sw_getline

/* random i srandom: xorshift, glibc nie ma tu sobie równych. */
static unsigned long long sw_rnd = 88172645463325252ULL;
static long sw_random(void) {
    sw_rnd ^= sw_rnd << 13;
    sw_rnd ^= sw_rnd >> 7;
    sw_rnd ^= sw_rnd << 17;
    return (long)(sw_rnd & 0x7FFFFFFF);
}
static void sw_srandom(unsigned s) {
    sw_rnd = (unsigned long long)s * 2685821657736338717ULL + 1;
}
#define random sw_random
#define srandom sw_srandom

/* setjmp i longjmp: SEH-owe długie skoki (RtlUnwind) w krzyczą i zawalają program
 * przy -O2, więc na Windows używamy własnych, czysto rejestrowych, z wbudowanych gcc.
 * Klątwy zawsze lecą z wartością 1, iáko tego wbudowania żądáią. */
typedef void *sw_jmp_buf[5];
#define jmp_buf sw_jmp_buf
#undef setjmp
#define setjmp(env) __builtin_setjmp(env)
#undef longjmp
#define longjmp(env, val) ((void)(val), __builtin_longjmp(env, 1))
#endif

typedef enum { T_NIC = 0, T_LICZBA, T_ULAMEK, T_SAD, T_PISMO, T_REGESTR, T_ROD, T_SPIS } Typ;

typedef struct Pismo { size_t len; size_t zn; char d[]; } Pismo;
typedef struct Regestr Regestr;
typedef struct Rod Rod;
typedef struct Spis Spis;
typedef struct W {
    Typ t;
    union { long long i; double f; int b; Pismo *s; Regestr *r; Rod *o; Spis *p; } u;
} W;
struct Regestr { size_t len, cap; W *el; };
struct Rod { int rod; W *pola; };
/* Spis: klucze w porządku wkładania, obok tablica skrótów (numery kluczy, -1 = wolne miejsce). */
struct Spis { size_t len, cap, nidx; W *klucze, *wart; long long *idx; };

#define SW_NIC ((W){0})

/*@@TABLICE@@*/

/* ─────────── grzechy i próby ─────────── */

/* Każde «Spróbuj» kładzie na stos miejsce powrotu. Grzech albo klątwa pod próbą
 * nie zabija programu, lecz wraca do najbliższej biady z obwieszczeniem w sw_bieda. */
static jmp_buf *sw_proby;
static int sw_proby_n, sw_proby_cap;
static W sw_bieda;

static W sw_pismo_n(const char *s, size_t n);

static void sw_grzech(int wiersz, const char *fmt, ...) {
    if (sw_proby_n > 0) {
        va_list ap;
        va_start(ap, fmt);
        char *t = NULL;
        int n = vasprintf(&t, fmt, ap);
        va_end(ap);
        sw_bieda = n > 0 ? sw_pismo_n(t, (size_t)n) : sw_pismo_n("", 0);
        free(t);
        longjmp(sw_proby[--sw_proby_n], 1);
    }
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

static jmp_buf *sw_proba_wejdz(void) {
    if (sw_proby_n == sw_proby_cap) {
        sw_proby_cap = sw_proby_cap ? sw_proby_cap * 2 : 16;
        sw_proby = sw_realloc(sw_proby, (size_t)sw_proby_cap * sizeof(jmp_buf));
    }
    return &sw_proby[sw_proby_n++];
}

static void sw_proba_wyjdz(void) { sw_proby_n--; }

/* ─────────── wartości proste ─────────── */

static W sw_liczba(long long i) { W w; w.t = T_LICZBA; w.u.i = i; return w; }
static W sw_ulamek(double f) { W w; w.t = T_ULAMEK; w.u.f = f; return w; }
static W sw_sad(int b) { W w; w.t = T_SAD; w.u.b = b != 0; return w; }

static size_t sw_utf8_zn(const char *s, size_t n) {
    size_t k = 0;
    /* Tak jak w sw_off: znak to bajt z doczepionymi bajtami kontynuacji, więc i zabłąkany bajt
     * kontynuacji na samym początku (z pliku w złym kodowaniu) liczy się za znak. */
    for (size_t i = 0; i < n; i++)
        if (i == 0 || ((unsigned char)s[i] & 0xC0) != 0x80) k++;
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
    case T_SPIS: return "ſpis";
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

/* Rzeczy właśnie obwieszczane, od wierzchu w głąb: regestr, co mieści sam siebie, kończy się „…”
 * zamiast rozrastać się w 2^40 słów, gdy siebie samego ma dwakroć. */
static const void *sw_fmt_droga[48];

static void sw_fmt(Buf *b, W a, int zagniezdzone, int glebia) {
    if (glebia > 40) { buf_str(b, "…"); return; }
    const void *p = a.t == T_REGESTR ? (const void *)a.u.r : a.t == T_ROD ? (const void *)a.u.o :
                    a.t == T_SPIS ? (const void *)a.u.p : NULL;
    if (p) {
        for (int i = 0; i < glebia; i++)
            if (sw_fmt_droga[i] == p) { buf_str(b, "…"); return; }
        sw_fmt_droga[glebia] = p;
    }
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
    case T_SPIS:
        buf_str(b, "{");
        for (size_t i = 0; i < a.u.p->len; i++) {
            if (i) buf_str(b, "; ");
            sw_fmt(b, a.u.p->klucze[i], 1, glebia + 1);
            buf_str(b, ": ");
            sw_fmt(b, a.u.p->wart[i], 1, glebia + 1);
        }
        buf_str(b, "}");
        break;
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
    case T_SPIS: return a.u.p->len > 0;
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

/* Liczba cała, co nie zmieści się w 64 bitach, przechodzi w ułamek (jak przy dzieleniu
 * LLONG_MIN przez -1), miast się przekręcić — w C takie przekręcenie to i tak rzecz niedozwolona. */
static W sw_dodac(W a, W b, int L) {
    long long r;
    if (a.t == T_LICZBA && b.t == T_LICZBA && !__builtin_add_overflow(a.u.i, b.u.i, &r)) return sw_liczba(r);
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
    long long r;
    if (a.t == T_LICZBA && b.t == T_LICZBA && !__builtin_sub_overflow(a.u.i, b.u.i, &r)) return sw_liczba(r);
    if (sw_jest_liczba(a) && sw_jest_liczba(b)) return sw_ulamek(sw_d(a) - sw_d(b));
    sw_grzech(L, "Odeymowáć możná ieno liczby, á tu ſą: %s i %s.", sw_typ(a), sw_typ(b));
    return SW_NIC;
}

static W sw_powiel(W s, long long n, int L) {
    if (n < 0) sw_grzech(L, "Piſmá vieymną liczbę rázy powielić nie ſpoſób.");
    if (!s.u.s->len) return s; /* „” razy 10^18 kręciłoby się wieki, by dać „” */
    Buf b = {0};
    for (long long i = 0; i < n; i++) buf_put(&b, s.u.s->d, s.u.s->len);
    W w = sw_pismo_n(b.d ? b.d : "", b.len);
    free(b.d);
    return w;
}

static W sw_razy(W a, W b, int L) {
    long long r;
    if (a.t == T_LICZBA && b.t == T_LICZBA && !__builtin_mul_overflow(a.u.i, b.u.i, &r)) return sw_liczba(r);
    if (sw_jest_liczba(a) && sw_jest_liczba(b)) return sw_ulamek(sw_d(a) * sw_d(b));
    if (a.t == T_PISMO && sw_jest_liczba(b)) return sw_powiel(a, sw_calk(b, L), L);
    if (sw_jest_liczba(a) && b.t == T_PISMO) return sw_powiel(b, sw_calk(a, L), L);
    sw_grzech(L, "Mnożyć możná ieno liczby, á tu ſą: %s i %s.", sw_typ(a), sw_typ(b));
    return SW_NIC;
}

static W sw_przez(W a, W b, int L) {
    if (!sw_jest_liczba(a) || !sw_jest_liczba(b))
        sw_grzech(L, "Dźielić możná ieno liczby, á tu ſą: %s i %s.", sw_typ(a), sw_typ(b));
    if (sw_d(b) == 0) sw_grzech(L, "Dźielić przez zero ſię ważyſz? Rzecz bezbożna.");
    if (a.t == T_LICZBA && b.t == T_LICZBA) {
        /* LLONG_MIN / -1 nie mieści się w liczbie (a w C pada od SIGFPE). */
        if (b.u.i == -1) return a.u.i == LLONG_MIN ? sw_ulamek(-(double)a.u.i) : sw_liczba(-a.u.i);
        if (a.u.i % b.u.i == 0) return sw_liczba(a.u.i / b.u.i);
    }
    return sw_ulamek(sw_d(a) / sw_d(b));
}

static W sw_modulo(W a, W b, int L) {
    if (a.t == T_LICZBA && b.t == T_LICZBA) {
        if (b.u.i == 0) sw_grzech(L, "Reſzty z dźielenia przez zero nie dobędźieſz.");
        if (b.u.i == -1) return sw_liczba(0); /* LLONG_MIN % -1 pada w C od SIGFPE */
        long long r = a.u.i % b.u.i;
        if (r != 0 && ((r < 0) != (b.u.i < 0))) r += b.u.i;
        return sw_liczba(r);
    }
    if (sw_jest_liczba(a) && sw_jest_liczba(b)) {
        if (sw_d(b) == 0) sw_grzech(L, "Reſzty z dźielenia przez zero nie dobędźieſz.");
        /* Jak u liczb całych: reszta ma znak dzielnika. */
        double r = fmod(sw_d(a), sw_d(b));
        if (r != 0 && ((r < 0) != (sw_d(b) < 0))) r += sw_d(b);
        return sw_ulamek(r);
    }
    sw_grzech(L, "Reſztę z dźielenia brác możná ieno z liczb, á tu ſą: %s i %s.", sw_typ(a), sw_typ(b));
    return SW_NIC;
}

static W sw_potega(W a, W b, int L) {
    if (a.t == T_LICZBA && b.t == T_LICZBA && b.u.i >= 0) {
        long long r = 1, x = a.u.i, e = b.u.i;
        int za_wielka = 0;
        while (e > 0 && !za_wielka) {
            if (e & 1) za_wielka |= __builtin_mul_overflow(r, x, &r);
            e >>= 1;
            if (e) za_wielka |= __builtin_mul_overflow(x, x, &x);
        }
        if (!za_wielka) return sw_liczba(r);
    }
    if (sw_jest_liczba(a) && sw_jest_liczba(b)) return sw_ulamek(pow(sw_d(a), sw_d(b)));
    sw_grzech(L, "Do potęgi podnośić możná ieno liczby, á tu ſą: %s i %s.", sw_typ(a), sw_typ(b));
    return SW_NIC;
}

static W sw_przeciwne(W a, int L) {
    if (a.t == T_LICZBA) return a.u.i == LLONG_MIN ? sw_ulamek(-(double)a.u.i) : sw_liczba(-a.u.i);
    if (a.t == T_ULAMEK) return sw_ulamek(-a.u.f);
    sw_grzech(L, "Przećiwieńſtwá ma ieno liczbá, nie %s.", sw_typ(a));
    return SW_NIC;
}

/* ─────────── porównania ─────────── */

static long long sw_spis_szukaj(Spis *s, W k);

/* Pary regestrów (albo spisów) właśnie porównywanych. Para, co wraca w głąb samej siebie, uchodzi
 * za równą — inaczej regestr zawierający sam siebie porównywałby się do przepełnienia stosu. */
static const void **sw_rowne_pary;
static size_t sw_rowne_n, sw_rowne_cap;

static int sw_rowne_(W a, W b);

static int sw_rowne_w_toku(const void *x, const void *y) {
    for (size_t i = 0; i < sw_rowne_n; i += 2)
        if (sw_rowne_pary[i] == x && sw_rowne_pary[i + 1] == y) return 1;
    if (sw_rowne_n + 2 > sw_rowne_cap) {
        sw_rowne_cap = sw_rowne_cap ? sw_rowne_cap * 2 : 64;
        sw_rowne_pary = realloc(sw_rowne_pary, sw_rowne_cap * sizeof *sw_rowne_pary);
        if (!sw_rowne_pary) { fputs("Pámięći zábrákło.\n", stderr); exit(1); }
    }
    sw_rowne_pary[sw_rowne_n++] = x;
    sw_rowne_pary[sw_rowne_n++] = y;
    return 0;
}

static int sw_rowne_regestry(Regestr *x, Regestr *y) {
    if (sw_rowne_w_toku(x, y)) return 1;
    int r = 1;
    for (size_t i = 0; r && i < x->len; i++) r = sw_rowne_(x->el[i], y->el[i]);
    sw_rowne_n -= 2;
    return r;
}

static int sw_rowne_spisy(Spis *x, Spis *y) {
    if (sw_rowne_w_toku(x, y)) return 1;
    int r = 1;
    for (size_t i = 0; r && i < x->len; i++) {
        long long j = sw_spis_szukaj(y, x->klucze[i]);
        r = j >= 0 && sw_rowne_(x->wart[i], y->wart[j]);
    }
    sw_rowne_n -= 2;
    return r;
}

/* Liczba cała przeciw ułamkowi: -1, 0, 1, a 2 dla niewiadomej. Nie przez (double)i, bo powyżej
 * 2^53 rzutowanie gubi cyfry i 9007199254740993 wyszłoby równe 9007199254740992,0. */
static int sw_cmp_i_f(long long i, double f) {
    if (isnan(f)) return 2;
    if (f >= 9223372036854775808.0) return -1;
    if (f < -9223372036854775808.0) return 1;
    double fl = floor(f);
    long long k = (long long)fl;
    if (i != k) return i < k ? -1 : 1;
    return f > fl ? -1 : 0;
}

static int sw_rowne_(W a, W b) {
    if (sw_jest_liczba(a) && sw_jest_liczba(b)) {
        if (a.t == T_LICZBA && b.t == T_LICZBA) return a.u.i == b.u.i;
        if (a.t == T_LICZBA && b.t == T_ULAMEK) return sw_cmp_i_f(a.u.i, b.u.f) == 0;
        if (a.t == T_ULAMEK && b.t == T_LICZBA) return sw_cmp_i_f(b.u.i, a.u.f) == 0;
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
        return sw_rowne_regestry(a.u.r, b.u.r);
    case T_ROD: return a.u.o == b.u.o;
    case T_SPIS:
        if (a.u.p == b.u.p) return 1;
        if (a.u.p->len != b.u.p->len) return 0;
        return sw_rowne_spisy(a.u.p, b.u.p);
    default: return 0;
    }
}

static int sw_porownaj(W a, W b, int L) {
    if (sw_jest_liczba(a) && sw_jest_liczba(b)) {
        if (a.t == T_LICZBA && b.t == T_LICZBA) return (a.u.i > b.u.i) - (a.u.i < b.u.i);
        if (a.t == T_LICZBA && b.t == T_ULAMEK) { int c = sw_cmp_i_f(a.u.i, b.u.f); return c == 2 ? 0 : c; }
        if (a.t == T_ULAMEK && b.t == T_LICZBA) { int c = sw_cmp_i_f(b.u.i, a.u.f); return c == 2 ? 0 : -c; }
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
    return sw_sad(y == -1 || x % y == 0);
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
    case T_SPIS: return sw_sad(a.u.p->len == 0);
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

/* ─────────── spisy ─────────── */

static unsigned long long sw_skrot(W k) {
    unsigned long long h;
    switch (k.t) {
    case T_LICZBA:
        h = (unsigned long long)k.u.i;
        h ^= h >> 33;
        h *= 0xff51afd7ed558ccdULL;
        return h ^ (h >> 33);
    case T_SAD: return k.u.b ? 0x9e3779b97f4a7c15ULL : 0x7f4a7c159e3779b9ULL;
    case T_PISMO:
        h = 1469598103934665603ULL;
        for (size_t i = 0; i < k.u.s->len; i++) {
            h ^= (unsigned char)k.u.s->d[i];
            h *= 1099511628211ULL;
        }
        return h;
    default: return 0;
    }
}

/* Klucz zdatny do spisu; ułamek bez reszty staje się liczbą, by 2 i 2,0 trafiały w jedno. */
static W sw_klucz(W k, int L) {
    if (k.t == T_ULAMEK && k.u.f == floor(k.u.f) && fabs(k.u.f) < 9e18) return sw_liczba((long long)k.u.f);
    if (k.t == T_LICZBA || k.t == T_PISMO || k.t == T_SAD) return k;
    sw_grzech(L, "Kluczem w ſpiśie bywá liczbá, piſmo álbo prawdá, nie %s.", sw_typ(k));
    return SW_NIC;
}

static W sw_spis_nowy(void) {
    Spis *s = sw_alloc(sizeof *s);
    memset(s, 0, sizeof *s);
    W w;
    w.t = T_SPIS;
    w.u.p = s;
    return w;
}

static long long sw_spis_szukaj(Spis *s, W k) {
    if (k.t == T_ULAMEK && k.u.f == floor(k.u.f) && fabs(k.u.f) < 9e18) k = sw_liczba((long long)k.u.f);
    if (!s->nidx || (k.t != T_LICZBA && k.t != T_PISMO && k.t != T_SAD)) return -1;
    size_t m = s->nidx - 1, i = (size_t)sw_skrot(k) & m;
    for (;;) {
        long long j = s->idx[i];
        if (j < 0) return -1;
        if (sw_rowne_(s->klucze[j], k)) return j;
        i = (i + 1) & m;
    }
}

static void sw_spis_indeksuj(Spis *s) {
    size_t n = 16;
    while (n < s->len * 2 + 2) n *= 2;
    s->idx = sw_realloc(s->idx, n * sizeof(long long));
    s->nidx = n;
    for (size_t i = 0; i < n; i++) s->idx[i] = -1;
    for (size_t j = 0; j < s->len; j++) {
        size_t i = (size_t)sw_skrot(s->klucze[j]) & (n - 1);
        while (s->idx[i] >= 0) i = (i + 1) & (n - 1);
        s->idx[i] = (long long)j;
    }
}

static void sw_spis_wloz(W sp, W k, W v, int L) {
    Spis *s = sp.u.p;
    k = sw_klucz(k, L);
    long long j = sw_spis_szukaj(s, k);
    if (j >= 0) {
        s->wart[j] = v;
        return;
    }
    if (s->len == s->cap) {
        s->cap = s->cap ? s->cap * 2 : 8;
        s->klucze = sw_realloc(s->klucze, s->cap * sizeof(W));
        s->wart = sw_realloc(s->wart, s->cap * sizeof(W));
    }
    s->klucze[s->len] = k;
    s->wart[s->len] = v;
    s->len++;
    if (s->len * 2 + 2 > s->nidx) {
        sw_spis_indeksuj(s);
    } else {
        size_t i = (size_t)sw_skrot(k) & (s->nidx - 1);
        while (s->idx[i] >= 0) i = (i + 1) & (s->nidx - 1);
        s->idx[i] = (long long)(s->len - 1);
    }
}

/* Spis z par: (W[]){klucz, wartość, klucz, wartość, …}. */
static W sw_spis_z(int n, W *pary, int L) {
    W w = sw_spis_nowy();
    for (int i = 0; i < n; i++) sw_spis_wloz(w, pary[2 * i], pary[2 * i + 1], L);
    return w;
}

static W sw_spis_wez(W sp, W k, int L) {
    long long j = sw_spis_szukaj(sp.u.p, sw_klucz(k, L));
    return j < 0 ? SW_NIC : sp.u.p->wart[j];
}

static void sw_spis_wyrzuc(Spis *s, W k) {
    long long j = sw_spis_szukaj(s, k);
    if (j < 0) return;
    size_t reszta = s->len - (size_t)j - 1;
    memmove(s->klucze + j, s->klucze + j + 1, reszta * sizeof(W));
    memmove(s->wart + j, s->wart + j + 1, reszta * sizeof(W));
    s->len--;
    sw_spis_indeksuj(s);
}

static W sw_b_klucze(W x, int L) {
    if (x.t != T_SPIS) sw_grzech(L, "Klucze ma ieno ſpis, nie %s.", sw_typ(x));
    return sw_regestr_z((int)x.u.p->len, x.u.p->klucze);
}

static W sw_b_wartosci(W x, int L) {
    if (x.t != T_SPIS) sw_grzech(L, "Wártośći po kluczach ma ieno ſpis, nie %s.", sw_typ(x));
    return sw_regestr_z((int)x.u.p->len, x.u.p->wart);
}

/* Zaczątek dla «Zwiększ ceny pod „chleb”», gdy klucza jeszcze nie ma: zero, jedynka,
 * puste pismo albo nowy regestr — zależnie od tego, co się dokłada.
 * Rachunki: 0 dodać, 1 odjąć, 2 mnożyć, 3 dzielić, 4 dołożyć. */
static W sw_zaczatek(W cur, W v, int rachunek) {
    if (cur.t != T_NIC) return cur;
    /* «Dodaj» zaczyna od regestru i dla liczb (statut: „brakujący klucz zaczyna od pustego
     * regestru”), inaczej «Dodaj 5 do ocen pod „Jan”» liczyłoby 0 + 5, a nie [5]. */
    if (rachunek == 4) return sw_regestr_nowy();
    if (sw_jest_liczba(v)) return rachunek == 2 ? sw_liczba(1) : rachunek == 3 ? cur : sw_liczba(0);
    if (rachunek == 0 && v.t == T_PISMO) return sw_pismo_n("", 0);
    return cur;
}

/* ─────────── regestry ─────────── */

static W sw_element(W c, W i, int L) {
    if (c.t == T_SPIS) return sw_spis_wez(c, i, L);
    if (c.t == T_REGESTR) return c.u.r->el[sw_indeks(i, (long long)c.u.r->len, L)];
    if (c.t == T_PISMO) return sw_znak(c.u.s, (size_t)sw_indeks(i, (long long)c.u.s->zn, L));
    sw_grzech(L, "Z rzeczy tákiey (%s) elementu nie wyiąć.", sw_typ(c));
    return SW_NIC;
}

static void sw_ustaw(W c, W i, W v, int L) {
    if (c.t == T_PISMO) sw_grzech(L, "Piſmá zmieniáć nie lza — vtwórz nowe.");
    if (c.t == T_SPIS) {
        sw_spis_wloz(c, i, v, L);
        return;
    }
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
    if (c.t == T_SPIS) return sw_liczba((long long)c.u.p->len);
    sw_grzech(L, "Długośći nie ma rzecz táka: %s.", sw_typ(c));
    return SW_NIC;
}

static long long sw_iter_dl(W c, int L) {
    if (c.t == T_REGESTR) return (long long)c.u.r->len;
    if (c.t == T_PISMO) return (long long)c.u.s->zn;
    if (c.t == T_SPIS) return (long long)c.u.p->len;
    sw_grzech(L, "Po rzeczy tákiey (%s) chodźić element po elemenćie nie ſpoſób.", sw_typ(c));
    return 0;
}

static W sw_iter_el(W c, long long k, int L) {
    (void)L;
    if (c.t == T_REGESTR) return c.u.r->el[k];
    if (c.t == T_SPIS) return c.u.p->klucze[k];
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
        if (x.t == T_NIC) return sw_sad(0); /* inaczej „nicpoń” zawierałby nic */
        W s = sw_napis(x);
        return sw_sad(memmem(c.u.s->d, c.u.s->len, s.u.s->d, s.u.s->len) != NULL);
    }
    if (c.t == T_SPIS) return sw_sad(sw_spis_szukaj(c.u.p, x) >= 0);
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

/* «Usuń „chleb” z cen»: ze spisu wyrzuca klucz, z regestru pierwszy równy element. */
static W sw_usun_wartosc(W c, W v, int L) {
    if (c.t == T_SPIS) {
        sw_spis_wyrzuc(c.u.p, v);
        return SW_NIC;
    }
    if (c.t != T_REGESTR) sw_grzech(L, "Uſuwáć możná ieno z regeſtru álbo ze ſpiſu, nie z: %s.", sw_typ(c));
    Regestr *r = c.u.r;
    for (size_t k = 0; k < r->len; k++)
        if (sw_rowne_(r->el[k], v)) {
            memmove(r->el + k, r->el + k + 1, (r->len - k - 1) * sizeof(W));
            r->len--;
            break;
        }
    return SW_NIC;
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

/* Miejsce pola liczymy osobno, nim sięgniemy po `o.u.o->pola`: w jednym wyrażeniu C wolno
 * najpierw odczytać `o.u.o` — a gdy `o` to nie ród, lecz liczba, pada SIGSEGV, nim
 * sw_slot_w zdąży zgłosić grzech, który próba mogłaby złapać. */
static W sw_pole(W o, int pole, int L) {
    int s = sw_slot_w(o, pole, L);
    return o.u.o->pola[s];
}

static void sw_ustaw_pole(W o, int pole, W v, int L) {
    int s = sw_slot_w(o, pole, L);
    o.u.o->pola[s] = v;
}

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
        int cyfra = 0, obce = 0;
        for (char *c = t; *c; c++) {
            if (*c == ',') *c = '.';
            if (*c >= '0' && *c <= '9') cyfra = 1;
            /* strtod czyta i liczby szesnastkowe („0x10”); te są pismem, nie liczbą. */
            if (*c == 'x' || *c == 'X') obce = 1;
        }
        if (cyfra && !obce) {
            char *end;
            errno = 0;
            long long i = strtoll(t, &end, 10);
            /* Liczba ponad miarę („99999999999999999999”) nie przycina się po cichu do LLONG_MAX. */
            if (*end == 0 && errno != ERANGE) return sw_liczba(i);
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

/* Pismo w roli ścieżki. Znak zerowy w środku uciąłby ją po cichu (C czyta do zera)
 * i program sięgnąłby po cudzy plik — „tajne\0.txt” znaczyłoby „tajne”. */
static int sw_ma_zero(W path) { return memchr(path.u.s->d, 0, path.u.s->len) != NULL; }
static const char *sw_sciezka(W p, int L) {
    W path = sw_napis(p);
    if (sw_ma_zero(path)) sw_grzech(L, "Śćieżká nie może záwieráć znáku zerowego.");
    return path.u.s->d;
}

static W sw_b_tresc(W p, int L) {
    const char *path = sw_sciezka(p, L);
    FILE *f = fopen(path, "rb");
    if (!f) sw_grzech(L, "Pliku „%s” nie názleźiono.", path);
    struct stat st;
    if (fstat(fileno(f), &st) == 0 && S_ISDIR(st.st_mode)) {
        fclose(f);
        sw_grzech(L, "„%s” ieſt kátálogiem, nie plikiem.", path);
    }
    Buf b = {0};
    char t[4096];
    size_t n;
    while ((n = fread(t, 1, sizeof t, f)) > 0) buf_put(&b, t, n);
    if (ferror(f)) {
        int blad = errno;
        fclose(f);
        free(b.d);
        sw_grzech(L, "Pliku „%s” odczytáć nie ſpoſób: %s.", path, strerror(blad));
    }
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
    const char *path = sw_sciezka(p, L);
    FILE *f = fopen(path, dopisz ? "ab" : "wb");
    if (!f) sw_grzech(L, "Do pliku „%s” piſáć nie podobná.", path);
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
    if (x.t == T_LICZBA) {
        /* Liczbę całą daje tylko kwadrat liczby całej. Sprawdzamy w liczbach całych, bo powyżej 2^53
         * (double)x gubi cyfry i sqrt(10^16 + 1) wychodzi okrągłe 10^8. */
        unsigned long long n = (unsigned long long)x.u.i, k = (unsigned long long)llround(r);
        while (k * k > n) k--;
        while ((k + 1) * (k + 1) <= n) k++;
        if (k * k == n) return sw_liczba((long long)k);
    }
    return sw_ulamek(r);
}

static W sw_b_abs(W x, int L) {
    if (x.t == T_LICZBA && x.u.i == LLONG_MIN) return sw_ulamek(-(double)x.u.i);
    if (x.t == T_LICZBA) return sw_liczba(x.u.i < 0 ? -x.u.i : x.u.i);
    if (x.t == T_ULAMEK) return sw_ulamek(fabs(x.u.f));
    sw_grzech(L, "Wártość bezwzględną ma ieno liczbá, nie %s.", sw_typ(x));
    return SW_NIC;
}

/* Ułamek bez reszty jako liczba cała; zbyt wielki, by się w niej zmieścić, zostaje ułamkiem. */
static W sw_ulamek_w_liczbe(double f, int L) {
    if (isnan(f) || isinf(f)) sw_grzech(L, "Niewiádomey ániżeli nieſkończoności w liczbę cáłą nie obróciſz.");
    if (f >= -9223372036854775808.0 && f < 9223372036854775808.0) return sw_liczba((long long)f);
    return sw_ulamek(f);
}

static W sw_b_zaokraglenie(W x, int L) {
    if (x.t == T_LICZBA) return x;
    if (x.t == T_ULAMEK) return sw_ulamek_w_liczbe(round(x.u.f), L);
    sw_grzech(L, "Zaokrąglić możná ieno liczbę, nie %s.", sw_typ(x));
    return SW_NIC;
}

static W sw_b_calkowita(W x, int L) {
    if (x.t == T_LICZBA) return x;
    if (x.t == T_ULAMEK) return sw_ulamek_w_liczbe(trunc(x.u.f), L);
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
    /* Rozpiętość liczona bez znaku: od LLONG_MIN do LLONG_MAX to 2^64, czyli 0 po zawinięciu. */
    unsigned long long span = (unsigned long long)y - (unsigned long long)x + 1;
    unsigned long long r = ((unsigned long long)random() << 62) ^ ((unsigned long long)random() << 31) ^
                           (unsigned long long)random();
    if (span) r %= span;
    return sw_liczba((long long)((unsigned long long)x + r));
}

/* Znak UTF-8 spod `s`: zwraca kod znaku i jego długość w `dl`; -1 dla bajtu, co znakiem nie jest. */
static int sw_dekoduj(const unsigned char *s, size_t n, size_t *dl) {
    *dl = 1;
    if (s[0] < 0x80) return s[0];
    size_t k = (s[0] & 0xE0) == 0xC0 ? 2 : (s[0] & 0xF0) == 0xE0 ? 3 : (s[0] & 0xF8) == 0xF0 ? 4 : 0;
    if (!k || n < k) return -1;
    int cp = s[0] & (0x7F >> k);
    for (size_t i = 1; i < k; i++) {
        if ((s[i] & 0xC0) != 0x80) return -1;
        cp = (cp << 6) | (s[i] & 0x3F);
    }
    *dl = k;
    return cp;
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

/* Wielka (albo mała) para litery z Latin-1 i Latin Extended-A, gdzie leżą litery polskie
 * (ą ć ę ł ń ó ś ź ż), dawne (á é ſ) i sąsiedzkie (ä ö ü č ř…). Bez setlocale: wynik nie zależy
 * od tego, gdzie program puszczono. */
static int sw_wielka(int c) {
    if (c >= 'a' && c <= 'z') return c - 32;
    if (c >= 0xE0 && c <= 0xFE && c != 0xF7) return c - 0x20;
    if (c == 0xFF) return 0x178;
    if (c == 0x131) return 'I';
    if (c == 0x17F) return 'S'; /* ſ */
    if ((c >= 0x100 && c <= 0x137) || (c >= 0x14A && c <= 0x177)) return c & 1 ? c - 1 : c;
    if ((c >= 0x139 && c <= 0x148) || (c >= 0x179 && c <= 0x17E)) return c & 1 ? c : c - 1;
    return c;
}

static int sw_mala(int c) {
    if (c >= 'A' && c <= 'Z') return c + 32;
    if (c >= 0xC0 && c <= 0xDE && c != 0xD7) return c + 0x20;
    if (c == 0x178) return 0xFF;
    if (c == 0x130) return 'i';
    if ((c >= 0x100 && c <= 0x137) || (c >= 0x14A && c <= 0x177)) return c & 1 ? c : c + 1;
    if ((c >= 0x139 && c <= 0x148) || (c >= 0x179 && c <= 0x17E)) return c & 1 ? c + 1 : c;
    return c;
}

static W sw_litery(W x, int wielkie, int L) {
    if (x.t != T_PISMO) sw_grzech(L, "Litery zmieniáć możná ieno w piśmie, nie w: %s.", sw_typ(x));
    Buf b = {0};
    const unsigned char *s = (const unsigned char *)x.u.s->d;
    size_t n = x.u.s->len, i = 0;
    while (i < n) {
        size_t dl;
        int stary = sw_dekoduj(s + i, n - i, &dl), cp = stary;
        if (cp >= 0) cp = wielkie ? sw_wielka(cp) : sw_mala(cp);
        /* Znak bez pary (emoji, ß, zły bajt) przechodzi bajt w bajt. */
        if (cp == stary) buf_put(&b, (const char *)s + i, dl);
        else sw_koduj(&b, cp);
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
    if (x.t == T_SPIS) {
        W w = sw_spis_nowy();
        for (size_t i = 0; i < x.u.p->len; i++) sw_spis_wloz(w, x.u.p->klucze[i], x.u.p->wart[i], L);
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

/* ─────────── klątwy i rodzaje ─────────── */

static void sw_rzuc(W v, int L) {
    if (sw_proby_n > 0) {
        sw_bieda = v;
        longjmp(sw_proby[--sw_proby_n], 1);
    }
    sw_grzech(L, "Klątwá nie zdięta: %s", sw_napis(v).u.s->d);
}

/* Rodzaje: 0 nicość, 1 liczba, 2 ułamek, 3 pismo, 4 regestr, 5 spis, 6 prawda albo fałsz,
 * 7 liczba całkowita, 100 + n — ród o numerze n. */
static W sw_jest_typu(W a, int rodzaj, int L) {
    (void)L;
    switch (rodzaj) {
    case 0: return sw_sad(a.t == T_NIC);
    case 1: return sw_sad(sw_jest_liczba(a));
    case 2: return sw_sad(a.t == T_ULAMEK);
    case 3: return sw_sad(a.t == T_PISMO);
    case 4: return sw_sad(a.t == T_REGESTR);
    case 5: return sw_sad(a.t == T_SPIS);
    case 6: return sw_sad(a.t == T_SAD);
    case 7: return sw_sad(a.t == T_LICZBA);
    default: return sw_sad(a.t == T_ROD && a.u.o->rod == rodzaj - 100);
    }
}

static W sw_b_rodzaj(W a, int L) {
    (void)L;
    const char *s = sw_typ(a);
    return sw_pismo_n(s, strlen(s));
}

/* ─────────── liczby rzymskie i słowne ─────────── */

static W sw_rzymskie(W a, int L) {
    long long n = sw_calk(a, L);
    if (n < 0) sw_grzech(L, "Rzymiánie liczb vieymnych nie ználi.");
    if (n > 100000) sw_grzech(L, "Liczby %lld Rzymiánie nie piſáli — tyle «M» nie zmieśći żadna kártá.", n);
    if (n == 0) return sw_pismo_n("nulla", 5);
    static const int v[] = {1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1};
    static const char *const r[] = {"M", "CM", "D", "CD", "C", "XC", "L", "XL", "X", "IX", "V", "IV", "I"};
    Buf b = {0};
    for (int i = 0; i < 13; i++)
        while (n >= v[i]) {
            buf_str(&b, r[i]);
            n -= v[i];
        }
    W w = sw_pismo_n(b.d, b.len);
    free(b.d);
    return w;
}

/* Liczebniki pisane tak, by skryba umiał je przeczytać z powrotem. */
static const char *const sw_sl_jedn[] = {"", "ieden", "dwá", "trzy", "cztery", "pięć", "ſześć", "ſiedem", "ośiem", "dźiewięć"};
static const char *const sw_sl_nast[] = {"dźieśięć", "iedenaśćie", "dwánaśćie", "trzynaśćie", "czternaśćie",
                                         "piętnaśćie", "ſzesnaśćie", "ſiedemnaśćie", "ośiemnaśćie", "dźiewiętnaśćie"};
static const char *const sw_sl_dzies[] = {"", "", "dwádźieśćiá", "trzydźieśći", "czterdźieśći", "pięćdźieśiąt",
                                          "ſześćdźieśiąt", "ſiedemdźieśiąt", "ośiemdźieśiąt", "dźiewięćdźieśiąt"};
static const char *const sw_sl_setki[] = {"", "ſto", "dwieśćie", "trzyſtá", "czteryſtá", "pięćſet",
                                          "ſześćſet", "ſiedemſet", "ośiemſet", "dźiewięćſet"};
static const char *const sw_sl_rzedy[][3] = {
    {"", "", ""},
    {"tyśiąc", "tyśiące", "tyśięcy"},
    {"milion", "miliony", "milionów"},
    {"miliárd", "miliárdy", "miliárdów"},
    {"bilion", "biliony", "bilionów"},
    {"biliárd", "biliárdy", "biliárdów"},
    {"trylion", "tryliony", "trylionów"},
};

static void sw_sl_slowo(Buf *b, const char *s) {
    if (!*s) return;
    if (b->len && b->d[b->len - 1] != ' ') buf_str(b, " ");
    buf_str(b, s);
}

static W sw_slownie(W a, int L) {
    long long x = sw_calk(a, L);
    if (x == 0) return sw_pismo_n("zero", 4);
    Buf b = {0};
    unsigned long long n = x < 0 ? 0ULL - (unsigned long long)x : (unsigned long long)x;
    if (x < 0) buf_str(&b, "minus");
    unsigned long long dziel = 1;
    int rzad = 0;
    while (n / dziel >= 1000) {
        dziel *= 1000;
        rzad++;
    }
    for (; rzad >= 0; rzad--, dziel /= 1000) {
        int t = (int)(n / dziel % 1000);
        if (!t) continue;
        if (!(rzad > 0 && t == 1)) {
            sw_sl_slowo(&b, sw_sl_setki[t / 100]);
            int d = t % 100;
            if (d >= 10 && d < 20) sw_sl_slowo(&b, sw_sl_nast[d - 10]);
            else {
                sw_sl_slowo(&b, sw_sl_dzies[d / 10]);
                sw_sl_slowo(&b, sw_sl_jedn[d % 10]);
            }
        }
        if (rzad > 0) {
            int j = t % 10, d = t % 100;
            int forma = t == 1 ? 0 : (j >= 2 && j <= 4 && !(d >= 12 && d <= 14)) ? 1 : 2;
            sw_sl_slowo(&b, sw_sl_rzedy[rzad][forma]);
        }
    }
    W w = sw_pismo_n(b.d, b.len);
    free(b.d);
    return w;
}

/* ─────────── pliki i katalogi ─────────── */

static W sw_b_istnieje(W p, int L) {
    (void)L;
    W path = sw_napis(p);
    struct stat st;
    return sw_sad(!sw_ma_zero(path) && stat(path.u.s->d, &st) == 0);
}

static W sw_b_usun_plik(W p, int L) {
    const char *path = sw_sciezka(p, L);
    if (remove(path) != 0) sw_grzech(L, "Pliku „%s” vſunąć nie podobná: %s.", path, strerror(errno));
    return SW_NIC;
}

static W sw_b_katalog(W p, int L) {
    W path = sw_napis(p);
    sw_sciezka(path, L);
    if (!path.u.s->len) sw_grzech(L, "Kátálog o puſtym miániu záłożony być nie może.");
    char *d = path.u.s->d;
    for (size_t i = 1; i <= path.u.s->len; i++) {
        if (i < path.u.s->len && d[i] != '/') continue;
        char c = d[i];
        d[i] = 0;
        int r = mkdir(d, 0777);
        int blad = errno;
        d[i] = c;
        if (r != 0 && blad != EEXIST)
            sw_grzech(L, "Kátálogu „%s” záłożyć nie podobná: %s.", d, strerror(blad));
    }
    struct stat st;
    if (stat(d, &st) != 0 || !S_ISDIR(st.st_mode))
        sw_grzech(L, "„%s” ſtoi iuż ná tym mieyſcu, á kátálogiem nie ieſt.", d);
    return SW_NIC;
}

static W sw_b_przenies(W a, W b, int L) {
    const char *z = sw_sciezka(a, L), *dokad = sw_sciezka(b, L);
    if (rename(z, dokad) != 0) sw_grzech(L, "Pliku „%s” do „%s” przenieść nie podobná: %s.", z, dokad, strerror(errno));
    return SW_NIC;
}

static int sw_por_miana(const void *a, const void *b) { return strcmp(*(char *const *)a, *(char *const *)b); }

static W sw_b_zawartosc(W p, int L) {
    const char *path = sw_sciezka(p, L);
    DIR *dir = opendir(path);
    if (!dir) sw_grzech(L, "Kátálogu „%s” przeyrzeć nie podobná: %s.", path, strerror(errno));
    char **miana = NULL;
    size_t n = 0, cap = 0;
    struct dirent *e;
    while ((e = readdir(dir))) {
        if (!strcmp(e->d_name, ".") || !strcmp(e->d_name, "..")) continue;
        if (n == cap) {
            cap = cap ? cap * 2 : 16;
            miana = sw_realloc(miana, cap * sizeof(char *));
        }
        miana[n++] = strdup(e->d_name);
    }
    closedir(dir);
    if (n) qsort(miana, n, sizeof(char *), sw_por_miana); /* qsort(NULL, 0…) to w C rzecz niedozwolona */
    W w = sw_regestr_nowy();
    for (size_t i = 0; i < n; i++) {
        sw_wloz(w, sw_pismo_n(miana[i], strlen(miana[i])));
        free(miana[i]);
    }
    free(miana);
    return w;
}

/* ─────────── początek i koniec ─────────── */

static int sw_argc;
static char **sw_argv;

static void sw_start(int argc, char **argv) {
    sw_argc = argc;
    sw_argv = argv;
#if defined(_WIN32)
    sw_konsola_utf8();
#endif
    srandom((unsigned)time(NULL) ^ (unsigned)getpid());
}

/* Słowa, z którymi program wywołano (bez miana samego programu). */
static W sw_b_slowa_wywolania(int L) {
    (void)L;
    W w = sw_regestr_nowy();
    for (int i = 1; i < sw_argc; i++) sw_wloz(w, sw_auto(sw_argv[i], strlen(sw_argv[i])));
    return w;
}

static void sw_koniec(void) { fflush(stdout); }

static void sw_zakoncz(W kod, int L) {
    long long k = sw_calk(kod, L);
    sw_koniec();
    exit((int)(k & 255));
}
