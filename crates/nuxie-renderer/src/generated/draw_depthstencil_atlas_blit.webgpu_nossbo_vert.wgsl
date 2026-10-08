enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct VB {
    td: f32,
    ze: f32,
    Dg: f32,
    Eg: f32,
    L6_: u32,
    va: u32,
    pg: u32,
    qg: u32,
    B8_: vec4<i32>,
    xi: vec2<f32>,
    Ae: vec2<f32>,
    r2_: u32,
    Bi: f32,
    p6_: u32,
    h3_: f32,
    Be: f32,
    jg: u32,
    E3_: f32,
    F3_: f32,
    Ce: f32,
    ui: u32,
    ua: u32,
    ad: f32,
    g7_: f32,
    Bb: f32,
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

@id(0) override Ui: bool = true;
@id(2) override Wi: bool = true;
@id(1) override Vi: bool = true;
@id(8) override cj: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(2)
var KB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> T2_: vec2<f32>;
@group(0) @binding(3)
var WC: texture_2d<u32>;
var<private> e4_: f32;
var<private> P0_: f32;
@group(0) @binding(4)
var JB: texture_2d<f32>;
var<private> O0_: vec4<f32>;
var<private> V0_: vec3<f32>;
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(0) @binding(5)
var BD: texture_2d<u32>;
@group(3) @binding(9)
var Ta: sampler;

fn main_1() {
    var phi_878_: u32;
    var phi_879_: f32;
    var phi_880_: f32;
    var phi_889_: vec4<f32>;
    var phi_890_: vec4<f32>;
    var phi_496_: bool;
    var phi_891_: f32;

    let _e55 = LB_1;
    let _e57 = bitcast<u32>(_e55.z);
    let _e58 = (_e57 & 65535u);
    let _e60 = ((_e58 * 4u) + 2u);
    let _e67 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e60 & 255u)), bitcast<i32>((_e60 >> bitcast<u32>(8i)))), 0i);
    let _e69 = _e55.xy;
    let _e71 = bitcast<vec3<f32>>(_e67.yzw);
    let _e77 = j.xi;
    T2_ = (((_e69 * _e71.x) + _e71.yz) * _e77);
    let _e85 = textureLoad(WC, vec2<i32>(bitcast<i32>((_e57 & 255u)), bitcast<i32>((_e58 >> bitcast<u32>(8i)))), 0i);
    let _e87 = (_e85.x & 15u);
    if Ui {
        let _e88 = (_e87 == 0u);
        if _e88 {
            phi_878_ = _e85.y;
        } else {
            phi_878_ = _e85.x;
        }
        let _e91 = phi_878_;
        let _e93 = (_e91 >> bitcast<u32>(16i));
        let _e95 = j.p6_;
        if (_e93 == 0u) {
            phi_879_ = 0f;
        } else {
            phi_879_ = unpack2x16float(((_e93 + 1023u) * _e95)).x;
        }
        let _e102 = phi_879_;
        phi_880_ = _e102;
        if _e88 {
            phi_880_ = -(_e102);
        }
        let _e105 = phi_880_;
        e4_ = _e105;
    }
    if Wi {
        P0_ = f32(((_e85.x >> bitcast<u32>(4i)) & 15u));
    }
    if Vi {
        let _e110 = (_e58 * 8u);
        let _e111 = (_e110 + 2u);
        let _e118 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e111 & 255u)), bitcast<i32>((_e111 >> bitcast<u32>(8i)))), 0i);
        let _e126 = (_e110 + 3u);
        let _e133 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e126 & 255u)), bitcast<i32>((_e126 >> bitcast<u32>(8i)))), 0i);
        if any((_e118 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e148 = ((mat2x2<f32>(vec2<f32>(_e118.x, _e118.y), vec2<f32>(_e118.z, _e118.w)) * _e69) + _e133.xy);
            unnamed.gl_ClipDistance[0i] = (_e148.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e148.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e148.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e148.y);
        } else {
            let _e138 = (_e133.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e138;
            unnamed.gl_ClipDistance[2i] = _e138;
            unnamed.gl_ClipDistance[1i] = _e138;
            unnamed.gl_ClipDistance[0i] = _e138;
        }
    }
    if (_e87 == 1u) {
        O0_ = unpack4x8unorm(_e85.y);
    } else {
        let _e164 = (_e58 * 8u);
        let _e171 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e164 & 255u)), bitcast<i32>((_e164 >> bitcast<u32>(8i)))), 0i);
        let _e179 = (_e164 + 1u);
        let _e186 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e179 & 255u)), bitcast<i32>((_e179 >> bitcast<u32>(8i)))), 0i);
        let _e195 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e85.y));
        let _e197 = ((mat2x2<f32>(vec2<f32>(_e171.x, _e171.y), vec2<f32>(_e171.z, _e171.w)) * _e69) + _e186.xy);
        if (_e186.z > 0.9f) {
            phi_889_ = vec4<f32>(_e195.x, _e195.y, 2f, _e195.w);
        } else {
            phi_889_ = vec4<f32>(_e195.x, _e195.y, _e186.w, _e195.w);
        }
        let _e212 = phi_889_;
        if (f32(_e87) == 2f) {
            let _e238 = vec4<f32>(_e197.x, _e212.y, _e212.z, _e212.w);
            phi_890_ = vec4<f32>(_e238.x, 0f, _e238.z, _e238.w);
        } else {
            let _e220 = vec4<f32>(_e212.x, _e212.y, -(_e212.z), _e212.w);
            let _e226 = vec4<f32>(_e197.x, _e220.y, _e220.z, _e220.w);
            phi_890_ = vec4<f32>(_e226.x, _e197.y, _e226.z, _e226.w);
        }
        let _e245 = phi_890_;
        O0_ = _e245;
        let _e247 = O0_[3u];
        O0_[3u] = -(_e247);
    }
    phi_496_ = cj;
    if cj {
        phi_496_ = ((_e85.x & 2048u) != 0u);
    }
    let _e254 = phi_496_;
    if _e254 {
        let _e255 = (_e58 * 8u);
        let _e256 = (_e255 + 4u);
        let _e263 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e256 & 255u)), bitcast<i32>((_e256 >> bitcast<u32>(8i)))), 0i);
        let _e271 = (_e255 + 5u);
        let _e278 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e271 & 255u)), bitcast<i32>((_e271 >> bitcast<u32>(8i)))), 0i);
        let _e281 = ((mat2x2<f32>(vec2<f32>(_e263.x, _e263.y), vec2<f32>(_e263.z, _e263.w)) * _e69) + _e278.xy);
        phi_891_ = (1f + _e278.z);
        if ((_e85.x & 4096u) != 0u) {
            phi_891_ = (-1f - f32(((_e85.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e292 = phi_891_;
        V0_ = vec3<f32>(_e281.x, _e281.y, _e292);
    } else {
        V0_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e297 = j.Dg;
    let _e299 = j.Eg;
    let _e307 = vec4<f32>(((_e55.x * _e297) - 1f), ((_e55.y * _e299) - sign(_e299)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e307.x, _e307.y, ((f32(((_e67.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e307.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e13 = unnamed.gl_Position;
    let _e14 = unnamed.gl_ClipDistance;
    let _e15 = T2_;
    let _e16 = e4_;
    let _e17 = P0_;
    let _e18 = O0_;
    let _e19 = V0_;
    return VertexOutput(_e13, _e14, _e15, _e16, _e17, _e18, _e19);
}
