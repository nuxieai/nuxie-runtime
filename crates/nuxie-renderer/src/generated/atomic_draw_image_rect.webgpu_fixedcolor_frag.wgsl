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
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;
var<private> X1_1: vec2<f32>;
var<private> R4_1: f32;
var<private> L0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> q4_: q4xd;
var<private> v3_1: u32;
var<private> H1_1: f32;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var Z9_: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> A1_1: u32;

fn main_1() {
    var phi_1241_: f32;
    var phi_843_: bool;
    var phi_1188_: f32;
    var phi_1187_: f32;
    var phi_1189_: f32;
    var phi_1192_: f32;
    var phi_1191_: f32;
    var phi_880_: bool;
    var phi_1194_: f32;
    var phi_1221_: u32;
    var phi_1193_: f32;
    var phi_1220_: u32;
    var phi_1218_: vec4<f32>;
    var phi_636_: bool;
    var phi_1232_: u32;
    var phi_1247_: f32;
    var phi_1248_: f32;
    var phi_1269_: vec3<f32>;

    let _e58 = gl_FragCoord_1;
    let _e59 = _e58.xy;
    let _e62 = bitcast<vec2<u32>>(vec2<i32>(floor(_e59)));
    let _e64 = m.m6_;
    let _e93 = bitcast<i32>((((((_e62.y >> bitcast<u32>(5u)) * (((_e64 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e62.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e62.x & 28u) << bitcast<u32>(5u)) + ((_e62.y & 28u) << bitcast<u32>(2i)))) + (((_e62.y & 3u) << bitcast<u32>(2i)) + (_e62.x & 3u))));
    let _e94 = X1_1;
    let _e95 = textureSample(IC, S5_, _e94);
    let _e96 = R4_1;
    let _e97 = min(_e96, 1f);
    phi_1241_ = _e97;
    if Vg {
        let _e98 = L0_1;
        let _e101 = min(_e98.xy, _e98.zw);
        phi_1241_ = clamp(min(_e101.x, _e101.y), 0f, _e97);
    }
    let _e107 = phi_1241_;
    let _e110 = q4_.c2_[_e93];
    let _e112 = (_e110 >> bitcast<u32>(17u));
    let _e116 = ((f32((_e110 & 131071u)) * 0.00048828125f) + -32f);
    let _e119 = AD.c2_[_e112];
    phi_1187_ = _e116;
    if ((_e119.x & 768u) != 0u) {
        let _e123 = abs(_e116);
        phi_843_ = Yg;
        if Yg {
            phi_843_ = ((_e119.x & 512u) != 0u);
        }
        let _e127 = phi_843_;
        phi_1188_ = _e123;
        if _e127 {
            phi_1188_ = (1f - abs(((fract((_e123 * 0.5f)) * 2f) + -1f)));
        }
        let _e135 = phi_1188_;
        phi_1187_ = _e135;
    }
    let _e137 = phi_1187_;
    let _e138 = clamp(_e137, 0f, 1f);
    phi_1191_ = _e138;
    if Ug {
        let _e140 = (_e119.x >> bitcast<u32>(16u));
        phi_1192_ = _e138;
        if (_e140 != 0u) {
            let _e144 = h0_.c2_[_e93];
            if (_e140 == (_e144 >> bitcast<u32>(16i))) {
                phi_1189_ = min(_e138, unpack2x16float(_e144).x);
            } else {
                phi_1189_ = 0f;
            }
            let _e152 = phi_1189_;
            phi_1192_ = _e152;
        }
        let _e154 = phi_1192_;
        phi_1191_ = _e154;
    }
    let _e156 = phi_1191_;
    phi_880_ = Vg;
    if Vg {
        phi_880_ = ((_e119.x & 1024u) != 0u);
    }
    let _e160 = phi_880_;
    phi_1194_ = _e156;
    if _e160 {
        let _e161 = (_e112 * 4u);
        let _e165 = RB.c2_[(_e161 + 2u)];
        let _e176 = RB.c2_[(_e161 + 3u)];
        let _e181 = _e176.zw;
        let _e183 = ((abs(((mat2x2<f32>(vec2<f32>(_e165.x, _e165.y), vec2<f32>(_e165.z, _e165.w)) * _e59) + _e176.xy)) * _e181) - _e181);
        phi_1194_ = min(_e156, clamp((min(_e183.x, _e183.y) + 0.5f), 0f, 1f));
    }
    let _e191 = phi_1194_;
    let _e192 = (_e119.x & 15u);
    if (_e192 <= 1u) {
        let _e197 = (Ug && (_e192 == 0u));
        phi_1221_ = 0u;
        if _e197 {
            phi_1221_ = (_e119.y | pack2x16float(vec2<f32>(_e191, 0f)));
        }
        let _e202 = phi_1221_;
        phi_1220_ = _e202;
        phi_1218_ = select(unpack4x8unorm(_e119.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e197));
    } else {
        let _e205 = (_e112 * 4u);
        let _e208 = RB.c2_[_e205];
        let _e219 = RB.c2_[(_e205 + 1u)];
        let _e222 = ((mat2x2<f32>(vec2<f32>(_e208.x, _e208.y), vec2<f32>(_e208.z, _e208.w)) * _e59) + _e219.xy);
        if (_e192 == 2u) {
            phi_1193_ = _e222.x;
        } else {
            phi_1193_ = length(_e222);
        }
        let _e227 = phi_1193_;
        let _e236 = textureSampleLevel(KD, Jb, vec2<f32>(((clamp(_e227, 0f, 1f) * _e219.z) + _e219.w), bitcast<f32>(_e119.y)), 0f);
        phi_1220_ = 0u;
        phi_1218_ = _e236;
    }
    let _e238 = phi_1220_;
    let _e240 = phi_1218_;
    let _e242 = (_e240.w * _e191);
    let _e244 = (_e240.xyz * _e242);
    phi_636_ = Ug;
    if Ug {
        let _e249 = v3_1;
        phi_636_ = (_e249 != 0u);
    }
    let _e252 = phi_636_;
    phi_1248_ = _e107;
    if _e252 {
        if (_e238 != 0u) {
            phi_1232_ = _e238;
        } else {
            let _e256 = h0_.c2_[_e93];
            phi_1232_ = _e256;
        }
        let _e258 = phi_1232_;
        let _e259 = v3_1;
        if (_e259 == (_e258 >> bitcast<u32>(16i))) {
            phi_1247_ = min(_e107, unpack2x16float(_e258).x);
        } else {
            phi_1247_ = 0f;
        }
        let _e267 = phi_1247_;
        phi_1248_ = _e267;
    }
    let _e269 = phi_1248_;
    let _e270 = H1_1;
    let _e272 = (_e95 * (_e269 * _e270));
    let _e276 = ((vec4<f32>(_e244.x, _e244.y, _e244.z, _e242) * (1f - _e272.w)) + _e272);
    let _e277 = _e276.xyz;
    let _e279 = m.y3_;
    let _e281 = m.z3_;
    if bh {
        phi_1269_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e58.x) + (0.00583715f * _e58.y))))) * _e279) + _e281)) + _e277);
    } else {
        phi_1269_ = _e277;
    }
    let _e295 = phi_1269_;
    let _e301 = vec4<f32>(_e295.x, _e276.y, _e276.z, _e276.w);
    let _e307 = vec4<f32>(_e301.x, _e295.y, _e301.z, _e301.w);
    C1_ = vec4<f32>(_e307.x, _e307.y, _e295.z, _e307.w);
    if (_e238 != 0u) {
        h0_.c2_[_e93] = _e238;
    }
    q4_.c2_[_e93] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) X1_: vec2<f32>, @location(1) R4_: f32, @location(2) L0_: vec4<f32>, @location(4) @interpolate(flat) v3_: u32, @location(3) @interpolate(flat) H1_: f32, @location(5) @interpolate(flat) A1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    X1_1 = X1_;
    R4_1 = R4_;
    L0_1 = L0_;
    v3_1 = v3_;
    H1_1 = H1_;
    A1_1 = A1_;
    main_1();
    let _e15 = C1_;
    return _e15;
}
