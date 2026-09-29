enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct BC {
    jc: f32,
    sd: f32,
    of_: f32,
    pf: f32,
    p6_: u32,
    Pg: u32,
    Ze: u32,
    af: u32,
    U7_: vec4<i32>,
    Lg: vec2<f32>,
    td: vec2<f32>,
    c2_: u32,
    Qg: f32,
    d6_: u32,
    R2_: f32,
    ud: f32,
    Ue: u32,
    A3_: f32,
    B3_: f32,
    vd: f32,
    Ig: u32,
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

@id(0) override jh: bool = true;
@id(2) override lh: bool = true;
@id(1) override kh: bool = true;
@id(8) override rh: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(2)
var PB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> m: BC;
var<private> gl_VertexIndex_1: i32;
var<private> KB_1: vec3<f32>;
var<private> D2_: vec2<f32>;
@group(0) @binding(3)
var AD: texture_2d<u32>;
var<private> J3_: f32;
var<private> f2_: f32;
@group(0) @binding(4)
var QB: texture_2d<f32>;
var<private> f1_: vec4<f32>;
var<private> A2_: vec3<f32>;
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(5)
var ED: texture_2d<u32>;
@group(3) @binding(9)
var Z9_: sampler;

fn main_1() {
    var phi_773_: u32;
    var phi_774_: f32;
    var phi_775_: f32;
    var phi_776_: vec4<f32>;
    var phi_473_: bool;

    let _e49 = KB_1;
    let _e51 = bitcast<u32>(_e49.z);
    let _e52 = (_e51 & 65535u);
    let _e54 = ((_e52 * 4u) + 2u);
    let _e61 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e54 & 255u)), bitcast<i32>((_e54 >> bitcast<u32>(8i)))), 0i);
    let _e63 = _e49.xy;
    let _e65 = bitcast<vec3<f32>>(_e61.yzw);
    let _e71 = m.Lg;
    D2_ = (((_e63 * _e65.x) + _e65.yz) * _e71);
    let _e79 = textureLoad(AD, vec2<i32>(bitcast<i32>((_e51 & 255u)), bitcast<i32>((_e52 >> bitcast<u32>(8i)))), 0i);
    let _e81 = (_e79.x & 15u);
    if jh {
        let _e82 = (_e81 == 0u);
        if _e82 {
            phi_773_ = _e79.y;
        } else {
            phi_773_ = _e79.x;
        }
        let _e85 = phi_773_;
        let _e87 = (_e85 >> bitcast<u32>(16i));
        let _e89 = m.d6_;
        if (_e87 == 0u) {
            phi_774_ = 0f;
        } else {
            phi_774_ = unpack2x16float(((_e87 + 1023u) * _e89)).x;
        }
        let _e96 = phi_774_;
        phi_775_ = _e96;
        if _e82 {
            phi_775_ = -(_e96);
        }
        let _e99 = phi_775_;
        J3_ = _e99;
    }
    if lh {
        f2_ = f32(((_e79.x >> bitcast<u32>(4i)) & 15u));
    }
    if kh {
        let _e104 = (_e52 * 8u);
        let _e105 = (_e104 + 2u);
        let _e112 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e105 & 255u)), bitcast<i32>((_e105 >> bitcast<u32>(8i)))), 0i);
        let _e120 = (_e104 + 3u);
        let _e127 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e120 & 255u)), bitcast<i32>((_e120 >> bitcast<u32>(8i)))), 0i);
        if any((_e112 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e142 = ((mat2x2<f32>(vec2<f32>(_e112.x, _e112.y), vec2<f32>(_e112.z, _e112.w)) * _e63) + _e127.xy);
            unnamed.gl_ClipDistance[0i] = (_e142.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e142.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e142.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e142.y);
        } else {
            let _e132 = (_e127.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e132;
            unnamed.gl_ClipDistance[2i] = _e132;
            unnamed.gl_ClipDistance[1i] = _e132;
            unnamed.gl_ClipDistance[0i] = _e132;
        }
    }
    if (_e81 == 1u) {
        let _e205 = unpack4x8unorm(_e79.y);
        if lh {
            phi_776_ = _e205;
        } else {
            let _e208 = (_e205.xyz * _e205.w);
            let _e214 = vec4<f32>(_e208.x, _e205.y, _e205.z, _e205.w);
            let _e220 = vec4<f32>(_e214.x, _e208.y, _e214.z, _e214.w);
            phi_776_ = vec4<f32>(_e220.x, _e220.y, _e208.z, _e220.w);
        }
        let _e228 = phi_776_;
        f1_ = _e228;
    } else {
        let _e158 = (_e52 * 8u);
        let _e165 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e158 & 255u)), bitcast<i32>((_e158 >> bitcast<u32>(8i)))), 0i);
        let _e173 = (_e158 + 1u);
        let _e180 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e173 & 255u)), bitcast<i32>((_e173 >> bitcast<u32>(8i)))), 0i);
        let _e183 = ((mat2x2<f32>(vec2<f32>(_e165.x, _e165.y), vec2<f32>(_e165.z, _e165.w)) * _e63) + _e180.xy);
        f1_[3u] = -(bitcast<f32>(_e79.y));
        if (_e180.z > 0.9f) {
            f1_[2u] = 2f;
        } else {
            f1_[2u] = _e180.w;
        }
        if (_e81 == 2u) {
            f1_[1u] = 0f;
            f1_[0u] = _e183.x;
        } else {
            let _e195 = f1_[2u];
            f1_[2u] = -(_e195);
            f1_[0u] = _e183.x;
            f1_[1u] = _e183.y;
        }
    }
    phi_473_ = rh;
    if rh {
        phi_473_ = ((_e79.x & 2048u) != 0u);
    }
    let _e232 = phi_473_;
    if _e232 {
        let _e233 = (_e52 * 8u);
        let _e234 = (_e233 + 4u);
        let _e241 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e234 & 255u)), bitcast<i32>((_e234 >> bitcast<u32>(8i)))), 0i);
        let _e249 = (_e233 + 5u);
        let _e256 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e249 & 255u)), bitcast<i32>((_e249 >> bitcast<u32>(8i)))), 0i);
        let _e259 = ((mat2x2<f32>(vec2<f32>(_e241.x, _e241.y), vec2<f32>(_e241.z, _e241.w)) * _e63) + _e256.xy);
        A2_ = vec3<f32>(_e259.x, _e259.y, (1f + _e256.z));
    } else {
        A2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e266 = m.of_;
    let _e268 = m.pf;
    let _e276 = vec4<f32>(((_e49.x * _e266) - 1f), ((_e49.y * _e268) - sign(_e268)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e276.x, _e276.y, (1f - (f32(_e61.x) * 0.000061035156f)), _e276.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) KB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    KB_1 = KB;
    main_1();
    let _e13 = unnamed.gl_Position;
    let _e14 = unnamed.gl_ClipDistance;
    let _e15 = D2_;
    let _e16 = J3_;
    let _e17 = f2_;
    let _e18 = f1_;
    let _e19 = A2_;
    return VertexOutput(_e13, _e14, _e15, _e16, _e17, _e18, _e19);
}
