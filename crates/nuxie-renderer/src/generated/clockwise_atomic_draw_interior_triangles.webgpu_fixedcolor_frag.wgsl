struct WB {
    Fd: f32,
    Pe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    J8_: vec4<i32>,
    Ki: vec2<f32>,
    Qe: vec2<f32>,
    r2_: u32,
    Oi: f32,
    z6_: u32,
    i3_: f32,
    Re: f32,
    xg: u32,
    H3_: f32,
    I3_: f32,
    Se: f32,
    Hi: u32,
    Ea: u32,
    Q8_: f32,
    R8_: f32,
}

struct Kf {
    v2_: array<u32>,
}

struct m0cf {
    v2_: array<u32>,
}

struct Kf_1 {
    v2_: array<atomic<u32>>,
}

@id(8) override rj: bool = true;
@id(9) override sj: bool = true;
@id(2) override lj: bool = true;
@id(4) override nj: bool = true;
@id(1) override kj: bool = true;
@id(0) override jj: bool = true;

@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var S8_: sampler;
@group(1) @binding(11)
var RB: texture_2d<f32>;
@group(1) @binding(13)
var v4_: sampler;
@group(0) @binding(6)
var<storage, read_write> Y0_: Kf_1;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> O0_1: vec3<f32>;
var<private> S0_1: f32;
var<private> N0_1: vec4<f32>;
var<private> p1_1: f32;
var<private> P4_1: vec2<f32>;
var<private> A3_1: vec2<u32>;
var<private> W0_1: vec4<f32>;
var<private> k2_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> m0_: m0cf;
var<private> Q1_: vec4<f32>;
@group(3) @binding(9)
var eb: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
var<private> I0_1: f32;

fn main_1() {
    var phi_1615_: f32;
    var phi_1616_: f32;
    var phi_1617_: vec2<f32>;
    var phi_1634_: vec4<f32>;
    var phi_1633_: vec4<f32>;
    var phi_1030_: bool;
    var phi_1045_: bool;
    var phi_1618_: f32;
    var phi_1629_: vec4<f32>;
    var phi_1636_: vec4<f32>;
    var phi_1637_: vec4<f32>;
    var phi_1639_: f32;
    var phi_812_: bool;
    var phi_1640_: f32;
    var phi_1340_: bool;
    var phi_1342_: bool;
    var phi_1661_: f32;
    var phi_1656_: u32;
    var phi_1653_: f32;
    var phi_1660_: f32;
    var phi_1655_: u32;
    var phi_1652_: f32;
    var phi_1657_: f32;
    var phi_1654_: u32;
    var phi_1651_: f32;
    var phi_1662_: f32;
    var phi_1663_: f32;
    var phi_1664_: f32;
    var phi_1674_: f32;
    var phi_1697_: vec3<f32>;

    let _e63 = gl_FragCoord_1;
    let _e67 = bitcast<vec2<u32>>(vec2<i32>(floor(_e63.xy)));
    let _e69 = j.T6_;
    let _e98 = bitcast<i32>((((((_e67.y >> bitcast<u32>(5u)) * (((_e69 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e67.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e67.x & 28u) << bitcast<u32>(5u)) + ((_e67.y & 28u) << bitcast<u32>(2i)))) + (((_e67.y & 3u) << bitcast<u32>(2i)) + (_e67.x & 3u))));
    let _e99 = S0_1;
    let _e101 = O0_1;
    let _e102 = N0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e105 = (lj && (u32(_e99) != 0u));
            if (_e102.w >= 0f) {
                phi_1633_ = _e102;
            } else {
                let _e109 = j.Q8_;
                let _e111 = j.R8_;
                let _e114 = abs(_e102.z);
                let _e115 = floor(_e114);
                let _e118 = (((_e114 - _e115) * 8f) + 0.0009765625f);
                let _e120 = ((_e115 * _e109) + _e111);
                let _e121 = floor(_e118);
                switch bitcast<i32>(0u) {
                    default: {
                        if (sj && (_e121 == 4f)) {
                            let _e140 = normalize(vec2<f32>(_e102.x, -(_e102.y)));
                            let _e143 = acos(clamp(_e140.x, -1f, 1f));
                            if (_e140.y >= 0f) {
                                phi_1616_ = _e143;
                            } else {
                                phi_1616_ = -(_e143);
                            }
                            let _e148 = phi_1616_;
                            phi_1617_ = vec2<f32>((_e148 * 0.15915494f), _e120);
                            break;
                        } else {
                            if (_e121 == 2f) {
                                phi_1615_ = _e102.x;
                            } else {
                                phi_1615_ = length(_e102.xy);
                            }
                            let _e130 = phi_1615_;
                            phi_1617_ = vec2<f32>(((clamp(_e130, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e102.z < 0f))) + _e118), _e120);
                            break;
                        }
                    }
                }
                let _e152 = phi_1617_;
                let _e153 = textureSampleLevel(ZC, S8_, _e152, 0f);
                phi_1634_ = _e153;
                if !(_e105) {
                    let _e159 = (_e153.xyz * _e153.w);
                    phi_1634_ = vec4<f32>(_e159.x, _e159.y, _e159.z, (_e153.w * (ceil(_e102.w) * -0.003921569f)));
                }
                let _e166 = phi_1634_;
                phi_1633_ = _e166;
            }
            let _e168 = phi_1633_;
            phi_1030_ = nj;
            if nj {
                phi_1030_ = (_e101.z < 0f);
            }
            let _e172 = phi_1030_;
            if _e172 {
                let _e174 = textureSampleLevel(RB, v4_, _e101.xy, 0f);
                phi_1637_ = _e174;
                break;
            }
            phi_1045_ = nj;
            if nj {
                phi_1045_ = (_e101.z > 0f);
            }
            let _e178 = phi_1045_;
            phi_1636_ = _e168;
            if _e178 {
                let _e182 = textureSampleLevel(RB, v4_, _e101.xy, (_e101.z - 1f));
                phi_1629_ = _e182;
                if _e105 {
                    if (_e182.w != 0f) {
                        phi_1618_ = (1f / _e182.w);
                    } else {
                        phi_1618_ = 0f;
                    }
                    let _e188 = phi_1618_;
                    let _e189 = (_e182.xyz * _e188);
                    phi_1629_ = vec4<f32>(_e189.x, _e189.y, _e189.z, _e182.w);
                }
                let _e195 = phi_1629_;
                phi_1636_ = (_e168 * _e195);
            }
            let _e198 = phi_1636_;
            phi_1637_ = _e198;
            break;
        }
    }
    let _e200 = phi_1637_;
    let _e201 = p1_1;
    let _e202 = P4_1;
    let _e205 = A3_1[1u];
    let _e207 = A3_1[0u];
    let _e208 = vec2<u32>(floor(_e202));
    phi_1639_ = 1f;
    if kj {
        let _e236 = W0_1;
        let _e239 = min(_e236.xy, _e236.zw);
        phi_1639_ = min(min(_e239.x, _e239.y), 1f);
    }
    let _e245 = phi_1639_;
    phi_812_ = jj;
    if jj {
        let _e247 = k2_1[0u];
        phi_812_ = (_e247 != 0f);
    }
    let _e250 = phi_812_;
    phi_1640_ = _e245;
    if _e250 {
        let _e253 = m0_.v2_[_e98];
        phi_1640_ = min(unpack4x8unorm(_e253).x, _e245);
    }
    let _e258 = phi_1640_;
    let _e260 = clamp(_e201, 0f, max(_e258, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e266 = u32(((abs(_e260) * 1024f) + 0.5f));
            let _e269 = atomicLoad((&Y0_.v2_[(_e207 + (((((_e208.y >> bitcast<u32>(5u)) * (_e205 << bitcast<u32>(5u))) + ((_e208.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e208.x & 28u) << bitcast<u32>(5u)) + ((_e208.y & 28u) << bitcast<u32>(2i)))) + (((_e208.y & 3u) << bitcast<u32>(2i)) + (_e208.x & 3u))))]));
            let _e271 = (min(_e200.w, _e260) >= 1f);
            phi_1342_ = _e271;
            if _e271 {
                let _e273 = j.r2_;
                let _e274 = (_e269 < _e273);
                phi_1340_ = _e274;
                if !(_e274) {
                    phi_1340_ = (_e269 >= (_e273 | 262144u));
                }
                let _e279 = phi_1340_;
                phi_1342_ = _e279;
            }
            let _e281 = phi_1342_;
            if _e281 {
                phi_1663_ = 1f;
                break;
            }
            let _e283 = j.r2_;
            phi_1657_ = 0f;
            phi_1654_ = _e266;
            phi_1651_ = _e260;
            if (_e269 < _e283) {
                let _e286 = (_e283 | (262144u + _e266));
                let _e287 = atomicMax((&Y0_.v2_[(_e207 + (((((_e208.y >> bitcast<u32>(5u)) * (_e205 << bitcast<u32>(5u))) + ((_e208.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e208.x & 28u) << bitcast<u32>(5u)) + ((_e208.y & 28u) << bitcast<u32>(2i)))) + (((_e208.y & 3u) << bitcast<u32>(2i)) + (_e208.x & 3u))))]), _e286);
                if (_e287 <= _e283) {
                    phi_1660_ = min(_e260, 1f);
                    phi_1655_ = _e266;
                    phi_1652_ = 0f;
                } else {
                    phi_1661_ = 0f;
                    phi_1656_ = _e266;
                    phi_1653_ = _e260;
                    if (_e287 < _e286) {
                        let _e291 = ((_e287 & 524287u) - 262144u);
                        let _e293 = (f32(_e291) * 0.0009765625f);
                        phi_1661_ = ((min(_e260, 1f) - _e293) / max((1f - (_e293 * _e200.w)), 0.000062f));
                        phi_1656_ = _e291;
                        phi_1653_ = _e293;
                    }
                    let _e301 = phi_1661_;
                    let _e303 = phi_1656_;
                    let _e305 = phi_1653_;
                    phi_1660_ = _e301;
                    phi_1655_ = _e303;
                    phi_1652_ = _e305;
                }
                let _e308 = phi_1660_;
                let _e310 = phi_1655_;
                let _e312 = phi_1652_;
                phi_1657_ = _e308;
                phi_1654_ = _e310;
                phi_1651_ = _e312;
            }
            let _e314 = phi_1657_;
            let _e316 = phi_1654_;
            let _e318 = phi_1651_;
            phi_1662_ = _e314;
            if (_e318 > 0f) {
                let _e320 = atomicAdd((&Y0_.v2_[(_e207 + (((((_e208.y >> bitcast<u32>(5u)) * (_e205 << bitcast<u32>(5u))) + ((_e208.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e208.x & 28u) << bitcast<u32>(5u)) + ((_e208.y & 28u) << bitcast<u32>(2i)))) + (((_e208.y & 3u) << bitcast<u32>(2i)) + (_e208.x & 3u))))]), _e316);
                let _e325 = (f32(bitcast<i32>(((_e320 & 524287u) - 262144u))) * 0.0009765625f);
                let _e327 = clamp(_e325, 0f, 1f);
                phi_1662_ = (_e314 + ((1f - (_e314 * _e200.w)) * ((clamp((_e325 + _e318), 0f, 1f) - _e327) / max((1f - (_e327 * _e200.w)), 0.000062f))));
            }
            let _e339 = phi_1662_;
            phi_1663_ = _e339;
            break;
        }
    }
    let _e341 = phi_1663_;
    phi_1674_ = f32();
    if rj {
        let _e343 = j.H3_;
        let _e345 = j.I3_;
        if rj {
            phi_1664_ = ((fract((52.982918f * fract(((0.06711056f * _e63.x) + (0.00583715f * _e63.y))))) * _e343) + _e345);
        } else {
            phi_1664_ = 0f;
        }
        let _e357 = phi_1664_;
        phi_1674_ = _e357;
    }
    let _e359 = phi_1674_;
    let _e360 = (_e200 * _e341);
    let _e361 = _e360.xyz;
    if (rj && (_e360.w != 0f)) {
        phi_1697_ = (vec3(_e359) + _e361);
    } else {
        phi_1697_ = _e361;
    }
    let _e368 = phi_1697_;
    let _e374 = vec4<f32>(_e368.x, _e360.y, _e360.z, _e360.w);
    let _e380 = vec4<f32>(_e374.x, _e368.y, _e374.z, _e374.w);
    m0_.v2_[_e98] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    Q1_ = vec4<f32>(_e380.x, _e380.y, _e368.z, _e380.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(9) O0_: vec3<f32>, @location(6) @interpolate(flat, either) S0_: f32, @location(0) N0_: vec4<f32>, @location(1) @interpolate(flat, either) p1_: f32, @location(8) P4_: vec2<f32>, @location(7) @interpolate(flat, either) A3_: vec2<u32>, @location(5) W0_: vec4<f32>, @location(4) @interpolate(flat, either) k2_: vec2<f32>, @location(3) @interpolate(flat, either) I0_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    O0_1 = O0_;
    S0_1 = S0_;
    N0_1 = N0_;
    p1_1 = p1_;
    P4_1 = P4_;
    A3_1 = A3_;
    W0_1 = W0_;
    k2_1 = k2_;
    I0_1 = I0_;
    main_1();
    let _e21 = Q1_;
    return _e21;
}
