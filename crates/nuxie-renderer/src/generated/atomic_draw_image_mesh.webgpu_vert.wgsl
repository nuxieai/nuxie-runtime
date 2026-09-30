struct UB {
    Rc: f32,
    Ud: f32,
    cg: f32,
    dg: f32,
    B6_: u32,
    Y9_: u32,
    Of: u32,
    Pf: u32,
    k8_: vec4<i32>,
    Mh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Qh: f32,
    U4_: u32,
    c3_: f32,
    Wd: f32,
    If: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Jh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct nh {
    k2_: array<vec4<u32>>,
}

struct Ff {
    k2_: array<vec2<u32>>,
}

struct Gf {
    k2_: array<vec4<f32>>,
}

struct oh {
    k2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @location(1) member_1: vec4<f32>,
    @location(3) @interpolate(flat, either) member_2: vec4<f32>,
    @location(4) @interpolate(flat, either) member_3: u32,
    @location(5) @interpolate(flat, either) member_4: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(1) override mi: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> YB_1: vec4<f32>;
var<private> QC_1: vec2<f32>;
var<private> PB_1: vec4<f32>;
var<private> f2_: vec2<f32>;
var<private> RC_1: vec2<f32>;
var<private> IC_1: vec4<f32>;
var<private> S0_: vec4<f32>;
var<private> SB_1: vec4<f32>;
var<private> Q1_: vec4<f32>;
var<private> ZB_1: u32;
var<private> J3_: u32;
var<private> AC_1: u32;
var<private> H1_: u32;
var<private> BC_1: u32;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> LB: nh;
@group(0) @binding(3)
var<storage> XC: Ff;
@group(0) @binding(4)
var<storage> JB: Gf;
@group(0) @binding(5)
var<storage> AD: oh;
@group(3) @binding(9)
var xa: sampler;
var<private> MC_1: u32;

fn main_1() {
    var phi_318_: bool;
    var phi_381_: vec4<f32>;

    let _e38 = YB_1;
    let _e46 = QC_1;
    let _e48 = PB_1;
    let _e50 = ((mat2x2<f32>(vec2<f32>(_e38.x, _e38.y), vec2<f32>(_e38.z, _e38.w)) * _e46) + _e48.xy);
    let _e51 = RC_1;
    let _e52 = IC_1;
    f2_ = ((_e51 * _e52.zw) + _e52.xy);
    if mi {
        let _e57 = SB_1;
        let _e62 = vec2<f32>(_e57.x, _e57.y);
        let _e63 = vec2<f32>(_e57.z, _e57.w);
        switch bitcast<i32>(0u) {
            default: {
                let _e69 = (abs(_e62) + abs(_e63));
                let _e71 = (_e69.x != 0f);
                phi_318_ = _e71;
                if _e71 {
                    phi_318_ = (_e69.y != 0f);
                }
                let _e75 = phi_318_;
                if _e75 {
                    let _e79 = ((mat2x2<f32>(_e62, _e63) * _e50) + _e48.zw);
                    let _e80 = -(_e79);
                    let _e86 = (vec2<f32>(1f, 1f) / _e69).xyxy;
                    phi_381_ = (((vec4<f32>(_e79.x, _e79.y, _e80.x, _e80.y) * _e86) + _e86) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_381_ = _e48.zwzw;
                    break;
                }
            }
        }
        let _e91 = phi_381_;
        S0_ = _e91;
    }
    let _e92 = ZB_1;
    Q1_ = unpack4x8unorm(_e92);
    let _e94 = AC_1;
    J3_ = _e94;
    let _e95 = BC_1;
    H1_ = _e95;
    let _e97 = j.cg;
    let _e99 = j.dg;
    unnamed.gl_Position = vec4<f32>(((_e50.x * _e97) - 1f), ((_e50.y * _e99) - sign(_e99)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(2) YB: vec4<f32>, @location(0) QC: vec2<f32>, @location(4) PB: vec4<f32>, @location(1) RC: vec2<f32>, @location(9) IC: vec4<f32>, @location(3) SB: vec4<f32>, @location(5) ZB: u32, @location(6) AC: u32, @location(7) BC: u32, @location(8) MC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    YB_1 = YB;
    QC_1 = QC;
    PB_1 = PB;
    RC_1 = RC;
    IC_1 = IC;
    SB_1 = SB;
    ZB_1 = ZB;
    AC_1 = AC;
    BC_1 = BC;
    MC_1 = MC;
    main_1();
    let _e33 = f2_;
    let _e34 = S0_;
    let _e35 = Q1_;
    let _e36 = J3_;
    let _e37 = H1_;
    let _e38 = unnamed.gl_Position;
    return VertexOutput(_e33, _e34, _e35, _e36, _e37, _e38);
}
