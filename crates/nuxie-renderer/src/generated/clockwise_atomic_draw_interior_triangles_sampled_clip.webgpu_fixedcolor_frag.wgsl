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

struct zf_1 {
    r2_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(0) member: vec4<f32>,
    @location(1) member_1: vec4<f32>,
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
var<private> U0_1: vec3<f32>;
var<private> P0_1: f32;
var<private> O0_1: vec4<f32>;
var<private> n1_1: f32;
var<private> L4_1: vec2<f32>;
var<private> z3_1: vec2<u32>;
var<private> V0_1: vec4<f32>;
var<private> i2_1: vec2<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> L1_: vec4<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> G0_1: f32;
var<private> m0_: vec4<f32>;

fn main_1() {
    var phi_1500_: f32;
    var phi_1501_: f32;
    var phi_1502_: vec2<f32>;
    var phi_1519_: vec4<f32>;
    var phi_1518_: vec4<f32>;
    var phi_942_: bool;
    var phi_957_: bool;
    var phi_1503_: f32;
    var phi_1514_: vec4<f32>;
    var phi_1521_: vec4<f32>;
    var phi_1522_: vec4<f32>;
    var phi_1524_: f32;
    var phi_772_: bool;
    var phi_1525_: f32;
    var phi_1251_: bool;
    var phi_1253_: bool;
    var phi_1546_: f32;
    var phi_1541_: u32;
    var phi_1538_: f32;
    var phi_1545_: f32;
    var phi_1540_: u32;
    var phi_1537_: f32;
    var phi_1542_: f32;
    var phi_1539_: u32;
    var phi_1536_: f32;
    var phi_1547_: f32;
    var phi_1548_: f32;
    var phi_1549_: f32;
    var phi_1559_: f32;
    var phi_1582_: vec3<f32>;

    let _e58 = P0_1;
    let _e60 = U0_1;
    let _e61 = O0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e64 = (Zi && (u32(_e58) != 0u));
            if (_e61.w >= 0f) {
                phi_1518_ = _e61;
            } else {
                let _e68 = j.M8_;
                let _e70 = j.N8_;
                let _e73 = abs(_e61.z);
                let _e74 = floor(_e73);
                let _e77 = (((_e73 - _e74) * 8f) + 0.0009765625f);
                let _e79 = ((_e74 * _e68) + _e70);
                let _e80 = floor(_e77);
                switch bitcast<i32>(0u) {
                    default: {
                        if (_e80 == 4f) {
                            let _e98 = normalize(vec2<f32>(_e61.x, -(_e61.y)));
                            let _e101 = acos(clamp(_e98.x, -1f, 1f));
                            if (_e98.y >= 0f) {
                                phi_1501_ = _e101;
                            } else {
                                phi_1501_ = -(_e101);
                            }
                            let _e106 = phi_1501_;
                            phi_1502_ = vec2<f32>((_e106 * 0.15915507f), _e79);
                            break;
                        } else {
                            if (_e80 == 2f) {
                                phi_1500_ = _e61.x;
                            } else {
                                phi_1500_ = length(_e61.xy);
                            }
                            let _e88 = phi_1500_;
                            phi_1502_ = vec2<f32>(((clamp(_e88, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e61.z < 0f))) + _e77), _e79);
                            break;
                        }
                    }
                }
                let _e110 = phi_1502_;
                let _e111 = textureSampleLevel(XC, O8_, _e110, 0f);
                phi_1519_ = _e111;
                if !(_e64) {
                    let _e117 = (_e111.xyz * _e111.w);
                    phi_1519_ = vec4<f32>(_e117.x, _e117.y, _e117.z, (_e111.w * (ceil(_e61.w) * -0.003921569f)));
                }
                let _e124 = phi_1519_;
                phi_1518_ = _e124;
            }
            let _e126 = phi_1518_;
            phi_942_ = fj;
            if fj {
                phi_942_ = (_e60.z < 0f);
            }
            let _e130 = phi_942_;
            if _e130 {
                let _e132 = textureSampleLevel(TB, U4_, _e60.xy, 0f);
                phi_1522_ = _e132;
                break;
            }
            phi_957_ = fj;
            if fj {
                phi_957_ = (_e60.z > 0f);
            }
            let _e136 = phi_957_;
            phi_1521_ = _e126;
            if _e136 {
                let _e140 = textureSampleLevel(TB, U4_, _e60.xy, (_e60.z - 1f));
                phi_1514_ = _e140;
                if _e64 {
                    if (_e140.w != 0f) {
                        phi_1503_ = (1f / _e140.w);
                    } else {
                        phi_1503_ = 0f;
                    }
                    let _e146 = phi_1503_;
                    let _e147 = (_e140.xyz * _e146);
                    phi_1514_ = vec4<f32>(_e147.x, _e147.y, _e147.z, _e140.w);
                }
                let _e153 = phi_1514_;
                phi_1521_ = (_e126 * _e153);
            }
            let _e156 = phi_1521_;
            phi_1522_ = _e156;
            break;
        }
    }
    let _e158 = phi_1522_;
    let _e159 = n1_1;
    let _e160 = L4_1;
    let _e163 = z3_1[1u];
    let _e165 = z3_1[0u];
    let _e166 = vec2<u32>(floor(_e160));
    phi_1524_ = 1f;
    if Yi {
        let _e194 = V0_1;
        let _e197 = min(_e194.xy, _e194.zw);
        phi_1524_ = min(min(_e197.x, _e197.y), 1f);
    }
    let _e203 = phi_1524_;
    phi_772_ = Xi;
    if Xi {
        let _e205 = i2_1[0u];
        phi_772_ = (_e205 != 0f);
    }
    let _e208 = phi_772_;
    phi_1525_ = _e203;
    if _e208 {
        phi_1525_ = min(0f, _e203);
    }
    let _e211 = phi_1525_;
    let _e213 = clamp(_e159, 0f, max(_e211, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e219 = u32(((abs(_e213) * 1024f) + 0.5f));
            let _e222 = atomicLoad((&Y0_.r2_[(_e165 + (((((_e166.y >> bitcast<u32>(5u)) * (_e163 << bitcast<u32>(5u))) + ((_e166.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e166.x & 28u) << bitcast<u32>(5u)) + ((_e166.y & 28u) << bitcast<u32>(2i)))) + (((_e166.y & 3u) << bitcast<u32>(2i)) + (_e166.x & 3u))))]));
            let _e224 = (min(_e158.w, _e213) >= 1f);
            phi_1253_ = _e224;
            if _e224 {
                let _e226 = j.q2_;
                let _e227 = (_e222 < _e226);
                phi_1251_ = _e227;
                if !(_e227) {
                    phi_1251_ = (_e222 >= (_e226 | 262144u));
                }
                let _e232 = phi_1251_;
                phi_1253_ = _e232;
            }
            let _e234 = phi_1253_;
            if _e234 {
                phi_1548_ = 1f;
                break;
            }
            let _e236 = j.q2_;
            phi_1542_ = 0f;
            phi_1539_ = _e219;
            phi_1536_ = _e213;
            if (_e222 < _e236) {
                let _e239 = (_e236 | (262144u + _e219));
                let _e240 = atomicMax((&Y0_.r2_[(_e165 + (((((_e166.y >> bitcast<u32>(5u)) * (_e163 << bitcast<u32>(5u))) + ((_e166.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e166.x & 28u) << bitcast<u32>(5u)) + ((_e166.y & 28u) << bitcast<u32>(2i)))) + (((_e166.y & 3u) << bitcast<u32>(2i)) + (_e166.x & 3u))))]), _e239);
                if (_e240 <= _e236) {
                    phi_1545_ = min(_e213, 1f);
                    phi_1540_ = _e219;
                    phi_1537_ = 0f;
                } else {
                    phi_1546_ = 0f;
                    phi_1541_ = _e219;
                    phi_1538_ = _e213;
                    if (_e240 < _e239) {
                        let _e244 = ((_e240 & 524287u) - 262144u);
                        let _e246 = (f32(_e244) * 0.0009765625f);
                        phi_1546_ = ((min(_e213, 1f) - _e246) / max((1f - (_e246 * _e158.w)), 0.000062f));
                        phi_1541_ = _e244;
                        phi_1538_ = _e246;
                    }
                    let _e254 = phi_1546_;
                    let _e256 = phi_1541_;
                    let _e258 = phi_1538_;
                    phi_1545_ = _e254;
                    phi_1540_ = _e256;
                    phi_1537_ = _e258;
                }
                let _e261 = phi_1545_;
                let _e263 = phi_1540_;
                let _e265 = phi_1537_;
                phi_1542_ = _e261;
                phi_1539_ = _e263;
                phi_1536_ = _e265;
            }
            let _e267 = phi_1542_;
            let _e269 = phi_1539_;
            let _e271 = phi_1536_;
            phi_1547_ = _e267;
            if (_e271 > 0f) {
                let _e273 = atomicAdd((&Y0_.r2_[(_e165 + (((((_e166.y >> bitcast<u32>(5u)) * (_e163 << bitcast<u32>(5u))) + ((_e166.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e166.x & 28u) << bitcast<u32>(5u)) + ((_e166.y & 28u) << bitcast<u32>(2i)))) + (((_e166.y & 3u) << bitcast<u32>(2i)) + (_e166.x & 3u))))]), _e269);
                let _e278 = (f32(bitcast<i32>(((_e273 & 524287u) - 262144u))) * 0.0009765625f);
                let _e280 = clamp(_e278, 0f, 1f);
                phi_1547_ = (_e267 + ((1f - (_e267 * _e158.w)) * ((clamp((_e278 + _e271), 0f, 1f) - _e280) / max((1f - (_e280 * _e158.w)), 0.000062f))));
            }
            let _e292 = phi_1547_;
            phi_1548_ = _e292;
            break;
        }
    }
    let _e294 = phi_1548_;
    phi_1559_ = f32();
    if ej {
        let _e295 = gl_FragCoord_1;
        let _e297 = j.F3_;
        let _e299 = j.G3_;
        if ej {
            phi_1549_ = ((fract((52.982918f * fract(((0.06711056f * _e295.x) + (0.00583715f * _e295.y))))) * _e297) + _e299);
        } else {
            phi_1549_ = 0f;
        }
        let _e311 = phi_1549_;
        phi_1559_ = _e311;
    }
    let _e313 = phi_1559_;
    let _e314 = (_e158 * _e294);
    let _e315 = _e314.xyz;
    if (ej && (_e314.w != 0f)) {
        phi_1582_ = (vec3(_e313) + _e315);
    } else {
        phi_1582_ = _e315;
    }
    let _e322 = phi_1582_;
    let _e328 = vec4<f32>(_e322.x, _e314.y, _e314.z, _e314.w);
    let _e334 = vec4<f32>(_e328.x, _e322.y, _e328.z, _e328.w);
    L1_ = vec4<f32>(_e334.x, _e334.y, _e322.z, _e334.w);
    return;
}

@fragment
fn main(@location(9) U0_: vec3<f32>, @location(6) @interpolate(flat, either) P0_: f32, @location(0) O0_: vec4<f32>, @location(1) @interpolate(flat, either) n1_: f32, @location(8) L4_: vec2<f32>, @location(7) @interpolate(flat, either) z3_: vec2<u32>, @location(5) V0_: vec4<f32>, @location(4) @interpolate(flat, either) i2_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32) -> FragmentOutput {
    U0_1 = U0_;
    P0_1 = P0_;
    O0_1 = O0_;
    n1_1 = n1_;
    L4_1 = L4_;
    z3_1 = z3_;
    V0_1 = V0_;
    i2_1 = i2_;
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    main_1();
    let _e22 = L1_;
    let _e23 = m0_;
    return FragmentOutput(_e22, _e23);
}
