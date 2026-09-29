enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct lg {
    d2_: array<vec4<u32>>,
}

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
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(1) member: vec2<f32>,
    @location(4) @interpolate(flat, either) member_1: f32,
    @location(6) @interpolate(flat, either) member_2: f32,
    @location(0) member_3: vec4<f32>,
    @location(9) member_4: vec3<f32>,
}

@id(0) override jh: bool = true;
@id(2) override lh: bool = true;
@id(1) override kh: bool = true;
@id(8) override rh: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(2)
var<storage> PB: lg;
@group(0) @binding(0)
var<uniform> m: BC;
var<private> gl_VertexIndex_1: i32;
var<private> KB_1: vec3<f32>;
var<private> D2_: vec2<f32>;
@group(0) @binding(3)
var<storage> AD: Re;
var<private> J3_: f32;
var<private> f2_: f32;
@group(0) @binding(4)
var<storage> QB: Se;
var<private> f1_: vec4<f32>;
var<private> A2_: vec3<f32>;
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> ED: mg;
@group(3) @binding(9)
var Z9_: sampler;

fn main_1() {
    var phi_715_: u32;
    var phi_716_: f32;
    var phi_717_: f32;
    var phi_718_: vec4<f32>;
    var phi_432_: bool;

    let _e47 = KB_1;
    let _e50 = (bitcast<u32>(_e47.z) & 65535u);
    let _e55 = PB.d2_[((_e50 * 4u) + 2u)];
    let _e57 = _e47.xy;
    let _e59 = bitcast<vec3<f32>>(_e55.yzw);
    let _e65 = m.Lg;
    D2_ = (((_e57 * _e59.x) + _e59.yz) * _e65);
    let _e69 = AD.d2_[_e50];
    let _e71 = (_e69.x & 15u);
    if jh {
        let _e72 = (_e71 == 0u);
        if _e72 {
            phi_715_ = _e69.y;
        } else {
            phi_715_ = _e69.x;
        }
        let _e75 = phi_715_;
        let _e77 = (_e75 >> bitcast<u32>(16i));
        let _e79 = m.d6_;
        if (_e77 == 0u) {
            phi_716_ = 0f;
        } else {
            phi_716_ = unpack2x16float(((_e77 + 1023u) * _e79)).x;
        }
        let _e86 = phi_716_;
        phi_717_ = _e86;
        if _e72 {
            phi_717_ = -(_e86);
        }
        let _e89 = phi_717_;
        J3_ = _e89;
    }
    if lh {
        f2_ = f32(((_e69.x >> bitcast<u32>(4i)) & 15u));
    }
    if kh {
        let _e94 = (_e50 * 8u);
        let _e98 = QB.d2_[(_e94 + 2u)];
        let _e109 = QB.d2_[(_e94 + 3u)];
        if any((_e98 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e124 = ((mat2x2<f32>(vec2<f32>(_e98.x, _e98.y), vec2<f32>(_e98.z, _e98.w)) * _e57) + _e109.xy);
            unnamed.gl_ClipDistance[0i] = (_e124.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e124.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e124.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e124.y);
        } else {
            let _e114 = (_e109.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e114;
            unnamed.gl_ClipDistance[2i] = _e114;
            unnamed.gl_ClipDistance[1i] = _e114;
            unnamed.gl_ClipDistance[0i] = _e114;
        }
    }
    if (_e71 == 1u) {
        let _e179 = unpack4x8unorm(_e69.y);
        if lh {
            phi_718_ = _e179;
        } else {
            let _e182 = (_e179.xyz * _e179.w);
            let _e188 = vec4<f32>(_e182.x, _e179.y, _e179.z, _e179.w);
            let _e194 = vec4<f32>(_e188.x, _e182.y, _e188.z, _e188.w);
            phi_718_ = vec4<f32>(_e194.x, _e194.y, _e182.z, _e194.w);
        }
        let _e202 = phi_718_;
        f1_ = _e202;
    } else {
        let _e140 = (_e50 * 8u);
        let _e143 = QB.d2_[_e140];
        let _e154 = QB.d2_[(_e140 + 1u)];
        let _e157 = ((mat2x2<f32>(vec2<f32>(_e143.x, _e143.y), vec2<f32>(_e143.z, _e143.w)) * _e57) + _e154.xy);
        f1_[3u] = -(bitcast<f32>(_e69.y));
        if (_e154.z > 0.9f) {
            f1_[2u] = 2f;
        } else {
            f1_[2u] = _e154.w;
        }
        if (_e71 == 2u) {
            f1_[1u] = 0f;
            f1_[0u] = _e157.x;
        } else {
            let _e169 = f1_[2u];
            f1_[2u] = -(_e169);
            f1_[0u] = _e157.x;
            f1_[1u] = _e157.y;
        }
    }
    phi_432_ = rh;
    if rh {
        phi_432_ = ((_e69.x & 2048u) != 0u);
    }
    let _e206 = phi_432_;
    if _e206 {
        let _e207 = (_e50 * 8u);
        let _e211 = QB.d2_[(_e207 + 4u)];
        let _e222 = QB.d2_[(_e207 + 5u)];
        let _e225 = ((mat2x2<f32>(vec2<f32>(_e211.x, _e211.y), vec2<f32>(_e211.z, _e211.w)) * _e57) + _e222.xy);
        A2_ = vec3<f32>(_e225.x, _e225.y, (1f + _e222.z));
    } else {
        A2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e232 = m.of_;
    let _e234 = m.pf;
    let _e242 = vec4<f32>(((_e47.x * _e232) - 1f), ((_e47.y * _e234) - sign(_e234)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e242.x, _e242.y, (1f - (f32(_e55.x) * 0.000061035156f)), _e242.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) KB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    KB_1 = KB;
    main_1();
    let _e13 = unnamed.gl_Position;
    let _e14 = unnamed.gl_ClipDistance;
    let _e15 = D2_;
    let _e16 = J3_;
    let _e17 = f2_;
    let _e18 = f1_;
    let _e19 = A2_;
    return VertexOutput(_e13, _e14, _e15, _e16, _e17, _e18, _e19);
}
