enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct AC {
    tc: f32,
    Dd: f32,
    Hf: f32,
    If: f32,
    q6_: u32,
    Qb: u32,
    tf: u32,
    uf: u32,
    X7_: vec4<i32>,
    eh: vec2<f32>,
    Ed: vec2<f32>,
    f2_: u32,
    ih: f32,
    f6_: u32,
    U2_: f32,
    Fd: f32,
    of_: u32,
    F3_: f32,
    G3_: f32,
    Gd: f32,
    bh: u32,
    Pb: u32,
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

@id(0) override Eh: bool = true;
@id(2) override Gh: bool = true;
@id(1) override Fh: bool = true;
@id(8) override Mh: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(2)
var OB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: AC;
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
var ga: sampler;

fn main_1() {
    var phi_840_: u32;
    var phi_841_: f32;
    var phi_842_: f32;
    var phi_851_: vec4<f32>;
    var phi_852_: vec4<f32>;
    var phi_488_: bool;

    let _e50 = JB_1;
    let _e52 = bitcast<u32>(_e50.z);
    let _e53 = (_e52 & 65535u);
    let _e55 = ((_e53 * 4u) + 2u);
    let _e62 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e55 & 255u)), bitcast<i32>((_e55 >> bitcast<u32>(8i)))), 0i);
    let _e64 = _e50.xy;
    let _e66 = bitcast<vec3<f32>>(_e62.yzw);
    let _e72 = j.eh;
    F2_ = (((_e64 * _e66.x) + _e66.yz) * _e72);
    let _e80 = textureLoad(CD, vec2<i32>(bitcast<i32>((_e52 & 255u)), bitcast<i32>((_e53 >> bitcast<u32>(8i)))), 0i);
    let _e82 = (_e80.x & 15u);
    if Eh {
        let _e83 = (_e82 == 0u);
        if _e83 {
            phi_840_ = _e80.y;
        } else {
            phi_840_ = _e80.x;
        }
        let _e86 = phi_840_;
        let _e88 = (_e86 >> bitcast<u32>(16i));
        let _e90 = j.f6_;
        if (_e88 == 0u) {
            phi_841_ = 0f;
        } else {
            phi_841_ = unpack2x16float(((_e88 + 1023u) * _e90)).x;
        }
        let _e97 = phi_841_;
        phi_842_ = _e97;
        if _e83 {
            phi_842_ = -(_e97);
        }
        let _e100 = phi_842_;
        O3_ = _e100;
    }
    if Gh {
        g1_ = f32(((_e80.x >> bitcast<u32>(4i)) & 15u));
    }
    if Fh {
        let _e105 = (_e53 * 8u);
        let _e106 = (_e105 + 2u);
        let _e113 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e106 & 255u)), bitcast<i32>((_e106 >> bitcast<u32>(8i)))), 0i);
        let _e121 = (_e105 + 3u);
        let _e128 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e121 & 255u)), bitcast<i32>((_e121 >> bitcast<u32>(8i)))), 0i);
        if any((_e113 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e143 = ((mat2x2<f32>(vec2<f32>(_e113.x, _e113.y), vec2<f32>(_e113.z, _e113.w)) * _e64) + _e128.xy);
            unnamed.gl_ClipDistance[0i] = (_e143.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e143.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e143.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e143.y);
        } else {
            let _e133 = (_e128.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e133;
            unnamed.gl_ClipDistance[2i] = _e133;
            unnamed.gl_ClipDistance[1i] = _e133;
            unnamed.gl_ClipDistance[0i] = _e133;
        }
    }
    if (_e82 == 1u) {
        X1_ = unpack4x8unorm(_e80.y);
    } else {
        let _e159 = (_e53 * 8u);
        let _e166 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e159 & 255u)), bitcast<i32>((_e159 >> bitcast<u32>(8i)))), 0i);
        let _e174 = (_e159 + 1u);
        let _e181 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e174 & 255u)), bitcast<i32>((_e174 >> bitcast<u32>(8i)))), 0i);
        let _e190 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e80.y));
        let _e192 = ((mat2x2<f32>(vec2<f32>(_e166.x, _e166.y), vec2<f32>(_e166.z, _e166.w)) * _e64) + _e181.xy);
        if (_e181.z > 0.9f) {
            phi_851_ = vec4<f32>(_e190.x, _e190.y, 2f, _e190.w);
        } else {
            phi_851_ = vec4<f32>(_e190.x, _e190.y, _e181.w, _e190.w);
        }
        let _e207 = phi_851_;
        if (f32(_e82) == 2f) {
            let _e233 = vec4<f32>(_e192.x, _e207.y, _e207.z, _e207.w);
            phi_852_ = vec4<f32>(_e233.x, 0f, _e233.z, _e233.w);
        } else {
            let _e215 = vec4<f32>(_e207.x, _e207.y, -(_e207.z), _e207.w);
            let _e221 = vec4<f32>(_e192.x, _e215.y, _e215.z, _e215.w);
            phi_852_ = vec4<f32>(_e221.x, _e192.y, _e221.z, _e221.w);
        }
        let _e240 = phi_852_;
        X1_ = _e240;
        let _e242 = X1_[3u];
        X1_[3u] = -(_e242);
    }
    phi_488_ = Mh;
    if Mh {
        phi_488_ = ((_e80.x & 2048u) != 0u);
    }
    let _e249 = phi_488_;
    if _e249 {
        let _e250 = (_e53 * 8u);
        let _e251 = (_e250 + 4u);
        let _e258 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e251 & 255u)), bitcast<i32>((_e251 >> bitcast<u32>(8i)))), 0i);
        let _e266 = (_e250 + 5u);
        let _e273 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e266 & 255u)), bitcast<i32>((_e266 >> bitcast<u32>(8i)))), 0i);
        let _e276 = ((mat2x2<f32>(vec2<f32>(_e258.x, _e258.y), vec2<f32>(_e258.z, _e258.w)) * _e64) + _e273.xy);
        C2_ = vec3<f32>(_e276.x, _e276.y, (1f + _e273.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e283 = j.Hf;
    let _e285 = j.If;
    let _e293 = vec4<f32>(((_e50.x * _e283) - 1f), ((_e50.y * _e285) - sign(_e285)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e293.x, _e293.y, (1f - (f32(_e62.x) * 0.000061035156f)), _e293.w);
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
