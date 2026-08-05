struct Je {
    c2_: array<vec2<u32>>,
}

struct h0Bd {
    c2_: array<u32>,
}

struct Ke {
    c2_: array<vec4<f32>>,
}

struct CC {
    ec: f32,
    od: f32,
    ff: f32,
    gf: f32,
    m6_: u32,
    Fg: u32,
    Re: u32,
    Se: u32,
    R7_: vec4<i32>,
    Bg: vec2<f32>,
    pd: vec2<f32>,
    a2_: u32,
    Gg: f32,
    Z5_: u32,
    P2_: f32,
    qd: f32,
    Me: u32,
    z3_: f32,
    A3_: f32,
    rd: f32,
    yg: u32,
}

struct q4Bd {
    c2_: array<u32>,
}

@id(7) override fh: bool = true;
@id(4) override ch: bool = true;
@id(0) override Yg: bool = true;
@id(1) override Zg: bool = true;

@group(0) @binding(3)
var<storage> AD: Je;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Bd;
@group(0) @binding(4)
var<storage> RB: Ke;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(3) @binding(8)
var Mb: sampler;
@group(0) @binding(0)
var<uniform> n: CC;
@group(2) @binding(3)
var<storage, read_write> q4_: q4Bd;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;

fn main_1() {
    var phi_680_: bool;
    var phi_986_: f32;
    var phi_985_: f32;
    var phi_987_: f32;
    var phi_990_: f32;
    var phi_989_: f32;
    var phi_717_: bool;
    var phi_1003_: f32;
    var phi_991_: f32;
    var phi_1005_: vec4<f32>;
    var phi_1007_: vec3<f32>;

    let _e51 = gl_FragCoord_1;
    let _e52 = _e51.xy;
    let _e55 = bitcast<vec2<u32>>(vec2<i32>(floor(_e52)));
    let _e57 = n.m6_;
    let _e86 = bitcast<i32>((((((_e55.y >> bitcast<u32>(5u)) * (((_e57 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e55.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e55.x & 28u) << bitcast<u32>(5u)) + ((_e55.y & 28u) << bitcast<u32>(2i)))) + (((_e55.y & 3u) << bitcast<u32>(2i)) + (_e55.x & 3u))));
    let _e89 = q4_.c2_[_e86];
    let _e93 = ((f32((_e89 & 131071u)) * 0.00048828125f) + -32f);
    let _e95 = (_e89 >> bitcast<u32>(17u));
    let _e98 = AD.c2_[_e95];
    phi_985_ = _e93;
    if ((_e98.x & 768u) != 0u) {
        let _e102 = abs(_e93);
        phi_680_ = ch;
        if ch {
            phi_680_ = ((_e98.x & 512u) != 0u);
        }
        let _e106 = phi_680_;
        phi_986_ = _e102;
        if _e106 {
            phi_986_ = (1f - abs(((fract((_e102 * 0.5f)) * 2f) + -1f)));
        }
        let _e114 = phi_986_;
        phi_985_ = _e114;
    }
    let _e116 = phi_985_;
    let _e117 = clamp(_e116, 0f, 1f);
    phi_989_ = _e117;
    if Yg {
        let _e119 = (_e98.x >> bitcast<u32>(16u));
        phi_990_ = _e117;
        if (_e119 != 0u) {
            let _e123 = h0_.c2_[_e86];
            if (_e119 == (_e123 >> bitcast<u32>(16i))) {
                phi_987_ = min(_e117, unpack2x16float(_e123).x);
            } else {
                phi_987_ = 0f;
            }
            let _e131 = phi_987_;
            phi_990_ = _e131;
        }
        let _e133 = phi_990_;
        phi_989_ = _e133;
    }
    let _e135 = phi_989_;
    phi_717_ = Zg;
    if Zg {
        phi_717_ = ((_e98.x & 1024u) != 0u);
    }
    let _e139 = phi_717_;
    phi_1003_ = _e135;
    if _e139 {
        let _e140 = (_e95 * 4u);
        let _e144 = RB.c2_[(_e140 + 2u)];
        let _e155 = RB.c2_[(_e140 + 3u)];
        let _e160 = _e155.zw;
        let _e162 = ((abs(((mat2x2<f32>(vec2<f32>(_e144.x, _e144.y), vec2<f32>(_e144.z, _e144.w)) * _e52) + _e155.xy)) * _e160) - _e160);
        phi_1003_ = min(_e135, clamp((min(_e162.x, _e162.y) + 0.5f), 0f, 1f));
    }
    let _e170 = phi_1003_;
    let _e171 = (_e98.x & 15u);
    if (_e171 <= 1u) {
        phi_1005_ = select(unpack4x8unorm(_e98.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((Yg && (_e171 == 0u))));
    } else {
        let _e179 = (_e95 * 4u);
        let _e182 = RB.c2_[_e179];
        let _e193 = RB.c2_[(_e179 + 1u)];
        let _e196 = ((mat2x2<f32>(vec2<f32>(_e182.x, _e182.y), vec2<f32>(_e182.z, _e182.w)) * _e52) + _e193.xy);
        if (_e171 == 2u) {
            phi_991_ = _e196.x;
        } else {
            phi_991_ = length(_e196);
        }
        let _e201 = phi_991_;
        let _e210 = textureSampleLevel(KD, Mb, vec2<f32>(((clamp(_e201, 0f, 1f) * _e193.z) + _e193.w), bitcast<f32>(_e98.y)), 0f);
        phi_1005_ = _e210;
    }
    let _e212 = phi_1005_;
    let _e214 = (_e212.w * _e170);
    let _e216 = (_e212.xyz * _e214);
    let _e220 = vec4<f32>(_e216.x, _e216.y, _e216.z, _e214);
    let _e221 = _e220.xyz;
    let _e223 = n.z3_;
    let _e225 = n.A3_;
    if (fh && (_e214 != 0f)) {
        phi_1007_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e51.x) + (0.00583715f * _e51.y))))) * _e223) + _e225)) + _e221);
    } else {
        phi_1007_ = _e221;
    }
    let _e241 = phi_1007_;
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
