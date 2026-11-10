// ERZEUGT aus fremden MIT-Shadern bzw. roehre (MIT OR Apache-2.0) — eigen/omniscale.slangp, Pass 0 »omniscale« (Herkunft: shader/fremd/mit/).
// Übersetzt von librashader 0.12.1 (slang → SPIR-V → naga → WGSL). Nicht von Hand ändern.
diagnostic(off, derivative_uniformity);
struct Push {
    SourceSize: vec4<f32>,
    OriginalSize: vec4<f32>,
    OutputSize: vec4<f32>,
    FrameCount: u32,
}

struct UBO {
    MVP: mat4x4<f32>,
}

@group(0) @binding(3) 
var<uniform> params: Push;
var<private> FragColor: vec4<f32>;
@group(0) @binding(2) 
var Source: texture_2d<f32>;
var<private> vTexCoord_1: vec2<f32>;
@group(0) @binding(0) 
var<uniform> global: UBO;
@group(1) @binding(2) 
var _Source_sampler: sampler;

fn rgb_to_hq_colospace_u0028_vf4_u003b(rgb: ptr<function, vec4<f32>>) -> vec3<f32> {
    let _e95 = (*rgb)[0u];
    let _e98 = (*rgb)[1u];
    let _e102 = (*rgb)[2u];
    let _e106 = (*rgb)[0u];
    let _e109 = (*rgb)[1u];
    let _e113 = (*rgb)[2u];
    let _e117 = (*rgb)[0u];
    let _e120 = (*rgb)[1u];
    let _e124 = (*rgb)[2u];
    return vec3<f32>((((0.25f * _e95) + (0.25f * _e98)) + (0.25f * _e102)), (((0.25f * _e106) - (0f * _e109)) - (0.25f * _e113)), (((-0.125f * _e117) + (0.25f * _e120)) - (0.125f * _e124)));
}

fn is_different_u0028_vf4_u003b_vf4_u003b(a: ptr<function, vec4<f32>>, b: ptr<function, vec4<f32>>) -> bool {
    var diff: vec3<f32>;
    var param: vec4<f32>;
    var param_1: vec4<f32>;
    var phi_95_: bool;
    var phi_103_: bool;

    let _e98 = (*a);
    param = _e98;
    let _e99 = rgb_to_hq_colospace_u0028_vf4_u003b((&param));
    let _e100 = (*b);
    param_1 = _e100;
    let _e101 = rgb_to_hq_colospace_u0028_vf4_u003b((&param_1));
    diff = abs((_e99 - _e101));
    let _e105 = diff[0u];
    let _e106 = (_e105 > 0.125f);
    phi_95_ = _e106;
    if !(_e106) {
        let _e109 = diff[1u];
        phi_95_ = (_e109 > 0.027f);
    }
    let _e112 = phi_95_;
    phi_103_ = _e112;
    if !(_e112) {
        let _e115 = diff[2u];
        phi_103_ = (_e115 > 0.031f);
    }
    let _e118 = phi_103_;
    return _e118;
}

fn scale_u0028_s21_u003b_vf2_u003b(image: texture_2d<f32>, param_2: sampler, coord: ptr<function, vec2<f32>>) -> vec4<f32> {
    var o: vec2<f32>;
    var texCoord: vec2<f32>;
    var p: vec2<f32>;
    var w0_: vec4<f32>;
    var w1_: vec4<f32>;
    var w2_: vec4<f32>;
    var w3_: vec4<f32>;
    var w4_: vec4<f32>;
    var w5_: vec4<f32>;
    var w6_: vec4<f32>;
    var w7_: vec4<f32>;
    var w8_: vec4<f32>;
    var pattern: i32;
    var param_3: vec4<f32>;
    var param_4: vec4<f32>;
    var param_5: vec4<f32>;
    var param_6: vec4<f32>;
    var param_7: vec4<f32>;
    var param_8: vec4<f32>;
    var param_9: vec4<f32>;
    var param_10: vec4<f32>;
    var param_11: vec4<f32>;
    var param_12: vec4<f32>;
    var param_13: vec4<f32>;
    var param_14: vec4<f32>;
    var param_15: vec4<f32>;
    var param_16: vec4<f32>;
    var param_17: vec4<f32>;
    var param_18: vec4<f32>;
    var param_19: vec4<f32>;
    var param_20: vec4<f32>;
    var param_21: vec4<f32>;
    var param_22: vec4<f32>;
    var param_23: vec4<f32>;
    var param_24: vec4<f32>;
    var param_25: vec4<f32>;
    var param_26: vec4<f32>;
    var dist: f32;
    var pixel_size: f32;
    var param_27: vec4<f32>;
    var param_28: vec4<f32>;
    var param_29: vec4<f32>;
    var param_30: vec4<f32>;
    var r: vec4<f32>;
    var dist_1: f32;
    var pixel_size_1: f32;
    var r_1: vec4<f32>;
    var dist_2: f32;
    var pixel_size_2: f32;
    var r_2: vec4<f32>;
    var dist_3: f32;
    var pixel_size_3: f32;
    var param_31: vec4<f32>;
    var param_32: vec4<f32>;
    var param_33: vec4<f32>;
    var param_34: vec4<f32>;
    var r_3: vec4<f32>;
    var dist_4: f32;
    var pixel_size_4: f32;
    var param_35: vec4<f32>;
    var param_36: vec4<f32>;
    var param_37: vec4<f32>;
    var param_38: vec4<f32>;
    var r_4: vec4<f32>;
    var dist_5: f32;
    var pixel_size_5: f32;
    var param_39: vec4<f32>;
    var param_40: vec4<f32>;
    var param_41: vec4<f32>;
    var param_42: vec4<f32>;
    var r_5: vec4<f32>;
    var dist_6: f32;
    var pixel_size_6: f32;
    var x0_: vec4<f32>;
    var x1_: vec4<f32>;
    var x2_: vec4<f32>;
    var x3_: vec4<f32>;
    var x4_: vec4<f32>;
    var x5_: vec4<f32>;
    var x6_: vec4<f32>;
    var param_43: vec4<f32>;
    var param_44: vec4<f32>;
    var param_45: vec4<f32>;
    var param_46: vec4<f32>;
    var param_47: vec4<f32>;
    var param_48: vec4<f32>;
    var param_49: vec4<f32>;
    var param_50: vec4<f32>;
    var param_51: vec4<f32>;
    var param_52: vec4<f32>;
    var param_53: vec4<f32>;
    var param_54: vec4<f32>;
    var param_55: vec4<f32>;
    var param_56: vec4<f32>;
    var diagonal_bias: i32;
    var r_6: vec4<f32>;
    var phi_334_: bool;
    var phi_342_: bool;
    var phi_365_: bool;
    var phi_373_: bool;
    var phi_396_: bool;
    var phi_404_: bool;
    var phi_412_: bool;
    var phi_430_: bool;
    var phi_438_: bool;
    var phi_447_: bool;
    var phi_456_: bool;
    var phi_464_: bool;
    var phi_472_: bool;
    var phi_481_: bool;
    var phi_489_: bool;
    var phi_497_: bool;
    var phi_505_: bool;
    var phi_513_: bool;
    var phi_521_: bool;
    var phi_529_: bool;
    var phi_652_: bool;
    var phi_717_: bool;
    var phi_785_: bool;
    var phi_859_: bool;
    var phi_903_: bool;
    var phi_969_: bool;
    var phi_1018_: bool;
    var phi_1087_: bool;
    var phi_1096_: bool;
    var phi_1104_: bool;
    var phi_1127_: bool;
    var phi_1136_: bool;
    var phi_1144_: bool;
    var phi_1166_: bool;
    var phi_1174_: bool;
    var phi_1182_: bool;
    var phi_1190_: bool;
    var phi_1198_: bool;
    var phi_1222_: bool;
    var phi_1229_: bool;
    var phi_1237_: bool;
    var phi_1245_: bool;
    var phi_1252_: bool;
    var phi_1259_: bool;
    var phi_1266_: bool;
    var phi_1308_: bool;

    let _e193 = params.SourceSize;
    o = (vec2(1f) / _e193.xy);
    let _e197 = (*coord);
    texCoord = _e197;
    let _e198 = texCoord;
    let _e200 = params.SourceSize;
    p = fract((_e198 * _e200.xy));
    let _e205 = p[0u];
    if (_e205 > 0.5f) {
        let _e208 = o[0u];
        o[0u] = -(_e208);
        let _e212 = p[0u];
        p[0u] = (1f - _e212);
    }
    let _e216 = p[1u];
    if (_e216 > 0.5f) {
        let _e219 = o[1u];
        o[1u] = -(_e219);
        let _e223 = p[1u];
        p[1u] = (1f - _e223);
    }
    let _e226 = texCoord;
    let _e228 = o[0u];
    let _e231 = o[1u];
    let _e235 = textureSample(image, param_2, (_e226 + vec2<f32>(-(_e228), -(_e231))));
    w0_ = _e235;
    let _e236 = texCoord;
    let _e238 = o[1u];
    let _e242 = textureSample(image, param_2, (_e236 + vec2<f32>(0f, -(_e238))));
    w1_ = _e242;
    let _e243 = texCoord;
    let _e245 = o[0u];
    let _e247 = o[1u];
    let _e251 = textureSample(image, param_2, (_e243 + vec2<f32>(_e245, -(_e247))));
    w2_ = _e251;
    let _e252 = texCoord;
    let _e254 = o[0u];
    let _e258 = textureSample(image, param_2, (_e252 + vec2<f32>(-(_e254), 0f)));
    w3_ = _e258;
    let _e259 = texCoord;
    let _e261 = textureSample(image, param_2, (_e259 + vec2<f32>(0f, 0f)));
    w4_ = _e261;
    let _e262 = texCoord;
    let _e264 = o[0u];
    let _e267 = textureSample(image, param_2, (_e262 + vec2<f32>(_e264, 0f)));
    w5_ = _e267;
    let _e268 = texCoord;
    let _e270 = o[0u];
    let _e273 = o[1u];
    let _e276 = textureSample(image, param_2, (_e268 + vec2<f32>(-(_e270), _e273)));
    w6_ = _e276;
    let _e277 = texCoord;
    let _e279 = o[1u];
    let _e282 = textureSample(image, param_2, (_e277 + vec2<f32>(0f, _e279)));
    w7_ = _e282;
    let _e283 = texCoord;
    let _e285 = o[0u];
    let _e287 = o[1u];
    let _e290 = textureSample(image, param_2, (_e283 + vec2<f32>(_e285, _e287)));
    w8_ = _e290;
    pattern = 0i;
    let _e291 = w0_;
    param_3 = _e291;
    let _e292 = w4_;
    param_4 = _e292;
    let _e293 = is_different_u0028_vf4_u003b_vf4_u003b((&param_3), (&param_4));
    if _e293 {
        let _e294 = pattern;
        pattern = (_e294 | 1i);
    }
    let _e296 = w1_;
    param_5 = _e296;
    let _e297 = w4_;
    param_6 = _e297;
    let _e298 = is_different_u0028_vf4_u003b_vf4_u003b((&param_5), (&param_6));
    if _e298 {
        let _e299 = pattern;
        pattern = (_e299 | 2i);
    }
    let _e301 = w2_;
    param_7 = _e301;
    let _e302 = w4_;
    param_8 = _e302;
    let _e303 = is_different_u0028_vf4_u003b_vf4_u003b((&param_7), (&param_8));
    if _e303 {
        let _e304 = pattern;
        pattern = (_e304 | 4i);
    }
    let _e306 = w3_;
    param_9 = _e306;
    let _e307 = w4_;
    param_10 = _e307;
    let _e308 = is_different_u0028_vf4_u003b_vf4_u003b((&param_9), (&param_10));
    if _e308 {
        let _e309 = pattern;
        pattern = (_e309 | 8i);
    }
    let _e311 = w5_;
    param_11 = _e311;
    let _e312 = w4_;
    param_12 = _e312;
    let _e313 = is_different_u0028_vf4_u003b_vf4_u003b((&param_11), (&param_12));
    if _e313 {
        let _e314 = pattern;
        pattern = (_e314 | 16i);
    }
    let _e316 = w6_;
    param_13 = _e316;
    let _e317 = w4_;
    param_14 = _e317;
    let _e318 = is_different_u0028_vf4_u003b_vf4_u003b((&param_13), (&param_14));
    if _e318 {
        let _e319 = pattern;
        pattern = (_e319 | 32i);
    }
    let _e321 = w7_;
    param_15 = _e321;
    let _e322 = w4_;
    param_16 = _e322;
    let _e323 = is_different_u0028_vf4_u003b_vf4_u003b((&param_15), (&param_16));
    if _e323 {
        let _e324 = pattern;
        pattern = (_e324 | 64i);
    }
    let _e326 = w8_;
    param_17 = _e326;
    let _e327 = w4_;
    param_18 = _e327;
    let _e328 = is_different_u0028_vf4_u003b_vf4_u003b((&param_17), (&param_18));
    if _e328 {
        let _e329 = pattern;
        pattern = (_e329 | 128i);
    }
    let _e331 = pattern;
    let _e333 = ((_e331 & 191i) == 55i);
    phi_334_ = _e333;
    if !(_e333) {
        let _e335 = pattern;
        phi_334_ = ((_e335 & 219i) == 19i);
    }
    let _e339 = phi_334_;
    phi_342_ = _e339;
    if _e339 {
        let _e340 = w1_;
        param_19 = _e340;
        let _e341 = w5_;
        param_20 = _e341;
        let _e342 = is_different_u0028_vf4_u003b_vf4_u003b((&param_19), (&param_20));
        phi_342_ = _e342;
    }
    let _e344 = phi_342_;
    if _e344 {
        let _e345 = w4_;
        let _e346 = w3_;
        let _e348 = p[0u];
        return mix(_e345, _e346, vec4((0.5f - _e348)));
    }
    let _e352 = pattern;
    let _e354 = ((_e352 & 219i) == 73i);
    phi_365_ = _e354;
    if !(_e354) {
        let _e356 = pattern;
        phi_365_ = ((_e356 & 239i) == 109i);
    }
    let _e360 = phi_365_;
    phi_373_ = _e360;
    if _e360 {
        let _e361 = w7_;
        param_21 = _e361;
        let _e362 = w3_;
        param_22 = _e362;
        let _e363 = is_different_u0028_vf4_u003b_vf4_u003b((&param_21), (&param_22));
        phi_373_ = _e363;
    }
    let _e365 = phi_373_;
    if _e365 {
        let _e366 = w4_;
        let _e367 = w1_;
        let _e369 = p[1u];
        return mix(_e366, _e367, vec4((0.5f - _e369)));
    }
    let _e373 = pattern;
    let _e375 = ((_e373 & 11i) == 11i);
    phi_396_ = _e375;
    if !(_e375) {
        let _e377 = pattern;
        phi_396_ = ((_e377 & 254i) == 74i);
    }
    let _e381 = phi_396_;
    phi_404_ = _e381;
    if !(_e381) {
        let _e383 = pattern;
        phi_404_ = ((_e383 & 254i) == 26i);
    }
    let _e387 = phi_404_;
    phi_412_ = _e387;
    if _e387 {
        let _e388 = w3_;
        param_23 = _e388;
        let _e389 = w1_;
        param_24 = _e389;
        let _e390 = is_different_u0028_vf4_u003b_vf4_u003b((&param_23), (&param_24));
        phi_412_ = _e390;
    }
    let _e392 = phi_412_;
    if _e392 {
        let _e393 = w4_;
        return _e393;
    }
    let _e394 = pattern;
    let _e396 = ((_e394 & 111i) == 42i);
    phi_430_ = _e396;
    if !(_e396) {
        let _e398 = pattern;
        phi_430_ = ((_e398 & 91i) == 10i);
    }
    let _e402 = phi_430_;
    phi_438_ = _e402;
    if !(_e402) {
        let _e404 = pattern;
        phi_438_ = ((_e404 & 191i) == 58i);
    }
    let _e408 = phi_438_;
    phi_447_ = _e408;
    if !(_e408) {
        let _e410 = pattern;
        phi_447_ = ((_e410 & 223i) == 90i);
    }
    let _e414 = phi_447_;
    phi_456_ = _e414;
    if !(_e414) {
        let _e416 = pattern;
        phi_456_ = ((_e416 & 159i) == 138i);
    }
    let _e420 = phi_456_;
    phi_464_ = _e420;
    if !(_e420) {
        let _e422 = pattern;
        phi_464_ = ((_e422 & 207i) == 138i);
    }
    let _e426 = phi_464_;
    phi_472_ = _e426;
    if !(_e426) {
        let _e428 = pattern;
        phi_472_ = ((_e428 & 239i) == 78i);
    }
    let _e432 = phi_472_;
    phi_481_ = _e432;
    if !(_e432) {
        let _e434 = pattern;
        phi_481_ = ((_e434 & 63i) == 14i);
    }
    let _e438 = phi_481_;
    phi_489_ = _e438;
    if !(_e438) {
        let _e440 = pattern;
        phi_489_ = ((_e440 & 251i) == 90i);
    }
    let _e444 = phi_489_;
    phi_497_ = _e444;
    if !(_e444) {
        let _e446 = pattern;
        phi_497_ = ((_e446 & 187i) == 138i);
    }
    let _e450 = phi_497_;
    phi_505_ = _e450;
    if !(_e450) {
        let _e452 = pattern;
        phi_505_ = ((_e452 & 127i) == 90i);
    }
    let _e456 = phi_505_;
    phi_513_ = _e456;
    if !(_e456) {
        let _e458 = pattern;
        phi_513_ = ((_e458 & 175i) == 138i);
    }
    let _e462 = phi_513_;
    phi_521_ = _e462;
    if !(_e462) {
        let _e464 = pattern;
        phi_521_ = ((_e464 & 235i) == 138i);
    }
    let _e468 = phi_521_;
    phi_529_ = _e468;
    if _e468 {
        let _e469 = w3_;
        param_25 = _e469;
        let _e470 = w1_;
        param_26 = _e470;
        let _e471 = is_different_u0028_vf4_u003b_vf4_u003b((&param_25), (&param_26));
        phi_529_ = _e471;
    }
    let _e473 = phi_529_;
    if _e473 {
        let _e474 = w4_;
        let _e475 = w4_;
        let _e476 = w0_;
        let _e478 = p[0u];
        let _e483 = p[1u];
        return mix(_e474, mix(_e475, _e476, vec4((0.5f - _e478))), vec4((0.5f - _e483)));
    }
    let _e487 = pattern;
    if ((_e487 & 11i) == 8i) {
        let _e490 = w0_;
        let _e492 = w1_;
        let _e495 = w4_;
        let _e498 = w4_;
        let _e500 = w1_;
        let _e504 = p[0u];
        let _e508 = w4_;
        let _e510 = p[1u];
        return mix(mix((((_e490 * 0.375f) + (_e492 * 0.25f)) + (_e495 * 0.375f)), ((_e498 * 0.5f) + (_e500 * 0.5f)), vec4((_e504 * 2f))), _e508, vec4((_e510 * 2f)));
    }
    let _e514 = pattern;
    if ((_e514 & 11i) == 2i) {
        let _e517 = w0_;
        let _e519 = w3_;
        let _e522 = w4_;
        let _e525 = w4_;
        let _e527 = w3_;
        let _e531 = p[1u];
        let _e535 = w4_;
        let _e537 = p[0u];
        return mix(mix((((_e517 * 0.375f) + (_e519 * 0.25f)) + (_e522 * 0.375f)), ((_e525 * 0.5f) + (_e527 * 0.5f)), vec4((_e531 * 2f))), _e535, vec4((_e537 * 2f)));
    }
    let _e541 = pattern;
    if ((_e541 & 47i) == 47i) {
        let _e544 = p;
        dist = length((_e544 - vec2<f32>(0.5f, 0.5f)));
        let _e548 = params.OutputSize;
        let _e551 = params.SourceSize;
        pixel_size = length((vec2(1f) / (_e548.xy / _e551.xy)));
        let _e557 = dist;
        let _e558 = pixel_size;
        if (_e557 < (0.5f - (_e558 / 2f))) {
            let _e562 = w4_;
            return _e562;
        }
        let _e563 = w0_;
        param_27 = _e563;
        let _e564 = w1_;
        param_28 = _e564;
        let _e565 = is_different_u0028_vf4_u003b_vf4_u003b((&param_27), (&param_28));
        phi_652_ = _e565;
        if !(_e565) {
            let _e567 = w0_;
            param_29 = _e567;
            let _e568 = w3_;
            param_30 = _e568;
            let _e569 = is_different_u0028_vf4_u003b_vf4_u003b((&param_29), (&param_30));
            phi_652_ = _e569;
        }
        let _e571 = phi_652_;
        if _e571 {
            let _e572 = w1_;
            let _e573 = w3_;
            let _e575 = p[1u];
            let _e577 = p[0u];
            r = mix(_e572, _e573, vec4(((_e575 - _e577) + 0.5f)));
        } else {
            let _e582 = w1_;
            let _e584 = w0_;
            let _e587 = w3_;
            let _e590 = w3_;
            let _e592 = p[1u];
            let _e596 = w1_;
            let _e598 = p[0u];
            r = mix(mix((((_e582 * 0.375f) + (_e584 * 0.25f)) + (_e587 * 0.375f)), _e590, vec4((_e592 * 2f))), _e596, vec4((_e598 * 2f)));
        }
        let _e602 = dist;
        let _e603 = pixel_size;
        if (_e602 > (0.5f + (_e603 / 2f))) {
            let _e607 = r;
            return _e607;
        }
        let _e608 = w4_;
        let _e609 = r;
        let _e610 = dist;
        let _e612 = pixel_size;
        let _e615 = pixel_size;
        return mix(_e608, _e609, vec4((((_e610 - 0.5f) + (_e612 / 2f)) / _e615)));
    }
    let _e619 = pattern;
    let _e621 = ((_e619 & 191i) == 55i);
    phi_717_ = _e621;
    if !(_e621) {
        let _e623 = pattern;
        phi_717_ = ((_e623 & 219i) == 19i);
    }
    let _e627 = phi_717_;
    if _e627 {
        let _e629 = p[0u];
        let _e631 = p[1u];
        dist_1 = (_e629 - (2f * _e631));
        let _e635 = params.OutputSize;
        let _e638 = params.SourceSize;
        pixel_size_1 = (length((vec2(1f) / (_e635.xy / _e638.xy))) * 2.236068f);
        let _e645 = dist_1;
        let _e646 = pixel_size_1;
        if (_e645 > (_e646 / 2f)) {
            let _e649 = w1_;
            return _e649;
        }
        let _e650 = w3_;
        let _e651 = w4_;
        let _e653 = p[0u];
        r_1 = mix(_e650, _e651, vec4((_e653 + 0.5f)));
        let _e657 = dist_1;
        let _e658 = pixel_size_1;
        if (_e657 < (-(_e658) / 2f)) {
            let _e662 = r_1;
            return _e662;
        }
        let _e663 = r_1;
        let _e664 = w1_;
        let _e665 = dist_1;
        let _e666 = pixel_size_1;
        let _e669 = pixel_size_1;
        return mix(_e663, _e664, vec4(((_e665 + (_e666 / 2f)) / _e669)));
    }
    let _e673 = pattern;
    let _e675 = ((_e673 & 219i) == 73i);
    phi_785_ = _e675;
    if !(_e675) {
        let _e677 = pattern;
        phi_785_ = ((_e677 & 239i) == 109i);
    }
    let _e681 = phi_785_;
    if _e681 {
        let _e683 = p[1u];
        let _e685 = p[0u];
        dist_2 = (_e683 - (2f * _e685));
        let _e689 = params.OutputSize;
        let _e692 = params.SourceSize;
        pixel_size_2 = (length((vec2(1f) / (_e689.xy / _e692.xy))) * 2.236068f);
        let _e700 = p[1u];
        let _e702 = p[0u];
        let _e705 = pixel_size_2;
        if ((_e700 - (2f * _e702)) > (_e705 / 2f)) {
            let _e708 = w3_;
            return _e708;
        }
        let _e709 = w1_;
        let _e710 = w4_;
        let _e712 = p[0u];
        r_2 = mix(_e709, _e710, vec4((_e712 + 0.5f)));
        let _e716 = dist_2;
        let _e717 = pixel_size_2;
        if (_e716 < (-(_e717) / 2f)) {
            let _e721 = r_2;
            return _e721;
        }
        let _e722 = r_2;
        let _e723 = w3_;
        let _e724 = dist_2;
        let _e725 = pixel_size_2;
        let _e728 = pixel_size_2;
        return mix(_e722, _e723, vec4(((_e724 + (_e725 / 2f)) / _e728)));
    }
    let _e732 = pattern;
    let _e734 = ((_e732 & 191i) == 143i);
    phi_859_ = _e734;
    if !(_e734) {
        let _e736 = pattern;
        phi_859_ = ((_e736 & 126i) == 14i);
    }
    let _e740 = phi_859_;
    if _e740 {
        let _e742 = p[0u];
        let _e744 = p[1u];
        dist_3 = (_e742 + (2f * _e744));
        let _e748 = params.OutputSize;
        let _e751 = params.SourceSize;
        pixel_size_3 = (length((vec2(1f) / (_e748.xy / _e751.xy))) * 2.236068f);
        let _e758 = dist_3;
        let _e759 = pixel_size_3;
        if (_e758 > (1f + (_e759 / 2f))) {
            let _e763 = w4_;
            return _e763;
        }
        let _e764 = w0_;
        param_31 = _e764;
        let _e765 = w1_;
        param_32 = _e765;
        let _e766 = is_different_u0028_vf4_u003b_vf4_u003b((&param_31), (&param_32));
        phi_903_ = _e766;
        if !(_e766) {
            let _e768 = w0_;
            param_33 = _e768;
            let _e769 = w3_;
            param_34 = _e769;
            let _e770 = is_different_u0028_vf4_u003b_vf4_u003b((&param_33), (&param_34));
            phi_903_ = _e770;
        }
        let _e772 = phi_903_;
        if _e772 {
            let _e773 = w1_;
            let _e774 = w3_;
            let _e776 = p[1u];
            let _e778 = p[0u];
            r_3 = mix(_e773, _e774, vec4(((_e776 - _e778) + 0.5f)));
        } else {
            let _e783 = w1_;
            let _e785 = w0_;
            let _e788 = w3_;
            let _e791 = w3_;
            let _e793 = p[1u];
            let _e797 = w1_;
            let _e799 = p[0u];
            r_3 = mix(mix((((_e783 * 0.375f) + (_e785 * 0.25f)) + (_e788 * 0.375f)), _e791, vec4((_e793 * 2f))), _e797, vec4((_e799 * 2f)));
        }
        let _e803 = dist_3;
        let _e804 = pixel_size_3;
        if (_e803 < (1f - (_e804 / 2f))) {
            let _e808 = r_3;
            return _e808;
        }
        let _e809 = r_3;
        let _e810 = w4_;
        let _e811 = dist_3;
        let _e812 = pixel_size_3;
        let _e816 = pixel_size_3;
        return mix(_e809, _e810, vec4((((_e811 + (_e812 / 2f)) - 1f) / _e816)));
    }
    let _e820 = pattern;
    let _e822 = ((_e820 & 126i) == 42i);
    phi_969_ = _e822;
    if !(_e822) {
        let _e824 = pattern;
        phi_969_ = ((_e824 & 239i) == 171i);
    }
    let _e828 = phi_969_;
    if _e828 {
        let _e830 = p[1u];
        let _e832 = p[0u];
        dist_4 = (_e830 + (2f * _e832));
        let _e836 = params.OutputSize;
        let _e839 = params.SourceSize;
        pixel_size_4 = (length((vec2(1f) / (_e836.xy / _e839.xy))) * 2.236068f);
        let _e847 = p[1u];
        let _e849 = p[0u];
        let _e852 = pixel_size_4;
        if ((_e847 + (2f * _e849)) > (1f + (_e852 / 2f))) {
            let _e856 = w4_;
            return _e856;
        }
        let _e857 = w0_;
        param_35 = _e857;
        let _e858 = w1_;
        param_36 = _e858;
        let _e859 = is_different_u0028_vf4_u003b_vf4_u003b((&param_35), (&param_36));
        phi_1018_ = _e859;
        if !(_e859) {
            let _e861 = w0_;
            param_37 = _e861;
            let _e862 = w3_;
            param_38 = _e862;
            let _e863 = is_different_u0028_vf4_u003b_vf4_u003b((&param_37), (&param_38));
            phi_1018_ = _e863;
        }
        let _e865 = phi_1018_;
        if _e865 {
            let _e866 = w1_;
            let _e867 = w3_;
            let _e869 = p[1u];
            let _e871 = p[0u];
            r_4 = mix(_e866, _e867, vec4(((_e869 - _e871) + 0.5f)));
        } else {
            let _e876 = w1_;
            let _e878 = w0_;
            let _e881 = w3_;
            let _e884 = w3_;
            let _e886 = p[1u];
            let _e890 = w1_;
            let _e892 = p[0u];
            r_4 = mix(mix((((_e876 * 0.375f) + (_e878 * 0.25f)) + (_e881 * 0.375f)), _e884, vec4((_e886 * 2f))), _e890, vec4((_e892 * 2f)));
        }
        let _e896 = dist_4;
        let _e897 = pixel_size_4;
        if (_e896 < (1f - (_e897 / 2f))) {
            let _e901 = r_4;
            return _e901;
        }
        let _e902 = r_4;
        let _e903 = w4_;
        let _e904 = dist_4;
        let _e905 = pixel_size_4;
        let _e909 = pixel_size_4;
        return mix(_e902, _e903, vec4((((_e904 + (_e905 / 2f)) - 1f) / _e909)));
    }
    let _e913 = pattern;
    let _e915 = ((_e913 & 27i) == 3i);
    phi_1087_ = _e915;
    if !(_e915) {
        let _e917 = pattern;
        phi_1087_ = ((_e917 & 79i) == 67i);
    }
    let _e921 = phi_1087_;
    phi_1096_ = _e921;
    if !(_e921) {
        let _e923 = pattern;
        phi_1096_ = ((_e923 & 139i) == 131i);
    }
    let _e927 = phi_1096_;
    phi_1104_ = _e927;
    if !(_e927) {
        let _e929 = pattern;
        phi_1104_ = ((_e929 & 107i) == 67i);
    }
    let _e933 = phi_1104_;
    if _e933 {
        let _e934 = w4_;
        let _e935 = w3_;
        let _e937 = p[0u];
        return mix(_e934, _e935, vec4((0.5f - _e937)));
    }
    let _e941 = pattern;
    let _e943 = ((_e941 & 75i) == 9i);
    phi_1127_ = _e943;
    if !(_e943) {
        let _e945 = pattern;
        phi_1127_ = ((_e945 & 139i) == 137i);
    }
    let _e949 = phi_1127_;
    phi_1136_ = _e949;
    if !(_e949) {
        let _e951 = pattern;
        phi_1136_ = ((_e951 & 31i) == 25i);
    }
    let _e955 = phi_1136_;
    phi_1144_ = _e955;
    if !(_e955) {
        let _e957 = pattern;
        phi_1144_ = ((_e957 & 59i) == 25i);
    }
    let _e961 = phi_1144_;
    if _e961 {
        let _e962 = w4_;
        let _e963 = w1_;
        let _e965 = p[1u];
        return mix(_e962, _e963, vec4((0.5f - _e965)));
    }
    let _e969 = pattern;
    let _e971 = ((_e969 & 251i) == 106i);
    phi_1166_ = _e971;
    if !(_e971) {
        let _e973 = pattern;
        phi_1166_ = ((_e973 & 111i) == 110i);
    }
    let _e977 = phi_1166_;
    phi_1174_ = _e977;
    if !(_e977) {
        let _e979 = pattern;
        phi_1174_ = ((_e979 & 63i) == 62i);
    }
    let _e983 = phi_1174_;
    phi_1182_ = _e983;
    if !(_e983) {
        let _e985 = pattern;
        phi_1182_ = ((_e985 & 251i) == 250i);
    }
    let _e989 = phi_1182_;
    phi_1190_ = _e989;
    if !(_e989) {
        let _e991 = pattern;
        phi_1190_ = ((_e991 & 223i) == 222i);
    }
    let _e995 = phi_1190_;
    phi_1198_ = _e995;
    if !(_e995) {
        let _e997 = pattern;
        phi_1198_ = ((_e997 & 223i) == 30i);
    }
    let _e1001 = phi_1198_;
    if _e1001 {
        let _e1002 = w4_;
        let _e1003 = w0_;
        let _e1005 = p[0u];
        let _e1008 = p[1u];
        return mix(_e1002, _e1003, vec4((((1f - _e1005) - _e1008) / 2f)));
    }
    let _e1013 = pattern;
    let _e1015 = ((_e1013 & 79i) == 75i);
    phi_1222_ = _e1015;
    if !(_e1015) {
        let _e1017 = pattern;
        phi_1222_ = ((_e1017 & 159i) == 27i);
    }
    let _e1021 = phi_1222_;
    phi_1229_ = _e1021;
    if !(_e1021) {
        let _e1023 = pattern;
        phi_1229_ = ((_e1023 & 47i) == 11i);
    }
    let _e1027 = phi_1229_;
    phi_1237_ = _e1027;
    if !(_e1027) {
        let _e1029 = pattern;
        phi_1237_ = ((_e1029 & 190i) == 10i);
    }
    let _e1033 = phi_1237_;
    phi_1245_ = _e1033;
    if !(_e1033) {
        let _e1035 = pattern;
        phi_1245_ = ((_e1035 & 238i) == 10i);
    }
    let _e1039 = phi_1245_;
    phi_1252_ = _e1039;
    if !(_e1039) {
        let _e1041 = pattern;
        phi_1252_ = ((_e1041 & 126i) == 10i);
    }
    let _e1045 = phi_1252_;
    phi_1259_ = _e1045;
    if !(_e1045) {
        let _e1047 = pattern;
        phi_1259_ = ((_e1047 & 235i) == 75i);
    }
    let _e1051 = phi_1259_;
    phi_1266_ = _e1051;
    if !(_e1051) {
        let _e1053 = pattern;
        phi_1266_ = ((_e1053 & 59i) == 27i);
    }
    let _e1057 = phi_1266_;
    if _e1057 {
        let _e1059 = p[0u];
        let _e1061 = p[1u];
        dist_5 = (_e1059 + _e1061);
        let _e1064 = params.OutputSize;
        let _e1067 = params.SourceSize;
        pixel_size_5 = length((vec2(1f) / (_e1064.xy / _e1067.xy)));
        let _e1073 = dist_5;
        let _e1074 = pixel_size_5;
        if (_e1073 > (0.5f + (_e1074 / 2f))) {
            let _e1078 = w4_;
            return _e1078;
        }
        let _e1079 = w0_;
        param_39 = _e1079;
        let _e1080 = w1_;
        param_40 = _e1080;
        let _e1081 = is_different_u0028_vf4_u003b_vf4_u003b((&param_39), (&param_40));
        phi_1308_ = _e1081;
        if !(_e1081) {
            let _e1083 = w0_;
            param_41 = _e1083;
            let _e1084 = w3_;
            param_42 = _e1084;
            let _e1085 = is_different_u0028_vf4_u003b_vf4_u003b((&param_41), (&param_42));
            phi_1308_ = _e1085;
        }
        let _e1087 = phi_1308_;
        if _e1087 {
            let _e1088 = w1_;
            let _e1089 = w3_;
            let _e1091 = p[1u];
            let _e1093 = p[0u];
            r_5 = mix(_e1088, _e1089, vec4(((_e1091 - _e1093) + 0.5f)));
        } else {
            let _e1098 = w1_;
            let _e1100 = w0_;
            let _e1103 = w3_;
            let _e1106 = w3_;
            let _e1108 = p[1u];
            let _e1112 = w1_;
            let _e1114 = p[0u];
            r_5 = mix(mix((((_e1098 * 0.375f) + (_e1100 * 0.25f)) + (_e1103 * 0.375f)), _e1106, vec4((_e1108 * 2f))), _e1112, vec4((_e1114 * 2f)));
        }
        let _e1118 = dist_5;
        let _e1119 = pixel_size_5;
        if (_e1118 < (0.5f - (_e1119 / 2f))) {
            let _e1123 = r_5;
            return _e1123;
        }
        let _e1124 = r_5;
        let _e1125 = w4_;
        let _e1126 = dist_5;
        let _e1127 = pixel_size_5;
        let _e1131 = pixel_size_5;
        return mix(_e1124, _e1125, vec4((((_e1126 + (_e1127 / 2f)) - 0.5f) / _e1131)));
    }
    let _e1135 = pattern;
    if ((_e1135 & 11i) == 1i) {
        let _e1138 = w4_;
        let _e1139 = w3_;
        let _e1141 = p[0u];
        let _e1145 = w1_;
        let _e1146 = w1_;
        let _e1147 = w3_;
        let _e1152 = p[0u];
        let _e1157 = p[1u];
        return mix(mix(_e1138, _e1139, vec4((0.5f - _e1141))), mix(_e1145, ((_e1146 + _e1147) / vec4(2f)), vec4((0.5f - _e1152))), vec4((0.5f - _e1157)));
    }
    let _e1161 = pattern;
    if ((_e1161 & 11i) == 0i) {
        let _e1164 = w4_;
        let _e1165 = w3_;
        let _e1167 = p[0u];
        let _e1171 = w1_;
        let _e1172 = w0_;
        let _e1174 = p[0u];
        let _e1179 = p[1u];
        return mix(mix(_e1164, _e1165, vec4((0.5f - _e1167))), mix(_e1171, _e1172, vec4((0.5f - _e1174))), vec4((0.5f - _e1179)));
    }
    let _e1184 = p[0u];
    let _e1186 = p[1u];
    dist_6 = (_e1184 + _e1186);
    let _e1189 = params.OutputSize;
    let _e1192 = params.SourceSize;
    pixel_size_6 = length((vec2(1f) / (_e1189.xy / _e1192.xy)));
    let _e1198 = dist_6;
    let _e1199 = pixel_size_6;
    if (_e1198 > (0.5f + (_e1199 / 2f))) {
        let _e1203 = w4_;
        return _e1203;
    }
    let _e1204 = texCoord;
    let _e1206 = o[0u];
    let _e1210 = o[1u];
    let _e1215 = textureSample(image, param_2, (_e1204 + vec2<f32>((-(_e1206) * 2f), (-(_e1210) * 2f))));
    x0_ = _e1215;
    let _e1216 = texCoord;
    let _e1218 = o[0u];
    let _e1221 = o[1u];
    let _e1226 = textureSample(image, param_2, (_e1216 + vec2<f32>(-(_e1218), (-(_e1221) * 2f))));
    x1_ = _e1226;
    let _e1227 = texCoord;
    let _e1229 = o[1u];
    let _e1234 = textureSample(image, param_2, (_e1227 + vec2<f32>(0f, (-(_e1229) * 2f))));
    x2_ = _e1234;
    let _e1235 = texCoord;
    let _e1237 = o[0u];
    let _e1239 = o[1u];
    let _e1244 = textureSample(image, param_2, (_e1235 + vec2<f32>(_e1237, (-(_e1239) * 2f))));
    x3_ = _e1244;
    let _e1245 = texCoord;
    let _e1247 = o[0u];
    let _e1251 = o[1u];
    let _e1255 = textureSample(image, param_2, (_e1245 + vec2<f32>((-(_e1247) * 2f), -(_e1251))));
    x4_ = _e1255;
    let _e1256 = texCoord;
    let _e1258 = o[0u];
    let _e1263 = textureSample(image, param_2, (_e1256 + vec2<f32>((-(_e1258) * 2f), 0f)));
    x5_ = _e1263;
    let _e1264 = texCoord;
    let _e1266 = o[0u];
    let _e1270 = o[1u];
    let _e1273 = textureSample(image, param_2, (_e1264 + vec2<f32>((-(_e1266) * 2f), _e1270)));
    x6_ = _e1273;
    let _e1274 = x0_;
    param_43 = _e1274;
    let _e1275 = w4_;
    param_44 = _e1275;
    let _e1276 = is_different_u0028_vf4_u003b_vf4_u003b((&param_43), (&param_44));
    if _e1276 {
        let _e1277 = pattern;
        pattern = (_e1277 | 256i);
    }
    let _e1279 = x1_;
    param_45 = _e1279;
    let _e1280 = w4_;
    param_46 = _e1280;
    let _e1281 = is_different_u0028_vf4_u003b_vf4_u003b((&param_45), (&param_46));
    if _e1281 {
        let _e1282 = pattern;
        pattern = (_e1282 | 512i);
    }
    let _e1284 = x2_;
    param_47 = _e1284;
    let _e1285 = w4_;
    param_48 = _e1285;
    let _e1286 = is_different_u0028_vf4_u003b_vf4_u003b((&param_47), (&param_48));
    if _e1286 {
        let _e1287 = pattern;
        pattern = (_e1287 | 1024i);
    }
    let _e1289 = x3_;
    param_49 = _e1289;
    let _e1290 = w4_;
    param_50 = _e1290;
    let _e1291 = is_different_u0028_vf4_u003b_vf4_u003b((&param_49), (&param_50));
    if _e1291 {
        let _e1292 = pattern;
        pattern = (_e1292 | 2048i);
    }
    let _e1294 = x4_;
    param_51 = _e1294;
    let _e1295 = w4_;
    param_52 = _e1295;
    let _e1296 = is_different_u0028_vf4_u003b_vf4_u003b((&param_51), (&param_52));
    if _e1296 {
        let _e1297 = pattern;
        pattern = (_e1297 | 4096i);
    }
    let _e1299 = x5_;
    param_53 = _e1299;
    let _e1300 = w4_;
    param_54 = _e1300;
    let _e1301 = is_different_u0028_vf4_u003b_vf4_u003b((&param_53), (&param_54));
    if _e1301 {
        let _e1302 = pattern;
        pattern = (_e1302 | 8192i);
    }
    let _e1304 = x6_;
    param_55 = _e1304;
    let _e1305 = w4_;
    param_56 = _e1305;
    let _e1306 = is_different_u0028_vf4_u003b_vf4_u003b((&param_55), (&param_56));
    if _e1306 {
        let _e1307 = pattern;
        pattern = (_e1307 | 16384i);
    }
    diagonal_bias = -7i;
    loop {
        let _e1309 = pattern;
        if (_e1309 != 0i) {
            let _e1311 = pattern;
            let _e1313 = diagonal_bias;
            diagonal_bias = (_e1313 + (_e1311 & 1i));
            let _e1315 = pattern;
            pattern = (_e1315 >> bitcast<u32>(1i));
            continue;
        } else {
            break;
        }
    }
    let _e1318 = diagonal_bias;
    if (_e1318 <= 0i) {
        let _e1320 = w1_;
        let _e1321 = w3_;
        let _e1323 = p[1u];
        let _e1325 = p[0u];
        r_6 = mix(_e1320, _e1321, vec4(((_e1323 - _e1325) + 0.5f)));
        let _e1330 = dist_6;
        let _e1331 = pixel_size_6;
        if (_e1330 < (0.5f - (_e1331 / 2f))) {
            let _e1335 = r_6;
            return _e1335;
        }
        let _e1336 = r_6;
        let _e1337 = w4_;
        let _e1338 = dist_6;
        let _e1339 = pixel_size_6;
        let _e1343 = pixel_size_6;
        return mix(_e1336, _e1337, vec4((((_e1338 + (_e1339 / 2f)) - 0.5f) / _e1343)));
    }
    let _e1347 = w4_;
    return _e1347;
}

fn main_1() {
    var param_57: vec2<f32>;

    let _e94 = vTexCoord_1;
    param_57 = _e94;
    let _e95 = scale_u0028_s21_u003b_vf2_u003b(Source, _Source_sampler, (&param_57));
    FragColor = _e95;
    return;
}

@fragment 
fn main(@location(0) vTexCoord: vec2<f32>) -> @location(0) vec4<f32> {
    vTexCoord_1 = vTexCoord;
    main_1();
    let _e3 = FragColor;
    return _e3;
}
