struct ki {
    v2_: array<vec4<u32>>,
}

struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    H8_: vec4<i32>,
    Fi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ji: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Ci: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

struct og {
    v2_: array<vec2<u32>>,
}

struct pg {
    v2_: array<vec4<f32>>,
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

struct VertexOutput {
    @location(1) member: vec2<f32>,
    @location(4) @interpolate(flat, either) member_1: f32,
    @location(6) @interpolate(flat, either) member_2: f32,
    @location(0) member_3: vec4<f32>,
    @location(9) member_4: vec3<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override ej: bool = true;
@id(2) override gj: bool = true;
@id(8) override mj: bool = true;

@group(0) @binding(2)
var<storage> KB: ki;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> U2_: vec2<f32>;
@group(0) @binding(3)
var<storage> XC: og;
var<private> h4_: f32;
var<private> Q0_: f32;
@group(0) @binding(4)
var<storage> JB: pg;
var<private> P0_: vec4<f32>;
var<private> V0_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var VB: texture_2d<u32>;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(0) @binding(5)
var<storage> DD: li;
@group(3) @binding(9)
var db: sampler;

fn main_1() {
    var phi_752_: u32;
    var phi_753_: f32;
    var phi_754_: f32;
    var phi_761_: f32;
    var phi_416_: bool;
    var phi_763_: f32;

    let _e50 = LB_1;
    let _e53 = (bitcast<u32>(_e50.z) & 65535u);
    let _e58 = KB.v2_[((_e53 * 4u) + 2u)];
    let _e60 = _e50.xy;
    let _e62 = bitcast<vec3<f32>>(_e58.yzw);
    let _e68 = j.Fi;
    U2_ = (((_e60 * _e62.x) + _e62.yz) * _e68);
    let _e72 = XC.v2_[_e53];
    let _e74 = (_e72.x & 15u);
    if ej {
        let _e75 = (_e74 == 0u);
        if _e75 {
            phi_752_ = _e72.y;
        } else {
            phi_752_ = _e72.x;
        }
        let _e78 = phi_752_;
        let _e80 = (_e78 >> bitcast<u32>(16i));
        let _e82 = j.z6_;
        if (_e80 == 0u) {
            phi_753_ = 0f;
        } else {
            phi_753_ = unpack2x16float(((_e80 + 1023u) * _e82)).x;
        }
        let _e89 = phi_753_;
        phi_754_ = _e89;
        if _e75 {
            phi_754_ = -(_e89);
        }
        let _e92 = phi_754_;
        h4_ = _e92;
    }
    if gj {
        Q0_ = f32(((_e72.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e74 == 1u) {
        P0_ = unpack4x8unorm(_e72.y);
    } else {
        let _e98 = (_e53 * 8u);
        let _e101 = JB.v2_[_e98];
        let _e112 = JB.v2_[(_e98 + 1u)];
        let _e117 = ((mat2x2<f32>(vec2<f32>(_e101.x, _e101.y), vec2<f32>(_e101.z, _e101.w)) * _e60) + _e112.xy);
        let _e128 = ((_e112.w + (f32(_e74) * 0.125f)) + (max(_e112.z, 0f) * 0.00024414063f));
        if (_e112.z < 0f) {
            phi_761_ = -(_e128);
        } else {
            phi_761_ = _e128;
        }
        let _e131 = phi_761_;
        P0_ = vec4<f32>(_e117.x, _e117.y, _e131, (-0.75f - round((bitcast<f32>(_e72.y) * 255f))));
    }
    phi_416_ = mj;
    if mj {
        phi_416_ = ((_e72.x & 2048u) != 0u);
    }
    let _e141 = phi_416_;
    if _e141 {
        let _e142 = (_e53 * 8u);
        let _e146 = JB.v2_[(_e142 + 4u)];
        let _e157 = JB.v2_[(_e142 + 5u)];
        let _e160 = ((mat2x2<f32>(vec2<f32>(_e146.x, _e146.y), vec2<f32>(_e146.z, _e146.w)) * _e60) + _e157.xy);
        phi_763_ = (1f + _e157.z);
        if ((_e72.x & 4096u) != 0u) {
            phi_763_ = (-1f - f32(((_e72.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e171 = phi_763_;
        V0_ = vec3<f32>(_e160.x, _e160.y, _e171);
    } else {
        V0_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e176 = j.Mg;
    let _e178 = j.Ng;
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
    let _e13 = h4_;
    let _e14 = Q0_;
    let _e15 = P0_;
    let _e16 = V0_;
    let _e17 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
