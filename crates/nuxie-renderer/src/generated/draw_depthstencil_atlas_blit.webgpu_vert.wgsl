enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

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

struct Kg {
    g2_: array<vec4<u32>>,
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

@id(0) override Jh: bool = true;
@id(2) override Lh: bool = true;
@id(1) override Kh: bool = true;
@id(8) override Rh: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
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
@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> HD: Kg;
@group(3) @binding(9)
var ha: sampler;

fn main_1() {
    var phi_782_: u32;
    var phi_783_: f32;
    var phi_784_: f32;
    var phi_793_: vec4<f32>;
    var phi_794_: vec4<f32>;
    var phi_447_: bool;

    let _e48 = JB_1;
    let _e51 = (bitcast<u32>(_e48.z) & 65535u);
    let _e56 = OB.g2_[((_e51 * 4u) + 2u)];
    let _e58 = _e48.xy;
    let _e60 = bitcast<vec3<f32>>(_e56.yzw);
    let _e66 = j.jh;
    F2_ = (((_e58 * _e60.x) + _e60.yz) * _e66);
    let _e70 = CD.g2_[_e51];
    let _e72 = (_e70.x & 15u);
    if Jh {
        let _e73 = (_e72 == 0u);
        if _e73 {
            phi_782_ = _e70.y;
        } else {
            phi_782_ = _e70.x;
        }
        let _e76 = phi_782_;
        let _e78 = (_e76 >> bitcast<u32>(16i));
        let _e80 = j.g6_;
        if (_e78 == 0u) {
            phi_783_ = 0f;
        } else {
            phi_783_ = unpack2x16float(((_e78 + 1023u) * _e80)).x;
        }
        let _e87 = phi_783_;
        phi_784_ = _e87;
        if _e73 {
            phi_784_ = -(_e87);
        }
        let _e90 = phi_784_;
        O3_ = _e90;
    }
    if Lh {
        g1_ = f32(((_e70.x >> bitcast<u32>(4i)) & 15u));
    }
    if Kh {
        let _e95 = (_e51 * 8u);
        let _e99 = PB.g2_[(_e95 + 2u)];
        let _e110 = PB.g2_[(_e95 + 3u)];
        if any((_e99 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e125 = ((mat2x2<f32>(vec2<f32>(_e99.x, _e99.y), vec2<f32>(_e99.z, _e99.w)) * _e58) + _e110.xy);
            unnamed.gl_ClipDistance[0i] = (_e125.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e125.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e125.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e125.y);
        } else {
            let _e115 = (_e110.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e115;
            unnamed.gl_ClipDistance[2i] = _e115;
            unnamed.gl_ClipDistance[1i] = _e115;
            unnamed.gl_ClipDistance[0i] = _e115;
        }
    }
    if (_e72 == 1u) {
        X1_ = unpack4x8unorm(_e70.y);
    } else {
        let _e141 = (_e51 * 8u);
        let _e144 = PB.g2_[_e141];
        let _e155 = PB.g2_[(_e141 + 1u)];
        let _e164 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e70.y));
        let _e166 = ((mat2x2<f32>(vec2<f32>(_e144.x, _e144.y), vec2<f32>(_e144.z, _e144.w)) * _e58) + _e155.xy);
        if (_e155.z > 0.9f) {
            phi_793_ = vec4<f32>(_e164.x, _e164.y, 2f, _e164.w);
        } else {
            phi_793_ = vec4<f32>(_e164.x, _e164.y, _e155.w, _e164.w);
        }
        let _e181 = phi_793_;
        if (f32(_e72) == 2f) {
            let _e207 = vec4<f32>(_e166.x, _e181.y, _e181.z, _e181.w);
            phi_794_ = vec4<f32>(_e207.x, 0f, _e207.z, _e207.w);
        } else {
            let _e189 = vec4<f32>(_e181.x, _e181.y, -(_e181.z), _e181.w);
            let _e195 = vec4<f32>(_e166.x, _e189.y, _e189.z, _e189.w);
            phi_794_ = vec4<f32>(_e195.x, _e166.y, _e195.z, _e195.w);
        }
        let _e214 = phi_794_;
        X1_ = _e214;
        let _e216 = X1_[3u];
        X1_[3u] = -(_e216);
    }
    phi_447_ = Rh;
    if Rh {
        phi_447_ = ((_e70.x & 2048u) != 0u);
    }
    let _e223 = phi_447_;
    if _e223 {
        let _e224 = (_e51 * 8u);
        let _e228 = PB.g2_[(_e224 + 4u)];
        let _e239 = PB.g2_[(_e224 + 5u)];
        let _e242 = ((mat2x2<f32>(vec2<f32>(_e228.x, _e228.y), vec2<f32>(_e228.z, _e228.w)) * _e58) + _e239.xy);
        C2_ = vec3<f32>(_e242.x, _e242.y, (1f + _e239.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e249 = j.Mf;
    let _e251 = j.Nf;
    let _e259 = vec4<f32>(((_e48.x * _e249) - 1f), ((_e48.y * _e251) - sign(_e251)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e259.x, _e259.y, (1f - (f32(_e56.x) * 0.000061035156f)), _e259.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) JB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    JB_1 = JB;
    main_1();
    let _e13 = unnamed.gl_Position;
    let _e14 = unnamed.gl_ClipDistance;
    let _e15 = F2_;
    let _e16 = O3_;
    let _e17 = g1_;
    let _e18 = X1_;
    let _e19 = C2_;
    return VertexOutput(_e13, _e14, _e15, _e16, _e17, _e18, _e19);
}
