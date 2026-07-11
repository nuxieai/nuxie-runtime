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

struct Rf {
    X1_: array<vec4<u32>>,
}

struct Be {
    X1_: array<vec2<u32>>,
}

struct Ce {
    X1_: array<vec4<f32>>,
}

struct Sf {
    X1_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(1) member: f32,
    @location(0) member_1: vec2<f32>,
    @location(2) member_2: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(1) override Ng: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> ZB_1: vec4<f32>;
var<private> S4_: f32;
@group(0) @binding(2) 
var<uniform> A0_: LC;
var<private> U0_: vec2<f32>;
var<private> N0_: vec4<f32>;
@group(0) @binding(0) 
var<uniform> k: NB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(8) 
var DC: texture_2d<u32>;
@group(0) @binding(10) 
var QC: texture_2d<f32>;
@group(0) @binding(3) 
var<storage> MB: Rf;
@group(0) @binding(4) 
var<storage> TC: Be;
@group(0) @binding(5) 
var<storage> PB: Ce;
@group(0) @binding(6) 
var<storage> XC: Sf;
@group(3) @binding(10) 
var T9_: sampler;

fn main_1() {
    var phi_165_: bool;
    var phi_534_: vec2<f32>;
    var phi_536_: vec2<f32>;
    var phi_535_: vec2<f32>;
    var phi_537_: vec2<f32>;
    var phi_459_: bool;
    var phi_538_: vec4<f32>;

    let _e33 = ZB_1[2u];
    let _e34 = (_e33 == 0f);
    phi_165_ = _e34;
    if !(_e34) {
        let _e37 = ZB_1[3u];
        phi_165_ = (_e37 == 0f);
    }
    let _e40 = phi_165_;
    S4_ = select(1f, 0f, _e40);
    let _e42 = ZB_1;
    let _e43 = _e42.xy;
    let _e45 = A0_.r9_;
    let _e50 = vec2<f32>(_e45.x, _e45.y);
    let _e51 = vec2<f32>(_e45.z, _e45.w);
    let _e52 = mat2x2<f32>(_e50, _e51);
    let _e54 = transpose(_naga_inverse_2x2_f32(_e52));
    phi_535_ = _e43;
    if !(_e40) {
        let _e64 = ((0.5f * (abs(_e54[1].x) + abs(_e54[1].y))) / dot(_e51, _e54[1]));
        if (_e64 >= 0.5f) {
            let _e76 = S4_;
            S4_ = (_e76 * (0.5f / _e64));
            phi_534_ = vec2<f32>(0.5f, _e43.y);
        } else {
            phi_534_ = vec2<f32>((_e42.x + (_e64 * _e33)), _e43.y);
        }
        let _e79 = phi_534_;
        let _e88 = ((0.5f * (abs(_e54[0].x) + abs(_e54[0].y))) / dot(_e50, _e54[0]));
        if (_e88 >= 0.5f) {
            let _e102 = S4_;
            S4_ = (_e102 * (0.5f / _e88));
            phi_536_ = vec2<f32>(_e79.x, 0.5f);
        } else {
            let _e91 = ZB_1[3u];
            phi_536_ = vec2<f32>(_e79.x, (_e79.y + (_e88 * _e91)));
        }
        let _e105 = phi_536_;
        phi_535_ = _e105;
    }
    let _e107 = phi_535_;
    U0_ = _e107;
    let _e110 = A0_.c2_;
    let _e111 = ((_e52 * _e107) + _e110);
    phi_537_ = _e111;
    if _e40 {
        let _e113 = (_e54 * _e42.zw);
        phi_537_ = (_e111 + ((_e113 * ((abs(_e113.x) + abs(_e113.y)) / dot(_e113, _e113))) * 0.5f));
    }
    let _e125 = phi_537_;
    if Ng {
        let _e127 = A0_.k2_;
        let _e132 = vec2<f32>(_e127.x, _e127.y);
        let _e133 = vec2<f32>(_e127.z, _e127.w);
        let _e136 = A0_.D2_;
        switch bitcast<i32>(0u) {
            default: {
                let _e140 = (abs(_e132) + abs(_e133));
                let _e142 = (_e140.x != 0f);
                phi_459_ = _e142;
                if _e142 {
                    phi_459_ = (_e140.y != 0f);
                }
                let _e146 = phi_459_;
                if _e146 {
                    let _e150 = ((mat2x2<f32>(_e132, _e133) * _e125) + _e136);
                    let _e151 = -(_e150);
                    let _e157 = (vec2<f32>(1f, 1f) / _e140).xyxy;
                    phi_538_ = (((vec4<f32>(_e150.x, _e150.y, _e151.x, _e151.y) * _e157) + _e157) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_538_ = _e136.xyxy;
                    break;
                }
            }
        }
        let _e162 = phi_538_;
        N0_ = _e162;
    }
    let _e164 = k.Xe;
    let _e166 = k.Ye;
    unnamed.gl_Position = vec4<f32>(((_e125.x * _e164) - 1f), ((_e125.y * _e166) - sign(_e166)), 0f, 1f);
    return;
}

@vertex 
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) ZB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    ZB_1 = ZB;
    main_1();
    let _e13 = S4_;
    let _e14 = U0_;
    let _e15 = N0_;
    let _e16 = unnamed.gl_Position;
    return VertexOutput(_e13, _e14, _e15, _e16);
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
