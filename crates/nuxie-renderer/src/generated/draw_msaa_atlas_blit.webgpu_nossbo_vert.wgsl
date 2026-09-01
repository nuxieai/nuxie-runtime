enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct DC {
    gc: f32,
    qd: f32,
    jf: f32,
    kf: f32,
    o6_: u32,
    Lg: u32,
    Ue: u32,
    Ve: u32,
    T7_: vec4<i32>,
    Hg: vec2<f32>,
    rd: vec2<f32>,
    a2_: u32,
    Mg: f32,
    c6_: u32,
    R2_: f32,
    sd: f32,
    Pe: u32,
    B3_: f32,
    C3_: f32,
    td: f32,
    Eg: u32,
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

@id(0) override eh: bool = true;
@id(2) override gh: bool = true;
@id(1) override fh: bool = true;
@id(8) override mh: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(2)
var QB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> D2_: vec2<f32>;
@group(0) @binding(3)
var BD: texture_2d<u32>;
var<private> K3_: f32;
var<private> e2_: f32;
@group(0) @binding(4)
var RB: texture_2d<f32>;
var<private> f1_: vec4<f32>;
var<private> A2_: vec3<f32>;
@group(0) @binding(7)
var MC: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(5)
var FD: texture_2d<u32>;
@group(3) @binding(9)
var aa: sampler;

fn main_1() {
    var phi_780_: u32;
    var phi_781_: f32;
    var phi_782_: f32;
    var phi_783_: vec4<f32>;
    var phi_480_: bool;

    let _e49 = LB_1;
    let _e51 = bitcast<u32>(_e49.z);
    let _e52 = (_e51 & 65535u);
    let _e54 = ((_e52 * 4u) + 2u);
    let _e61 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e54 & 255u)), bitcast<i32>((_e54 >> bitcast<u32>(8i)))), 0i);
    let _e63 = _e49.xy;
    let _e65 = bitcast<vec3<f32>>(_e61.yzw);
    let _e71 = m.Hg;
    D2_ = (((_e63 * _e65.x) + _e65.yz) * _e71);
    let _e79 = textureLoad(BD, vec2<i32>(bitcast<i32>((_e51 & 255u)), bitcast<i32>((_e52 >> bitcast<u32>(8i)))), 0i);
    let _e81 = (_e79.x & 15u);
    if eh {
        let _e82 = (_e81 == 0u);
        if _e82 {
            phi_780_ = _e79.y;
        } else {
            phi_780_ = _e79.x;
        }
        let _e85 = phi_780_;
        let _e87 = (_e85 >> bitcast<u32>(16i));
        let _e89 = m.c6_;
        if (_e87 == 0u) {
            phi_781_ = 0f;
        } else {
            phi_781_ = unpack2x16float(((_e87 + 1023u) * _e89)).x;
        }
        let _e96 = phi_781_;
        phi_782_ = _e96;
        if _e82 {
            phi_782_ = -(_e96);
        }
        let _e99 = phi_782_;
        K3_ = _e99;
    }
    if gh {
        e2_ = f32(((_e79.x >> bitcast<u32>(4i)) & 15u));
    }
    if fh {
        let _e104 = (_e52 * 8u);
        let _e105 = (_e104 + 2u);
        let _e112 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e105 & 255u)), bitcast<i32>((_e105 >> bitcast<u32>(8i)))), 0i);
        let _e120 = (_e104 + 3u);
        let _e127 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e120 & 255u)), bitcast<i32>((_e120 >> bitcast<u32>(8i)))), 0i);
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
        let _e207 = unpack4x8unorm(_e79.y);
        if gh {
            phi_783_ = _e207;
        } else {
            let _e210 = (_e207.xyz * _e207.w);
            let _e216 = vec4<f32>(_e210.x, _e207.y, _e207.z, _e207.w);
            let _e222 = vec4<f32>(_e216.x, _e210.y, _e216.z, _e216.w);
            phi_783_ = vec4<f32>(_e222.x, _e222.y, _e210.z, _e222.w);
        }
        let _e230 = phi_783_;
        f1_ = _e230;
    } else {
        let _e158 = (_e52 * 8u);
        let _e165 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e158 & 255u)), bitcast<i32>((_e158 >> bitcast<u32>(8i)))), 0i);
        let _e173 = (_e158 + 1u);
        let _e180 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e173 & 255u)), bitcast<i32>((_e173 >> bitcast<u32>(8i)))), 0i);
        let _e183 = ((mat2x2<f32>(vec2<f32>(_e165.x, _e165.y), vec2<f32>(_e165.z, _e165.w)) * _e63) + _e180.xy);
        let _e184 = (_e81 == 2u);
        if (_e184 || (_e81 == 3u)) {
            f1_[3u] = -(bitcast<f32>(_e79.y));
            if (_e180.z > 0.9f) {
                f1_[2u] = 2f;
            } else {
                f1_[2u] = _e180.w;
            }
            if _e184 {
                f1_[1u] = 0f;
                f1_[0u] = _e183.x;
            } else {
                let _e197 = f1_[2u];
                f1_[2u] = -(_e197);
                f1_[0u] = _e183.x;
                f1_[1u] = _e183.y;
            }
        }
    }
    phi_480_ = mh;
    if mh {
        phi_480_ = ((_e79.x & 2048u) != 0u);
    }
    let _e234 = phi_480_;
    if _e234 {
        let _e235 = (_e52 * 8u);
        let _e236 = (_e235 + 4u);
        let _e243 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e236 & 255u)), bitcast<i32>((_e236 >> bitcast<u32>(8i)))), 0i);
        let _e251 = (_e235 + 5u);
        let _e258 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e251 & 255u)), bitcast<i32>((_e251 >> bitcast<u32>(8i)))), 0i);
        let _e261 = ((mat2x2<f32>(vec2<f32>(_e243.x, _e243.y), vec2<f32>(_e243.z, _e243.w)) * _e63) + _e258.xy);
        A2_ = vec3<f32>(_e261.x, _e261.y, (1f + _e258.z));
    } else {
        A2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e268 = m.jf;
    let _e270 = m.kf;
    let _e278 = vec4<f32>(((_e49.x * _e268) - 1f), ((_e49.y * _e270) - sign(_e270)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e278.x, _e278.y, (1f - (f32(_e61.x) * 0.000061035156f)), _e278.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e13 = unnamed.gl_Position;
    let _e14 = unnamed.gl_ClipDistance;
    let _e15 = D2_;
    let _e16 = K3_;
    let _e17 = e2_;
    let _e18 = f1_;
    let _e19 = A2_;
    return VertexOutput(_e13, _e14, _e15, _e16, _e17, _e18, _e19);
}
