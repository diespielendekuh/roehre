// ERZEUGT aus crt-beans (Andrew Duffey, MIT) bzw. roehre (MIT OR Apache-2.0) — eigen/beans/signal.slangp, Pass 2 »calculate_widths«.
// Übersetzt von librashader 0.12.1 (slang → SPIR-V → naga → WGSL). Nicht von Hand ändern.
diagnostic(off, derivative_uniformity);
struct Push {
    SourceSize: vec4<f32>,
    MaxSpotSize: f32,
    MinSpotSize: f32,
}

struct UBO {
    MVP: mat4x4<f32>,
}

var<private> vTexCoord_1: vec2<f32>;
@group(0) @binding(2) 
var<uniform> params: Push;
@group(0) @binding(1) 
var Source: texture_2d<f32>;
var<private> FragColor: vec4<f32>;
@group(0) @binding(0) 
var<uniform> unnamed: UBO;
@group(1) @binding(1) 
var _Source_sampler: sampler;

fn main_1() {
    var pix_loc: vec2<i32>;
    var sample_value: vec3<f32>;
    var width_rcp: vec3<f32>;

    let _e16 = vTexCoord_1;
    let _e18 = params.SourceSize;
    pix_loc = vec2<i32>(floor((_e16 * _e18.xy)));
    let _e23 = pix_loc;
    let _e24 = textureLoad(Source, _e23, 0i);
    sample_value = _e24.xyz;
    let _e27 = params.MinSpotSize;
    let _e29 = params.MaxSpotSize;
    let _e31 = sample_value;
    let _e34 = params.MinSpotSize;
    let _e36 = params.MaxSpotSize;
    let _e39 = params.MaxSpotSize;
    width_rcp = (vec3(1f) / (vec3((_e27 * _e29)) - (sqrt(_e31) * ((_e34 * _e36) - _e39))));
    let _e46 = width_rcp;
    FragColor[0u] = _e46.x;
    FragColor[1u] = _e46.y;
    FragColor[2u] = _e46.z;
    return;
}

@fragment 
fn main(@location(0) vTexCoord: vec2<f32>) -> @location(0) vec4<f32> {
    vTexCoord_1 = vTexCoord;
    main_1();
    let _e3 = FragColor;
    return _e3;
}
