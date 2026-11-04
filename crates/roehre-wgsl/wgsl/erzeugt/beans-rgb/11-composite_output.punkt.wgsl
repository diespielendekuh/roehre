// ERZEUGT aus fremden MIT-Shadern bzw. roehre (MIT OR Apache-2.0) — eigen/beans/rgb.slangp, Pass 11 »composite_output« (Herkunft: shader/fremd/mit/).
// Übersetzt von librashader 0.12.1 (slang → SPIR-V → naga → WGSL). Nicht von Hand ändern.
diagnostic(off, derivative_uniformity);
struct Push {
    OutputGamma: f32,
    SourceSize: vec4<f32>,
    ScanlinesSize: vec4<f32>,
    FrameCount: u32,
    GlowSigma: f32,
    GlowAmount: f32,
}

struct UBO {
    MVP: mat4x4<f32>,
}

@group(0) @binding(4) 
var<uniform> params: Push;
@group(0) @binding(3) 
var BlueNoiseTex: texture_2d<f32>;
@group(0) @binding(1) 
var Source: texture_2d<f32>;
var<private> vTexCoord_1: vec2<f32>;
@group(0) @binding(2) 
var Scanlines: texture_2d<f32>;
var<private> FragColor: vec4<f32>;
@group(0) @binding(0) 
var<uniform> unnamed: UBO;
@group(1) @binding(3) 
var _BlueNoiseTex_sampler: sampler;
@group(1) @binding(1) 
var _Source_sampler: sampler;
@group(1) @binding(2) 
var _Scanlines_sampler: sampler;

fn gamma_to_linear_u0028_vf3_u003b(rgb: ptr<function, vec3<f32>>) -> vec3<f32> {
    let _e42 = (*rgb);
    (*rgb) = clamp(_e42, vec3(0f), vec3(1f));
    let _e46 = (*rgb);
    return pow(_e46, vec3<f32>(2.2f, 2.2f, 2.2f));
}

fn linear_to_gamma_u0028_vf3_u003b(rgb_1: ptr<function, vec3<f32>>) -> vec3<f32> {
    let _e42 = (*rgb_1);
    (*rgb_1) = clamp(_e42, vec3(0f), vec3(1f));
    let _e46 = (*rgb_1);
    return pow(_e46, vec3<f32>(0.45454547f, 0.45454547f, 0.45454547f));
}

fn dither_gamma_u0028_vf3_u003b_vi2_u003b(rgb_2: ptr<function, vec3<f32>>, icoords: ptr<function, vec2<i32>>) -> vec3<f32> {
    var offset: vec2<i32>;
    var noise: vec3<f32>;
    var lower: vec3<f32>;
    var param: vec3<f32>;
    var upper: vec3<f32>;
    var threshold: vec3<f32>;
    var param_1: vec3<f32>;
    var param_2: vec3<f32>;

    let _e52 = params.FrameCount;
    offset = (vec2(bitcast<i32>((_e52 & 63u))) * vec2<i32>(17i, 13i));
    let _e57 = (*icoords);
    let _e58 = offset;
    let _e62 = textureLoad(BlueNoiseTex, ((_e57 + _e58) & vec2(63i)), 0i);
    noise = _e62.xyz;
    let _e64 = (*rgb_2);
    param = _e64;
    let _e65 = linear_to_gamma_u0028_vf3_u003b((&param));
    lower = (floor((_e65 / vec3(0.003921569f))) * 0.003921569f);
    let _e70 = lower;
    upper = (_e70 + vec3(0.003921569f));
    let _e73 = lower;
    param_1 = _e73;
    let _e74 = gamma_to_linear_u0028_vf3_u003b((&param_1));
    let _e75 = upper;
    param_2 = _e75;
    let _e76 = gamma_to_linear_u0028_vf3_u003b((&param_2));
    let _e77 = noise;
    threshold = mix(_e74, _e76, _e77);
    let _e79 = lower;
    let _e80 = upper;
    let _e81 = threshold;
    let _e82 = (*rgb_2);
    return select(_e79, _e80, (_e81 < _e82));
}

fn srgb_to_linear_u0028_vf3_u003b(rgb_3: ptr<function, vec3<f32>>) -> vec3<f32> {
    var cutoff: vec3<bool>;
    var lower_1: vec3<f32>;
    var higher: vec3<f32>;

    let _e45 = (*rgb_3);
    (*rgb_3) = clamp(_e45, vec3(0f), vec3(1f));
    let _e49 = (*rgb_3);
    cutoff = (_e49 < vec3<f32>(0.04045f, 0.04045f, 0.04045f));
    let _e51 = (*rgb_3);
    lower_1 = (_e51 / vec3(12.92f));
    let _e54 = (*rgb_3);
    higher = pow(((_e54 + vec3(0.055f)) / vec3(1.055f)), vec3<f32>(2.4f, 2.4f, 2.4f));
    let _e60 = higher;
    let _e61 = lower_1;
    let _e62 = cutoff;
    return select(_e60, _e61, _e62);
}

fn linear_to_srgb_u0028_vf3_u003b(rgb_4: ptr<function, vec3<f32>>) -> vec3<f32> {
    var cutoff_1: vec3<bool>;
    var lower_2: vec3<f32>;
    var higher_1: vec3<f32>;

    let _e45 = (*rgb_4);
    (*rgb_4) = clamp(_e45, vec3(0f), vec3(1f));
    let _e49 = (*rgb_4);
    cutoff_1 = (_e49 < vec3<f32>(0.0031308f, 0.0031308f, 0.0031308f));
    let _e51 = (*rgb_4);
    lower_2 = (_e51 * vec3<f32>(12.92f, 12.92f, 12.92f));
    let _e53 = (*rgb_4);
    higher_1 = ((vec3<f32>(1.055f, 1.055f, 1.055f) * pow(_e53, vec3<f32>(0.41666666f, 0.41666666f, 0.41666666f))) - vec3<f32>(0.055f, 0.055f, 0.055f));
    let _e57 = higher_1;
    let _e58 = lower_2;
    let _e59 = cutoff_1;
    return select(_e57, _e58, _e59);
}

fn dither_srgb_u0028_vf3_u003b_vi2_u003b(rgb_5: ptr<function, vec3<f32>>, icoords_1: ptr<function, vec2<i32>>) -> vec3<f32> {
    var offset_1: vec2<i32>;
    var noise_1: vec3<f32>;
    var lower_3: vec3<f32>;
    var param_3: vec3<f32>;
    var upper_1: vec3<f32>;
    var threshold_1: vec3<f32>;
    var param_4: vec3<f32>;
    var param_5: vec3<f32>;

    let _e52 = params.FrameCount;
    offset_1 = (vec2(bitcast<i32>((_e52 & 63u))) * vec2<i32>(17i, 13i));
    let _e57 = (*icoords_1);
    let _e58 = offset_1;
    let _e62 = textureLoad(BlueNoiseTex, ((_e57 + _e58) & vec2(63i)), 0i);
    noise_1 = _e62.xyz;
    let _e64 = (*rgb_5);
    param_3 = _e64;
    let _e65 = linear_to_srgb_u0028_vf3_u003b((&param_3));
    lower_3 = (floor((_e65 / vec3(0.003921569f))) * 0.003921569f);
    let _e70 = lower_3;
    upper_1 = (_e70 + vec3(0.003921569f));
    let _e73 = lower_3;
    param_4 = _e73;
    let _e74 = srgb_to_linear_u0028_vf3_u003b((&param_4));
    let _e75 = upper_1;
    param_5 = _e75;
    let _e76 = srgb_to_linear_u0028_vf3_u003b((&param_5));
    let _e77 = noise_1;
    threshold_1 = mix(_e74, _e76, _e77);
    let _e79 = lower_3;
    let _e80 = upper_1;
    let _e81 = threshold_1;
    let _e82 = (*rgb_5);
    return select(_e79, _e80, (_e81 < _e82));
}

fn main_1() {
    var blur_value: vec3<f32>;
    var icoords_2: vec2<i32>;
    var scanlines_value: vec3<f32>;
    var out_value: vec3<f32>;
    var param_6: vec3<f32>;
    var param_7: vec2<i32>;
    var param_8: vec3<f32>;
    var param_9: vec2<i32>;

    let _e49 = vTexCoord_1;
    let _e50 = textureSampleLevel(Source, _Source_sampler, _e49, 0f);
    blur_value = _e50.xyz;
    let _e52 = vTexCoord_1;
    let _e54 = params.ScanlinesSize;
    icoords_2 = vec2<i32>(floor((_e52 * _e54.xy)));
    let _e59 = icoords_2;
    let _e60 = textureLoad(Scanlines, _e59, 0i);
    scanlines_value = _e60.xyz;
    let _e62 = scanlines_value;
    let _e63 = blur_value;
    let _e65 = params.GlowAmount;
    out_value = mix(_e62, _e63, vec3(_e65));
    let _e69 = params.OutputGamma;
    if (_e69 < 0.5f) {
        let _e71 = out_value;
        param_6 = _e71;
        let _e72 = icoords_2;
        param_7 = _e72;
        let _e73 = dither_srgb_u0028_vf3_u003b_vi2_u003b((&param_6), (&param_7));
        out_value = _e73;
    } else {
        let _e74 = out_value;
        param_8 = _e74;
        let _e75 = icoords_2;
        param_9 = _e75;
        let _e76 = dither_gamma_u0028_vf3_u003b_vi2_u003b((&param_8), (&param_9));
        out_value = _e76;
    }
    let _e77 = out_value;
    FragColor = vec4<f32>(_e77.x, _e77.y, _e77.z, 1f);
    return;
}

@fragment 
fn main(@location(0) vTexCoord: vec2<f32>) -> @location(0) vec4<f32> {
    vTexCoord_1 = vTexCoord;
    main_1();
    let _e3 = FragColor;
    return _e3;
}
