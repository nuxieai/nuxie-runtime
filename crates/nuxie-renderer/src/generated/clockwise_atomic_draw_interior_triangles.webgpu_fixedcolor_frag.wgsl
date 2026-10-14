struct VB {
    xd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    F8_: vec4<i32>,
    zi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Di: f32,
    w6_: u32,
    i3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    wi: u32,
    Ba: u32,
    M8_: f32,
    N8_: f32,
}

struct zf {
    r2_: array<u32>,
}

struct m0Pe {
    r2_: array<u32>,
}

struct zf_1 {
    r2_: array<atomic<u32>>,
}

@id(7) override ej: bool = true;
@id(2) override Zi: bool = true;
@id(8) override fj: bool = true;
@id(1) override Yi: bool = true;
@id(0) override Xi: bool = true;

@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var O8_: sampler;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;
@group(0) @binding(6)
var<storage, read_write> Y0_: zf_1;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> U0_1: vec3<f32>;
var<private> P0_1: f32;
var<private> O0_1: vec4<f32>;
var<private> n1_1: f32;
var<private> L4_1: vec2<f32>;
var<private> z3_1: vec2<u32>;
var<private> V0_1: vec4<f32>;
var<private> i2_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Pe;
var<private> L1_: vec4<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> G0_1: f32;

fn main_1() {
    var phi_1612_: f32;
    var phi_1613_: f32;
    var phi_1614_: vec2<f32>;
    var phi_1631_: vec4<f32>;
    var phi_1630_: vec4<f32>;
    var phi_1028_: bool;
    var phi_1043_: bool;
    var phi_1615_: f32;
    var phi_1626_: vec4<f32>;
    var phi_1633_: vec4<f32>;
    var phi_1634_: vec4<f32>;
    var phi_1636_: f32;
    var phi_810_: bool;
    var phi_1637_: f32;
    var phi_1337_: bool;
    var phi_1339_: bool;
    var phi_1658_: f32;
    var phi_1653_: u32;
    var phi_1650_: f32;
    var phi_1657_: f32;
    var phi_1652_: u32;
    var phi_1649_: f32;
    var phi_1654_: f32;
    var phi_1651_: u32;
    var phi_1648_: f32;
    var phi_1659_: f32;
    var phi_1660_: f32;
    var phi_1661_: f32;
    var phi_1671_: f32;
    var phi_1694_: vec3<f32>;

    let _e62 = gl_FragCoord_1;
    let _e66 = bitcast<vec2<u32>>(vec2<i32>(floor(_e62.xy)));
    let _e68 = j.P6_;
    let _e97 = bitcast<i32>((((((_e66.y >> bitcast<u32>(5u)) * (((_e68 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e66.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e66.x & 28u) << bitcast<u32>(5u)) + ((_e66.y & 28u) << bitcast<u32>(2i)))) + (((_e66.y & 3u) << bitcast<u32>(2i)) + (_e66.x & 3u))));
    let _e98 = P0_1;
    let _e100 = U0_1;
    let _e101 = O0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e104 = (Zi && (u32(_e98) != 0u));
            if (_e101.w >= 0f) {
                phi_1630_ = _e101;
            } else {
                let _e108 = j.M8_;
                let _e110 = j.N8_;
                let _e113 = abs(_e101.z);
                let _e114 = floor(_e113);
                let _e117 = (((_e113 - _e114) * 8f) + 0.0009765625f);
                let _e119 = ((_e114 * _e108) + _e110);
                let _e120 = floor(_e117);
                switch bitcast<i32>(0u) {
                    default: {
                        if (_e120 == 4f) {
                            let _e138 = normalize(vec2<f32>(_e101.x, -(_e101.y)));
                            let _e141 = acos(clamp(_e138.x, -1f, 1f));
                            if (_e138.y >= 0f) {
                                phi_1613_ = _e141;
                            } else {
                                phi_1613_ = -(_e141);
                            }
                            let _e146 = phi_1613_;
                            phi_1614_ = vec2<f32>((_e146 * 0.15915507f), _e119);
                            break;
                        } else {
                            if (_e120 == 2f) {
                                phi_1612_ = _e101.x;
                            } else {
                                phi_1612_ = length(_e101.xy);
                            }
                            let _e128 = phi_1612_;
                            phi_1614_ = vec2<f32>(((clamp(_e128, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e101.z < 0f))) + _e117), _e119);
                            break;
                        }
                    }
                }
                let _e150 = phi_1614_;
                let _e151 = textureSampleLevel(XC, O8_, _e150, 0f);
                phi_1631_ = _e151;
                if !(_e104) {
                    let _e157 = (_e151.xyz * _e151.w);
                    phi_1631_ = vec4<f32>(_e157.x, _e157.y, _e157.z, (_e151.w * (ceil(_e101.w) * -0.003921569f)));
                }
                let _e164 = phi_1631_;
                phi_1630_ = _e164;
            }
            let _e166 = phi_1630_;
            phi_1028_ = fj;
            if fj {
                phi_1028_ = (_e100.z < 0f);
            }
            let _e170 = phi_1028_;
            if _e170 {
                let _e172 = textureSampleLevel(TB, U4_, _e100.xy, 0f);
                phi_1634_ = _e172;
                break;
            }
            phi_1043_ = fj;
            if fj {
                phi_1043_ = (_e100.z > 0f);
            }
            let _e176 = phi_1043_;
            phi_1633_ = _e166;
            if _e176 {
                let _e180 = textureSampleLevel(TB, U4_, _e100.xy, (_e100.z - 1f));
                phi_1626_ = _e180;
                if _e104 {
                    if (_e180.w != 0f) {
                        phi_1615_ = (1f / _e180.w);
                    } else {
                        phi_1615_ = 0f;
                    }
                    let _e186 = phi_1615_;
                    let _e187 = (_e180.xyz * _e186);
                    phi_1626_ = vec4<f32>(_e187.x, _e187.y, _e187.z, _e180.w);
                }
                let _e193 = phi_1626_;
                phi_1633_ = (_e166 * _e193);
            }
            let _e196 = phi_1633_;
            phi_1634_ = _e196;
            break;
        }
    }
    let _e198 = phi_1634_;
    let _e199 = n1_1;
    let _e200 = L4_1;
    let _e203 = z3_1[1u];
    let _e205 = z3_1[0u];
    let _e206 = vec2<u32>(floor(_e200));
    phi_1636_ = 1f;
    if Yi {
        let _e234 = V0_1;
        let _e237 = min(_e234.xy, _e234.zw);
        phi_1636_ = min(min(_e237.x, _e237.y), 1f);
    }
    let _e243 = phi_1636_;
    phi_810_ = Xi;
    if Xi {
        let _e245 = i2_1[0u];
        phi_810_ = (_e245 != 0f);
    }
    let _e248 = phi_810_;
    phi_1637_ = _e243;
    if _e248 {
        let _e251 = m0_.r2_[_e97];
        phi_1637_ = min(unpack4x8unorm(_e251).x, _e243);
    }
    let _e256 = phi_1637_;
    let _e258 = clamp(_e199, 0f, max(_e256, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e264 = u32(((abs(_e258) * 1024f) + 0.5f));
            let _e267 = atomicLoad((&Y0_.r2_[(_e205 + (((((_e206.y >> bitcast<u32>(5u)) * (_e203 << bitcast<u32>(5u))) + ((_e206.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e206.x & 28u) << bitcast<u32>(5u)) + ((_e206.y & 28u) << bitcast<u32>(2i)))) + (((_e206.y & 3u) << bitcast<u32>(2i)) + (_e206.x & 3u))))]));
            let _e269 = (min(_e198.w, _e258) >= 1f);
            phi_1339_ = _e269;
            if _e269 {
                let _e271 = j.q2_;
                let _e272 = (_e267 < _e271);
                phi_1337_ = _e272;
                if !(_e272) {
                    phi_1337_ = (_e267 >= (_e271 | 262144u));
                }
                let _e277 = phi_1337_;
                phi_1339_ = _e277;
            }
            let _e279 = phi_1339_;
            if _e279 {
                phi_1660_ = 1f;
                break;
            }
            let _e281 = j.q2_;
            phi_1654_ = 0f;
            phi_1651_ = _e264;
            phi_1648_ = _e258;
            if (_e267 < _e281) {
                let _e284 = (_e281 | (262144u + _e264));
                let _e285 = atomicMax((&Y0_.r2_[(_e205 + (((((_e206.y >> bitcast<u32>(5u)) * (_e203 << bitcast<u32>(5u))) + ((_e206.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e206.x & 28u) << bitcast<u32>(5u)) + ((_e206.y & 28u) << bitcast<u32>(2i)))) + (((_e206.y & 3u) << bitcast<u32>(2i)) + (_e206.x & 3u))))]), _e284);
                if (_e285 <= _e281) {
                    phi_1657_ = min(_e258, 1f);
                    phi_1652_ = _e264;
                    phi_1649_ = 0f;
                } else {
                    phi_1658_ = 0f;
                    phi_1653_ = _e264;
                    phi_1650_ = _e258;
                    if (_e285 < _e284) {
                        let _e289 = ((_e285 & 524287u) - 262144u);
                        let _e291 = (f32(_e289) * 0.0009765625f);
                        phi_1658_ = ((min(_e258, 1f) - _e291) / max((1f - (_e291 * _e198.w)), 0.000062f));
                        phi_1653_ = _e289;
                        phi_1650_ = _e291;
                    }
                    let _e299 = phi_1658_;
                    let _e301 = phi_1653_;
                    let _e303 = phi_1650_;
                    phi_1657_ = _e299;
                    phi_1652_ = _e301;
                    phi_1649_ = _e303;
                }
                let _e306 = phi_1657_;
                let _e308 = phi_1652_;
                let _e310 = phi_1649_;
                phi_1654_ = _e306;
                phi_1651_ = _e308;
                phi_1648_ = _e310;
            }
            let _e312 = phi_1654_;
            let _e314 = phi_1651_;
            let _e316 = phi_1648_;
            phi_1659_ = _e312;
            if (_e316 > 0f) {
                let _e318 = atomicAdd((&Y0_.r2_[(_e205 + (((((_e206.y >> bitcast<u32>(5u)) * (_e203 << bitcast<u32>(5u))) + ((_e206.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e206.x & 28u) << bitcast<u32>(5u)) + ((_e206.y & 28u) << bitcast<u32>(2i)))) + (((_e206.y & 3u) << bitcast<u32>(2i)) + (_e206.x & 3u))))]), _e314);
                let _e323 = (f32(bitcast<i32>(((_e318 & 524287u) - 262144u))) * 0.0009765625f);
                let _e325 = clamp(_e323, 0f, 1f);
                phi_1659_ = (_e312 + ((1f - (_e312 * _e198.w)) * ((clamp((_e323 + _e316), 0f, 1f) - _e325) / max((1f - (_e325 * _e198.w)), 0.000062f))));
            }
            let _e337 = phi_1659_;
            phi_1660_ = _e337;
            break;
        }
    }
    let _e339 = phi_1660_;
    phi_1671_ = f32();
    if ej {
        let _e341 = j.F3_;
        let _e343 = j.G3_;
        if ej {
            phi_1661_ = ((fract((52.982918f * fract(((0.06711056f * _e62.x) + (0.00583715f * _e62.y))))) * _e341) + _e343);
        } else {
            phi_1661_ = 0f;
        }
        let _e355 = phi_1661_;
        phi_1671_ = _e355;
    }
    let _e357 = phi_1671_;
    let _e358 = (_e198 * _e339);
    let _e359 = _e358.xyz;
    if (ej && (_e358.w != 0f)) {
        phi_1694_ = (vec3(_e357) + _e359);
    } else {
        phi_1694_ = _e359;
    }
    let _e366 = phi_1694_;
    let _e372 = vec4<f32>(_e366.x, _e358.y, _e358.z, _e358.w);
    let _e378 = vec4<f32>(_e372.x, _e366.y, _e372.z, _e372.w);
    m0_.r2_[_e97] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    L1_ = vec4<f32>(_e378.x, _e378.y, _e366.z, _e378.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(9) U0_: vec3<f32>, @location(6) @interpolate(flat, either) P0_: f32, @location(0) O0_: vec4<f32>, @location(1) @interpolate(flat, either) n1_: f32, @location(8) L4_: vec2<f32>, @location(7) @interpolate(flat, either) z3_: vec2<u32>, @location(5) V0_: vec4<f32>, @location(4) @interpolate(flat, either) i2_: vec2<f32>, @location(3) @interpolate(flat, either) G0_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    U0_1 = U0_;
    P0_1 = P0_;
    O0_1 = O0_;
    n1_1 = n1_;
    L4_1 = L4_;
    z3_1 = z3_;
    V0_1 = V0_;
    i2_1 = i2_;
    G0_1 = G0_;
    main_1();
    let _e21 = L1_;
    return _e21;
}
