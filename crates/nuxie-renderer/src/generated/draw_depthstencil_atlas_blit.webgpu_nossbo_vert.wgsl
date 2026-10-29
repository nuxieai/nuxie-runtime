enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct WB {
    Gd: f32,
    Pe: f32,
    Rg: f32,
    Sg: f32,
    R6_: u32,
    Fa: u32,
    Dg: u32,
    Eg: u32,
    H8_: vec4<i32>,
    Ji: vec2<f32>,
    Qe: vec2<f32>,
    w2_: u32,
    Ni: f32,
    w6_: u32,
    j3_: f32,
    Re: f32,
    wg: u32,
    I3_: f32,
    J3_: f32,
    Se: f32,
    Gi: u32,
    Ea: u32,
    O8_: f32,
    P8_: f32,
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

@id(0) override jj: bool = true;
@id(2) override lj: bool = true;
@id(1) override kj: bool = true;
@id(4) override nj: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(2)
var JB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> W2_: vec2<f32>;
var<private> h4_: f32;
var<private> R0_: f32;
@group(0) @binding(3)
var KB: texture_2d<f32>;
var<private> M0_: vec4<f32>;
var<private> N0_: vec3<f32>;
@group(0) @binding(6)
var VB: texture_2d<u32>;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(4)
var CD: texture_2d<u32>;
@group(3) @binding(8)
var fb: sampler;

fn main_1() {
    var phi_924_: u32;
    var phi_925_: f32;
    var phi_926_: f32;
    var phi_935_: f32;
    var phi_522_: bool;
    var phi_937_: f32;

    let _e56 = LB_1;
    let _e59 = (bitcast<u32>(_e56.z) & 65535u);
    let _e60 = (_e59 * 4u);
    let _e61 = (_e60 + 1u);
    let _e67 = vec2<i32>(bitcast<i32>((_e61 & 255u)), bitcast<i32>((_e61 >> bitcast<u32>(8i))));
    let _e68 = textureLoad(JB, _e67, 0i);
    let _e72 = _e56.xy;
    let _e73 = (_e60 + 3u);
    let _e80 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e73 & 255u)), bitcast<i32>((_e73 >> bitcast<u32>(8i)))), 0i);
    let _e82 = bitcast<vec3<f32>>(_e80.xyz);
    let _e88 = j.Ji;
    W2_ = (((_e72 * _e82.x) + _e82.yz) * _e88);
    let _e90 = textureLoad(JB, _e67, 0i);
    let _e92 = (_e90.z & 15u);
    if jj {
        let _e93 = (_e92 == 0u);
        if _e93 {
            phi_924_ = _e90.w;
        } else {
            phi_924_ = _e90.z;
        }
        let _e96 = phi_924_;
        let _e98 = (_e96 >> bitcast<u32>(16i));
        let _e100 = j.w6_;
        if (_e98 == 0u) {
            phi_925_ = 0f;
        } else {
            phi_925_ = unpack2x16float(((_e98 + 1023u) * _e100)).x;
        }
        let _e107 = phi_925_;
        phi_926_ = _e107;
        if _e93 {
            phi_926_ = -(_e107);
        }
        let _e110 = phi_926_;
        h4_ = _e110;
    }
    if lj {
        R0_ = f32(((_e90.z >> bitcast<u32>(4i)) & 15u));
    }
    if kj {
        let _e115 = (_e59 * 8u);
        let _e116 = (_e115 + 2u);
        let _e123 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e116 & 255u)), bitcast<i32>((_e116 >> bitcast<u32>(8i)))), 0i);
        let _e131 = (_e115 + 3u);
        let _e138 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e131 & 255u)), bitcast<i32>((_e131 >> bitcast<u32>(8i)))), 0i);
        if any((_e123 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e153 = ((mat2x2<f32>(vec2<f32>(_e123.x, _e123.y), vec2<f32>(_e123.z, _e123.w)) * _e72) + _e138.xy);
            unnamed.gl_ClipDistance[0i] = (_e153.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e153.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e153.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e153.y);
        } else {
            let _e143 = (_e138.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e143;
            unnamed.gl_ClipDistance[2i] = _e143;
            unnamed.gl_ClipDistance[1i] = _e143;
            unnamed.gl_ClipDistance[0i] = _e143;
        }
    }
    if (_e92 == 1u) {
        M0_ = unpack4x8unorm(_e90.w);
    } else {
        let _e169 = (_e59 * 8u);
        let _e176 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e169 & 255u)), bitcast<i32>((_e169 >> bitcast<u32>(8i)))), 0i);
        let _e184 = (_e169 + 1u);
        let _e191 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e184 & 255u)), bitcast<i32>((_e184 >> bitcast<u32>(8i)))), 0i);
        let _e196 = ((mat2x2<f32>(vec2<f32>(_e176.x, _e176.y), vec2<f32>(_e176.z, _e176.w)) * _e72) + _e191.xy);
        let _e207 = ((_e191.w + (f32(_e92) * 0.125f)) + (max(_e191.z, 0f) * 0.00024414063f));
        if (_e191.z < 0f) {
            phi_935_ = -(_e207);
        } else {
            phi_935_ = _e207;
        }
        let _e210 = phi_935_;
        M0_ = vec4<f32>(_e196.x, _e196.y, _e210, (-0.75f - round((bitcast<f32>(_e90.w) * 255f))));
    }
    phi_522_ = nj;
    if nj {
        phi_522_ = ((_e90.z & 2048u) != 0u);
    }
    let _e220 = phi_522_;
    if _e220 {
        let _e221 = (_e59 * 8u);
        let _e222 = (_e221 + 4u);
        let _e229 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e222 & 255u)), bitcast<i32>((_e222 >> bitcast<u32>(8i)))), 0i);
        let _e237 = (_e221 + 5u);
        let _e244 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e237 & 255u)), bitcast<i32>((_e237 >> bitcast<u32>(8i)))), 0i);
        let _e247 = ((mat2x2<f32>(vec2<f32>(_e229.x, _e229.y), vec2<f32>(_e229.z, _e229.w)) * _e72) + _e244.xy);
        phi_937_ = (1f + _e244.z);
        if ((_e90.z & 4096u) != 0u) {
            phi_937_ = (-1f - f32(((_e90.z & 24576u) >> bitcast<u32>(13u))));
        }
        let _e258 = phi_937_;
        N0_ = vec3<f32>(_e247.x, _e247.y, _e258);
    } else {
        N0_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e263 = j.Rg;
    let _e265 = j.Sg;
    let _e273 = vec4<f32>(((_e56.x * _e263) - 1f), ((_e56.y * _e265) - sign(_e265)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e273.x, _e273.y, ((f32((((_e68.z >> bitcast<u32>(16u)) << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e273.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e13 = unnamed.gl_Position;
    let _e14 = unnamed.gl_ClipDistance;
    let _e15 = W2_;
    let _e16 = h4_;
    let _e17 = R0_;
    let _e18 = M0_;
    let _e19 = N0_;
    return VertexOutput(_e13, _e14, _e15, _e16, _e17, _e18, _e19);
}
