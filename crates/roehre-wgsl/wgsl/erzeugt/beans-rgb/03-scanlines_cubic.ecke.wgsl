// ERZEUGT aus fremden MIT-Shadern bzw. roehre (MIT OR Apache-2.0) — eigen/beans/rgb.slangp, Pass 3 »scanlines_cubic« (Herkunft: shader/fremd/mit/).
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

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat) member_1: f32,
    @location(2) member_2: vec2<f32>,
}

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(0) 
var<uniform> unnamed_1: UBO;
var<private> Position_1: vec4<f32>;
var<private> vTexCoord: vec2<f32>;
@group(0) @binding(3) 
var<uniform> params: Push;
var<private> TexCoord_1: vec2<f32>;
var<private> delta: f32;
var<private> viewportCoord: vec2<f32>;

fn main_1() {
    var local: f32;

    let _e21 = unnamed_1.MVP;
    let _e22 = Position_1;
    unnamed.gl_Position = (_e21 * _e22);
    let _e26 = params.OverscanHorizontal;
    let _e28 = params.OverscanVertical;
    let _e32 = TexCoord_1;
    vTexCoord = (((vec2(1f) - vec2<f32>(_e26, _e28)) * (_e32 - vec2(0.5f))) + vec2(0.5f));
    let _e40 = params.OutputSize[0u];
    let _e43 = params.OutputSize[3u];
    let _e47 = params.SourceSize[1u];
    let _e51 = params.SourceSize[2u];
    let _e54 = params.OverscanVertical;
    let _e58 = params.OverscanHorizontal;
    delta = (((((_e40 * _e43) * _e47) * _e51) * (1f - _e54)) / (1f - _e58));
    let _e63 = params.SourceSize[1u];
    if (_e63 > 300f) {
        let _e65 = delta;
        local = (0.5f * _e65);
    } else {
        let _e67 = delta;
        local = _e67;
    }
    let _e68 = local;
    delta = _e68;
    let _e69 = TexCoord_1;
    viewportCoord = _e69;
    return;
}

@vertex 
fn main(@location(0) Position: vec4<f32>, @location(1) TexCoord: vec2<f32>) -> VertexOutput {
    Position_1 = Position;
    TexCoord_1 = TexCoord;
    main_1();
    let _e10 = unnamed.gl_Position.y;
    unnamed.gl_Position.y = -(_e10);
    let _e12 = unnamed.gl_Position;
    let _e13 = vTexCoord;
    let _e14 = delta;
    let _e15 = viewportCoord;
    return VertexOutput(_e12, _e13, _e14, _e15);
}
