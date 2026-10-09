struct VB {
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct di {
    r2_: array<vec4<u32>>,
}

struct jg {
    r2_: array<vec2<u32>>,
}

struct kg {
    r2_: array<vec4<f32>>,
}

struct ei {
    r2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @location(1) member_1: vec4<f32>,
    @location(3) @interpolate(flat, either) member_2: vec4<f32>,
    @location(4) @interpolate(flat, either) member_3: u32,
    @location(5) @interpolate(flat, either) member_4: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(1) override Xi: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> ZB_1: vec4<f32>;
var<private> OC_1: vec2<f32>;
var<private> PB_1: vec4<f32>;
var<private> l2_: vec2<f32>;
var<private> PC_1: vec2<f32>;
var<private> HC_1: vec4<f32>;
var<private> V0_: vec4<f32>;
var<private> SB_1: vec4<f32>;
var<private> T1_: vec4<f32>;
var<private> AC_1: u32;
var<private> S3_: u32;
var<private> BC_1: u32;
var<private> J1_: u32;
var<private> CC_1: u32;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> KB: di;
@group(0) @binding(3)
var<storage> VC: jg;
@group(0) @binding(4)
var<storage> JB: kg;
@group(0) @binding(5)
var<storage> BD: ei;
@group(3) @binding(9)
var ab: sampler;
var<private> LC_1: u32;

fn main_1() {
    var phi_320_: bool;
    var phi_383_: vec4<f32>;

    let _e38 = ZB_1;
    let _e46 = OC_1;
    let _e48 = PB_1;
    let _e50 = ((mat2x2<f32>(vec2<f32>(_e38.x, _e38.y), vec2<f32>(_e38.z, _e38.w)) * _e46) + _e48.xy);
    let _e51 = PC_1;
    let _e52 = HC_1;
    l2_ = ((_e51 * _e52.zw) + _e52.xy);
    if Xi {
        let _e57 = SB_1;
        let _e62 = vec2<f32>(_e57.x, _e57.y);
        let _e63 = vec2<f32>(_e57.z, _e57.w);
        switch bitcast<i32>(0u) {
            default: {
                let _e69 = (abs(_e62) + abs(_e63));
                let _e71 = (_e69.x != 0f);
                phi_320_ = _e71;
                if _e71 {
                    phi_320_ = (_e69.y != 0f);
                }
                let _e75 = phi_320_;
                if _e75 {
                    let _e79 = ((mat2x2<f32>(_e62, _e63) * _e50) + _e48.zw);
                    let _e80 = -(_e79);
                    let _e86 = (vec2<f32>(1f, 1f) / _e69).xyxy;
                    phi_383_ = (((vec4<f32>(_e79.x, _e79.y, _e80.x, _e80.y) * _e86) + _e86) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_383_ = _e48.zwzw;
                    break;
                }
            }
        }
        let _e91 = phi_383_;
        V0_ = _e91;
    }
    let _e92 = AC_1;
    T1_ = unpack4x8unorm(_e92);
    let _e94 = BC_1;
    S3_ = _e94;
    let _e95 = CC_1;
    J1_ = _e95;
    let _e97 = j.Hg;
    let _e99 = j.Ig;
    unnamed.gl_Position = vec4<f32>(((_e50.x * _e97) - 1f), ((_e50.y * _e99) - sign(_e99)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(2) ZB: vec4<f32>, @location(0) OC: vec2<f32>, @location(4) PB: vec4<f32>, @location(1) PC: vec2<f32>, @location(9) HC: vec4<f32>, @location(3) SB: vec4<f32>, @location(5) AC: u32, @location(6) BC: u32, @location(7) CC: u32, @location(8) LC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    ZB_1 = ZB;
    OC_1 = OC;
    PB_1 = PB;
    PC_1 = PC;
    HC_1 = HC;
    SB_1 = SB;
    AC_1 = AC;
    BC_1 = BC;
    CC_1 = CC;
    LC_1 = LC;
    main_1();
    let _e33 = l2_;
    let _e34 = V0_;
    let _e35 = T1_;
    let _e36 = S3_;
    let _e37 = J1_;
    let _e38 = unnamed.gl_Position;
    return VertexOutput(_e33, _e34, _e35, _e36, _e37, _e38);
}
