enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct LC {
    r9_: vec4<f32>,
    c2_: vec2<f32>,
    x4_: f32,
    ki: f32,
    k2_: vec4<f32>,
    D2_: vec2<f32>,
    V0_: u32,
    n2_: u32,
    Z6_: u32,
}

struct NB {
    Ub: f32,
    dd: f32,
    Xe: f32,
    Ye: f32,
    q5_: u32,
    ug: u32,
    Je: u32,
    Ke: u32,
    U7_: vec4<i32>,
    rg: vec2<f32>,
    ed: vec2<f32>,
    W1_: u32,
    vg: f32,
    Y5_: u32,
    P2_: f32,
    fd: f32,
    Ee: u32,
    y3_: f32,
    z3_: f32,
    gd: f32,
    og: u32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat) member_1: f32,
}

@id(0) override Mg: bool = true;
@id(1) override Ng: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
var<private> gl_VertexIndex_1: i32;
@group(0) @binding(2) 
var<uniform> A0_: LC;
var<private> GC_1: vec2<f32>;
var<private> U0_: vec2<f32>;
var<private> HC_1: vec2<f32>;
var<private> I3_: f32;
@group(0) @binding(0) 
var<uniform> k: NB;

fn main_1() {
    var phi_362_: f32;

    let _e29 = A0_.r9_;
    let _e37 = GC_1;
    let _e40 = A0_.c2_;
    let _e41 = ((mat2x2<f32>(vec2<f32>(_e29.x, _e29.y), vec2<f32>(_e29.z, _e29.w)) * _e37) + _e40);
    let _e42 = HC_1;
    U0_ = _e42;
    if Mg {
        let _e44 = A0_.V0_;
        let _e46 = k.Y5_;
        if (_e44 == 0u) {
            phi_362_ = 0f;
        } else {
            phi_362_ = unpack2x16float(((_e44 + 1023u) * _e46)).x;
        }
        let _e53 = phi_362_;
        I3_ = _e53;
    }
    if Ng {
        let _e55 = A0_.k2_;
        let _e64 = A0_.D2_;
        if any((_e55 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e68 = ((mat2x2<f32>(vec2<f32>(_e55.x, _e55.y), vec2<f32>(_e55.z, _e55.w)) * _e41) + _e64);
            unnamed.gl_ClipDistance[0i] = (_e68.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e68.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e68.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e68.y);
        } else {
            let _e84 = (_e64.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e84;
            unnamed.gl_ClipDistance[2i] = _e84;
            unnamed.gl_ClipDistance[1i] = _e84;
            unnamed.gl_ClipDistance[0i] = _e84;
        }
    }
    let _e94 = k.Xe;
    let _e96 = k.Ye;
    let _e104 = vec4<f32>(((_e41.x * _e94) - 1f), ((_e41.y * _e96) - sign(_e96)), 0f, 1f);
    let _e106 = A0_.Z6_;
    unnamed.gl_Position = vec4<f32>(_e104.x, _e104.y, (1f - (f32(_e106) * 0.000061035156f)), _e104.w);
    return;
}

@vertex 
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) GC: vec2<f32>, @location(1) HC: vec2<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    GC_1 = GC;
    HC_1 = HC;
    main_1();
    let _e12 = unnamed.gl_Position;
    let _e13 = unnamed.gl_ClipDistance;
    let _e14 = U0_;
    let _e15 = I3_;
    return VertexOutput(_e12, _e13, _e14, _e15);
}
