// ERZEUGT aus fremden MIT-Shadern bzw. roehre (MIT OR Apache-2.0) — eigen/beans/schnell.slangp, Pass 0 »linearize« (Herkunft: shader/fremd/mit/).
// Übersetzt von librashader 0.12.1 (slang → SPIR-V → naga → WGSL). Nicht von Hand ändern.
diagnostic(off, derivative_uniformity);
struct Push {
    SourceSize: vec4<f32>,
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

fn gamma_to_linear_u0028_vf3_u003b(rgb: ptr<function, vec3<f32>>) -> vec3<f32> {
    let _e15 = (*rgb);
    return pow(clamp(_e15, vec3(0f), vec3(1f)), vec3<f32>(2.4f, 2.4f, 2.4f));
}

fn main_1() {
    var pix_loc: vec2<i32>;
    var rgb_1: vec3<f32>;
    var param: vec3<f32>;

    let _e18 = vTexCoord_1[0u];
    let _e21 = params.SourceSize[0u];
    let _e26 = vTexCoord_1[1u];
    let _e29 = params.SourceSize[1u];
    pix_loc = vec2<i32>(i32(floor((_e18 * _e21))), i32(floor((_e26 * _e29))));
    let _e34 = pix_loc;
    let _e35 = textureLoad(Source, _e34, 0i);
    rgb_1 = _e35.xyz;
    let _e37 = rgb_1;
    param = _e37;
    let _e38 = gamma_to_linear_u0028_vf3_u003b((&param));
    FragColor[0u] = _e38.x;
    FragColor[1u] = _e38.y;
    FragColor[2u] = _e38.z;
    return;
}

@fragment 
fn main(@location(0) vTexCoord: vec2<f32>) -> @location(0) vec4<f32> {
    vTexCoord_1 = vTexCoord;
    main_1();
    let _e3 = FragColor;
    return _e3;
}
