// ERZEUGT aus fremden MIT-Shadern bzw. roehre (MIT OR Apache-2.0) — eigen/beans/signal.slangp, Pass 0 »transform« (Herkunft: shader/fremd/mit/).
// Übersetzt von librashader 0.12.1 (slang → SPIR-V → naga → WGSL). Nicht von Hand ändern.
diagnostic(off, derivative_uniformity);
struct Push {
    SourceSize: vec4<f32>,
    YIQ: f32,
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

fn gamma_to_yiq_u0028_vf3_u003b(rgb: ptr<function, vec3<f32>>) -> vec3<f32> {
    let _e26 = (*rgb);
    return (mat3x3<f32>(vec3<f32>(0.3f, 0.599f, 0.213f), vec3<f32>(0.59f, -0.2773f, -0.5251f), vec3<f32>(0.11f, -0.3217f, 0.3121f)) * _e26);
}

fn main_1() {
    var pix_loc: vec2<i32>;
    var rgb_1: vec3<f32>;
    var param: vec3<f32>;

    let _e29 = vTexCoord_1[0u];
    let _e32 = params.SourceSize[0u];
    let _e37 = vTexCoord_1[1u];
    let _e40 = params.SourceSize[1u];
    pix_loc = vec2<i32>(i32(floor((_e29 * _e32))), i32(floor((_e37 * _e40))));
    let _e45 = pix_loc;
    let _e46 = textureLoad(Source, _e45, 0i);
    rgb_1 = _e46.xyz;
    let _e49 = params.YIQ;
    if (_e49 > 0.5f) {
        let _e51 = rgb_1;
        param = _e51;
        let _e52 = gamma_to_yiq_u0028_vf3_u003b((&param));
        rgb_1 = _e52;
    }
    let _e53 = rgb_1;
    FragColor[0u] = _e53.x;
    FragColor[1u] = _e53.y;
    FragColor[2u] = _e53.z;
    return;
}

@fragment 
fn main(@location(0) vTexCoord: vec2<f32>) -> @location(0) vec4<f32> {
    vTexCoord_1 = vTexCoord;
    main_1();
    let _e3 = FragColor;
    return _e3;
}
