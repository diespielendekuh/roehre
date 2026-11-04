// ERZEUGT aus fremden MIT-Shadern bzw. roehre (MIT OR Apache-2.0) — eigen/scalefx.slangp, Pass 1 »scalefx-pass0« (Herkunft: shader/fremd/mit/).
// Übersetzt von librashader 0.12.1 (slang → SPIR-V → naga → WGSL). Nicht von Hand ändern.
diagnostic(off, derivative_uniformity);
struct Push {
    SourceSize: vec4<f32>,
}

struct UBO {
    MVP: mat4x4<f32>,
}

@group(0) @binding(1) 
var Source: texture_2d<f32>;
var<private> vTexCoord_1: vec2<f32>;
var<private> FragColor: vec4<f32>;
@group(0) @binding(2) 
var<uniform> params: Push;
@group(0) @binding(0) 
var<uniform> global: UBO;
@group(1) @binding(1) 
var _Source_sampler: sampler;

fn dist_u0028_vf3_u003b_vf3_u003b(A: ptr<function, vec3<f32>>, B: ptr<function, vec3<f32>>) -> f32 {
    var r: f32;
    var d: vec3<f32>;
    var c: vec3<f32>;

    let _e25 = (*A)[0u];
    let _e27 = (*B)[0u];
    r = (0.5f * (_e25 + _e27));
    let _e30 = (*A);
    let _e31 = (*B);
    d = (_e30 - _e31);
    let _e33 = r;
    let _e35 = r;
    c = vec3<f32>((2f + _e33), 4f, (3f - _e35));
    let _e38 = c;
    let _e39 = d;
    let _e41 = d;
    return (sqrt(dot((_e38 * _e39), _e41)) / 3f);
}

fn main_1() {
    var A_1: vec3<f32>;
    var B_1: vec3<f32>;
    var C: vec3<f32>;
    var E: vec3<f32>;
    var F: vec3<f32>;
    var param: vec3<f32>;
    var param_1: vec3<f32>;
    var param_2: vec3<f32>;
    var param_3: vec3<f32>;
    var param_4: vec3<f32>;
    var param_5: vec3<f32>;
    var param_6: vec3<f32>;
    var param_7: vec3<f32>;

    let _e32 = vTexCoord_1;
    let _e33 = textureSample(Source, _Source_sampler, _e32, vec2<i32>(-1i, -1i));
    A_1 = _e33.xyz;
    let _e35 = vTexCoord_1;
    let _e36 = textureSample(Source, _Source_sampler, _e35, vec2<i32>(0i, -1i));
    B_1 = _e36.xyz;
    let _e38 = vTexCoord_1;
    let _e39 = textureSample(Source, _Source_sampler, _e38, vec2<i32>(1i, -1i));
    C = _e39.xyz;
    let _e41 = vTexCoord_1;
    let _e42 = textureSample(Source, _Source_sampler, _e41, vec2<i32>(0i, 0i));
    E = _e42.xyz;
    let _e44 = vTexCoord_1;
    let _e45 = textureSample(Source, _Source_sampler, _e44, vec2<i32>(1i, 0i));
    F = _e45.xyz;
    let _e47 = E;
    param = _e47;
    let _e48 = A_1;
    param_1 = _e48;
    let _e49 = dist_u0028_vf3_u003b_vf3_u003b((&param), (&param_1));
    let _e50 = E;
    param_2 = _e50;
    let _e51 = B_1;
    param_3 = _e51;
    let _e52 = dist_u0028_vf3_u003b_vf3_u003b((&param_2), (&param_3));
    let _e53 = E;
    param_4 = _e53;
    let _e54 = C;
    param_5 = _e54;
    let _e55 = dist_u0028_vf3_u003b_vf3_u003b((&param_4), (&param_5));
    let _e56 = E;
    param_6 = _e56;
    let _e57 = F;
    param_7 = _e57;
    let _e58 = dist_u0028_vf3_u003b_vf3_u003b((&param_6), (&param_7));
    FragColor = vec4<f32>(_e49, _e52, _e55, _e58);
    return;
}

@fragment 
fn main(@location(0) vTexCoord: vec2<f32>) -> @location(0) vec4<f32> {
    vTexCoord_1 = vTexCoord;
    main_1();
    let _e3 = FragColor;
    return _e3;
}
