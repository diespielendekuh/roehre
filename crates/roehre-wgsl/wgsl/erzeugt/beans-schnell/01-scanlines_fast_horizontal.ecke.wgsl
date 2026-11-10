// ERZEUGT aus fremden MIT-Shadern bzw. roehre (MIT OR Apache-2.0) — eigen/beans/schnell.slangp, Pass 1 »scanlines_fast_horizontal« (Herkunft: shader/fremd/mit/).
// Übersetzt von librashader 0.12.1 (slang → SPIR-V → naga → WGSL). Nicht von Hand ändern.
diagnostic(off, derivative_uniformity);
struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct UBO {
    MVP: mat4x4<f32>,
}

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

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat) member_1: f32,
}

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(0) 
var<uniform> unnamed_1: UBO;
var<private> Position_1: vec4<f32>;
var<private> vTexCoord: vec2<f32>;
var<private> TexCoord_1: vec2<f32>;
var<private> delta: f32;
@group(0) @binding(2) 
var<uniform> params: Push;

fn main_1() {
    var phi_79_: bool;
    var phi_96_: bool;

    let _e23 = unnamed_1.MVP;
    let _e24 = Position_1;
    unnamed.gl_Position = (_e23 * _e24);
    let _e27 = TexCoord_1;
    vTexCoord = _e27;
    let _e30 = params.FinalViewportSize[0u];
    let _e33 = params.FinalViewportSize[3u];
    let _e37 = params.SourceSize[1u];
    let _e41 = params.SourceSize[2u];
    let _e44 = params.OverscanVertical;
    let _e48 = params.OverscanHorizontal;
    delta = (((((_e30 * _e33) * _e37) * _e41) * (1f - _e44)) / (1f - _e48));
    let _e52 = params.OddFieldFirst;
    let _e53 = (_e52 <= 2f);
    phi_79_ = _e53;
    if _e53 {
        let _e56 = params.SourceSize[1u];
        phi_79_ = (_e56 > 300f);
    }
    let _e59 = phi_79_;
    if _e59 {
        let _e60 = delta;
        delta = (0.5f * _e60);
    } else {
        let _e63 = params.OddFieldFirst;
        let _e64 = (_e63 == 3f);
        phi_96_ = _e64;
        if _e64 {
            let _e67 = params.SourceSize[1u];
            phi_96_ = (_e67 < 350f);
        }
        let _e70 = phi_96_;
        if _e70 {
            let _e71 = delta;
            delta = (2f * _e71);
        }
    }
    return;
}

@vertex 
fn main(@location(0) Position: vec4<f32>, @location(1) TexCoord: vec2<f32>) -> VertexOutput {
    Position_1 = Position;
    TexCoord_1 = TexCoord;
    main_1();
    let _e9 = unnamed.gl_Position.y;
    unnamed.gl_Position.y = -(_e9);
    let _e11 = unnamed.gl_Position;
    let _e12 = vTexCoord;
    let _e13 = delta;
    return VertexOutput(_e11, _e12, _e13);
}
