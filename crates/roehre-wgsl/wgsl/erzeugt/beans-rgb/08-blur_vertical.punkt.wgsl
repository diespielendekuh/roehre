// ERZEUGT aus fremden MIT-Shadern bzw. roehre (MIT OR Apache-2.0) — eigen/beans/rgb.slangp, Pass 8 »blur_vertical« (Herkunft: shader/fremd/mit/).
// Übersetzt von librashader 0.12.1 (slang → SPIR-V → naga → WGSL). Nicht von Hand ändern.
diagnostic(off, derivative_uniformity);
struct Push {
    SourceSize: vec4<f32>,
    GlowSigma: f32,
}

struct UBO {
    MVP: mat4x4<f32>,
}

@group(0) @binding(2) 
var<uniform> params: Push;
var<private> vTexCoord_1: vec2<f32>;
@group(0) @binding(1) 
var Source: texture_2d<f32>;
var<private> FragColor: vec4<f32>;
@group(0) @binding(0) 
var<uniform> unnamed: UBO;
@group(1) @binding(1) 
var _Source_sampler: sampler;

fn main_1() {
    var sigma: f32;
    var side_taps: i32;
    var pos: vec2<f32>;
    var weight_sum: f32;
    var value: vec3<f32>;
    var g0_: f32;
    var g2_: f32;
    var g1_: f32;
    var distance_y: i32;
    var weight0_: f32;
    var weight1_: f32;
    var scale: f32;
    var delta: f32;

    let _e33 = params.GlowSigma;
    let _e36 = params.SourceSize[1u];
    sigma = (_e33 * _e36);
    let _e38 = sigma;
    side_taps = (2i * i32(ceil((_e38 * 1.5f))));
    let _e43 = vTexCoord_1;
    let _e45 = params.SourceSize;
    pos = (_e43 * _e45.xy);
    weight_sum = 0f;
    value = vec3<f32>(0f, 0f, 0f);
    let _e48 = weight_sum;
    weight_sum = (_e48 + 1f);
    let _e51 = pos[0u];
    let _e55 = pos[1u];
    let _e59 = textureLoad(Source, vec2<i32>(i32(floor(_e51)), i32(floor(_e55))), 0i);
    let _e61 = value;
    value = (_e61 + _e59.xyz);
    let _e63 = sigma;
    let _e65 = sigma;
    g0_ = exp((-1f / ((2f * _e63) * _e65)));
    let _e69 = g0_;
    let _e70 = g0_;
    g2_ = (_e69 * _e70);
    let _e72 = g0_;
    let _e73 = g2_;
    g1_ = (_e72 * _e73);
    distance_y = 1i;
    loop {
        let _e75 = distance_y;
        let _e76 = side_taps;
        if (_e75 <= _e76) {
            let _e78 = g0_;
            weight0_ = _e78;
            let _e79 = g1_;
            let _e80 = g0_;
            g0_ = (_e80 * _e79);
            let _e82 = g2_;
            let _e83 = g1_;
            g1_ = (_e83 * _e82);
            let _e85 = g0_;
            weight1_ = _e85;
            let _e86 = g1_;
            let _e87 = g0_;
            g0_ = (_e87 * _e86);
            let _e89 = g2_;
            let _e90 = g1_;
            g1_ = (_e90 * _e89);
            let _e92 = weight0_;
            let _e93 = weight1_;
            scale = (_e92 + _e93);
            let _e95 = scale;
            let _e97 = weight_sum;
            weight_sum = ((2f * _e95) + _e97);
            let _e99 = distance_y;
            let _e101 = weight1_;
            let _e102 = scale;
            let _e107 = params.SourceSize[3u];
            delta = ((f32(_e99) + (_e101 / _e102)) * _e107);
            let _e110 = vTexCoord_1[0u];
            let _e112 = vTexCoord_1[1u];
            let _e113 = delta;
            let _e116 = textureSampleLevel(Source, _Source_sampler, vec2<f32>(_e110, (_e112 - _e113)), 0f);
            let _e118 = scale;
            let _e120 = value;
            value = (_e120 + (_e116.xyz * _e118));
            let _e123 = vTexCoord_1[0u];
            let _e125 = vTexCoord_1[1u];
            let _e126 = delta;
            let _e129 = textureSampleLevel(Source, _Source_sampler, vec2<f32>(_e123, (_e125 + _e126)), 0f);
            let _e131 = scale;
            let _e133 = value;
            value = (_e133 + (_e129.xyz * _e131));
            continue;
        } else {
            break;
        }
        continuing {
            let _e135 = distance_y;
            distance_y = (_e135 + 2i);
        }
    }
    let _e137 = value;
    let _e138 = weight_sum;
    let _e140 = (_e137 / vec3(_e138));
    FragColor[0u] = _e140.x;
    FragColor[1u] = _e140.y;
    FragColor[2u] = _e140.z;
    return;
}

@fragment 
fn main(@location(0) vTexCoord: vec2<f32>) -> @location(0) vec4<f32> {
    vTexCoord_1 = vTexCoord;
    main_1();
    let _e3 = FragColor;
    return _e3;
}
