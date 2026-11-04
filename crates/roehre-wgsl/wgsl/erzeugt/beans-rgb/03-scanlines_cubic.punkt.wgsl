// ERZEUGT aus fremden MIT-Shadern bzw. roehre (MIT OR Apache-2.0) — eigen/beans/rgb.slangp, Pass 3 »scanlines_cubic« (Herkunft: shader/fremd/mit/).
// Übersetzt von librashader 0.12.1 (slang → SPIR-V → naga → WGSL). Nicht von Hand ändern.
diagnostic(off, derivative_uniformity);
struct lines_t {
    upper_sample_y: f32,
    lower_sample_y: f32,
    upper_distance_y: f32,
    lower_distance_y: f32,
}

struct Push {
    SourceSize: vec4<f32>,
    OutputSize: vec4<f32>,
    FrameCount: u32,
    MaxSpotSize: f32,
    MinSpotSize: f32,
    OddFieldFirst: f32,
    OverscanHorizontal: f32,
    OverscanVertical: f32,
    MaskType: f32,
    SubpixelMaskPattern: f32,
    SubpixelPattern: f32,
    DynamicMaskTriads: f32,
}

struct UBO {
    MVP: mat4x4<f32>,
}

@group(0) @binding(3) 
var<uniform> params: Push;
var<private> delta_1: f32;
@group(0) @binding(2) 
var Filtered: texture_2d<f32>;
@group(0) @binding(1) 
var Source: texture_2d<f32>;
var<private> vTexCoord_1: vec2<f32>;
var<private> viewportCoord_1: vec2<f32>;
var<private> FragColor: vec4<f32>;
@group(0) @binding(0) 
var<uniform> unnamed: UBO;
@group(1) @binding(2) 
var _Filtered_sampler: sampler;
@group(1) @binding(1) 
var _Source_sampler: sampler;

fn dynamic_aperture_grille_u0028_vf2_u003b_vf4_u003b_f1_u003b(coord: ptr<function, vec2<f32>>, output_size: ptr<function, vec4<f32>>, mask_triads: ptr<function, f32>) -> vec4<f32> {
    var kernel_radius: f32;
    var x: f32;
    var mask: vec3<f32>;
    var x1_: f32;
    var p0_: vec3<f32>;
    var indexable: array<vec3<f32>, 3>;
    var p1_: vec3<f32>;
    var x1_1: f32;
    var x2_: f32;
    var p0_1: vec3<f32>;
    var indexable_1: array<vec3<f32>, 3>;
    var p1_1: vec3<f32>;
    var p2_: vec3<f32>;
    var x1_2: f32;
    var x2_1: f32;
    var x3_: f32;
    var p0_2: vec3<f32>;
    var indexable_2: array<vec3<f32>, 3>;
    var p1_2: vec3<f32>;
    var p2_1: vec3<f32>;
    var p3_: vec3<f32>;

    let _e77 = (*mask_triads);
    let _e80 = (*output_size)[2u];
    kernel_radius = ((3f * _e77) * _e80);
    let _e82 = (*mask_triads);
    let _e85 = (*coord)[0u];
    x = ((3f * _e82) * _e85);
    mask = vec3<f32>(0f, 0f, 0f);
    let _e87 = kernel_radius;
    if (_e87 < 0.5f) {
        let _e89 = x;
        let _e90 = x;
        let _e93 = kernel_radius;
        x1_ = clamp(((_e89 - round(_e90)) / _e93), -1f, 1f);
        let _e96 = x;
        let _e98 = (round(_e96) - 1f);
        indexable = array<vec3<f32>, 3>(vec3<f32>(1f, 0f, 0f), vec3<f32>(0f, 1f, 0f), vec3<f32>(0f, 0f, 1f));
        let _e106 = indexable[i32(((_e98 - (floor((_e98 / 3f)) * 3f)) + 0.001f))];
        p0_ = _e106;
        let _e107 = p0_;
        p1_ = _e107.zxy;
        let _e109 = p0_;
        let _e110 = p1_;
        let _e112 = p1_;
        let _e113 = p0_;
        let _e115 = x1_;
        let _e116 = x1_;
        mask = ((_e109 + _e110) + ((_e112 - _e113) * (_e115 + (sin((3.1415927f * _e116)) / 3.1415927f))));
        let _e123 = mask;
        mask = (_e123 * 0.5f);
    } else {
        let _e125 = kernel_radius;
        if (_e125 < 1f) {
            let _e127 = x;
            let _e128 = x;
            let _e132 = kernel_radius;
            x1_1 = clamp(((_e127 - (floor(_e128) + 0f)) / _e132), -1f, 1f);
            let _e135 = x;
            let _e136 = x;
            let _e140 = kernel_radius;
            x2_ = clamp(((_e135 - (floor(_e136) + 1f)) / _e140), -1f, 1f);
            let _e143 = x;
            let _e145 = (floor(_e143) - 1f);
            indexable_1 = array<vec3<f32>, 3>(vec3<f32>(1f, 0f, 0f), vec3<f32>(0f, 1f, 0f), vec3<f32>(0f, 0f, 1f));
            let _e153 = indexable_1[i32(((_e145 - (floor((_e145 / 3f)) * 3f)) + 0.001f))];
            p0_1 = _e153;
            let _e154 = p0_1;
            p1_1 = _e154.zxy;
            let _e156 = p1_1;
            p2_ = _e156.zxy;
            let _e158 = p0_1;
            let _e159 = p2_;
            let _e161 = p1_1;
            let _e162 = p0_1;
            let _e164 = x1_1;
            let _e165 = x1_1;
            let _e172 = p2_;
            let _e173 = p1_1;
            let _e175 = x2_;
            let _e176 = x2_;
            mask = (((_e158 + _e159) + ((_e161 - _e162) * (_e164 + (sin((3.1415927f * _e165)) / 3.1415927f)))) + ((_e172 - _e173) * (_e175 + (sin((3.1415927f * _e176)) / 3.1415927f))));
            let _e183 = mask;
            mask = (_e183 * 0.5f);
        } else {
            let _e185 = x;
            let _e186 = x;
            let _e190 = kernel_radius;
            x1_2 = clamp(((_e185 - (round(_e186) - 1f)) / _e190), -1f, 1f);
            let _e193 = x;
            let _e194 = x;
            let _e198 = kernel_radius;
            x2_1 = clamp(((_e193 - (round(_e194) + 0f)) / _e198), -1f, 1f);
            let _e201 = x;
            let _e202 = x;
            let _e206 = kernel_radius;
            x3_ = clamp(((_e201 - (round(_e202) + 1f)) / _e206), -1f, 1f);
            let _e209 = x;
            let _e211 = (round(_e209) - 2f);
            indexable_2 = array<vec3<f32>, 3>(vec3<f32>(1f, 0f, 0f), vec3<f32>(0f, 1f, 0f), vec3<f32>(0f, 0f, 1f));
            let _e219 = indexable_2[i32(((_e211 - (floor((_e211 / 3f)) * 3f)) + 0.001f))];
            p0_2 = _e219;
            let _e220 = p0_2;
            p1_2 = _e220.zxy;
            let _e222 = p1_2;
            p2_1 = _e222.zxy;
            let _e224 = p2_1;
            p3_ = _e224.zxy;
            let _e226 = p0_2;
            let _e227 = p3_;
            let _e229 = p1_2;
            let _e230 = p0_2;
            let _e232 = x1_2;
            let _e233 = x1_2;
            let _e240 = p2_1;
            let _e241 = p1_2;
            let _e243 = x2_1;
            let _e244 = x2_1;
            let _e251 = p3_;
            let _e252 = p2_1;
            let _e254 = x3_;
            let _e255 = x3_;
            mask = ((((_e226 + _e227) + ((_e229 - _e230) * (_e232 + (sin((3.1415927f * _e233)) / 3.1415927f)))) + ((_e240 - _e241) * (_e243 + (sin((3.1415927f * _e244)) / 3.1415927f)))) + ((_e251 - _e252) * (_e254 + (sin((3.1415927f * _e255)) / 3.1415927f))));
            let _e262 = mask;
            mask = (_e262 * 0.5f);
        }
    }
    let _e264 = mask;
    return vec4<f32>(_e264.x, _e264.y, _e264.z, 3f);
}

fn blend_mask_u0028_vf3_u003b_vf3_u003b_f1_u003b(mask_1: ptr<function, vec3<f32>>, pixel_value: ptr<function, vec3<f32>>, mask_coverage: ptr<function, f32>) -> vec3<f32> {
    var a: vec3<f32>;
    var b: vec3<f32>;

    let _e58 = (*pixel_value);
    let _e61 = (*mask_coverage);
    let _e65 = (*pixel_value);
    a = clamp(((_e58 - vec3(1f)) / vec3((1f - _e61))), vec3<f32>(0f, 0f, 0f), _e65);
    let _e67 = (*mask_coverage);
    let _e68 = (*pixel_value);
    let _e72 = (*mask_coverage);
    let _e76 = (*pixel_value);
    b = clamp(((vec3(1f) - (_e68 * _e67)) / vec3((1f - _e72))), vec3<f32>(0f, 0f, 0f), _e76);
    let _e78 = (*mask_coverage);
    let _e79 = a;
    let _e81 = (*mask_1);
    let _e83 = b;
    return (((_e79 * _e78) * _e81) + _e83);
}

fn subpixel_aperture_grille_u0028_vf2_u003b_vf4_u003b_f1_u003b_f1_u003b(coord_1: ptr<function, vec2<f32>>, output_size_1: ptr<function, vec4<f32>>, subpixel_pattern: ptr<function, f32>, pixels_per_triad: ptr<function, f32>) -> vec4<f32> {
    var x_1: f32;
    var mask_2: vec3<f32>;
    var mask_coverage_1: f32;
    var mask_tile: array<vec3<f32>, 2>;
    var mask_tile_1: array<vec3<f32>, 3>;
    var mask_tile_2: array<vec3<f32>, 4>;
    var mask_tile_3: array<vec3<f32>, 5>;
    var mask_tile_4: array<vec3<f32>, 2>;
    var mask_tile_5: array<vec3<f32>, 3>;
    var mask_tile_6: array<vec3<f32>, 4>;
    var mask_tile_7: array<vec3<f32>, 5>;

    let _e69 = (*coord_1)[0u];
    let _e71 = (*output_size_1)[0u];
    x_1 = (_e69 * _e71);
    mask_2 = vec3<f32>(1f, 1f, 1f);
    mask_coverage_1 = 1f;
    let _e73 = (*subpixel_pattern);
    if (_e73 == 0f) {
        let _e75 = (*pixels_per_triad);
        if (_e75 == 2f) {
            mask_tile = array<vec3<f32>, 2>(vec3<f32>(1f, 0f, 1f), vec3<f32>(0f, 1f, 0f));
            let _e77 = x_1;
            let _e84 = mask_tile[i32((_e77 - (floor((_e77 / 2f)) * 2f)))];
            mask_2 = _e84;
            mask_coverage_1 = 2f;
        } else {
            let _e85 = (*pixels_per_triad);
            if (_e85 == 3f) {
                mask_tile_1 = array<vec3<f32>, 3>(vec3<f32>(0f, 0f, 1f), vec3<f32>(0f, 1f, 0f), vec3<f32>(1f, 0f, 0f));
                let _e87 = x_1;
                let _e94 = mask_tile_1[i32((_e87 - (floor((_e87 / 3f)) * 3f)))];
                mask_2 = _e94;
                mask_coverage_1 = 3f;
            } else {
                let _e95 = (*pixels_per_triad);
                if (_e95 == 4f) {
                    mask_tile_2 = array<vec3<f32>, 4>(vec3<f32>(1f, 0f, 0f), vec3<f32>(1f, 1f, 0f), vec3<f32>(0f, 1f, 1f), vec3<f32>(0f, 0f, 1f));
                    let _e97 = x_1;
                    let _e104 = mask_tile_2[i32((_e97 - (floor((_e97 / 4f)) * 4f)))];
                    mask_2 = _e104;
                    mask_coverage_1 = 2f;
                } else {
                    let _e105 = (*pixels_per_triad);
                    if (_e105 == 5f) {
                        mask_tile_3 = array<vec3<f32>, 5>(vec3<f32>(1f, 0f, 0f), vec3<f32>(1f, 0f, 1f), vec3<f32>(0f, 0f, 1f), vec3<f32>(0f, 1f, 0f), vec3<f32>(0f, 1f, 0f));
                        let _e107 = x_1;
                        let _e114 = mask_tile_3[i32((_e107 - (floor((_e107 / 5f)) * 5f)))];
                        mask_2 = _e114;
                        mask_coverage_1 = 2.5f;
                    }
                }
            }
        }
    } else {
        let _e115 = (*pixels_per_triad);
        if (_e115 == 2f) {
            mask_tile_4 = array<vec3<f32>, 2>(vec3<f32>(1f, 0f, 1f), vec3<f32>(0f, 1f, 0f));
            let _e117 = x_1;
            let _e124 = mask_tile_4[i32((_e117 - (floor((_e117 / 2f)) * 2f)))];
            mask_2 = _e124;
            mask_coverage_1 = 2f;
        } else {
            let _e125 = (*pixels_per_triad);
            if (_e125 == 3f) {
                mask_tile_5 = array<vec3<f32>, 3>(vec3<f32>(1f, 0f, 0f), vec3<f32>(0f, 1f, 0f), vec3<f32>(0f, 0f, 1f));
                let _e127 = x_1;
                let _e134 = mask_tile_5[i32((_e127 - (floor((_e127 / 3f)) * 3f)))];
                mask_2 = _e134;
                mask_coverage_1 = 3f;
            } else {
                let _e135 = (*pixels_per_triad);
                if (_e135 == 4f) {
                    mask_tile_6 = array<vec3<f32>, 4>(vec3<f32>(0f, 0f, 1f), vec3<f32>(0f, 1f, 1f), vec3<f32>(1f, 1f, 0f), vec3<f32>(1f, 0f, 0f));
                    let _e137 = x_1;
                    let _e144 = mask_tile_6[i32((_e137 - (floor((_e137 / 4f)) * 4f)))];
                    mask_2 = _e144;
                    mask_coverage_1 = 2f;
                } else {
                    let _e145 = (*pixels_per_triad);
                    if (_e145 == 5f) {
                        mask_tile_7 = array<vec3<f32>, 5>(vec3<f32>(0f, 0f, 1f), vec3<f32>(1f, 0f, 1f), vec3<f32>(1f, 0f, 0f), vec3<f32>(0f, 1f, 0f), vec3<f32>(0f, 1f, 0f));
                        let _e147 = x_1;
                        let _e154 = mask_tile_7[i32((_e147 - (floor((_e147 / 5f)) * 5f)))];
                        mask_2 = _e154;
                        mask_coverage_1 = 2.5f;
                    }
                }
            }
        }
    }
    let _e155 = mask_2;
    let _e156 = mask_coverage_1;
    return vec4<f32>(_e155.x, _e155.y, _e155.z, _e156);
}

fn get_odd_lines_u0028_f1_u003b_vf4_u003b(v: ptr<function, f32>, sourceSize: ptr<function, vec4<f32>>) -> lines_t {
    var upper_sample_y: f32;
    var lower_sample_y: f32;
    var upper_distance_y: f32;
    var lower_distance_y: f32;

    let _e59 = (*v);
    let _e62 = (*sourceSize)[1u];
    upper_sample_y = ((ceil((((0.5f * _e59) * _e62) + 0.25f)) * 2f) - 0.5f);
    let _e68 = upper_sample_y;
    lower_sample_y = (_e68 - 2f);
    let _e70 = upper_sample_y;
    let _e71 = (*v);
    let _e73 = (*sourceSize)[1u];
    upper_distance_y = (0.5f * (_e70 - (_e71 * _e73)));
    let _e77 = upper_distance_y;
    lower_distance_y = (_e77 - 1f);
    let _e80 = (*sourceSize)[3u];
    let _e81 = upper_sample_y;
    upper_sample_y = (_e81 * _e80);
    let _e84 = (*sourceSize)[3u];
    let _e85 = lower_sample_y;
    lower_sample_y = (_e85 * _e84);
    let _e87 = upper_sample_y;
    let _e88 = lower_sample_y;
    let _e89 = upper_distance_y;
    let _e90 = lower_distance_y;
    return lines_t(_e87, _e88, _e89, _e90);
}

fn get_even_lines_u0028_f1_u003b_vf4_u003b(v_1: ptr<function, f32>, sourceSize_1: ptr<function, vec4<f32>>) -> lines_t {
    var upper_sample_y_1: f32;
    var lower_sample_y_1: f32;
    var upper_distance_y_1: f32;
    var lower_distance_y_1: f32;

    let _e59 = (*v_1);
    let _e62 = (*sourceSize_1)[1u];
    upper_sample_y_1 = ((ceil((((0.5f * _e59) * _e62) - 0.25f)) * 2f) + 0.5f);
    let _e68 = upper_sample_y_1;
    lower_sample_y_1 = (_e68 - 2f);
    let _e70 = upper_sample_y_1;
    let _e71 = (*v_1);
    let _e73 = (*sourceSize_1)[1u];
    upper_distance_y_1 = (0.5f * (_e70 - (_e71 * _e73)));
    let _e77 = upper_distance_y_1;
    lower_distance_y_1 = (_e77 - 1f);
    let _e80 = (*sourceSize_1)[3u];
    let _e81 = upper_sample_y_1;
    upper_sample_y_1 = (_e81 * _e80);
    let _e84 = (*sourceSize_1)[3u];
    let _e85 = lower_sample_y_1;
    lower_sample_y_1 = (_e85 * _e84);
    let _e87 = upper_sample_y_1;
    let _e88 = lower_sample_y_1;
    let _e89 = upper_distance_y_1;
    let _e90 = lower_distance_y_1;
    return lines_t(_e87, _e88, _e89, _e90);
}

fn spot_u0028_vf3_u003b_vf3_u003b_f1_u003b_f1_u003b(sample_value: ptr<function, vec3<f32>>, width_rcp: ptr<function, vec3<f32>>, distance_x: ptr<function, f32>, distance_y: ptr<function, f32>) -> vec3<f32> {
    var x_2: vec3<f32>;
    var y: vec3<f32>;

    let _e59 = (*distance_x);
    let _e61 = (*width_rcp);
    x_2 = clamp((_e61 * abs(_e59)), vec3(0f), vec3(1f));
    let _e66 = (*distance_y);
    let _e68 = (*width_rcp);
    y = clamp((_e68 * abs(_e66)), vec3(0f), vec3(1f));
    let _e73 = (*sample_value);
    let _e74 = (*width_rcp);
    let _e76 = (*width_rcp);
    let _e78 = x_2;
    let _e79 = x_2;
    let _e81 = x_2;
    let _e89 = y;
    let _e90 = y;
    let _e92 = y;
    return ((((_e73 * _e74) * _e76) * (((_e78 * _e79) * ((_e81 * 2f) - vec3(3f))) + vec3(1f))) * (((_e89 * _e90) * ((_e92 * 2f) - vec3(3f))) + vec3(1f)));
}

fn scanlines_u0028_struct_u002d_lines_t_u002d_f1_u002d_f1_u002d_f1_u002d_f11_u003b_f1_u003b(lines: ptr<function, lines_t>, u: ptr<function, f32>) -> vec3<f32> {
    var value: vec3<f32>;
    var start_x: f32;
    var stop_x: f32;
    var distance_x_1: f32;
    var sample_x: f32;
    var upper_sample: vec3<f32>;
    var upper_width_rcp: vec3<f32>;
    var lower_sample: vec3<f32>;
    var lower_width_rcp: vec3<f32>;
    var param: vec3<f32>;
    var param_1: vec3<f32>;
    var param_2: f32;
    var param_3: f32;
    var param_4: vec3<f32>;
    var param_5: vec3<f32>;
    var param_6: f32;
    var param_7: f32;

    value = vec3<f32>(0f, 0f, 0f);
    let _e74 = params.SourceSize[2u];
    let _e75 = (*u);
    let _e78 = params.SourceSize[0u];
    let _e81 = params.MaxSpotSize;
    let _e82 = delta_1;
    start_x = (_e74 * (round(((_e75 * _e78) - (_e81 / _e82))) + 0.5f));
    let _e90 = params.SourceSize[2u];
    let _e91 = (*u);
    let _e94 = params.SourceSize[0u];
    let _e97 = params.MaxSpotSize;
    let _e98 = delta_1;
    stop_x = (_e90 * round(((_e91 * _e94) + (_e97 / _e98))));
    let _e103 = delta_1;
    let _e106 = params.SourceSize[0u];
    let _e108 = start_x;
    let _e109 = (*u);
    distance_x_1 = ((_e103 * _e106) * (_e108 - _e109));
    let _e112 = start_x;
    sample_x = _e112;
    loop {
        let _e113 = sample_x;
        let _e114 = stop_x;
        if (_e113 < _e114) {
            let _e116 = sample_x;
            let _e118 = (*lines).upper_sample_y;
            let _e120 = textureSampleLevel(Filtered, _Filtered_sampler, vec2<f32>(_e116, _e118), 0f);
            upper_sample = _e120.xyz;
            let _e122 = sample_x;
            let _e124 = (*lines).upper_sample_y;
            let _e126 = textureSampleLevel(Source, _Source_sampler, vec2<f32>(_e122, _e124), 0f);
            upper_width_rcp = _e126.xyz;
            let _e128 = sample_x;
            let _e130 = (*lines).lower_sample_y;
            let _e132 = textureSampleLevel(Filtered, _Filtered_sampler, vec2<f32>(_e128, _e130), 0f);
            lower_sample = _e132.xyz;
            let _e134 = sample_x;
            let _e136 = (*lines).lower_sample_y;
            let _e138 = textureSampleLevel(Source, _Source_sampler, vec2<f32>(_e134, _e136), 0f);
            lower_width_rcp = _e138.xyz;
            let _e140 = upper_sample;
            param = _e140;
            let _e141 = upper_width_rcp;
            param_1 = _e141;
            let _e142 = distance_x_1;
            param_2 = _e142;
            let _e144 = (*lines).upper_distance_y;
            param_3 = _e144;
            let _e145 = spot_u0028_vf3_u003b_vf3_u003b_f1_u003b_f1_u003b((&param), (&param_1), (&param_2), (&param_3));
            let _e146 = value;
            value = (_e146 + _e145);
            let _e148 = lower_sample;
            param_4 = _e148;
            let _e149 = lower_width_rcp;
            param_5 = _e149;
            let _e150 = distance_x_1;
            param_6 = _e150;
            let _e152 = (*lines).lower_distance_y;
            param_7 = _e152;
            let _e153 = spot_u0028_vf3_u003b_vf3_u003b_f1_u003b_f1_u003b((&param_4), (&param_5), (&param_6), (&param_7));
            let _e154 = value;
            value = (_e154 + _e153);
            let _e156 = delta_1;
            let _e157 = distance_x_1;
            distance_x_1 = (_e157 + _e156);
            continue;
        } else {
            break;
        }
        continuing {
            let _e161 = params.SourceSize[2u];
            let _e162 = sample_x;
            sample_x = (_e162 + _e161);
        }
    }
    let _e164 = delta_1;
    let _e166 = params.MaxSpotSize;
    let _e168 = value;
    return (_e168 * (_e164 * _e166));
}

fn get_closest_lines_u0028_f1_u003b_vf4_u003b(v_2: ptr<function, f32>, sourceSize_2: ptr<function, vec4<f32>>) -> lines_t {
    var upper_sample_y_2: f32;
    var lower_sample_y_2: f32;
    var upper_distance_y_2: f32;
    var lower_distance_y_2: f32;

    let _e59 = (*v_2);
    let _e61 = (*sourceSize_2)[1u];
    upper_sample_y_2 = (round((_e59 * _e61)) + 0.5f);
    let _e65 = upper_sample_y_2;
    lower_sample_y_2 = (_e65 - 1f);
    let _e67 = upper_sample_y_2;
    let _e68 = (*v_2);
    let _e70 = (*sourceSize_2)[1u];
    upper_distance_y_2 = (_e67 - (_e68 * _e70));
    let _e73 = upper_distance_y_2;
    lower_distance_y_2 = (_e73 - 1f);
    let _e76 = (*sourceSize_2)[3u];
    let _e77 = upper_sample_y_2;
    upper_sample_y_2 = (_e77 * _e76);
    let _e80 = (*sourceSize_2)[3u];
    let _e81 = lower_sample_y_2;
    lower_sample_y_2 = (_e81 * _e80);
    let _e83 = upper_sample_y_2;
    let _e84 = lower_sample_y_2;
    let _e85 = upper_distance_y_2;
    let _e86 = lower_distance_y_2;
    return lines_t(_e83, _e84, _e85, _e86);
}

fn main_1() {
    var value_1: vec3<f32>;
    var lines_1: lines_t;
    var param_8: f32;
    var param_9: vec4<f32>;
    var param_10: lines_t;
    var param_11: f32;
    var lines_even: lines_t;
    var param_12: f32;
    var param_13: vec4<f32>;
    var param_14: lines_t;
    var param_15: f32;
    var lines_odd: lines_t;
    var param_16: f32;
    var param_17: vec4<f32>;
    var param_18: lines_t;
    var param_19: f32;
    var lines_2: lines_t;
    var param_20: f32;
    var param_21: vec4<f32>;
    var param_22: lines_t;
    var param_23: f32;
    var lines_3: lines_t;
    var param_24: f32;
    var param_25: vec4<f32>;
    var param_26: lines_t;
    var param_27: f32;
    var mask_3: vec4<f32>;
    var param_28: vec2<f32>;
    var param_29: vec4<f32>;
    var param_30: f32;
    var param_31: f32;
    var param_32: vec3<f32>;
    var param_33: vec3<f32>;
    var param_34: f32;
    var mask_4: vec4<f32>;
    var param_35: vec2<f32>;
    var param_36: vec4<f32>;
    var param_37: f32;
    var param_38: vec3<f32>;
    var param_39: vec3<f32>;
    var param_40: f32;

    value_1 = vec3<f32>(0f, 0f, 0f);
    let _e96 = params.SourceSize[1u];
    if (_e96 <= 300f) {
        let _e99 = vTexCoord_1[1u];
        param_8 = _e99;
        let _e101 = params.SourceSize;
        param_9 = _e101;
        let _e102 = get_closest_lines_u0028_f1_u003b_vf4_u003b((&param_8), (&param_9));
        lines_1 = _e102;
        let _e103 = lines_1;
        param_10 = _e103;
        let _e105 = vTexCoord_1[0u];
        param_11 = _e105;
        let _e106 = scanlines_u0028_struct_u002d_lines_t_u002d_f1_u002d_f1_u002d_f1_u002d_f11_u003b_f1_u003b((&param_10), (&param_11));
        value_1 = _e106;
    } else {
        let _e108 = params.OddFieldFirst;
        if (_e108 == 2f) {
            let _e111 = vTexCoord_1[1u];
            param_12 = _e111;
            let _e113 = params.SourceSize;
            param_13 = _e113;
            let _e114 = get_even_lines_u0028_f1_u003b_vf4_u003b((&param_12), (&param_13));
            lines_even = _e114;
            let _e115 = lines_even;
            param_14 = _e115;
            let _e117 = vTexCoord_1[0u];
            param_15 = _e117;
            let _e118 = scanlines_u0028_struct_u002d_lines_t_u002d_f1_u002d_f1_u002d_f1_u002d_f11_u003b_f1_u003b((&param_14), (&param_15));
            value_1 = _e118;
            let _e120 = vTexCoord_1[1u];
            param_16 = _e120;
            let _e122 = params.SourceSize;
            param_17 = _e122;
            let _e123 = get_odd_lines_u0028_f1_u003b_vf4_u003b((&param_16), (&param_17));
            lines_odd = _e123;
            let _e124 = value_1;
            let _e125 = lines_odd;
            param_18 = _e125;
            let _e127 = vTexCoord_1[0u];
            param_19 = _e127;
            let _e128 = scanlines_u0028_struct_u002d_lines_t_u002d_f1_u002d_f1_u002d_f1_u002d_f11_u003b_f1_u003b((&param_18), (&param_19));
            value_1 = ((_e124 + _e128) * 0.5f);
        } else {
            let _e132 = params.FrameCount;
            let _e134 = params.OddFieldFirst;
            if (((_e132 + select(0u, 1u, (_e134 == 1f))) % 2u) == 0u) {
                let _e141 = vTexCoord_1[1u];
                param_20 = _e141;
                let _e143 = params.SourceSize;
                param_21 = _e143;
                let _e144 = get_odd_lines_u0028_f1_u003b_vf4_u003b((&param_20), (&param_21));
                lines_2 = _e144;
                let _e145 = lines_2;
                param_22 = _e145;
                let _e147 = vTexCoord_1[0u];
                param_23 = _e147;
                let _e148 = scanlines_u0028_struct_u002d_lines_t_u002d_f1_u002d_f1_u002d_f1_u002d_f11_u003b_f1_u003b((&param_22), (&param_23));
                value_1 = _e148;
            } else {
                let _e150 = vTexCoord_1[1u];
                param_24 = _e150;
                let _e152 = params.SourceSize;
                param_25 = _e152;
                let _e153 = get_even_lines_u0028_f1_u003b_vf4_u003b((&param_24), (&param_25));
                lines_3 = _e153;
                let _e154 = lines_3;
                param_26 = _e154;
                let _e156 = vTexCoord_1[0u];
                param_27 = _e156;
                let _e157 = scanlines_u0028_struct_u002d_lines_t_u002d_f1_u002d_f1_u002d_f1_u002d_f11_u003b_f1_u003b((&param_26), (&param_27));
                value_1 = _e157;
            }
        }
    }
    let _e159 = params.MaskType;
    if (_e159 == 1f) {
        let _e161 = viewportCoord_1;
        param_28 = _e161;
        let _e163 = params.OutputSize;
        param_29 = _e163;
        let _e165 = params.SubpixelPattern;
        param_30 = _e165;
        let _e167 = params.SubpixelMaskPattern;
        param_31 = _e167;
        let _e168 = subpixel_aperture_grille_u0028_vf2_u003b_vf4_u003b_f1_u003b_f1_u003b((&param_28), (&param_29), (&param_30), (&param_31));
        mask_3 = _e168;
        let _e169 = mask_3;
        param_32 = _e169.xyz;
        let _e171 = value_1;
        param_33 = _e171;
        let _e173 = mask_3[3u];
        param_34 = _e173;
        let _e174 = blend_mask_u0028_vf3_u003b_vf3_u003b_f1_u003b((&param_32), (&param_33), (&param_34));
        value_1 = _e174;
    } else {
        let _e176 = params.MaskType;
        if (_e176 == 2f) {
            let _e178 = viewportCoord_1;
            param_35 = _e178;
            let _e180 = params.OutputSize;
            param_36 = _e180;
            let _e182 = params.DynamicMaskTriads;
            param_37 = _e182;
            let _e183 = dynamic_aperture_grille_u0028_vf2_u003b_vf4_u003b_f1_u003b((&param_35), (&param_36), (&param_37));
            mask_4 = _e183;
            let _e184 = mask_4;
            param_38 = _e184.xyz;
            let _e186 = value_1;
            param_39 = _e186;
            let _e188 = mask_4[3u];
            param_40 = _e188;
            let _e189 = blend_mask_u0028_vf3_u003b_vf3_u003b_f1_u003b((&param_38), (&param_39), (&param_40));
            value_1 = _e189;
        }
    }
    let _e190 = value_1;
    FragColor[0u] = _e190.x;
    FragColor[1u] = _e190.y;
    FragColor[2u] = _e190.z;
    return;
}

@fragment 
fn main(@location(1) @interpolate(flat) delta: f32, @location(0) vTexCoord: vec2<f32>, @location(2) viewportCoord: vec2<f32>) -> @location(0) vec4<f32> {
    delta_1 = delta;
    vTexCoord_1 = vTexCoord;
    viewportCoord_1 = viewportCoord;
    main_1();
    let _e7 = FragColor;
    return _e7;
}
