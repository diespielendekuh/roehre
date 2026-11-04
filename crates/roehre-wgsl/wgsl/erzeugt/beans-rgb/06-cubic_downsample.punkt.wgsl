// ERZEUGT aus crt-beans (Andrew Duffey, MIT) bzw. roehre (MIT OR Apache-2.0) — eigen/beans/rgb.slangp, Pass 6 »cubic_downsample«.
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

fn main_1() {
    var pos: vec2<f32>;
    var center: vec2<f32>;
    var x: vec2<f32>;
    var weight0_: vec2<f32>;
    var weight1_: vec2<f32>;
    var weight2_: vec2<f32>;
    var weight3_: vec2<f32>;
    var scale0_: vec2<f32>;
    var scale1_: vec2<f32>;
    var coord0_: vec2<f32>;
    var coord1_: vec2<f32>;
    var value: vec3<f32>;
    var weight_sum: vec2<f32>;

    let _e28 = vTexCoord_1;
    let _e30 = params.SourceSize;
    pos = (_e28 * _e30.xy);
    let _e33 = pos;
    center = (floor((_e33 - vec2(0.5f))) + vec2(0.5f));
    let _e39 = pos;
    let _e40 = center;
    x = ((_e39 - _e40) + vec2(1f));
    let _e44 = x;
    let _e45 = x;
    let _e47 = x;
    weight0_ = (((_e44 * _e45) * ((abs(_e47) * 0.25f) - vec2(0.75f))) + vec2(1f));
    let _e55 = pos;
    let _e56 = center;
    x = (_e55 - _e56);
    let _e58 = x;
    let _e59 = x;
    let _e61 = x;
    weight1_ = (((_e58 * _e59) * ((abs(_e61) * 0.25f) - vec2(0.75f))) + vec2(1f));
    let _e69 = pos;
    let _e70 = center;
    x = ((_e69 - _e70) - vec2(1f));
    let _e74 = x;
    let _e75 = x;
    let _e77 = x;
    weight2_ = (((_e74 * _e75) * ((abs(_e77) * 0.25f) - vec2(0.75f))) + vec2(1f));
    let _e85 = pos;
    let _e86 = center;
    x = ((_e85 - _e86) - vec2(2f));
    let _e90 = x;
    let _e91 = x;
    let _e93 = x;
    weight3_ = (((_e90 * _e91) * ((abs(_e93) * 0.25f) - vec2(0.75f))) + vec2(1f));
    let _e101 = weight0_;
    let _e102 = weight1_;
    scale0_ = (_e101 + _e102);
    let _e104 = weight2_;
    let _e105 = weight3_;
    scale1_ = (_e104 + _e105);
    let _e107 = center;
    let _e110 = weight1_;
    let _e111 = scale0_;
    let _e115 = params.SourceSize;
    coord0_ = (((_e107 - vec2(1f)) + (_e110 / _e111)) * _e115.zw);
    let _e118 = center;
    let _e121 = weight3_;
    let _e122 = scale1_;
    let _e126 = params.SourceSize;
    coord1_ = (((_e118 + vec2(1f)) + (_e121 / _e122)) * _e126.zw);
    let _e130 = coord0_[0u];
    let _e132 = coord0_[1u];
    let _e134 = textureSampleLevel(Source, _Source_sampler, vec2<f32>(_e130, _e132), 0f);
    let _e137 = scale0_[0u];
    let _e140 = coord1_[0u];
    let _e142 = coord0_[1u];
    let _e144 = textureSampleLevel(Source, _Source_sampler, vec2<f32>(_e140, _e142), 0f);
    let _e147 = scale1_[0u];
    let _e151 = scale0_[1u];
    let _e154 = coord0_[0u];
    let _e156 = coord1_[1u];
    let _e158 = textureSampleLevel(Source, _Source_sampler, vec2<f32>(_e154, _e156), 0f);
    let _e161 = scale0_[0u];
    let _e164 = coord1_[0u];
    let _e166 = coord1_[1u];
    let _e168 = textureSampleLevel(Source, _Source_sampler, vec2<f32>(_e164, _e166), 0f);
    let _e171 = scale1_[0u];
    let _e175 = scale1_[1u];
    value = ((((_e134.xyz * _e137) + (_e144.xyz * _e147)) * _e151) + (((_e158.xyz * _e161) + (_e168.xyz * _e171)) * _e175));
    let _e178 = scale0_;
    let _e179 = scale1_;
    weight_sum = (_e178 + _e179);
    let _e182 = weight_sum[0u];
    let _e184 = weight_sum[1u];
    let _e186 = value;
    value = (_e186 / vec3((_e182 * _e184)));
    let _e189 = value;
    FragColor = vec4<f32>(_e189.x, _e189.y, _e189.z, 1f);
    return;
}

@fragment 
fn main(@location(0) vTexCoord: vec2<f32>) -> @location(0) vec4<f32> {
    vTexCoord_1 = vTexCoord;
    main_1();
    let _e3 = FragColor;
    return _e3;
}
