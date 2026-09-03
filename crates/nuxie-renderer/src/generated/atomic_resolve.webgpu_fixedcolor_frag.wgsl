struct Ne {
    d2_: array<vec2<u32>>,
}

struct h0Ed {
    d2_: array<u32>,
}

struct Oe {
    d2_: array<vec4<f32>>,
}

struct DC {
    hc: f32,
    rd: f32,
    kf: f32,
    lf: f32,
    p6_: u32,
    Mg: u32,
    Ve: u32,
    We: u32,
    U7_: vec4<i32>,
    Ig: vec2<f32>,
    sd: vec2<f32>,
    c2_: u32,
    Ng: f32,
    d6_: u32,
    R2_: f32,
    td: f32,
    Qe: u32,
    B3_: f32,
    C3_: f32,
    ud: f32,
    Fg: u32,
}

struct v4Ed {
    d2_: array<u32>,
}

@id(7) override mh: bool = true;
@id(4) override jh: bool = true;
@id(0) override fh: bool = true;
@id(1) override gh: bool = true;

@group(0) @binding(3)
var<storage> BD: Ne;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Ed;
@group(0) @binding(4)
var<storage> RB: Oe;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(3) @binding(8)
var Pb: sampler;
@group(0) @binding(0)
var<uniform> m: DC;
@group(2) @binding(3)
var<storage, read_write> v4_: v4Ed;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(1) @binding(13)
var V5_: sampler;

fn main_1() {
    var phi_682_: bool;
    var phi_988_: f32;
    var phi_987_: f32;
    var phi_989_: f32;
    var phi_992_: f32;
    var phi_991_: f32;
    var phi_719_: bool;
    var phi_1005_: f32;
    var phi_993_: f32;
    var phi_1007_: vec4<f32>;
    var phi_1009_: vec3<f32>;

    let _e51 = gl_FragCoord_1;
    let _e52 = _e51.xy;
    let _e55 = bitcast<vec2<u32>>(vec2<i32>(floor(_e52)));
    let _e57 = m.p6_;
    let _e86 = bitcast<i32>((((((_e55.y >> bitcast<u32>(5u)) * (((_e57 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e55.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e55.x & 28u) << bitcast<u32>(5u)) + ((_e55.y & 28u) << bitcast<u32>(2i)))) + (((_e55.y & 3u) << bitcast<u32>(2i)) + (_e55.x & 3u))));
    let _e89 = v4_.d2_[_e86];
    let _e93 = ((f32((_e89 & 131071u)) * 0.00048828125f) + -32f);
    let _e95 = (_e89 >> bitcast<u32>(17u));
    let _e98 = BD.d2_[_e95];
    phi_987_ = _e93;
    if ((_e98.x & 768u) != 0u) {
        let _e102 = abs(_e93);
        phi_682_ = jh;
        if jh {
            phi_682_ = ((_e98.x & 512u) != 0u);
        }
        let _e106 = phi_682_;
        phi_988_ = _e102;
        if _e106 {
            phi_988_ = (1f - abs(((fract((_e102 * 0.5f)) * 2f) + -1f)));
        }
        let _e114 = phi_988_;
        phi_987_ = _e114;
    }
    let _e116 = phi_987_;
    let _e117 = clamp(_e116, 0f, 1f);
    phi_991_ = _e117;
    if fh {
        let _e119 = (_e98.x >> bitcast<u32>(16u));
        phi_992_ = _e117;
        if (_e119 != 0u) {
            let _e123 = h0_.d2_[_e86];
            if (_e119 == (_e123 >> bitcast<u32>(16i))) {
                phi_989_ = min(_e117, unpack2x16float(_e123).x);
            } else {
                phi_989_ = 0f;
            }
            let _e131 = phi_989_;
            phi_992_ = _e131;
        }
        let _e133 = phi_992_;
        phi_991_ = _e133;
    }
    let _e135 = phi_991_;
    phi_719_ = gh;
    if gh {
        phi_719_ = ((_e98.x & 1024u) != 0u);
    }
    let _e139 = phi_719_;
    phi_1005_ = _e135;
    if _e139 {
        let _e140 = (_e95 * 8u);
        let _e144 = RB.d2_[(_e140 + 2u)];
        let _e155 = RB.d2_[(_e140 + 3u)];
        let _e160 = _e155.zw;
        let _e162 = ((abs(((mat2x2<f32>(vec2<f32>(_e144.x, _e144.y), vec2<f32>(_e144.z, _e144.w)) * _e52) + _e155.xy)) * _e160) - _e160);
        phi_1005_ = min(_e135, clamp((min(_e162.x, _e162.y) + 0.5f), 0f, 1f));
    }
    let _e170 = phi_1005_;
    let _e171 = (_e98.x & 15u);
    if (_e171 <= 1u) {
        phi_1007_ = select(unpack4x8unorm(_e98.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((fh && (_e171 == 0u))));
    } else {
        let _e179 = (_e95 * 8u);
        let _e182 = RB.d2_[_e179];
        let _e193 = RB.d2_[(_e179 + 1u)];
        let _e196 = ((mat2x2<f32>(vec2<f32>(_e182.x, _e182.y), vec2<f32>(_e182.z, _e182.w)) * _e52) + _e193.xy);
        if (_e171 == 2u) {
            phi_993_ = _e196.x;
        } else {
            phi_993_ = length(_e196);
        }
        let _e201 = phi_993_;
        let _e210 = textureSampleLevel(MD, Pb, vec2<f32>(((clamp(_e201, 0f, 1f) * _e193.z) + _e193.w), bitcast<f32>(_e98.y)), 0f);
        phi_1007_ = _e210;
    }
    let _e212 = phi_1007_;
    let _e214 = (_e212.w * _e170);
    let _e216 = (_e212.xyz * _e214);
    let _e220 = vec4<f32>(_e216.x, _e216.y, _e216.z, _e214);
    let _e221 = _e220.xyz;
    let _e223 = m.B3_;
    let _e225 = m.C3_;
    if (mh && (_e214 != 0f)) {
        phi_1009_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e51.x) + (0.00583715f * _e51.y))))) * _e223) + _e225)) + _e221);
    } else {
        phi_1009_ = _e221;
    }
    let _e241 = phi_1009_;
    let _e247 = vec4<f32>(_e241.x, _e220.y, _e220.z, _e220.w);
    let _e253 = vec4<f32>(_e247.x, _e241.y, _e247.z, _e247.w);
    C1_ = vec4<f32>(_e253.x, _e253.y, _e241.z, _e253.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = C1_;
    return _e3;
}
