// ERZEUGT aus fremden MIT-Shadern bzw. roehre (MIT OR Apache-2.0) — eigen/beans/signal.slangp, Pass 10 »bilinear_upsample« (Herkunft: shader/fremd/mit/).
// Übersetzt von librashader 0.12.1 (slang → SPIR-V → naga → WGSL). Nicht von Hand ändern.
diagnostic(off, derivative_uniformity);
struct UBO {
    MVP: mat4x4<f32>,
}

var<private> FragColor: vec4<f32>;
@group(0) @binding(2) 
var Source: texture_2d<f32>;
var<private> vTexCoord_1: vec2<f32>;
@group(0) @binding(0) 
var<uniform> unnamed: UBO;
@group(1) @binding(2) 
var _Source_sampler: sampler;

fn main_1() {
    let _e8 = vTexCoord_1;
    let _e9 = textureSample(Source, _Source_sampler, _e8);
    let _e10 = _e9.xyz;
    FragColor[0u] = _e10.x;
    FragColor[1u] = _e10.y;
    FragColor[2u] = _e10.z;
    return;
}

@fragment 
fn main(@location(0) vTexCoord: vec2<f32>) -> @location(0) vec4<f32> {
    vTexCoord_1 = vTexCoord;
    main_1();
    let _e3 = FragColor;
    return _e3;
}
