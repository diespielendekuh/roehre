/*
 * Beispiel und Prüflauf für Windows: ein „Spiel“ mit eigenem D3D12-Gerät benutzt roehre über
 * den Austausch (geteilte Ressourcen, geteilter Zaun) und liest das Ergebnis zurück.
 * SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Bauen (MSVC):   cl /EHsc /I crates\roehre-c\include crates\roehre-c\beispiele\d3d12_spiel.cpp
 *                    target\release\roehre.dll.lib d3d12.lib dxgi.lib
 * Bauen (MinGW):  x86_64-w64-mingw32-g++ -O2 -Icrates/roehre-c/include crates/roehre-c/beispiele/d3d12_spiel.cpp
 *                    -Ltarget/x86_64-pc-windows-gnu/release -lroehre -ld3d12 -ldxgi -o d3d12_spiel.exe
 * Starten:        d3d12_spiel.exe shader ausgabe   (roehre.dll und shader\ daneben)
 * Schreibt ausgabe_flach.ppm und ausgabe_raum.ppm.
 */
#include <windows.h>
#include <d3d12.h>
#include <dxgi1_4.h>
#include <cstdio>
#include <cstring>
#include <vector>
#include "roehre.h"

#define PRUEFE(x) do { HRESULT h_ = (x); if (FAILED(h_)) { std::fprintf(stderr, "%s: 0x%08lx (Zeile %d)\n", #x, (unsigned long)h_, __LINE__); std::exit(1); } } while (0)
#define ROEHRE(x) do { if ((x) < 0) { std::fprintf(stderr, "%s: %s\n", #x, roehre_letzter_fehler()); std::exit(1); } } while (0)

static ID3D12Device *geraet;

static ID3D12Resource *puffer(UINT64 groesse, D3D12_HEAP_TYPE art) {
    D3D12_HEAP_PROPERTIES heap = {}; heap.Type = art;
    D3D12_RESOURCE_DESC d = {};
    d.Dimension = D3D12_RESOURCE_DIMENSION_BUFFER; d.Width = groesse; d.Height = 1; d.DepthOrArraySize = 1;
    d.MipLevels = 1; d.SampleDesc.Count = 1; d.Layout = D3D12_TEXTURE_LAYOUT_ROW_MAJOR;
    ID3D12Resource *r = nullptr;
    PRUEFE(geraet->CreateCommittedResource(&heap, D3D12_HEAP_FLAG_NONE, &d,
        art == D3D12_HEAP_TYPE_UPLOAD ? D3D12_RESOURCE_STATE_GENERIC_READ : D3D12_RESOURCE_STATE_COPY_DEST, nullptr, IID_PPV_ARGS(&r)));
    return r;
}

static void zustand(ID3D12GraphicsCommandList *l, ID3D12Resource *r, D3D12_RESOURCE_STATES von, D3D12_RESOURCE_STATES nach) {
    D3D12_RESOURCE_BARRIER b = {};
    b.Type = D3D12_RESOURCE_BARRIER_TYPE_TRANSITION;
    b.Transition.pResource = r; b.Transition.Subresource = D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES;
    b.Transition.StateBefore = von; b.Transition.StateAfter = nach;
    l->ResourceBarrier(1, &b);
}

int main(int argc, char **argv) {
    const char *shader = argc > 1 ? argv[1] : "shader";
    const char *ausgabe = argc > 2 ? argv[2] : "d3d12";
    const UINT EB = 320, EH = 240, ZB = 960, ZH = 720;
    const D3D12_RESOURCE_STATES LESEN = D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE | D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE;

    /* --- das Gerät des Spiels --- */
    IDXGIFactory4 *fabrik; PRUEFE(CreateDXGIFactory1(IID_PPV_ARGS(&fabrik)));
    IDXGIAdapter1 *adapter; PRUEFE(fabrik->EnumAdapters1(0, &adapter));
    DXGI_ADAPTER_DESC1 ad; adapter->GetDesc1(&ad);
    PRUEFE(D3D12CreateDevice(adapter, D3D_FEATURE_LEVEL_11_0, IID_PPV_ARGS(&geraet)));
    D3D12_COMMAND_QUEUE_DESC qd = {}; qd.Type = D3D12_COMMAND_LIST_TYPE_DIRECT;
    ID3D12CommandQueue *schlange; PRUEFE(geraet->CreateCommandQueue(&qd, IID_PPV_ARGS(&schlange)));
    ID3D12CommandAllocator *zuteiler; PRUEFE(geraet->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT, IID_PPV_ARGS(&zuteiler)));
    ID3D12GraphicsCommandList *liste; PRUEFE(geraet->CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, zuteiler, nullptr, IID_PPV_ARGS(&liste)));
    liste->Close();
    ID3D12Fence *cpu_zaun; PRUEFE(geraet->CreateFence(0, D3D12_FENCE_FLAG_NONE, IID_PPV_ARGS(&cpu_zaun)));
    UINT64 cpu_wert = 0;
    HANDLE ereignis = CreateEventW(nullptr, FALSE, FALSE, nullptr);

    /* --- roehre --- */
    RoehreEinstellungen *e = roehre_einstellungen_voreinstellung("wohnzimmer-90er-composite");
    RoehreAustauschParameter par; std::memset(&par, 0, sizeof par);
    par.struktur_groesse = sizeof par;
    par.shader_ordner = shader;
    std::memcpy(par.geraet_kennung, &ad.AdapterLuid.LowPart, 4);
    std::memcpy(par.geraet_kennung + 4, &ad.AdapterLuid.HighPart, 4);
    par.eingang_breite = EB; par.eingang_hoehe = EH; par.pixel_seitenverhaeltnis = 1.0f;
    par.ziel_breite = ZB; par.ziel_hoehe = ZH; par.ausgabe = ROEHRE_AUSGABE_SDR; par.einstellungen = e;
    RoehreAustausch *a = roehre_d3d12_erstellen(&par);
    if (!a) { std::fprintf(stderr, "roehre_d3d12_erstellen: %s\n", roehre_letzter_fehler()); return 1; }

    ID3D12Resource *eingang, *ziel; ID3D12Fence *zaun;
    RoehreD3D12Bild b;
    ROEHRE(roehre_d3d12_bild(a, 0, &b)); PRUEFE(geraet->OpenSharedHandle(b.handle, IID_PPV_ARGS(&eingang))); CloseHandle(b.handle);
    ROEHRE(roehre_d3d12_bild(a, 1, &b)); PRUEFE(geraet->OpenSharedHandle(b.handle, IID_PPV_ARGS(&ziel))); CloseHandle(b.handle);
    void *zh; ROEHRE(roehre_d3d12_zaun(a, &zh)); PRUEFE(geraet->OpenSharedHandle(zh, IID_PPV_ARGS(&zaun))); CloseHandle(zh);

    /* Testmuster hochladen, Rückleseplatz */
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT fe, fz; UINT64 ge, gz;
    D3D12_RESOURCE_DESC de = eingang->GetDesc(), dz = ziel->GetDesc();
    geraet->GetCopyableFootprints(&de, 0, 1, 0, &fe, nullptr, nullptr, &ge);
    geraet->GetCopyableFootprints(&dz, 0, 1, 0, &fz, nullptr, nullptr, &gz);
    ID3D12Resource *hoch = puffer(ge, D3D12_HEAP_TYPE_UPLOAD), *runter = puffer(gz, D3D12_HEAP_TYPE_READBACK);
    uint8_t *ph; PRUEFE(hoch->Map(0, nullptr, (void **)&ph));
    for (UINT y = 0; y < EH; y++)
        for (UINT x = 0; x < EB; x++) {
            uint8_t *p = ph + y * fe.Footprint.RowPitch + 4 * x;
            uint8_t v = (y < EH / 2) ? (uint8_t)(x * 255 / EB) : (((x / 16 + y / 16) & 1) ? 200 : 40);
            p[0] = v; p[1] = (y < EH / 2) ? 255 - v : v; p[2] = v; p[3] = 255;
        }
    hoch->Unmap(0, nullptr);

    const char *namen[2] = { "flach", "raum" };
    D3D12_RESOURCE_STATES eingang_zustand = D3D12_RESOURCE_STATE_COMMON, ziel_zustand = D3D12_RESOURCE_STATE_COMMON;
    for (int modus = 0; modus <= 1; modus++) {
        for (int bild = 0; bild < 4; bild++) {
            UINT64 bereit, fertig;
            ROEHRE(roehre_d3d12_zaunwerte(a, &bereit, &fertig));
            PRUEFE(zuteiler->Reset()); PRUEFE(liste->Reset(zuteiler, nullptr));
            zustand(liste, eingang, eingang_zustand, D3D12_RESOURCE_STATE_COPY_DEST);
            D3D12_TEXTURE_COPY_LOCATION qu = {}, zi = {};
            qu.pResource = hoch; qu.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT; qu.PlacedFootprint = fe;
            zi.pResource = eingang; zi.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
            liste->CopyTextureRegion(&zi, 0, 0, 0, &qu, nullptr);
            zustand(liste, eingang, D3D12_RESOURCE_STATE_COPY_DEST, LESEN);
            eingang_zustand = LESEN;
            PRUEFE(liste->Close());
            ID3D12CommandList *l1[] = { liste };
            schlange->ExecuteCommandLists(1, l1);
            PRUEFE(schlange->Signal(zaun, bereit));

            ROEHRE(roehre_austausch_zeichnen(a, modus, 3.0f));

            PRUEFE(schlange->Wait(zaun, fertig));
            /* Nach dem ersten Bild steht ZIEL im Lesezustand (roehre hat es dorthin gebracht) */
            ziel_zustand = LESEN;
            PRUEFE(zuteiler->Reset()); PRUEFE(liste->Reset(zuteiler, nullptr));
            zustand(liste, ziel, ziel_zustand, D3D12_RESOURCE_STATE_COPY_SOURCE);
            D3D12_TEXTURE_COPY_LOCATION q2 = {}, z2 = {};
            q2.pResource = ziel; q2.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
            z2.pResource = runter; z2.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT; z2.PlacedFootprint = fz;
            liste->CopyTextureRegion(&z2, 0, 0, 0, &q2, nullptr);
            zustand(liste, ziel, D3D12_RESOURCE_STATE_COPY_SOURCE, LESEN);
            PRUEFE(liste->Close());
            ID3D12CommandList *l2[] = { liste };
            schlange->ExecuteCommandLists(1, l2);
            PRUEFE(schlange->Signal(cpu_zaun, ++cpu_wert));
            PRUEFE(cpu_zaun->SetEventOnCompletion(cpu_wert, ereignis));
            WaitForSingleObject(ereignis, INFINITE);
        }
        uint8_t *pr; PRUEFE(runter->Map(0, nullptr, (void **)&pr));
        char pfad[512]; std::snprintf(pfad, sizeof pfad, "%s_%s.ppm", ausgabe, namen[modus]);
        FILE *f = std::fopen(pfad, "wb");
        std::fprintf(f, "P6\n%u %u\n255\n", ZB, ZH);
        for (UINT y = 0; y < ZH; y++)
            for (UINT x = 0; x < ZB; x++) std::fwrite(pr + y * fz.Footprint.RowPitch + 4 * x, 1, 3, f);
        std::fclose(f);
        const uint8_t *m = pr + (ZH / 2) * fz.Footprint.RowPitch + 4 * (ZB / 2);
        unsigned summe = m[0] + m[1] + m[2];
        runter->Unmap(0, nullptr);
        std::printf("%s geschrieben, Helligkeit Bildmitte %u\n", pfad, summe);
        if (summe < 30) { std::fprintf(stderr, "Bildmitte schwarz: Austausch hat nichts geliefert\n"); return 2; }
    }
    roehre_austausch_freigeben(a);
    roehre_einstellungen_freigeben(e);
    std::printf("Austausch mit D3D12 in Ordnung\n");
    return 0;
}
