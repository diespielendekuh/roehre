// ERZEUGT aus crt-beans (Andrew Duffey, MIT) bzw. roehre (MIT OR Apache-2.0) — eigen/beans/schnell.slangp, Pass 2 »scanlines_fast_vertical«.
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
    OutputGamma: f32,
}

struct UBO {
    MVP: mat4x4<f32>,
}

@group(0) @binding(1) 
var Source: texture_2d<f32>;
@group(0) @binding(2) 
var<uniform> params: Push;
var<private> vTexCoord_1: vec2<f32>;
var<private> FragColor: vec4<f32>;
@group(0) @binding(0) 
var<uniform> unnamed: UBO;
@group(1) @binding(1) 
var _Source_sampler: sampler;

fn linear_to_gamma_u0028_vf3_u003b(rgb: ptr<function, vec3<f32>>) -> vec3<f32> {
    let _e65 = (*rgb);
    (*rgb) = clamp(_e65, vec3(0f), vec3(1f));
    let _e69 = (*rgb);
    return pow(_e69, vec3<f32>(0.45454547f, 0.45454547f, 0.45454547f));
}

fn linear_to_srgb_u0028_vf3_u003b(rgb_1: ptr<function, vec3<f32>>) -> vec3<f32> {
    var cutoff: vec3<bool>;
    var higher: vec3<f32>;
    var lower: vec3<f32>;

    let _e68 = (*rgb_1);
    (*rgb_1) = clamp(_e68, vec3(0f), vec3(1f));
    let _e72 = (*rgb_1);
    cutoff = (_e72 < vec3<f32>(0.0031308f, 0.0031308f, 0.0031308f));
    let _e74 = (*rgb_1);
    higher = ((vec3<f32>(1.055f, 1.055f, 1.055f) * pow(_e74, vec3<f32>(0.41666666f, 0.41666666f, 0.41666666f))) - vec3<f32>(0.055f, 0.055f, 0.055f));
    let _e78 = (*rgb_1);
    lower = (_e78 * vec3<f32>(12.92f, 12.92f, 12.92f));
    let _e80 = higher;
    let _e81 = lower;
    let _e82 = cutoff;
    return select(_e80, _e81, _e82);
}

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

    let _e88 = (*mask_triads);
    let _e91 = (*output_size)[2u];
    kernel_radius = ((3f * _e88) * _e91);
    let _e93 = (*mask_triads);
    let _e96 = (*coord)[0u];
    x = ((3f * _e93) * _e96);
    mask = vec3<f32>(0f, 0f, 0f);
    let _e98 = kernel_radius;
    if (_e98 < 0.5f) {
        let _e100 = x;
        let _e101 = x;
        let _e104 = kernel_radius;
        x1_ = clamp(((_e100 - round(_e101)) / _e104), -1f, 1f);
        let _e107 = x;
        let _e109 = (round(_e107) - 1f);
        indexable = array<vec3<f32>, 3>(vec3<f32>(1f, 0f, 0f), vec3<f32>(0f, 1f, 0f), vec3<f32>(0f, 0f, 1f));
        let _e117 = indexable[i32(((_e109 - (floor((_e109 / 3f)) * 3f)) + 0.001f))];
        p0_ = _e117;
        let _e118 = p0_;
        p1_ = _e118.zxy;
        let _e120 = p0_;
        let _e121 = p1_;
        let _e123 = p1_;
        let _e124 = p0_;
        let _e126 = x1_;
        let _e127 = x1_;
        mask = ((_e120 + _e121) + ((_e123 - _e124) * (_e126 + (sin((3.1415927f * _e127)) / 3.1415927f))));
        let _e134 = mask;
        mask = (_e134 * 0.5f);
    } else {
        let _e136 = kernel_radius;
        if (_e136 < 1f) {
            let _e138 = x;
            let _e139 = x;
            let _e143 = kernel_radius;
            x1_1 = clamp(((_e138 - (floor(_e139) + 0f)) / _e143), -1f, 1f);
            let _e146 = x;
            let _e147 = x;
            let _e151 = kernel_radius;
            x2_ = clamp(((_e146 - (floor(_e147) + 1f)) / _e151), -1f, 1f);
            let _e154 = x;
            let _e156 = (floor(_e154) - 1f);
            indexable_1 = array<vec3<f32>, 3>(vec3<f32>(1f, 0f, 0f), vec3<f32>(0f, 1f, 0f), vec3<f32>(0f, 0f, 1f));
            let _e164 = indexable_1[i32(((_e156 - (floor((_e156 / 3f)) * 3f)) + 0.001f))];
            p0_1 = _e164;
            let _e165 = p0_1;
            p1_1 = _e165.zxy;
            let _e167 = p1_1;
            p2_ = _e167.zxy;
            let _e169 = p0_1;
            let _e170 = p2_;
            let _e172 = p1_1;
            let _e173 = p0_1;
            let _e175 = x1_1;
            let _e176 = x1_1;
            let _e183 = p2_;
            let _e184 = p1_1;
            let _e186 = x2_;
            let _e187 = x2_;
            mask = (((_e169 + _e170) + ((_e172 - _e173) * (_e175 + (sin((3.1415927f * _e176)) / 3.1415927f)))) + ((_e183 - _e184) * (_e186 + (sin((3.1415927f * _e187)) / 3.1415927f))));
            let _e194 = mask;
            mask = (_e194 * 0.5f);
        } else {
            let _e196 = x;
            let _e197 = x;
            let _e201 = kernel_radius;
            x1_2 = clamp(((_e196 - (round(_e197) - 1f)) / _e201), -1f, 1f);
            let _e204 = x;
            let _e205 = x;
            let _e209 = kernel_radius;
            x2_1 = clamp(((_e204 - (round(_e205) + 0f)) / _e209), -1f, 1f);
            let _e212 = x;
            let _e213 = x;
            let _e217 = kernel_radius;
            x3_ = clamp(((_e212 - (round(_e213) + 1f)) / _e217), -1f, 1f);
            let _e220 = x;
            let _e222 = (round(_e220) - 2f);
            indexable_2 = array<vec3<f32>, 3>(vec3<f32>(1f, 0f, 0f), vec3<f32>(0f, 1f, 0f), vec3<f32>(0f, 0f, 1f));
            let _e230 = indexable_2[i32(((_e222 - (floor((_e222 / 3f)) * 3f)) + 0.001f))];
            p0_2 = _e230;
            let _e231 = p0_2;
            p1_2 = _e231.zxy;
            let _e233 = p1_2;
            p2_1 = _e233.zxy;
            let _e235 = p2_1;
            p3_ = _e235.zxy;
            let _e237 = p0_2;
            let _e238 = p3_;
            let _e240 = p1_2;
            let _e241 = p0_2;
            let _e243 = x1_2;
            let _e244 = x1_2;
            let _e251 = p2_1;
            let _e252 = p1_2;
            let _e254 = x2_1;
            let _e255 = x2_1;
            let _e262 = p3_;
            let _e263 = p2_1;
            let _e265 = x3_;
            let _e266 = x3_;
            mask = ((((_e237 + _e238) + ((_e240 - _e241) * (_e243 + (sin((3.1415927f * _e244)) / 3.1415927f)))) + ((_e251 - _e252) * (_e254 + (sin((3.1415927f * _e255)) / 3.1415927f)))) + ((_e262 - _e263) * (_e265 + (sin((3.1415927f * _e266)) / 3.1415927f))));
            let _e273 = mask;
            mask = (_e273 * 0.5f);
        }
    }
    let _e275 = mask;
    return vec4<f32>(_e275.x, _e275.y, _e275.z, 3f);
}

fn blend_mask_u0028_vf3_u003b_vf3_u003b_f1_u003b(mask_1: ptr<function, vec3<f32>>, pixel_value: ptr<function, vec3<f32>>, mask_coverage: ptr<function, f32>) -> vec3<f32> {
    var a: vec3<f32>;
    var b: vec3<f32>;

    let _e69 = (*pixel_value);
    let _e72 = (*mask_coverage);
    let _e76 = (*pixel_value);
    a = clamp(((_e69 - vec3(1f)) / vec3((1f - _e72))), vec3<f32>(0f, 0f, 0f), _e76);
    let _e78 = (*mask_coverage);
    let _e79 = (*pixel_value);
    let _e83 = (*mask_coverage);
    let _e87 = (*pixel_value);
    b = clamp(((vec3(1f) - (_e79 * _e78)) / vec3((1f - _e83))), vec3<f32>(0f, 0f, 0f), _e87);
    let _e89 = (*mask_coverage);
    let _e90 = a;
    let _e92 = (*mask_1);
    let _e94 = b;
    return (((_e90 * _e89) * _e92) + _e94);
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

    let _e80 = (*coord_1)[0u];
    let _e82 = (*output_size_1)[0u];
    x_1 = (_e80 * _e82);
    mask_2 = vec3<f32>(1f, 1f, 1f);
    mask_coverage_1 = 1f;
    let _e84 = (*subpixel_pattern);
    if (_e84 == 0f) {
        let _e86 = (*pixels_per_triad);
        if (_e86 == 2f) {
            mask_tile = array<vec3<f32>, 2>(vec3<f32>(1f, 0f, 1f), vec3<f32>(0f, 1f, 0f));
            let _e88 = x_1;
            let _e95 = mask_tile[i32((_e88 - (floor((_e88 / 2f)) * 2f)))];
            mask_2 = _e95;
            mask_coverage_1 = 2f;
        } else {
            let _e96 = (*pixels_per_triad);
            if (_e96 == 3f) {
                mask_tile_1 = array<vec3<f32>, 3>(vec3<f32>(0f, 0f, 1f), vec3<f32>(0f, 1f, 0f), vec3<f32>(1f, 0f, 0f));
                let _e98 = x_1;
                let _e105 = mask_tile_1[i32((_e98 - (floor((_e98 / 3f)) * 3f)))];
                mask_2 = _e105;
                mask_coverage_1 = 3f;
            } else {
                let _e106 = (*pixels_per_triad);
                if (_e106 == 4f) {
                    mask_tile_2 = array<vec3<f32>, 4>(vec3<f32>(1f, 0f, 0f), vec3<f32>(1f, 1f, 0f), vec3<f32>(0f, 1f, 1f), vec3<f32>(0f, 0f, 1f));
                    let _e108 = x_1;
                    let _e115 = mask_tile_2[i32((_e108 - (floor((_e108 / 4f)) * 4f)))];
                    mask_2 = _e115;
                    mask_coverage_1 = 2f;
                } else {
                    let _e116 = (*pixels_per_triad);
                    if (_e116 == 5f) {
                        mask_tile_3 = array<vec3<f32>, 5>(vec3<f32>(1f, 0f, 0f), vec3<f32>(1f, 0f, 1f), vec3<f32>(0f, 0f, 1f), vec3<f32>(0f, 1f, 0f), vec3<f32>(0f, 1f, 0f));
                        let _e118 = x_1;
                        let _e125 = mask_tile_3[i32((_e118 - (floor((_e118 / 5f)) * 5f)))];
                        mask_2 = _e125;
                        mask_coverage_1 = 2.5f;
                    }
                }
            }
        }
    } else {
        let _e126 = (*pixels_per_triad);
        if (_e126 == 2f) {
            mask_tile_4 = array<vec3<f32>, 2>(vec3<f32>(1f, 0f, 1f), vec3<f32>(0f, 1f, 0f));
            let _e128 = x_1;
            let _e135 = mask_tile_4[i32((_e128 - (floor((_e128 / 2f)) * 2f)))];
            mask_2 = _e135;
            mask_coverage_1 = 2f;
        } else {
            let _e136 = (*pixels_per_triad);
            if (_e136 == 3f) {
                mask_tile_5 = array<vec3<f32>, 3>(vec3<f32>(1f, 0f, 0f), vec3<f32>(0f, 1f, 0f), vec3<f32>(0f, 0f, 1f));
                let _e138 = x_1;
                let _e145 = mask_tile_5[i32((_e138 - (floor((_e138 / 3f)) * 3f)))];
                mask_2 = _e145;
                mask_coverage_1 = 3f;
            } else {
                let _e146 = (*pixels_per_triad);
                if (_e146 == 4f) {
                    mask_tile_6 = array<vec3<f32>, 4>(vec3<f32>(0f, 0f, 1f), vec3<f32>(0f, 1f, 1f), vec3<f32>(1f, 1f, 0f), vec3<f32>(1f, 0f, 0f));
                    let _e148 = x_1;
                    let _e155 = mask_tile_6[i32((_e148 - (floor((_e148 / 4f)) * 4f)))];
                    mask_2 = _e155;
                    mask_coverage_1 = 2f;
                } else {
                    let _e156 = (*pixels_per_triad);
                    if (_e156 == 5f) {
                        mask_tile_7 = array<vec3<f32>, 5>(vec3<f32>(0f, 0f, 1f), vec3<f32>(1f, 0f, 1f), vec3<f32>(1f, 0f, 0f), vec3<f32>(0f, 1f, 0f), vec3<f32>(0f, 1f, 0f));
                        let _e158 = x_1;
                        let _e165 = mask_tile_7[i32((_e158 - (floor((_e158 / 5f)) * 5f)))];
                        mask_2 = _e165;
                        mask_coverage_1 = 2.5f;
                    }
                }
            }
        }
    }
    let _e166 = mask_2;
    let _e167 = mask_coverage_1;
    return vec4<f32>(_e166.x, _e166.y, _e166.z, _e167);
}

fn get_odd_lines_u0028_f1_u003b_vf4_u003b(v: ptr<function, f32>, sourceSize: ptr<function, vec4<f32>>) -> lines_t {
    var upper_sample_y: f32;
    var lower_sample_y: f32;
    var upper_distance_y: f32;
    var lower_distance_y: f32;

    let _e70 = (*v);
    let _e73 = (*sourceSize)[1u];
    upper_sample_y = ((ceil((((0.5f * _e70) * _e73) + 0.25f)) * 2f) - 0.5f);
    let _e79 = upper_sample_y;
    lower_sample_y = (_e79 - 2f);
    let _e81 = upper_sample_y;
    let _e82 = (*v);
    let _e84 = (*sourceSize)[1u];
    upper_distance_y = (0.5f * (_e81 - (_e82 * _e84)));
    let _e88 = upper_distance_y;
    lower_distance_y = (_e88 - 1f);
    let _e91 = (*sourceSize)[3u];
    let _e92 = upper_sample_y;
    upper_sample_y = (_e92 * _e91);
    let _e95 = (*sourceSize)[3u];
    let _e96 = lower_sample_y;
    lower_sample_y = (_e96 * _e95);
    let _e98 = upper_sample_y;
    let _e99 = lower_sample_y;
    let _e100 = upper_distance_y;
    let _e101 = lower_distance_y;
    return lines_t(_e98, _e99, _e100, _e101);
}

fn get_even_lines_u0028_f1_u003b_vf4_u003b(v_1: ptr<function, f32>, sourceSize_1: ptr<function, vec4<f32>>) -> lines_t {
    var upper_sample_y_1: f32;
    var lower_sample_y_1: f32;
    var upper_distance_y_1: f32;
    var lower_distance_y_1: f32;

    let _e70 = (*v_1);
    let _e73 = (*sourceSize_1)[1u];
    upper_sample_y_1 = ((ceil((((0.5f * _e70) * _e73) - 0.25f)) * 2f) + 0.5f);
    let _e79 = upper_sample_y_1;
    lower_sample_y_1 = (_e79 - 2f);
    let _e81 = upper_sample_y_1;
    let _e82 = (*v_1);
    let _e84 = (*sourceSize_1)[1u];
    upper_distance_y_1 = (0.5f * (_e81 - (_e82 * _e84)));
    let _e88 = upper_distance_y_1;
    lower_distance_y_1 = (_e88 - 1f);
    let _e91 = (*sourceSize_1)[3u];
    let _e92 = upper_sample_y_1;
    upper_sample_y_1 = (_e92 * _e91);
    let _e95 = (*sourceSize_1)[3u];
    let _e96 = lower_sample_y_1;
    lower_sample_y_1 = (_e96 * _e95);
    let _e98 = upper_sample_y_1;
    let _e99 = lower_sample_y_1;
    let _e100 = upper_distance_y_1;
    let _e101 = lower_distance_y_1;
    return lines_t(_e98, _e99, _e100, _e101);
}

fn get_closest_lines_u0028_f1_u003b_vf4_u003b(v_2: ptr<function, f32>, sourceSize_2: ptr<function, vec4<f32>>) -> lines_t {
    var upper_sample_y_2: f32;
    var lower_sample_y_2: f32;
    var upper_distance_y_2: f32;
    var lower_distance_y_2: f32;

    let _e70 = (*v_2);
    let _e72 = (*sourceSize_2)[1u];
    upper_sample_y_2 = (round((_e70 * _e72)) + 0.5f);
    let _e76 = upper_sample_y_2;
    lower_sample_y_2 = (_e76 - 1f);
    let _e78 = upper_sample_y_2;
    let _e79 = (*v_2);
    let _e81 = (*sourceSize_2)[1u];
    upper_distance_y_2 = (_e78 - (_e79 * _e81));
    let _e84 = upper_distance_y_2;
    lower_distance_y_2 = (_e84 - 1f);
    let _e87 = (*sourceSize_2)[3u];
    let _e88 = upper_sample_y_2;
    upper_sample_y_2 = (_e88 * _e87);
    let _e91 = (*sourceSize_2)[3u];
    let _e92 = lower_sample_y_2;
    lower_sample_y_2 = (_e92 * _e91);
    let _e94 = upper_sample_y_2;
    let _e95 = lower_sample_y_2;
    let _e96 = upper_distance_y_2;
    let _e97 = lower_distance_y_2;
    return lines_t(_e94, _e95, _e96, _e97);
}

fn scanlines_u0028_struct_u002d_lines_t_u002d_f1_u002d_f1_u002d_f1_u002d_f11_u003b_f1_u003b(lines: ptr<function, lines_t>, u: ptr<function, f32>) -> vec3<f32> {
    var upper_sample: vec3<f32>;
    var upper_width_rcp: vec3<f32>;
    var upper_y: vec3<f32>;
    var upper_value: vec3<f32>;
    var lower_sample: vec3<f32>;
    var lower_width_rcp: vec3<f32>;
    var lower_y: vec3<f32>;
    var lower_value: vec3<f32>;

    let _e74 = (*u);
    let _e76 = (*lines).upper_sample_y;
    let _e78 = textureSampleLevel(Source, _Source_sampler, vec2<f32>(_e74, _e76), 0f);
    upper_sample = _e78.xyz;
    let _e81 = params.MinSpotSize;
    let _e83 = params.MaxSpotSize;
    let _e85 = upper_sample;
    let _e88 = params.MinSpotSize;
    let _e90 = params.MaxSpotSize;
    let _e93 = params.MaxSpotSize;
    upper_width_rcp = (vec3(1f) / (vec3((_e81 * _e83)) - (sqrt(_e85) * ((_e88 * _e90) - _e93))));
    let _e101 = (*lines).upper_distance_y;
    let _e103 = upper_width_rcp;
    upper_y = clamp((_e103 * abs(_e101)), vec3(0f), vec3(1f));
    let _e108 = upper_sample;
    let _e109 = upper_width_rcp;
    let _e111 = upper_y;
    let _e112 = upper_y;
    let _e114 = upper_y;
    upper_value = ((_e108 * _e109) * (((_e111 * _e112) * ((_e114 * 2f) - vec3(3f))) + vec3(1f)));
    let _e122 = (*u);
    let _e124 = (*lines).lower_sample_y;
    let _e126 = textureSampleLevel(Source, _Source_sampler, vec2<f32>(_e122, _e124), 0f);
    lower_sample = _e126.xyz;
    let _e129 = params.MinSpotSize;
    let _e131 = params.MaxSpotSize;
    let _e133 = lower_sample;
    let _e136 = params.MinSpotSize;
    let _e138 = params.MaxSpotSize;
    let _e141 = params.MaxSpotSize;
    lower_width_rcp = (vec3(1f) / (vec3((_e129 * _e131)) - (sqrt(_e133) * ((_e136 * _e138) - _e141))));
    let _e149 = (*lines).lower_distance_y;
    let _e151 = lower_width_rcp;
    lower_y = clamp((_e151 * abs(_e149)), vec3(0f), vec3(1f));
    let _e156 = lower_sample;
    let _e157 = lower_width_rcp;
    let _e159 = lower_y;
    let _e160 = lower_y;
    let _e162 = lower_y;
    lower_value = ((_e156 * _e157) * (((_e159 * _e160) * ((_e162 * 2f) - vec3(3f))) + vec3(1f)));
    let _e171 = params.MaxSpotSize;
    let _e172 = upper_value;
    let _e173 = lower_value;
    return ((_e172 + _e173) * _e171);
}

fn get_doubled_lines_u0028_f1_u003b_vf4_u003b(v_3: ptr<function, f32>, sourceSize_3: ptr<function, vec4<f32>>) -> lines_t {
    var upper_distance_y_3: f32;
    var lower_distance_y_3: f32;
    var upper_sample_y_3: f32;
    var lower_sample_y_3: f32;

    let _e70 = (*v_3);
    let _e72 = (*sourceSize_3)[1u];
    let _e78 = (*v_3);
    let _e80 = (*sourceSize_3)[1u];
    upper_distance_y_3 = ((ceil(((2f * (_e70 * _e72)) - 0.5f)) + 0.5f) - (2f * (_e78 * _e80)));
    let _e84 = upper_distance_y_3;
    lower_distance_y_3 = (_e84 - 1f);
    let _e86 = (*v_3);
    let _e88 = (*sourceSize_3)[1u];
    let _e94 = (*sourceSize_3)[3u];
    upper_sample_y_3 = ((ceil(((_e86 * _e88) + 0.25f)) - 0.5f) * _e94);
    let _e96 = (*v_3);
    let _e98 = (*sourceSize_3)[1u];
    let _e104 = (*sourceSize_3)[3u];
    lower_sample_y_3 = ((ceil(((_e96 * _e98) - 0.25f)) - 0.5f) * _e104);
    let _e106 = upper_sample_y_3;
    let _e107 = lower_sample_y_3;
    let _e108 = upper_distance_y_3;
    let _e109 = lower_distance_y_3;
    return lines_t(_e106, _e107, _e108, _e109);
}

fn main_1() {
    var value: vec3<f32>;
    var lines_1: lines_t;
    var param: f32;
    var param_1: vec4<f32>;
    var param_2: lines_t;
    var param_3: f32;
    var lines_2: lines_t;
    var param_4: f32;
    var param_5: vec4<f32>;
    var param_6: lines_t;
    var param_7: f32;
    var lines_even: lines_t;
    var param_8: f32;
    var param_9: vec4<f32>;
    var lines_odd: lines_t;
    var param_10: f32;
    var param_11: vec4<f32>;
    var param_12: lines_t;
    var param_13: f32;
    var param_14: lines_t;
    var param_15: f32;
    var lines_3: lines_t;
    var param_16: f32;
    var param_17: vec4<f32>;
    var param_18: lines_t;
    var param_19: f32;
    var lines_4: lines_t;
    var param_20: f32;
    var param_21: vec4<f32>;
    var param_22: lines_t;
    var param_23: f32;
    var mask_3: vec4<f32>;
    var param_24: vec2<f32>;
    var param_25: vec4<f32>;
    var param_26: f32;
    var param_27: f32;
    var param_28: vec3<f32>;
    var param_29: vec3<f32>;
    var param_30: f32;
    var mask_4: vec4<f32>;
    var param_31: vec2<f32>;
    var param_32: vec4<f32>;
    var param_33: f32;
    var param_34: vec3<f32>;
    var param_35: vec3<f32>;
    var param_36: f32;
    var param_37: vec3<f32>;
    var param_38: vec3<f32>;
    var phi_799_: bool;
    var phi_830_: bool;
    var phi_838_: bool;

    value = vec3<f32>(0f, 0f, 0f);
    let _e113 = params.OddFieldFirst;
    let _e114 = (_e113 == 3f);
    phi_799_ = _e114;
    if _e114 {
        let _e117 = params.SourceSize[1u];
        phi_799_ = (_e117 < 350f);
    }
    let _e120 = phi_799_;
    if _e120 {
        let _e122 = vTexCoord_1[1u];
        param = _e122;
        let _e124 = params.SourceSize;
        param_1 = _e124;
        let _e125 = get_doubled_lines_u0028_f1_u003b_vf4_u003b((&param), (&param_1));
        lines_1 = _e125;
        let _e126 = lines_1;
        param_2 = _e126;
        let _e128 = vTexCoord_1[0u];
        param_3 = _e128;
        let _e129 = scanlines_u0028_struct_u002d_lines_t_u002d_f1_u002d_f1_u002d_f1_u002d_f11_u003b_f1_u003b((&param_2), (&param_3));
        value = _e129;
    } else {
        let _e131 = params.OddFieldFirst;
        let _e132 = (_e131 == 4f);
        phi_830_ = _e132;
        if !(_e132) {
            let _e135 = params.OddFieldFirst;
            phi_830_ = (_e135 == 3f);
        }
        let _e138 = phi_830_;
        phi_838_ = _e138;
        if !(_e138) {
            let _e142 = params.SourceSize[1u];
            phi_838_ = (_e142 <= 300f);
        }
        let _e145 = phi_838_;
        if _e145 {
            let _e147 = vTexCoord_1[1u];
            param_4 = _e147;
            let _e149 = params.SourceSize;
            param_5 = _e149;
            let _e150 = get_closest_lines_u0028_f1_u003b_vf4_u003b((&param_4), (&param_5));
            lines_2 = _e150;
            let _e151 = lines_2;
            param_6 = _e151;
            let _e153 = vTexCoord_1[0u];
            param_7 = _e153;
            let _e154 = scanlines_u0028_struct_u002d_lines_t_u002d_f1_u002d_f1_u002d_f1_u002d_f11_u003b_f1_u003b((&param_6), (&param_7));
            value = _e154;
        } else {
            let _e156 = params.OddFieldFirst;
            if (_e156 == 2f) {
                let _e159 = vTexCoord_1[1u];
                param_8 = _e159;
                let _e161 = params.SourceSize;
                param_9 = _e161;
                let _e162 = get_even_lines_u0028_f1_u003b_vf4_u003b((&param_8), (&param_9));
                lines_even = _e162;
                let _e164 = vTexCoord_1[1u];
                param_10 = _e164;
                let _e166 = params.SourceSize;
                param_11 = _e166;
                let _e167 = get_odd_lines_u0028_f1_u003b_vf4_u003b((&param_10), (&param_11));
                lines_odd = _e167;
                let _e168 = lines_even;
                param_12 = _e168;
                let _e170 = vTexCoord_1[0u];
                param_13 = _e170;
                let _e171 = scanlines_u0028_struct_u002d_lines_t_u002d_f1_u002d_f1_u002d_f1_u002d_f11_u003b_f1_u003b((&param_12), (&param_13));
                let _e172 = lines_odd;
                param_14 = _e172;
                let _e174 = vTexCoord_1[0u];
                param_15 = _e174;
                let _e175 = scanlines_u0028_struct_u002d_lines_t_u002d_f1_u002d_f1_u002d_f1_u002d_f11_u003b_f1_u003b((&param_14), (&param_15));
                value = ((_e171 + _e175) * 0.5f);
            } else {
                let _e179 = params.FrameCount;
                let _e181 = params.OddFieldFirst;
                if (((_e179 + select(0u, 1u, (_e181 == 1f))) % 2u) == 0u) {
                    let _e188 = vTexCoord_1[1u];
                    param_16 = _e188;
                    let _e190 = params.SourceSize;
                    param_17 = _e190;
                    let _e191 = get_odd_lines_u0028_f1_u003b_vf4_u003b((&param_16), (&param_17));
                    lines_3 = _e191;
                    let _e192 = lines_3;
                    param_18 = _e192;
                    let _e194 = vTexCoord_1[0u];
                    param_19 = _e194;
                    let _e195 = scanlines_u0028_struct_u002d_lines_t_u002d_f1_u002d_f1_u002d_f1_u002d_f11_u003b_f1_u003b((&param_18), (&param_19));
                    value = _e195;
                } else {
                    let _e197 = vTexCoord_1[1u];
                    param_20 = _e197;
                    let _e199 = params.SourceSize;
                    param_21 = _e199;
                    let _e200 = get_even_lines_u0028_f1_u003b_vf4_u003b((&param_20), (&param_21));
                    lines_4 = _e200;
                    let _e201 = lines_4;
                    param_22 = _e201;
                    let _e203 = vTexCoord_1[0u];
                    param_23 = _e203;
                    let _e204 = scanlines_u0028_struct_u002d_lines_t_u002d_f1_u002d_f1_u002d_f1_u002d_f11_u003b_f1_u003b((&param_22), (&param_23));
                    value = _e204;
                }
            }
        }
    }
    let _e206 = params.MaskType;
    if (_e206 == 1f) {
        let _e208 = vTexCoord_1;
        param_24 = _e208;
        let _e210 = params.OutputSize;
        param_25 = _e210;
        let _e212 = params.SubpixelPattern;
        param_26 = _e212;
        let _e214 = params.SubpixelMaskPattern;
        param_27 = _e214;
        let _e215 = subpixel_aperture_grille_u0028_vf2_u003b_vf4_u003b_f1_u003b_f1_u003b((&param_24), (&param_25), (&param_26), (&param_27));
        mask_3 = _e215;
        let _e216 = mask_3;
        param_28 = _e216.xyz;
        let _e218 = value;
        param_29 = _e218;
        let _e220 = mask_3[3u];
        param_30 = _e220;
        let _e221 = blend_mask_u0028_vf3_u003b_vf3_u003b_f1_u003b((&param_28), (&param_29), (&param_30));
        value = _e221;
    } else {
        let _e223 = params.MaskType;
        if (_e223 == 2f) {
            let _e225 = vTexCoord_1;
            param_31 = _e225;
            let _e227 = params.OutputSize;
            param_32 = _e227;
            let _e229 = params.DynamicMaskTriads;
            param_33 = _e229;
            let _e230 = dynamic_aperture_grille_u0028_vf2_u003b_vf4_u003b_f1_u003b((&param_31), (&param_32), (&param_33));
            mask_4 = _e230;
            let _e231 = mask_4;
            param_34 = _e231.xyz;
            let _e233 = value;
            param_35 = _e233;
            let _e235 = mask_4[3u];
            param_36 = _e235;
            let _e236 = blend_mask_u0028_vf3_u003b_vf3_u003b_f1_u003b((&param_34), (&param_35), (&param_36));
            value = _e236;
        }
    }
    let _e238 = params.OutputGamma;
    if (_e238 < 0.5f) {
        let _e240 = value;
        param_37 = _e240;
        let _e241 = linear_to_srgb_u0028_vf3_u003b((&param_37));
        value = _e241;
    } else {
        let _e242 = value;
        param_38 = _e242;
        let _e243 = linear_to_gamma_u0028_vf3_u003b((&param_38));
        value = _e243;
    }
    let _e244 = value;
    FragColor[0u] = _e244.x;
    FragColor[1u] = _e244.y;
    FragColor[2u] = _e244.z;
    return;
}

@fragment 
fn main(@location(0) vTexCoord: vec2<f32>) -> @location(0) vec4<f32> {
    vTexCoord_1 = vTexCoord;
    main_1();
    let _e3 = FragColor;
    return _e3;
}
