#version 450
// roehre – eigener Pass „scharf-bilinear“ für die Voreinstellung „Sauber/aus“.
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Vergrößert pixelgenau ohne Unschärfe, auch bei nicht ganzzahligen Faktoren: Innerhalb
// eines Quellpixels wird nicht gefiltert, nur auf dem schmalen Saum zwischen zwei Pixeln
// (Breite ein Ausgabepixel) wird linear überblendet. So bleiben Pixelkanten scharf und
// gleich breit, ohne die ungleichmäßigen Breiten der reinen Pixelwiederholung.
// Erwartet einen linear filternden Sampler (filter_linear = true im Preset).

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

void main()
{
   vec2 texel = vTexCoord * params.SourceSize.xy;
   // Ausgabepixel je Quellpixel, getrennt nach Achsen
   vec2 faktor = max(params.OutputSize.xy * params.SourceSize.zw, vec2(1.0));
   vec2 ganz = floor(texel - 0.5);
   vec2 bruch = texel - 0.5 - ganz;
   // Überblendung nur in einem Saum von 1/faktor Breite um die Pixelgrenze
   vec2 saum = 0.5 / faktor;
   vec2 t = clamp((bruch - 0.5 + saum) / (2.0 * saum), 0.0, 1.0);
   vec2 uv = (ganz + 0.5 + t) * params.SourceSize.zw;
   FragColor = vec4(texture(Source, uv).rgb, 1.0);
}
