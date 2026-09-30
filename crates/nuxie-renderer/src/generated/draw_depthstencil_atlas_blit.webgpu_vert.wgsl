enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct dh {
    j2_: array<vec4<u32>>,
}

struct UB {
    Qc: f32,
    Td: f32,
    Yf: f32,
    Zf: f32,
    z6_: u32,
    X9_: u32,
    Kf: u32,
    Lf: u32,
    i8_: vec4<i32>,
    Ch: vec2<f32>,
    Ud: vec2<f32>,
    i2_: u32,
    Gh: f32,
    T4_: u32,
    c3_: f32,
    Vd: f32,
    Ef: u32,
    M3_: f32,
    N3_: f32,
    Wd: f32,
    zh: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

struct Bf {
    j2_: array<vec2<u32>>,
}

struct Cf {
    j2_: array<vec4<f32>>,
}

struct eh {
    j2_: array<vec4<u32>>,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(1) member: vec2<f32>,
    @location(4) @interpolate(flat, either) member_1: f32,
    @location(6) @interpolate(flat, either) member_2: f32,
    @location(0) member_3: vec4<f32>,
    @location(9) member_4: vec3<f32>,
}

@id(0) override bi: bool = true;
@id(2) override di: bool = true;
@id(1) override ci: bool = true;
@id(8) override ji: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(2)
var<storage> LB: dh;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> gl_VertexIndex_1: i32;
var<private> MB_1: vec3<f32>;
var<private> K2_: vec2<f32>;
@group(0) @binding(3)
var<storage> XC: Bf;
var<private> Z3_: f32;
var<private> Q0_: f32;
@group(0) @binding(4)
var<storage> JB: Cf;
var<private> a1_: vec4<f32>;
var<private> F1_: vec3<f32>;
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> AD: eh;
@group(3) @binding(9)
var wa: sampler;

fn main_1() {
    var phi_798_: u32;
    var phi_799_: f32;
    var phi_800_: f32;
    var phi_809_: vec4<f32>;
    var phi_810_: vec4<f32>;
    var phi_455_: bool;

    let _e50 = MB_1;
    let _e53 = (bitcast<u32>(_e50.z) & 65535u);
    let _e58 = LB.j2_[((_e53 * 4u) + 2u)];
    let _e60 = _e50.xy;
    let _e62 = bitcast<vec3<f32>>(_e58.yzw);
    let _e68 = j.Ch;
    K2_ = (((_e60 * _e62.x) + _e62.yz) * _e68);
    let _e72 = XC.j2_[_e53];
    let _e74 = (_e72.x & 15u);
    if bi {
        let _e75 = (_e74 == 0u);
        if _e75 {
            phi_798_ = _e72.y;
        } else {
            phi_798_ = _e72.x;
        }
        let _e78 = phi_798_;
        let _e80 = (_e78 >> bitcast<u32>(16i));
        let _e82 = j.T4_;
        if (_e80 == 0u) {
            phi_799_ = 0f;
        } else {
            phi_799_ = unpack2x16float(((_e80 + 1023u) * _e82)).x;
        }
        let _e89 = phi_799_;
        phi_800_ = _e89;
        if _e75 {
            phi_800_ = -(_e89);
        }
        let _e92 = phi_800_;
        Z3_ = _e92;
    }
    if di {
        Q0_ = f32(((_e72.x >> bitcast<u32>(4i)) & 15u));
    }
    if ci {
        let _e97 = (_e53 * 8u);
        let _e101 = JB.j2_[(_e97 + 2u)];
        let _e112 = JB.j2_[(_e97 + 3u)];
        if any((_e101 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e127 = ((mat2x2<f32>(vec2<f32>(_e101.x, _e101.y), vec2<f32>(_e101.z, _e101.w)) * _e60) + _e112.xy);
            unnamed.gl_ClipDistance[0i] = (_e127.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e127.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e127.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e127.y);
        } else {
            let _e117 = (_e112.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e117;
            unnamed.gl_ClipDistance[2i] = _e117;
            unnamed.gl_ClipDistance[1i] = _e117;
            unnamed.gl_ClipDistance[0i] = _e117;
        }
    }
    if (_e74 == 1u) {
        a1_ = unpack4x8unorm(_e72.y);
    } else {
        let _e143 = (_e53 * 8u);
        let _e146 = JB.j2_[_e143];
        let _e157 = JB.j2_[(_e143 + 1u)];
        let _e166 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e72.y));
        let _e168 = ((mat2x2<f32>(vec2<f32>(_e146.x, _e146.y), vec2<f32>(_e146.z, _e146.w)) * _e60) + _e157.xy);
        if (_e157.z > 0.9f) {
            phi_809_ = vec4<f32>(_e166.x, _e166.y, 2f, _e166.w);
        } else {
            phi_809_ = vec4<f32>(_e166.x, _e166.y, _e157.w, _e166.w);
        }
        let _e183 = phi_809_;
        if (f32(_e74) == 2f) {
            let _e209 = vec4<f32>(_e168.x, _e183.y, _e183.z, _e183.w);
            phi_810_ = vec4<f32>(_e209.x, 0f, _e209.z, _e209.w);
        } else {
            let _e191 = vec4<f32>(_e183.x, _e183.y, -(_e183.z), _e183.w);
            let _e197 = vec4<f32>(_e168.x, _e191.y, _e191.z, _e191.w);
            phi_810_ = vec4<f32>(_e197.x, _e168.y, _e197.z, _e197.w);
        }
        let _e216 = phi_810_;
        a1_ = _e216;
        let _e218 = a1_[3u];
        a1_[3u] = -(_e218);
    }
    phi_455_ = ji;
    if ji {
        phi_455_ = ((_e72.x & 2048u) != 0u);
    }
    let _e225 = phi_455_;
    if _e225 {
        let _e226 = (_e53 * 8u);
        let _e230 = JB.j2_[(_e226 + 4u)];
        let _e241 = JB.j2_[(_e226 + 5u)];
        let _e244 = ((mat2x2<f32>(vec2<f32>(_e230.x, _e230.y), vec2<f32>(_e230.z, _e230.w)) * _e60) + _e241.xy);
        F1_ = vec3<f32>(_e244.x, _e244.y, (1f + _e241.z));
    } else {
        F1_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e251 = j.Yf;
    let _e253 = j.Zf;
    let _e261 = vec4<f32>(((_e50.x * _e251) - 1f), ((_e50.y * _e253) - sign(_e253)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e261.x, _e261.y, ((f32(((_e58.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e261.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) MB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    MB_1 = MB;
    main_1();
    let _e13 = unnamed.gl_Position;
    let _e14 = unnamed.gl_ClipDistance;
    let _e15 = K2_;
    let _e16 = Z3_;
    let _e17 = Q0_;
    let _e18 = a1_;
    let _e19 = F1_;
    return VertexOutput(_e13, _e14, _e15, _e16, _e17, _e18, _e19);
}
