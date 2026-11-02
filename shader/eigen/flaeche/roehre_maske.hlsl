// roehre – Flächen-Maske für 3D-Szenen (HLSL, Fassung der WGSL-Datei roehre_maske.wgsl)
// SPDX-License-Identifier: MIT OR Apache-2.0
// Erklärung und Parameter: siehe roehre_maske.wgsl. Ergebnis mit der linearen Farbe des
// Röhrenbilds multiplizieren; der Mittelwert über eine Periode ist 1.

struct RoehreMaske
{
    float triaden;
    float staerke;
    float schlitz;
    float seitenverhaeltnis;
    float schlitz_abstand;
    float bgr;
};

static const float ROEHRE_TAU = 6.283185307179586;

float roehre_bandgrenze(float perioden_je_pixel)
{
    return 1.0 - smoothstep(0.2, 0.45, perioden_je_pixel);
}

float3 roehre_maske(float2 uv, RoehreMaske m)
{
    float t = uv.x * m.triaden;
    float v = uv.y * m.triaden / (m.seitenverhaeltnis * max(m.schlitz_abstand, 0.1));
    float wt = length(float2(ddx(t), ddy(t)));
    float wv = length(float2(ddx(v), ddy(v)));

    float a1 = m.staerke * 0.8 * roehre_bandgrenze(wt);
    float a2 = m.staerke * 0.3 * roehre_bandgrenze(2.0 * wt);
    float3 phase = float3(0.0, 1.0 / 3.0, 2.0 / 3.0);
    if (m.bgr > 0.5) phase = phase.zyx;
    float3 x = ROEHRE_TAU * (float3(t, t, t) - phase);
    float3 streifen = max(1.0 + a1 * cos(x) + a2 * cos(2.0 * x), 0.0);

    float s = m.schlitz * roehre_bandgrenze(wv) * roehre_bandgrenze(0.5 * wt);
    float schlitz = 1.0 + s * 0.5 * cos(ROEHRE_TAU * v) * cos(0.5 * ROEHRE_TAU * t);

    return streifen * schlitz;
}
