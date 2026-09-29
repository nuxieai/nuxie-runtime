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

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct mg {
    d2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(1) member: vec2<f32>,
    @location(4) @interpolate(flat, either) member_1: f32,
    @location(6) @interpolate(flat, either) member_2: f32,
    @location(0) member_3: vec4<f32>,
    @location(9) member_4: vec3<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override jh: bool = true;
@id(2) override lh: bool = true;
@id(8) override rh: bool = true;

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
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> ED: mg;
@group(3) @binding(9)
var Z9_: sampler;

fn main_1() {
    var phi_625_: u32;
    var phi_626_: f32;
    var phi_627_: f32;
    var phi_628_: vec4<f32>;
    var phi_381_: bool;

    let _e43 = KB_1;
    let _e46 = (bitcast<u32>(_e43.z) & 65535u);
    let _e51 = PB.d2_[((_e46 * 4u) + 2u)];
    let _e53 = _e43.xy;
    let _e55 = bitcast<vec3<f32>>(_e51.yzw);
    let _e61 = m.Lg;
    D2_ = (((_e53 * _e55.x) + _e55.yz) * _e61);
    let _e65 = AD.d2_[_e46];
    let _e67 = (_e65.x & 15u);
    if jh {
        let _e68 = (_e67 == 0u);
        if _e68 {
            phi_625_ = _e65.y;
        } else {
            phi_625_ = _e65.x;
        }
        let _e71 = phi_625_;
        let _e73 = (_e71 >> bitcast<u32>(16i));
        let _e75 = m.d6_;
        if (_e73 == 0u) {
            phi_626_ = 0f;
        } else {
            phi_626_ = unpack2x16float(((_e73 + 1023u) * _e75)).x;
        }
        let _e82 = phi_626_;
        phi_627_ = _e82;
        if _e68 {
            phi_627_ = -(_e82);
        }
        let _e85 = phi_627_;
        J3_ = _e85;
    }
    if lh {
        f2_ = f32(((_e65.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e67 == 1u) {
        let _e130 = unpack4x8unorm(_e65.y);
        if lh {
            phi_628_ = _e130;
        } else {
            let _e133 = (_e130.xyz * _e130.w);
            let _e139 = vec4<f32>(_e133.x, _e130.y, _e130.z, _e130.w);
            let _e145 = vec4<f32>(_e139.x, _e133.y, _e139.z, _e139.w);
            phi_628_ = vec4<f32>(_e145.x, _e145.y, _e133.z, _e145.w);
        }
        let _e153 = phi_628_;
        f1_ = _e153;
    } else {
        let _e91 = (_e46 * 8u);
        let _e94 = QB.d2_[_e91];
        let _e105 = QB.d2_[(_e91 + 1u)];
        let _e108 = ((mat2x2<f32>(vec2<f32>(_e94.x, _e94.y), vec2<f32>(_e94.z, _e94.w)) * _e53) + _e105.xy);
        f1_[3u] = -(bitcast<f32>(_e65.y));
        if (_e105.z > 0.9f) {
            f1_[2u] = 2f;
        } else {
            f1_[2u] = _e105.w;
        }
        if (_e67 == 2u) {
            f1_[1u] = 0f;
            f1_[0u] = _e108.x;
        } else {
            let _e120 = f1_[2u];
            f1_[2u] = -(_e120);
            f1_[0u] = _e108.x;
            f1_[1u] = _e108.y;
        }
    }
    phi_381_ = rh;
    if rh {
        phi_381_ = ((_e65.x & 2048u) != 0u);
    }
    let _e157 = phi_381_;
    if _e157 {
        let _e158 = (_e46 * 8u);
        let _e162 = QB.d2_[(_e158 + 4u)];
        let _e173 = QB.d2_[(_e158 + 5u)];
        let _e176 = ((mat2x2<f32>(vec2<f32>(_e162.x, _e162.y), vec2<f32>(_e162.z, _e162.w)) * _e53) + _e173.xy);
        A2_ = vec3<f32>(_e176.x, _e176.y, (1f + _e173.z));
    } else {
        A2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e183 = m.of_;
    let _e185 = m.pf;
    let _e193 = vec4<f32>(((_e43.x * _e183) - 1f), ((_e43.y * _e185) - sign(_e185)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e193.x, _e193.y, (1f - (f32(_e51.x) * 0.000061035156f)), _e193.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) KB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    KB_1 = KB;
    main_1();
    let _e12 = D2_;
    let _e13 = J3_;
    let _e14 = f2_;
    let _e15 = f1_;
    let _e16 = A2_;
    let _e17 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
