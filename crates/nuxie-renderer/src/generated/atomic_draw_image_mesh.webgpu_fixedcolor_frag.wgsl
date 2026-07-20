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
    var phi_1237_: f32;
    var phi_839_: bool;
    var phi_1184_: f32;
    var phi_1183_: f32;
    var phi_1185_: f32;
    var phi_1188_: f32;
    var phi_1187_: f32;
    var phi_876_: bool;
    var phi_1190_: f32;
    var phi_1217_: u32;
    var phi_1189_: f32;
    var phi_1216_: u32;
    var phi_1214_: vec4<f32>;
    var phi_631_: bool;
    var phi_1228_: u32;
    var phi_1243_: f32;
    var phi_1244_: f32;
    var phi_1265_: vec3<f32>;

    let _e57 = gl_FragCoord_1;
    let _e58 = _e57.xy;
    let _e61 = bitcast<vec2<u32>>(vec2<i32>(floor(_e58)));
    let _e63 = m.m6_;
    let _e92 = bitcast<i32>((((((_e61.y >> bitcast<u32>(5u)) * (((_e63 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e61.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e61.x & 28u) << bitcast<u32>(5u)) + ((_e61.y & 28u) << bitcast<u32>(2i)))) + (((_e61.y & 3u) << bitcast<u32>(2i)) + (_e61.x & 3u))));
    let _e93 = X1_1;
    let _e94 = textureSample(IC, S5_, _e93);
    phi_1237_ = 1f;
    if Vg {
        let _e95 = L0_1;
        let _e98 = min(_e95.xy, _e95.zw);
        phi_1237_ = clamp(min(_e98.x, _e98.y), 0f, 1f);
    }
    let _e104 = phi_1237_;
    let _e107 = q4_.c2_[_e92];
    let _e109 = (_e107 >> bitcast<u32>(17u));
    let _e113 = ((f32((_e107 & 131071u)) * 0.00048828125f) + -32f);
    let _e116 = AD.c2_[_e109];
    phi_1183_ = _e113;
    if ((_e116.x & 768u) != 0u) {
        let _e120 = abs(_e113);
        phi_839_ = Yg;
        if Yg {
            phi_839_ = ((_e116.x & 512u) != 0u);
        }
        let _e124 = phi_839_;
        phi_1184_ = _e120;
        if _e124 {
            phi_1184_ = (1f - abs(((fract((_e120 * 0.5f)) * 2f) + -1f)));
        }
        let _e132 = phi_1184_;
        phi_1183_ = _e132;
    }
    let _e134 = phi_1183_;
    let _e135 = clamp(_e134, 0f, 1f);
    phi_1187_ = _e135;
    if Ug {
        let _e137 = (_e116.x >> bitcast<u32>(16u));
        phi_1188_ = _e135;
        if (_e137 != 0u) {
            let _e141 = h0_.c2_[_e92];
            if (_e137 == (_e141 >> bitcast<u32>(16i))) {
                phi_1185_ = min(_e135, unpack2x16float(_e141).x);
            } else {
                phi_1185_ = 0f;
            }
            let _e149 = phi_1185_;
            phi_1188_ = _e149;
        }
        let _e151 = phi_1188_;
        phi_1187_ = _e151;
    }
    let _e153 = phi_1187_;
    phi_876_ = Vg;
    if Vg {
        phi_876_ = ((_e116.x & 1024u) != 0u);
    }
    let _e157 = phi_876_;
    phi_1190_ = _e153;
    if _e157 {
        let _e158 = (_e109 * 4u);
        let _e162 = RB.c2_[(_e158 + 2u)];
        let _e173 = RB.c2_[(_e158 + 3u)];
        let _e178 = _e173.zw;
        let _e180 = ((abs(((mat2x2<f32>(vec2<f32>(_e162.x, _e162.y), vec2<f32>(_e162.z, _e162.w)) * _e58) + _e173.xy)) * _e178) - _e178);
        phi_1190_ = min(_e153, clamp((min(_e180.x, _e180.y) + 0.5f), 0f, 1f));
    }
    let _e188 = phi_1190_;
    let _e189 = (_e116.x & 15u);
    if (_e189 <= 1u) {
        let _e194 = (Ug && (_e189 == 0u));
        phi_1217_ = 0u;
        if _e194 {
            phi_1217_ = (_e116.y | pack2x16float(vec2<f32>(_e188, 0f)));
        }
        let _e199 = phi_1217_;
        phi_1216_ = _e199;
        phi_1214_ = select(unpack4x8unorm(_e116.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e194));
    } else {
        let _e202 = (_e109 * 4u);
        let _e205 = RB.c2_[_e202];
        let _e216 = RB.c2_[(_e202 + 1u)];
        let _e219 = ((mat2x2<f32>(vec2<f32>(_e205.x, _e205.y), vec2<f32>(_e205.z, _e205.w)) * _e58) + _e216.xy);
        if (_e189 == 2u) {
            phi_1189_ = _e219.x;
        } else {
            phi_1189_ = length(_e219);
        }
        let _e224 = phi_1189_;
        let _e233 = textureSampleLevel(KD, Jb, vec2<f32>(((clamp(_e224, 0f, 1f) * _e216.z) + _e216.w), bitcast<f32>(_e116.y)), 0f);
        phi_1216_ = 0u;
        phi_1214_ = _e233;
    }
    let _e235 = phi_1216_;
    let _e237 = phi_1214_;
    let _e239 = (_e237.w * _e188);
    let _e241 = (_e237.xyz * _e239);
    phi_631_ = Ug;
    if Ug {
        let _e246 = v3_1;
        phi_631_ = (_e246 != 0u);
    }
    let _e249 = phi_631_;
    phi_1244_ = _e104;
    if _e249 {
        if (_e235 != 0u) {
            phi_1228_ = _e235;
        } else {
            let _e253 = h0_.c2_[_e92];
            phi_1228_ = _e253;
        }
        let _e255 = phi_1228_;
        let _e256 = v3_1;
        if (_e256 == (_e255 >> bitcast<u32>(16i))) {
            phi_1243_ = min(_e104, unpack2x16float(_e255).x);
        } else {
            phi_1243_ = 0f;
        }
        let _e264 = phi_1243_;
        phi_1244_ = _e264;
    }
    let _e266 = phi_1244_;
    let _e267 = H1_1;
    let _e269 = (_e94 * (_e266 * _e267));
    let _e273 = ((vec4<f32>(_e241.x, _e241.y, _e241.z, _e239) * (1f - _e269.w)) + _e269);
    let _e274 = _e273.xyz;
    let _e276 = m.y3_;
    let _e278 = m.z3_;
    if bh {
        phi_1265_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e57.x) + (0.00583715f * _e57.y))))) * _e276) + _e278)) + _e274);
    } else {
        phi_1265_ = _e274;
    }
    let _e292 = phi_1265_;
    let _e298 = vec4<f32>(_e292.x, _e273.y, _e273.z, _e273.w);
    let _e304 = vec4<f32>(_e298.x, _e292.y, _e298.z, _e298.w);
    C1_ = vec4<f32>(_e304.x, _e304.y, _e292.z, _e304.w);
    if (_e235 != 0u) {
        h0_.c2_[_e92] = _e235;
    }
    q4_.c2_[_e92] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) X1_: vec2<f32>, @location(1) L0_: vec4<f32>, @location(4) @interpolate(flat, either) v3_: u32, @location(3) @interpolate(flat, either) H1_: f32, @location(5) @interpolate(flat, either) A1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    X1_1 = X1_;
    L0_1 = L0_;
    v3_1 = v3_;
    H1_1 = H1_;
    A1_1 = A1_;
    main_1();
    let _e13 = C1_;
    return _e13;
}
