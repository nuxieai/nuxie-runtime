struct jd {
    x2_: array<vec4<u32>>,
}

struct WB {
    Gd: f32,
    Pe: f32,
    Rg: f32,
    Sg: f32,
    R6_: u32,
    Fa: u32,
    Dg: u32,
    Eg: u32,
    H8_: vec4<i32>,
    Ji: vec2<f32>,
    Qe: vec2<f32>,
    w2_: u32,
    Ni: f32,
    w6_: u32,
    j3_: f32,
    Re: f32,
    wg: u32,
    I3_: f32,
    J3_: f32,
    Se: f32,
    Gi: u32,
    Ea: u32,
    O8_: f32,
    P8_: f32,
}

struct ug {
    x2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct pi {
    x2_: array<vec4<u32>>,
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
var<storage> JB: jd;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> W2_: vec2<f32>;
var<private> h4_: f32;
var<private> R0_: f32;
@group(0) @binding(3)
var<storage> KB: ug;
var<private> M0_: vec4<f32>;
var<private> N0_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(6)
var VB: texture_2d<u32>;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(4)
var<storage> CD: pi;
@group(3) @binding(8)
var fb: sampler;

fn main_1() {
    var phi_754_: u32;
    var phi_755_: f32;
    var phi_756_: f32;
    var phi_763_: f32;
    var phi_416_: bool;
    var phi_765_: f32;

    let _e51 = LB_1;
    let _e54 = (bitcast<u32>(_e51.z) & 65535u);
    let _e55 = (_e54 * 4u);
    let _e56 = (_e55 + 1u);
    let _e60 = JB.x2_[_e56][2u];
    let _e63 = _e51.xy;
    let _e67 = JB.x2_[(_e55 + 3u)];
    let _e69 = bitcast<vec3<f32>>(_e67.xyz);
    let _e75 = j.Ji;
    W2_ = (((_e63 * _e69.x) + _e69.yz) * _e75);
    let _e79 = JB.x2_[_e56];
    let _e81 = (_e79.z & 15u);
    if jj {
        let _e82 = (_e81 == 0u);
        if _e82 {
            phi_754_ = _e79.w;
        } else {
            phi_754_ = _e79.z;
        }
        let _e85 = phi_754_;
        let _e87 = (_e85 >> bitcast<u32>(16i));
        let _e89 = j.w6_;
        if (_e87 == 0u) {
            phi_755_ = 0f;
        } else {
            phi_755_ = unpack2x16float(((_e87 + 1023u) * _e89)).x;
        }
        let _e96 = phi_755_;
        phi_756_ = _e96;
        if _e82 {
            phi_756_ = -(_e96);
        }
        let _e99 = phi_756_;
        h4_ = _e99;
    }
    if lj {
        R0_ = f32(((_e79.z >> bitcast<u32>(4i)) & 15u));
    }
    if (_e81 == 1u) {
        M0_ = unpack4x8unorm(_e79.w);
    } else {
        let _e105 = (_e54 * 8u);
        let _e108 = KB.x2_[_e105];
        let _e119 = KB.x2_[(_e105 + 1u)];
        let _e124 = ((mat2x2<f32>(vec2<f32>(_e108.x, _e108.y), vec2<f32>(_e108.z, _e108.w)) * _e63) + _e119.xy);
        let _e135 = ((_e119.w + (f32(_e81) * 0.125f)) + (max(_e119.z, 0f) * 0.00024414063f));
        if (_e119.z < 0f) {
            phi_763_ = -(_e135);
        } else {
            phi_763_ = _e135;
        }
        let _e138 = phi_763_;
        M0_ = vec4<f32>(_e124.x, _e124.y, _e138, (-0.75f - round((bitcast<f32>(_e79.w) * 255f))));
    }
    phi_416_ = nj;
    if nj {
        phi_416_ = ((_e79.z & 2048u) != 0u);
    }
    let _e148 = phi_416_;
    if _e148 {
        let _e149 = (_e54 * 8u);
        let _e153 = KB.x2_[(_e149 + 4u)];
        let _e164 = KB.x2_[(_e149 + 5u)];
        let _e167 = ((mat2x2<f32>(vec2<f32>(_e153.x, _e153.y), vec2<f32>(_e153.z, _e153.w)) * _e63) + _e164.xy);
        phi_765_ = (1f + _e164.z);
        if ((_e79.z & 4096u) != 0u) {
            phi_765_ = (-1f - f32(((_e79.z & 24576u) >> bitcast<u32>(13u))));
        }
        let _e178 = phi_765_;
        N0_ = vec3<f32>(_e167.x, _e167.y, _e178);
    } else {
        N0_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e183 = j.Rg;
    let _e185 = j.Sg;
    let _e193 = vec4<f32>(((_e51.x * _e183) - 1f), ((_e51.y * _e185) - sign(_e185)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e193.x, _e193.y, ((f32((((_e60 >> bitcast<u32>(16u)) << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e193.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e12 = W2_;
    let _e13 = h4_;
    let _e14 = R0_;
    let _e15 = M0_;
    let _e16 = N0_;
    let _e17 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
