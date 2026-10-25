struct og {
    v2_: array<vec2<u32>>,
}

struct m0Ue {
    v2_: array<u32>,
}

struct pg {
    v2_: array<vec4<f32>>,
}

struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
    P8_: f32,
    Q8_: f32,
}

struct T4Ue {
    v2_: array<u32>,
}

@id(8) override nj: bool = true;
@id(9) override oj: bool = true;
@id(5) override kj: bool = true;
@id(0) override fj: bool = true;
@id(1) override gj: bool = true;
@id(2) override hj: bool = true;

@group(0) @binding(3)
var<storage> XC: og;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Ue;
@group(0) @binding(4)
var<storage> JB: pg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var R8_: sampler;
@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(1) @binding(13)
var W4_: sampler;
var<private> n2_1: vec2<f32>;
var<private> v5_1: f32;
var<private> W0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> T4_: T4Ue;
var<private> U3_1: u32;
var<private> w5_1: vec4<f32>;
var<private> Y1_1: vec4<f32>;
var<private> Q1_: vec4<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
var<private> N1_1: u32;

fn main_1() {
    var phi_1792_: f32;
    var phi_1103_: bool;
    var phi_1730_: f32;
    var phi_1729_: f32;
    var phi_1731_: f32;
    var phi_1734_: f32;
    var phi_1733_: f32;
    var phi_1140_: bool;
    var phi_1738_: f32;
    var phi_1735_: f32;
    var phi_1736_: f32;
    var phi_1737_: vec2<f32>;
    var phi_1767_: vec4<f32>;
    var phi_1769_: u32;
    var phi_1768_: u32;
    var phi_1765_: vec4<f32>;
    var phi_839_: bool;
    var phi_1783_: u32;
    var phi_1801_: f32;
    var phi_1848_: f32;
    var phi_1821_: f32;
    var phi_1822_: f32;
    var phi_1823_: vec2<f32>;
    var phi_1846_: vec4<f32>;
    var phi_1860_: vec3<f32>;

    let _e69 = gl_FragCoord_1;
    let _e70 = _e69.xy;
    let _e73 = bitcast<vec2<u32>>(vec2<i32>(floor(_e70)));
    let _e75 = j.T6_;
    let _e104 = bitcast<i32>((((((_e73.y >> bitcast<u32>(5u)) * (((_e75 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e73.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e73.x & 28u) << bitcast<u32>(5u)) + ((_e73.y & 28u) << bitcast<u32>(2i)))) + (((_e73.y & 3u) << bitcast<u32>(2i)) + (_e73.x & 3u))));
    let _e105 = n2_1;
    let _e106 = textureSample(UB, W4_, _e105);
    let _e107 = v5_1;
    let _e108 = min(_e107, 1f);
    phi_1792_ = _e108;
    if gj {
        let _e109 = W0_1;
        let _e112 = min(_e109.xy, _e109.zw);
        phi_1792_ = clamp(min(_e112.x, _e112.y), 0f, _e108);
    }
    let _e118 = phi_1792_;
    let _e121 = T4_.v2_[_e104];
    let _e123 = (_e121 >> bitcast<u32>(17u));
    let _e127 = ((f32((_e121 & 131071u)) * 0.00048828125f) + -32f);
    let _e130 = XC.v2_[_e123];
    phi_1729_ = _e127;
    if ((_e130.x & 768u) != 0u) {
        let _e134 = abs(_e127);
        phi_1103_ = kj;
        if kj {
            phi_1103_ = ((_e130.x & 512u) != 0u);
        }
        let _e138 = phi_1103_;
        phi_1730_ = _e134;
        if _e138 {
            phi_1730_ = (1f - abs(((fract((_e134 * 0.5f)) * 2f) + -1f)));
        }
        let _e146 = phi_1730_;
        phi_1729_ = _e146;
    }
    let _e148 = phi_1729_;
    let _e149 = clamp(_e148, 0f, 1f);
    phi_1733_ = _e149;
    if fj {
        let _e151 = (_e130.x >> bitcast<u32>(16u));
        phi_1734_ = _e149;
        if (_e151 != 0u) {
            let _e155 = m0_.v2_[_e104];
            if (_e151 == (_e155 >> bitcast<u32>(16i))) {
                phi_1731_ = min(_e149, unpack2x16float(_e155).x);
            } else {
                phi_1731_ = 0f;
            }
            let _e163 = phi_1731_;
            phi_1734_ = _e163;
        }
        let _e165 = phi_1734_;
        phi_1733_ = _e165;
    }
    let _e167 = phi_1733_;
    phi_1140_ = gj;
    if gj {
        phi_1140_ = ((_e130.x & 1024u) != 0u);
    }
    let _e171 = phi_1140_;
    phi_1738_ = _e167;
    if _e171 {
        let _e172 = (_e123 * 8u);
        let _e176 = JB.v2_[(_e172 + 2u)];
        let _e187 = JB.v2_[(_e172 + 3u)];
        let _e192 = _e187.zw;
        let _e194 = ((abs(((mat2x2<f32>(vec2<f32>(_e176.x, _e176.y), vec2<f32>(_e176.z, _e176.w)) * _e70) + _e187.xy)) * _e192) - _e192);
        phi_1738_ = min(_e167, clamp((min(_e194.x, _e194.y) + 0.5f), 0f, 1f));
    }
    let _e202 = phi_1738_;
    let _e203 = (_e130.x & 15u);
    if (_e203 <= 1u) {
        let _e289 = (fj && (_e203 == 0u));
        phi_1769_ = 0u;
        if _e289 {
            phi_1769_ = (_e130.y | pack2x16float(vec2<f32>(_e202, 0f)));
        }
        let _e294 = phi_1769_;
        phi_1768_ = _e294;
        phi_1765_ = select(unpack4x8unorm(_e130.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e289));
    } else {
        let _e210 = (_e123 * 8u);
        let _e213 = JB.v2_[_e210];
        let _e224 = JB.v2_[(_e210 + 1u)];
        let _e227 = ((mat2x2<f32>(vec2<f32>(_e213.x, _e213.y), vec2<f32>(_e213.z, _e213.w)) * _e70) + _e224.xy);
        let _e231 = f32(_e203);
        let _e233 = j.P8_;
        let _e235 = j.Q8_;
        let _e240 = ((_e224.w * _e233) + _e235);
        switch bitcast<i32>(0u) {
            default: {
                if (oj && (_e231 == 4f)) {
                    let _e258 = normalize(vec2<f32>(_e227.x, -(_e227.y)));
                    let _e261 = acos(clamp(_e258.x, -1f, 1f));
                    if (_e258.y >= 0f) {
                        phi_1736_ = _e261;
                    } else {
                        phi_1736_ = -(_e261);
                    }
                    let _e266 = phi_1736_;
                    phi_1737_ = vec2<f32>((_e266 * 0.15915494f), _e240);
                    break;
                } else {
                    if (_e231 == 2f) {
                        phi_1735_ = _e227.x;
                    } else {
                        phi_1735_ = length(_e227);
                    }
                    let _e248 = phi_1735_;
                    phi_1737_ = vec2<f32>(((clamp(_e248, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e224.z < 0f))) + ((max(0f, _e224.z) * 0.001953125f) + 0.0009765625f)), _e240);
                    break;
                }
            }
        }
        let _e270 = phi_1737_;
        let _e271 = textureSampleLevel(ZC, R8_, _e270, 0f);
        phi_1767_ = _e271;
        if !((hj && (((_e130.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e275 = (_e271.xyz * _e271.w);
            phi_1767_ = vec4<f32>(_e275.x, _e275.y, _e275.z, (_e271.w * abs(bitcast<f32>(_e130.y))));
        }
        let _e285 = phi_1767_;
        phi_1768_ = 0u;
        phi_1765_ = _e285;
    }
    let _e298 = phi_1768_;
    let _e300 = phi_1765_;
    phi_839_ = fj;
    if fj {
        let _e302 = U3_1;
        phi_839_ = (_e302 != 0u);
    }
    let _e305 = phi_839_;
    phi_1848_ = _e118;
    if _e305 {
        if (_e298 != 0u) {
            phi_1783_ = _e298;
        } else {
            let _e309 = m0_.v2_[_e104];
            phi_1783_ = _e309;
        }
        let _e311 = phi_1783_;
        let _e312 = U3_1;
        if (_e312 == (_e311 >> bitcast<u32>(16i))) {
            phi_1801_ = min(_e118, unpack2x16float(_e311).x);
        } else {
            phi_1801_ = 0f;
        }
        let _e320 = phi_1801_;
        phi_1848_ = _e320;
    }
    let _e322 = phi_1848_;
    let _e324 = w5_1[3u];
    phi_1846_ = _e106;
    if (_e324 != 0f) {
        let _e326 = w5_1;
        let _e328 = j.P8_;
        let _e330 = j.Q8_;
        let _e333 = abs(_e326.z);
        let _e334 = floor(_e333);
        let _e337 = (((_e333 - _e334) * 8f) + 0.0009765625f);
        let _e339 = ((_e334 * _e328) + _e330);
        let _e340 = floor(_e337);
        switch bitcast<i32>(0u) {
            default: {
                if (oj && (_e340 == 4f)) {
                    let _e359 = normalize(vec2<f32>(_e326.x, -(_e326.y)));
                    let _e362 = acos(clamp(_e359.x, -1f, 1f));
                    if (_e359.y >= 0f) {
                        phi_1822_ = _e362;
                    } else {
                        phi_1822_ = -(_e362);
                    }
                    let _e367 = phi_1822_;
                    phi_1823_ = vec2<f32>((_e367 * 0.15915494f), _e339);
                    break;
                } else {
                    if (_e340 == 2f) {
                        phi_1821_ = _e326.x;
                    } else {
                        phi_1821_ = length(_e326.xy);
                    }
                    let _e349 = phi_1821_;
                    phi_1823_ = vec2<f32>(((clamp(_e349, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e326.z < 0f))) + _e337), _e339);
                    break;
                }
            }
        }
        let _e371 = phi_1823_;
        let _e372 = textureSampleLevel(ZC, R8_, _e371, 0f);
        let _e375 = (_e372.xyz * _e372.w);
        let _e381 = vec4<f32>(_e375.x, _e372.y, _e372.z, _e372.w);
        let _e387 = vec4<f32>(_e381.x, _e375.y, _e381.z, _e381.w);
        phi_1846_ = (_e106 * vec4<f32>(_e387.x, _e387.y, _e375.z, _e387.w));
    }
    let _e396 = phi_1846_;
    let _e397 = Y1_1;
    let _e399 = ((_e396 * _e397) * _e322);
    let _e403 = (((_e300 * _e202) * (1f - _e399.w)) + _e399);
    let _e404 = _e403.xyz;
    let _e407 = j.G3_;
    let _e409 = j.H3_;
    if (nj && (_e403.w != 0f)) {
        phi_1860_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e69.x) + (0.00583715f * _e69.y))))) * _e407) + _e409)) + _e404);
    } else {
        phi_1860_ = _e404;
    }
    let _e425 = phi_1860_;
    let _e431 = vec4<f32>(_e425.x, _e403.y, _e403.z, _e403.w);
    let _e437 = vec4<f32>(_e431.x, _e425.y, _e431.z, _e431.w);
    Q1_ = vec4<f32>(_e437.x, _e437.y, _e425.z, _e437.w);
    if (_e298 != 0u) {
        m0_.v2_[_e104] = _e298;
    }
    T4_.v2_[_e104] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) n2_: vec2<f32>, @location(1) v5_: f32, @location(3) W0_: vec4<f32>, @location(5) @interpolate(flat, either) U3_: u32, @location(2) w5_: vec4<f32>, @location(4) @interpolate(flat, either) Y1_: vec4<f32>, @location(6) @interpolate(flat, either) N1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    n2_1 = n2_;
    v5_1 = v5_;
    W0_1 = W0_;
    U3_1 = U3_;
    w5_1 = w5_;
    Y1_1 = Y1_;
    N1_1 = N1_;
    main_1();
    let _e17 = Q1_;
    return _e17;
}
