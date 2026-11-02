/*
 * Beispiel: Katalog lesen, Einstellungen setzen, Vorschaubilder ohne Fenster rechnen.
 * SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Bauen (Linux):
 *   cargo build --release -p roehre-c
 *   cc crates/roehre-c/beispiele/vorschau.c -Icrates/roehre-c/include -Ltarget/release -lroehre -o vorschau
 *   LD_LIBRARY_PATH=target/release ./vorschau shader ausgabe
 * Schreibt ausgabe_flach.ppm und ausgabe_raum.ppm (PPM: jedes Bildprogramm kann es öffnen).
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "roehre.h"

static int ppm_schreiben(const char *pfad, const uint8_t *rgba, uint32_t b, uint32_t h) {
    FILE *f = fopen(pfad, "wb");
    if (!f) return -1;
    fprintf(f, "P6\n%u %u\n255\n", b, h);
    for (uint32_t i = 0; i < b * h; i++) fwrite(rgba + 4 * i, 1, 3, f);
    fclose(f);
    return 0;
}

int main(int argc, char **argv) {
    const char *shader = argc > 1 ? argv[1] : "shader";
    const char *ausgabe = argc > 2 ? argv[2] : "vorschau";
    printf("roehre %s\n", roehre_version());

    /* Katalog: nur zeigen, dass er da ist */
    const char *katalog = roehre_katalog_json();
    printf("Katalog: %zu Zeichen, enthält raum.naehe: %s\n", strlen(katalog), strstr(katalog, "\"raum.naehe\"") ? "ja" : "nein");

    RoehreEinstellungen *e = roehre_einstellungen_voreinstellung("wohnzimmer-90er-composite");
    if (!e) { fprintf(stderr, "Fehler: %s\n", roehre_letzter_fehler()); return 1; }
    if (roehre_einstellungen_setzen(e, "raum.naehe", "0,5") < 0) { fprintf(stderr, "%s\n", roehre_letzter_fehler()); return 1; }
    roehre_einstellungen_setzen(e, "raum.lage", "1");
    if (roehre_einstellungen_setzen(e, "raum.gibtsnicht", "1") >= 0) { fprintf(stderr, "unbekannte Option nicht abgelehnt\n"); return 1; }
    printf("abgelehnt wie erwartet: %s\n", roehre_letzter_fehler());
    char *wert = roehre_einstellungen_wert(e, "raum.naehe");
    printf("raum.naehe = %s\n", wert);
    roehre_text_freigeben(wert);
    char *toml = roehre_einstellungen_als_text(e, 0, 1);
    printf("--- gespeichert (nur Abweichungen) ---\n%s---\n", toml);
    RoehreEinstellungen *wieder = roehre_einstellungen_aus_text(toml);
    roehre_text_freigeben(toml);
    if (!wieder) { fprintf(stderr, "Einlesen: %s\n", roehre_letzter_fehler()); return 1; }

    RoehreVorschau *v = roehre_vorschau_erstellen(shader);
    if (!v) { fprintf(stderr, "Vorschau: %s\n", roehre_letzter_fehler()); return 1; }
    const uint32_t b = 640, h = 360;
    uint8_t *px = malloc(b * h * 4);
    char pfad[512];
    for (int modus = 0; modus <= 1; modus++) {
        if (roehre_vorschau_zeichnen(v, wieder, modus, 0.0f, NULL, 0, 0, 1.0f, px, b, h) < 0) {
            fprintf(stderr, "Zeichnen: %s\n", roehre_letzter_fehler());
            return 1;
        }
        snprintf(pfad, sizeof pfad, "%s_%s.ppm", ausgabe, modus ? "raum" : "flach");
        ppm_schreiben(pfad, px, b, h);
        printf("%s geschrieben\n", pfad);
    }
    free(px);
    roehre_vorschau_freigeben(v);
    roehre_einstellungen_freigeben(wieder);
    roehre_einstellungen_freigeben(e);
    return 0;
}
