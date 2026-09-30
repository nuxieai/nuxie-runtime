struct Jg {
    g2_: array<vec4<u32>>,
}

struct SB {
    xc: f32,
    Hd: f32,
    Mf: f32,
    Nf: f32,
    r6_: u32,
    Rb: u32,
    yf: u32,
    zf: u32,
    X7_: vec4<i32>,
    jh: vec2<f32>,
    Id: vec2<f32>,
    f2_: u32,
    nh: f32,
    g6_: u32,
    W2_: f32,
    Jd: f32,
    sf: u32,
    F3_: f32,
    G3_: f32,
    Kd: f32,
    gh: u32,
    Qb: u32,
    dc: f32,
    ec: f32,
}

struct pf {
    g2_: array<vec2<u32>>,
}

struct qf {
    g2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Kg {
    g2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(1) member: vec2<f32>,
    @location(4) @interpolate(flat, either) member_1: f32,
    @location(6) @interpolate(flat, either) member_2: f32,
    @location(0) member_3: vec4<f32>,
    @location(9) member_4: vec3<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override Jh: bool = true;
@id(2) override Lh: bool = true;
@id(8) override Rh: bool = true;

@group(0) @binding(2)
var<storage> OB: Jg;
@group(0) @binding(0)
var<uniform> j: SB;
var<private> gl_VertexIndex_1: i32;
var<private> JB_1: vec3<f32>;
var<private> F2_: vec2<f32>;
@group(0) @binding(3)
var<storage> CD: pf;
var<private> O3_: f32;
var<private> g1_: f32;
@group(0) @binding(4)
var<storage> PB: qf;
var<private> X1_: vec4<f32>;
var<private> C2_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> HD: Kg;
@group(3) @binding(9)
var ha: sampler;

fn main_1() {
    var phi_692_: u32;
    var phi_693_: f32;
    var phi_694_: f32;
    var phi_701_: vec4<f32>;
    var phi_702_: vec4<f32>;
    var phi_396_: bool;

    let _e44 = JB_1;
    let _e47 = (bitcast<u32>(_e44.z) & 65535u);
    let _e52 = OB.g2_[((_e47 * 4u) + 2u)];
    let _e54 = _e44.xy;
    let _e56 = bitcast<vec3<f32>>(_e52.yzw);
    let _e62 = j.jh;
    F2_ = (((_e54 * _e56.x) + _e56.yz) * _e62);
    let _e66 = CD.g2_[_e47];
    let _e68 = (_e66.x & 15u);
    if Jh {
        let _e69 = (_e68 == 0u);
        if _e69 {
            phi_692_ = _e66.y;
        } else {
            phi_692_ = _e66.x;
        }
        let _e72 = phi_692_;
        let _e74 = (_e72 >> bitcast<u32>(16i));
        let _e76 = j.g6_;
        if (_e74 == 0u) {
            phi_693_ = 0f;
        } else {
            phi_693_ = unpack2x16float(((_e74 + 1023u) * _e76)).x;
        }
        let _e83 = phi_693_;
        phi_694_ = _e83;
        if _e69 {
            phi_694_ = -(_e83);
        }
        let _e86 = phi_694_;
        O3_ = _e86;
    }
    if Lh {
        g1_ = f32(((_e66.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e68 == 1u) {
        X1_ = unpack4x8unorm(_e66.y);
    } else {
        let _e92 = (_e47 * 8u);
        let _e95 = PB.g2_[_e92];
        let _e106 = PB.g2_[(_e92 + 1u)];
        let _e115 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e66.y));
        let _e117 = ((mat2x2<f32>(vec2<f32>(_e95.x, _e95.y), vec2<f32>(_e95.z, _e95.w)) * _e54) + _e106.xy);
        if (_e106.z > 0.9f) {
            phi_701_ = vec4<f32>(_e115.x, _e115.y, 2f, _e115.w);
        } else {
            phi_701_ = vec4<f32>(_e115.x, _e115.y, _e106.w, _e115.w);
        }
        let _e132 = phi_701_;
        if (f32(_e68) == 2f) {
            let _e158 = vec4<f32>(_e117.x, _e132.y, _e132.z, _e132.w);
            phi_702_ = vec4<f32>(_e158.x, 0f, _e158.z, _e158.w);
        } else {
            let _e140 = vec4<f32>(_e132.x, _e132.y, -(_e132.z), _e132.w);
            let _e146 = vec4<f32>(_e117.x, _e140.y, _e140.z, _e140.w);
            phi_702_ = vec4<f32>(_e146.x, _e117.y, _e146.z, _e146.w);
        }
        let _e165 = phi_702_;
        X1_ = _e165;
        let _e167 = X1_[3u];
        X1_[3u] = -(_e167);
    }
    phi_396_ = Rh;
    if Rh {
        phi_396_ = ((_e66.x & 2048u) != 0u);
    }
    let _e174 = phi_396_;
    if _e174 {
        let _e175 = (_e47 * 8u);
        let _e179 = PB.g2_[(_e175 + 4u)];
        let _e190 = PB.g2_[(_e175 + 5u)];
        let _e193 = ((mat2x2<f32>(vec2<f32>(_e179.x, _e179.y), vec2<f32>(_e179.z, _e179.w)) * _e54) + _e190.xy);
        C2_ = vec3<f32>(_e193.x, _e193.y, (1f + _e190.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e200 = j.Mf;
    let _e202 = j.Nf;
    let _e210 = vec4<f32>(((_e44.x * _e200) - 1f), ((_e44.y * _e202) - sign(_e202)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e210.x, _e210.y, (1f - (f32(_e52.x) * 0.000061035156f)), _e210.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) JB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    JB_1 = JB;
    main_1();
    let _e12 = F2_;
    let _e13 = O3_;
    let _e14 = g1_;
    let _e15 = X1_;
    let _e16 = C2_;
    let _e17 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
