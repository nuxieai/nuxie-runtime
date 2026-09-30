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

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct eh {
    j2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(1) member: vec2<f32>,
    @location(4) @interpolate(flat, either) member_1: f32,
    @location(6) @interpolate(flat, either) member_2: f32,
    @location(0) member_3: vec4<f32>,
    @location(9) member_4: vec3<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override bi: bool = true;
@id(2) override di: bool = true;
@id(8) override ji: bool = true;

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
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> AD: eh;
@group(3) @binding(9)
var wa: sampler;

fn main_1() {
    var phi_708_: u32;
    var phi_709_: f32;
    var phi_710_: f32;
    var phi_717_: vec4<f32>;
    var phi_718_: vec4<f32>;
    var phi_404_: bool;

    let _e46 = MB_1;
    let _e49 = (bitcast<u32>(_e46.z) & 65535u);
    let _e54 = LB.j2_[((_e49 * 4u) + 2u)];
    let _e56 = _e46.xy;
    let _e58 = bitcast<vec3<f32>>(_e54.yzw);
    let _e64 = j.Ch;
    K2_ = (((_e56 * _e58.x) + _e58.yz) * _e64);
    let _e68 = XC.j2_[_e49];
    let _e70 = (_e68.x & 15u);
    if bi {
        let _e71 = (_e70 == 0u);
        if _e71 {
            phi_708_ = _e68.y;
        } else {
            phi_708_ = _e68.x;
        }
        let _e74 = phi_708_;
        let _e76 = (_e74 >> bitcast<u32>(16i));
        let _e78 = j.T4_;
        if (_e76 == 0u) {
            phi_709_ = 0f;
        } else {
            phi_709_ = unpack2x16float(((_e76 + 1023u) * _e78)).x;
        }
        let _e85 = phi_709_;
        phi_710_ = _e85;
        if _e71 {
            phi_710_ = -(_e85);
        }
        let _e88 = phi_710_;
        Z3_ = _e88;
    }
    if di {
        Q0_ = f32(((_e68.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e70 == 1u) {
        a1_ = unpack4x8unorm(_e68.y);
    } else {
        let _e94 = (_e49 * 8u);
        let _e97 = JB.j2_[_e94];
        let _e108 = JB.j2_[(_e94 + 1u)];
        let _e117 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e68.y));
        let _e119 = ((mat2x2<f32>(vec2<f32>(_e97.x, _e97.y), vec2<f32>(_e97.z, _e97.w)) * _e56) + _e108.xy);
        if (_e108.z > 0.9f) {
            phi_717_ = vec4<f32>(_e117.x, _e117.y, 2f, _e117.w);
        } else {
            phi_717_ = vec4<f32>(_e117.x, _e117.y, _e108.w, _e117.w);
        }
        let _e134 = phi_717_;
        if (f32(_e70) == 2f) {
            let _e160 = vec4<f32>(_e119.x, _e134.y, _e134.z, _e134.w);
            phi_718_ = vec4<f32>(_e160.x, 0f, _e160.z, _e160.w);
        } else {
            let _e142 = vec4<f32>(_e134.x, _e134.y, -(_e134.z), _e134.w);
            let _e148 = vec4<f32>(_e119.x, _e142.y, _e142.z, _e142.w);
            phi_718_ = vec4<f32>(_e148.x, _e119.y, _e148.z, _e148.w);
        }
        let _e167 = phi_718_;
        a1_ = _e167;
        let _e169 = a1_[3u];
        a1_[3u] = -(_e169);
    }
    phi_404_ = ji;
    if ji {
        phi_404_ = ((_e68.x & 2048u) != 0u);
    }
    let _e176 = phi_404_;
    if _e176 {
        let _e177 = (_e49 * 8u);
        let _e181 = JB.j2_[(_e177 + 4u)];
        let _e192 = JB.j2_[(_e177 + 5u)];
        let _e195 = ((mat2x2<f32>(vec2<f32>(_e181.x, _e181.y), vec2<f32>(_e181.z, _e181.w)) * _e56) + _e192.xy);
        F1_ = vec3<f32>(_e195.x, _e195.y, (1f + _e192.z));
    } else {
        F1_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e202 = j.Yf;
    let _e204 = j.Zf;
    let _e212 = vec4<f32>(((_e46.x * _e202) - 1f), ((_e46.y * _e204) - sign(_e204)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e212.x, _e212.y, ((f32(((_e54.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e212.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) MB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    MB_1 = MB;
    main_1();
    let _e12 = K2_;
    let _e13 = Z3_;
    let _e14 = Q0_;
    let _e15 = a1_;
    let _e16 = F1_;
    let _e17 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
