enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct nd {
    z2_: array<vec4<u32>>,
}

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

struct Cg {
    z2_: array<vec4<f32>>,
}

struct xi {
    z2_: array<vec4<u32>>,
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

@id(0) override uj: bool = true;
@id(2) override wj: bool = true;
@id(1) override vj: bool = true;
@id(4) override yj: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(2)
var<storage> JB: nd;
@group(0) @binding(0)
var<uniform> j: RB;
var<private> gl_VertexIndex_1: i32;
var<private> MB_1: vec3<f32>;
var<private> W2_: vec2<f32>;
var<private> j4_: f32;
var<private> S0_: f32;
@group(0) @binding(3)
var<storage> KB: Cg;
var<private> N0_: vec4<f32>;
var<private> O0_: vec3<f32>;
@group(0) @binding(6)
var WB: texture_2d<u32>;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(4)
var<storage> CD: xi;
@group(3) @binding(8)
var ib: sampler;

fn main_1() {
    var phi_844_: u32;
    var phi_845_: f32;
    var phi_846_: f32;
    var phi_855_: f32;
    var phi_467_: bool;
    var phi_857_: f32;

    let _e55 = MB_1;
    let _e58 = (bitcast<u32>(_e55.z) & 65535u);
    let _e59 = (_e58 * 4u);
    let _e60 = (_e59 + 1u);
    let _e64 = JB.z2_[_e60][2u];
    let _e67 = _e55.xy;
    let _e71 = JB.z2_[(_e59 + 3u)];
    let _e73 = bitcast<vec3<f32>>(_e71.xyz);
    let _e79 = j.Ri;
    W2_ = (((_e67 * _e73.x) + _e73.yz) * _e79);
    let _e83 = JB.z2_[_e60];
    let _e85 = (_e83.z & 15u);
    if uj {
        let _e86 = (_e85 == 0u);
        if _e86 {
            phi_844_ = _e83.w;
        } else {
            phi_844_ = _e83.z;
        }
        let _e89 = phi_844_;
        let _e91 = (_e89 >> bitcast<u32>(16i));
        let _e93 = j.z6_;
        if (_e91 == 0u) {
            phi_845_ = 0f;
        } else {
            phi_845_ = unpack2x16float(((_e91 + 1023u) * _e93)).x;
        }
        let _e100 = phi_845_;
        phi_846_ = _e100;
        if _e86 {
            phi_846_ = -(_e100);
        }
        let _e103 = phi_846_;
        j4_ = _e103;
    }
    if wj {
        S0_ = f32(((_e83.z >> bitcast<u32>(4i)) & 15u));
    }
    if vj {
        let _e108 = (_e58 * 8u);
        let _e112 = KB.z2_[(_e108 + 2u)];
        let _e123 = KB.z2_[(_e108 + 3u)];
        if any((_e112 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e138 = ((mat2x2<f32>(vec2<f32>(_e112.x, _e112.y), vec2<f32>(_e112.z, _e112.w)) * _e67) + _e123.xy);
            unnamed.gl_ClipDistance[0i] = (_e138.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e138.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e138.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e138.y);
        } else {
            let _e128 = (_e123.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e128;
            unnamed.gl_ClipDistance[2i] = _e128;
            unnamed.gl_ClipDistance[1i] = _e128;
            unnamed.gl_ClipDistance[0i] = _e128;
        }
    }
    if (_e85 == 1u) {
        N0_ = unpack4x8unorm(_e83.w);
    } else {
        let _e154 = (_e58 * 8u);
        let _e157 = KB.z2_[_e154];
        let _e168 = KB.z2_[(_e154 + 1u)];
        let _e173 = ((mat2x2<f32>(vec2<f32>(_e157.x, _e157.y), vec2<f32>(_e157.z, _e157.w)) * _e67) + _e168.xy);
        let _e184 = ((_e168.w + (f32(_e85) * 0.125f)) + (max(_e168.z, 0f) * 0.00024414063f));
        if (_e168.z < 0f) {
            phi_855_ = -(_e184);
        } else {
            phi_855_ = _e184;
        }
        let _e187 = phi_855_;
        N0_ = vec4<f32>(_e173.x, _e173.y, _e187, (-0.75f - round((bitcast<f32>(_e83.w) * 255f))));
    }
    phi_467_ = yj;
    if yj {
        phi_467_ = ((_e83.z & 2048u) != 0u);
    }
    let _e197 = phi_467_;
    if _e197 {
        let _e198 = (_e58 * 8u);
        let _e202 = KB.z2_[(_e198 + 4u)];
        let _e213 = KB.z2_[(_e198 + 5u)];
        let _e216 = ((mat2x2<f32>(vec2<f32>(_e202.x, _e202.y), vec2<f32>(_e202.z, _e202.w)) * _e67) + _e213.xy);
        phi_857_ = (1f + _e213.z);
        if ((_e83.z & 4096u) != 0u) {
            phi_857_ = (-1f - f32(((_e83.z & 24576u) >> bitcast<u32>(13u))));
        }
        let _e227 = phi_857_;
        O0_ = vec3<f32>(_e216.x, _e216.y, _e227);
    } else {
        O0_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e232 = j.ah;
    let _e234 = j.bh;
    let _e242 = vec4<f32>(((_e55.x * _e232) - 1f), ((_e55.y * _e234) - sign(_e234)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e242.x, _e242.y, ((f32((((_e64 >> bitcast<u32>(16u)) << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e242.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) MB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    MB_1 = MB;
    main_1();
    let _e13 = unnamed.gl_Position;
    let _e14 = unnamed.gl_ClipDistance;
    let _e15 = W2_;
    let _e16 = j4_;
    let _e17 = S0_;
    let _e18 = N0_;
    let _e19 = O0_;
    return VertexOutput(_e13, _e14, _e15, _e16, _e17, _e18, _e19);
}
