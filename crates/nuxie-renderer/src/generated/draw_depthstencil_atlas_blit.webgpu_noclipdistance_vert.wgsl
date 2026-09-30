struct Cg {
    e2_: array<vec4<u32>>,
}

struct BC {
    rc: f32,
    Bd: f32,
    Ff: f32,
    Gf: f32,
    q6_: u32,
    Ob: u32,
    rf: u32,
    sf: u32,
    V7_: vec4<i32>,
    ch: vec2<f32>,
    Cd: vec2<f32>,
    d2_: u32,
    gh: f32,
    f6_: u32,
    U2_: f32,
    Dd: f32,
    mf: u32,
    C3_: f32,
    D3_: f32,
    Ed: f32,
    Zg: u32,
    Nb: u32,
}

struct jf {
    e2_: array<vec2<u32>>,
}

struct kf {
    e2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Dg {
    e2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(1) member: vec2<f32>,
    @location(4) @interpolate(flat, either) member_1: f32,
    @location(6) @interpolate(flat, either) member_2: f32,
    @location(0) member_3: vec4<f32>,
    @location(9) member_4: vec3<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override Ch: bool = true;
@id(2) override Eh: bool = true;
@id(8) override Kh: bool = true;

@group(0) @binding(2)
var<storage> PB: Cg;
@group(0) @binding(0)
var<uniform> l: BC;
var<private> gl_VertexIndex_1: i32;
var<private> KB_1: vec3<f32>;
var<private> F2_: vec2<f32>;
@group(0) @binding(3)
var<storage> DD: jf;
var<private> L3_: f32;
var<private> g2_: f32;
@group(0) @binding(4)
var<storage> QB: kf;
var<private> V1_: vec4<f32>;
var<private> C2_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> ID: Dg;
@group(3) @binding(9)
var ea: sampler;

fn main_1() {
    var phi_715_: u32;
    var phi_716_: f32;
    var phi_717_: f32;
    var phi_724_: vec4<f32>;
    var phi_725_: vec4<f32>;
    var phi_726_: vec4<f32>;
    var phi_412_: bool;

    let _e44 = KB_1;
    let _e47 = (bitcast<u32>(_e44.z) & 65535u);
    let _e52 = PB.e2_[((_e47 * 4u) + 2u)];
    let _e54 = _e44.xy;
    let _e56 = bitcast<vec3<f32>>(_e52.yzw);
    let _e62 = l.ch;
    F2_ = (((_e54 * _e56.x) + _e56.yz) * _e62);
    let _e66 = DD.e2_[_e47];
    let _e68 = (_e66.x & 15u);
    if Ch {
        let _e69 = (_e68 == 0u);
        if _e69 {
            phi_715_ = _e66.y;
        } else {
            phi_715_ = _e66.x;
        }
        let _e72 = phi_715_;
        let _e74 = (_e72 >> bitcast<u32>(16i));
        let _e76 = l.f6_;
        if (_e74 == 0u) {
            phi_716_ = 0f;
        } else {
            phi_716_ = unpack2x16float(((_e74 + 1023u) * _e76)).x;
        }
        let _e83 = phi_716_;
        phi_717_ = _e83;
        if _e69 {
            phi_717_ = -(_e83);
        }
        let _e86 = phi_717_;
        L3_ = _e86;
    }
    if Eh {
        g2_ = f32(((_e66.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e68 == 1u) {
        let _e170 = unpack4x8unorm(_e66.y);
        if Eh {
            phi_726_ = _e170;
        } else {
            let _e173 = (_e170.xyz * _e170.w);
            let _e179 = vec4<f32>(_e173.x, _e170.y, _e170.z, _e170.w);
            let _e185 = vec4<f32>(_e179.x, _e173.y, _e179.z, _e179.w);
            phi_726_ = vec4<f32>(_e185.x, _e185.y, _e173.z, _e185.w);
        }
        let _e193 = phi_726_;
        V1_ = _e193;
    } else {
        let _e92 = (_e47 * 8u);
        let _e95 = QB.e2_[_e92];
        let _e106 = QB.e2_[(_e92 + 1u)];
        let _e115 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e66.y));
        let _e117 = ((mat2x2<f32>(vec2<f32>(_e95.x, _e95.y), vec2<f32>(_e95.z, _e95.w)) * _e54) + _e106.xy);
        if (_e106.z > 0.9f) {
            phi_724_ = vec4<f32>(_e115.x, _e115.y, 2f, _e115.w);
        } else {
            phi_724_ = vec4<f32>(_e115.x, _e115.y, _e106.w, _e115.w);
        }
        let _e132 = phi_724_;
        if (f32(_e68) == 2f) {
            let _e158 = vec4<f32>(_e117.x, _e132.y, _e132.z, _e132.w);
            phi_725_ = vec4<f32>(_e158.x, 0f, _e158.z, _e158.w);
        } else {
            let _e140 = vec4<f32>(_e132.x, _e132.y, -(_e132.z), _e132.w);
            let _e146 = vec4<f32>(_e117.x, _e140.y, _e140.z, _e140.w);
            phi_725_ = vec4<f32>(_e146.x, _e117.y, _e146.z, _e146.w);
        }
        let _e165 = phi_725_;
        V1_ = _e165;
        let _e167 = V1_[3u];
        V1_[3u] = -(_e167);
    }
    phi_412_ = Kh;
    if Kh {
        phi_412_ = ((_e66.x & 2048u) != 0u);
    }
    let _e197 = phi_412_;
    if _e197 {
        let _e198 = (_e47 * 8u);
        let _e202 = QB.e2_[(_e198 + 4u)];
        let _e213 = QB.e2_[(_e198 + 5u)];
        let _e216 = ((mat2x2<f32>(vec2<f32>(_e202.x, _e202.y), vec2<f32>(_e202.z, _e202.w)) * _e54) + _e213.xy);
        C2_ = vec3<f32>(_e216.x, _e216.y, (1f + _e213.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e223 = l.Ff;
    let _e225 = l.Gf;
    let _e233 = vec4<f32>(((_e44.x * _e223) - 1f), ((_e44.y * _e225) - sign(_e225)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e233.x, _e233.y, (1f - (f32(_e52.x) * 0.000061035156f)), _e233.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) KB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    KB_1 = KB;
    main_1();
    let _e12 = F2_;
    let _e13 = L3_;
    let _e14 = g2_;
    let _e15 = V1_;
    let _e16 = C2_;
    let _e17 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
