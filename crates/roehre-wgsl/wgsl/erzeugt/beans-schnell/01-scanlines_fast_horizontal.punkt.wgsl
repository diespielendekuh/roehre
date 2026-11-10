// ERZEUGT aus fremden MIT-Shadern bzw. roehre (MIT OR Apache-2.0) — eigen/beans/schnell.slangp, Pass 1 »scanlines_fast_horizontal« (Herkunft: shader/fremd/mit/).
// Übersetzt von librashader 0.12.1 (slang → SPIR-V → naga → WGSL). Nicht von Hand ändern.
diagnostic(off, derivative_uniformity);
struct Push {
    SourceSize: vec4<f32>,
    OutputSize: vec4<f32>,
    FinalViewportSize: vec4<f32>,
    MaxSpotSize: f32,
    MinSpotSize: f32,
    OverscanHorizontal: f32,
    OverscanVertical: f32,
    OddFieldFirst: f32,
}

struct UBO {
    MVP: mat4x4<f32>,
}

@group(0) @binding(2) 
var<uniform> params: Push;
var<private> vTexCoord_1: vec2<f32>;
var<private> delta_1: f32;
@group(0) @binding(1) 
var Source: texture_2d<f32>;
var<private> FragColor: vec4<f32>;
@group(0) @binding(0) 
var<uniform> unnamed: UBO;
@group(1) @binding(1) 
var _Source_sampler: sampler;

fn spot_integral_u0028_vf3_u003b_f1_u003b_f1_u003b_f1_u003b(sample_value: ptr<function, vec3<f32>>, x: ptr<function, f32>, x0_: ptr<function, f32>, x1_: ptr<function, f32>) -> vec3<f32> {
    var w_rcp: vec3<f32>;
    var u0_: vec3<f32>;
    var u1_: vec3<f32>;

    let _e28 = params.MinSpotSize;
    let _e30 = params.MaxSpotSize;
    let _e32 = (*sample_value);
    let _e35 = params.MinSpotSize;
    let _e37 = params.MaxSpotSize;
    let _e40 = params.MaxSpotSize;
    w_rcp = (vec3(1f) / (vec3((_e28 * _e30)) - (sqrt(_e32) * ((_e35 * _e37) - _e40))));
    let _e47 = w_rcp;
    let _e48 = (*x);
    let _e49 = (*x0_);
    u0_ = clamp((_e47 * (_e48 - _e49)), vec3(-1f), vec3(1f));
    let _e55 = w_rcp;
    let _e56 = (*x);
    let _e57 = (*x1_);
    u1_ = clamp((_e55 * (_e56 - _e57)), vec3(-1f), vec3(1f));
    let _e63 = (*sample_value);
    let _e64 = u0_;
    let _e66 = u0_;
    let _e70 = u1_;
    let _e73 = u1_;
    return (_e63 * ((((_e64 * 3.1415927f) + sin((_e66 * 3.1415927f))) - (_e70 * 3.1415927f)) - sin((_e73 * 3.1415927f))));
}

fn main_1() {
    var value: vec3<f32>;
    var start_x: f32;
    var stop_x: f32;
    var x_1: f32;
    var sample_x: f32;
    var sample_value_1: vec3<f32>;
    var x0_1: f32;
    var x1_1: f32;
    var param: vec3<f32>;
    var param_1: f32;
    var param_2: f32;
    var param_3: f32;

    value = vec3<f32>(0f, 0f, 0f);
    let _e34 = params.SourceSize[2u];
    let _e36 = vTexCoord_1[0u];
    let _e39 = params.SourceSize[0u];
    let _e42 = params.MaxSpotSize;
    let _e43 = delta_1;
    start_x = (_e34 * (floor(((_e36 * _e39) - (_e42 / _e43))) + 0.5f));
    let _e51 = params.SourceSize[2u];
    let _e53 = vTexCoord_1[0u];
    let _e56 = params.SourceSize[0u];
    let _e59 = params.MaxSpotSize;
    let _e60 = delta_1;
    stop_x = (_e51 * (floor(((_e53 * _e56) + (_e59 / _e60))) + 1f));
    let _e67 = vTexCoord_1[0u];
    let _e70 = params.SourceSize[0u];
    let _e72 = delta_1;
    x_1 = ((_e67 * _e70) * _e72);
    let _e74 = start_x;
    sample_x = _e74;
    loop {
        let _e75 = sample_x;
        let _e76 = stop_x;
        if (_e75 < _e76) {
            let _e78 = sample_x;
            let _e80 = vTexCoord_1[1u];
            let _e82 = textureSampleLevel(Source, _Source_sampler, vec2<f32>(_e78, _e80), 0f);
            sample_value_1 = _e82.xyz;
            let _e84 = delta_1;
            let _e87 = params.SourceSize[0u];
            let _e88 = sample_x;
            x0_1 = (_e84 * ((_e87 * _e88) - 0.5f));
            let _e92 = x0_1;
            let _e93 = delta_1;
            x1_1 = (_e92 + _e93);
            let _e95 = sample_value_1;
            param = _e95;
            let _e96 = x_1;
            param_1 = _e96;
            let _e97 = x0_1;
            param_2 = _e97;
            let _e98 = x1_1;
            param_3 = _e98;
            let _e99 = spot_integral_u0028_vf3_u003b_f1_u003b_f1_u003b_f1_u003b((&param), (&param_1), (&param_2), (&param_3));
            let _e100 = value;
            value = (_e100 + _e99);
            continue;
        } else {
            break;
        }
        continuing {
            let _e104 = params.SourceSize[2u];
            let _e105 = sample_x;
            sample_x = (_e105 + _e104);
        }
    }
    let _e107 = value;
    value = (_e107 / vec3(6.2831855f));
    let _e110 = value;
    FragColor[0u] = _e110.x;
    FragColor[1u] = _e110.y;
    FragColor[2u] = _e110.z;
    return;
}

@fragment 
fn main(@location(0) vTexCoord: vec2<f32>, @location(1) @interpolate(flat) delta: f32) -> @location(0) vec4<f32> {
    vTexCoord_1 = vTexCoord;
    delta_1 = delta;
    main_1();
    let _e5 = FragColor;
    return _e5;
}
