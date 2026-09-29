enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct BC {
    qc: f32,
    Ad: f32,
    Ef: f32,
    Ff: f32,
    q6_: u32,
    Nb: u32,
    qf: u32,
    rf: u32,
    V7_: vec4<i32>,
    bh: vec2<f32>,
    Bd: vec2<f32>,
    d2_: u32,
    fh: f32,
    e6_: u32,
    T2_: f32,
    Cd: f32,
    lf: u32,
    B3_: f32,
    C3_: f32,
    Dd: f32,
    Yg: u32,
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

@id(0) override Bh: bool = true;
@id(2) override Dh: bool = true;
@id(1) override Ch: bool = true;
@id(8) override Jh: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(2)
var PB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> n: BC;
var<private> gl_VertexIndex_1: i32;
var<private> KB_1: vec3<f32>;
var<private> F2_: vec2<f32>;
@group(0) @binding(3)
var DD: texture_2d<u32>;
var<private> K3_: f32;
var<private> g2_: f32;
@group(0) @binding(4)
var QB: texture_2d<f32>;
var<private> V1_: vec4<f32>;
var<private> C2_: vec3<f32>;
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(5)
var ID: texture_2d<u32>;
@group(3) @binding(9)
var ea: sampler;

fn main_1() {
    var phi_863_: u32;
    var phi_864_: f32;
    var phi_865_: f32;
    var phi_874_: vec4<f32>;
    var phi_875_: vec4<f32>;
    var phi_876_: vec4<f32>;
    var phi_504_: bool;

    let _e50 = KB_1;
    let _e52 = bitcast<u32>(_e50.z);
    let _e53 = (_e52 & 65535u);
    let _e55 = ((_e53 * 4u) + 2u);
    let _e62 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e55 & 255u)), bitcast<i32>((_e55 >> bitcast<u32>(8i)))), 0i);
    let _e64 = _e50.xy;
    let _e66 = bitcast<vec3<f32>>(_e62.yzw);
    let _e72 = n.bh;
    F2_ = (((_e64 * _e66.x) + _e66.yz) * _e72);
    let _e80 = textureLoad(DD, vec2<i32>(bitcast<i32>((_e52 & 255u)), bitcast<i32>((_e53 >> bitcast<u32>(8i)))), 0i);
    let _e82 = (_e80.x & 15u);
    if Bh {
        let _e83 = (_e82 == 0u);
        if _e83 {
            phi_863_ = _e80.y;
        } else {
            phi_863_ = _e80.x;
        }
        let _e86 = phi_863_;
        let _e88 = (_e86 >> bitcast<u32>(16i));
        let _e90 = n.e6_;
        if (_e88 == 0u) {
            phi_864_ = 0f;
        } else {
            phi_864_ = unpack2x16float(((_e88 + 1023u) * _e90)).x;
        }
        let _e97 = phi_864_;
        phi_865_ = _e97;
        if _e83 {
            phi_865_ = -(_e97);
        }
        let _e100 = phi_865_;
        K3_ = _e100;
    }
    if Dh {
        g2_ = f32(((_e80.x >> bitcast<u32>(4i)) & 15u));
    }
    if Ch {
        let _e105 = (_e53 * 8u);
        let _e106 = (_e105 + 2u);
        let _e113 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e106 & 255u)), bitcast<i32>((_e106 >> bitcast<u32>(8i)))), 0i);
        let _e121 = (_e105 + 3u);
        let _e128 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e121 & 255u)), bitcast<i32>((_e121 >> bitcast<u32>(8i)))), 0i);
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
        let _e245 = unpack4x8unorm(_e80.y);
        if Dh {
            phi_876_ = _e245;
        } else {
            let _e248 = (_e245.xyz * _e245.w);
            let _e254 = vec4<f32>(_e248.x, _e245.y, _e245.z, _e245.w);
            let _e260 = vec4<f32>(_e254.x, _e248.y, _e254.z, _e254.w);
            phi_876_ = vec4<f32>(_e260.x, _e260.y, _e248.z, _e260.w);
        }
        let _e268 = phi_876_;
        V1_ = _e268;
    } else {
        let _e159 = (_e53 * 8u);
        let _e166 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e159 & 255u)), bitcast<i32>((_e159 >> bitcast<u32>(8i)))), 0i);
        let _e174 = (_e159 + 1u);
        let _e181 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e174 & 255u)), bitcast<i32>((_e174 >> bitcast<u32>(8i)))), 0i);
        let _e190 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e80.y));
        let _e192 = ((mat2x2<f32>(vec2<f32>(_e166.x, _e166.y), vec2<f32>(_e166.z, _e166.w)) * _e64) + _e181.xy);
        if (_e181.z > 0.9f) {
            phi_874_ = vec4<f32>(_e190.x, _e190.y, 2f, _e190.w);
        } else {
            phi_874_ = vec4<f32>(_e190.x, _e190.y, _e181.w, _e190.w);
        }
        let _e207 = phi_874_;
        if (f32(_e82) == 2f) {
            let _e233 = vec4<f32>(_e192.x, _e207.y, _e207.z, _e207.w);
            phi_875_ = vec4<f32>(_e233.x, 0f, _e233.z, _e233.w);
        } else {
            let _e215 = vec4<f32>(_e207.x, _e207.y, -(_e207.z), _e207.w);
            let _e221 = vec4<f32>(_e192.x, _e215.y, _e215.z, _e215.w);
            phi_875_ = vec4<f32>(_e221.x, _e192.y, _e221.z, _e221.w);
        }
        let _e240 = phi_875_;
        V1_ = _e240;
        let _e242 = V1_[3u];
        V1_[3u] = -(_e242);
    }
    phi_504_ = Jh;
    if Jh {
        phi_504_ = ((_e80.x & 2048u) != 0u);
    }
    let _e272 = phi_504_;
    if _e272 {
        let _e273 = (_e53 * 8u);
        let _e274 = (_e273 + 4u);
        let _e281 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e274 & 255u)), bitcast<i32>((_e274 >> bitcast<u32>(8i)))), 0i);
        let _e289 = (_e273 + 5u);
        let _e296 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e289 & 255u)), bitcast<i32>((_e289 >> bitcast<u32>(8i)))), 0i);
        let _e299 = ((mat2x2<f32>(vec2<f32>(_e281.x, _e281.y), vec2<f32>(_e281.z, _e281.w)) * _e64) + _e296.xy);
        C2_ = vec3<f32>(_e299.x, _e299.y, (1f + _e296.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e306 = n.Ef;
    let _e308 = n.Ff;
    let _e316 = vec4<f32>(((_e50.x * _e306) - 1f), ((_e50.y * _e308) - sign(_e308)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e316.x, _e316.y, (1f - (f32(_e62.x) * 0.000061035156f)), _e316.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) KB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    KB_1 = KB;
    main_1();
    let _e13 = unnamed.gl_Position;
    let _e14 = unnamed.gl_ClipDistance;
    let _e15 = F2_;
    let _e16 = K3_;
    let _e17 = g2_;
    let _e18 = V1_;
    let _e19 = C2_;
    return VertexOutput(_e13, _e14, _e15, _e16, _e17, _e18, _e19);
}
