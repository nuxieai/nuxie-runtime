struct TB {
    uc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Ob: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    ih: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    mh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    fh: u32,
    Nb: u32,
    ac: f32,
    bc: f32,
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

@id(0) override Ih: bool = true;
@id(2) override Kh: bool = true;
@id(8) override Qh: bool = true;

@group(0) @binding(2)
var OB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> gl_VertexIndex_1: i32;
var<private> JB_1: vec3<f32>;
var<private> F2_: vec2<f32>;
@group(0) @binding(3)
var DD: texture_2d<u32>;
var<private> O3_: f32;
var<private> g1_: f32;
@group(0) @binding(4)
var PB: texture_2d<f32>;
var<private> X1_: vec4<f32>;
var<private> C2_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(5)
var ID: texture_2d<u32>;
@group(3) @binding(9)
var da: sampler;

fn main_1() {
    var phi_766_: u32;
    var phi_767_: f32;
    var phi_768_: f32;
    var phi_775_: vec4<f32>;
    var phi_776_: vec4<f32>;
    var phi_445_: bool;

    let _e47 = JB_1;
    let _e49 = bitcast<u32>(_e47.z);
    let _e50 = (_e49 & 65535u);
    let _e52 = ((_e50 * 4u) + 2u);
    let _e59 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e52 & 255u)), bitcast<i32>((_e52 >> bitcast<u32>(8i)))), 0i);
    let _e61 = _e47.xy;
    let _e63 = bitcast<vec3<f32>>(_e59.yzw);
    let _e69 = j.ih;
    F2_ = (((_e61 * _e63.x) + _e63.yz) * _e69);
    let _e77 = textureLoad(DD, vec2<i32>(bitcast<i32>((_e49 & 255u)), bitcast<i32>((_e50 >> bitcast<u32>(8i)))), 0i);
    let _e79 = (_e77.x & 15u);
    if Ih {
        let _e80 = (_e79 == 0u);
        if _e80 {
            phi_766_ = _e77.y;
        } else {
            phi_766_ = _e77.x;
        }
        let _e83 = phi_766_;
        let _e85 = (_e83 >> bitcast<u32>(16i));
        let _e87 = j.c6_;
        if (_e85 == 0u) {
            phi_767_ = 0f;
        } else {
            phi_767_ = unpack2x16float(((_e85 + 1023u) * _e87)).x;
        }
        let _e94 = phi_767_;
        phi_768_ = _e94;
        if _e80 {
            phi_768_ = -(_e94);
        }
        let _e97 = phi_768_;
        O3_ = _e97;
    }
    if Kh {
        g1_ = f32(((_e77.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e79 == 1u) {
        X1_ = unpack4x8unorm(_e77.y);
    } else {
        let _e103 = (_e50 * 8u);
        let _e110 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e103 & 255u)), bitcast<i32>((_e103 >> bitcast<u32>(8i)))), 0i);
        let _e118 = (_e103 + 1u);
        let _e125 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e118 & 255u)), bitcast<i32>((_e118 >> bitcast<u32>(8i)))), 0i);
        let _e134 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e77.y));
        let _e136 = ((mat2x2<f32>(vec2<f32>(_e110.x, _e110.y), vec2<f32>(_e110.z, _e110.w)) * _e61) + _e125.xy);
        if (_e125.z > 0.9f) {
            phi_775_ = vec4<f32>(_e134.x, _e134.y, 2f, _e134.w);
        } else {
            phi_775_ = vec4<f32>(_e134.x, _e134.y, _e125.w, _e134.w);
        }
        let _e151 = phi_775_;
        if (f32(_e79) == 2f) {
            let _e177 = vec4<f32>(_e136.x, _e151.y, _e151.z, _e151.w);
            phi_776_ = vec4<f32>(_e177.x, 0f, _e177.z, _e177.w);
        } else {
            let _e159 = vec4<f32>(_e151.x, _e151.y, -(_e151.z), _e151.w);
            let _e165 = vec4<f32>(_e136.x, _e159.y, _e159.z, _e159.w);
            phi_776_ = vec4<f32>(_e165.x, _e136.y, _e165.z, _e165.w);
        }
        let _e184 = phi_776_;
        X1_ = _e184;
        let _e186 = X1_[3u];
        X1_[3u] = -(_e186);
    }
    phi_445_ = Qh;
    if Qh {
        phi_445_ = ((_e77.x & 2048u) != 0u);
    }
    let _e193 = phi_445_;
    if _e193 {
        let _e194 = (_e50 * 8u);
        let _e195 = (_e194 + 4u);
        let _e202 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e195 & 255u)), bitcast<i32>((_e195 >> bitcast<u32>(8i)))), 0i);
        let _e210 = (_e194 + 5u);
        let _e217 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e210 & 255u)), bitcast<i32>((_e210 >> bitcast<u32>(8i)))), 0i);
        let _e220 = ((mat2x2<f32>(vec2<f32>(_e202.x, _e202.y), vec2<f32>(_e202.z, _e202.w)) * _e61) + _e217.xy);
        C2_ = vec3<f32>(_e220.x, _e220.y, (1f + _e217.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e227 = j.Hf;
    let _e229 = j.If;
    let _e237 = vec4<f32>(((_e47.x * _e227) - 1f), ((_e47.y * _e229) - sign(_e229)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e237.x, _e237.y, ((f32(((_e59.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e237.w);
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
