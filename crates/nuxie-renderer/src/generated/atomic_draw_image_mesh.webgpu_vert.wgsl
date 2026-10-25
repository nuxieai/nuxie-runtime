struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
    P8_: f32,
    Q8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct li {
    v2_: array<vec4<u32>>,
}

struct og {
    v2_: array<vec2<u32>>,
}

struct pg {
    v2_: array<vec4<f32>>,
}

struct mi {
    v2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @location(1) member_1: vec4<f32>,
    @location(3) @interpolate(flat, either) member_2: vec4<f32>,
    @location(4) @interpolate(flat, either) member_3: u32,
    @location(5) @interpolate(flat, either) member_4: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(1) override gj: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> AC_1: vec4<f32>;
var<private> PC_1: vec2<f32>;
var<private> PB_1: vec4<f32>;
var<private> n2_: vec2<f32>;
var<private> QC_1: vec2<f32>;
var<private> IC_1: vec4<f32>;
var<private> W0_: vec4<f32>;
var<private> TB_1: vec4<f32>;
var<private> Y1_: vec4<f32>;
var<private> BC_1: u32;
var<private> U3_: u32;
var<private> CC_1: u32;
var<private> N1_: u32;
var<private> DC_1: u32;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var VB: texture_2d<u32>;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(0) @binding(2)
var<storage> KB: li;
@group(0) @binding(3)
var<storage> XC: og;
@group(0) @binding(4)
var<storage> JB: pg;
@group(0) @binding(5)
var<storage> DD: mi;
@group(3) @binding(9)
var db: sampler;
var<private> MC_1: u32;

fn main_1() {
    var phi_321_: bool;
    var phi_384_: vec4<f32>;

    let _e38 = AC_1;
    let _e46 = PC_1;
    let _e48 = PB_1;
    let _e50 = ((mat2x2<f32>(vec2<f32>(_e38.x, _e38.y), vec2<f32>(_e38.z, _e38.w)) * _e46) + _e48.xy);
    let _e51 = QC_1;
    let _e52 = IC_1;
    n2_ = ((_e51 * _e52.zw) + _e52.xy);
    if gj {
        let _e57 = TB_1;
        let _e62 = vec2<f32>(_e57.x, _e57.y);
        let _e63 = vec2<f32>(_e57.z, _e57.w);
        switch bitcast<i32>(0u) {
            default: {
                let _e69 = (abs(_e62) + abs(_e63));
                let _e71 = (_e69.x != 0f);
                phi_321_ = _e71;
                if _e71 {
                    phi_321_ = (_e69.y != 0f);
                }
                let _e75 = phi_321_;
                if _e75 {
                    let _e79 = ((mat2x2<f32>(_e62, _e63) * _e50) + _e48.zw);
                    let _e80 = -(_e79);
                    let _e86 = (vec2<f32>(1f, 1f) / _e69).xyxy;
                    phi_384_ = (((vec4<f32>(_e79.x, _e79.y, _e80.x, _e80.y) * _e86) + _e86) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_384_ = _e48.zwzw;
                    break;
                }
            }
        }
        let _e91 = phi_384_;
        W0_ = _e91;
    }
    let _e92 = BC_1;
    Y1_ = unpack4x8unorm(_e92);
    let _e94 = CC_1;
    U3_ = _e94;
    let _e95 = DC_1;
    N1_ = _e95;
    let _e97 = j.Mg;
    let _e99 = j.Ng;
    unnamed.gl_Position = vec4<f32>(((_e50.x * _e97) - 1f), ((_e50.y * _e99) - sign(_e99)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(2) AC: vec4<f32>, @location(0) PC: vec2<f32>, @location(4) PB: vec4<f32>, @location(1) QC: vec2<f32>, @location(9) IC: vec4<f32>, @location(3) TB: vec4<f32>, @location(5) BC: u32, @location(6) CC: u32, @location(7) DC: u32, @location(8) MC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    AC_1 = AC;
    PC_1 = PC;
    PB_1 = PB;
    QC_1 = QC;
    IC_1 = IC;
    TB_1 = TB;
    BC_1 = BC;
    CC_1 = CC;
    DC_1 = DC;
    MC_1 = MC;
    main_1();
    let _e33 = n2_;
    let _e34 = W0_;
    let _e35 = Y1_;
    let _e36 = U3_;
    let _e37 = N1_;
    let _e38 = unnamed.gl_Position;
    return VertexOutput(_e33, _e34, _e35, _e36, _e37, _e38);
}
