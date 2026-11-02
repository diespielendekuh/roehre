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
 *   "voreinstellungen": [ { "schluessel": "scart-rgb", "name": {"de","en"}, "beschreibung": {"de","en"}, "lizenz": "gpl" | "permissiv" } ]
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

/* ------------------------------------------------------------------ Austausch mit Vulkan/D3D12 */

/**
 * Für Spiele, die selbst mit Vulkan oder D3D12 zeichnen (z. B. hinter RT64).
 *
 * roehre rechnet auf einem eigenen Gerät derselben Grafikkarte. Es legt zwei Bilder in
 * geteiltem Grafikspeicher an – EINGANG (das Spielbild) und ZIEL (das fertige Bild) – und gibt
 * dem Spiel Handles dafür. Das Spiel öffnet sie einmal auf seinem Gerät und tut dann je Bild:
 *
 *   1. sein Bild in EINGANG kopieren (vkCmdBlitImage/CopyResource, skaliert/konvertiert nach
 *      Belieben) und EINGANG in den Lesezustand bringen
 *      (Vulkan: SHADER_READ_ONLY_OPTIMAL; D3D12: PIXEL_SHADER_RESOURCE|NON_PIXEL_SHADER_RESOURCE),
 *   2. „bereit“ melden (Vulkan: Semaphore 0 signalisieren; D3D12: Zaun auf den Wert setzen,
 *      den roehre_austausch_zeichnen erwartet, siehe D3D12-Teil),
 *   3. roehre_austausch_zeichnen() aufrufen (kehrt sofort zurück),
 *   4. auf „fertig“ warten (auf der Grafikkarte) und ZIEL lesen oder kopieren; ZIEL danach
 *      wieder im Lesezustand lassen.
 *
 * Wer EINGANG beschreibt, setzt vorher eine Barriere ab ALL_COMMANDS (der Lesevorgang von
 * roehre aus dem letzten Bild liegt in der Warteschlange davor). Formate: EINGANG ist RGBA8
 * UNORM mit sRGB-kodierten Werten wie ein gewöhnliches Spielbild; ZIEL ist RGBA8 sRGB (SDR),
 * RGBA16F (scRGB) oder RGB10A2 (HDR10).
 */
typedef struct RoehreAustausch RoehreAustausch;

enum {
    ROEHRE_AUSGABE_SDR = 0,   /**< ZIEL in RGBA8 sRGB */
    ROEHRE_AUSGABE_SCRGB = 1, /**< ZIEL in RGBA16F, linear, 1.0 = 80 nit */
    ROEHRE_AUSGABE_HDR10 = 2  /**< ZIEL in RGB10A2, PQ, BT.2020 */
};

typedef struct RoehreAustauschParameter {
    uint32_t struktur_groesse;          /**< sizeof(RoehreAustauschParameter) */
    const char *shader_ordner;          /**< der mitgelieferte Ordner shader/ */
    uint8_t geraet_kennung[16];         /**< Vulkan: VkPhysicalDeviceIDProperties::deviceUUID;
                                             D3D12: LUID des Adapters in den ersten 8 Byte;
                                             alles 0 = erste Grafikkarte */
    uint32_t eingang_breite, eingang_hoehe; /**< Größe von EINGANG (z. B. 320×240 oder die Ausgabegröße von RT64) */
    float pixel_seitenverhaeltnis;      /**< Breite/Höhe eines Spielpixels in EINGANG */
    uint32_t ziel_breite, ziel_hoehe;   /**< Größe von ZIEL, meist die Fenstergröße */
    int32_t ausgabe;                    /**< ROEHRE_AUSGABE_* */
    const RoehreEinstellungen *einstellungen; /**< NULL = Standard */
} RoehreAustauschParameter;

/** Neue Einstellungen. Blockiert, bis eine neu nötige Shaderkette gebaut ist (beim Wechsel der
 *  Kette bis etwa eine Sekunde, sonst sofort). Kamera und Raum wirken sofort. */
int32_t roehre_austausch_einstellungen(RoehreAustausch *a, const RoehreEinstellungen *e);
/** Zeichnet ein Bild (modus: ROEHRE_MODUS_*; zeit in Sekunden, < 0 = Uhr seit dem Anlegen).
 *  Wartet auf der Grafikkarte auf „bereit“ und meldet „fertig“; kehrt sofort zurück. */
int32_t roehre_austausch_zeichnen(RoehreAustausch *a, int32_t modus, float zeit);
/** Für Bildschirme mit mehr Hertz als das Spiel (Optionen "bewegung.*", nur ROEHRE_MODUS_FLACH):
 *  je Bildschirmauffrischung einmal aufrufen; neues_spielbild != 0, wenn seit der letzten
 *  Auffrischung ein neues Spielbild in EINGANG liegt; teilbilder_je_spielbild = Hertz des
 *  Bildschirms / Bildrate des Spiels (z. B. 240/60 = 4). Austauschprotokoll wie beim Zeichnen. */
int32_t roehre_austausch_zeichnen_im_takt(RoehreAustausch *a, int32_t modus, float zeit,
                                          int32_t neues_spielbild, float teilbilder_je_spielbild);
/** Wartet auf der CPU, bis roehre auf der Grafikkarte nichts mehr zu tun hat (für Tests). */
int32_t roehre_austausch_warten(RoehreAustausch *a);
/** Freigeben. Andere Fenstergröße: freigeben und neu anlegen (Handles neu öffnen). */
void roehre_austausch_freigeben(RoehreAustausch *a);

/* --- Vulkan ---
 * Gerät des Spiels: Vulkan 1.1 oder neuer, mit den Geräteerweiterungen
 *   Linux:   VK_KHR_external_memory_fd    und VK_KHR_external_semaphore_fd
 *   Windows: VK_KHR_external_memory_win32 und VK_KHR_external_semaphore_win32
 * Bild öffnen: VkImage mit VkExternalMemoryImageCreateInfo{handle_art}, Format vk_format,
 * Nutzung vk_nutzung, 2D, 1 Mip, 1 Schicht, OPTIMAL, EXCLUSIVE; Speicher mit
 * VkImportMemoryFdInfoKHR bzw. VkImportMemoryWin32HandleInfoKHR, Größe speicher_groesse, Typ
 * speicher_typ, dazu VkMemoryDedicatedAllocateInfo (dediziert = 1).
 * Semaphoren: binär, dauerhaft importieren (vkImportSemaphoreFdKHR bzw. ...Win32HandleKHR).
 * Je Bild: „bereit“ (0) im Submit nach dem Beschreiben von EINGANG signalisieren, „fertig“ (1)
 * im Submit vor dem Lesen von ZIEL abwarten – jede genau einmal je roehre_austausch_zeichnen.
 *
 * Notweg ohne geteilte Semaphoren (Treiber ohne VK_KHR_external_semaphore_*, z. B. Mesa
 * lavapipe; roehre_vulkan_semaphore liefert dann < 0): vor roehre_austausch_zeichnen auf das
 * Ende des eigenen Submits warten (Zaun), vor dem Lesen von ZIEL roehre_austausch_warten()
 * aufrufen. Kostet die Gleichzeitigkeit; nur zum Prüfen gedacht.
 */
typedef struct RoehreVulkanBild {
    int64_t handle;            /**< Linux: fd; Windows: HANDLE. Neu je Aufruf, gehört dem Spiel. */
    uint32_t handle_art;       /**< VkExternalMemoryHandleTypeFlagBits */
    uint32_t vk_format;        /**< VkFormat */
    uint32_t vk_nutzung;       /**< VkImageUsageFlags */
    uint32_t breite, hoehe;
    uint64_t speicher_groesse; /**< VkMemoryAllocateInfo::allocationSize */
    uint32_t speicher_typ;     /**< VkMemoryAllocateInfo::memoryTypeIndex (gleiche Grafikkarte) */
    uint32_t dediziert;        /**< 1: VkMemoryDedicatedAllocateInfo verwenden */
} RoehreVulkanBild;

/** Legt den Austausch an (eigenes Gerät auf der Grafikkarte mit geraet_kennung = deviceUUID). */
RoehreAustausch *roehre_vulkan_erstellen(const RoehreAustauschParameter *p);
/** welches: 0 = EINGANG, 1 = ZIEL. Liefert jedes Mal ein neues Handle. */
int32_t roehre_vulkan_bild(const RoehreAustausch *a, int32_t welches, RoehreVulkanBild *aus);
/** welches: 0 = „bereit“ (Spiel signalisiert), 1 = „fertig“ (roehre signalisiert).
 *  handle_art: VkExternalSemaphoreHandleTypeFlagBits (darf NULL sein). */
int32_t roehre_vulkan_semaphore(const RoehreAustausch *a, int32_t welches, int64_t *handle, uint32_t *handle_art);

/* --- D3D12 (nur Windows) ---
 * Am Gerät des Spiels muss nichts geändert werden. roehre legt auf dem Adapter mit der LUID aus
 * geraet_kennung ein eigenes Gerät an, dazu zwei geteilte Ressourcen (D3D12_HEAP_FLAG_SHARED)
 * und einen geteilten Zaun (D3D12_FENCE_FLAG_SHARED).
 * Öffnen: ID3D12Device::OpenSharedHandle(handle, IID_PPV_ARGS(&ressource bzw. &zaun)), danach
 * CloseHandle(handle). Je Bild:
 *   roehre_d3d12_zaunwerte(a, &bereit, &fertig);
 *   ... Kopie nach EINGANG, EINGANG in PIXEL_SHADER_RESOURCE|NON_PIXEL_SHADER_RESOURCE ...
 *   ExecuteCommandLists(...); queue->Signal(zaun, bereit);
 *   roehre_austausch_zeichnen(a, modus, zeit);
 *   queue->Wait(zaun, fertig); ... ZIEL lesen/kopieren, danach wieder im Lesezustand lassen ...
 * Die Ressourcen stehen nach dem Anlegen im Zustand COMMON.
 */
typedef struct RoehreD3D12Bild {
    void *handle;          /**< NT-Handle (HANDLE), neu je Aufruf; nach dem Öffnen schließen */
    uint32_t dxgi_format;  /**< DXGI_FORMAT */
    uint32_t breite, hoehe;
} RoehreD3D12Bild;

/** Legt den Austausch an (geraet_kennung: LUID des Adapters in den ersten 8 Byte). */
RoehreAustausch *roehre_d3d12_erstellen(const RoehreAustauschParameter *p);
/** welches: 0 = EINGANG, 1 = ZIEL. */
int32_t roehre_d3d12_bild(const RoehreAustausch *a, int32_t welches, RoehreD3D12Bild *aus);
/** Der geteilte Zaun (neues NT-Handle). */
int32_t roehre_d3d12_zaun(const RoehreAustausch *a, void **handle);
/** Zaunwerte für den nächsten roehre_austausch_zeichnen (k-ter Aufruf: bereit 2k−1, fertig 2k). */
int32_t roehre_d3d12_zaunwerte(const RoehreAustausch *a, uint64_t *bereit, uint64_t *fertig);

#ifdef __cplusplus
}
#endif

#endif /* ROEHRE_H */
