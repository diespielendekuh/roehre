#version 450
// roehre – eigener Pass „virtuelle Zeilen“: bringt ein hochaufgelöstes Bild (z. B. 1080p
// oder 4K eines 3D-Spiels) auf die Zeilenzahl, die die Röhre zeichnen soll.
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Verfahren: flächentreues Verkleinern (Box-Filter mit Bruchteilgewichten). Jeder
// Ausgabepunkt mittelt genau die Quellfläche, die er überdeckt. Das ist für Zeilen
// physikalisch passend: Eine Röhrenzeile sammelt das Licht ihres ganzen Streifens.
// Gemittelt wird in linearem Licht (sRGB dekodiert), sonst würden feine helle Details
// beim Verkleinern zu dunkel.
//
// Die Größe der Ausgabe bestimmt roehre (Viewport = Zieltextur), nicht das Preset.

layout(push_constant) uniform Push
{
	vec4 SourceSize;
	vec4 OutputSize;
} params;

layout(std140, set = 0, binding = 0) uniform UBO
{
	mat4 MVP;
} global;

#pragma stage vertex
layout(location = 0) in vec4 Position;
layout(location = 1) in vec2 TexCoord;
layout(location = 0) out vec2 vTexCoord;

void main()
{
   gl_Position = global.MVP * Position;
   vTexCoord = TexCoord;
}

#pragma stage fragment
layout(location = 0) in vec2 vTexCoord;
layout(location = 0) out vec4 FragColor;
layout(set = 0, binding = 2) uniform sampler2D Source;

vec3 zu_linear(vec3 c)
{
   return mix(pow((c + 0.055) / 1.055, vec3(2.4)), c / 12.92, lessThanEqual(c, vec3(0.04045)));
}

vec3 zu_srgb(vec3 c)
{
   return mix(1.055 * pow(c, vec3(1.0 / 2.4)) - 0.055, c * 12.92, lessThanEqual(c, vec3(0.0031308)));
}

// Höchstens so viele Quellpunkte je Achse (reicht für 8K → 240 Zeilen)
#define MAX_SCHRITTE 40

void main()
{
   vec2 faktor = params.SourceSize.xy * params.OutputSize.zw;   // Quellpunkte je Ausgabepunkt
   vec2 ausgabe = floor(vTexCoord * params.OutputSize.xy);       // Index des Ausgabepunkts
   vec2 a = ausgabe * faktor;                                    // Fußabdruck in Quellpunkten
   vec2 b = a + faktor;
   ivec2 i0 = ivec2(floor(a));
   ivec2 i1 = ivec2(ceil(b)) - 1;
   ivec2 maxi = ivec2(params.SourceSize.xy) - 1;

   vec3 summe = vec3(0.0);
   float gewichte = 0.0;
   for (int dy = 0; dy < MAX_SCHRITTE; dy++)
   {
      int y = i0.y + dy;
      if (y > i1.y) break;
      float wy = min(float(y + 1), b.y) - max(float(y), a.y);
      for (int dx = 0; dx < MAX_SCHRITTE; dx++)
      {
         int x = i0.x + dx;
         if (x > i1.x) break;
         float wx = min(float(x + 1), b.x) - max(float(x), a.x);
         vec3 c = texelFetch(Source, clamp(ivec2(x, y), ivec2(0), maxi), 0).rgb;
         summe += zu_linear(c) * (wx * wy);
         gewichte += wx * wy;
      }
   }
   FragColor = vec4(zu_srgb(summe / max(gewichte, 1e-6)), 1.0);
}
