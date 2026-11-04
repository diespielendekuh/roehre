// ERZEUGT aus crt-beans (Andrew Duffey, MIT) bzw. roehre (MIT OR Apache-2.0) — eigen/beans/rgb.slangp, Pass 1 »filter«.
// Übersetzt von librashader 0.12.1 (slang → SPIR-V → naga → WGSL). Nicht von Hand ändern.
diagnostic(off, derivative_uniformity);
struct Push {
    SourceSize: vec4<f32>,
    Cutoff: f32,
    ICutoff: f32,
    QCutoff: f32,
    YIQ: f32,
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

fn gamma_to_linear_u0028_vf3_u003b(rgb: ptr<function, vec3<f32>>) -> vec3<f32> {
    let _e36 = (*rgb);
    return pow(clamp(_e36, vec3(0f), vec3(1f)), vec3<f32>(2.4f, 2.4f, 2.4f));
}

fn yiq_to_linear_u0028_vf3_u003b(rgb_1: ptr<function, vec3<f32>>) -> vec3<f32> {
    var param: vec3<f32>;

    let _e37 = (*rgb_1);
    param = (mat3x3<f32>(vec3<f32>(1f, 1f, 1f), vec3<f32>(0.9468822f, -0.27478763f, -1.1085451f), vec3<f32>(0.62355655f, -0.6356911f, 1.7090069f)) * _e37);
    let _e39 = gamma_to_linear_u0028_vf3_u003b((&param));
    return _e39;
}

fn main_1() {
    var L: vec3<f32>;
    var local: vec3<f32>;
    var L_max: f32;
    var L_rcp: vec3<f32>;
    var filtered: vec3<f32>;
    var pix_y: i32;
    var t: f32;
    var start: i32;
    var end: i32;
    var t0_: f32;
    var u0_: vec3<f32>;
    var left: vec3<f32>;
    var pix_x: i32;
    var s: vec3<f32>;
    var t1_: f32;
    var u1_: vec3<f32>;
    var right: vec3<f32>;
    var local_1: vec3<f32>;
    var param_1: vec3<f32>;
    var param_2: vec3<f32>;

    let _e56 = params.YIQ;
    if (_e56 > 0.5f) {
        let _e59 = params.Cutoff;
        let _e61 = params.ICutoff;
        let _e63 = params.QCutoff;
        local = (vec3(1f) / ((vec3<f32>(_e59, _e61, _e63) * 53.33f) * 2f));
    } else {
        let _e70 = params.Cutoff;
        local = (vec3(1f) / ((vec3(_e70) * 53.33f) * 2f));
    }
    let _e76 = local;
    L = _e76;
    let _e78 = L[0u];
    let _e80 = L[1u];
    let _e82 = L[2u];
    L_max = max(_e78, max(_e80, _e82));
    let _e85 = L;
    L_rcp = (vec3(1f) / _e85);
    filtered = vec3<f32>(0f, 0f, 0f);
    let _e89 = vTexCoord_1[1u];
    let _e92 = params.SourceSize[1u];
    pix_y = i32(floor((_e89 * _e92)));
    let _e97 = vTexCoord_1[0u];
    t = _e97;
    let _e100 = params.SourceSize[0u];
    let _e102 = vTexCoord_1[0u];
    let _e103 = L_max;
    start = max(i32(floor((_e100 * (_e102 - _e103)))), 0i);
    let _e111 = params.SourceSize[0u];
    let _e113 = vTexCoord_1[0u];
    let _e114 = L_max;
    let _e121 = params.SourceSize[0u];
    end = min(i32(floor((_e111 * (_e113 + _e114)))), (i32(_e121) - 1i));
    let _e125 = start;
    let _e129 = params.SourceSize[2u];
    t0_ = (f32(_e125) * _e129);
    let _e131 = t;
    let _e132 = t0_;
    let _e134 = L_rcp;
    u0_ = clamp((_e134 * (_e131 - _e132)), vec3(-1f), vec3(1f));
    let _e139 = u0_;
    let _e141 = u0_;
    left = ((_e139 * 3.1415927f) + sin((_e141 * 3.1415927f)));
    let _e145 = start;
    pix_x = _e145;
    loop {
        let _e146 = pix_x;
        let _e147 = end;
        if (_e146 <= _e147) {
            let _e149 = pix_x;
            let _e150 = pix_y;
            let _e152 = textureLoad(Source, vec2<i32>(_e149, _e150), 0i);
            s = _e152.xyz;
            let _e154 = pix_x;
            let _e159 = params.SourceSize[2u];
            t1_ = (f32((_e154 + 1i)) * _e159);
            let _e161 = t;
            let _e162 = t1_;
            let _e164 = L_rcp;
            u1_ = clamp((_e164 * (_e161 - _e162)), vec3(-1f), vec3(1f));
            let _e169 = u1_;
            let _e171 = u1_;
            right = ((_e169 * 3.1415927f) + sin((_e171 * 3.1415927f)));
            let _e175 = s;
            let _e176 = left;
            let _e177 = right;
            let _e180 = filtered;
            filtered = (_e180 + (_e175 * (_e176 - _e177)));
            let _e182 = right;
            left = _e182;
            continue;
        } else {
            break;
        }
        continuing {
            let _e183 = pix_x;
            pix_x = (_e183 + 1i);
        }
    }
    let _e185 = filtered;
    filtered = (_e185 / vec3(6.2831855f));
    let _e189 = params.YIQ;
    if (_e189 > 0.5f) {
        let _e191 = filtered;
        param_1 = _e191;
        let _e192 = yiq_to_linear_u0028_vf3_u003b((&param_1));
        local_1 = _e192;
    } else {
        let _e193 = filtered;
        param_2 = _e193;
        let _e194 = gamma_to_linear_u0028_vf3_u003b((&param_2));
        local_1 = _e194;
    }
    let _e195 = local_1;
    FragColor[0u] = _e195.x;
    FragColor[1u] = _e195.y;
    FragColor[2u] = _e195.z;
    return;
}

@fragment 
fn main(@location(0) vTexCoord: vec2<f32>) -> @location(0) vec4<f32> {
    vTexCoord_1 = vTexCoord;
    main_1();
    let _e3 = FragColor;
    return _e3;
}
