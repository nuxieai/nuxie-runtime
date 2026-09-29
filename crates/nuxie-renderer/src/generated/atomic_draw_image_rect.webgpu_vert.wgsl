struct BC {
    jc: f32,
    sd: f32,
    of_: f32,
    pf: f32,
    p6_: u32,
    Pg: u32,
    Ze: u32,
    af: u32,
    U7_: vec4<i32>,
    Lg: vec2<f32>,
    td: vec2<f32>,
    c2_: u32,
    Qg: f32,
    d6_: u32,
    R2_: f32,
    ud: f32,
    Ue: u32,
    A3_: f32,
    B3_: f32,
    vd: f32,
    Ig: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct lg {
    d2_: array<vec4<u32>>,
}

struct Re {
    d2_: array<vec2<u32>>,
}

struct Se {
    d2_: array<vec4<f32>>,
}

struct mg {
    d2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(1) member: f32,
    @location(0) member_1: vec2<f32>,
    @location(2) member_2: vec4<f32>,
    @location(3) @interpolate(flat, either) member_3: f32,
    @location(4) @interpolate(flat, either) member_4: u32,
    @location(5) @interpolate(flat, either) member_5: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(1) override kh: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> GC_1: vec4<f32>;
var<private> V4_: f32;
var<private> WB_1: vec4<f32>;
var<private> Y1_: vec2<f32>;
var<private> NB_1: vec4<f32>;
var<private> M0_: vec4<f32>;
var<private> SB_1: vec4<f32>;
var<private> I1_: f32;
var<private> XB_1: f32;
var<private> w3_: u32;
var<private> YB_1: u32;
var<private> B1_: u32;
var<private> ZB_1: u32;
@group(0) @binding(0)
var<uniform> m: BC;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> PB: lg;
@group(0) @binding(3)
var<storage> AD: Re;
@group(0) @binding(4)
var<storage> QB: Se;
@group(0) @binding(5)
var<storage> ED: mg;
@group(3) @binding(9)
var Z9_: sampler;
var<private> MC_1: u32;

fn main_1() {
    var phi_175_: bool;
    var phi_566_: vec2<f32>;
    var phi_568_: vec2<f32>;
    var phi_567_: vec2<f32>;
    var phi_569_: vec2<f32>;
    var phi_481_: bool;
    var phi_570_: vec4<f32>;

    let _e40 = GC_1[2u];
    let _e41 = (_e40 == 0f);
    phi_175_ = _e41;
    if !(_e41) {
        let _e44 = GC_1[3u];
        phi_175_ = (_e44 == 0f);
    }
    let _e47 = phi_175_;
    V4_ = select(1f, 0f, _e47);
    let _e49 = GC_1;
    let _e50 = _e49.xy;
    let _e51 = WB_1;
    let _e56 = vec2<f32>(_e51.x, _e51.y);
    let _e57 = vec2<f32>(_e51.z, _e51.w);
    let _e58 = mat2x2<f32>(_e56, _e57);
    let _e60 = transpose(_naga_inverse_2x2_f32(_e58));
    phi_567_ = _e50;
    if !(_e47) {
        let _e70 = ((0.5f * (abs(_e60[1].x) + abs(_e60[1].y))) / dot(_e57, _e60[1]));
        if (_e70 >= 0.5f) {
            let _e82 = V4_;
            V4_ = (_e82 * (0.5f / _e70));
            phi_566_ = vec2<f32>(0.5f, _e50.y);
        } else {
            phi_566_ = vec2<f32>((_e49.x + (_e70 * _e40)), _e50.y);
        }
        let _e85 = phi_566_;
        let _e94 = ((0.5f * (abs(_e60[0].x) + abs(_e60[0].y))) / dot(_e56, _e60[0]));
        if (_e94 >= 0.5f) {
            let _e108 = V4_;
            V4_ = (_e108 * (0.5f / _e94));
            phi_568_ = vec2<f32>(_e85.x, 0.5f);
        } else {
            let _e97 = GC_1[3u];
            phi_568_ = vec2<f32>(_e85.x, (_e85.y + (_e94 * _e97)));
        }
        let _e111 = phi_568_;
        phi_567_ = _e111;
    }
    let _e113 = phi_567_;
    Y1_ = _e113;
    let _e115 = NB_1;
    let _e117 = ((_e58 * _e113) + _e115.xy);
    phi_569_ = _e117;
    if _e47 {
        let _e119 = (_e60 * _e49.zw);
        phi_569_ = (_e117 + ((_e119 * ((abs(_e119.x) + abs(_e119.y)) / dot(_e119, _e119))) * 0.5f));
    }
    let _e131 = phi_569_;
    if kh {
        let _e132 = SB_1;
        let _e137 = vec2<f32>(_e132.x, _e132.y);
        let _e138 = vec2<f32>(_e132.z, _e132.w);
        switch bitcast<i32>(0u) {
            default: {
                let _e144 = (abs(_e137) + abs(_e138));
                let _e146 = (_e144.x != 0f);
                phi_481_ = _e146;
                if _e146 {
                    phi_481_ = (_e144.y != 0f);
                }
                let _e150 = phi_481_;
                if _e150 {
                    let _e154 = ((mat2x2<f32>(_e137, _e138) * _e131) + _e115.zw);
                    let _e155 = -(_e154);
                    let _e161 = (vec2<f32>(1f, 1f) / _e144).xyxy;
                    phi_570_ = (((vec4<f32>(_e154.x, _e154.y, _e155.x, _e155.y) * _e161) + _e161) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_570_ = _e115.zwzw;
                    break;
                }
            }
        }
        let _e166 = phi_570_;
        M0_ = _e166;
    }
    let _e167 = XB_1;
    I1_ = _e167;
    let _e168 = YB_1;
    w3_ = _e168;
    let _e169 = ZB_1;
    B1_ = _e169;
    let _e171 = m.of_;
    let _e173 = m.pf;
    unnamed.gl_Position = vec4<f32>(((_e131.x * _e171) - 1f), ((_e131.y * _e173) - sign(_e173)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) GC: vec4<f32>, @location(2) WB: vec4<f32>, @location(4) NB: vec4<f32>, @location(3) SB: vec4<f32>, @location(5) XB: f32, @location(6) YB: u32, @location(7) ZB: u32, @location(8) MC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    GC_1 = GC;
    WB_1 = WB;
    NB_1 = NB;
    SB_1 = SB;
    XB_1 = XB;
    YB_1 = YB;
    ZB_1 = ZB;
    MC_1 = MC;
    main_1();
    let _e30 = V4_;
    let _e31 = Y1_;
    let _e32 = M0_;
    let _e33 = I1_;
    let _e34 = w3_;
    let _e35 = B1_;
    let _e36 = unnamed.gl_Position;
    return VertexOutput(_e30, _e31, _e32, _e33, _e34, _e35, _e36);
}

fn _naga_inverse_2x2_f32(m: mat2x2<f32>) -> mat2x2<f32> {
    var adj: mat2x2<f32>;
    adj[0][0] = m[1][1];
    adj[0][1] = -m[0][1];
    adj[1][0] = -m[1][0];
    adj[1][1] = m[0][0];

    let det: f32 = m[0][0] * m[1][1] - m[1][0] * m[0][1];
    return adj * (1 / det);
}
