struct RB {
    Kd: f32,
    Ue: f32,
    ah: f32,
    bh: f32,
    V5_: u32,
    Ia: u32,
    Lg: u32,
    Ng: u32,
    M8_: vec4<i32>,
    Ri: vec2<f32>,
    Ve: vec2<f32>,
    y2_: u32,
    Vi: f32,
    z6_: u32,
    n3_: f32,
    We: f32,
    Eg: u32,
    K3_: f32,
    L3_: f32,
    Xe: f32,
    Oi: u32,
    Ha: u32,
    T8_: f32,
    U8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct nd {
    z2_: array<vec4<u32>>,
}

struct Cg {
    z2_: array<vec4<f32>>,
}

struct xi {
    z2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(1) member: f32,
    @location(0) member_1: vec2<f32>,
    @location(3) member_2: vec4<f32>,
    @location(4) @interpolate(flat, either) member_3: vec4<f32>,
    @location(5) @interpolate(flat, either) member_4: u32,
    @location(6) @interpolate(flat, either) member_5: u32,
    @location(2) member_6: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(1) override vj: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> HC_1: vec4<f32>;
var<private> D5_: f32;
var<private> AC_1: vec4<f32>;
var<private> TD_1: vec4<f32>;
var<private> o2_: vec2<f32>;
var<private> FD_1: vec4<f32>;
var<private> QB_1: vec4<f32>;
var<private> Y0_: vec4<f32>;
var<private> VB_1: vec4<f32>;
var<private> a2_: vec4<f32>;
var<private> BC_1: u32;
var<private> V3_: u32;
var<private> CC_1: u32;
var<private> Q1_: u32;
var<private> DC_1: u32;
@group(0) @binding(0)
var<uniform> j: RB;
var<private> GD_1: vec4<f32>;
var<private> UD_1: vec4<f32>;
var<private> E5_: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(6)
var WB: texture_2d<u32>;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> JB: nd;
@group(0) @binding(3)
var<storage> KB: Cg;
@group(0) @binding(4)
var<storage> CD: xi;
@group(3) @binding(8)
var ib: sampler;
var<private> NC_1: u32;

fn main_1() {
    var phi_242_: bool;
    var phi_774_: vec2<f32>;
    var phi_776_: vec2<f32>;
    var phi_775_: vec2<f32>;
    var phi_777_: vec2<f32>;
    var phi_606_: bool;
    var phi_778_: vec4<f32>;
    var phi_791_: f32;

    let _e49 = HC_1[2u];
    let _e50 = (_e49 == 0f);
    phi_242_ = _e50;
    if !(_e50) {
        let _e53 = HC_1[3u];
        phi_242_ = (_e53 == 0f);
    }
    let _e56 = phi_242_;
    D5_ = select(1f, 0f, _e56);
    let _e58 = HC_1;
    let _e59 = _e58.xy;
    let _e60 = AC_1;
    let _e65 = vec2<f32>(_e60.x, _e60.y);
    let _e66 = vec2<f32>(_e60.z, _e60.w);
    let _e67 = mat2x2<f32>(_e65, _e66);
    let _e69 = transpose(_naga_inverse_2x2_f32(_e67));
    phi_775_ = _e59;
    if !(_e56) {
        let _e79 = ((0.5f * (abs(_e69[1].x) + abs(_e69[1].y))) / dot(_e66, _e69[1]));
        if (_e79 >= 0.5f) {
            let _e91 = D5_;
            D5_ = (_e91 * (0.5f / _e79));
            phi_774_ = vec2<f32>(0.5f, _e59.y);
        } else {
            phi_774_ = vec2<f32>((_e58.x + (_e79 * _e49)), _e59.y);
        }
        let _e94 = phi_774_;
        let _e103 = ((0.5f * (abs(_e69[0].x) + abs(_e69[0].y))) / dot(_e65, _e69[0]));
        if (_e103 >= 0.5f) {
            let _e117 = D5_;
            D5_ = (_e117 * (0.5f / _e103));
            phi_776_ = vec2<f32>(_e94.x, 0.5f);
        } else {
            let _e106 = HC_1[3u];
            phi_776_ = vec2<f32>(_e94.x, (_e94.y + (_e103 * _e106)));
        }
        let _e120 = phi_776_;
        phi_775_ = _e120;
    }
    let _e122 = phi_775_;
    let _e123 = TD_1;
    let _e132 = FD_1;
    o2_ = ((mat2x2<f32>(vec2<f32>(_e123.x, _e123.y), vec2<f32>(_e123.z, _e123.w)) * _e122) + _e132.xy);
    let _e136 = QB_1;
    let _e138 = ((_e67 * _e122) + _e136.xy);
    phi_777_ = _e138;
    if _e56 {
        let _e140 = (_e69 * _e58.zw);
        phi_777_ = (_e138 + ((_e140 * ((abs(_e140.x) + abs(_e140.y)) / dot(_e140, _e140))) * 0.5f));
    }
    let _e152 = phi_777_;
    if vj {
        let _e153 = VB_1;
        let _e158 = vec2<f32>(_e153.x, _e153.y);
        let _e159 = vec2<f32>(_e153.z, _e153.w);
        switch bitcast<i32>(0u) {
            default: {
                let _e165 = (abs(_e158) + abs(_e159));
                let _e167 = (_e165.x != 0f);
                phi_606_ = _e167;
                if _e167 {
                    phi_606_ = (_e165.y != 0f);
                }
                let _e171 = phi_606_;
                if _e171 {
                    let _e175 = ((mat2x2<f32>(_e158, _e159) * _e152) + _e136.zw);
                    let _e176 = -(_e175);
                    let _e182 = (vec2<f32>(1f, 1f) / _e165).xyxy;
                    phi_778_ = (((vec4<f32>(_e175.x, _e175.y, _e176.x, _e176.y) * _e182) + _e182) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_778_ = _e136.zwzw;
                    break;
                }
            }
        }
        let _e187 = phi_778_;
        Y0_ = _e187;
    }
    let _e188 = BC_1;
    a2_ = unpack4x8unorm(_e188);
    let _e190 = CC_1;
    V3_ = _e190;
    let _e191 = DC_1;
    Q1_ = _e191;
    let _e193 = j.ah;
    let _e195 = j.bh;
    let _e205 = GD_1[2u];
    let _e206 = bitcast<u32>(_e205);
    if (_e206 != 0u) {
        let _e208 = UD_1;
        let _e217 = GD_1;
        let _e219 = ((mat2x2<f32>(vec2<f32>(_e208.x, _e208.y), vec2<f32>(_e208.z, _e208.w)) * _e152) + _e132.zw);
        let _e230 = ((_e217.y + (f32(_e206) * 0.125f)) + (max(_e217.x, 0f) * 0.00024414063f));
        if (_e217.x < 0f) {
            phi_791_ = -(_e230);
        } else {
            phi_791_ = _e230;
        }
        let _e233 = phi_791_;
        E5_ = vec4<f32>(_e219.x, _e219.y, _e233, (-0.75f - round(255f)));
    } else {
        E5_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    unnamed.gl_Position = vec4<f32>(((_e152.x * _e193) - 1f), ((_e152.y * _e195) - sign(_e195)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) HC: vec4<f32>, @location(2) AC: vec4<f32>, @location(9) TD: vec4<f32>, @location(11) FD: vec4<f32>, @location(4) QB: vec4<f32>, @location(3) VB: vec4<f32>, @location(5) BC: u32, @location(6) CC: u32, @location(7) DC: u32, @location(12) GD: vec4<f32>, @location(10) UD: vec4<f32>, @location(8) NC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    HC_1 = HC;
    AC_1 = AC;
    TD_1 = TD;
    FD_1 = FD;
    QB_1 = QB;
    VB_1 = VB;
    BC_1 = BC;
    CC_1 = CC;
    DC_1 = DC;
    GD_1 = GD;
    UD_1 = UD;
    NC_1 = NC;
    main_1();
    let _e39 = D5_;
    let _e40 = o2_;
    let _e41 = Y0_;
    let _e42 = a2_;
    let _e43 = V3_;
    let _e44 = Q1_;
    let _e45 = E5_;
    let _e46 = unnamed.gl_Position;
    return VertexOutput(_e39, _e40, _e41, _e42, _e43, _e44, _e45, _e46);
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
