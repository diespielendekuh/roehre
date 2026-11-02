/*
 * Beispiel und Prüflauf: ein „Spiel“ mit eigenem Vulkan-Gerät benutzt roehre über den
 * Austausch. Es öffnet EINGANG, ZIEL und die beiden Semaphoren auf seinem Gerät, kopiert je
 * Bild ein Testmuster nach EINGANG, lässt roehre rechnen und liest ZIEL zurück.
 * SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Bauen und starten (Linux, ohne Grafikkarte geht es mit Mesa lavapipe):
 *   cargo build --release -p roehre-c
 *   cc crates/roehre-c/beispiele/vulkan_spiel.c -Icrates/roehre-c/include -Ltarget/release -lroehre -lvulkan -o vulkan_spiel
 *   LD_LIBRARY_PATH=target/release ./vulkan_spiel shader ausgabe
 * Schreibt ausgabe_flach.ppm und ausgabe_raum.ppm. Rückgabe 0, wenn das Bild plausibel ist.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <vulkan/vulkan.h>
#include "roehre.h"

#define PRUEFE(x) do { VkResult r_ = (x); if (r_ != VK_SUCCESS) { fprintf(stderr, "%s: %d (Zeile %d)\n", #x, r_, __LINE__); exit(1); } } while (0)
#define ROEHRE(x) do { if ((x) < 0) { fprintf(stderr, "%s: %s\n", #x, roehre_letzter_fehler()); exit(1); } } while (0)

static VkDevice geraet;
static VkPhysicalDevice karte;

static uint32_t speichertyp(uint32_t bits, VkMemoryPropertyFlags wunsch) {
    VkPhysicalDeviceMemoryProperties p;
    vkGetPhysicalDeviceMemoryProperties(karte, &p);
    for (uint32_t i = 0; i < p.memoryTypeCount; i++)
        if ((bits & (1u << i)) && (p.memoryTypes[i].propertyFlags & wunsch) == wunsch) return i;
    fprintf(stderr, "kein Speichertyp\n");
    exit(1);
}

static void puffer(VkDeviceSize groesse, VkBufferUsageFlags nutzung, VkBuffer *b, VkDeviceMemory *m, void **abbild) {
    VkBufferCreateInfo bi = { VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO, NULL, 0, groesse, nutzung, VK_SHARING_MODE_EXCLUSIVE, 0, NULL };
    PRUEFE(vkCreateBuffer(geraet, &bi, NULL, b));
    VkMemoryRequirements req;
    vkGetBufferMemoryRequirements(geraet, *b, &req);
    VkMemoryAllocateInfo ai = { VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO, NULL, req.size,
                                speichertyp(req.memoryTypeBits, VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT) };
    PRUEFE(vkAllocateMemory(geraet, &ai, NULL, m));
    PRUEFE(vkBindBufferMemory(geraet, *b, *m, 0));
    PRUEFE(vkMapMemory(geraet, *m, 0, groesse, 0, abbild));
}

/* Öffnet ein geteiltes Bild von roehre auf dem eigenen Gerät. */
static VkImage bild_oeffnen(const RoehreVulkanBild *g) {
    VkExternalMemoryImageCreateInfo ext = { VK_STRUCTURE_TYPE_EXTERNAL_MEMORY_IMAGE_CREATE_INFO, NULL, g->handle_art };
    VkImageCreateInfo ii = { VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO, &ext, 0, VK_IMAGE_TYPE_2D, (VkFormat)g->vk_format,
                             { g->breite, g->hoehe, 1 }, 1, 1, VK_SAMPLE_COUNT_1_BIT, VK_IMAGE_TILING_OPTIMAL,
                             g->vk_nutzung, VK_SHARING_MODE_EXCLUSIVE, 0, NULL, VK_IMAGE_LAYOUT_UNDEFINED };
    VkImage bild;
    PRUEFE(vkCreateImage(geraet, &ii, NULL, &bild));
    VkMemoryDedicatedAllocateInfo ded = { VK_STRUCTURE_TYPE_MEMORY_DEDICATED_ALLOCATE_INFO, NULL, bild, VK_NULL_HANDLE };
    VkImportMemoryFdInfoKHR imp = { VK_STRUCTURE_TYPE_IMPORT_MEMORY_FD_INFO_KHR, g->dediziert ? &ded : NULL,
                                    (VkExternalMemoryHandleTypeFlagBits)g->handle_art, (int)g->handle };
    VkMemoryAllocateInfo ai = { VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO, &imp, g->speicher_groesse, g->speicher_typ };
    VkDeviceMemory m;
    PRUEFE(vkAllocateMemory(geraet, &ai, NULL, &m));
    PRUEFE(vkBindImageMemory(geraet, bild, m, 0));
    return bild;
}

static VkSemaphore semaphore_oeffnen(RoehreAustausch *a, int welches) {
    int64_t h; uint32_t art;
    if (roehre_vulkan_semaphore(a, welches, &h, &art) < 0) {
        printf("ohne geteilte Semaphoren (%s)\n", roehre_letzter_fehler());
        return VK_NULL_HANDLE;
    }
    VkSemaphoreCreateInfo si = { VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO, NULL, 0 };
    VkSemaphore s;
    PRUEFE(vkCreateSemaphore(geraet, &si, NULL, &s));
    PFN_vkImportSemaphoreFdKHR imp = (PFN_vkImportSemaphoreFdKHR)vkGetDeviceProcAddr(geraet, "vkImportSemaphoreFdKHR");
    VkImportSemaphoreFdInfoKHR ii = { VK_STRUCTURE_TYPE_IMPORT_SEMAPHORE_FD_INFO_KHR, NULL, s, 0, (VkExternalSemaphoreHandleTypeFlagBits)art, (int)h };
    PRUEFE(imp(geraet, &ii));
    return s;
}

static void barriere(VkCommandBuffer cb, VkImage bild, VkImageLayout von, VkImageLayout nach) {
    VkImageMemoryBarrier b = { VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER, NULL, VK_ACCESS_MEMORY_WRITE_BIT,
                               VK_ACCESS_MEMORY_READ_BIT | VK_ACCESS_MEMORY_WRITE_BIT, von, nach,
                               VK_QUEUE_FAMILY_IGNORED, VK_QUEUE_FAMILY_IGNORED, bild, { VK_IMAGE_ASPECT_COLOR_BIT, 0, 1, 0, 1 } };
    vkCmdPipelineBarrier(cb, VK_PIPELINE_STAGE_ALL_COMMANDS_BIT, VK_PIPELINE_STAGE_ALL_COMMANDS_BIT, 0, 0, NULL, 0, NULL, 1, &b);
}

int main(int argc, char **argv) {
    const char *shader = argc > 1 ? argv[1] : "shader";
    const char *ausgabe = argc > 2 ? argv[2] : "vulkan";
    const uint32_t EB = 320, EH = 240, ZB = 960, ZH = 720;

    /* --- das Gerät des Spiels --- */
    VkApplicationInfo app = { VK_STRUCTURE_TYPE_APPLICATION_INFO, NULL, "vulkan_spiel", 1, NULL, 0, VK_API_VERSION_1_2 };
    VkInstanceCreateInfo ici = { VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO, NULL, 0, &app, 0, NULL, 0, NULL };
    VkInstance instanz;
    PRUEFE(vkCreateInstance(&ici, NULL, &instanz));
    uint32_t n = 1;
    vkEnumeratePhysicalDevices(instanz, &n, &karte);
    if (n == 0) { fprintf(stderr, "keine Vulkan-Grafikkarte\n"); return 1; }
    VkPhysicalDeviceIDProperties id = { VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_ID_PROPERTIES };
    VkPhysicalDeviceProperties2 p2 = { VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2, &id };
    vkGetPhysicalDeviceProperties2(karte, &p2);
    printf("Grafikkarte des Spiels: %s\n", p2.properties.deviceName);
    uint32_t familie = 0;
    float prio = 1.0f;
    VkDeviceQueueCreateInfo qi = { VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO, NULL, 0, familie, 1, &prio };
    /* Geteilte Semaphoren nur, wenn der Treiber sie anbietet (lavapipe nicht) */
    const char *erw[2] = { "VK_KHR_external_memory_fd", "VK_KHR_external_semaphore_fd" };
    uint32_t anzahl_erw = 1, m = 0;
    vkEnumerateDeviceExtensionProperties(karte, NULL, &m, NULL);
    VkExtensionProperties *vorhanden = malloc(m * sizeof *vorhanden);
    vkEnumerateDeviceExtensionProperties(karte, NULL, &m, vorhanden);
    for (uint32_t i = 0; i < m; i++)
        if (!strcmp(vorhanden[i].extensionName, erw[1])) anzahl_erw = 2;
    free(vorhanden);
    VkDeviceCreateInfo dci = { VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO, NULL, 0, 1, &qi, 0, NULL, anzahl_erw, erw, NULL };
    PRUEFE(vkCreateDevice(karte, &dci, NULL, &geraet));
    VkQueue schlange;
    vkGetDeviceQueue(geraet, familie, 0, &schlange);
    VkCommandPoolCreateInfo pci = { VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO, NULL, VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT, familie };
    VkCommandPool pool;
    PRUEFE(vkCreateCommandPool(geraet, &pci, NULL, &pool));
    VkCommandBuffer cb[2];
    VkCommandBufferAllocateInfo cai = { VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO, NULL, pool, VK_COMMAND_BUFFER_LEVEL_PRIMARY, 2 };
    PRUEFE(vkAllocateCommandBuffers(geraet, &cai, cb));
    VkFenceCreateInfo fci = { VK_STRUCTURE_TYPE_FENCE_CREATE_INFO, NULL, 0 };
    VkFence zaun;
    PRUEFE(vkCreateFence(geraet, &fci, NULL, &zaun));

    /* --- roehre --- */
    RoehreEinstellungen *e = roehre_einstellungen_voreinstellung("wohnzimmer-90er-composite");
    RoehreAustauschParameter par;
    memset(&par, 0, sizeof par);
    par.struktur_groesse = sizeof par;
    par.shader_ordner = shader;
    memcpy(par.geraet_kennung, id.deviceUUID, 16);
    par.eingang_breite = EB; par.eingang_hoehe = EH; par.pixel_seitenverhaeltnis = 1.0f;
    par.ziel_breite = ZB; par.ziel_hoehe = ZH;
    par.ausgabe = ROEHRE_AUSGABE_SDR;
    par.einstellungen = e;
    RoehreAustausch *a = roehre_vulkan_erstellen(&par);
    if (!a) { fprintf(stderr, "roehre_vulkan_erstellen: %s\n", roehre_letzter_fehler()); return 1; }

    RoehreVulkanBild ge, gz;
    ROEHRE(roehre_vulkan_bild(a, 0, &ge));
    ROEHRE(roehre_vulkan_bild(a, 1, &gz));
    VkImage eingang = bild_oeffnen(&ge), ziel = bild_oeffnen(&gz);
    VkSemaphore bereit = VK_NULL_HANDLE, fertig = VK_NULL_HANDLE;
    if (anzahl_erw == 2) { bereit = semaphore_oeffnen(a, 0); fertig = semaphore_oeffnen(a, 1); }
    int notweg = (bereit == VK_NULL_HANDLE || fertig == VK_NULL_HANDLE);
    if (notweg) printf("Notweg: Ordnung über die CPU\n");
    printf("EINGANG %ux%u Format %u, ZIEL %ux%u Format %u geöffnet\n", ge.breite, ge.hoehe, ge.vk_format, gz.breite, gz.hoehe, gz.vk_format);

    /* Testmuster: Farbbalken oben, Schachbrett unten, weißer Rahmen */
    VkBuffer hoch, runter; VkDeviceMemory mh, mr; uint8_t *ph, *pr;
    puffer(EB * EH * 4, VK_BUFFER_USAGE_TRANSFER_SRC_BIT, &hoch, &mh, (void **)&ph);
    puffer((VkDeviceSize)ZB * ZH * 4, VK_BUFFER_USAGE_TRANSFER_DST_BIT, &runter, &mr, (void **)&pr);
    static const uint8_t balken[8][3] = { {255,255,255},{255,255,0},{0,255,255},{0,255,0},{255,0,255},{255,0,0},{0,0,255},{0,0,0} };
    for (uint32_t y = 0; y < EH; y++)
        for (uint32_t x = 0; x < EB; x++) {
            uint8_t *p = ph + 4 * (y * EB + x);
            if (x < 2 || y < 2 || x >= EB - 2 || y >= EH - 2) { p[0] = p[1] = p[2] = 255; }
            else if (y < EH / 2) { const uint8_t *c = balken[x * 8 / EB]; p[0] = c[0]; p[1] = c[1]; p[2] = c[2]; }
            else { uint8_t v = ((x / 16 + y / 16) & 1) ? 200 : 40; p[0] = p[1] = p[2] = v; }
            p[3] = 255;
        }

    const char *namen[2] = { "flach", "raum" };
    for (int modus = 0; modus <= 1; modus++) {
        for (int bild = 0; bild < 4; bild++) {
            int erstes = (modus == 0 && bild == 0);
            /* 1. Spielbild nach EINGANG, danach Lesezustand; „bereit“ melden */
            VkCommandBufferBeginInfo bi = { VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO, NULL, VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT, NULL };
            PRUEFE(vkBeginCommandBuffer(cb[0], &bi));
            barriere(cb[0], eingang, erstes ? VK_IMAGE_LAYOUT_UNDEFINED : VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL, VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL);
            VkBufferImageCopy k = { 0, 0, 0, { VK_IMAGE_ASPECT_COLOR_BIT, 0, 0, 1 }, { 0, 0, 0 }, { EB, EH, 1 } };
            vkCmdCopyBufferToImage(cb[0], hoch, eingang, VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, 1, &k);
            barriere(cb[0], eingang, VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL);
            PRUEFE(vkEndCommandBuffer(cb[0]));
            VkSubmitInfo s1 = { VK_STRUCTURE_TYPE_SUBMIT_INFO, NULL, 0, NULL, NULL, 1, &cb[0], notweg ? 0 : 1, &bereit };
            PRUEFE(vkQueueSubmit(schlange, 1, &s1, notweg ? zaun : VK_NULL_HANDLE));
            if (notweg) { PRUEFE(vkWaitForFences(geraet, 1, &zaun, VK_TRUE, UINT64_MAX)); PRUEFE(vkResetFences(geraet, 1, &zaun)); }

            /* 2. roehre rechnen lassen */
            ROEHRE(roehre_austausch_zeichnen(a, modus, 3.0f));
            if (notweg) ROEHRE(roehre_austausch_warten(a));

            /* 3. auf „fertig“ warten, ZIEL lesen, Lesezustand wiederherstellen */
            PRUEFE(vkBeginCommandBuffer(cb[1], &bi));
            barriere(cb[1], ziel, VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL, VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL);
            VkBufferImageCopy kz = { 0, 0, 0, { VK_IMAGE_ASPECT_COLOR_BIT, 0, 0, 1 }, { 0, 0, 0 }, { ZB, ZH, 1 } };
            vkCmdCopyImageToBuffer(cb[1], ziel, VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL, runter, 1, &kz);
            barriere(cb[1], ziel, VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL, VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL);
            PRUEFE(vkEndCommandBuffer(cb[1]));
            VkPipelineStageFlags stufe = VK_PIPELINE_STAGE_ALL_COMMANDS_BIT;
            VkSubmitInfo s2 = { VK_STRUCTURE_TYPE_SUBMIT_INFO, NULL, notweg ? 0 : 1, &fertig, &stufe, 1, &cb[1], 0, NULL };
            PRUEFE(vkQueueSubmit(schlange, 1, &s2, zaun));
            PRUEFE(vkWaitForFences(geraet, 1, &zaun, VK_TRUE, UINT64_MAX));
            PRUEFE(vkResetFences(geraet, 1, &zaun));
        }
        /* Plausibel? Die Bildmitte ist nicht schwarz, die Ecken im Flachmodus dunkel */
        uint32_t mitte = 4 * ((ZH / 2) * ZB + ZB / 2), summe = pr[mitte] + pr[mitte + 1] + pr[mitte + 2];
        char pfad[512];
        snprintf(pfad, sizeof pfad, "%s_%s.ppm", ausgabe, namen[modus]);
        FILE *f = fopen(pfad, "wb");
        fprintf(f, "P6\n%u %u\n255\n", ZB, ZH);
        for (uint32_t i = 0; i < ZB * ZH; i++) fwrite(pr + 4 * i, 1, 3, f);
        fclose(f);
        printf("%s geschrieben, Helligkeit Bildmitte %u\n", pfad, summe);
        if (summe < 30) { fprintf(stderr, "Bildmitte schwarz: Austausch hat nichts geliefert\n"); return 2; }
    }
    roehre_austausch_freigeben(a);
    roehre_einstellungen_freigeben(e);
    vkDeviceWaitIdle(geraet);
    printf("Austausch mit Vulkan in Ordnung\n");
    return 0;
}
