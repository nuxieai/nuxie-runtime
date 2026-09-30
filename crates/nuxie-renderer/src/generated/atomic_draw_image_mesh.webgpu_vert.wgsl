struct TB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    hh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    eh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Hg {
    g2_: array<vec4<u32>>,
}

struct kf {
    g2_: array<vec2<u32>>,
}

struct lf {
    g2_: array<vec4<f32>>,
}

struct Ig {
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

@id(1) override Ih: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> XB_1: vec4<f32>;
var<private> OC_1: vec2<f32>;
var<private> NB_1: vec4<f32>;
var<private> c2_: vec2<f32>;
var<private> PC_1: vec2<f32>;
var<private> O0_: vec4<f32>;
var<private> RB_1: vec4<f32>;
var<private> K1_: vec4<f32>;
var<private> YB_1: u32;
var<private> B3_: u32;
var<private> ZB_1: u32;
var<private> D1_: u32;
var<private> AC_1: u32;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> OB: Hg;
@group(0) @binding(3)
var<storage> CD: kf;
@group(0) @binding(4)
var<storage> PB: lf;
@group(0) @binding(5)
var<storage> HD: Ig;
@group(3) @binding(9)
var ca: sampler;
var<private> LC_1: u32;

fn main_1() {
    var phi_312_: bool;
    var phi_375_: vec4<f32>;

    let _e37 = XB_1;
    let _e45 = OC_1;
    let _e47 = NB_1;
    let _e49 = ((mat2x2<f32>(vec2<f32>(_e37.x, _e37.y), vec2<f32>(_e37.z, _e37.w)) * _e45) + _e47.xy);
    let _e50 = PC_1;
    c2_ = _e50;
    if Ih {
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
        O0_ = _e85;
    }
    let _e86 = YB_1;
    K1_ = unpack4x8unorm(_e86);
    let _e88 = ZB_1;
    B3_ = _e88;
    let _e89 = AC_1;
    D1_ = _e89;
    let _e91 = j.Hf;
    let _e93 = j.If;
    unnamed.gl_Position = vec4<f32>(((_e49.x * _e91) - 1f), ((_e49.y * _e93) - sign(_e93)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(2) XB: vec4<f32>, @location(0) OC: vec2<f32>, @location(4) NB: vec4<f32>, @location(1) PC: vec2<f32>, @location(3) RB: vec4<f32>, @location(5) YB: u32, @location(6) ZB: u32, @location(7) AC: u32, @location(8) LC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    XB_1 = XB;
    OC_1 = OC;
    NB_1 = NB;
    PC_1 = PC;
    RB_1 = RB;
    YB_1 = YB;
    ZB_1 = ZB;
    AC_1 = AC;
    LC_1 = LC;
    main_1();
    let _e31 = c2_;
    let _e32 = O0_;
    let _e33 = K1_;
    let _e34 = B3_;
    let _e35 = D1_;
    let _e36 = unnamed.gl_Position;
    return VertexOutput(_e31, _e32, _e33, _e34, _e35, _e36);
}
