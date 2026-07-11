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

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat) member_1: f32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override Mg: bool = true;

var<private> gl_VertexIndex_1: i32;
@group(0) @binding(2) 
var<uniform> A0_: LC;
var<private> GC_1: vec2<f32>;
var<private> U0_: vec2<f32>;
var<private> HC_1: vec2<f32>;
var<private> I3_: f32;
@group(0) @binding(0) 
var<uniform> k: NB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    var phi_272_: f32;

    let _e23 = A0_.r9_;
    let _e31 = GC_1;
    let _e34 = A0_.c2_;
    let _e35 = ((mat2x2<f32>(vec2<f32>(_e23.x, _e23.y), vec2<f32>(_e23.z, _e23.w)) * _e31) + _e34);
    let _e36 = HC_1;
    U0_ = _e36;
    if Mg {
        let _e38 = A0_.V0_;
        let _e40 = k.Y5_;
        if (_e38 == 0u) {
            phi_272_ = 0f;
        } else {
            phi_272_ = unpack2x16float(((_e38 + 1023u) * _e40)).x;
        }
        let _e47 = phi_272_;
        I3_ = _e47;
    }
    let _e49 = k.Xe;
    let _e51 = k.Ye;
    let _e59 = vec4<f32>(((_e35.x * _e49) - 1f), ((_e35.y * _e51) - sign(_e51)), 0f, 1f);
    let _e61 = A0_.Z6_;
    unnamed.gl_Position = vec4<f32>(_e59.x, _e59.y, (1f - (f32(_e61) * 0.000061035156f)), _e59.w);
    return;
}

@vertex 
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) GC: vec2<f32>, @location(1) HC: vec2<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    GC_1 = GC;
    HC_1 = HC;
    main_1();
    let _e11 = U0_;
    let _e12 = I3_;
    let _e13 = unnamed.gl_Position;
    return VertexOutput(_e11, _e12, _e13);
}
