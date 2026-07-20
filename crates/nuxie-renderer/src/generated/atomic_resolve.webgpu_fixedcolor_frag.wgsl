struct Fe {
    c2_: array<vec2<u32>>,
}

struct h0xd {
    c2_: array<u32>,
}

struct Ge {
    c2_: array<vec4<f32>>,
}

struct CC {
    bc: f32,
    kd: f32,
    bf: f32,
    cf: f32,
    m6_: u32,
    Bg: u32,
    Ne: u32,
    Oe: u32,
    Q7_: vec4<i32>,
    xg: vec2<f32>,
    ld: vec2<f32>,
    a2_: u32,
    Cg: f32,
    Z5_: u32,
    N2_: f32,
    md: f32,
    Ie: u32,
    y3_: f32,
    z3_: f32,
    nd: f32,
    ug: u32,
}

struct q4xd {
    c2_: array<u32>,
}

@id(7) override bh: bool = true;
@id(4) override Yg: bool = true;
@id(0) override Ug: bool = true;
@id(1) override Vg: bool = true;

@group(0) @binding(3)
var<storage> AD: Fe;
@group(2) @binding(1)
var<storage, read_write> h0_: h0xd;
@group(0) @binding(4)
var<storage> RB: Ge;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(3) @binding(8)
var Jb: sampler;
@group(0) @binding(0)
var<uniform> m: CC;
@group(2) @binding(3)
var<storage, read_write> q4_: q4xd;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var Z9_: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;

fn main_1() {
    var phi_673_: bool;
    var phi_975_: f32;
    var phi_974_: f32;
    var phi_976_: f32;
    var phi_979_: f32;
    var phi_978_: f32;
    var phi_710_: bool;
    var phi_992_: f32;
    var phi_980_: f32;
    var phi_994_: vec4<f32>;
    var phi_996_: vec3<f32>;

    let _e51 = gl_FragCoord_1;
    let _e52 = _e51.xy;
    let _e55 = bitcast<vec2<u32>>(vec2<i32>(floor(_e52)));
    let _e57 = m.m6_;
    let _e86 = bitcast<i32>((((((_e55.y >> bitcast<u32>(5u)) * (((_e57 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e55.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e55.x & 28u) << bitcast<u32>(5u)) + ((_e55.y & 28u) << bitcast<u32>(2i)))) + (((_e55.y & 3u) << bitcast<u32>(2i)) + (_e55.x & 3u))));
    let _e89 = q4_.c2_[_e86];
    let _e93 = ((f32((_e89 & 131071u)) * 0.00048828125f) + -32f);
    let _e95 = (_e89 >> bitcast<u32>(17u));
    let _e98 = AD.c2_[_e95];
    phi_974_ = _e93;
    if ((_e98.x & 768u) != 0u) {
        let _e102 = abs(_e93);
        phi_673_ = Yg;
        if Yg {
            phi_673_ = ((_e98.x & 512u) != 0u);
        }
        let _e106 = phi_673_;
        phi_975_ = _e102;
        if _e106 {
            phi_975_ = (1f - abs(((fract((_e102 * 0.5f)) * 2f) + -1f)));
        }
        let _e114 = phi_975_;
        phi_974_ = _e114;
    }
    let _e116 = phi_974_;
    let _e117 = clamp(_e116, 0f, 1f);
    phi_978_ = _e117;
    if Ug {
        let _e119 = (_e98.x >> bitcast<u32>(16u));
        phi_979_ = _e117;
        if (_e119 != 0u) {
            let _e123 = h0_.c2_[_e86];
            if (_e119 == (_e123 >> bitcast<u32>(16i))) {
                phi_976_ = min(_e117, unpack2x16float(_e123).x);
            } else {
                phi_976_ = 0f;
            }
            let _e131 = phi_976_;
            phi_979_ = _e131;
        }
        let _e133 = phi_979_;
        phi_978_ = _e133;
    }
    let _e135 = phi_978_;
    phi_710_ = Vg;
    if Vg {
        phi_710_ = ((_e98.x & 1024u) != 0u);
    }
    let _e139 = phi_710_;
    phi_992_ = _e135;
    if _e139 {
        let _e140 = (_e95 * 4u);
        let _e144 = RB.c2_[(_e140 + 2u)];
        let _e155 = RB.c2_[(_e140 + 3u)];
        let _e160 = _e155.zw;
        let _e162 = ((abs(((mat2x2<f32>(vec2<f32>(_e144.x, _e144.y), vec2<f32>(_e144.z, _e144.w)) * _e52) + _e155.xy)) * _e160) - _e160);
        phi_992_ = min(_e135, clamp((min(_e162.x, _e162.y) + 0.5f), 0f, 1f));
    }
    let _e170 = phi_992_;
    let _e171 = (_e98.x & 15u);
    if (_e171 <= 1u) {
        phi_994_ = select(unpack4x8unorm(_e98.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((Ug && (_e171 == 0u))));
    } else {
        let _e179 = (_e95 * 4u);
        let _e182 = RB.c2_[_e179];
        let _e193 = RB.c2_[(_e179 + 1u)];
        let _e196 = ((mat2x2<f32>(vec2<f32>(_e182.x, _e182.y), vec2<f32>(_e182.z, _e182.w)) * _e52) + _e193.xy);
        if (_e171 == 2u) {
            phi_980_ = _e196.x;
        } else {
            phi_980_ = length(_e196);
        }
        let _e201 = phi_980_;
        let _e210 = textureSampleLevel(KD, Jb, vec2<f32>(((clamp(_e201, 0f, 1f) * _e193.z) + _e193.w), bitcast<f32>(_e98.y)), 0f);
        phi_994_ = _e210;
    }
    let _e212 = phi_994_;
    let _e214 = (_e212.w * _e170);
    let _e216 = (_e212.xyz * _e214);
    let _e220 = vec4<f32>(_e216.x, _e216.y, _e216.z, _e214);
    let _e221 = _e220.xyz;
    let _e223 = m.y3_;
    let _e225 = m.z3_;
    if bh {
        phi_996_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e51.x) + (0.00583715f * _e51.y))))) * _e223) + _e225)) + _e221);
    } else {
        phi_996_ = _e221;
    }
    let _e239 = phi_996_;
    let _e245 = vec4<f32>(_e239.x, _e220.y, _e220.z, _e220.w);
    let _e251 = vec4<f32>(_e245.x, _e239.y, _e245.z, _e245.w);
    C1_ = vec4<f32>(_e251.x, _e251.y, _e239.z, _e251.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = C1_;
    return _e3;
}
