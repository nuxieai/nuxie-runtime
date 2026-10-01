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

@id(0) override ki: bool = true;
@id(2) override mi: bool = true;
@id(8) override si: bool = true;

@group(0) @binding(2)
var LB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> gl_VertexIndex_1: i32;
var<private> MB_1: vec3<f32>;
var<private> J2_: vec2<f32>;
@group(0) @binding(3)
var WC: texture_2d<u32>;
var<private> Y3_: f32;
var<private> Q0_: f32;
@group(0) @binding(4)
var JB: texture_2d<f32>;
var<private> a1_: vec4<f32>;
var<private> r1_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(5)
var ZC: texture_2d<u32>;
@group(3) @binding(9)
var wa: sampler;

fn main_1() {
    var phi_787_: u32;
    var phi_788_: f32;
    var phi_789_: f32;
    var phi_796_: vec4<f32>;
    var phi_797_: vec4<f32>;
    var phi_445_: bool;
    var phi_798_: f32;

    let _e51 = MB_1;
    let _e53 = bitcast<u32>(_e51.z);
    let _e54 = (_e53 & 65535u);
    let _e56 = ((_e54 * 4u) + 2u);
    let _e63 = textureLoad(LB, vec2<i32>(bitcast<i32>((_e56 & 255u)), bitcast<i32>((_e56 >> bitcast<u32>(8i)))), 0i);
    let _e65 = _e51.xy;
    let _e67 = bitcast<vec3<f32>>(_e63.yzw);
    let _e73 = j.Lh;
    J2_ = (((_e65 * _e67.x) + _e67.yz) * _e73);
    let _e81 = textureLoad(WC, vec2<i32>(bitcast<i32>((_e53 & 255u)), bitcast<i32>((_e54 >> bitcast<u32>(8i)))), 0i);
    let _e83 = (_e81.x & 15u);
    if ki {
        let _e84 = (_e83 == 0u);
        if _e84 {
            phi_787_ = _e81.y;
        } else {
            phi_787_ = _e81.x;
        }
        let _e87 = phi_787_;
        let _e89 = (_e87 >> bitcast<u32>(16i));
        let _e91 = j.T4_;
        if (_e89 == 0u) {
            phi_788_ = 0f;
        } else {
            phi_788_ = unpack2x16float(((_e89 + 1023u) * _e91)).x;
        }
        let _e98 = phi_788_;
        phi_789_ = _e98;
        if _e84 {
            phi_789_ = -(_e98);
        }
        let _e101 = phi_789_;
        Y3_ = _e101;
    }
    if mi {
        Q0_ = f32(((_e81.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e83 == 1u) {
        a1_ = unpack4x8unorm(_e81.y);
    } else {
        let _e107 = (_e54 * 8u);
        let _e114 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e107 & 255u)), bitcast<i32>((_e107 >> bitcast<u32>(8i)))), 0i);
        let _e122 = (_e107 + 1u);
        let _e129 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e122 & 255u)), bitcast<i32>((_e122 >> bitcast<u32>(8i)))), 0i);
        let _e138 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e81.y));
        let _e140 = ((mat2x2<f32>(vec2<f32>(_e114.x, _e114.y), vec2<f32>(_e114.z, _e114.w)) * _e65) + _e129.xy);
        if (_e129.z > 0.9f) {
            phi_796_ = vec4<f32>(_e138.x, _e138.y, 2f, _e138.w);
        } else {
            phi_796_ = vec4<f32>(_e138.x, _e138.y, _e129.w, _e138.w);
        }
        let _e155 = phi_796_;
        if (f32(_e83) == 2f) {
            let _e181 = vec4<f32>(_e140.x, _e155.y, _e155.z, _e155.w);
            phi_797_ = vec4<f32>(_e181.x, 0f, _e181.z, _e181.w);
        } else {
            let _e163 = vec4<f32>(_e155.x, _e155.y, -(_e155.z), _e155.w);
            let _e169 = vec4<f32>(_e140.x, _e163.y, _e163.z, _e163.w);
            phi_797_ = vec4<f32>(_e169.x, _e140.y, _e169.z, _e169.w);
        }
        let _e188 = phi_797_;
        a1_ = _e188;
        let _e190 = a1_[3u];
        a1_[3u] = -(_e190);
    }
    phi_445_ = si;
    if si {
        phi_445_ = ((_e81.x & 2048u) != 0u);
    }
    let _e197 = phi_445_;
    if _e197 {
        let _e198 = (_e54 * 8u);
        let _e199 = (_e198 + 4u);
        let _e206 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e199 & 255u)), bitcast<i32>((_e199 >> bitcast<u32>(8i)))), 0i);
        let _e214 = (_e198 + 5u);
        let _e221 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e214 & 255u)), bitcast<i32>((_e214 >> bitcast<u32>(8i)))), 0i);
        let _e224 = ((mat2x2<f32>(vec2<f32>(_e206.x, _e206.y), vec2<f32>(_e206.z, _e206.w)) * _e65) + _e221.xy);
        phi_798_ = (1f + _e221.z);
        if ((_e81.x & 4096u) != 0u) {
            phi_798_ = (-1f - f32(((_e81.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e235 = phi_798_;
        r1_ = vec3<f32>(_e224.x, _e224.y, _e235);
    } else {
        r1_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e240 = j.bg;
    let _e242 = j.cg;
    let _e250 = vec4<f32>(((_e51.x * _e240) - 1f), ((_e51.y * _e242) - sign(_e242)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e250.x, _e250.y, ((f32(((_e63.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e250.w);
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
