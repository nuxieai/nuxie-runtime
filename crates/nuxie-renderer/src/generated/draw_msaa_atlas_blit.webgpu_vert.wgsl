enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct Xf {
    c2_: array<vec4<u32>>,
}

struct CC {
    bc: f32,
    kd: f32,
    bf: f32,
    cf: f32,
    m6_: u32,
    Bg: u32,
    Ne: u32,
    Oe: u32,
    Q7_: vec4<i32>,
    xg: vec2<f32>,
    ld: vec2<f32>,
    a2_: u32,
    Cg: f32,
    Z5_: u32,
    N2_: f32,
    md: f32,
    Ie: u32,
    y3_: f32,
    z3_: f32,
    nd: f32,
    ug: u32,
}

struct Fe {
    c2_: array<vec2<u32>>,
}

struct Ge {
    c2_: array<vec4<f32>>,
}

struct Yf {
    c2_: array<vec4<u32>>,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(1) member: vec2<f32>,
    @location(4) @interpolate(flat) member_1: f32,
    @location(6) @interpolate(flat) member_2: f32,
    @location(0) member_3: vec4<f32>,
}

@id(0) override Ug: bool = true;
@id(2) override Wg: bool = true;
@id(1) override Vg: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(2)
var<storage> PB: Xf;
@group(0) @binding(0)
var<uniform> m: CC;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> B2_: vec2<f32>;
@group(0) @binding(3)
var<storage> AD: Fe;
var<private> H3_: f32;
var<private> e2_: f32;
@group(0) @binding(4)
var<storage> RB: Ge;
var<private> f1_: vec4<f32>;
@group(0) @binding(7)
var LC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> ED: Yf;
@group(3) @binding(9)
var Z9_: sampler;

fn main_1() {
    var phi_679_: u32;
    var phi_680_: f32;
    var phi_681_: f32;
    var phi_682_: vec4<f32>;

    let _e42 = LB_1;
    let _e45 = (bitcast<u32>(_e42.z) & 65535u);
    let _e46 = (_e45 * 4u);
    let _e47 = (_e46 + 2u);
    let _e50 = PB.c2_[_e47];
    let _e52 = _e42.xy;
    let _e54 = bitcast<vec3<f32>>(_e50.yzw);
    let _e60 = m.xg;
    B2_ = (((_e52 * _e54.x) + _e54.yz) * _e60);
    let _e64 = AD.c2_[_e45];
    let _e66 = (_e64.x & 15u);
    if Ug {
        let _e67 = (_e66 == 0u);
        if _e67 {
            phi_679_ = _e64.y;
        } else {
            phi_679_ = _e64.x;
        }
        let _e70 = phi_679_;
        let _e72 = (_e70 >> bitcast<u32>(16i));
        let _e74 = m.Z5_;
        if (_e72 == 0u) {
            phi_680_ = 0f;
        } else {
            phi_680_ = unpack2x16float(((_e72 + 1023u) * _e74)).x;
        }
        let _e81 = phi_680_;
        phi_681_ = _e81;
        if _e67 {
            phi_681_ = -(_e81);
        }
        let _e84 = phi_681_;
        H3_ = _e84;
    }
    if Wg {
        e2_ = f32(((_e64.x >> bitcast<u32>(4i)) & 15u));
    }
    if Vg {
        let _e91 = RB.c2_[_e47];
        let _e102 = RB.c2_[(_e46 + 3u)];
        if any((_e91 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e117 = ((mat2x2<f32>(vec2<f32>(_e91.x, _e91.y), vec2<f32>(_e91.z, _e91.w)) * _e52) + _e102.xy);
            unnamed.gl_ClipDistance[0i] = (_e117.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e117.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e117.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e117.y);
        } else {
            let _e107 = (_e102.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e107;
            unnamed.gl_ClipDistance[2i] = _e107;
            unnamed.gl_ClipDistance[1i] = _e107;
            unnamed.gl_ClipDistance[0i] = _e107;
        }
    }
    if (_e66 == 1u) {
        let _e180 = unpack4x8unorm(_e64.y);
        if Wg {
            phi_682_ = _e180;
        } else {
            let _e183 = (_e180.xyz * _e180.w);
            let _e189 = vec4<f32>(_e183.x, _e180.y, _e180.z, _e180.w);
            let _e195 = vec4<f32>(_e189.x, _e183.y, _e189.z, _e189.w);
            phi_682_ = vec4<f32>(_e195.x, _e195.y, _e183.z, _e195.w);
        }
        let _e203 = phi_682_;
        f1_ = _e203;
    } else {
        let _e135 = RB.c2_[_e46];
        let _e146 = RB.c2_[(_e46 + 1u)];
        let _e149 = ((mat2x2<f32>(vec2<f32>(_e135.x, _e135.y), vec2<f32>(_e135.z, _e135.w)) * _e52) + _e146.xy);
        let _e150 = (_e66 == 2u);
        if (_e150 || (_e66 == 3u)) {
            f1_[3u] = -(bitcast<f32>(_e64.y));
            if (_e146.z > 0.9f) {
                f1_[2u] = 2f;
            } else {
                f1_[2u] = _e146.w;
            }
            if _e150 {
                f1_[1u] = 0f;
                f1_[0u] = _e149.x;
            } else {
                let _e170 = f1_[2u];
                f1_[2u] = -(_e170);
                f1_[0u] = _e149.x;
                f1_[1u] = _e149.y;
            }
        } else {
            f1_ = vec4<f32>(_e149.x, _e149.y, bitcast<f32>(_e64.y), (-2f - _e146.z));
        }
    }
    let _e205 = m.bf;
    let _e207 = m.cf;
    let _e215 = vec4<f32>(((_e42.x * _e205) - 1f), ((_e42.y * _e207) - sign(_e207)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e215.x, _e215.y, (1f - (f32(_e50.x) * 0.000061035156f)), _e215.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e12 = unnamed.gl_Position;
    let _e13 = unnamed.gl_ClipDistance;
    let _e14 = B2_;
    let _e15 = H3_;
    let _e16 = e2_;
    let _e17 = f1_;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
