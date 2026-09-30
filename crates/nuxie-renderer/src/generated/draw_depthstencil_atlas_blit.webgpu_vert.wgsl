enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct Cg {
    e2_: array<vec4<u32>>,
}

struct BC {
    rc: f32,
    Bd: f32,
    Ff: f32,
    Gf: f32,
    q6_: u32,
    Ob: u32,
    rf: u32,
    sf: u32,
    V7_: vec4<i32>,
    ch: vec2<f32>,
    Cd: vec2<f32>,
    d2_: u32,
    gh: f32,
    f6_: u32,
    U2_: f32,
    Dd: f32,
    mf: u32,
    C3_: f32,
    D3_: f32,
    Ed: f32,
    Zg: u32,
    Nb: u32,
}

struct jf {
    e2_: array<vec2<u32>>,
}

struct kf {
    e2_: array<vec4<f32>>,
}

struct Dg {
    e2_: array<vec4<u32>>,
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

@id(0) override Ch: bool = true;
@id(2) override Eh: bool = true;
@id(1) override Dh: bool = true;
@id(8) override Kh: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(2)
var<storage> PB: Cg;
@group(0) @binding(0)
var<uniform> l: BC;
var<private> gl_VertexIndex_1: i32;
var<private> KB_1: vec3<f32>;
var<private> F2_: vec2<f32>;
@group(0) @binding(3)
var<storage> DD: jf;
var<private> L3_: f32;
var<private> g2_: f32;
@group(0) @binding(4)
var<storage> QB: kf;
var<private> V1_: vec4<f32>;
var<private> C2_: vec3<f32>;
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> ID: Dg;
@group(3) @binding(9)
var ea: sampler;

fn main_1() {
    var phi_805_: u32;
    var phi_806_: f32;
    var phi_807_: f32;
    var phi_816_: vec4<f32>;
    var phi_817_: vec4<f32>;
    var phi_818_: vec4<f32>;
    var phi_463_: bool;

    let _e48 = KB_1;
    let _e51 = (bitcast<u32>(_e48.z) & 65535u);
    let _e56 = PB.e2_[((_e51 * 4u) + 2u)];
    let _e58 = _e48.xy;
    let _e60 = bitcast<vec3<f32>>(_e56.yzw);
    let _e66 = l.ch;
    F2_ = (((_e58 * _e60.x) + _e60.yz) * _e66);
    let _e70 = DD.e2_[_e51];
    let _e72 = (_e70.x & 15u);
    if Ch {
        let _e73 = (_e72 == 0u);
        if _e73 {
            phi_805_ = _e70.y;
        } else {
            phi_805_ = _e70.x;
        }
        let _e76 = phi_805_;
        let _e78 = (_e76 >> bitcast<u32>(16i));
        let _e80 = l.f6_;
        if (_e78 == 0u) {
            phi_806_ = 0f;
        } else {
            phi_806_ = unpack2x16float(((_e78 + 1023u) * _e80)).x;
        }
        let _e87 = phi_806_;
        phi_807_ = _e87;
        if _e73 {
            phi_807_ = -(_e87);
        }
        let _e90 = phi_807_;
        L3_ = _e90;
    }
    if Eh {
        g2_ = f32(((_e70.x >> bitcast<u32>(4i)) & 15u));
    }
    if Dh {
        let _e95 = (_e51 * 8u);
        let _e99 = QB.e2_[(_e95 + 2u)];
        let _e110 = QB.e2_[(_e95 + 3u)];
        if any((_e99 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e125 = ((mat2x2<f32>(vec2<f32>(_e99.x, _e99.y), vec2<f32>(_e99.z, _e99.w)) * _e58) + _e110.xy);
            unnamed.gl_ClipDistance[0i] = (_e125.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e125.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e125.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e125.y);
        } else {
            let _e115 = (_e110.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e115;
            unnamed.gl_ClipDistance[2i] = _e115;
            unnamed.gl_ClipDistance[1i] = _e115;
            unnamed.gl_ClipDistance[0i] = _e115;
        }
    }
    if (_e72 == 1u) {
        let _e219 = unpack4x8unorm(_e70.y);
        if Eh {
            phi_818_ = _e219;
        } else {
            let _e222 = (_e219.xyz * _e219.w);
            let _e228 = vec4<f32>(_e222.x, _e219.y, _e219.z, _e219.w);
            let _e234 = vec4<f32>(_e228.x, _e222.y, _e228.z, _e228.w);
            phi_818_ = vec4<f32>(_e234.x, _e234.y, _e222.z, _e234.w);
        }
        let _e242 = phi_818_;
        V1_ = _e242;
    } else {
        let _e141 = (_e51 * 8u);
        let _e144 = QB.e2_[_e141];
        let _e155 = QB.e2_[(_e141 + 1u)];
        let _e164 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e70.y));
        let _e166 = ((mat2x2<f32>(vec2<f32>(_e144.x, _e144.y), vec2<f32>(_e144.z, _e144.w)) * _e58) + _e155.xy);
        if (_e155.z > 0.9f) {
            phi_816_ = vec4<f32>(_e164.x, _e164.y, 2f, _e164.w);
        } else {
            phi_816_ = vec4<f32>(_e164.x, _e164.y, _e155.w, _e164.w);
        }
        let _e181 = phi_816_;
        if (f32(_e72) == 2f) {
            let _e207 = vec4<f32>(_e166.x, _e181.y, _e181.z, _e181.w);
            phi_817_ = vec4<f32>(_e207.x, 0f, _e207.z, _e207.w);
        } else {
            let _e189 = vec4<f32>(_e181.x, _e181.y, -(_e181.z), _e181.w);
            let _e195 = vec4<f32>(_e166.x, _e189.y, _e189.z, _e189.w);
            phi_817_ = vec4<f32>(_e195.x, _e166.y, _e195.z, _e195.w);
        }
        let _e214 = phi_817_;
        V1_ = _e214;
        let _e216 = V1_[3u];
        V1_[3u] = -(_e216);
    }
    phi_463_ = Kh;
    if Kh {
        phi_463_ = ((_e70.x & 2048u) != 0u);
    }
    let _e246 = phi_463_;
    if _e246 {
        let _e247 = (_e51 * 8u);
        let _e251 = QB.e2_[(_e247 + 4u)];
        let _e262 = QB.e2_[(_e247 + 5u)];
        let _e265 = ((mat2x2<f32>(vec2<f32>(_e251.x, _e251.y), vec2<f32>(_e251.z, _e251.w)) * _e58) + _e262.xy);
        C2_ = vec3<f32>(_e265.x, _e265.y, (1f + _e262.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e272 = l.Ff;
    let _e274 = l.Gf;
    let _e282 = vec4<f32>(((_e48.x * _e272) - 1f), ((_e48.y * _e274) - sign(_e274)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e282.x, _e282.y, (1f - (f32(_e56.x) * 0.000061035156f)), _e282.w);
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
    let _e16 = L3_;
    let _e17 = g2_;
    let _e18 = V1_;
    let _e19 = C2_;
    return VertexOutput(_e13, _e14, _e15, _e16, _e17, _e18, _e19);
}
