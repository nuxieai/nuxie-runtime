struct mh {
    k2_: array<vec4<u32>>,
}

struct UB {
    Qc: f32,
    Td: f32,
    bg: f32,
    cg: f32,
    A6_: u32,
    X9_: u32,
    Nf: u32,
    Of: u32,
    j8_: vec4<i32>,
    Lh: vec2<f32>,
    Ud: vec2<f32>,
    j2_: u32,
    Ph: f32,
    T4_: u32,
    a3_: f32,
    Vd: f32,
    Hf: u32,
    L3_: f32,
    M3_: f32,
    Wd: f32,
    Ih: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

struct Ef {
    k2_: array<vec2<u32>>,
}

struct Ff {
    k2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct nh {
    k2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(1) member: vec2<f32>,
    @location(4) @interpolate(flat, either) member_1: f32,
    @location(6) @interpolate(flat, either) member_2: f32,
    @location(0) member_3: vec4<f32>,
    @location(9) member_4: vec3<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override ki: bool = true;
@id(2) override mi: bool = true;
@id(8) override si: bool = true;

@group(0) @binding(2)
var<storage> LB: mh;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> gl_VertexIndex_1: i32;
var<private> MB_1: vec3<f32>;
var<private> J2_: vec2<f32>;
@group(0) @binding(3)
var<storage> WC: Ef;
var<private> Y3_: f32;
var<private> Q0_: f32;
@group(0) @binding(4)
var<storage> JB: Ff;
var<private> a1_: vec4<f32>;
var<private> r1_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> ZC: nh;
@group(3) @binding(9)
var wa: sampler;

fn main_1() {
    var phi_730_: u32;
    var phi_731_: f32;
    var phi_732_: f32;
    var phi_739_: vec4<f32>;
    var phi_740_: vec4<f32>;
    var phi_404_: bool;
    var phi_741_: f32;

    let _e50 = MB_1;
    let _e53 = (bitcast<u32>(_e50.z) & 65535u);
    let _e58 = LB.k2_[((_e53 * 4u) + 2u)];
    let _e60 = _e50.xy;
    let _e62 = bitcast<vec3<f32>>(_e58.yzw);
    let _e68 = j.Lh;
    J2_ = (((_e60 * _e62.x) + _e62.yz) * _e68);
    let _e72 = WC.k2_[_e53];
    let _e74 = (_e72.x & 15u);
    if ki {
        let _e75 = (_e74 == 0u);
        if _e75 {
            phi_730_ = _e72.y;
        } else {
            phi_730_ = _e72.x;
        }
        let _e78 = phi_730_;
        let _e80 = (_e78 >> bitcast<u32>(16i));
        let _e82 = j.T4_;
        if (_e80 == 0u) {
            phi_731_ = 0f;
        } else {
            phi_731_ = unpack2x16float(((_e80 + 1023u) * _e82)).x;
        }
        let _e89 = phi_731_;
        phi_732_ = _e89;
        if _e75 {
            phi_732_ = -(_e89);
        }
        let _e92 = phi_732_;
        Y3_ = _e92;
    }
    if mi {
        Q0_ = f32(((_e72.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e74 == 1u) {
        a1_ = unpack4x8unorm(_e72.y);
    } else {
        let _e98 = (_e53 * 8u);
        let _e101 = JB.k2_[_e98];
        let _e112 = JB.k2_[(_e98 + 1u)];
        let _e121 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e72.y));
        let _e123 = ((mat2x2<f32>(vec2<f32>(_e101.x, _e101.y), vec2<f32>(_e101.z, _e101.w)) * _e60) + _e112.xy);
        if (_e112.z > 0.9f) {
            phi_739_ = vec4<f32>(_e121.x, _e121.y, 2f, _e121.w);
        } else {
            phi_739_ = vec4<f32>(_e121.x, _e121.y, _e112.w, _e121.w);
        }
        let _e138 = phi_739_;
        if (f32(_e74) == 2f) {
            let _e164 = vec4<f32>(_e123.x, _e138.y, _e138.z, _e138.w);
            phi_740_ = vec4<f32>(_e164.x, 0f, _e164.z, _e164.w);
        } else {
            let _e146 = vec4<f32>(_e138.x, _e138.y, -(_e138.z), _e138.w);
            let _e152 = vec4<f32>(_e123.x, _e146.y, _e146.z, _e146.w);
            phi_740_ = vec4<f32>(_e152.x, _e123.y, _e152.z, _e152.w);
        }
        let _e171 = phi_740_;
        a1_ = _e171;
        let _e173 = a1_[3u];
        a1_[3u] = -(_e173);
    }
    phi_404_ = si;
    if si {
        phi_404_ = ((_e72.x & 2048u) != 0u);
    }
    let _e180 = phi_404_;
    if _e180 {
        let _e181 = (_e53 * 8u);
        let _e185 = JB.k2_[(_e181 + 4u)];
        let _e196 = JB.k2_[(_e181 + 5u)];
        let _e199 = ((mat2x2<f32>(vec2<f32>(_e185.x, _e185.y), vec2<f32>(_e185.z, _e185.w)) * _e60) + _e196.xy);
        phi_741_ = (1f + _e196.z);
        if ((_e72.x & 4096u) != 0u) {
            phi_741_ = (-1f - f32(((_e72.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e210 = phi_741_;
        r1_ = vec3<f32>(_e199.x, _e199.y, _e210);
    } else {
        r1_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e215 = j.bg;
    let _e217 = j.cg;
    let _e225 = vec4<f32>(((_e50.x * _e215) - 1f), ((_e50.y * _e217) - sign(_e217)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e225.x, _e225.y, ((f32(((_e58.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e225.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) MB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    MB_1 = MB;
    main_1();
    let _e12 = J2_;
    let _e13 = Y3_;
    let _e14 = Q0_;
    let _e15 = a1_;
    let _e16 = r1_;
    let _e17 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
