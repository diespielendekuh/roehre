// ERZEUGT aus fremden MIT-Shadern bzw. roehre (MIT OR Apache-2.0) — eigen/scalefx.slangp, Pass 2 »scalefx-pass1« (Herkunft: shader/fremd/mit/).
// Übersetzt von librashader 0.12.1 (slang → SPIR-V → naga → WGSL). Nicht von Hand ändern.
diagnostic(off, derivative_uniformity);
struct Push {
    SourceSize: vec4<f32>,
    SFX_CLR: f32,
    SFX_SAA: f32,
}

struct UBO {
    MVP: mat4x4<f32>,
}

@group(0) @binding(2) 
var<uniform> params: Push;
@group(0) @binding(1) 
var Source: texture_2d<f32>;
var<private> vTexCoord_1: vec2<f32>;
var<private> FragColor: vec4<f32>;
@group(0) @binding(0) 
var<uniform> global: UBO;
@group(1) @binding(1) 
var _Source_sampler: sampler;

fn str_u0028_f1_u003b_vf2_u003b_vf2_u003b(d: ptr<function, f32>, a: ptr<function, vec2<f32>>, b: ptr<function, vec2<f32>>) -> f32 {
    var diff: f32;
    var wght1_: f32;
    var wght2_: f32;
    var local: f32;
    var local_1: f32;
    var phi_90_: bool;

    let _e34 = (*a)[0u];
    let _e36 = (*a)[1u];
    diff = (_e34 - _e36);
    let _e39 = params.SFX_CLR;
    let _e40 = (*d);
    let _e44 = params.SFX_CLR;
    wght1_ = (max((_e39 - _e40), 0f) / _e44);
    let _e46 = (*d);
    let _e49 = (*a)[0u];
    let _e51 = (*b)[0u];
    let _e54 = (*a)[0u];
    let _e57 = (*a)[1u];
    let _e59 = (*b)[1u];
    let _e62 = (*a)[1u];
    if ((min(_e49, _e51) + _e54) > (min(_e57, _e59) + _e62)) {
        let _e65 = diff;
        local = _e65;
    } else {
        let _e66 = diff;
        local = -(_e66);
    }
    let _e68 = local;
    wght2_ = clamp(((1f - _e46) + _e68), 0f, 1f);
    let _e72 = params.SFX_SAA;
    let _e73 = (_e72 == 1f);
    phi_90_ = _e73;
    if !(_e73) {
        let _e75 = (*d);
        let _e78 = (*a)[0u];
        let _e80 = (*a)[1u];
        phi_90_ = ((2f * _e75) < (_e78 + _e80));
    }
    let _e84 = phi_90_;
    if _e84 {
        let _e85 = wght1_;
        let _e86 = wght2_;
        let _e89 = (*a)[0u];
        let _e91 = (*a)[1u];
        local_1 = ((_e85 * _e86) * (_e89 * _e91));
    } else {
        local_1 = 0f;
    }
    let _e94 = local_1;
    return _e94;
}

fn main_1() {
    var A: vec4<f32>;
    var B: vec4<f32>;
    var D: vec4<f32>;
    var E: vec4<f32>;
    var F: vec4<f32>;
    var G: vec4<f32>;
    var H: vec4<f32>;
    var I: vec4<f32>;
    var res: vec4<f32>;
    var param: f32;
    var param_1: vec2<f32>;
    var param_2: vec2<f32>;
    var param_3: f32;
    var param_4: vec2<f32>;
    var param_5: vec2<f32>;
    var param_6: f32;
    var param_7: vec2<f32>;
    var param_8: vec2<f32>;
    var param_9: f32;
    var param_10: vec2<f32>;
    var param_11: vec2<f32>;

    let _e46 = vTexCoord_1;
    let _e47 = textureSample(Source, _Source_sampler, _e46, vec2<i32>(-1i, -1i));
    A = _e47;
    let _e48 = vTexCoord_1;
    let _e49 = textureSample(Source, _Source_sampler, _e48, vec2<i32>(0i, -1i));
    B = _e49;
    let _e50 = vTexCoord_1;
    let _e51 = textureSample(Source, _Source_sampler, _e50, vec2<i32>(-1i, 0i));
    D = _e51;
    let _e52 = vTexCoord_1;
    let _e53 = textureSample(Source, _Source_sampler, _e52, vec2<i32>(0i, 0i));
    E = _e53;
    let _e54 = vTexCoord_1;
    let _e55 = textureSample(Source, _Source_sampler, _e54, vec2<i32>(1i, 0i));
    F = _e55;
    let _e56 = vTexCoord_1;
    let _e57 = textureSample(Source, _Source_sampler, _e56, vec2<i32>(-1i, 1i));
    G = _e57;
    let _e58 = vTexCoord_1;
    let _e59 = textureSample(Source, _Source_sampler, _e58, vec2<i32>(0i, 1i));
    H = _e59;
    let _e60 = vTexCoord_1;
    let _e61 = textureSample(Source, _Source_sampler, _e60, vec2<i32>(1i, 1i));
    I = _e61;
    let _e63 = D[3u];
    let _e65 = E[1u];
    let _e68 = A[3u];
    let _e70 = D[1u];
    let _e73 = D[2u];
    param = _e73;
    param_1 = vec2<f32>(_e63, _e65);
    param_2 = vec2<f32>(_e68, _e70);
    let _e74 = str_u0028_f1_u003b_vf2_u003b_vf2_u003b((&param), (&param_1), (&param_2));
    res[0u] = _e74;
    let _e77 = E[3u];
    let _e79 = E[1u];
    let _e82 = B[3u];
    let _e84 = F[1u];
    let _e87 = F[0u];
    param_3 = _e87;
    param_4 = vec2<f32>(_e77, _e79);
    param_5 = vec2<f32>(_e82, _e84);
    let _e88 = str_u0028_f1_u003b_vf2_u003b_vf2_u003b((&param_3), (&param_4), (&param_5));
    res[1u] = _e88;
    let _e91 = E[3u];
    let _e93 = H[1u];
    let _e96 = H[3u];
    let _e98 = I[1u];
    let _e101 = H[2u];
    param_6 = _e101;
    param_7 = vec2<f32>(_e91, _e93);
    param_8 = vec2<f32>(_e96, _e98);
    let _e102 = str_u0028_f1_u003b_vf2_u003b_vf2_u003b((&param_6), (&param_7), (&param_8));
    res[2u] = _e102;
    let _e105 = D[3u];
    let _e107 = H[1u];
    let _e110 = G[3u];
    let _e112 = G[1u];
    let _e115 = H[0u];
    param_9 = _e115;
    param_10 = vec2<f32>(_e105, _e107);
    param_11 = vec2<f32>(_e110, _e112);
    let _e116 = str_u0028_f1_u003b_vf2_u003b_vf2_u003b((&param_9), (&param_10), (&param_11));
    res[3u] = _e116;
    let _e118 = res;
    FragColor = _e118;
    return;
}

@fragment 
fn main(@location(0) vTexCoord: vec2<f32>) -> @location(0) vec4<f32> {
    vTexCoord_1 = vTexCoord;
    main_1();
    let _e3 = FragColor;
    return _e3;
}
