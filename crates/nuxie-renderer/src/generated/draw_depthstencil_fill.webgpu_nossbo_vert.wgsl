enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
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

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(4) @interpolate(flat, either) member: vec2<f32>,
    @location(6) @interpolate(flat, either) member_1: f32,
    @location(0) member_2: vec4<f32>,
    @location(9) member_3: vec3<f32>,
}

@id(0) override ki: bool = true;
@id(2) override mi: bool = true;
@id(1) override li: bool = true;
@id(8) override si: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
var<private> gl_VertexIndex_1: i32;
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(5)
var ZC: texture_2d<u32>;
@group(0) @binding(2)
var LB: texture_2d<u32>;
@group(0) @binding(3)
var WC: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> l1_: vec2<f32>;
var<private> Q0_: f32;
@group(0) @binding(4)
var JB: texture_2d<f32>;
var<private> a1_: vec4<f32>;
var<private> r1_: vec3<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var wa: sampler;

fn main_1() {
    var phi_1031_: i32;
    var phi_1033_: vec4<u32>;
    var phi_1034_: vec4<u32>;
    var phi_1036_: vec2<f32>;
    var phi_1037_: u32;
    var phi_1038_: f32;
    var phi_1039_: f32;
    var phi_1054_: f32;
    var phi_1052_: vec4<f32>;
    var phi_1053_: vec4<f32>;
    var phi_657_: bool;

    let _e58 = gl_VertexIndex_1;
    let _e60 = ((_e58 & 1073741824i) != 0i);
    let _e63 = (_e58 & 536870911i);
    let _e64 = select(4i, 5i, _e60);
    let _e70 = (_e63 & ((1i << bitcast<u32>(_e64)) - 1i));
    let _e71 = select(8i, 17i, _e60);
    let _e72 = !(_e60);
    let _e74 = (_e72 && (_e70 == 9i));
    let _e75 = select(_e70, 0i, _e74);
    let _e77 = min(_e75, (_e71 - 1i));
    let _e79 = (((_e63 >> bitcast<u32>(_e64)) * _e71) + _e77);
    let _e84 = textureLoad(TB, vec2<i32>((_e79 & 2047i), (_e79 >> bitcast<u32>(11i))), 0i);
    let _e88 = (max((_e84.w & 65535u), 1u) - 1u);
    let _e95 = textureLoad(ZC, vec2<i32>(bitcast<i32>((_e88 & 255u)), bitcast<i32>((_e88 >> bitcast<u32>(8i)))), 0i);
    let _e99 = (_e95.z & 65535u);
    let _e101 = (_e99 * 4u);
    let _e108 = textureLoad(LB, vec2<i32>(bitcast<i32>((_e101 & 255u)), bitcast<i32>((_e101 >> bitcast<u32>(8i)))), 0i);
    let _e109 = bitcast<vec4<f32>>(_e108);
    let _e117 = (_e101 + 1u);
    let _e124 = textureLoad(LB, vec2<i32>(bitcast<i32>((_e117 & 255u)), bitcast<i32>((_e117 >> bitcast<u32>(8i)))), 0i);
    phi_1031_ = _e75;
    if ((((_e84.w & 8388608u) != 0u) && _e72) && !(_e74)) {
        phi_1031_ = (_e75 - 1i);
    }
    let _e134 = phi_1031_;
    phi_1034_ = _e84;
    if (_e134 != _e77) {
        let _e137 = ((_e79 + _e134) - _e77);
        let _e142 = textureLoad(TB, vec2<i32>((_e137 & 2047i), (_e137 >> bitcast<u32>(11i))), 0i);
        if ((_e142.w & 8454143u) != (_e84.w & 8454143u)) {
            let _e147 = bitcast<i32>(_e95.w);
            let _e152 = textureLoad(TB, vec2<i32>((_e147 & 2047i), (_e147 >> bitcast<u32>(11i))), 0i);
            phi_1033_ = _e152;
        } else {
            phi_1033_ = _e142;
        }
        let _e154 = phi_1033_;
        phi_1034_ = _e154;
    }
    let _e156 = phi_1034_;
    if _e74 {
        phi_1036_ = bitcast<vec2<f32>>(_e95.xy);
    } else {
        phi_1036_ = bitcast<vec2<f32>>(_e156.xy);
    }
    let _e160 = phi_1036_;
    let _e162 = ((mat2x2<f32>(vec2<f32>(_e109.x, _e109.y), vec2<f32>(_e109.z, _e109.w)) * _e160) + bitcast<vec2<f32>>(_e124.xy));
    let _e169 = textureLoad(WC, vec2<i32>(bitcast<i32>((_e95.z & 255u)), bitcast<i32>((_e99 >> bitcast<u32>(8i)))), 0i);
    let _e171 = (_e169.x & 15u);
    if ki {
        let _e172 = (_e171 == 0u);
        if _e172 {
            phi_1037_ = _e169.y;
        } else {
            phi_1037_ = _e169.x;
        }
        let _e175 = phi_1037_;
        let _e177 = (_e175 >> bitcast<u32>(16i));
        let _e179 = j.T4_;
        if (_e177 == 0u) {
            phi_1038_ = 0f;
        } else {
            phi_1038_ = unpack2x16float(((_e177 + 1023u) * _e179)).x;
        }
        let _e186 = phi_1038_;
        phi_1039_ = _e186;
        if _e172 {
            phi_1039_ = -(_e186);
        }
        let _e189 = phi_1039_;
        l1_[0u] = _e189;
    }
    if mi {
        Q0_ = f32(((_e169.x >> bitcast<u32>(4i)) & 15u));
    }
    if li {
        let _e195 = (_e99 * 8u);
        let _e196 = (_e195 + 2u);
        let _e203 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e196 & 255u)), bitcast<i32>((_e196 >> bitcast<u32>(8i)))), 0i);
        let _e211 = (_e195 + 3u);
        let _e218 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e211 & 255u)), bitcast<i32>((_e211 >> bitcast<u32>(8i)))), 0i);
        if any((_e203 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e223 = ((mat2x2<f32>(vec2<f32>(_e203.x, _e203.y), vec2<f32>(_e203.z, _e203.w)) * _e162) + _e218.xy);
            unnamed.gl_ClipDistance[0i] = (_e223.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e223.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e223.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e223.y);
        } else {
            let _e239 = (_e218.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e239;
            unnamed.gl_ClipDistance[2i] = _e239;
            unnamed.gl_ClipDistance[1i] = _e239;
            unnamed.gl_ClipDistance[0i] = _e239;
        }
    }
    if (_e171 == 1u) {
        a1_ = unpack4x8unorm(_e169.y);
    } else {
        if (ki && (_e171 == 0u)) {
            let _e254 = (_e169.x >> bitcast<u32>(16i));
            let _e256 = j.T4_;
            if (_e254 == 0u) {
                phi_1054_ = 0f;
            } else {
                phi_1054_ = unpack2x16float(((_e254 + 1023u) * _e256)).x;
            }
            let _e263 = phi_1054_;
            l1_[1u] = _e263;
        } else {
            let _e265 = (_e99 * 8u);
            let _e272 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e265 & 255u)), bitcast<i32>((_e265 >> bitcast<u32>(8i)))), 0i);
            let _e280 = (_e265 + 1u);
            let _e287 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e280 & 255u)), bitcast<i32>((_e280 >> bitcast<u32>(8i)))), 0i);
            let _e296 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e169.y));
            let _e298 = ((mat2x2<f32>(vec2<f32>(_e272.x, _e272.y), vec2<f32>(_e272.z, _e272.w)) * _e162) + _e287.xy);
            if (_e287.z > 0.9f) {
                phi_1052_ = vec4<f32>(_e296.x, _e296.y, 2f, _e296.w);
            } else {
                phi_1052_ = vec4<f32>(_e296.x, _e296.y, _e287.w, _e296.w);
            }
            let _e313 = phi_1052_;
            if (f32(_e171) == 2f) {
                let _e320 = vec4<f32>(_e298.x, _e313.y, _e313.z, _e313.w);
                phi_1053_ = vec4<f32>(_e320.x, 0f, _e320.z, _e320.w);
            } else {
                let _e332 = vec4<f32>(_e313.x, _e313.y, -(_e313.z), _e313.w);
                let _e338 = vec4<f32>(_e298.x, _e332.y, _e332.z, _e332.w);
                phi_1053_ = vec4<f32>(_e338.x, _e298.y, _e338.z, _e338.w);
            }
            let _e346 = phi_1053_;
            a1_ = _e346;
            let _e348 = a1_[3u];
            a1_[3u] = -(_e348);
        }
    }
    if ((_e58 & 536870912i) != 0i) {
        a1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    phi_657_ = si;
    if si {
        phi_657_ = ((_e169.x & 2048u) != 0u);
    }
    let _e353 = phi_657_;
    if _e353 {
        let _e354 = (_e99 * 8u);
        let _e355 = (_e354 + 4u);
        let _e362 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e355 & 255u)), bitcast<i32>((_e355 >> bitcast<u32>(8i)))), 0i);
        let _e370 = (_e354 + 5u);
        let _e377 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e370 & 255u)), bitcast<i32>((_e370 >> bitcast<u32>(8i)))), 0i);
        let _e380 = ((mat2x2<f32>(vec2<f32>(_e362.x, _e362.y), vec2<f32>(_e362.z, _e362.w)) * _e162) + _e377.xy);
        r1_ = vec3<f32>(_e380.x, _e380.y, (1f + _e377.z));
    } else {
        r1_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e387 = j.bg;
    let _e389 = j.cg;
    let _e397 = vec4<f32>(((_e162.x * _e387) - 1f), ((_e162.y * _e389) - sign(_e389)), 0f, 1f);
    let _e398 = (_e101 + 2u);
    let _e405 = textureLoad(LB, vec2<i32>(bitcast<i32>((_e398 & 255u)), bitcast<i32>((_e398 >> bitcast<u32>(8i)))), 0i);
    unnamed.gl_Position = vec4<f32>(_e397.x, _e397.y, ((f32(((_e405.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e397.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e10 = unnamed.gl_Position;
    let _e11 = unnamed.gl_ClipDistance;
    let _e12 = l1_;
    let _e13 = Q0_;
    let _e14 = a1_;
    let _e15 = r1_;
    return VertexOutput(_e10, _e11, _e12, _e13, _e14, _e15);
}
