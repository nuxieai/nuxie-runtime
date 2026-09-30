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
    f6_: u32,
    U2_: f32,
    Cd: f32,
    lf: u32,
    C3_: f32,
    D3_: f32,
    Dd: f32,
    Yg: u32,
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

@id(0) override Bh: bool = true;
@id(2) override Dh: bool = true;
@id(8) override Jh: bool = true;

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
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(5)
var ID: texture_2d<u32>;
@group(3) @binding(9)
var ea: sampler;

fn main_1() {
    var phi_773_: u32;
    var phi_774_: f32;
    var phi_775_: f32;
    var phi_782_: vec4<f32>;
    var phi_783_: vec4<f32>;
    var phi_784_: vec4<f32>;
    var phi_453_: bool;

    let _e46 = KB_1;
    let _e48 = bitcast<u32>(_e46.z);
    let _e49 = (_e48 & 65535u);
    let _e51 = ((_e49 * 4u) + 2u);
    let _e58 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e51 & 255u)), bitcast<i32>((_e51 >> bitcast<u32>(8i)))), 0i);
    let _e60 = _e46.xy;
    let _e62 = bitcast<vec3<f32>>(_e58.yzw);
    let _e68 = n.bh;
    F2_ = (((_e60 * _e62.x) + _e62.yz) * _e68);
    let _e76 = textureLoad(DD, vec2<i32>(bitcast<i32>((_e48 & 255u)), bitcast<i32>((_e49 >> bitcast<u32>(8i)))), 0i);
    let _e78 = (_e76.x & 15u);
    if Bh {
        let _e79 = (_e78 == 0u);
        if _e79 {
            phi_773_ = _e76.y;
        } else {
            phi_773_ = _e76.x;
        }
        let _e82 = phi_773_;
        let _e84 = (_e82 >> bitcast<u32>(16i));
        let _e86 = n.f6_;
        if (_e84 == 0u) {
            phi_774_ = 0f;
        } else {
            phi_774_ = unpack2x16float(((_e84 + 1023u) * _e86)).x;
        }
        let _e93 = phi_774_;
        phi_775_ = _e93;
        if _e79 {
            phi_775_ = -(_e93);
        }
        let _e96 = phi_775_;
        K3_ = _e96;
    }
    if Dh {
        g2_ = f32(((_e76.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e78 == 1u) {
        let _e188 = unpack4x8unorm(_e76.y);
        if Dh {
            phi_784_ = _e188;
        } else {
            let _e191 = (_e188.xyz * _e188.w);
            let _e197 = vec4<f32>(_e191.x, _e188.y, _e188.z, _e188.w);
            let _e203 = vec4<f32>(_e197.x, _e191.y, _e197.z, _e197.w);
            phi_784_ = vec4<f32>(_e203.x, _e203.y, _e191.z, _e203.w);
        }
        let _e211 = phi_784_;
        V1_ = _e211;
    } else {
        let _e102 = (_e49 * 8u);
        let _e109 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e102 & 255u)), bitcast<i32>((_e102 >> bitcast<u32>(8i)))), 0i);
        let _e117 = (_e102 + 1u);
        let _e124 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e117 & 255u)), bitcast<i32>((_e117 >> bitcast<u32>(8i)))), 0i);
        let _e133 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e76.y));
        let _e135 = ((mat2x2<f32>(vec2<f32>(_e109.x, _e109.y), vec2<f32>(_e109.z, _e109.w)) * _e60) + _e124.xy);
        if (_e124.z > 0.9f) {
            phi_782_ = vec4<f32>(_e133.x, _e133.y, 2f, _e133.w);
        } else {
            phi_782_ = vec4<f32>(_e133.x, _e133.y, _e124.w, _e133.w);
        }
        let _e150 = phi_782_;
        if (f32(_e78) == 2f) {
            let _e176 = vec4<f32>(_e135.x, _e150.y, _e150.z, _e150.w);
            phi_783_ = vec4<f32>(_e176.x, 0f, _e176.z, _e176.w);
        } else {
            let _e158 = vec4<f32>(_e150.x, _e150.y, -(_e150.z), _e150.w);
            let _e164 = vec4<f32>(_e135.x, _e158.y, _e158.z, _e158.w);
            phi_783_ = vec4<f32>(_e164.x, _e135.y, _e164.z, _e164.w);
        }
        let _e183 = phi_783_;
        V1_ = _e183;
        let _e185 = V1_[3u];
        V1_[3u] = -(_e185);
    }
    phi_453_ = Jh;
    if Jh {
        phi_453_ = ((_e76.x & 2048u) != 0u);
    }
    let _e215 = phi_453_;
    if _e215 {
        let _e216 = (_e49 * 8u);
        let _e217 = (_e216 + 4u);
        let _e224 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e217 & 255u)), bitcast<i32>((_e217 >> bitcast<u32>(8i)))), 0i);
        let _e232 = (_e216 + 5u);
        let _e239 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e232 & 255u)), bitcast<i32>((_e232 >> bitcast<u32>(8i)))), 0i);
        let _e242 = ((mat2x2<f32>(vec2<f32>(_e224.x, _e224.y), vec2<f32>(_e224.z, _e224.w)) * _e60) + _e239.xy);
        C2_ = vec3<f32>(_e242.x, _e242.y, (1f + _e239.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e249 = n.Ef;
    let _e251 = n.Ff;
    let _e259 = vec4<f32>(((_e46.x * _e249) - 1f), ((_e46.y * _e251) - sign(_e251)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e259.x, _e259.y, (1f - (f32(_e58.x) * 0.000061035156f)), _e259.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) KB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    KB_1 = KB;
    main_1();
    let _e12 = F2_;
    let _e13 = K3_;
    let _e14 = g2_;
    let _e15 = V1_;
    let _e16 = C2_;
    let _e17 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
