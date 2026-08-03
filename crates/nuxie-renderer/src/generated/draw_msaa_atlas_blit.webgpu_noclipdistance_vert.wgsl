struct Zf {
    c2_: array<vec4<u32>>,
}

struct CC {
    cc: f32,
    md: f32,
    df: f32,
    ef: f32,
    m6_: u32,
    Dg: u32,
    Pe: u32,
    Qe: u32,
    R7_: vec4<i32>,
    zg: vec2<f32>,
    nd: vec2<f32>,
    a2_: u32,
    Eg: f32,
    Z5_: u32,
    P2_: f32,
    od: f32,
    Ke: u32,
    z3_: f32,
    A3_: f32,
    pd: f32,
    wg: u32,
}

struct He {
    c2_: array<vec2<u32>>,
}

struct Ie {
    c2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct ag {
    c2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(1) member: vec2<f32>,
    @location(4) @interpolate(flat, either) member_1: f32,
    @location(6) @interpolate(flat, either) member_2: f32,
    @location(0) member_3: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override Wg: bool = true;
@id(2) override Yg: bool = true;

@group(0) @binding(2)
var<storage> PB: Zf;
@group(0) @binding(0)
var<uniform> n: CC;
var<private> gl_VertexIndex_1: i32;
var<private> KB_1: vec3<f32>;
var<private> C2_: vec2<f32>;
@group(0) @binding(3)
var<storage> AD: He;
var<private> I3_: f32;
var<private> e2_: f32;
@group(0) @binding(4)
var<storage> RB: Ie;
var<private> f1_: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var LC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> ED: ag;
@group(3) @binding(9)
var aa: sampler;

fn main_1() {
    var phi_589_: u32;
    var phi_590_: f32;
    var phi_591_: f32;
    var phi_592_: vec4<f32>;

    let _e38 = KB_1;
    let _e41 = (bitcast<u32>(_e38.z) & 65535u);
    let _e42 = (_e41 * 4u);
    let _e46 = PB.c2_[(_e42 + 2u)];
    let _e48 = _e38.xy;
    let _e50 = bitcast<vec3<f32>>(_e46.yzw);
    let _e56 = n.zg;
    C2_ = (((_e48 * _e50.x) + _e50.yz) * _e56);
    let _e60 = AD.c2_[_e41];
    let _e62 = (_e60.x & 15u);
    if Wg {
        let _e63 = (_e62 == 0u);
        if _e63 {
            phi_589_ = _e60.y;
        } else {
            phi_589_ = _e60.x;
        }
        let _e66 = phi_589_;
        let _e68 = (_e66 >> bitcast<u32>(16i));
        let _e70 = n.Z5_;
        if (_e68 == 0u) {
            phi_590_ = 0f;
        } else {
            phi_590_ = unpack2x16float(((_e68 + 1023u) * _e70)).x;
        }
        let _e77 = phi_590_;
        phi_591_ = _e77;
        if _e63 {
            phi_591_ = -(_e77);
        }
        let _e80 = phi_591_;
        I3_ = _e80;
    }
    if Yg {
        e2_ = f32(((_e60.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e62 == 1u) {
        let _e133 = unpack4x8unorm(_e60.y);
        if Yg {
            phi_592_ = _e133;
        } else {
            let _e136 = (_e133.xyz * _e133.w);
            let _e142 = vec4<f32>(_e136.x, _e133.y, _e133.z, _e133.w);
            let _e148 = vec4<f32>(_e142.x, _e136.y, _e142.z, _e142.w);
            phi_592_ = vec4<f32>(_e148.x, _e148.y, _e136.z, _e148.w);
        }
        let _e156 = phi_592_;
        f1_ = _e156;
    } else {
        let _e88 = RB.c2_[_e42];
        let _e99 = RB.c2_[(_e42 + 1u)];
        let _e102 = ((mat2x2<f32>(vec2<f32>(_e88.x, _e88.y), vec2<f32>(_e88.z, _e88.w)) * _e48) + _e99.xy);
        let _e103 = (_e62 == 2u);
        if (_e103 || (_e62 == 3u)) {
            f1_[3u] = -(bitcast<f32>(_e60.y));
            if (_e99.z > 0.9f) {
                f1_[2u] = 2f;
            } else {
                f1_[2u] = _e99.w;
            }
            if _e103 {
                f1_[1u] = 0f;
                f1_[0u] = _e102.x;
            } else {
                let _e123 = f1_[2u];
                f1_[2u] = -(_e123);
                f1_[0u] = _e102.x;
                f1_[1u] = _e102.y;
            }
        } else {
            f1_ = vec4<f32>(_e102.x, _e102.y, bitcast<f32>(_e60.y), (-2f - _e99.z));
        }
    }
    let _e158 = n.df;
    let _e160 = n.ef;
    let _e168 = vec4<f32>(((_e38.x * _e158) - 1f), ((_e38.y * _e160) - sign(_e160)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e168.x, _e168.y, (1f - (f32(_e46.x) * 0.000061035156f)), _e168.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) KB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    KB_1 = KB;
    main_1();
    let _e11 = C2_;
    let _e12 = I3_;
    let _e13 = e2_;
    let _e14 = f1_;
    let _e15 = unnamed.gl_Position;
    return VertexOutput(_e11, _e12, _e13, _e14, _e15);
}
