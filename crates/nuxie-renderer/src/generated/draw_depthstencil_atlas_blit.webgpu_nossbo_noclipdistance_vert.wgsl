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

@id(0) override Eh: bool = true;
@id(2) override Gh: bool = true;
@id(8) override Mh: bool = true;

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
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(5)
var HD: texture_2d<u32>;
@group(3) @binding(9)
var ga: sampler;

fn main_1() {
    var phi_750_: u32;
    var phi_751_: f32;
    var phi_752_: f32;
    var phi_759_: vec4<f32>;
    var phi_760_: vec4<f32>;
    var phi_437_: bool;

    let _e46 = JB_1;
    let _e48 = bitcast<u32>(_e46.z);
    let _e49 = (_e48 & 65535u);
    let _e51 = ((_e49 * 4u) + 2u);
    let _e58 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e51 & 255u)), bitcast<i32>((_e51 >> bitcast<u32>(8i)))), 0i);
    let _e60 = _e46.xy;
    let _e62 = bitcast<vec3<f32>>(_e58.yzw);
    let _e68 = j.eh;
    F2_ = (((_e60 * _e62.x) + _e62.yz) * _e68);
    let _e76 = textureLoad(CD, vec2<i32>(bitcast<i32>((_e48 & 255u)), bitcast<i32>((_e49 >> bitcast<u32>(8i)))), 0i);
    let _e78 = (_e76.x & 15u);
    if Eh {
        let _e79 = (_e78 == 0u);
        if _e79 {
            phi_750_ = _e76.y;
        } else {
            phi_750_ = _e76.x;
        }
        let _e82 = phi_750_;
        let _e84 = (_e82 >> bitcast<u32>(16i));
        let _e86 = j.f6_;
        if (_e84 == 0u) {
            phi_751_ = 0f;
        } else {
            phi_751_ = unpack2x16float(((_e84 + 1023u) * _e86)).x;
        }
        let _e93 = phi_751_;
        phi_752_ = _e93;
        if _e79 {
            phi_752_ = -(_e93);
        }
        let _e96 = phi_752_;
        O3_ = _e96;
    }
    if Gh {
        g1_ = f32(((_e76.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e78 == 1u) {
        X1_ = unpack4x8unorm(_e76.y);
    } else {
        let _e102 = (_e49 * 8u);
        let _e109 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e102 & 255u)), bitcast<i32>((_e102 >> bitcast<u32>(8i)))), 0i);
        let _e117 = (_e102 + 1u);
        let _e124 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e117 & 255u)), bitcast<i32>((_e117 >> bitcast<u32>(8i)))), 0i);
        let _e133 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e76.y));
        let _e135 = ((mat2x2<f32>(vec2<f32>(_e109.x, _e109.y), vec2<f32>(_e109.z, _e109.w)) * _e60) + _e124.xy);
        if (_e124.z > 0.9f) {
            phi_759_ = vec4<f32>(_e133.x, _e133.y, 2f, _e133.w);
        } else {
            phi_759_ = vec4<f32>(_e133.x, _e133.y, _e124.w, _e133.w);
        }
        let _e150 = phi_759_;
        if (f32(_e78) == 2f) {
            let _e176 = vec4<f32>(_e135.x, _e150.y, _e150.z, _e150.w);
            phi_760_ = vec4<f32>(_e176.x, 0f, _e176.z, _e176.w);
        } else {
            let _e158 = vec4<f32>(_e150.x, _e150.y, -(_e150.z), _e150.w);
            let _e164 = vec4<f32>(_e135.x, _e158.y, _e158.z, _e158.w);
            phi_760_ = vec4<f32>(_e164.x, _e135.y, _e164.z, _e164.w);
        }
        let _e183 = phi_760_;
        X1_ = _e183;
        let _e185 = X1_[3u];
        X1_[3u] = -(_e185);
    }
    phi_437_ = Mh;
    if Mh {
        phi_437_ = ((_e76.x & 2048u) != 0u);
    }
    let _e192 = phi_437_;
    if _e192 {
        let _e193 = (_e49 * 8u);
        let _e194 = (_e193 + 4u);
        let _e201 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e194 & 255u)), bitcast<i32>((_e194 >> bitcast<u32>(8i)))), 0i);
        let _e209 = (_e193 + 5u);
        let _e216 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e209 & 255u)), bitcast<i32>((_e209 >> bitcast<u32>(8i)))), 0i);
        let _e219 = ((mat2x2<f32>(vec2<f32>(_e201.x, _e201.y), vec2<f32>(_e201.z, _e201.w)) * _e60) + _e216.xy);
        C2_ = vec3<f32>(_e219.x, _e219.y, (1f + _e216.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e226 = j.Hf;
    let _e228 = j.If;
    let _e236 = vec4<f32>(((_e46.x * _e226) - 1f), ((_e46.y * _e228) - sign(_e228)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e236.x, _e236.y, (1f - (f32(_e58.x) * 0.000061035156f)), _e236.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) JB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    JB_1 = JB;
    main_1();
    let _e12 = F2_;
    let _e13 = O3_;
    let _e14 = g1_;
    let _e15 = X1_;
    let _e16 = C2_;
    let _e17 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
