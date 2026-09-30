struct AC {
    tc: f32,
    Dd: f32,
    Hf: f32,
    If: f32,
    q6_: u32,
    Qb: u32,
    tf: u32,
    uf: u32,
    X7_: vec4<i32>,
    eh: vec2<f32>,
    Ed: vec2<f32>,
    f2_: u32,
    ih: f32,
    f6_: u32,
    U2_: f32,
    Fd: f32,
    of_: u32,
    F3_: f32,
    G3_: f32,
    Gd: f32,
    bh: u32,
    Pb: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Eg {
    g2_: array<vec4<u32>>,
}

struct lf {
    g2_: array<vec2<u32>>,
}

struct mf {
    g2_: array<vec4<f32>>,
}

struct Fg {
    g2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @location(1) member_1: vec4<f32>,
    @location(3) @interpolate(flat, either) member_2: vec4<f32>,
    @location(4) @interpolate(flat, either) member_3: u32,
    @location(5) @interpolate(flat, either) member_4: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(1) override Fh: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> VB_1: vec4<f32>;
var<private> OC_1: vec2<f32>;
var<private> NB_1: vec4<f32>;
var<private> c2_: vec2<f32>;
var<private> PC_1: vec2<f32>;
var<private> N0_: vec4<f32>;
var<private> RB_1: vec4<f32>;
var<private> J1_: vec4<f32>;
var<private> WB_1: u32;
var<private> B3_: u32;
var<private> XB_1: u32;
var<private> C1_: u32;
var<private> YB_1: u32;
@group(0) @binding(0)
var<uniform> j: AC;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> OB: Eg;
@group(0) @binding(3)
var<storage> CD: lf;
@group(0) @binding(4)
var<storage> PB: mf;
@group(0) @binding(5)
var<storage> HD: Fg;
@group(3) @binding(9)
var ga: sampler;
var<private> LC_1: u32;

fn main_1() {
    var phi_312_: bool;
    var phi_375_: vec4<f32>;

    let _e37 = VB_1;
    let _e45 = OC_1;
    let _e47 = NB_1;
    let _e49 = ((mat2x2<f32>(vec2<f32>(_e37.x, _e37.y), vec2<f32>(_e37.z, _e37.w)) * _e45) + _e47.xy);
    let _e50 = PC_1;
    c2_ = _e50;
    if Fh {
        let _e51 = RB_1;
        let _e56 = vec2<f32>(_e51.x, _e51.y);
        let _e57 = vec2<f32>(_e51.z, _e51.w);
        switch bitcast<i32>(0u) {
            default: {
                let _e63 = (abs(_e56) + abs(_e57));
                let _e65 = (_e63.x != 0f);
                phi_312_ = _e65;
                if _e65 {
                    phi_312_ = (_e63.y != 0f);
                }
                let _e69 = phi_312_;
                if _e69 {
                    let _e73 = ((mat2x2<f32>(_e56, _e57) * _e49) + _e47.zw);
                    let _e74 = -(_e73);
                    let _e80 = (vec2<f32>(1f, 1f) / _e63).xyxy;
                    phi_375_ = (((vec4<f32>(_e73.x, _e73.y, _e74.x, _e74.y) * _e80) + _e80) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_375_ = _e47.zwzw;
                    break;
                }
            }
        }
        let _e85 = phi_375_;
        N0_ = _e85;
    }
    let _e86 = WB_1;
    J1_ = unpack4x8unorm(_e86);
    let _e88 = XB_1;
    B3_ = _e88;
    let _e89 = YB_1;
    C1_ = _e89;
    let _e91 = j.Hf;
    let _e93 = j.If;
    unnamed.gl_Position = vec4<f32>(((_e49.x * _e91) - 1f), ((_e49.y * _e93) - sign(_e93)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(2) VB: vec4<f32>, @location(0) OC: vec2<f32>, @location(4) NB: vec4<f32>, @location(1) PC: vec2<f32>, @location(3) RB: vec4<f32>, @location(5) WB: u32, @location(6) XB: u32, @location(7) YB: u32, @location(8) LC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    VB_1 = VB;
    OC_1 = OC;
    NB_1 = NB;
    PC_1 = PC;
    RB_1 = RB;
    WB_1 = WB;
    XB_1 = XB;
    YB_1 = YB;
    LC_1 = LC;
    main_1();
    let _e31 = c2_;
    let _e32 = N0_;
    let _e33 = J1_;
    let _e34 = B3_;
    let _e35 = C1_;
    let _e36 = unnamed.gl_Position;
    return VertexOutput(_e31, _e32, _e33, _e34, _e35, _e36);
}
