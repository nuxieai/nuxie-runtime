struct Ig {
    g2_: array<vec4<u32>>,
}

struct TB {
    uc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Ob: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    ih: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    mh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    fh: u32,
    Nb: u32,
    ac: f32,
    bc: f32,
}

struct kf {
    g2_: array<vec2<u32>>,
}

struct lf {
    g2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Jg {
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

@id(0) override Ih: bool = true;
@id(2) override Kh: bool = true;
@id(8) override Qh: bool = true;

@group(0) @binding(2)
var<storage> OB: Ig;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> gl_VertexIndex_1: i32;
var<private> JB_1: vec3<f32>;
var<private> F2_: vec2<f32>;
@group(0) @binding(3)
var<storage> DD: kf;
var<private> O3_: f32;
var<private> g1_: f32;
@group(0) @binding(4)
var<storage> PB: lf;
var<private> X1_: vec4<f32>;
var<private> C2_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> ID: Jg;
@group(3) @binding(9)
var da: sampler;

fn main_1() {
    var phi_709_: u32;
    var phi_710_: f32;
    var phi_711_: f32;
    var phi_718_: vec4<f32>;
    var phi_719_: vec4<f32>;
    var phi_404_: bool;

    let _e46 = JB_1;
    let _e49 = (bitcast<u32>(_e46.z) & 65535u);
    let _e54 = OB.g2_[((_e49 * 4u) + 2u)];
    let _e56 = _e46.xy;
    let _e58 = bitcast<vec3<f32>>(_e54.yzw);
    let _e64 = j.ih;
    F2_ = (((_e56 * _e58.x) + _e58.yz) * _e64);
    let _e68 = DD.g2_[_e49];
    let _e70 = (_e68.x & 15u);
    if Ih {
        let _e71 = (_e70 == 0u);
        if _e71 {
            phi_709_ = _e68.y;
        } else {
            phi_709_ = _e68.x;
        }
        let _e74 = phi_709_;
        let _e76 = (_e74 >> bitcast<u32>(16i));
        let _e78 = j.c6_;
        if (_e76 == 0u) {
            phi_710_ = 0f;
        } else {
            phi_710_ = unpack2x16float(((_e76 + 1023u) * _e78)).x;
        }
        let _e85 = phi_710_;
        phi_711_ = _e85;
        if _e71 {
            phi_711_ = -(_e85);
        }
        let _e88 = phi_711_;
        O3_ = _e88;
    }
    if Kh {
        g1_ = f32(((_e68.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e70 == 1u) {
        X1_ = unpack4x8unorm(_e68.y);
    } else {
        let _e94 = (_e49 * 8u);
        let _e97 = PB.g2_[_e94];
        let _e108 = PB.g2_[(_e94 + 1u)];
        let _e117 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e68.y));
        let _e119 = ((mat2x2<f32>(vec2<f32>(_e97.x, _e97.y), vec2<f32>(_e97.z, _e97.w)) * _e56) + _e108.xy);
        if (_e108.z > 0.9f) {
            phi_718_ = vec4<f32>(_e117.x, _e117.y, 2f, _e117.w);
        } else {
            phi_718_ = vec4<f32>(_e117.x, _e117.y, _e108.w, _e117.w);
        }
        let _e134 = phi_718_;
        if (f32(_e70) == 2f) {
            let _e160 = vec4<f32>(_e119.x, _e134.y, _e134.z, _e134.w);
            phi_719_ = vec4<f32>(_e160.x, 0f, _e160.z, _e160.w);
        } else {
            let _e142 = vec4<f32>(_e134.x, _e134.y, -(_e134.z), _e134.w);
            let _e148 = vec4<f32>(_e119.x, _e142.y, _e142.z, _e142.w);
            phi_719_ = vec4<f32>(_e148.x, _e119.y, _e148.z, _e148.w);
        }
        let _e167 = phi_719_;
        X1_ = _e167;
        let _e169 = X1_[3u];
        X1_[3u] = -(_e169);
    }
    phi_404_ = Qh;
    if Qh {
        phi_404_ = ((_e68.x & 2048u) != 0u);
    }
    let _e176 = phi_404_;
    if _e176 {
        let _e177 = (_e49 * 8u);
        let _e181 = PB.g2_[(_e177 + 4u)];
        let _e192 = PB.g2_[(_e177 + 5u)];
        let _e195 = ((mat2x2<f32>(vec2<f32>(_e181.x, _e181.y), vec2<f32>(_e181.z, _e181.w)) * _e56) + _e192.xy);
        C2_ = vec3<f32>(_e195.x, _e195.y, (1f + _e192.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e202 = j.Hf;
    let _e204 = j.If;
    let _e212 = vec4<f32>(((_e46.x * _e202) - 1f), ((_e46.y * _e204) - sign(_e204)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e212.x, _e212.y, ((f32(((_e54.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e212.w);
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
