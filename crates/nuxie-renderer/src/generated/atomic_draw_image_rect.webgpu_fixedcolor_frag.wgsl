struct lg {
    r2_: array<vec2<u32>>,
}

struct m0Re {
    r2_: array<u32>,
}

struct mg {
    r2_: array<vec4<f32>>,
}

struct VB {
    yd: f32,
    Ee: f32,
    Jg: f32,
    Kg: f32,
    S6_: u32,
    Ea: u32,
    vg: u32,
    wg: u32,
    H8_: vec4<i32>,
    Bi: vec2<f32>,
    Fe: vec2<f32>,
    q2_: u32,
    Fi: f32,
    w6_: u32,
    i3_: f32,
    Ge: f32,
    og: u32,
    F3_: f32,
    G3_: f32,
    He: f32,
    yi: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

struct R4Re {
    r2_: array<u32>,
}

@id(7) override gj: bool = true;
@id(4) override dj: bool = true;
@id(0) override Zi: bool = true;
@id(1) override aj: bool = true;
@id(2) override bj: bool = true;

@group(0) @binding(3)
var<storage> XC: lg;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Re;
@group(0) @binding(4)
var<storage> JB: mg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var Q8_: sampler;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;
var<private> l2_1: vec2<f32>;
var<private> r5_1: f32;
var<private> V0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> R4_: R4Re;
var<private> S3_1: u32;
var<private> v5_1: vec4<f32>;
var<private> V1_1: vec4<f32>;
var<private> N1_: vec4<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
var<private> L1_1: u32;

fn main_1() {
    var phi_1788_: f32;
    var phi_1101_: bool;
    var phi_1726_: f32;
    var phi_1725_: f32;
    var phi_1727_: f32;
    var phi_1730_: f32;
    var phi_1729_: f32;
    var phi_1138_: bool;
    var phi_1734_: f32;
    var phi_1731_: f32;
    var phi_1732_: f32;
    var phi_1733_: vec2<f32>;
    var phi_1763_: vec4<f32>;
    var phi_1765_: u32;
    var phi_1764_: u32;
    var phi_1761_: vec4<f32>;
    var phi_837_: bool;
    var phi_1779_: u32;
    var phi_1797_: f32;
    var phi_1844_: f32;
    var phi_1817_: f32;
    var phi_1818_: f32;
    var phi_1819_: vec2<f32>;
    var phi_1842_: vec4<f32>;
    var phi_1856_: vec3<f32>;

    let _e68 = gl_FragCoord_1;
    let _e69 = _e68.xy;
    let _e72 = bitcast<vec2<u32>>(vec2<i32>(floor(_e69)));
    let _e74 = j.S6_;
    let _e103 = bitcast<i32>((((((_e72.y >> bitcast<u32>(5u)) * (((_e74 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e72.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e72.x & 28u) << bitcast<u32>(5u)) + ((_e72.y & 28u) << bitcast<u32>(2i)))) + (((_e72.y & 3u) << bitcast<u32>(2i)) + (_e72.x & 3u))));
    let _e104 = l2_1;
    let _e105 = textureSample(TB, U4_, _e104);
    let _e106 = r5_1;
    let _e107 = min(_e106, 1f);
    phi_1788_ = _e107;
    if aj {
        let _e108 = V0_1;
        let _e111 = min(_e108.xy, _e108.zw);
        phi_1788_ = clamp(min(_e111.x, _e111.y), 0f, _e107);
    }
    let _e117 = phi_1788_;
    let _e120 = R4_.r2_[_e103];
    let _e122 = (_e120 >> bitcast<u32>(17u));
    let _e126 = ((f32((_e120 & 131071u)) * 0.00048828125f) + -32f);
    let _e129 = XC.r2_[_e122];
    phi_1725_ = _e126;
    if ((_e129.x & 768u) != 0u) {
        let _e133 = abs(_e126);
        phi_1101_ = dj;
        if dj {
            phi_1101_ = ((_e129.x & 512u) != 0u);
        }
        let _e137 = phi_1101_;
        phi_1726_ = _e133;
        if _e137 {
            phi_1726_ = (1f - abs(((fract((_e133 * 0.5f)) * 2f) + -1f)));
        }
        let _e145 = phi_1726_;
        phi_1725_ = _e145;
    }
    let _e147 = phi_1725_;
    let _e148 = clamp(_e147, 0f, 1f);
    phi_1729_ = _e148;
    if Zi {
        let _e150 = (_e129.x >> bitcast<u32>(16u));
        phi_1730_ = _e148;
        if (_e150 != 0u) {
            let _e154 = m0_.r2_[_e103];
            if (_e150 == (_e154 >> bitcast<u32>(16i))) {
                phi_1727_ = min(_e148, unpack2x16float(_e154).x);
            } else {
                phi_1727_ = 0f;
            }
            let _e162 = phi_1727_;
            phi_1730_ = _e162;
        }
        let _e164 = phi_1730_;
        phi_1729_ = _e164;
    }
    let _e166 = phi_1729_;
    phi_1138_ = aj;
    if aj {
        phi_1138_ = ((_e129.x & 1024u) != 0u);
    }
    let _e170 = phi_1138_;
    phi_1734_ = _e166;
    if _e170 {
        let _e171 = (_e122 * 8u);
        let _e175 = JB.r2_[(_e171 + 2u)];
        let _e186 = JB.r2_[(_e171 + 3u)];
        let _e191 = _e186.zw;
        let _e193 = ((abs(((mat2x2<f32>(vec2<f32>(_e175.x, _e175.y), vec2<f32>(_e175.z, _e175.w)) * _e69) + _e186.xy)) * _e191) - _e191);
        phi_1734_ = min(_e166, clamp((min(_e193.x, _e193.y) + 0.5f), 0f, 1f));
    }
    let _e201 = phi_1734_;
    let _e202 = (_e129.x & 15u);
    if (_e202 <= 1u) {
        let _e287 = (Zi && (_e202 == 0u));
        phi_1765_ = 0u;
        if _e287 {
            phi_1765_ = (_e129.y | pack2x16float(vec2<f32>(_e201, 0f)));
        }
        let _e292 = phi_1765_;
        phi_1764_ = _e292;
        phi_1761_ = select(unpack4x8unorm(_e129.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e287));
    } else {
        let _e209 = (_e122 * 8u);
        let _e212 = JB.r2_[_e209];
        let _e223 = JB.r2_[(_e209 + 1u)];
        let _e226 = ((mat2x2<f32>(vec2<f32>(_e212.x, _e212.y), vec2<f32>(_e212.z, _e212.w)) * _e69) + _e223.xy);
        let _e230 = f32(_e202);
        let _e232 = j.O8_;
        let _e234 = j.P8_;
        let _e239 = ((_e223.w * _e232) + _e234);
        switch bitcast<i32>(0u) {
            default: {
                if (_e230 == 4f) {
                    let _e256 = normalize(vec2<f32>(_e226.x, -(_e226.y)));
                    let _e259 = acos(clamp(_e256.x, -1f, 1f));
                    if (_e256.y >= 0f) {
                        phi_1732_ = _e259;
                    } else {
                        phi_1732_ = -(_e259);
                    }
                    let _e264 = phi_1732_;
                    phi_1733_ = vec2<f32>((_e264 * 0.15915507f), _e239);
                    break;
                } else {
                    if (_e230 == 2f) {
                        phi_1731_ = _e226.x;
                    } else {
                        phi_1731_ = length(_e226);
                    }
                    let _e246 = phi_1731_;
                    phi_1733_ = vec2<f32>(((clamp(_e246, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e223.z < 0f))) + ((max(0f, _e223.z) * 0.001953125f) + 0.0009765625f)), _e239);
                    break;
                }
            }
        }
        let _e268 = phi_1733_;
        let _e269 = textureSampleLevel(ZC, Q8_, _e268, 0f);
        phi_1763_ = _e269;
        if !((bj && (((_e129.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e273 = (_e269.xyz * _e269.w);
            phi_1763_ = vec4<f32>(_e273.x, _e273.y, _e273.z, (_e269.w * abs(bitcast<f32>(_e129.y))));
        }
        let _e283 = phi_1763_;
        phi_1764_ = 0u;
        phi_1761_ = _e283;
    }
    let _e296 = phi_1764_;
    let _e298 = phi_1761_;
    phi_837_ = Zi;
    if Zi {
        let _e300 = S3_1;
        phi_837_ = (_e300 != 0u);
    }
    let _e303 = phi_837_;
    phi_1844_ = _e117;
    if _e303 {
        if (_e296 != 0u) {
            phi_1779_ = _e296;
        } else {
            let _e307 = m0_.r2_[_e103];
            phi_1779_ = _e307;
        }
        let _e309 = phi_1779_;
        let _e310 = S3_1;
        if (_e310 == (_e309 >> bitcast<u32>(16i))) {
            phi_1797_ = min(_e117, unpack2x16float(_e309).x);
        } else {
            phi_1797_ = 0f;
        }
        let _e318 = phi_1797_;
        phi_1844_ = _e318;
    }
    let _e320 = phi_1844_;
    let _e322 = v5_1[3u];
    phi_1842_ = _e105;
    if (_e322 != 0f) {
        let _e324 = v5_1;
        let _e326 = j.O8_;
        let _e328 = j.P8_;
        let _e331 = abs(_e324.z);
        let _e332 = floor(_e331);
        let _e335 = (((_e331 - _e332) * 8f) + 0.0009765625f);
        let _e337 = ((_e332 * _e326) + _e328);
        let _e338 = floor(_e335);
        switch bitcast<i32>(0u) {
            default: {
                if (_e338 == 4f) {
                    let _e356 = normalize(vec2<f32>(_e324.x, -(_e324.y)));
                    let _e359 = acos(clamp(_e356.x, -1f, 1f));
                    if (_e356.y >= 0f) {
                        phi_1818_ = _e359;
                    } else {
                        phi_1818_ = -(_e359);
                    }
                    let _e364 = phi_1818_;
                    phi_1819_ = vec2<f32>((_e364 * 0.15915507f), _e337);
                    break;
                } else {
                    if (_e338 == 2f) {
                        phi_1817_ = _e324.x;
                    } else {
                        phi_1817_ = length(_e324.xy);
                    }
                    let _e346 = phi_1817_;
                    phi_1819_ = vec2<f32>(((clamp(_e346, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e324.z < 0f))) + _e335), _e337);
                    break;
                }
            }
        }
        let _e368 = phi_1819_;
        let _e369 = textureSampleLevel(ZC, Q8_, _e368, 0f);
        let _e372 = (_e369.xyz * _e369.w);
        let _e378 = vec4<f32>(_e372.x, _e369.y, _e369.z, _e369.w);
        let _e384 = vec4<f32>(_e378.x, _e372.y, _e378.z, _e378.w);
        phi_1842_ = (_e105 * vec4<f32>(_e384.x, _e384.y, _e372.z, _e384.w));
    }
    let _e393 = phi_1842_;
    let _e394 = V1_1;
    let _e396 = ((_e393 * _e394) * _e320);
    let _e400 = (((_e298 * _e201) * (1f - _e396.w)) + _e396);
    let _e401 = _e400.xyz;
    let _e404 = j.F3_;
    let _e406 = j.G3_;
    if (gj && (_e400.w != 0f)) {
        phi_1856_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e68.x) + (0.00583715f * _e68.y))))) * _e404) + _e406)) + _e401);
    } else {
        phi_1856_ = _e401;
    }
    let _e422 = phi_1856_;
    let _e428 = vec4<f32>(_e422.x, _e400.y, _e400.z, _e400.w);
    let _e434 = vec4<f32>(_e428.x, _e422.y, _e428.z, _e428.w);
    N1_ = vec4<f32>(_e434.x, _e434.y, _e422.z, _e434.w);
    if (_e296 != 0u) {
        m0_.r2_[_e103] = _e296;
    }
    R4_.r2_[_e103] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) l2_: vec2<f32>, @location(1) r5_: f32, @location(3) V0_: vec4<f32>, @location(5) @interpolate(flat, either) S3_: u32, @location(2) v5_: vec4<f32>, @location(4) @interpolate(flat, either) V1_: vec4<f32>, @location(6) @interpolate(flat, either) L1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    l2_1 = l2_;
    r5_1 = r5_;
    V0_1 = V0_;
    S3_1 = S3_;
    v5_1 = v5_;
    V1_1 = V1_;
    L1_1 = L1_;
    main_1();
    let _e17 = N1_;
    return _e17;
}
