// ERZEUGT aus fremden MIT-Shadern bzw. roehre (MIT OR Apache-2.0) — eigen/scalefx.slangp, Pass 5 »scalefx-pass4« (Herkunft: shader/fremd/mit/).
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
@group(0) @binding(3) 
var<uniform> params: Push;
var<private> FragColor: vec4<f32>;
@group(0) @binding(2) 
var refpass: texture_2d<f32>;
@group(0) @binding(0) 
var<uniform> global: UBO;
@group(1) @binding(1) 
var _Source_sampler: sampler;
@group(1) @binding(2) 
var _refpass_sampler: sampler;

fn loadMid_u0028_vf4_u003b(x: ptr<function, vec4<f32>>) -> vec4<f32> {
    let _e38 = (*x);
    let _e41 = ((_e38 * 8.888888f) + vec4(0.055555f));
    let _e42 = vec4(9f);
    return floor((_e41 - (floor((_e41 / _e42)) * _e42)));
}

fn loadCrn_u0028_vf4_u003b(x_1: ptr<function, vec4<f32>>) -> vec4<f32> {
    let _e38 = (*x_1);
    let _e41 = ((_e38 * 80f) + vec4(0.5f));
    let _e42 = vec4(9f);
    return floor((_e41 - (floor((_e41 / _e42)) * _e42)));
}

fn main_1() {
    var E: vec4<f32>;
    var crn: vec4<f32>;
    var param: vec4<f32>;
    var mid: vec4<f32>;
    var param_1: vec4<f32>;
    var fp: vec2<f32>;
    var sp: f32;
    var local: f32;
    var local_1: f32;
    var local_2: f32;
    var local_3: f32;
    var local_4: f32;
    var local_5: f32;
    var local_6: f32;
    var local_7: f32;
    var res: vec2<f32>;
    var local_8: vec2<f32>;
    var local_9: vec2<f32>;
    var local_10: vec2<f32>;
    var local_11: vec2<f32>;
    var local_12: vec2<f32>;
    var local_13: vec2<f32>;
    var local_14: vec2<f32>;

    let _e60 = vTexCoord_1;
    let _e61 = textureSample(Source, _Source_sampler, _e60);
    E = _e61;
    let _e62 = E;
    param = _e62;
    let _e63 = loadCrn_u0028_vf4_u003b((&param));
    crn = _e63;
    let _e64 = E;
    param_1 = _e64;
    let _e65 = loadMid_u0028_vf4_u003b((&param_1));
    mid = _e65;
    let _e66 = vTexCoord_1;
    let _e68 = params.SourceSize;
    fp = floor((fract((_e66 * _e68.xy)) * 3f));
    let _e75 = fp[1u];
    if (_e75 == 0f) {
        let _e78 = fp[0u];
        if (_e78 == 0f) {
            let _e81 = crn[0u];
            local_1 = _e81;
        } else {
            let _e83 = fp[0u];
            if (_e83 == 1f) {
                let _e86 = mid[0u];
                local_2 = _e86;
            } else {
                let _e88 = crn[1u];
                local_2 = _e88;
            }
            let _e89 = local_2;
            local_1 = _e89;
        }
        let _e90 = local_1;
        local = _e90;
    } else {
        let _e92 = fp[1u];
        if (_e92 == 1f) {
            let _e95 = fp[0u];
            if (_e95 == 0f) {
                let _e98 = mid[3u];
                local_4 = _e98;
            } else {
                let _e100 = fp[0u];
                if (_e100 == 1f) {
                    local_5 = 0f;
                } else {
                    let _e103 = mid[1u];
                    local_5 = _e103;
                }
                let _e104 = local_5;
                local_4 = _e104;
            }
            let _e105 = local_4;
            local_3 = _e105;
        } else {
            let _e107 = fp[0u];
            if (_e107 == 0f) {
                let _e110 = crn[3u];
                local_6 = _e110;
            } else {
                let _e112 = fp[0u];
                if (_e112 == 1f) {
                    let _e115 = mid[2u];
                    local_7 = _e115;
                } else {
                    let _e117 = crn[2u];
                    local_7 = _e117;
                }
                let _e118 = local_7;
                local_6 = _e118;
            }
            let _e119 = local_6;
            local_3 = _e119;
        }
        let _e120 = local_3;
        local = _e120;
    }
    let _e121 = local;
    sp = _e121;
    let _e122 = sp;
    if (_e122 == 0f) {
        local_8 = vec2<f32>(0f, 0f);
    } else {
        let _e124 = sp;
        if (_e124 == 1f) {
            local_9 = vec2<f32>(-1f, 0f);
        } else {
            let _e126 = sp;
            if (_e126 == 2f) {
                local_10 = vec2<f32>(-2f, 0f);
            } else {
                let _e128 = sp;
                if (_e128 == 3f) {
                    local_11 = vec2<f32>(1f, 0f);
                } else {
                    let _e130 = sp;
                    if (_e130 == 4f) {
                        local_12 = vec2<f32>(2f, 0f);
                    } else {
                        let _e132 = sp;
                        if (_e132 == 5f) {
                            local_13 = vec2<f32>(0f, -1f);
                        } else {
                            let _e134 = sp;
                            if (_e134 == 6f) {
                                local_14 = vec2<f32>(0f, -2f);
                            } else {
                                let _e136 = sp;
                                local_14 = select(vec2<f32>(0f, 2f), vec2<f32>(0f, 1f), vec2((_e136 == 7f)));
                            }
                            let _e140 = local_14;
                            local_13 = _e140;
                        }
                        let _e141 = local_13;
                        local_12 = _e141;
                    }
                    let _e142 = local_12;
                    local_11 = _e142;
                }
                let _e143 = local_11;
                local_10 = _e143;
            }
            let _e144 = local_10;
            local_9 = _e144;
        }
        let _e145 = local_9;
        local_8 = _e145;
    }
    let _e146 = local_8;
    res = _e146;
    let _e147 = vTexCoord_1;
    let _e148 = res;
    let _e150 = params.SourceSize;
    let _e154 = textureSample(refpass, _refpass_sampler, (_e147 + (_e148 / _e150.xy)));
    FragColor = _e154;
    return;
}

@fragment 
fn main(@location(0) vTexCoord: vec2<f32>) -> @location(0) vec4<f32> {
    vTexCoord_1 = vTexCoord;
    main_1();
    let _e3 = FragColor;
    return _e3;
}
