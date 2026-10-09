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

@id(0) override Wi: bool = true;
@id(2) override Yi: bool = true;
@id(8) override ej: bool = true;

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
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(5)
var BD: texture_2d<u32>;
@group(3) @binding(9)
var ab: sampler;

fn main_1() {
    var phi_809_: u32;
    var phi_810_: f32;
    var phi_811_: f32;
    var phi_818_: f32;
    var phi_457_: bool;
    var phi_820_: f32;

    let _e51 = LB_1;
    let _e53 = bitcast<u32>(_e51.z);
    let _e54 = (_e53 & 65535u);
    let _e56 = ((_e54 * 4u) + 2u);
    let _e63 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e56 & 255u)), bitcast<i32>((_e56 >> bitcast<u32>(8i)))), 0i);
    let _e65 = _e51.xy;
    let _e67 = bitcast<vec3<f32>>(_e63.yzw);
    let _e73 = j.yi;
    S2_ = (((_e65 * _e67.x) + _e67.yz) * _e73);
    let _e81 = textureLoad(VC, vec2<i32>(bitcast<i32>((_e53 & 255u)), bitcast<i32>((_e54 >> bitcast<u32>(8i)))), 0i);
    let _e83 = (_e81.x & 15u);
    if Wi {
        let _e84 = (_e83 == 0u);
        if _e84 {
            phi_809_ = _e81.y;
        } else {
            phi_809_ = _e81.x;
        }
        let _e87 = phi_809_;
        let _e89 = (_e87 >> bitcast<u32>(16i));
        let _e91 = j.w6_;
        if (_e89 == 0u) {
            phi_810_ = 0f;
        } else {
            phi_810_ = unpack2x16float(((_e89 + 1023u) * _e91)).x;
        }
        let _e98 = phi_810_;
        phi_811_ = _e98;
        if _e84 {
            phi_811_ = -(_e98);
        }
        let _e101 = phi_811_;
        f4_ = _e101;
    }
    if Yi {
        P0_ = f32(((_e81.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e83 == 1u) {
        O0_ = unpack4x8unorm(_e81.y);
    } else {
        let _e107 = (_e54 * 8u);
        let _e114 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e107 & 255u)), bitcast<i32>((_e107 >> bitcast<u32>(8i)))), 0i);
        let _e122 = (_e107 + 1u);
        let _e129 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e122 & 255u)), bitcast<i32>((_e122 >> bitcast<u32>(8i)))), 0i);
        let _e134 = ((mat2x2<f32>(vec2<f32>(_e114.x, _e114.y), vec2<f32>(_e114.z, _e114.w)) * _e65) + _e129.xy);
        let _e145 = ((_e129.w + (f32(_e83) * 0.125f)) + (max(_e129.z, 0f) * 0.00024414063f));
        if (_e129.z < 0f) {
            phi_818_ = -(_e145);
        } else {
            phi_818_ = _e145;
        }
        let _e148 = phi_818_;
        O0_ = vec4<f32>(_e134.x, _e134.y, _e148, (-0.75f - round((bitcast<f32>(_e81.y) * 255f))));
    }
    phi_457_ = ej;
    if ej {
        phi_457_ = ((_e81.x & 2048u) != 0u);
    }
    let _e158 = phi_457_;
    if _e158 {
        let _e159 = (_e54 * 8u);
        let _e160 = (_e159 + 4u);
        let _e167 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e160 & 255u)), bitcast<i32>((_e160 >> bitcast<u32>(8i)))), 0i);
        let _e175 = (_e159 + 5u);
        let _e182 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e175 & 255u)), bitcast<i32>((_e175 >> bitcast<u32>(8i)))), 0i);
        let _e185 = ((mat2x2<f32>(vec2<f32>(_e167.x, _e167.y), vec2<f32>(_e167.z, _e167.w)) * _e65) + _e182.xy);
        phi_820_ = (1f + _e182.z);
        if ((_e81.x & 4096u) != 0u) {
            phi_820_ = (-1f - f32(((_e81.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e196 = phi_820_;
        U0_ = vec3<f32>(_e185.x, _e185.y, _e196);
    } else {
        U0_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e201 = j.Hg;
    let _e203 = j.Ig;
    let _e211 = vec4<f32>(((_e51.x * _e201) - 1f), ((_e51.y * _e203) - sign(_e203)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e211.x, _e211.y, ((f32(((_e63.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e211.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e12 = S2_;
    let _e13 = f4_;
    let _e14 = P0_;
    let _e15 = O0_;
    let _e16 = U0_;
    let _e17 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
