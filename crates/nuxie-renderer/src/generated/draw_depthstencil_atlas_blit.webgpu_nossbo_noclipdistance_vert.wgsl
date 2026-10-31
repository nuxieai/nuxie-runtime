struct WB {
    Hd: f32,
    Qe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    I8_: vec4<i32>,
    Ki: vec2<f32>,
    Re: vec2<f32>,
    w2_: u32,
    Oi: f32,
    w6_: u32,
    j3_: f32,
    Se: f32,
    xg: u32,
    I3_: f32,
    J3_: f32,
    Te: f32,
    Hi: u32,
    Ea: u32,
    P8_: f32,
    Q8_: f32,
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

@id(0) override kj: bool = true;
@id(2) override mj: bool = true;
@id(4) override oj: bool = true;

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
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(6)
var VB: texture_2d<u32>;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(4)
var CD: texture_2d<u32>;
@group(3) @binding(8)
var fb: sampler;

fn main_1() {
    var phi_834_: u32;
    var phi_835_: f32;
    var phi_836_: f32;
    var phi_843_: f32;
    var phi_471_: bool;
    var phi_845_: f32;

    let _e51 = LB_1;
    let _e54 = (bitcast<u32>(_e51.z) & 65535u);
    let _e55 = (_e54 * 4u);
    let _e56 = (_e55 + 1u);
    let _e62 = vec2<i32>(bitcast<i32>((_e56 & 255u)), bitcast<i32>((_e56 >> bitcast<u32>(8i))));
    let _e63 = textureLoad(JB, _e62, 0i);
    let _e67 = _e51.xy;
    let _e68 = (_e55 + 3u);
    let _e75 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e68 & 255u)), bitcast<i32>((_e68 >> bitcast<u32>(8i)))), 0i);
    let _e77 = bitcast<vec3<f32>>(_e75.xyz);
    let _e83 = j.Ki;
    W2_ = (((_e67 * _e77.x) + _e77.yz) * _e83);
    let _e85 = textureLoad(JB, _e62, 0i);
    let _e87 = (_e85.z & 15u);
    if kj {
        let _e88 = (_e87 == 0u);
        if _e88 {
            phi_834_ = _e85.w;
        } else {
            phi_834_ = _e85.z;
        }
        let _e91 = phi_834_;
        let _e93 = (_e91 >> bitcast<u32>(16i));
        let _e95 = j.w6_;
        if (_e93 == 0u) {
            phi_835_ = 0f;
        } else {
            phi_835_ = unpack2x16float(((_e93 + 1023u) * _e95)).x;
        }
        let _e102 = phi_835_;
        phi_836_ = _e102;
        if _e88 {
            phi_836_ = -(_e102);
        }
        let _e105 = phi_836_;
        h4_ = _e105;
    }
    if mj {
        R0_ = f32(((_e85.z >> bitcast<u32>(4i)) & 15u));
    }
    if (_e87 == 1u) {
        M0_ = unpack4x8unorm(_e85.w);
    } else {
        let _e111 = (_e54 * 8u);
        let _e118 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e111 & 255u)), bitcast<i32>((_e111 >> bitcast<u32>(8i)))), 0i);
        let _e126 = (_e111 + 1u);
        let _e133 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e126 & 255u)), bitcast<i32>((_e126 >> bitcast<u32>(8i)))), 0i);
        let _e138 = ((mat2x2<f32>(vec2<f32>(_e118.x, _e118.y), vec2<f32>(_e118.z, _e118.w)) * _e67) + _e133.xy);
        let _e149 = ((_e133.w + (f32(_e87) * 0.125f)) + (max(_e133.z, 0f) * 0.00024414063f));
        if (_e133.z < 0f) {
            phi_843_ = -(_e149);
        } else {
            phi_843_ = _e149;
        }
        let _e152 = phi_843_;
        M0_ = vec4<f32>(_e138.x, _e138.y, _e152, (-0.75f - round((bitcast<f32>(_e85.w) * 255f))));
    }
    phi_471_ = oj;
    if oj {
        phi_471_ = ((_e85.z & 2048u) != 0u);
    }
    let _e162 = phi_471_;
    if _e162 {
        let _e163 = (_e54 * 8u);
        let _e164 = (_e163 + 4u);
        let _e171 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e164 & 255u)), bitcast<i32>((_e164 >> bitcast<u32>(8i)))), 0i);
        let _e179 = (_e163 + 5u);
        let _e186 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e179 & 255u)), bitcast<i32>((_e179 >> bitcast<u32>(8i)))), 0i);
        let _e189 = ((mat2x2<f32>(vec2<f32>(_e171.x, _e171.y), vec2<f32>(_e171.z, _e171.w)) * _e67) + _e186.xy);
        phi_845_ = (1f + _e186.z);
        if ((_e85.z & 4096u) != 0u) {
            phi_845_ = (-1f - f32(((_e85.z & 24576u) >> bitcast<u32>(13u))));
        }
        let _e200 = phi_845_;
        N0_ = vec3<f32>(_e189.x, _e189.y, _e200);
    } else {
        N0_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e205 = j.Sg;
    let _e207 = j.Tg;
    let _e215 = vec4<f32>(((_e51.x * _e205) - 1f), ((_e51.y * _e207) - sign(_e207)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e215.x, _e215.y, ((f32((((_e63.z >> bitcast<u32>(16u)) << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e215.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e12 = W2_;
    let _e13 = h4_;
    let _e14 = R0_;
    let _e15 = M0_;
    let _e16 = N0_;
    let _e17 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
