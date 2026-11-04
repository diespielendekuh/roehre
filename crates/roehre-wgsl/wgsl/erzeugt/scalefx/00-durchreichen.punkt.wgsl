// ERZEUGT aus fremden MIT-Shadern bzw. roehre (MIT OR Apache-2.0) — eigen/scalefx.slangp, Pass 0 »durchreichen« (Herkunft: shader/fremd/mit/).
// Übersetzt von librashader 0.12.1 (slang → SPIR-V → naga → WGSL). Nicht von Hand ändern.
diagnostic(off, derivative_uniformity);
struct Push {
    SourceSize: vec4<f32>,
    OutputSize: vec4<f32>,
}

struct UBO {
    MVP: mat4x4<f32>,
}

var<private> FragColor: vec4<f32>;
@group(0) @binding(2) 
var Source: texture_2d<f32>;
var<private> vTexCoord_1: vec2<f32>;
@group(0) @binding(3) 
var<uniform> params: Push;
@group(0) @binding(0) 
var<uniform> global: UBO;
@group(1) @binding(2) 
var _Source_sampler: sampler;

fn main_1() {
    let _e7 = vTexCoord_1;
    let _e8 = textureSample(Source, _Source_sampler, _e7);
    let _e9 = _e8.xyz;
    FragColor = vec4<f32>(_e9.x, _e9.y, _e9.z, 1f);
    return;
}

@fragment 
fn main(@location(0) vTexCoord: vec2<f32>) -> @location(0) vec4<f32> {
    vTexCoord_1 = vTexCoord;
    main_1();
    let _e3 = FragColor;
    return _e3;
}
