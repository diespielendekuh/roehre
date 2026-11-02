/*
 * roehre – Röhrenbild für Spiele. C-Schnittstelle.
 * SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Bibliothek: libroehre.so / roehre.dll (+ roehre.dll.lib) / libroehre.a, gebaut aus
 * crates/roehre-c (cargo build --release -p roehre-c). Zur Laufzeit braucht roehre den Ordner
 * shader/ (enthält GPL-Shader als eigene Dateien, siehe THIRD_PARTY.md).
 *
 * Grundregeln
 *  - Alle Texte sind UTF-8 und nullterminiert.
 *  - Fehler: Rückgabe < 0 bzw. NULL; den Grund liefert roehre_letzter_fehler() (je Thread).
 *  - Texte mit "Freigeben mit roehre_text_freigeben" gehören dem Aufrufer; alle anderen nicht.
 *  - Ein Objekt (Einstellungen, Vorschau, Austausch) wird nicht von zwei Threads zugleich
 *    benutzt. Verschiedene Objekte dürfen in verschiedenen Threads leben.
 */
#ifndef ROEHRE_H
#define ROEHRE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ------------------------------------------------------------------ Allgemein */

/** Version der Bibliothek, z. B. "0.1.0". Nicht freigeben. */
const char *roehre_version(void);

/** Grund des letzten Fehlers in diesem Thread (oder Hinweise nach dem Laden). Nicht freigeben;
 *  gültig bis zum nächsten Aufruf von roehre in diesem Thread. */
const char *roehre_letzter_fehler(void);

/** Gibt einen Text frei, den roehre herausgegeben hat. NULL ist erlaubt. */
void roehre_text_freigeben(char *text);

/* ------------------------------------------------------------------ Katalog */

/**
 * Alle Optionen als JSON, damit ein eigenes Menü sie anzeigen kann. Nicht freigeben.
 *
 * {
 *   "version": "0.1.0", "einstellungsversion": 1,
 *   "gruppen":  [ { "schluessel": "raum", "name": { "de": "3D-Raum", "en": "3D room" } }, ... ],
 *   "optionen": [ {
 *       "schluessel": "raum.naehe", "gruppe": "raum",
 *       "name": {"de","en"}, "beschreibung": {"de","en"},
 *       "kosten": "keine" | "gering" | "hoch",
 *       "wirkt_wenn": null | { "option": "raum.schwenk", "werte": ["an"] },
 *       "art": "zahl",     "min", "max", "schritt", "standard", "einheit", "festwerte": [ {"schluessel","name","wert"} ]
 *            | "ganzzahl", "min", "max", "standard"
 *            | "schalter", "standard"
 *            | "auswahl",  "standard", "werte": [ {"schluessel","name"} ]
 *   }, ... ],
 *   "voreinstellungen": [ { "schluessel": "scart-rgb", "name", "beschreibung", "lizenz": "gpl" | "permissiv" } ]
 * }
 *
 * "festwerte" sind benannte Stufen einer stufenlosen Option (z. B. Nähe: weit/mittel/nah);
 * Stufe und Regler stellen dieselbe Option ein. "wirkt_wenn": die Option wirkt nur, wenn die
 * genannte Auswahl einen der Werte hat (im Menü ausgrauen).
 */
const char *roehre_katalog_json(void);

/* ------------------------------------------------------------------ Einstellungen */

typedef struct RoehreEinstellungen RoehreEinstellungen;

/** Alle Optionen auf ihrem Standard. */
RoehreEinstellungen *roehre_einstellungen_standard(void);
/** Eine Voreinstellung, z. B. "scart-rgb". NULL bei unbekanntem Namen. */
RoehreEinstellungen *roehre_einstellungen_voreinstellung(const char *name);
/** Aus TOML oder JSON (JSON, wenn der Text mit '{' beginnt). Unbekannte Schlüssel und
 *  korrigierte Werte sind kein Fehler: sie stehen danach in roehre_letzter_fehler(). */
RoehreEinstellungen *roehre_einstellungen_aus_text(const char *text);
RoehreEinstellungen *roehre_einstellungen_kopieren(const RoehreEinstellungen *e);
void roehre_einstellungen_freigeben(RoehreEinstellungen *e);

/** Als Text zum Speichern: format 0 = TOML, 1 = JSON; nur_abweichungen != 0 schreibt nur, was
 *  vom Standard abweicht. Freigeben mit roehre_text_freigeben. */
char *roehre_einstellungen_als_text(const RoehreEinstellungen *e, int32_t format, int32_t nur_abweichungen);

/** Setzt eine Option. Wert als Text: Zahl ("0.5" oder "0,5"), Schalter ("an"/"aus"),
 *  Auswahl ("composite"). 0 = übernommen, 1 = übernommen und auf die Grenze korrigiert,
 *  < 0 = abgelehnt (unbekannte Option, falscher Wert). */
int32_t roehre_einstellungen_setzen(RoehreEinstellungen *e, const char *schluessel, const char *wert);

/** Wert einer Option als Text. Freigeben mit roehre_text_freigeben. */
char *roehre_einstellungen_wert(const RoehreEinstellungen *e, const char *schluessel);

/* ------------------------------------------------------------------ Modus */

/** Was gezeichnet wird. */
enum {
    ROEHRE_MODUS_FLACH = 0, /**< Röhrenbild im Fenster (mit Gehäuse, falls eingestellt) */
    ROEHRE_MODUS_RAUM = 1   /**< 3D-Vorführraum mit dem Fernseher (Optionen der Gruppe "raum") */
};

/* ------------------------------------------------------------------ Vorschaubild */

/**
 * Vorschau für Programme, die selbst keine Shader ausführen (z. B. ein Launcher, der in
 * Software zeichnet). roehre rechnet auf der Grafikkarte ohne Fenster und liefert ein
 * RGBA-Bild in den Arbeitsspeicher.
 *
 * Einmal anlegen (einige hundert ms), dann bei jeder Regleränderung neu zeichnen (wenige bis
 * etwa hundert ms; beim Wechsel der Shaderkette länger, deshalb Eingaben entprellen).
 */
typedef struct RoehreVorschau RoehreVorschau;

/** shader_ordner: der mitgelieferte Ordner shader/. NULL bei Fehler (z. B. keine Grafikkarte). */
RoehreVorschau *roehre_vorschau_erstellen(const char *shader_ordner);
void roehre_vorschau_freigeben(RoehreVorschau *v);

/**
 * Zeichnet ein Vorschaubild.
 *  bild_rgba:  Spielbild, sRGB, 4 Byte je Pixel, Zeile für Zeile ohne Füllung; NULL = eingebautes
 *              Testbild (dann sind bild_breite/bild_hoehe/pixel_seitenverhaeltnis egal).
 *  pixel_seitenverhaeltnis: Breite/Höhe eines Spielpixels (N64 320×240: 1.0; SNES: 8/7).
 *  modus:      ROEHRE_MODUS_FLACH oder ROEHRE_MODUS_RAUM.
 *  zeit:       Sekunden für Schwenk und Flackern; < 0 = Uhr seit dem Anlegen.
 *  ziel_rgba:  ziel_breite × ziel_hoehe × 4 Byte, sRGB, ohne Füllung.
 * Rückgabe 0 oder < 0.
 */
int32_t roehre_vorschau_zeichnen(RoehreVorschau *v, const RoehreEinstellungen *e, int32_t modus, float zeit,
                                 const uint8_t *bild_rgba, uint32_t bild_breite, uint32_t bild_hoehe,
                                 float pixel_seitenverhaeltnis,
                                 uint8_t *ziel_rgba, uint32_t ziel_breite, uint32_t ziel_hoehe);

#ifdef __cplusplus
}
#endif

#endif /* ROEHRE_H */
