struct jg {
    v2_: array<vec2<u32>>,
}

struct m0Pe {
    v2_: array<u32>,
}

struct kg {
    v2_: array<vec4<f32>>,
}

struct VB {
    vd: f32,
    Ce: f32,
    Gg: f32,
    Hg: f32,
    L6_: u32,
    xa: u32,
    sg: u32,
    tg: u32,
    C8_: vec4<i32>,
    Bi: vec2<f32>,
    De: vec2<f32>,
    r2_: u32,
    Fi: f32,
    p6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    E3_: f32,
    F3_: f32,
    Fe: f32,
    yi: u32,
    wa: u32,
    cd: f32,
    g7_: f32,
    Db: f32,
}

struct P4Pe {
    v2_: array<u32>,
}

@id(7) override fj: bool = true;
@id(4) override cj: bool = true;
@id(0) override Yi: bool = true;
@id(1) override Zi: bool = true;
@id(2) override aj: bool = true;

@group(0) @binding(3)
var<storage> WC: jg;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Pe;
@group(0) @binding(4)
var<storage> JB: kg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var I8_: sampler;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var S4_: sampler;
var<private> m2_1: vec2<f32>;
var<private> o5_1: f32;
var<private> W0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> P4_: P4Pe;
var<private> R3_1: u32;
var<private> p5_1: vec4<f32>;
var<private> U1_1: vec4<f32>;
var<private> N1_: vec4<f32>;
@group(3) @binding(9)
var Va: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
var<private> K1_1: u32;

fn main_1() {
    var phi_1474_: f32;
    var phi_977_: bool;
    var phi_1418_: f32;
    var phi_1417_: f32;
    var phi_1419_: f32;
    var phi_1422_: f32;
    var phi_1421_: f32;
    var phi_1014_: bool;
    var phi_1424_: f32;
    var phi_1453_: u32;
    var phi_1423_: f32;
    var phi_1451_: vec4<f32>;
    var phi_1452_: u32;
    var phi_1449_: vec4<f32>;
    var phi_729_: bool;
    var phi_1465_: u32;
    var phi_1481_: f32;
    var phi_1505_: f32;
    var phi_1482_: f32;
    var phi_1483_: f32;
    var phi_1503_: vec4<f32>;
    var phi_1515_: vec3<f32>;

    let _e66 = gl_FragCoord_1;
    let _e67 = _e66.xy;
    let _e70 = bitcast<vec2<u32>>(vec2<i32>(floor(_e67)));
    let _e72 = j.L6_;
    let _e101 = bitcast<i32>((((((_e70.y >> bitcast<u32>(5u)) * (((_e72 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e70.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e70.x & 28u) << bitcast<u32>(5u)) + ((_e70.y & 28u) << bitcast<u32>(2i)))) + (((_e70.y & 3u) << bitcast<u32>(2i)) + (_e70.x & 3u))));
    let _e102 = m2_1;
    let _e103 = textureSample(TB, S4_, _e102);
    let _e104 = o5_1;
    let _e105 = min(_e104, 1f);
    phi_1474_ = _e105;
    if Zi {
        let _e106 = W0_1;
        let _e109 = min(_e106.xy, _e106.zw);
        phi_1474_ = clamp(min(_e109.x, _e109.y), 0f, _e105);
    }
    let _e115 = phi_1474_;
    let _e118 = P4_.v2_[_e101];
    let _e120 = (_e118 >> bitcast<u32>(17u));
    let _e124 = ((f32((_e118 & 131071u)) * 0.00048828125f) + -32f);
    let _e127 = WC.v2_[_e120];
    phi_1417_ = _e124;
    if ((_e127.x & 768u) != 0u) {
        let _e131 = abs(_e124);
        phi_977_ = cj;
        if cj {
            phi_977_ = ((_e127.x & 512u) != 0u);
        }
        let _e135 = phi_977_;
        phi_1418_ = _e131;
        if _e135 {
            phi_1418_ = (1f - abs(((fract((_e131 * 0.5f)) * 2f) + -1f)));
        }
        let _e143 = phi_1418_;
        phi_1417_ = _e143;
    }
    let _e145 = phi_1417_;
    let _e146 = clamp(_e145, 0f, 1f);
    phi_1421_ = _e146;
    if Yi {
        let _e148 = (_e127.x >> bitcast<u32>(16u));
        phi_1422_ = _e146;
        if (_e148 != 0u) {
            let _e152 = m0_.v2_[_e101];
            if (_e148 == (_e152 >> bitcast<u32>(16i))) {
                phi_1419_ = min(_e146, unpack2x16float(_e152).x);
            } else {
                phi_1419_ = 0f;
            }
            let _e160 = phi_1419_;
            phi_1422_ = _e160;
        }
        let _e162 = phi_1422_;
        phi_1421_ = _e162;
    }
    let _e164 = phi_1421_;
    phi_1014_ = Zi;
    if Zi {
        phi_1014_ = ((_e127.x & 1024u) != 0u);
    }
    let _e168 = phi_1014_;
    phi_1424_ = _e164;
    if _e168 {
        let _e169 = (_e120 * 8u);
        let _e173 = JB.v2_[(_e169 + 2u)];
        let _e184 = JB.v2_[(_e169 + 3u)];
        let _e189 = _e184.zw;
        let _e191 = ((abs(((mat2x2<f32>(vec2<f32>(_e173.x, _e173.y), vec2<f32>(_e173.z, _e173.w)) * _e67) + _e184.xy)) * _e189) - _e189);
        phi_1424_ = min(_e164, clamp((min(_e191.x, _e191.y) + 0.5f), 0f, 1f));
    }
    let _e199 = phi_1424_;
    let _e200 = (_e127.x & 15u);
    if (_e200 <= 1u) {
        let _e210 = (Yi && (_e200 == 0u));
        phi_1453_ = 0u;
        if _e210 {
            phi_1453_ = (_e127.y | pack2x16float(vec2<f32>(_e199, 0f)));
        }
        let _e215 = phi_1453_;
        phi_1452_ = _e215;
        phi_1449_ = select(unpack4x8unorm(_e127.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e210));
    } else {
        let _e218 = (_e120 * 8u);
        let _e221 = JB.v2_[_e218];
        let _e232 = JB.v2_[(_e218 + 1u)];
        let _e235 = ((mat2x2<f32>(vec2<f32>(_e221.x, _e221.y), vec2<f32>(_e221.z, _e221.w)) * _e67) + _e232.xy);
        if (_e200 == 2u) {
            phi_1423_ = _e235.x;
        } else {
            phi_1423_ = length(_e235);
        }
        let _e240 = phi_1423_;
        let _e247 = bitcast<f32>(_e127.y);
        let _e250 = j.cd;
        let _e253 = j.g7_;
        let _e256 = textureSampleLevel(YC, I8_, vec2<f32>(((clamp(_e240, 0f, 1f) * _e232.z) + _e232.w), ((floor(_e247) * _e250) + _e253)), 0f);
        phi_1451_ = _e256;
        if !((aj && (((_e127.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e260 = (_e256.xyz * _e256.w);
            phi_1451_ = vec4<f32>(_e260.x, _e260.y, _e260.z, (_e256.w * (fract(_e247) * 1.0039216f)));
        }
        let _e269 = phi_1451_;
        phi_1452_ = 0u;
        phi_1449_ = _e269;
    }
    let _e271 = phi_1452_;
    let _e273 = phi_1449_;
    phi_729_ = Yi;
    if Yi {
        let _e275 = R3_1;
        phi_729_ = (_e275 != 0u);
    }
    let _e278 = phi_729_;
    phi_1505_ = _e115;
    if _e278 {
        if (_e271 != 0u) {
            phi_1465_ = _e271;
        } else {
            let _e282 = m0_.v2_[_e101];
            phi_1465_ = _e282;
        }
        let _e284 = phi_1465_;
        let _e285 = R3_1;
        if (_e285 == (_e284 >> bitcast<u32>(16i))) {
            phi_1481_ = min(_e115, unpack2x16float(_e284).x);
        } else {
            phi_1481_ = 0f;
        }
        let _e293 = phi_1481_;
        phi_1505_ = _e293;
    }
    let _e295 = phi_1505_;
    let _e297 = p5_1[3u];
    phi_1503_ = _e103;
    if (_e297 != 0f) {
        let _e299 = p5_1;
        if (_e299.z > 0f) {
            phi_1482_ = _e299.x;
        } else {
            phi_1482_ = length(_e299.xy);
        }
        let _e306 = phi_1482_;
        let _e307 = clamp(_e306, 0f, 1f);
        let _e308 = abs(_e299.z);
        if (_e308 > 1f) {
            phi_1483_ = ((0.9980469f * _e307) + 0.0009765625f);
        } else {
            phi_1483_ = ((0.001953125f * _e307) + _e308);
        }
        let _e315 = phi_1483_;
        let _e318 = textureSampleLevel(YC, I8_, vec2<f32>(_e315, _e299.w), 0f);
        let _e321 = (_e318.xyz * _e318.w);
        let _e327 = vec4<f32>(_e321.x, _e318.y, _e318.z, _e318.w);
        let _e333 = vec4<f32>(_e327.x, _e321.y, _e327.z, _e327.w);
        phi_1503_ = (_e103 * vec4<f32>(_e333.x, _e333.y, _e321.z, _e333.w));
    }
    let _e342 = phi_1503_;
    let _e343 = U1_1;
    let _e345 = ((_e342 * _e343) * _e295);
    let _e349 = (((_e273 * _e199) * (1f - _e345.w)) + _e345);
    let _e350 = _e349.xyz;
    let _e353 = j.E3_;
    let _e355 = j.F3_;
    if (fj && (_e349.w != 0f)) {
        phi_1515_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e66.x) + (0.00583715f * _e66.y))))) * _e353) + _e355)) + _e350);
    } else {
        phi_1515_ = _e350;
    }
    let _e371 = phi_1515_;
    let _e377 = vec4<f32>(_e371.x, _e349.y, _e349.z, _e349.w);
    let _e383 = vec4<f32>(_e377.x, _e371.y, _e377.z, _e377.w);
    N1_ = vec4<f32>(_e383.x, _e383.y, _e371.z, _e383.w);
    if (_e271 != 0u) {
        m0_.v2_[_e101] = _e271;
    }
    P4_.v2_[_e101] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) m2_: vec2<f32>, @location(1) o5_: f32, @location(3) W0_: vec4<f32>, @location(5) @interpolate(flat, either) R3_: u32, @location(2) p5_: vec4<f32>, @location(4) @interpolate(flat, either) U1_: vec4<f32>, @location(6) @interpolate(flat, either) K1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    m2_1 = m2_;
    o5_1 = o5_;
    W0_1 = W0_;
    R3_1 = R3_;
    p5_1 = p5_;
    U1_1 = U1_;
    K1_1 = K1_;
    main_1();
    let _e17 = N1_;
    return _e17;
}
