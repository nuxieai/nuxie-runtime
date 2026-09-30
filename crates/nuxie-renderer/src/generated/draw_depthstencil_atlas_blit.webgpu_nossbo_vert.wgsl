enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct TB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    hh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    eh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
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

@id(0) override Hh: bool = true;
@id(2) override Jh: bool = true;
@id(1) override Ih: bool = true;
@id(8) override Ph: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(2)
var OB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> gl_VertexIndex_1: i32;
var<private> JB_1: vec3<f32>;
var<private> F2_: vec2<f32>;
@group(0) @binding(3)
var CD: texture_2d<u32>;
var<private> O3_: f32;
var<private> g1_: f32;
@group(0) @binding(4)
var PB: texture_2d<f32>;
var<private> X1_: vec4<f32>;
var<private> C2_: vec3<f32>;
@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(5)
var HD: texture_2d<u32>;
@group(3) @binding(9)
var ca: sampler;

fn main_1() {
    var phi_856_: u32;
    var phi_857_: f32;
    var phi_858_: f32;
    var phi_867_: vec4<f32>;
    var phi_868_: vec4<f32>;
    var phi_496_: bool;

    let _e51 = JB_1;
    let _e53 = bitcast<u32>(_e51.z);
    let _e54 = (_e53 & 65535u);
    let _e56 = ((_e54 * 4u) + 2u);
    let _e63 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e56 & 255u)), bitcast<i32>((_e56 >> bitcast<u32>(8i)))), 0i);
    let _e65 = _e51.xy;
    let _e67 = bitcast<vec3<f32>>(_e63.yzw);
    let _e73 = j.hh;
    F2_ = (((_e65 * _e67.x) + _e67.yz) * _e73);
    let _e81 = textureLoad(CD, vec2<i32>(bitcast<i32>((_e53 & 255u)), bitcast<i32>((_e54 >> bitcast<u32>(8i)))), 0i);
    let _e83 = (_e81.x & 15u);
    if Hh {
        let _e84 = (_e83 == 0u);
        if _e84 {
            phi_856_ = _e81.y;
        } else {
            phi_856_ = _e81.x;
        }
        let _e87 = phi_856_;
        let _e89 = (_e87 >> bitcast<u32>(16i));
        let _e91 = j.c6_;
        if (_e89 == 0u) {
            phi_857_ = 0f;
        } else {
            phi_857_ = unpack2x16float(((_e89 + 1023u) * _e91)).x;
        }
        let _e98 = phi_857_;
        phi_858_ = _e98;
        if _e84 {
            phi_858_ = -(_e98);
        }
        let _e101 = phi_858_;
        O3_ = _e101;
    }
    if Jh {
        g1_ = f32(((_e81.x >> bitcast<u32>(4i)) & 15u));
    }
    if Ih {
        let _e106 = (_e54 * 8u);
        let _e107 = (_e106 + 2u);
        let _e114 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e107 & 255u)), bitcast<i32>((_e107 >> bitcast<u32>(8i)))), 0i);
        let _e122 = (_e106 + 3u);
        let _e129 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e122 & 255u)), bitcast<i32>((_e122 >> bitcast<u32>(8i)))), 0i);
        if any((_e114 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e144 = ((mat2x2<f32>(vec2<f32>(_e114.x, _e114.y), vec2<f32>(_e114.z, _e114.w)) * _e65) + _e129.xy);
            unnamed.gl_ClipDistance[0i] = (_e144.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e144.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e144.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e144.y);
        } else {
            let _e134 = (_e129.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e134;
            unnamed.gl_ClipDistance[2i] = _e134;
            unnamed.gl_ClipDistance[1i] = _e134;
            unnamed.gl_ClipDistance[0i] = _e134;
        }
    }
    if (_e83 == 1u) {
        X1_ = unpack4x8unorm(_e81.y);
    } else {
        let _e160 = (_e54 * 8u);
        let _e167 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e160 & 255u)), bitcast<i32>((_e160 >> bitcast<u32>(8i)))), 0i);
        let _e175 = (_e160 + 1u);
        let _e182 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e175 & 255u)), bitcast<i32>((_e175 >> bitcast<u32>(8i)))), 0i);
        let _e191 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e81.y));
        let _e193 = ((mat2x2<f32>(vec2<f32>(_e167.x, _e167.y), vec2<f32>(_e167.z, _e167.w)) * _e65) + _e182.xy);
        if (_e182.z > 0.9f) {
            phi_867_ = vec4<f32>(_e191.x, _e191.y, 2f, _e191.w);
        } else {
            phi_867_ = vec4<f32>(_e191.x, _e191.y, _e182.w, _e191.w);
        }
        let _e208 = phi_867_;
        if (f32(_e83) == 2f) {
            let _e234 = vec4<f32>(_e193.x, _e208.y, _e208.z, _e208.w);
            phi_868_ = vec4<f32>(_e234.x, 0f, _e234.z, _e234.w);
        } else {
            let _e216 = vec4<f32>(_e208.x, _e208.y, -(_e208.z), _e208.w);
            let _e222 = vec4<f32>(_e193.x, _e216.y, _e216.z, _e216.w);
            phi_868_ = vec4<f32>(_e222.x, _e193.y, _e222.z, _e222.w);
        }
        let _e241 = phi_868_;
        X1_ = _e241;
        let _e243 = X1_[3u];
        X1_[3u] = -(_e243);
    }
    phi_496_ = Ph;
    if Ph {
        phi_496_ = ((_e81.x & 2048u) != 0u);
    }
    let _e250 = phi_496_;
    if _e250 {
        let _e251 = (_e54 * 8u);
        let _e252 = (_e251 + 4u);
        let _e259 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e252 & 255u)), bitcast<i32>((_e252 >> bitcast<u32>(8i)))), 0i);
        let _e267 = (_e251 + 5u);
        let _e274 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e267 & 255u)), bitcast<i32>((_e267 >> bitcast<u32>(8i)))), 0i);
        let _e277 = ((mat2x2<f32>(vec2<f32>(_e259.x, _e259.y), vec2<f32>(_e259.z, _e259.w)) * _e65) + _e274.xy);
        C2_ = vec3<f32>(_e277.x, _e277.y, (1f + _e274.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e284 = j.Hf;
    let _e286 = j.If;
    let _e294 = vec4<f32>(((_e51.x * _e284) - 1f), ((_e51.y * _e286) - sign(_e286)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e294.x, _e294.y, ((f32(((_e63.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e294.w);
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
