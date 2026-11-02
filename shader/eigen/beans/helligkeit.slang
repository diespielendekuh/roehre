#version 450
// roehre – eigener Pass hinter crt-beans: Helligkeitsausgleich in linearem Licht.
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// crt-beans (MIT) hat keinen Regler für die Gesamthelligkeit; Maske und Zeilenlücken
// verdunkeln das Bild gegenüber dem sauberen Bild (gemessen, roehre-pruefstand helligkeit:
// 75-%-Weiß 86 %, 40-%-Grau 76 %). Dieser Pass hebt es an, wie brightboost bei guest.
// Er liest das Bild Punkt für Punkt (gleiche Größe), dekodiert mit der Ausgabekurve von
// crt-beans, verstärkt, begrenzt auf 1 und kodiert wieder.

layout(push_constant) uniform Push
{
	vec4 SourceSize;
	vec4 OutputSize;
	float roehre_verstaerkung;
	float roehre_kurve;
} params;

#pragma parameter roehre_verstaerkung "roehre: Verstärkung (linear)" 1.0 0.5 2.0 0.01
#pragma parameter roehre_kurve "roehre: Kurve (0 sRGB, 1 Gamma 2,2)" 1.0 0.0 1.0 1.0

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

vec3 dekodieren(vec3 c) {
   if (params.roehre_kurve > 0.5) return pow(c, vec3(2.2));
   return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(vec3(0.04045), c));
}

vec3 kodieren(vec3 c) {
   if (params.roehre_kurve > 0.5) return pow(c, vec3(1.0 / 2.2));
   return mix(c * 12.92, 1.055 * pow(c, vec3(1.0 / 2.4)) - 0.055, step(vec3(0.0031308), c));
}

void main()
{
   vec3 c = texelFetch(Source, ivec2(vTexCoord * params.SourceSize.xy), 0).rgb;
   vec3 l = clamp(dekodieren(c) * params.roehre_verstaerkung, 0.0, 1.0);
   FragColor = vec4(kodieren(l), 1.0);
}
