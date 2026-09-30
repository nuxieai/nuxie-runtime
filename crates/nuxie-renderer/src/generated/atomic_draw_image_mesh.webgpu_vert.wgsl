struct BC {
    rc: f32,
    Bd: f32,
    Ff: f32,
    Gf: f32,
    q6_: u32,
    Ob: u32,
    rf: u32,
    sf: u32,
    V7_: vec4<i32>,
    ch: vec2<f32>,
    Cd: vec2<f32>,
    d2_: u32,
    gh: f32,
    f6_: u32,
    U2_: f32,
    Dd: f32,
    mf: u32,
    C3_: f32,
    D3_: f32,
    Ed: f32,
    Zg: u32,
    Nb: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Cg {
    e2_: array<vec4<u32>>,
}

struct jf {
    e2_: array<vec2<u32>>,
}

struct kf {
    e2_: array<vec4<f32>>,
}

struct Dg {
    e2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @location(1) member_1: vec4<f32>,
    @location(3) @interpolate(flat, either) member_2: vec4<f32>,
    @location(4) @interpolate(flat, either) member_3: u32,
    @location(5) @interpolate(flat, either) member_4: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(1) override Dh: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> WB_1: vec4<f32>;
var<private> PC_1: vec2<f32>;
var<private> NB_1: vec4<f32>;
var<private> Z1_: vec2<f32>;
var<private> QC_1: vec2<f32>;
var<private> M0_: vec4<f32>;
var<private> SB_1: vec4<f32>;
var<private> H1_: vec4<f32>;
var<private> XB_1: u32;
var<private> y3_: u32;
var<private> YB_1: u32;
var<private> A1_: u32;
var<private> ZB_1: u32;
@group(0) @binding(0)
var<uniform> l: BC;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> PB: Cg;
@group(0) @binding(3)
var<storage> DD: jf;
@group(0) @binding(4)
var<storage> QB: kf;
@group(0) @binding(5)
var<storage> ID: Dg;
@group(3) @binding(9)
var ea: sampler;
var<private> MC_1: u32;

fn main_1() {
    var phi_312_: bool;
    var phi_375_: vec4<f32>;

    let _e37 = WB_1;
    let _e45 = PC_1;
    let _e47 = NB_1;
    let _e49 = ((mat2x2<f32>(vec2<f32>(_e37.x, _e37.y), vec2<f32>(_e37.z, _e37.w)) * _e45) + _e47.xy);
    let _e50 = QC_1;
    Z1_ = _e50;
    if Dh {
        let _e51 = SB_1;
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
        M0_ = _e85;
    }
    let _e86 = XB_1;
    H1_ = unpack4x8unorm(_e86);
    let _e88 = YB_1;
    y3_ = _e88;
    let _e89 = ZB_1;
    A1_ = _e89;
    let _e91 = l.Ff;
    let _e93 = l.Gf;
    unnamed.gl_Position = vec4<f32>(((_e49.x * _e91) - 1f), ((_e49.y * _e93) - sign(_e93)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(2) WB: vec4<f32>, @location(0) PC: vec2<f32>, @location(4) NB: vec4<f32>, @location(1) QC: vec2<f32>, @location(3) SB: vec4<f32>, @location(5) XB: u32, @location(6) YB: u32, @location(7) ZB: u32, @location(8) MC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    WB_1 = WB;
    PC_1 = PC;
    NB_1 = NB;
    QC_1 = QC;
    SB_1 = SB;
    XB_1 = XB;
    YB_1 = YB;
    ZB_1 = ZB;
    MC_1 = MC;
    main_1();
    let _e31 = Z1_;
    let _e32 = M0_;
    let _e33 = H1_;
    let _e34 = y3_;
    let _e35 = A1_;
    let _e36 = unnamed.gl_Position;
    return VertexOutput(_e31, _e32, _e33, _e34, _e35, _e36);
}
