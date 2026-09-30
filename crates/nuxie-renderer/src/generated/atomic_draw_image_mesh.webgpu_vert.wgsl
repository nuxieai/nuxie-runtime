struct TB {
    tc: f32,
    Bd: f32,
    Hf: f32,
    If: f32,
    o6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    T7_: vec4<i32>,
    hh: vec2<f32>,
    Cd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    X2_: f32,
    Dd: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Ed: f32,
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

struct Ig {
    g2_: array<vec4<u32>>,
}

struct kf {
    g2_: array<vec2<u32>>,
}

struct lf {
    g2_: array<vec4<f32>>,
}

struct Jg {
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
var<private> PC_1: vec2<f32>;
var<private> NB_1: vec4<f32>;
var<private> c2_: vec2<f32>;
var<private> QC_1: vec2<f32>;
var<private> GC_1: vec4<f32>;
var<private> P0_: vec4<f32>;
var<private> RB_1: vec4<f32>;
var<private> K1_: vec4<f32>;
var<private> YB_1: u32;
var<private> B3_: u32;
var<private> ZB_1: u32;
var<private> C1_: u32;
var<private> AC_1: u32;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var MC: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> OB: Ig;
@group(0) @binding(3)
var<storage> DD: kf;
@group(0) @binding(4)
var<storage> PB: lf;
@group(0) @binding(5)
var<storage> ID: Jg;
@group(3) @binding(9)
var ca: sampler;
var<private> LC_1: u32;

fn main_1() {
    var phi_319_: bool;
    var phi_382_: vec4<f32>;

    let _e38 = XB_1;
    let _e46 = PC_1;
    let _e48 = NB_1;
    let _e50 = ((mat2x2<f32>(vec2<f32>(_e38.x, _e38.y), vec2<f32>(_e38.z, _e38.w)) * _e46) + _e48.xy);
    let _e51 = QC_1;
    let _e52 = GC_1;
    c2_ = ((_e51 * _e52.zw) + _e52.xy);
    if Ih {
        let _e57 = RB_1;
        let _e62 = vec2<f32>(_e57.x, _e57.y);
        let _e63 = vec2<f32>(_e57.z, _e57.w);
        switch bitcast<i32>(0u) {
            default: {
                let _e69 = (abs(_e62) + abs(_e63));
                let _e71 = (_e69.x != 0f);
                phi_319_ = _e71;
                if _e71 {
                    phi_319_ = (_e69.y != 0f);
                }
                let _e75 = phi_319_;
                if _e75 {
                    let _e79 = ((mat2x2<f32>(_e62, _e63) * _e50) + _e48.zw);
                    let _e80 = -(_e79);
                    let _e86 = (vec2<f32>(1f, 1f) / _e69).xyxy;
                    phi_382_ = (((vec4<f32>(_e79.x, _e79.y, _e80.x, _e80.y) * _e86) + _e86) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_382_ = _e48.zwzw;
                    break;
                }
            }
        }
        let _e91 = phi_382_;
        P0_ = _e91;
    }
    let _e92 = YB_1;
    K1_ = unpack4x8unorm(_e92);
    let _e94 = ZB_1;
    B3_ = _e94;
    let _e95 = AC_1;
    C1_ = _e95;
    let _e97 = j.Hf;
    let _e99 = j.If;
    unnamed.gl_Position = vec4<f32>(((_e50.x * _e97) - 1f), ((_e50.y * _e99) - sign(_e99)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(2) XB: vec4<f32>, @location(0) PC: vec2<f32>, @location(4) NB: vec4<f32>, @location(1) QC: vec2<f32>, @location(9) GC: vec4<f32>, @location(3) RB: vec4<f32>, @location(5) YB: u32, @location(6) ZB: u32, @location(7) AC: u32, @location(8) LC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    XB_1 = XB;
    PC_1 = PC;
    NB_1 = NB;
    QC_1 = QC;
    GC_1 = GC;
    RB_1 = RB;
    YB_1 = YB;
    ZB_1 = ZB;
    AC_1 = AC;
    LC_1 = LC;
    main_1();
    let _e33 = c2_;
    let _e34 = P0_;
    let _e35 = K1_;
    let _e36 = B3_;
    let _e37 = C1_;
    let _e38 = unnamed.gl_Position;
    return VertexOutput(_e33, _e34, _e35, _e36, _e37, _e38);
}
