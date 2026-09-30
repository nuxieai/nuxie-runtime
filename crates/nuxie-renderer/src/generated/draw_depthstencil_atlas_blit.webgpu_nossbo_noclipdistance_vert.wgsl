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

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
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
var LB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> gl_VertexIndex_1: i32;
var<private> MB_1: vec3<f32>;
var<private> K2_: vec2<f32>;
@group(0) @binding(3)
var XC: texture_2d<u32>;
var<private> Z3_: f32;
var<private> Q0_: f32;
@group(0) @binding(4)
var JB: texture_2d<f32>;
var<private> a1_: vec4<f32>;
var<private> F1_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(0) @binding(5)
var AD: texture_2d<u32>;
@group(3) @binding(9)
var wa: sampler;

fn main_1() {
    var phi_765_: u32;
    var phi_766_: f32;
    var phi_767_: f32;
    var phi_774_: vec4<f32>;
    var phi_775_: vec4<f32>;
    var phi_445_: bool;

    let _e47 = MB_1;
    let _e49 = bitcast<u32>(_e47.z);
    let _e50 = (_e49 & 65535u);
    let _e52 = ((_e50 * 4u) + 2u);
    let _e59 = textureLoad(LB, vec2<i32>(bitcast<i32>((_e52 & 255u)), bitcast<i32>((_e52 >> bitcast<u32>(8i)))), 0i);
    let _e61 = _e47.xy;
    let _e63 = bitcast<vec3<f32>>(_e59.yzw);
    let _e69 = j.Ch;
    K2_ = (((_e61 * _e63.x) + _e63.yz) * _e69);
    let _e77 = textureLoad(XC, vec2<i32>(bitcast<i32>((_e49 & 255u)), bitcast<i32>((_e50 >> bitcast<u32>(8i)))), 0i);
    let _e79 = (_e77.x & 15u);
    if bi {
        let _e80 = (_e79 == 0u);
        if _e80 {
            phi_765_ = _e77.y;
        } else {
            phi_765_ = _e77.x;
        }
        let _e83 = phi_765_;
        let _e85 = (_e83 >> bitcast<u32>(16i));
        let _e87 = j.T4_;
        if (_e85 == 0u) {
            phi_766_ = 0f;
        } else {
            phi_766_ = unpack2x16float(((_e85 + 1023u) * _e87)).x;
        }
        let _e94 = phi_766_;
        phi_767_ = _e94;
        if _e80 {
            phi_767_ = -(_e94);
        }
        let _e97 = phi_767_;
        Z3_ = _e97;
    }
    if di {
        Q0_ = f32(((_e77.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e79 == 1u) {
        a1_ = unpack4x8unorm(_e77.y);
    } else {
        let _e103 = (_e50 * 8u);
        let _e110 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e103 & 255u)), bitcast<i32>((_e103 >> bitcast<u32>(8i)))), 0i);
        let _e118 = (_e103 + 1u);
        let _e125 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e118 & 255u)), bitcast<i32>((_e118 >> bitcast<u32>(8i)))), 0i);
        let _e134 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e77.y));
        let _e136 = ((mat2x2<f32>(vec2<f32>(_e110.x, _e110.y), vec2<f32>(_e110.z, _e110.w)) * _e61) + _e125.xy);
        if (_e125.z > 0.9f) {
            phi_774_ = vec4<f32>(_e134.x, _e134.y, 2f, _e134.w);
        } else {
            phi_774_ = vec4<f32>(_e134.x, _e134.y, _e125.w, _e134.w);
        }
        let _e151 = phi_774_;
        if (f32(_e79) == 2f) {
            let _e177 = vec4<f32>(_e136.x, _e151.y, _e151.z, _e151.w);
            phi_775_ = vec4<f32>(_e177.x, 0f, _e177.z, _e177.w);
        } else {
            let _e159 = vec4<f32>(_e151.x, _e151.y, -(_e151.z), _e151.w);
            let _e165 = vec4<f32>(_e136.x, _e159.y, _e159.z, _e159.w);
            phi_775_ = vec4<f32>(_e165.x, _e136.y, _e165.z, _e165.w);
        }
        let _e184 = phi_775_;
        a1_ = _e184;
        let _e186 = a1_[3u];
        a1_[3u] = -(_e186);
    }
    phi_445_ = ji;
    if ji {
        phi_445_ = ((_e77.x & 2048u) != 0u);
    }
    let _e193 = phi_445_;
    if _e193 {
        let _e194 = (_e50 * 8u);
        let _e195 = (_e194 + 4u);
        let _e202 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e195 & 255u)), bitcast<i32>((_e195 >> bitcast<u32>(8i)))), 0i);
        let _e210 = (_e194 + 5u);
        let _e217 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e210 & 255u)), bitcast<i32>((_e210 >> bitcast<u32>(8i)))), 0i);
        let _e220 = ((mat2x2<f32>(vec2<f32>(_e202.x, _e202.y), vec2<f32>(_e202.z, _e202.w)) * _e61) + _e217.xy);
        F1_ = vec3<f32>(_e220.x, _e220.y, (1f + _e217.z));
    } else {
        F1_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e227 = j.Yf;
    let _e229 = j.Zf;
    let _e237 = vec4<f32>(((_e47.x * _e227) - 1f), ((_e47.y * _e229) - sign(_e229)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e237.x, _e237.y, ((f32(((_e59.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e237.w);
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
