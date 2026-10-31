struct WB {
    Hd: f32,
    Qe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    I8_: vec4<i32>,
    Ki: vec2<f32>,
    Re: vec2<f32>,
    w2_: u32,
    Oi: f32,
    w6_: u32,
    j3_: f32,
    Se: f32,
    xg: u32,
    I3_: f32,
    J3_: f32,
    Te: f32,
    Hi: u32,
    Ea: u32,
    P8_: f32,
    Q8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct kd {
    x2_: array<vec4<u32>>,
}

struct vg {
    x2_: array<vec4<f32>>,
}

struct qi {
    x2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @location(1) member_1: vec4<f32>,
    @location(3) @interpolate(flat, either) member_2: vec4<f32>,
    @location(4) @interpolate(flat, either) member_3: u32,
    @location(5) @interpolate(flat, either) member_4: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(1) override lj: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> AC_1: vec4<f32>;
var<private> PC_1: vec2<f32>;
var<private> PB_1: vec4<f32>;
var<private> o2_: vec2<f32>;
var<private> QC_1: vec2<f32>;
var<private> IC_1: vec4<f32>;
var<private> V0_: vec4<f32>;
var<private> UB_1: vec4<f32>;
var<private> Y1_: vec4<f32>;
var<private> BC_1: u32;
var<private> V3_: u32;
var<private> CC_1: u32;
var<private> O1_: u32;
var<private> DC_1: u32;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(6)
var VB: texture_2d<u32>;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> JB: kd;
@group(0) @binding(3)
var<storage> KB: vg;
@group(0) @binding(4)
var<storage> CD: qi;
@group(3) @binding(8)
var fb: sampler;
var<private> MC_1: u32;

fn main_1() {
    var phi_316_: bool;
    var phi_379_: vec4<f32>;

    let _e37 = AC_1;
    let _e45 = PC_1;
    let _e47 = PB_1;
    let _e49 = ((mat2x2<f32>(vec2<f32>(_e37.x, _e37.y), vec2<f32>(_e37.z, _e37.w)) * _e45) + _e47.xy);
    let _e50 = QC_1;
    let _e51 = IC_1;
    o2_ = ((_e50 * _e51.zw) + _e51.xy);
    if lj {
        let _e56 = UB_1;
        let _e61 = vec2<f32>(_e56.x, _e56.y);
        let _e62 = vec2<f32>(_e56.z, _e56.w);
        switch bitcast<i32>(0u) {
            default: {
                let _e68 = (abs(_e61) + abs(_e62));
                let _e70 = (_e68.x != 0f);
                phi_316_ = _e70;
                if _e70 {
                    phi_316_ = (_e68.y != 0f);
                }
                let _e74 = phi_316_;
                if _e74 {
                    let _e78 = ((mat2x2<f32>(_e61, _e62) * _e49) + _e47.zw);
                    let _e79 = -(_e78);
                    let _e85 = (vec2<f32>(1f, 1f) / _e68).xyxy;
                    phi_379_ = (((vec4<f32>(_e78.x, _e78.y, _e79.x, _e79.y) * _e85) + _e85) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_379_ = _e47.zwzw;
                    break;
                }
            }
        }
        let _e90 = phi_379_;
        V0_ = _e90;
    }
    let _e91 = BC_1;
    Y1_ = unpack4x8unorm(_e91);
    let _e93 = CC_1;
    V3_ = _e93;
    let _e94 = DC_1;
    O1_ = _e94;
    let _e96 = j.Sg;
    let _e98 = j.Tg;
    unnamed.gl_Position = vec4<f32>(((_e49.x * _e96) - 1f), ((_e49.y * _e98) - sign(_e98)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(2) AC: vec4<f32>, @location(0) PC: vec2<f32>, @location(4) PB: vec4<f32>, @location(1) QC: vec2<f32>, @location(9) IC: vec4<f32>, @location(3) UB: vec4<f32>, @location(5) BC: u32, @location(6) CC: u32, @location(7) DC: u32, @location(8) MC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    AC_1 = AC;
    PC_1 = PC;
    PB_1 = PB;
    QC_1 = QC;
    IC_1 = IC;
    UB_1 = UB;
    BC_1 = BC;
    CC_1 = CC;
    DC_1 = DC;
    MC_1 = MC;
    main_1();
    let _e33 = o2_;
    let _e34 = V0_;
    let _e35 = Y1_;
    let _e36 = V3_;
    let _e37 = O1_;
    let _e38 = unnamed.gl_Position;
    return VertexOutput(_e33, _e34, _e35, _e36, _e37, _e38);
}
