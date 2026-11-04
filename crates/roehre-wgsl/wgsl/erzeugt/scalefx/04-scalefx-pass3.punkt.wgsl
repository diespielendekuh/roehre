// ERZEUGT aus fremden MIT-Shadern bzw. roehre (MIT OR Apache-2.0) — eigen/scalefx.slangp, Pass 4 »scalefx-pass3« (Herkunft: shader/fremd/mit/).
// Übersetzt von librashader 0.12.1 (slang → SPIR-V → naga → WGSL). Nicht von Hand ändern.
diagnostic(off, derivative_uniformity);
struct Push {
    SourceSize: vec4<f32>,
    SFX_SCN: f32,
}

struct UBO {
    MVP: mat4x4<f32>,
}

@group(0) @binding(1) 
var Source: texture_2d<f32>;
var<private> vTexCoord_1: vec2<f32>;
@group(0) @binding(2) 
var<uniform> params: Push;
var<private> FragColor: vec4<f32>;
@group(0) @binding(0) 
var<uniform> global: UBO;
@group(1) @binding(1) 
var _Source_sampler: sampler;

fn loadOr_u0028_vf4_u003b(x: ptr<function, vec4<f32>>) -> vec4<bool> {
    let _e50 = (*x);
    let _e53 = ((_e50 * 1.875f) + vec4(0.0625f));
    let _e54 = vec4(2f);
    return (floor((_e53 - (floor((_e53 / _e54)) * _e54))) != vec4<f32>(0f, 0f, 0f, 0f));
}

fn loadVert_u0028_vf4_u003b(x_1: ptr<function, vec4<f32>>) -> vec4<bool> {
    let _e50 = (*x_1);
    let _e53 = ((_e50 * 3.75f) + vec4(0.125f));
    let _e54 = vec4(2f);
    return (floor((_e53 - (floor((_e53 / _e54)) * _e54))) != vec4<f32>(0f, 0f, 0f, 0f));
}

fn loadHori_u0028_vf4_u003b(x_2: ptr<function, vec4<f32>>) -> vec4<bool> {
    let _e50 = (*x_2);
    let _e53 = ((_e50 * 7.5f) + vec4(0.25f));
    let _e54 = vec4(2f);
    return (floor((_e53 - (floor((_e53 / _e54)) * _e54))) != vec4<f32>(0f, 0f, 0f, 0f));
}

fn loadCorn_u0028_vf4_u003b(x_3: ptr<function, vec4<f32>>) -> vec4<bool> {
    let _e50 = (*x_3);
    let _e53 = ((_e50 * 15f) + vec4(0.5f));
    let _e54 = vec4(2f);
    return (floor((_e53 - (floor((_e53 / _e54)) * _e54))) != vec4<f32>(0f, 0f, 0f, 0f));
}

fn main_1() {
    var E: vec4<f32>;
    var D: vec4<f32>;
    var D0_: vec4<f32>;
    var D1_: vec4<f32>;
    var F: vec4<f32>;
    var F0_: vec4<f32>;
    var F1_: vec4<f32>;
    var B: vec4<f32>;
    var B0_: vec4<f32>;
    var B1_: vec4<f32>;
    var H: vec4<f32>;
    var H0_: vec4<f32>;
    var H1_: vec4<f32>;
    var Ec: vec4<bool>;
    var param: vec4<f32>;
    var Eh: vec4<bool>;
    var param_1: vec4<f32>;
    var Ev: vec4<bool>;
    var param_2: vec4<f32>;
    var Eo: vec4<bool>;
    var param_3: vec4<f32>;
    var Dc: vec4<bool>;
    var param_4: vec4<f32>;
    var Dh: vec4<bool>;
    var param_5: vec4<f32>;
    var Do: vec4<bool>;
    var param_6: vec4<f32>;
    var D0c: vec4<bool>;
    var param_7: vec4<f32>;
    var D0h: vec4<bool>;
    var param_8: vec4<f32>;
    var D1h: vec4<bool>;
    var param_9: vec4<f32>;
    var Fc: vec4<bool>;
    var param_10: vec4<f32>;
    var Fh: vec4<bool>;
    var param_11: vec4<f32>;
    var Fo: vec4<bool>;
    var param_12: vec4<f32>;
    var F0c: vec4<bool>;
    var param_13: vec4<f32>;
    var F0h: vec4<bool>;
    var param_14: vec4<f32>;
    var F1h: vec4<bool>;
    var param_15: vec4<f32>;
    var Bc: vec4<bool>;
    var param_16: vec4<f32>;
    var Bv: vec4<bool>;
    var param_17: vec4<f32>;
    var Bo: vec4<bool>;
    var param_18: vec4<f32>;
    var B0c: vec4<bool>;
    var param_19: vec4<f32>;
    var B0v: vec4<bool>;
    var param_20: vec4<f32>;
    var B1v: vec4<bool>;
    var param_21: vec4<f32>;
    var Hc: vec4<bool>;
    var param_22: vec4<f32>;
    var Hv: vec4<bool>;
    var param_23: vec4<f32>;
    var Ho: vec4<bool>;
    var param_24: vec4<f32>;
    var H0c: vec4<bool>;
    var param_25: vec4<f32>;
    var H0v: vec4<bool>;
    var param_26: vec4<f32>;
    var H1v: vec4<bool>;
    var param_27: vec4<f32>;
    var lvl1x: bool;
    var lvl1y: bool;
    var lvl1z: bool;
    var lvl1w: bool;
    var lvl2x: vec2<bool>;
    var lvl2y: vec2<bool>;
    var lvl2z: vec2<bool>;
    var lvl2w: vec2<bool>;
    var lvl3x: vec2<bool>;
    var lvl3y: vec2<bool>;
    var lvl3z: vec2<bool>;
    var lvl3w: vec2<bool>;
    var lvl4x: vec2<bool>;
    var lvl4y: vec2<bool>;
    var lvl4z: vec2<bool>;
    var lvl4w: vec2<bool>;
    var lvl5x: vec2<bool>;
    var lvl5y: vec2<bool>;
    var lvl5z: vec2<bool>;
    var lvl5w: vec2<bool>;
    var lvl6x: vec2<bool>;
    var lvl6y: vec2<bool>;
    var lvl6z: vec2<bool>;
    var lvl6w: vec2<bool>;
    var crn: vec4<f32>;
    var local: i32;
    var local_1: i32;
    var local_2: i32;
    var local_3: i32;
    var local_4: i32;
    var local_5: i32;
    var local_6: i32;
    var local_7: i32;
    var local_8: i32;
    var local_9: i32;
    var local_10: i32;
    var local_11: i32;
    var local_12: i32;
    var local_13: i32;
    var local_14: i32;
    var local_15: i32;
    var local_16: i32;
    var local_17: i32;
    var local_18: i32;
    var local_19: i32;
    var local_20: i32;
    var local_21: i32;
    var local_22: i32;
    var local_23: i32;
    var local_24: i32;
    var local_25: i32;
    var local_26: i32;
    var local_27: i32;
    var mid: vec4<f32>;
    var local_28: i32;
    var local_29: i32;
    var local_30: i32;
    var local_31: i32;
    var local_32: i32;
    var local_33: i32;
    var local_34: i32;
    var local_35: i32;
    var local_36: i32;
    var local_37: i32;
    var local_38: i32;
    var local_39: i32;
    var local_40: i32;
    var local_41: i32;
    var local_42: i32;
    var local_43: i32;
    var local_44: i32;
    var local_45: i32;
    var local_46: i32;
    var local_47: i32;
    var local_48: i32;
    var local_49: i32;
    var local_50: i32;
    var local_51: i32;
    var local_52: i32;
    var local_53: i32;
    var local_54: i32;
    var local_55: i32;
    var phi_294_: bool;
    var phi_295_: bool;
    var phi_314_: bool;
    var phi_315_: bool;
    var phi_332_: bool;
    var phi_333_: bool;
    var phi_350_: bool;
    var phi_351_: bool;
    var phi_436_: bool;
    var phi_449_: bool;
    var phi_464_: bool;
    var phi_477_: bool;
    var phi_492_: bool;
    var phi_505_: bool;
    var phi_520_: bool;
    var phi_533_: bool;
    var phi_563_: bool;
    var phi_588_: bool;
    var phi_615_: bool;
    var phi_640_: bool;
    var phi_667_: bool;
    var phi_692_: bool;
    var phi_719_: bool;
    var phi_744_: bool;
    var phi_756_: bool;
    var phi_764_: bool;
    var phi_774_: bool;
    var phi_782_: bool;
    var phi_794_: bool;
    var phi_802_: bool;
    var phi_812_: bool;
    var phi_820_: bool;
    var phi_832_: bool;
    var phi_840_: bool;
    var phi_850_: bool;
    var phi_858_: bool;
    var phi_870_: bool;
    var phi_878_: bool;
    var phi_888_: bool;
    var phi_896_: bool;
    var phi_908_: bool;
    var phi_918_: bool;
    var phi_930_: bool;
    var phi_940_: bool;
    var phi_952_: bool;
    var phi_962_: bool;
    var phi_974_: bool;
    var phi_984_: bool;
    var phi_999_: bool;
    var phi_1008_: bool;
    var phi_1017_: bool;
    var phi_1035_: bool;
    var phi_1036_: bool;
    var phi_1047_: bool;
    var phi_1048_: bool;
    var phi_1059_: bool;
    var phi_1060_: bool;
    var phi_1124_: bool;
    var phi_1133_: bool;
    var phi_1142_: bool;
    var phi_1158_: bool;
    var phi_1159_: bool;
    var phi_1170_: bool;
    var phi_1171_: bool;
    var phi_1182_: bool;
    var phi_1183_: bool;
    var phi_1242_: bool;
    var phi_1251_: bool;
    var phi_1260_: bool;
    var phi_1276_: bool;
    var phi_1277_: bool;
    var phi_1288_: bool;
    var phi_1289_: bool;
    var phi_1300_: bool;
    var phi_1301_: bool;
    var phi_1360_: bool;
    var phi_1369_: bool;
    var phi_1378_: bool;
    var phi_1394_: bool;
    var phi_1395_: bool;
    var phi_1406_: bool;
    var phi_1407_: bool;
    var phi_1418_: bool;
    var phi_1419_: bool;
    var phi_1480_: bool;
    var phi_1489_: bool;
    var phi_1498_: bool;
    var phi_1567_: bool;
    var phi_1578_: bool;
    var phi_1579_: bool;
    var phi_1590_: bool;
    var phi_1591_: bool;
    var phi_1602_: bool;
    var phi_1603_: bool;
    var phi_1680_: bool;
    var phi_1689_: bool;
    var phi_1698_: bool;
    var phi_1767_: bool;
    var phi_1778_: bool;
    var phi_1779_: bool;
    var phi_1790_: bool;
    var phi_1791_: bool;
    var phi_1802_: bool;
    var phi_1803_: bool;

    let _e200 = vTexCoord_1;
    let _e201 = textureSample(Source, _Source_sampler, _e200, vec2<i32>(0i, 0i));
    E = _e201;
    let _e202 = vTexCoord_1;
    let _e203 = textureSample(Source, _Source_sampler, _e202, vec2<i32>(-1i, 0i));
    D = _e203;
    let _e204 = vTexCoord_1;
    let _e205 = textureSample(Source, _Source_sampler, _e204, vec2<i32>(-2i, 0i));
    D0_ = _e205;
    let _e206 = vTexCoord_1;
    let _e207 = textureSample(Source, _Source_sampler, _e206, vec2<i32>(-3i, 0i));
    D1_ = _e207;
    let _e208 = vTexCoord_1;
    let _e209 = textureSample(Source, _Source_sampler, _e208, vec2<i32>(1i, 0i));
    F = _e209;
    let _e210 = vTexCoord_1;
    let _e211 = textureSample(Source, _Source_sampler, _e210, vec2<i32>(2i, 0i));
    F0_ = _e211;
    let _e212 = vTexCoord_1;
    let _e213 = textureSample(Source, _Source_sampler, _e212, vec2<i32>(3i, 0i));
    F1_ = _e213;
    let _e214 = vTexCoord_1;
    let _e215 = textureSample(Source, _Source_sampler, _e214, vec2<i32>(0i, -1i));
    B = _e215;
    let _e216 = vTexCoord_1;
    let _e217 = textureSample(Source, _Source_sampler, _e216, vec2<i32>(0i, -2i));
    B0_ = _e217;
    let _e218 = vTexCoord_1;
    let _e219 = textureSample(Source, _Source_sampler, _e218, vec2<i32>(0i, -3i));
    B1_ = _e219;
    let _e220 = vTexCoord_1;
    let _e221 = textureSample(Source, _Source_sampler, _e220, vec2<i32>(0i, 1i));
    H = _e221;
    let _e222 = vTexCoord_1;
    let _e223 = textureSample(Source, _Source_sampler, _e222, vec2<i32>(0i, 2i));
    H0_ = _e223;
    let _e224 = vTexCoord_1;
    let _e225 = textureSample(Source, _Source_sampler, _e224, vec2<i32>(0i, 3i));
    H1_ = _e225;
    let _e226 = E;
    param = _e226;
    let _e227 = loadCorn_u0028_vf4_u003b((&param));
    Ec = _e227;
    let _e228 = E;
    param_1 = _e228;
    let _e229 = loadHori_u0028_vf4_u003b((&param_1));
    Eh = _e229;
    let _e230 = E;
    param_2 = _e230;
    let _e231 = loadVert_u0028_vf4_u003b((&param_2));
    Ev = _e231;
    let _e232 = E;
    param_3 = _e232;
    let _e233 = loadOr_u0028_vf4_u003b((&param_3));
    Eo = _e233;
    let _e234 = D;
    param_4 = _e234;
    let _e235 = loadCorn_u0028_vf4_u003b((&param_4));
    Dc = _e235;
    let _e236 = D;
    param_5 = _e236;
    let _e237 = loadHori_u0028_vf4_u003b((&param_5));
    Dh = _e237;
    let _e238 = D;
    param_6 = _e238;
    let _e239 = loadOr_u0028_vf4_u003b((&param_6));
    Do = _e239;
    let _e240 = D0_;
    param_7 = _e240;
    let _e241 = loadCorn_u0028_vf4_u003b((&param_7));
    D0c = _e241;
    let _e242 = D0_;
    param_8 = _e242;
    let _e243 = loadHori_u0028_vf4_u003b((&param_8));
    D0h = _e243;
    let _e244 = D1_;
    param_9 = _e244;
    let _e245 = loadHori_u0028_vf4_u003b((&param_9));
    D1h = _e245;
    let _e246 = F;
    param_10 = _e246;
    let _e247 = loadCorn_u0028_vf4_u003b((&param_10));
    Fc = _e247;
    let _e248 = F;
    param_11 = _e248;
    let _e249 = loadHori_u0028_vf4_u003b((&param_11));
    Fh = _e249;
    let _e250 = F;
    param_12 = _e250;
    let _e251 = loadOr_u0028_vf4_u003b((&param_12));
    Fo = _e251;
    let _e252 = F0_;
    param_13 = _e252;
    let _e253 = loadCorn_u0028_vf4_u003b((&param_13));
    F0c = _e253;
    let _e254 = F0_;
    param_14 = _e254;
    let _e255 = loadHori_u0028_vf4_u003b((&param_14));
    F0h = _e255;
    let _e256 = F1_;
    param_15 = _e256;
    let _e257 = loadHori_u0028_vf4_u003b((&param_15));
    F1h = _e257;
    let _e258 = B;
    param_16 = _e258;
    let _e259 = loadCorn_u0028_vf4_u003b((&param_16));
    Bc = _e259;
    let _e260 = B;
    param_17 = _e260;
    let _e261 = loadVert_u0028_vf4_u003b((&param_17));
    Bv = _e261;
    let _e262 = B;
    param_18 = _e262;
    let _e263 = loadOr_u0028_vf4_u003b((&param_18));
    Bo = _e263;
    let _e264 = B0_;
    param_19 = _e264;
    let _e265 = loadCorn_u0028_vf4_u003b((&param_19));
    B0c = _e265;
    let _e266 = B0_;
    param_20 = _e266;
    let _e267 = loadVert_u0028_vf4_u003b((&param_20));
    B0v = _e267;
    let _e268 = B1_;
    param_21 = _e268;
    let _e269 = loadVert_u0028_vf4_u003b((&param_21));
    B1v = _e269;
    let _e270 = H;
    param_22 = _e270;
    let _e271 = loadCorn_u0028_vf4_u003b((&param_22));
    Hc = _e271;
    let _e272 = H;
    param_23 = _e272;
    let _e273 = loadVert_u0028_vf4_u003b((&param_23));
    Hv = _e273;
    let _e274 = H;
    param_24 = _e274;
    let _e275 = loadOr_u0028_vf4_u003b((&param_24));
    Ho = _e275;
    let _e276 = H0_;
    param_25 = _e276;
    let _e277 = loadCorn_u0028_vf4_u003b((&param_25));
    H0c = _e277;
    let _e278 = H0_;
    param_26 = _e278;
    let _e279 = loadVert_u0028_vf4_u003b((&param_26));
    H0v = _e279;
    let _e280 = H1_;
    param_27 = _e280;
    let _e281 = loadVert_u0028_vf4_u003b((&param_27));
    H1v = _e281;
    let _e283 = Ec[0u];
    phi_295_ = _e283;
    if _e283 {
        let _e285 = Dc[2u];
        let _e287 = Bc[2u];
        let _e288 = (_e285 || _e287);
        phi_294_ = _e288;
        if !(_e288) {
            let _e291 = params.SFX_SCN;
            phi_294_ = (_e291 == 1f);
        }
        let _e294 = phi_294_;
        phi_295_ = _e294;
    }
    let _e296 = phi_295_;
    lvl1x = _e296;
    let _e298 = Ec[1u];
    phi_315_ = _e298;
    if _e298 {
        let _e300 = Fc[3u];
        let _e302 = Bc[3u];
        let _e303 = (_e300 || _e302);
        phi_314_ = _e303;
        if !(_e303) {
            let _e306 = params.SFX_SCN;
            phi_314_ = (_e306 == 1f);
        }
        let _e309 = phi_314_;
        phi_315_ = _e309;
    }
    let _e311 = phi_315_;
    lvl1y = _e311;
    let _e313 = Ec[2u];
    phi_333_ = _e313;
    if _e313 {
        let _e315 = Fc[0u];
        let _e317 = Hc[0u];
        let _e318 = (_e315 || _e317);
        phi_332_ = _e318;
        if !(_e318) {
            let _e321 = params.SFX_SCN;
            phi_332_ = (_e321 == 1f);
        }
        let _e324 = phi_332_;
        phi_333_ = _e324;
    }
    let _e326 = phi_333_;
    lvl1z = _e326;
    let _e328 = Ec[3u];
    phi_351_ = _e328;
    if _e328 {
        let _e330 = Dc[1u];
        let _e332 = Hc[1u];
        let _e333 = (_e330 || _e332);
        phi_350_ = _e333;
        if !(_e333) {
            let _e336 = params.SFX_SCN;
            phi_350_ = (_e336 == 1f);
        }
        let _e339 = phi_350_;
        phi_351_ = _e339;
    }
    let _e341 = phi_351_;
    lvl1w = _e341;
    let _e343 = Ec[0u];
    let _e345 = Eh[1u];
    let _e348 = Dc[2u];
    let _e351 = Ec[1u];
    let _e353 = Eh[0u];
    let _e356 = Fc[3u];
    lvl2x = vec2<bool>(((_e343 && _e345) && _e348), ((_e351 && _e353) && _e356));
    let _e360 = Ec[1u];
    let _e362 = Ev[2u];
    let _e365 = Bc[3u];
    let _e368 = Ec[2u];
    let _e370 = Ev[1u];
    let _e373 = Hc[0u];
    lvl2y = vec2<bool>(((_e360 && _e362) && _e365), ((_e368 && _e370) && _e373));
    let _e377 = Ec[3u];
    let _e379 = Eh[2u];
    let _e382 = Dc[1u];
    let _e385 = Ec[2u];
    let _e387 = Eh[3u];
    let _e390 = Fc[0u];
    lvl2z = vec2<bool>(((_e377 && _e379) && _e382), ((_e385 && _e387) && _e390));
    let _e394 = Ec[0u];
    let _e396 = Ev[3u];
    let _e399 = Bc[2u];
    let _e402 = Ec[3u];
    let _e404 = Ev[0u];
    let _e407 = Hc[1u];
    lvl2w = vec2<bool>(((_e394 && _e396) && _e399), ((_e402 && _e404) && _e407));
    let _e411 = lvl2x[1u];
    phi_436_ = _e411;
    if _e411 {
        let _e413 = Dh[1u];
        let _e415 = Dh[0u];
        phi_436_ = (_e413 && _e415);
    }
    let _e418 = phi_436_;
    let _e420 = Fh[2u];
    let _e423 = lvl2w[1u];
    phi_449_ = _e423;
    if _e423 {
        let _e425 = Bv[3u];
        let _e427 = Bv[0u];
        phi_449_ = (_e425 && _e427);
    }
    let _e430 = phi_449_;
    let _e432 = Hv[2u];
    lvl3x = vec2<bool>((_e418 && _e420), (_e430 && _e432));
    let _e436 = lvl2x[0u];
    phi_464_ = _e436;
    if _e436 {
        let _e438 = Fh[0u];
        let _e440 = Fh[1u];
        phi_464_ = (_e438 && _e440);
    }
    let _e443 = phi_464_;
    let _e445 = Dh[3u];
    let _e448 = lvl2y[1u];
    phi_477_ = _e448;
    if _e448 {
        let _e450 = Bv[2u];
        let _e452 = Bv[1u];
        phi_477_ = (_e450 && _e452);
    }
    let _e455 = phi_477_;
    let _e457 = Hv[3u];
    lvl3y = vec2<bool>((_e443 && _e445), (_e455 && _e457));
    let _e461 = lvl2z[0u];
    phi_492_ = _e461;
    if _e461 {
        let _e463 = Fh[3u];
        let _e465 = Fh[2u];
        phi_492_ = (_e463 && _e465);
    }
    let _e468 = phi_492_;
    let _e470 = Dh[0u];
    let _e473 = lvl2y[0u];
    phi_505_ = _e473;
    if _e473 {
        let _e475 = Hv[1u];
        let _e477 = Hv[2u];
        phi_505_ = (_e475 && _e477);
    }
    let _e480 = phi_505_;
    let _e482 = Bv[0u];
    lvl3z = vec2<bool>((_e468 && _e470), (_e480 && _e482));
    let _e486 = lvl2z[1u];
    phi_520_ = _e486;
    if _e486 {
        let _e488 = Dh[2u];
        let _e490 = Dh[3u];
        phi_520_ = (_e488 && _e490);
    }
    let _e493 = phi_520_;
    let _e495 = Fh[1u];
    let _e498 = lvl2w[0u];
    phi_533_ = _e498;
    if _e498 {
        let _e500 = Hv[0u];
        let _e502 = Hv[3u];
        phi_533_ = (_e500 && _e502);
    }
    let _e505 = phi_533_;
    let _e507 = Bv[1u];
    lvl3w = vec2<bool>((_e493 && _e495), (_e505 && _e507));
    let _e511 = Dc[0u];
    let _e513 = Dh[1u];
    let _e516 = Eh[0u];
    let _e519 = Eh[1u];
    let _e522 = Fh[0u];
    let _e525 = Fh[1u];
    let _e526 = (((((_e511 && _e513) && _e516) && _e519) && _e522) && _e525);
    phi_563_ = _e526;
    if _e526 {
        let _e528 = D0c[2u];
        let _e530 = D0h[3u];
        phi_563_ = (_e528 && _e530);
    }
    let _e533 = phi_563_;
    let _e535 = Bc[0u];
    let _e537 = Bv[3u];
    let _e540 = Ev[0u];
    let _e543 = Ev[3u];
    let _e546 = Hv[0u];
    let _e549 = Hv[3u];
    let _e550 = (((((_e535 && _e537) && _e540) && _e543) && _e546) && _e549);
    phi_588_ = _e550;
    if _e550 {
        let _e552 = B0c[2u];
        let _e554 = B0v[1u];
        phi_588_ = (_e552 && _e554);
    }
    let _e557 = phi_588_;
    lvl4x = vec2<bool>(_e533, _e557);
    let _e560 = Fc[1u];
    let _e562 = Fh[0u];
    let _e565 = Eh[1u];
    let _e568 = Eh[0u];
    let _e571 = Dh[1u];
    let _e574 = Dh[0u];
    let _e575 = (((((_e560 && _e562) && _e565) && _e568) && _e571) && _e574);
    phi_615_ = _e575;
    if _e575 {
        let _e577 = F0c[3u];
        let _e579 = F0h[2u];
        phi_615_ = (_e577 && _e579);
    }
    let _e582 = phi_615_;
    let _e584 = Bc[1u];
    let _e586 = Bv[2u];
    let _e589 = Ev[1u];
    let _e592 = Ev[2u];
    let _e595 = Hv[1u];
    let _e598 = Hv[2u];
    let _e599 = (((((_e584 && _e586) && _e589) && _e592) && _e595) && _e598);
    phi_640_ = _e599;
    if _e599 {
        let _e601 = B0c[3u];
        let _e603 = B0v[0u];
        phi_640_ = (_e601 && _e603);
    }
    let _e606 = phi_640_;
    lvl4y = vec2<bool>(_e582, _e606);
    let _e609 = Fc[2u];
    let _e611 = Fh[3u];
    let _e614 = Eh[2u];
    let _e617 = Eh[3u];
    let _e620 = Dh[2u];
    let _e623 = Dh[3u];
    let _e624 = (((((_e609 && _e611) && _e614) && _e617) && _e620) && _e623);
    phi_667_ = _e624;
    if _e624 {
        let _e626 = F0c[0u];
        let _e628 = F0h[1u];
        phi_667_ = (_e626 && _e628);
    }
    let _e631 = phi_667_;
    let _e633 = Hc[2u];
    let _e635 = Hv[1u];
    let _e638 = Ev[2u];
    let _e641 = Ev[1u];
    let _e644 = Bv[2u];
    let _e647 = Bv[1u];
    let _e648 = (((((_e633 && _e635) && _e638) && _e641) && _e644) && _e647);
    phi_692_ = _e648;
    if _e648 {
        let _e650 = H0c[0u];
        let _e652 = H0v[3u];
        phi_692_ = (_e650 && _e652);
    }
    let _e655 = phi_692_;
    lvl4z = vec2<bool>(_e631, _e655);
    let _e658 = Dc[3u];
    let _e660 = Dh[2u];
    let _e663 = Eh[3u];
    let _e666 = Eh[2u];
    let _e669 = Fh[3u];
    let _e672 = Fh[2u];
    let _e673 = (((((_e658 && _e660) && _e663) && _e666) && _e669) && _e672);
    phi_719_ = _e673;
    if _e673 {
        let _e675 = D0c[1u];
        let _e677 = D0h[0u];
        phi_719_ = (_e675 && _e677);
    }
    let _e680 = phi_719_;
    let _e682 = Hc[3u];
    let _e684 = Hv[0u];
    let _e687 = Ev[3u];
    let _e690 = Ev[0u];
    let _e693 = Bv[3u];
    let _e696 = Bv[0u];
    let _e697 = (((((_e682 && _e684) && _e687) && _e690) && _e693) && _e696);
    phi_744_ = _e697;
    if _e697 {
        let _e699 = H0c[1u];
        let _e701 = H0v[2u];
        phi_744_ = (_e699 && _e701);
    }
    let _e704 = phi_744_;
    lvl4w = vec2<bool>(_e680, _e704);
    let _e707 = lvl4x[0u];
    phi_756_ = _e707;
    if _e707 {
        let _e709 = F0h[0u];
        let _e711 = F0h[1u];
        phi_756_ = (_e709 && _e711);
    }
    let _e714 = phi_756_;
    phi_764_ = _e714;
    if _e714 {
        let _e716 = D1h[2u];
        let _e718 = D1h[3u];
        phi_764_ = (_e716 && _e718);
    }
    let _e721 = phi_764_;
    let _e723 = lvl4y[0u];
    phi_774_ = _e723;
    if _e723 {
        let _e725 = D0h[1u];
        let _e727 = D0h[0u];
        phi_774_ = (_e725 && _e727);
    }
    let _e730 = phi_774_;
    phi_782_ = _e730;
    if _e730 {
        let _e732 = F1h[3u];
        let _e734 = F1h[2u];
        phi_782_ = (_e732 && _e734);
    }
    let _e737 = phi_782_;
    lvl5x = vec2<bool>(_e721, _e737);
    let _e740 = lvl4y[1u];
    phi_794_ = _e740;
    if _e740 {
        let _e742 = H0v[1u];
        let _e744 = H0v[2u];
        phi_794_ = (_e742 && _e744);
    }
    let _e747 = phi_794_;
    phi_802_ = _e747;
    if _e747 {
        let _e749 = B1v[3u];
        let _e751 = B1v[0u];
        phi_802_ = (_e749 && _e751);
    }
    let _e754 = phi_802_;
    let _e756 = lvl4z[1u];
    phi_812_ = _e756;
    if _e756 {
        let _e758 = B0v[2u];
        let _e760 = B0v[1u];
        phi_812_ = (_e758 && _e760);
    }
    let _e763 = phi_812_;
    phi_820_ = _e763;
    if _e763 {
        let _e765 = H1v[0u];
        let _e767 = H1v[3u];
        phi_820_ = (_e765 && _e767);
    }
    let _e770 = phi_820_;
    lvl5y = vec2<bool>(_e754, _e770);
    let _e773 = lvl4w[0u];
    phi_832_ = _e773;
    if _e773 {
        let _e775 = F0h[3u];
        let _e777 = F0h[2u];
        phi_832_ = (_e775 && _e777);
    }
    let _e780 = phi_832_;
    phi_840_ = _e780;
    if _e780 {
        let _e782 = D1h[1u];
        let _e784 = D1h[0u];
        phi_840_ = (_e782 && _e784);
    }
    let _e787 = phi_840_;
    let _e789 = lvl4z[0u];
    phi_850_ = _e789;
    if _e789 {
        let _e791 = D0h[2u];
        let _e793 = D0h[3u];
        phi_850_ = (_e791 && _e793);
    }
    let _e796 = phi_850_;
    phi_858_ = _e796;
    if _e796 {
        let _e798 = F1h[0u];
        let _e800 = F1h[1u];
        phi_858_ = (_e798 && _e800);
    }
    let _e803 = phi_858_;
    lvl5z = vec2<bool>(_e787, _e803);
    let _e806 = lvl4x[1u];
    phi_870_ = _e806;
    if _e806 {
        let _e808 = H0v[0u];
        let _e810 = H0v[3u];
        phi_870_ = (_e808 && _e810);
    }
    let _e813 = phi_870_;
    phi_878_ = _e813;
    if _e813 {
        let _e815 = B1v[2u];
        let _e817 = B1v[1u];
        phi_878_ = (_e815 && _e817);
    }
    let _e820 = phi_878_;
    let _e822 = lvl4w[1u];
    phi_888_ = _e822;
    if _e822 {
        let _e824 = B0v[3u];
        let _e826 = B0v[0u];
        phi_888_ = (_e824 && _e826);
    }
    let _e829 = phi_888_;
    phi_896_ = _e829;
    if _e829 {
        let _e831 = H1v[1u];
        let _e833 = H1v[2u];
        phi_896_ = (_e831 && _e833);
    }
    let _e836 = phi_896_;
    lvl5w = vec2<bool>(_e820, _e836);
    let _e839 = lvl5x[1u];
    phi_908_ = _e839;
    if _e839 {
        let _e841 = D1h[1u];
        let _e843 = D1h[0u];
        phi_908_ = (_e841 && _e843);
    }
    let _e846 = phi_908_;
    let _e848 = lvl5w[1u];
    phi_918_ = _e848;
    if _e848 {
        let _e850 = B1v[3u];
        let _e852 = B1v[0u];
        phi_918_ = (_e850 && _e852);
    }
    let _e855 = phi_918_;
    lvl6x = vec2<bool>(_e846, _e855);
    let _e858 = lvl5x[0u];
    phi_930_ = _e858;
    if _e858 {
        let _e860 = F1h[0u];
        let _e862 = F1h[1u];
        phi_930_ = (_e860 && _e862);
    }
    let _e865 = phi_930_;
    let _e867 = lvl5y[1u];
    phi_940_ = _e867;
    if _e867 {
        let _e869 = B1v[2u];
        let _e871 = B1v[1u];
        phi_940_ = (_e869 && _e871);
    }
    let _e874 = phi_940_;
    lvl6y = vec2<bool>(_e865, _e874);
    let _e877 = lvl5z[0u];
    phi_952_ = _e877;
    if _e877 {
        let _e879 = F1h[3u];
        let _e881 = F1h[2u];
        phi_952_ = (_e879 && _e881);
    }
    let _e884 = phi_952_;
    let _e886 = lvl5y[0u];
    phi_962_ = _e886;
    if _e886 {
        let _e888 = H1v[1u];
        let _e890 = H1v[2u];
        phi_962_ = (_e888 && _e890);
    }
    let _e893 = phi_962_;
    lvl6z = vec2<bool>(_e884, _e893);
    let _e896 = lvl5z[1u];
    phi_974_ = _e896;
    if _e896 {
        let _e898 = D1h[2u];
        let _e900 = D1h[3u];
        phi_974_ = (_e898 && _e900);
    }
    let _e903 = phi_974_;
    let _e905 = lvl5w[0u];
    phi_984_ = _e905;
    if _e905 {
        let _e907 = H1v[0u];
        let _e909 = H1v[3u];
        phi_984_ = (_e907 && _e909);
    }
    let _e912 = phi_984_;
    lvl6w = vec2<bool>(_e903, _e912);
    let _e914 = lvl1x;
    let _e916 = Eo[0u];
    let _e917 = (_e914 && _e916);
    phi_999_ = _e917;
    if !(_e917) {
        let _e920 = lvl3x[0u];
        let _e922 = Eo[1u];
        phi_999_ = (_e920 && _e922);
    }
    let _e925 = phi_999_;
    phi_1008_ = _e925;
    if !(_e925) {
        let _e928 = lvl4x[0u];
        let _e930 = Do[0u];
        phi_1008_ = (_e928 && _e930);
    }
    let _e933 = phi_1008_;
    phi_1017_ = _e933;
    if !(_e933) {
        let _e936 = lvl6x[0u];
        let _e938 = Fo[1u];
        phi_1017_ = (_e936 && _e938);
    }
    let _e941 = phi_1017_;
    if _e941 {
        local = 5i;
    } else {
        let _e942 = lvl1x;
        phi_1036_ = _e942;
        if !(_e942) {
            let _e945 = lvl3x[1u];
            phi_1035_ = _e945;
            if _e945 {
                let _e947 = Eo[3u];
                phi_1035_ = !(_e947);
            }
            let _e950 = phi_1035_;
            phi_1036_ = _e950;
        }
        let _e952 = phi_1036_;
        phi_1048_ = _e952;
        if !(_e952) {
            let _e955 = lvl4x[1u];
            phi_1047_ = _e955;
            if _e955 {
                let _e957 = Bo[0u];
                phi_1047_ = !(_e957);
            }
            let _e960 = phi_1047_;
            phi_1048_ = _e960;
        }
        let _e962 = phi_1048_;
        phi_1060_ = _e962;
        if !(_e962) {
            let _e965 = lvl6x[1u];
            phi_1059_ = _e965;
            if _e965 {
                let _e967 = Ho[3u];
                phi_1059_ = !(_e967);
            }
            let _e970 = phi_1059_;
            phi_1060_ = _e970;
        }
        let _e972 = phi_1060_;
        if _e972 {
            local_1 = 1i;
        } else {
            let _e974 = lvl3x[0u];
            if _e974 {
                local_2 = 3i;
            } else {
                let _e976 = lvl3x[1u];
                if _e976 {
                    local_3 = 7i;
                } else {
                    let _e978 = lvl4x[0u];
                    if _e978 {
                        local_4 = 2i;
                    } else {
                        let _e980 = lvl4x[1u];
                        if _e980 {
                            local_5 = 6i;
                        } else {
                            let _e982 = lvl6x[0u];
                            if _e982 {
                                local_6 = 4i;
                            } else {
                                let _e984 = lvl6x[1u];
                                local_6 = select(0i, 8i, _e984);
                            }
                            let _e986 = local_6;
                            local_5 = _e986;
                        }
                        let _e987 = local_5;
                        local_4 = _e987;
                    }
                    let _e988 = local_4;
                    local_3 = _e988;
                }
                let _e989 = local_3;
                local_2 = _e989;
            }
            let _e990 = local_2;
            local_1 = _e990;
        }
        let _e991 = local_1;
        local = _e991;
    }
    let _e992 = local;
    crn[0u] = f32(_e992);
    let _e995 = lvl1y;
    let _e997 = Eo[1u];
    let _e998 = (_e995 && _e997);
    phi_1124_ = _e998;
    if !(_e998) {
        let _e1001 = lvl3y[0u];
        let _e1003 = Eo[0u];
        phi_1124_ = (_e1001 && _e1003);
    }
    let _e1006 = phi_1124_;
    phi_1133_ = _e1006;
    if !(_e1006) {
        let _e1009 = lvl4y[0u];
        let _e1011 = Fo[1u];
        phi_1133_ = (_e1009 && _e1011);
    }
    let _e1014 = phi_1133_;
    phi_1142_ = _e1014;
    if !(_e1014) {
        let _e1017 = lvl6y[0u];
        let _e1019 = Do[0u];
        phi_1142_ = (_e1017 && _e1019);
    }
    let _e1022 = phi_1142_;
    if _e1022 {
        local_7 = 5i;
    } else {
        let _e1023 = lvl1y;
        phi_1159_ = _e1023;
        if !(_e1023) {
            let _e1026 = lvl3y[1u];
            phi_1158_ = _e1026;
            if _e1026 {
                let _e1028 = Eo[2u];
                phi_1158_ = !(_e1028);
            }
            let _e1031 = phi_1158_;
            phi_1159_ = _e1031;
        }
        let _e1033 = phi_1159_;
        phi_1171_ = _e1033;
        if !(_e1033) {
            let _e1036 = lvl4y[1u];
            phi_1170_ = _e1036;
            if _e1036 {
                let _e1038 = Bo[1u];
                phi_1170_ = !(_e1038);
            }
            let _e1041 = phi_1170_;
            phi_1171_ = _e1041;
        }
        let _e1043 = phi_1171_;
        phi_1183_ = _e1043;
        if !(_e1043) {
            let _e1046 = lvl6y[1u];
            phi_1182_ = _e1046;
            if _e1046 {
                let _e1048 = Ho[2u];
                phi_1182_ = !(_e1048);
            }
            let _e1051 = phi_1182_;
            phi_1183_ = _e1051;
        }
        let _e1053 = phi_1183_;
        if _e1053 {
            local_8 = 3i;
        } else {
            let _e1055 = lvl3y[0u];
            if _e1055 {
                local_9 = 1i;
            } else {
                let _e1057 = lvl3y[1u];
                if _e1057 {
                    local_10 = 7i;
                } else {
                    let _e1059 = lvl4y[0u];
                    if _e1059 {
                        local_11 = 4i;
                    } else {
                        let _e1061 = lvl4y[1u];
                        if _e1061 {
                            local_12 = 6i;
                        } else {
                            let _e1063 = lvl6y[0u];
                            if _e1063 {
                                local_13 = 2i;
                            } else {
                                let _e1065 = lvl6y[1u];
                                local_13 = select(0i, 8i, _e1065);
                            }
                            let _e1067 = local_13;
                            local_12 = _e1067;
                        }
                        let _e1068 = local_12;
                        local_11 = _e1068;
                    }
                    let _e1069 = local_11;
                    local_10 = _e1069;
                }
                let _e1070 = local_10;
                local_9 = _e1070;
            }
            let _e1071 = local_9;
            local_8 = _e1071;
        }
        let _e1072 = local_8;
        local_7 = _e1072;
    }
    let _e1073 = local_7;
    crn[1u] = f32(_e1073);
    let _e1076 = lvl1z;
    let _e1078 = Eo[2u];
    let _e1079 = (_e1076 && _e1078);
    phi_1242_ = _e1079;
    if !(_e1079) {
        let _e1082 = lvl3z[0u];
        let _e1084 = Eo[3u];
        phi_1242_ = (_e1082 && _e1084);
    }
    let _e1087 = phi_1242_;
    phi_1251_ = _e1087;
    if !(_e1087) {
        let _e1090 = lvl4z[0u];
        let _e1092 = Fo[2u];
        phi_1251_ = (_e1090 && _e1092);
    }
    let _e1095 = phi_1251_;
    phi_1260_ = _e1095;
    if !(_e1095) {
        let _e1098 = lvl6z[0u];
        let _e1100 = Do[3u];
        phi_1260_ = (_e1098 && _e1100);
    }
    let _e1103 = phi_1260_;
    if _e1103 {
        local_14 = 7i;
    } else {
        let _e1104 = lvl1z;
        phi_1277_ = _e1104;
        if !(_e1104) {
            let _e1107 = lvl3z[1u];
            phi_1276_ = _e1107;
            if _e1107 {
                let _e1109 = Eo[1u];
                phi_1276_ = !(_e1109);
            }
            let _e1112 = phi_1276_;
            phi_1277_ = _e1112;
        }
        let _e1114 = phi_1277_;
        phi_1289_ = _e1114;
        if !(_e1114) {
            let _e1117 = lvl4z[1u];
            phi_1288_ = _e1117;
            if _e1117 {
                let _e1119 = Ho[2u];
                phi_1288_ = !(_e1119);
            }
            let _e1122 = phi_1288_;
            phi_1289_ = _e1122;
        }
        let _e1124 = phi_1289_;
        phi_1301_ = _e1124;
        if !(_e1124) {
            let _e1127 = lvl6z[1u];
            phi_1300_ = _e1127;
            if _e1127 {
                let _e1129 = Bo[1u];
                phi_1300_ = !(_e1129);
            }
            let _e1132 = phi_1300_;
            phi_1301_ = _e1132;
        }
        let _e1134 = phi_1301_;
        if _e1134 {
            local_15 = 3i;
        } else {
            let _e1136 = lvl3z[0u];
            if _e1136 {
                local_16 = 1i;
            } else {
                let _e1138 = lvl3z[1u];
                if _e1138 {
                    local_17 = 5i;
                } else {
                    let _e1140 = lvl4z[0u];
                    if _e1140 {
                        local_18 = 4i;
                    } else {
                        let _e1142 = lvl4z[1u];
                        if _e1142 {
                            local_19 = 8i;
                        } else {
                            let _e1144 = lvl6z[0u];
                            if _e1144 {
                                local_20 = 2i;
                            } else {
                                let _e1146 = lvl6z[1u];
                                local_20 = select(0i, 6i, _e1146);
                            }
                            let _e1148 = local_20;
                            local_19 = _e1148;
                        }
                        let _e1149 = local_19;
                        local_18 = _e1149;
                    }
                    let _e1150 = local_18;
                    local_17 = _e1150;
                }
                let _e1151 = local_17;
                local_16 = _e1151;
            }
            let _e1152 = local_16;
            local_15 = _e1152;
        }
        let _e1153 = local_15;
        local_14 = _e1153;
    }
    let _e1154 = local_14;
    crn[2u] = f32(_e1154);
    let _e1157 = lvl1w;
    let _e1159 = Eo[3u];
    let _e1160 = (_e1157 && _e1159);
    phi_1360_ = _e1160;
    if !(_e1160) {
        let _e1163 = lvl3w[0u];
        let _e1165 = Eo[2u];
        phi_1360_ = (_e1163 && _e1165);
    }
    let _e1168 = phi_1360_;
    phi_1369_ = _e1168;
    if !(_e1168) {
        let _e1171 = lvl4w[0u];
        let _e1173 = Do[3u];
        phi_1369_ = (_e1171 && _e1173);
    }
    let _e1176 = phi_1369_;
    phi_1378_ = _e1176;
    if !(_e1176) {
        let _e1179 = lvl6w[0u];
        let _e1181 = Fo[2u];
        phi_1378_ = (_e1179 && _e1181);
    }
    let _e1184 = phi_1378_;
    if _e1184 {
        local_21 = 7i;
    } else {
        let _e1185 = lvl1w;
        phi_1395_ = _e1185;
        if !(_e1185) {
            let _e1188 = lvl3w[1u];
            phi_1394_ = _e1188;
            if _e1188 {
                let _e1190 = Eo[0u];
                phi_1394_ = !(_e1190);
            }
            let _e1193 = phi_1394_;
            phi_1395_ = _e1193;
        }
        let _e1195 = phi_1395_;
        phi_1407_ = _e1195;
        if !(_e1195) {
            let _e1198 = lvl4w[1u];
            phi_1406_ = _e1198;
            if _e1198 {
                let _e1200 = Ho[3u];
                phi_1406_ = !(_e1200);
            }
            let _e1203 = phi_1406_;
            phi_1407_ = _e1203;
        }
        let _e1205 = phi_1407_;
        phi_1419_ = _e1205;
        if !(_e1205) {
            let _e1208 = lvl6w[1u];
            phi_1418_ = _e1208;
            if _e1208 {
                let _e1210 = Bo[0u];
                phi_1418_ = !(_e1210);
            }
            let _e1213 = phi_1418_;
            phi_1419_ = _e1213;
        }
        let _e1215 = phi_1419_;
        if _e1215 {
            local_22 = 1i;
        } else {
            let _e1217 = lvl3w[0u];
            if _e1217 {
                local_23 = 3i;
            } else {
                let _e1219 = lvl3w[1u];
                if _e1219 {
                    local_24 = 5i;
                } else {
                    let _e1221 = lvl4w[0u];
                    if _e1221 {
                        local_25 = 2i;
                    } else {
                        let _e1223 = lvl4w[1u];
                        if _e1223 {
                            local_26 = 8i;
                        } else {
                            let _e1225 = lvl6w[0u];
                            if _e1225 {
                                local_27 = 4i;
                            } else {
                                let _e1227 = lvl6w[1u];
                                local_27 = select(0i, 6i, _e1227);
                            }
                            let _e1229 = local_27;
                            local_26 = _e1229;
                        }
                        let _e1230 = local_26;
                        local_25 = _e1230;
                    }
                    let _e1231 = local_25;
                    local_24 = _e1231;
                }
                let _e1232 = local_24;
                local_23 = _e1232;
            }
            let _e1233 = local_23;
            local_22 = _e1233;
        }
        let _e1234 = local_22;
        local_21 = _e1234;
    }
    let _e1235 = local_21;
    crn[3u] = f32(_e1235);
    let _e1239 = lvl2x[0u];
    let _e1241 = Eo[0u];
    let _e1242 = (_e1239 && _e1241);
    phi_1480_ = _e1242;
    if !(_e1242) {
        let _e1245 = lvl2x[1u];
        let _e1247 = Eo[1u];
        phi_1480_ = (_e1245 && _e1247);
    }
    let _e1250 = phi_1480_;
    phi_1489_ = _e1250;
    if !(_e1250) {
        let _e1253 = lvl5x[0u];
        let _e1255 = Do[0u];
        phi_1489_ = (_e1253 && _e1255);
    }
    let _e1258 = phi_1489_;
    phi_1498_ = _e1258;
    if !(_e1258) {
        let _e1261 = lvl5x[1u];
        let _e1263 = Fo[1u];
        phi_1498_ = (_e1261 && _e1263);
    }
    let _e1266 = phi_1498_;
    if _e1266 {
        local_28 = 5i;
    } else {
        let _e1268 = lvl2x[0u];
        if _e1268 {
            local_29 = 1i;
        } else {
            let _e1270 = lvl2x[1u];
            if _e1270 {
                local_30 = 3i;
            } else {
                let _e1272 = lvl5x[0u];
                if _e1272 {
                    local_31 = 2i;
                } else {
                    let _e1274 = lvl5x[1u];
                    if _e1274 {
                        local_32 = 4i;
                    } else {
                        let _e1276 = Ec[0u];
                        let _e1278 = Dc[2u];
                        let _e1281 = Ec[1u];
                        let _e1284 = Fc[3u];
                        if (((_e1276 && _e1278) && _e1281) && _e1284) {
                            let _e1287 = Eo[0u];
                            if _e1287 {
                                let _e1289 = Eo[1u];
                                local_34 = select(3i, 5i, _e1289);
                            } else {
                                local_34 = 1i;
                            }
                            let _e1291 = local_34;
                            local_33 = _e1291;
                        } else {
                            local_33 = 0i;
                        }
                        let _e1292 = local_33;
                        local_32 = _e1292;
                    }
                    let _e1293 = local_32;
                    local_31 = _e1293;
                }
                let _e1294 = local_31;
                local_30 = _e1294;
            }
            let _e1295 = local_30;
            local_29 = _e1295;
        }
        let _e1296 = local_29;
        local_28 = _e1296;
    }
    let _e1297 = local_28;
    mid[0u] = f32(_e1297);
    let _e1301 = lvl2y[0u];
    phi_1567_ = _e1301;
    if _e1301 {
        let _e1303 = Eo[1u];
        phi_1567_ = !(_e1303);
    }
    let _e1306 = phi_1567_;
    phi_1579_ = _e1306;
    if !(_e1306) {
        let _e1309 = lvl2y[1u];
        phi_1578_ = _e1309;
        if _e1309 {
            let _e1311 = Eo[2u];
            phi_1578_ = !(_e1311);
        }
        let _e1314 = phi_1578_;
        phi_1579_ = _e1314;
    }
    let _e1316 = phi_1579_;
    phi_1591_ = _e1316;
    if !(_e1316) {
        let _e1319 = lvl5y[0u];
        phi_1590_ = _e1319;
        if _e1319 {
            let _e1321 = Bo[1u];
            phi_1590_ = !(_e1321);
        }
        let _e1324 = phi_1590_;
        phi_1591_ = _e1324;
    }
    let _e1326 = phi_1591_;
    phi_1603_ = _e1326;
    if !(_e1326) {
        let _e1329 = lvl5y[1u];
        phi_1602_ = _e1329;
        if _e1329 {
            let _e1331 = Ho[2u];
            phi_1602_ = !(_e1331);
        }
        let _e1334 = phi_1602_;
        phi_1603_ = _e1334;
    }
    let _e1336 = phi_1603_;
    if _e1336 {
        local_35 = 3i;
    } else {
        let _e1338 = lvl2y[0u];
        if _e1338 {
            local_36 = 5i;
        } else {
            let _e1340 = lvl2y[1u];
            if _e1340 {
                local_37 = 7i;
            } else {
                let _e1342 = lvl5y[0u];
                if _e1342 {
                    local_38 = 6i;
                } else {
                    let _e1344 = lvl5y[1u];
                    if _e1344 {
                        local_39 = 8i;
                    } else {
                        let _e1346 = Ec[1u];
                        let _e1348 = Bc[3u];
                        let _e1351 = Ec[2u];
                        let _e1354 = Hc[0u];
                        if (((_e1346 && _e1348) && _e1351) && _e1354) {
                            let _e1357 = Eo[1u];
                            if !(_e1357) {
                                let _e1360 = Eo[2u];
                                local_41 = select(7i, 3i, !(_e1360));
                            } else {
                                local_41 = 5i;
                            }
                            let _e1363 = local_41;
                            local_40 = _e1363;
                        } else {
                            local_40 = 0i;
                        }
                        let _e1364 = local_40;
                        local_39 = _e1364;
                    }
                    let _e1365 = local_39;
                    local_38 = _e1365;
                }
                let _e1366 = local_38;
                local_37 = _e1366;
            }
            let _e1367 = local_37;
            local_36 = _e1367;
        }
        let _e1368 = local_36;
        local_35 = _e1368;
    }
    let _e1369 = local_35;
    mid[1u] = f32(_e1369);
    let _e1373 = lvl2z[0u];
    let _e1375 = Eo[3u];
    let _e1376 = (_e1373 && _e1375);
    phi_1680_ = _e1376;
    if !(_e1376) {
        let _e1379 = lvl2z[1u];
        let _e1381 = Eo[2u];
        phi_1680_ = (_e1379 && _e1381);
    }
    let _e1384 = phi_1680_;
    phi_1689_ = _e1384;
    if !(_e1384) {
        let _e1387 = lvl5z[0u];
        let _e1389 = Do[3u];
        phi_1689_ = (_e1387 && _e1389);
    }
    let _e1392 = phi_1689_;
    phi_1698_ = _e1392;
    if !(_e1392) {
        let _e1395 = lvl5z[1u];
        let _e1397 = Fo[2u];
        phi_1698_ = (_e1395 && _e1397);
    }
    let _e1400 = phi_1698_;
    if _e1400 {
        local_42 = 7i;
    } else {
        let _e1402 = lvl2z[0u];
        if _e1402 {
            local_43 = 1i;
        } else {
            let _e1404 = lvl2z[1u];
            if _e1404 {
                local_44 = 3i;
            } else {
                let _e1406 = lvl5z[0u];
                if _e1406 {
                    local_45 = 2i;
                } else {
                    let _e1408 = lvl5z[1u];
                    if _e1408 {
                        local_46 = 4i;
                    } else {
                        let _e1410 = Ec[2u];
                        let _e1412 = Fc[0u];
                        let _e1415 = Ec[3u];
                        let _e1418 = Dc[1u];
                        if (((_e1410 && _e1412) && _e1415) && _e1418) {
                            let _e1421 = Eo[2u];
                            if _e1421 {
                                let _e1423 = Eo[3u];
                                local_48 = select(1i, 7i, _e1423);
                            } else {
                                local_48 = 3i;
                            }
                            let _e1425 = local_48;
                            local_47 = _e1425;
                        } else {
                            local_47 = 0i;
                        }
                        let _e1426 = local_47;
                        local_46 = _e1426;
                    }
                    let _e1427 = local_46;
                    local_45 = _e1427;
                }
                let _e1428 = local_45;
                local_44 = _e1428;
            }
            let _e1429 = local_44;
            local_43 = _e1429;
        }
        let _e1430 = local_43;
        local_42 = _e1430;
    }
    let _e1431 = local_42;
    mid[2u] = f32(_e1431);
    let _e1435 = lvl2w[0u];
    phi_1767_ = _e1435;
    if _e1435 {
        let _e1437 = Eo[0u];
        phi_1767_ = !(_e1437);
    }
    let _e1440 = phi_1767_;
    phi_1779_ = _e1440;
    if !(_e1440) {
        let _e1443 = lvl2w[1u];
        phi_1778_ = _e1443;
        if _e1443 {
            let _e1445 = Eo[3u];
            phi_1778_ = !(_e1445);
        }
        let _e1448 = phi_1778_;
        phi_1779_ = _e1448;
    }
    let _e1450 = phi_1779_;
    phi_1791_ = _e1450;
    if !(_e1450) {
        let _e1453 = lvl5w[0u];
        phi_1790_ = _e1453;
        if _e1453 {
            let _e1455 = Bo[0u];
            phi_1790_ = !(_e1455);
        }
        let _e1458 = phi_1790_;
        phi_1791_ = _e1458;
    }
    let _e1460 = phi_1791_;
    phi_1803_ = _e1460;
    if !(_e1460) {
        let _e1463 = lvl5w[1u];
        phi_1802_ = _e1463;
        if _e1463 {
            let _e1465 = Ho[3u];
            phi_1802_ = !(_e1465);
        }
        let _e1468 = phi_1802_;
        phi_1803_ = _e1468;
    }
    let _e1470 = phi_1803_;
    if _e1470 {
        local_49 = 1i;
    } else {
        let _e1472 = lvl2w[0u];
        if _e1472 {
            local_50 = 5i;
        } else {
            let _e1474 = lvl2w[1u];
            if _e1474 {
                local_51 = 7i;
            } else {
                let _e1476 = lvl5w[0u];
                if _e1476 {
                    local_52 = 6i;
                } else {
                    let _e1478 = lvl5w[1u];
                    if _e1478 {
                        local_53 = 8i;
                    } else {
                        let _e1480 = Ec[3u];
                        let _e1482 = Hc[1u];
                        let _e1485 = Ec[0u];
                        let _e1488 = Bc[2u];
                        if (((_e1480 && _e1482) && _e1485) && _e1488) {
                            let _e1491 = Eo[3u];
                            if !(_e1491) {
                                let _e1494 = Eo[0u];
                                local_55 = select(5i, 1i, !(_e1494));
                            } else {
                                local_55 = 7i;
                            }
                            let _e1497 = local_55;
                            local_54 = _e1497;
                        } else {
                            local_54 = 0i;
                        }
                        let _e1498 = local_54;
                        local_53 = _e1498;
                    }
                    let _e1499 = local_53;
                    local_52 = _e1499;
                }
                let _e1500 = local_52;
                local_51 = _e1500;
            }
            let _e1501 = local_51;
            local_50 = _e1501;
        }
        let _e1502 = local_50;
        local_49 = _e1502;
    }
    let _e1503 = local_49;
    mid[3u] = f32(_e1503);
    let _e1506 = crn;
    let _e1507 = mid;
    FragColor = ((_e1506 + (_e1507 * 9f)) / vec4(80f));
    return;
}

@fragment 
fn main(@location(0) vTexCoord: vec2<f32>) -> @location(0) vec4<f32> {
    vTexCoord_1 = vTexCoord;
    main_1();
    let _e3 = FragColor;
    return _e3;
}
