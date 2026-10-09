enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct VB {
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
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

@id(0) override Wi: bool = true;
@id(2) override Yi: bool = true;
@id(1) override Xi: bool = true;
@id(8) override ej: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(2)
var KB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> S2_: vec2<f32>;
@group(0) @binding(3)
var VC: texture_2d<u32>;
var<private> f4_: f32;
var<private> P0_: f32;
@group(0) @binding(4)
var JB: texture_2d<f32>;
var<private> O0_: vec4<f32>;
var<private> U0_: vec3<f32>;
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(5)
var BD: texture_2d<u32>;
@group(3) @binding(9)
var ab: sampler;

fn main_1() {
    var phi_899_: u32;
    var phi_900_: f32;
    var phi_901_: f32;
    var phi_910_: f32;
    var phi_508_: bool;
    var phi_912_: f32;

    let _e56 = LB_1;
    let _e58 = bitcast<u32>(_e56.z);
    let _e59 = (_e58 & 65535u);
    let _e61 = ((_e59 * 4u) + 2u);
    let _e68 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e61 & 255u)), bitcast<i32>((_e61 >> bitcast<u32>(8i)))), 0i);
    let _e70 = _e56.xy;
    let _e72 = bitcast<vec3<f32>>(_e68.yzw);
    let _e78 = j.yi;
    S2_ = (((_e70 * _e72.x) + _e72.yz) * _e78);
    let _e86 = textureLoad(VC, vec2<i32>(bitcast<i32>((_e58 & 255u)), bitcast<i32>((_e59 >> bitcast<u32>(8i)))), 0i);
    let _e88 = (_e86.x & 15u);
    if Wi {
        let _e89 = (_e88 == 0u);
        if _e89 {
            phi_899_ = _e86.y;
        } else {
            phi_899_ = _e86.x;
        }
        let _e92 = phi_899_;
        let _e94 = (_e92 >> bitcast<u32>(16i));
        let _e96 = j.w6_;
        if (_e94 == 0u) {
            phi_900_ = 0f;
        } else {
            phi_900_ = unpack2x16float(((_e94 + 1023u) * _e96)).x;
        }
        let _e103 = phi_900_;
        phi_901_ = _e103;
        if _e89 {
            phi_901_ = -(_e103);
        }
        let _e106 = phi_901_;
        f4_ = _e106;
    }
    if Yi {
        P0_ = f32(((_e86.x >> bitcast<u32>(4i)) & 15u));
    }
    if Xi {
        let _e111 = (_e59 * 8u);
        let _e112 = (_e111 + 2u);
        let _e119 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e112 & 255u)), bitcast<i32>((_e112 >> bitcast<u32>(8i)))), 0i);
        let _e127 = (_e111 + 3u);
        let _e134 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e127 & 255u)), bitcast<i32>((_e127 >> bitcast<u32>(8i)))), 0i);
        if any((_e119 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e149 = ((mat2x2<f32>(vec2<f32>(_e119.x, _e119.y), vec2<f32>(_e119.z, _e119.w)) * _e70) + _e134.xy);
            unnamed.gl_ClipDistance[0i] = (_e149.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e149.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e149.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e149.y);
        } else {
            let _e139 = (_e134.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e139;
            unnamed.gl_ClipDistance[2i] = _e139;
            unnamed.gl_ClipDistance[1i] = _e139;
            unnamed.gl_ClipDistance[0i] = _e139;
        }
    }
    if (_e88 == 1u) {
        O0_ = unpack4x8unorm(_e86.y);
    } else {
        let _e165 = (_e59 * 8u);
        let _e172 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e165 & 255u)), bitcast<i32>((_e165 >> bitcast<u32>(8i)))), 0i);
        let _e180 = (_e165 + 1u);
        let _e187 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e180 & 255u)), bitcast<i32>((_e180 >> bitcast<u32>(8i)))), 0i);
        let _e192 = ((mat2x2<f32>(vec2<f32>(_e172.x, _e172.y), vec2<f32>(_e172.z, _e172.w)) * _e70) + _e187.xy);
        let _e203 = ((_e187.w + (f32(_e88) * 0.125f)) + (max(_e187.z, 0f) * 0.00024414063f));
        if (_e187.z < 0f) {
            phi_910_ = -(_e203);
        } else {
            phi_910_ = _e203;
        }
        let _e206 = phi_910_;
        O0_ = vec4<f32>(_e192.x, _e192.y, _e206, (-0.75f - round((bitcast<f32>(_e86.y) * 255f))));
    }
    phi_508_ = ej;
    if ej {
        phi_508_ = ((_e86.x & 2048u) != 0u);
    }
    let _e216 = phi_508_;
    if _e216 {
        let _e217 = (_e59 * 8u);
        let _e218 = (_e217 + 4u);
        let _e225 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e218 & 255u)), bitcast<i32>((_e218 >> bitcast<u32>(8i)))), 0i);
        let _e233 = (_e217 + 5u);
        let _e240 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e233 & 255u)), bitcast<i32>((_e233 >> bitcast<u32>(8i)))), 0i);
        let _e243 = ((mat2x2<f32>(vec2<f32>(_e225.x, _e225.y), vec2<f32>(_e225.z, _e225.w)) * _e70) + _e240.xy);
        phi_912_ = (1f + _e240.z);
        if ((_e86.x & 4096u) != 0u) {
            phi_912_ = (-1f - f32(((_e86.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e254 = phi_912_;
        U0_ = vec3<f32>(_e243.x, _e243.y, _e254);
    } else {
        U0_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e259 = j.Hg;
    let _e261 = j.Ig;
    let _e269 = vec4<f32>(((_e56.x * _e259) - 1f), ((_e56.y * _e261) - sign(_e261)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e269.x, _e269.y, ((f32(((_e68.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e269.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e13 = unnamed.gl_Position;
    let _e14 = unnamed.gl_ClipDistance;
    let _e15 = S2_;
    let _e16 = f4_;
    let _e17 = P0_;
    let _e18 = O0_;
    let _e19 = U0_;
    return VertexOutput(_e13, _e14, _e15, _e16, _e17, _e18, _e19);
}
