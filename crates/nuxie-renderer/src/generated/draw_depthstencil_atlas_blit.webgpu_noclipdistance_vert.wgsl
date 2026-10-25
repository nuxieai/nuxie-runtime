struct pi {
    v2_: array<vec4<u32>>,
}

struct WB {
    Fd: f32,
    Pe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    J8_: vec4<i32>,
    Ki: vec2<f32>,
    Qe: vec2<f32>,
    r2_: u32,
    Oi: f32,
    z6_: u32,
    i3_: f32,
    Re: f32,
    xg: u32,
    H3_: f32,
    I3_: f32,
    Se: f32,
    Hi: u32,
    Ea: u32,
    Q8_: f32,
    R8_: f32,
}

struct ug {
    v2_: array<vec2<u32>>,
}

struct vg {
    v2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct qi {
    v2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(1) member: vec2<f32>,
    @location(4) @interpolate(flat, either) member_1: f32,
    @location(6) @interpolate(flat, either) member_2: f32,
    @location(0) member_3: vec4<f32>,
    @location(9) member_4: vec3<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override jj: bool = true;
@id(2) override lj: bool = true;
@id(4) override nj: bool = true;

@group(0) @binding(2)
var<storage> KB: pi;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> U2_: vec2<f32>;
@group(0) @binding(3)
var<storage> XC: ug;
var<private> i4_: f32;
var<private> S0_: f32;
@group(0) @binding(4)
var<storage> JB: vg;
var<private> N0_: vec4<f32>;
var<private> O0_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var VB: texture_2d<u32>;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(0) @binding(5)
var<storage> DD: qi;
@group(3) @binding(9)
var eb: sampler;

fn main_1() {
    var phi_753_: u32;
    var phi_754_: f32;
    var phi_755_: f32;
    var phi_762_: f32;
    var phi_416_: bool;
    var phi_764_: f32;

    let _e50 = LB_1;
    let _e53 = (bitcast<u32>(_e50.z) & 65535u);
    let _e58 = KB.v2_[((_e53 * 4u) + 2u)];
    let _e60 = _e50.xy;
    let _e62 = bitcast<vec3<f32>>(_e58.yzw);
    let _e68 = j.Ki;
    U2_ = (((_e60 * _e62.x) + _e62.yz) * _e68);
    let _e72 = XC.v2_[_e53];
    let _e74 = (_e72.x & 15u);
    if jj {
        let _e75 = (_e74 == 0u);
        if _e75 {
            phi_753_ = _e72.y;
        } else {
            phi_753_ = _e72.x;
        }
        let _e78 = phi_753_;
        let _e80 = (_e78 >> bitcast<u32>(16i));
        let _e82 = j.z6_;
        if (_e80 == 0u) {
            phi_754_ = 0f;
        } else {
            phi_754_ = unpack2x16float(((_e80 + 1023u) * _e82)).x;
        }
        let _e89 = phi_754_;
        phi_755_ = _e89;
        if _e75 {
            phi_755_ = -(_e89);
        }
        let _e92 = phi_755_;
        i4_ = _e92;
    }
    if lj {
        S0_ = f32(((_e72.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e74 == 1u) {
        N0_ = unpack4x8unorm(_e72.y);
    } else {
        let _e98 = (_e53 * 8u);
        let _e101 = JB.v2_[_e98];
        let _e112 = JB.v2_[(_e98 + 1u)];
        let _e117 = ((mat2x2<f32>(vec2<f32>(_e101.x, _e101.y), vec2<f32>(_e101.z, _e101.w)) * _e60) + _e112.xy);
        let _e128 = ((_e112.w + (f32(_e74) * 0.125f)) + (max(_e112.z, 0f) * 0.00024414063f));
        if (_e112.z < 0f) {
            phi_762_ = -(_e128);
        } else {
            phi_762_ = _e128;
        }
        let _e131 = phi_762_;
        N0_ = vec4<f32>(_e117.x, _e117.y, _e131, (-0.75f - round((bitcast<f32>(_e72.y) * 255f))));
    }
    phi_416_ = nj;
    if nj {
        phi_416_ = ((_e72.x & 2048u) != 0u);
    }
    let _e141 = phi_416_;
    if _e141 {
        let _e142 = (_e53 * 8u);
        let _e146 = JB.v2_[(_e142 + 4u)];
        let _e157 = JB.v2_[(_e142 + 5u)];
        let _e160 = ((mat2x2<f32>(vec2<f32>(_e146.x, _e146.y), vec2<f32>(_e146.z, _e146.w)) * _e60) + _e157.xy);
        phi_764_ = (1f + _e157.z);
        if ((_e72.x & 4096u) != 0u) {
            phi_764_ = (-1f - f32(((_e72.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e171 = phi_764_;
        O0_ = vec3<f32>(_e160.x, _e160.y, _e171);
    } else {
        O0_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e176 = j.Sg;
    let _e178 = j.Tg;
    let _e186 = vec4<f32>(((_e50.x * _e176) - 1f), ((_e50.y * _e178) - sign(_e178)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e186.x, _e186.y, ((f32(((_e58.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e186.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e12 = U2_;
    let _e13 = i4_;
    let _e14 = S0_;
    let _e15 = N0_;
    let _e16 = O0_;
    let _e17 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
