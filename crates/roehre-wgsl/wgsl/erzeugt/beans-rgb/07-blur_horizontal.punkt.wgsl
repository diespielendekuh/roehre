// ERZEUGT aus crt-beans (Andrew Duffey, MIT) bzw. roehre (MIT OR Apache-2.0) — eigen/beans/rgb.slangp, Pass 7 »blur_horizontal«.
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
    var distance_x: i32;
    var weight0_: f32;
    var weight1_: f32;
    var scale: f32;
    var delta: f32;

    let _e32 = params.GlowSigma;
    let _e35 = params.SourceSize[1u];
    sigma = (_e32 * _e35);
    let _e37 = sigma;
    side_taps = (2i * i32(ceil((_e37 * 1.5f))));
    let _e42 = vTexCoord_1;
    let _e44 = params.SourceSize;
    pos = (_e42 * _e44.xy);
    weight_sum = 0f;
    value = vec3<f32>(0f, 0f, 0f);
    let _e47 = weight_sum;
    weight_sum = (_e47 + 1f);
    let _e50 = pos[0u];
    let _e54 = pos[1u];
    let _e58 = textureLoad(Source, vec2<i32>(i32(floor(_e50)), i32(floor(_e54))), 0i);
    let _e60 = value;
    value = (_e60 + _e58.xyz);
    let _e62 = sigma;
    let _e64 = sigma;
    g0_ = exp((-1f / ((2f * _e62) * _e64)));
    let _e68 = g0_;
    let _e69 = g0_;
    g2_ = (_e68 * _e69);
    let _e71 = g0_;
    let _e72 = g2_;
    g1_ = (_e71 * _e72);
    distance_x = 1i;
    loop {
        let _e74 = distance_x;
        let _e75 = side_taps;
        if (_e74 <= _e75) {
            let _e77 = g0_;
            weight0_ = _e77;
            let _e78 = g1_;
            let _e79 = g0_;
            g0_ = (_e79 * _e78);
            let _e81 = g2_;
            let _e82 = g1_;
            g1_ = (_e82 * _e81);
            let _e84 = g0_;
            weight1_ = _e84;
            let _e85 = g1_;
            let _e86 = g0_;
            g0_ = (_e86 * _e85);
            let _e88 = g2_;
            let _e89 = g1_;
            g1_ = (_e89 * _e88);
            let _e91 = weight0_;
            let _e92 = weight1_;
            scale = (_e91 + _e92);
            let _e94 = scale;
            let _e96 = weight_sum;
            weight_sum = ((2f * _e94) + _e96);
            let _e98 = distance_x;
            let _e100 = weight1_;
            let _e101 = scale;
            let _e106 = params.SourceSize[2u];
            delta = ((f32(_e98) + (_e100 / _e101)) * _e106);
            let _e109 = vTexCoord_1[0u];
            let _e110 = delta;
            let _e113 = vTexCoord_1[1u];
            let _e115 = textureSampleLevel(Source, _Source_sampler, vec2<f32>((_e109 - _e110), _e113), 0f);
            let _e117 = scale;
            let _e119 = value;
            value = (_e119 + (_e115.xyz * _e117));
            let _e122 = vTexCoord_1[0u];
            let _e123 = delta;
            let _e126 = vTexCoord_1[1u];
            let _e128 = textureSampleLevel(Source, _Source_sampler, vec2<f32>((_e122 + _e123), _e126), 0f);
            let _e130 = scale;
            let _e132 = value;
            value = (_e132 + (_e128.xyz * _e130));
            continue;
        } else {
            break;
        }
        continuing {
            let _e134 = distance_x;
            distance_x = (_e134 + 2i);
        }
    }
    let _e136 = value;
    let _e137 = weight_sum;
    let _e139 = (_e136 / vec3(_e137));
    FragColor[0u] = _e139.x;
    FragColor[1u] = _e139.y;
    FragColor[2u] = _e139.z;
    return;
}

@fragment 
fn main(@location(0) vTexCoord: vec2<f32>) -> @location(0) vec4<f32> {
    vTexCoord_1 = vTexCoord;
    main_1();
    let _e3 = FragColor;
    return _e3;
}
