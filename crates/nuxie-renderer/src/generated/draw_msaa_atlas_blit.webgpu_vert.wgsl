enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct hg {
    c2_: array<vec4<u32>>,
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

struct Me {
    c2_: array<vec2<u32>>,
}

struct Ne {
    c2_: array<vec4<f32>>,
}

struct ig {
    c2_: array<vec4<u32>>,
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
var<storage> QB: hg;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> D2_: vec2<f32>;
@group(0) @binding(3)
var<storage> BD: Me;
var<private> K3_: f32;
var<private> e2_: f32;
@group(0) @binding(4)
var<storage> RB: Ne;
var<private> f1_: vec4<f32>;
var<private> A2_: vec3<f32>;
@group(0) @binding(7)
var MC: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> FD: ig;
@group(3) @binding(9)
var aa: sampler;

fn main_1() {
    var phi_722_: u32;
    var phi_723_: f32;
    var phi_724_: f32;
    var phi_725_: vec4<f32>;
    var phi_439_: bool;

    let _e47 = LB_1;
    let _e50 = (bitcast<u32>(_e47.z) & 65535u);
    let _e55 = QB.c2_[((_e50 * 4u) + 2u)];
    let _e57 = _e47.xy;
    let _e59 = bitcast<vec3<f32>>(_e55.yzw);
    let _e65 = m.Hg;
    D2_ = (((_e57 * _e59.x) + _e59.yz) * _e65);
    let _e69 = BD.c2_[_e50];
    let _e71 = (_e69.x & 15u);
    if eh {
        let _e72 = (_e71 == 0u);
        if _e72 {
            phi_722_ = _e69.y;
        } else {
            phi_722_ = _e69.x;
        }
        let _e75 = phi_722_;
        let _e77 = (_e75 >> bitcast<u32>(16i));
        let _e79 = m.c6_;
        if (_e77 == 0u) {
            phi_723_ = 0f;
        } else {
            phi_723_ = unpack2x16float(((_e77 + 1023u) * _e79)).x;
        }
        let _e86 = phi_723_;
        phi_724_ = _e86;
        if _e72 {
            phi_724_ = -(_e86);
        }
        let _e89 = phi_724_;
        K3_ = _e89;
    }
    if gh {
        e2_ = f32(((_e69.x >> bitcast<u32>(4i)) & 15u));
    }
    if fh {
        let _e94 = (_e50 * 8u);
        let _e98 = RB.c2_[(_e94 + 2u)];
        let _e109 = RB.c2_[(_e94 + 3u)];
        if any((_e98 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e124 = ((mat2x2<f32>(vec2<f32>(_e98.x, _e98.y), vec2<f32>(_e98.z, _e98.w)) * _e57) + _e109.xy);
            unnamed.gl_ClipDistance[0i] = (_e124.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e124.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e124.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e124.y);
        } else {
            let _e114 = (_e109.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e114;
            unnamed.gl_ClipDistance[2i] = _e114;
            unnamed.gl_ClipDistance[1i] = _e114;
            unnamed.gl_ClipDistance[0i] = _e114;
        }
    }
    if (_e71 == 1u) {
        let _e181 = unpack4x8unorm(_e69.y);
        if gh {
            phi_725_ = _e181;
        } else {
            let _e184 = (_e181.xyz * _e181.w);
            let _e190 = vec4<f32>(_e184.x, _e181.y, _e181.z, _e181.w);
            let _e196 = vec4<f32>(_e190.x, _e184.y, _e190.z, _e190.w);
            phi_725_ = vec4<f32>(_e196.x, _e196.y, _e184.z, _e196.w);
        }
        let _e204 = phi_725_;
        f1_ = _e204;
    } else {
        let _e140 = (_e50 * 8u);
        let _e143 = RB.c2_[_e140];
        let _e154 = RB.c2_[(_e140 + 1u)];
        let _e157 = ((mat2x2<f32>(vec2<f32>(_e143.x, _e143.y), vec2<f32>(_e143.z, _e143.w)) * _e57) + _e154.xy);
        let _e158 = (_e71 == 2u);
        if (_e158 || (_e71 == 3u)) {
            f1_[3u] = -(bitcast<f32>(_e69.y));
            if (_e154.z > 0.9f) {
                f1_[2u] = 2f;
            } else {
                f1_[2u] = _e154.w;
            }
            if _e158 {
                f1_[1u] = 0f;
                f1_[0u] = _e157.x;
            } else {
                let _e171 = f1_[2u];
                f1_[2u] = -(_e171);
                f1_[0u] = _e157.x;
                f1_[1u] = _e157.y;
            }
        }
    }
    phi_439_ = mh;
    if mh {
        phi_439_ = ((_e69.x & 2048u) != 0u);
    }
    let _e208 = phi_439_;
    if _e208 {
        let _e209 = (_e50 * 8u);
        let _e213 = RB.c2_[(_e209 + 4u)];
        let _e224 = RB.c2_[(_e209 + 5u)];
        let _e227 = ((mat2x2<f32>(vec2<f32>(_e213.x, _e213.y), vec2<f32>(_e213.z, _e213.w)) * _e57) + _e224.xy);
        A2_ = vec3<f32>(_e227.x, _e227.y, (1f + _e224.z));
    } else {
        A2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e234 = m.jf;
    let _e236 = m.kf;
    let _e244 = vec4<f32>(((_e47.x * _e234) - 1f), ((_e47.y * _e236) - sign(_e236)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e244.x, _e244.y, (1f - (f32(_e55.x) * 0.000061035156f)), _e244.w);
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
