enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct li {
    v2_: array<vec4<u32>>,
}

struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
    P8_: f32,
    Q8_: f32,
}

struct og {
    v2_: array<vec2<u32>>,
}

struct pg {
    v2_: array<vec4<f32>>,
}

struct mi {
    v2_: array<vec4<u32>>,
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

@id(0) override fj: bool = true;
@id(2) override hj: bool = true;
@id(1) override gj: bool = true;
@id(4) override jj: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(2)
var<storage> KB: li;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> U2_: vec2<f32>;
@group(0) @binding(3)
var<storage> XC: og;
var<private> h4_: f32;
var<private> Q0_: f32;
@group(0) @binding(4)
var<storage> JB: pg;
var<private> P0_: vec4<f32>;
var<private> V0_: vec3<f32>;
@group(0) @binding(7)
var VB: texture_2d<u32>;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(0) @binding(5)
var<storage> DD: mi;
@group(3) @binding(9)
var db: sampler;

fn main_1() {
    var phi_843_: u32;
    var phi_844_: f32;
    var phi_845_: f32;
    var phi_854_: f32;
    var phi_467_: bool;
    var phi_856_: f32;

    let _e55 = LB_1;
    let _e58 = (bitcast<u32>(_e55.z) & 65535u);
    let _e63 = KB.v2_[((_e58 * 4u) + 2u)];
    let _e65 = _e55.xy;
    let _e67 = bitcast<vec3<f32>>(_e63.yzw);
    let _e73 = j.Gi;
    U2_ = (((_e65 * _e67.x) + _e67.yz) * _e73);
    let _e77 = XC.v2_[_e58];
    let _e79 = (_e77.x & 15u);
    if fj {
        let _e80 = (_e79 == 0u);
        if _e80 {
            phi_843_ = _e77.y;
        } else {
            phi_843_ = _e77.x;
        }
        let _e83 = phi_843_;
        let _e85 = (_e83 >> bitcast<u32>(16i));
        let _e87 = j.z6_;
        if (_e85 == 0u) {
            phi_844_ = 0f;
        } else {
            phi_844_ = unpack2x16float(((_e85 + 1023u) * _e87)).x;
        }
        let _e94 = phi_844_;
        phi_845_ = _e94;
        if _e80 {
            phi_845_ = -(_e94);
        }
        let _e97 = phi_845_;
        h4_ = _e97;
    }
    if hj {
        Q0_ = f32(((_e77.x >> bitcast<u32>(4i)) & 15u));
    }
    if gj {
        let _e102 = (_e58 * 8u);
        let _e106 = JB.v2_[(_e102 + 2u)];
        let _e117 = JB.v2_[(_e102 + 3u)];
        if any((_e106 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e132 = ((mat2x2<f32>(vec2<f32>(_e106.x, _e106.y), vec2<f32>(_e106.z, _e106.w)) * _e65) + _e117.xy);
            unnamed.gl_ClipDistance[0i] = (_e132.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e132.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e132.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e132.y);
        } else {
            let _e122 = (_e117.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e122;
            unnamed.gl_ClipDistance[2i] = _e122;
            unnamed.gl_ClipDistance[1i] = _e122;
            unnamed.gl_ClipDistance[0i] = _e122;
        }
    }
    if (_e79 == 1u) {
        P0_ = unpack4x8unorm(_e77.y);
    } else {
        let _e148 = (_e58 * 8u);
        let _e151 = JB.v2_[_e148];
        let _e162 = JB.v2_[(_e148 + 1u)];
        let _e167 = ((mat2x2<f32>(vec2<f32>(_e151.x, _e151.y), vec2<f32>(_e151.z, _e151.w)) * _e65) + _e162.xy);
        let _e178 = ((_e162.w + (f32(_e79) * 0.125f)) + (max(_e162.z, 0f) * 0.00024414063f));
        if (_e162.z < 0f) {
            phi_854_ = -(_e178);
        } else {
            phi_854_ = _e178;
        }
        let _e181 = phi_854_;
        P0_ = vec4<f32>(_e167.x, _e167.y, _e181, (-0.75f - round((bitcast<f32>(_e77.y) * 255f))));
    }
    phi_467_ = jj;
    if jj {
        phi_467_ = ((_e77.x & 2048u) != 0u);
    }
    let _e191 = phi_467_;
    if _e191 {
        let _e192 = (_e58 * 8u);
        let _e196 = JB.v2_[(_e192 + 4u)];
        let _e207 = JB.v2_[(_e192 + 5u)];
        let _e210 = ((mat2x2<f32>(vec2<f32>(_e196.x, _e196.y), vec2<f32>(_e196.z, _e196.w)) * _e65) + _e207.xy);
        phi_856_ = (1f + _e207.z);
        if ((_e77.x & 4096u) != 0u) {
            phi_856_ = (-1f - f32(((_e77.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e221 = phi_856_;
        V0_ = vec3<f32>(_e210.x, _e210.y, _e221);
    } else {
        V0_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e226 = j.Mg;
    let _e228 = j.Ng;
    let _e236 = vec4<f32>(((_e55.x * _e226) - 1f), ((_e55.y * _e228) - sign(_e228)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e236.x, _e236.y, ((f32(((_e63.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e236.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e13 = unnamed.gl_Position;
    let _e14 = unnamed.gl_ClipDistance;
    let _e15 = U2_;
    let _e16 = h4_;
    let _e17 = Q0_;
    let _e18 = P0_;
    let _e19 = V0_;
    return VertexOutput(_e13, _e14, _e15, _e16, _e17, _e18, _e19);
}
