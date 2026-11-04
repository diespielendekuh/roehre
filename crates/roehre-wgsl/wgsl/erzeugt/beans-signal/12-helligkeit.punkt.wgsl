// ERZEUGT aus crt-beans (Andrew Duffey, MIT) bzw. roehre (MIT OR Apache-2.0) — eigen/beans/signal.slangp, Pass 12 »helligkeit«.
// Übersetzt von librashader 0.12.1 (slang → SPIR-V → naga → WGSL). Nicht von Hand ändern.
diagnostic(off, derivative_uniformity);
struct Push {
    SourceSize: vec4<f32>,
    OutputSize: vec4<f32>,
    roehre_verstaerkung: f32,
    roehre_kurve: f32,
}

struct UBO {
    MVP: mat4x4<f32>,
}

@group(0) @binding(3) 
var<uniform> params: Push;
@group(0) @binding(2) 
var Source: texture_2d<f32>;
var<private> vTexCoord_1: vec2<f32>;
var<private> FragColor: vec4<f32>;
@group(0) @binding(0) 
var<uniform> global: UBO;
@group(1) @binding(2) 
var _Source_sampler: sampler;

fn kodieren_u0028_vf3_u003b(c: ptr<function, vec3<f32>>) -> vec3<f32> {
    let _e29 = params.roehre_kurve;
    if (_e29 > 0.5f) {
        let _e31 = (*c);
        return pow(_e31, vec3<f32>(0.45454547f, 0.45454547f, 0.45454547f));
    }
    let _e33 = (*c);
    let _e35 = (*c);
    let _e40 = (*c);
    return mix((_e33 * 12.92f), ((pow(_e35, vec3<f32>(0.41666666f, 0.41666666f, 0.41666666f)) * 1.055f) - vec3(0.055f)), step(vec3<f32>(0.0031308f, 0.0031308f, 0.0031308f), _e40));
}

fn dekodieren_u0028_vf3_u003b(c_1: ptr<function, vec3<f32>>) -> vec3<f32> {
    let _e29 = params.roehre_kurve;
    if (_e29 > 0.5f) {
        let _e31 = (*c_1);
        return pow(_e31, vec3<f32>(2.2f, 2.2f, 2.2f));
    }
    let _e33 = (*c_1);
    let _e36 = (*c_1);
    let _e42 = (*c_1);
    return mix((_e33 / vec3(12.92f)), pow(((_e36 + vec3(0.055f)) / vec3(1.055f)), vec3<f32>(2.4f, 2.4f, 2.4f)), step(vec3<f32>(0.04045f, 0.04045f, 0.04045f), _e42));
}

fn main_1() {
    var c_2: vec3<f32>;
    var l: vec3<f32>;
    var param: vec3<f32>;
    var param_1: vec3<f32>;

    let _e31 = vTexCoord_1;
    let _e33 = params.SourceSize;
    let _e37 = textureLoad(Source, vec2<i32>((_e31 * _e33.xy)), 0i);
    c_2 = _e37.xyz;
    let _e39 = c_2;
    param = _e39;
    let _e40 = dekodieren_u0028_vf3_u003b((&param));
    let _e42 = params.roehre_verstaerkung;
    l = clamp((_e40 * _e42), vec3(0f), vec3(1f));
    let _e47 = l;
    param_1 = _e47;
    let _e48 = kodieren_u0028_vf3_u003b((&param_1));
    FragColor = vec4<f32>(_e48.x, _e48.y, _e48.z, 1f);
    return;
}

@fragment 
fn main(@location(0) vTexCoord: vec2<f32>) -> @location(0) vec4<f32> {
    vTexCoord_1 = vTexCoord;
    main_1();
    let _e3 = FragColor;
    return _e3;
}
