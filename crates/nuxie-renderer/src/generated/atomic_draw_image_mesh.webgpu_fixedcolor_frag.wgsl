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
    Gg: u32,
    Re: u32,
    Se: u32,
    R7_: vec4<i32>,
    Cg: vec2<f32>,
    pd: vec2<f32>,
    a2_: u32,
    Hg: f32,
    Z5_: u32,
    P2_: f32,
    qd: f32,
    Me: u32,
    z3_: f32,
    A3_: f32,
    rd: f32,
    zg: u32,
}

struct q4Bd {
    c2_: array<u32>,
}

@id(7) override gh: bool = true;
@id(4) override dh: bool = true;
@id(0) override Zg: bool = true;
@id(1) override ah: bool = true;

@group(0) @binding(3)
var<storage> AD: Je;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Bd;
@group(0) @binding(4)
var<storage> RB: Ke;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var LD: texture_2d<f32>;
@group(3) @binding(8)
var Mb: sampler;
@group(0) @binding(0)
var<uniform> n: CC;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;
var<private> X1_1: vec2<f32>;
var<private> L0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> q4_: q4Bd;
var<private> w3_1: u32;
var<private> H1_1: f32;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> A1_1: u32;

fn main_1() {
    var phi_1249_: f32;
    var phi_847_: bool;
    var phi_1196_: f32;
    var phi_1195_: f32;
    var phi_1197_: f32;
    var phi_1200_: f32;
    var phi_1199_: f32;
    var phi_884_: bool;
    var phi_1202_: f32;
    var phi_1229_: u32;
    var phi_1201_: f32;
    var phi_1228_: u32;
    var phi_1226_: vec4<f32>;
    var phi_635_: bool;
    var phi_1240_: u32;
    var phi_1255_: f32;
    var phi_1256_: f32;
    var phi_1277_: vec3<f32>;

    let _e57 = gl_FragCoord_1;
    let _e58 = _e57.xy;
    let _e61 = bitcast<vec2<u32>>(vec2<i32>(floor(_e58)));
    let _e63 = n.m6_;
    let _e92 = bitcast<i32>((((((_e61.y >> bitcast<u32>(5u)) * (((_e63 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e61.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e61.x & 28u) << bitcast<u32>(5u)) + ((_e61.y & 28u) << bitcast<u32>(2i)))) + (((_e61.y & 3u) << bitcast<u32>(2i)) + (_e61.x & 3u))));
    let _e93 = X1_1;
    let _e94 = textureSample(IC, S5_, _e93);
    phi_1249_ = 1f;
    if ah {
        let _e95 = L0_1;
        let _e98 = min(_e95.xy, _e95.zw);
        phi_1249_ = clamp(min(_e98.x, _e98.y), 0f, 1f);
    }
    let _e104 = phi_1249_;
    let _e107 = q4_.c2_[_e92];
    let _e109 = (_e107 >> bitcast<u32>(17u));
    let _e113 = ((f32((_e107 & 131071u)) * 0.00048828125f) + -32f);
    let _e116 = AD.c2_[_e109];
    phi_1195_ = _e113;
    if ((_e116.x & 768u) != 0u) {
        let _e120 = abs(_e113);
        phi_847_ = dh;
        if dh {
            phi_847_ = ((_e116.x & 512u) != 0u);
        }
        let _e124 = phi_847_;
        phi_1196_ = _e120;
        if _e124 {
            phi_1196_ = (1f - abs(((fract((_e120 * 0.5f)) * 2f) + -1f)));
        }
        let _e132 = phi_1196_;
        phi_1195_ = _e132;
    }
    let _e134 = phi_1195_;
    let _e135 = clamp(_e134, 0f, 1f);
    phi_1199_ = _e135;
    if Zg {
        let _e137 = (_e116.x >> bitcast<u32>(16u));
        phi_1200_ = _e135;
        if (_e137 != 0u) {
            let _e141 = h0_.c2_[_e92];
            if (_e137 == (_e141 >> bitcast<u32>(16i))) {
                phi_1197_ = min(_e135, unpack2x16float(_e141).x);
            } else {
                phi_1197_ = 0f;
            }
            let _e149 = phi_1197_;
            phi_1200_ = _e149;
        }
        let _e151 = phi_1200_;
        phi_1199_ = _e151;
    }
    let _e153 = phi_1199_;
    phi_884_ = ah;
    if ah {
        phi_884_ = ((_e116.x & 1024u) != 0u);
    }
    let _e157 = phi_884_;
    phi_1202_ = _e153;
    if _e157 {
        let _e158 = (_e109 * 4u);
        let _e162 = RB.c2_[(_e158 + 2u)];
        let _e173 = RB.c2_[(_e158 + 3u)];
        let _e178 = _e173.zw;
        let _e180 = ((abs(((mat2x2<f32>(vec2<f32>(_e162.x, _e162.y), vec2<f32>(_e162.z, _e162.w)) * _e58) + _e173.xy)) * _e178) - _e178);
        phi_1202_ = min(_e153, clamp((min(_e180.x, _e180.y) + 0.5f), 0f, 1f));
    }
    let _e188 = phi_1202_;
    let _e189 = (_e116.x & 15u);
    if (_e189 <= 1u) {
        let _e194 = (Zg && (_e189 == 0u));
        phi_1229_ = 0u;
        if _e194 {
            phi_1229_ = (_e116.y | pack2x16float(vec2<f32>(_e188, 0f)));
        }
        let _e199 = phi_1229_;
        phi_1228_ = _e199;
        phi_1226_ = select(unpack4x8unorm(_e116.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e194));
    } else {
        let _e202 = (_e109 * 4u);
        let _e205 = RB.c2_[_e202];
        let _e216 = RB.c2_[(_e202 + 1u)];
        let _e219 = ((mat2x2<f32>(vec2<f32>(_e205.x, _e205.y), vec2<f32>(_e205.z, _e205.w)) * _e58) + _e216.xy);
        if (_e189 == 2u) {
            phi_1201_ = _e219.x;
        } else {
            phi_1201_ = length(_e219);
        }
        let _e224 = phi_1201_;
        let _e233 = textureSampleLevel(LD, Mb, vec2<f32>(((clamp(_e224, 0f, 1f) * _e216.z) + _e216.w), bitcast<f32>(_e116.y)), 0f);
        phi_1228_ = 0u;
        phi_1226_ = _e233;
    }
    let _e235 = phi_1228_;
    let _e237 = phi_1226_;
    let _e239 = (_e237.w * _e188);
    let _e241 = (_e237.xyz * _e239);
    phi_635_ = Zg;
    if Zg {
        let _e246 = w3_1;
        phi_635_ = (_e246 != 0u);
    }
    let _e249 = phi_635_;
    phi_1256_ = _e104;
    if _e249 {
        if (_e235 != 0u) {
            phi_1240_ = _e235;
        } else {
            let _e253 = h0_.c2_[_e92];
            phi_1240_ = _e253;
        }
        let _e255 = phi_1240_;
        let _e256 = w3_1;
        if (_e256 == (_e255 >> bitcast<u32>(16i))) {
            phi_1255_ = min(_e104, unpack2x16float(_e255).x);
        } else {
            phi_1255_ = 0f;
        }
        let _e264 = phi_1255_;
        phi_1256_ = _e264;
    }
    let _e266 = phi_1256_;
    let _e267 = H1_1;
    let _e269 = (_e94 * (_e266 * _e267));
    let _e273 = ((vec4<f32>(_e241.x, _e241.y, _e241.z, _e239) * (1f - _e269.w)) + _e269);
    let _e274 = _e273.xyz;
    let _e277 = n.z3_;
    let _e279 = n.A3_;
    if (gh && (_e273.w != 0f)) {
        phi_1277_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e57.x) + (0.00583715f * _e57.y))))) * _e277) + _e279)) + _e274);
    } else {
        phi_1277_ = _e274;
    }
    let _e295 = phi_1277_;
    let _e301 = vec4<f32>(_e295.x, _e273.y, _e273.z, _e273.w);
    let _e307 = vec4<f32>(_e301.x, _e295.y, _e301.z, _e301.w);
    C1_ = vec4<f32>(_e307.x, _e307.y, _e295.z, _e307.w);
    if (_e235 != 0u) {
        h0_.c2_[_e92] = _e235;
    }
    q4_.c2_[_e92] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) X1_: vec2<f32>, @location(1) L0_: vec4<f32>, @location(4) @interpolate(flat, either) w3_: u32, @location(3) @interpolate(flat, either) H1_: f32, @location(5) @interpolate(flat, either) A1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    X1_1 = X1_;
    L0_1 = L0_;
    w3_1 = w3_;
    H1_1 = H1_;
    A1_1 = A1_;
    main_1();
    let _e13 = C1_;
    return _e13;
}
