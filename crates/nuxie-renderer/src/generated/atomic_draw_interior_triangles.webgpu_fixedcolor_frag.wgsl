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
var<private> A0_1: u32;
var<private> i1_1: f32;
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
    var phi_1117_: u32;
    var phi_782_: bool;
    var phi_1122_: f32;
    var phi_1121_: f32;
    var phi_1123_: f32;
    var phi_1126_: f32;
    var phi_1125_: f32;
    var phi_819_: bool;
    var phi_1128_: f32;
    var phi_1154_: u32;
    var phi_1127_: f32;
    var phi_1153_: u32;
    var phi_1151_: vec4<f32>;
    var phi_1168_: u32;
    var phi_1164_: vec4<f32>;
    var phi_1165_: vec3<f32>;

    let _e55 = gl_FragCoord_1;
    let _e56 = _e55.xy;
    let _e59 = bitcast<vec2<u32>>(vec2<i32>(floor(_e56)));
    let _e61 = m.m6_;
    let _e90 = bitcast<i32>((((((_e59.y >> bitcast<u32>(5u)) * (((_e61 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e59.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e59.x & 28u) << bitcast<u32>(5u)) + ((_e59.y & 28u) << bitcast<u32>(2i)))) + (((_e59.y & 3u) << bitcast<u32>(2i)) + (_e59.x & 3u))));
    let _e93 = q4_.c2_[_e90];
    let _e95 = (_e93 >> bitcast<u32>(17u));
    let _e96 = A0_1;
    if (_e95 == _e96) {
        phi_1117_ = _e93;
    } else {
        phi_1117_ = ((_e96 << bitcast<u32>(17u)) + 65536u);
    }
    let _e102 = phi_1117_;
    let _e103 = i1_1;
    q4_.c2_[_e90] = (_e102 + bitcast<u32>(i32(round((_e103 * 2048f)))));
    phi_1168_ = 0u;
    phi_1164_ = vec4<f32>(0f, 0f, 0f, 0f);
    if (_e95 != _e96) {
        let _e113 = ((f32((_e93 & 131071u)) * 0.00048828125f) + -32f);
        let _e116 = AD.c2_[_e95];
        phi_1121_ = _e113;
        if ((_e116.x & 768u) != 0u) {
            let _e120 = abs(_e113);
            phi_782_ = Yg;
            if Yg {
                phi_782_ = ((_e116.x & 512u) != 0u);
            }
            let _e124 = phi_782_;
            phi_1122_ = _e120;
            if _e124 {
                phi_1122_ = (1f - abs(((fract((_e120 * 0.5f)) * 2f) + -1f)));
            }
            let _e132 = phi_1122_;
            phi_1121_ = _e132;
        }
        let _e134 = phi_1121_;
        let _e135 = clamp(_e134, 0f, 1f);
        phi_1125_ = _e135;
        if Ug {
            let _e137 = (_e116.x >> bitcast<u32>(16u));
            phi_1126_ = _e135;
            if (_e137 != 0u) {
                let _e141 = h0_.c2_[_e90];
                if (_e137 == (_e141 >> bitcast<u32>(16i))) {
                    phi_1123_ = min(_e135, unpack2x16float(_e141).x);
                } else {
                    phi_1123_ = 0f;
                }
                let _e149 = phi_1123_;
                phi_1126_ = _e149;
            }
            let _e151 = phi_1126_;
            phi_1125_ = _e151;
        }
        let _e153 = phi_1125_;
        phi_819_ = Vg;
        if Vg {
            phi_819_ = ((_e116.x & 1024u) != 0u);
        }
        let _e157 = phi_819_;
        phi_1128_ = _e153;
        if _e157 {
            let _e158 = (_e95 * 4u);
            let _e162 = RB.c2_[(_e158 + 2u)];
            let _e173 = RB.c2_[(_e158 + 3u)];
            let _e178 = _e173.zw;
            let _e180 = ((abs(((mat2x2<f32>(vec2<f32>(_e162.x, _e162.y), vec2<f32>(_e162.z, _e162.w)) * _e56) + _e173.xy)) * _e178) - _e178);
            phi_1128_ = min(_e153, clamp((min(_e180.x, _e180.y) + 0.5f), 0f, 1f));
        }
        let _e188 = phi_1128_;
        let _e189 = (_e116.x & 15u);
        if (_e189 <= 1u) {
            let _e194 = (Ug && (_e189 == 0u));
            phi_1154_ = 0u;
            if _e194 {
                phi_1154_ = (_e116.y | pack2x16float(vec2<f32>(_e188, 0f)));
            }
            let _e199 = phi_1154_;
            phi_1153_ = _e199;
            phi_1151_ = select(unpack4x8unorm(_e116.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e194));
        } else {
            let _e202 = (_e95 * 4u);
            let _e205 = RB.c2_[_e202];
            let _e216 = RB.c2_[(_e202 + 1u)];
            let _e219 = ((mat2x2<f32>(vec2<f32>(_e205.x, _e205.y), vec2<f32>(_e205.z, _e205.w)) * _e56) + _e216.xy);
            if (_e189 == 2u) {
                phi_1127_ = _e219.x;
            } else {
                phi_1127_ = length(_e219);
            }
            let _e224 = phi_1127_;
            let _e233 = textureSampleLevel(KD, Jb, vec2<f32>(((clamp(_e224, 0f, 1f) * _e216.z) + _e216.w), bitcast<f32>(_e116.y)), 0f);
            phi_1153_ = 0u;
            phi_1151_ = _e233;
        }
        let _e235 = phi_1153_;
        let _e237 = phi_1151_;
        let _e239 = (_e237.w * _e188);
        let _e241 = (_e237.xyz * _e239);
        phi_1168_ = _e235;
        phi_1164_ = vec4<f32>(_e241.x, _e241.y, _e241.z, _e239);
    }
    let _e247 = phi_1168_;
    let _e249 = phi_1164_;
    let _e250 = _e249.xyz;
    let _e252 = m.y3_;
    let _e254 = m.z3_;
    if bh {
        phi_1165_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e55.x) + (0.00583715f * _e55.y))))) * _e252) + _e254)) + _e250);
    } else {
        phi_1165_ = _e250;
    }
    let _e268 = phi_1165_;
    let _e274 = vec4<f32>(_e268.x, _e249.y, _e249.z, _e249.w);
    let _e280 = vec4<f32>(_e274.x, _e268.y, _e274.z, _e274.w);
    C1_ = vec4<f32>(_e280.x, _e280.y, _e268.z, _e280.w);
    if (_e247 != 0u) {
        h0_.c2_[_e90] = _e247;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat) A0_: u32, @location(0) @interpolate(flat) i1_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    A0_1 = A0_;
    i1_1 = i1_;
    main_1();
    let _e7 = C1_;
    return _e7;
}
