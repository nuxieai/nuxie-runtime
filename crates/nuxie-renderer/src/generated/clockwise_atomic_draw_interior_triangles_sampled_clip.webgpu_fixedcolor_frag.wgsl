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
var<private> O0_1: vec3<f32>;
var<private> S0_1: f32;
var<private> N0_1: vec4<f32>;
var<private> p1_1: f32;
var<private> P4_1: vec2<f32>;
var<private> A3_1: vec2<u32>;
var<private> W0_1: vec4<f32>;
var<private> k2_1: vec2<f32>;
@group(2) @binding(1)
var m0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Q1_: vec4<f32>;
@group(3) @binding(9)
var eb: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
var<private> I0_1: f32;

fn main_1() {
    var phi_1499_: f32;
    var phi_1500_: f32;
    var phi_1501_: vec2<f32>;
    var phi_1518_: vec4<f32>;
    var phi_1517_: vec4<f32>;
    var phi_947_: bool;
    var phi_962_: bool;
    var phi_1502_: f32;
    var phi_1513_: vec4<f32>;
    var phi_1520_: vec4<f32>;
    var phi_1521_: vec4<f32>;
    var phi_1523_: f32;
    var phi_769_: bool;
    var phi_1524_: f32;
    var phi_1252_: bool;
    var phi_1254_: bool;
    var phi_1545_: f32;
    var phi_1540_: u32;
    var phi_1537_: f32;
    var phi_1544_: f32;
    var phi_1539_: u32;
    var phi_1536_: f32;
    var phi_1541_: f32;
    var phi_1538_: u32;
    var phi_1535_: f32;
    var phi_1546_: f32;
    var phi_1547_: f32;
    var phi_1548_: f32;
    var phi_1558_: f32;
    var phi_1581_: vec3<f32>;

    let _e59 = S0_1;
    let _e61 = O0_1;
    let _e62 = N0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e65 = (lj && (u32(_e59) != 0u));
            if (_e62.w >= 0f) {
                phi_1517_ = _e62;
            } else {
                let _e69 = j.Q8_;
                let _e71 = j.R8_;
                let _e74 = abs(_e62.z);
                let _e75 = floor(_e74);
                let _e78 = (((_e74 - _e75) * 8f) + 0.0009765625f);
                let _e80 = ((_e75 * _e69) + _e71);
                let _e81 = floor(_e78);
                let _e82 = _e62.xy;
                switch bitcast<i32>(0u) {
                    default: {
                        if (sj && (_e81 == 4f)) {
                            let _e96 = normalize(_e82);
                            let _e99 = acos(clamp(_e96.x, -1f, 1f));
                            if (_e96.y >= 0f) {
                                phi_1500_ = _e99;
                            } else {
                                phi_1500_ = -(_e99);
                            }
                            let _e104 = phi_1500_;
                            phi_1501_ = vec2<f32>((_e104 * 0.15915494f), _e80);
                            break;
                        } else {
                            if (_e81 == 2f) {
                                phi_1499_ = _e62.x;
                            } else {
                                phi_1499_ = length(_e82);
                            }
                            let _e90 = phi_1499_;
                            phi_1501_ = vec2<f32>(((clamp(_e90, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e62.z < 0f))) + _e78), _e80);
                            break;
                        }
                    }
                }
                let _e108 = phi_1501_;
                let _e109 = textureSampleLevel(ZC, S8_, _e108, 0f);
                phi_1518_ = _e109;
                if !(_e65) {
                    let _e115 = (_e109.xyz * _e109.w);
                    phi_1518_ = vec4<f32>(_e115.x, _e115.y, _e115.z, (_e109.w * (ceil(_e62.w) * -0.003921569f)));
                }
                let _e122 = phi_1518_;
                phi_1517_ = _e122;
            }
            let _e124 = phi_1517_;
            phi_947_ = nj;
            if nj {
                phi_947_ = (_e61.z < 0f);
            }
            let _e128 = phi_947_;
            if _e128 {
                let _e130 = textureSampleLevel(RB, v4_, _e61.xy, 0f);
                phi_1521_ = _e130;
                break;
            }
            phi_962_ = nj;
            if nj {
                phi_962_ = (_e61.z > 0f);
            }
            let _e134 = phi_962_;
            phi_1520_ = _e124;
            if _e134 {
                let _e138 = textureSampleLevel(RB, v4_, _e61.xy, (_e61.z - 1f));
                phi_1513_ = _e138;
                if _e65 {
                    if (_e138.w != 0f) {
                        phi_1502_ = (1f / _e138.w);
                    } else {
                        phi_1502_ = 0f;
                    }
                    let _e144 = phi_1502_;
                    let _e145 = (_e138.xyz * _e144);
                    phi_1513_ = vec4<f32>(_e145.x, _e145.y, _e145.z, _e138.w);
                }
                let _e151 = phi_1513_;
                phi_1520_ = (_e124 * _e151);
            }
            let _e154 = phi_1520_;
            phi_1521_ = _e154;
            break;
        }
    }
    let _e156 = phi_1521_;
    let _e157 = p1_1;
    let _e158 = P4_1;
    let _e161 = A3_1[1u];
    let _e163 = A3_1[0u];
    let _e164 = vec2<u32>(floor(_e158));
    phi_1523_ = 1f;
    if kj {
        let _e192 = W0_1;
        let _e195 = min(_e192.xy, _e192.zw);
        phi_1523_ = min(min(_e195.x, _e195.y), 1f);
    }
    let _e201 = phi_1523_;
    phi_769_ = jj;
    if jj {
        let _e203 = k2_1[0u];
        phi_769_ = (_e203 != 0f);
    }
    let _e206 = phi_769_;
    phi_1524_ = _e201;
    if _e206 {
        let _e207 = gl_FragCoord_1;
        let _e211 = textureLoad(m0_, vec2<i32>(floor(_e207.xy)), 0i);
        phi_1524_ = min(_e211.x, _e201);
    }
    let _e215 = phi_1524_;
    let _e217 = clamp(_e157, 0f, max(_e215, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e223 = u32(((abs(_e217) * 1024f) + 0.5f));
            let _e226 = atomicLoad((&Y0_.v2_[(_e163 + (((((_e164.y >> bitcast<u32>(5u)) * (_e161 << bitcast<u32>(5u))) + ((_e164.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e164.x & 28u) << bitcast<u32>(5u)) + ((_e164.y & 28u) << bitcast<u32>(2i)))) + (((_e164.y & 3u) << bitcast<u32>(2i)) + (_e164.x & 3u))))]));
            let _e228 = (min(_e156.w, _e217) >= 1f);
            phi_1254_ = _e228;
            if _e228 {
                let _e230 = j.r2_;
                let _e231 = (_e226 < _e230);
                phi_1252_ = _e231;
                if !(_e231) {
                    phi_1252_ = (_e226 >= (_e230 | 262144u));
                }
                let _e236 = phi_1252_;
                phi_1254_ = _e236;
            }
            let _e238 = phi_1254_;
            if _e238 {
                phi_1547_ = 1f;
                break;
            }
            let _e240 = j.r2_;
            phi_1541_ = 0f;
            phi_1538_ = _e223;
            phi_1535_ = _e217;
            if (_e226 < _e240) {
                let _e243 = (_e240 | (262144u + _e223));
                let _e244 = atomicMax((&Y0_.v2_[(_e163 + (((((_e164.y >> bitcast<u32>(5u)) * (_e161 << bitcast<u32>(5u))) + ((_e164.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e164.x & 28u) << bitcast<u32>(5u)) + ((_e164.y & 28u) << bitcast<u32>(2i)))) + (((_e164.y & 3u) << bitcast<u32>(2i)) + (_e164.x & 3u))))]), _e243);
                if (_e244 <= _e240) {
                    phi_1544_ = min(_e217, 1f);
                    phi_1539_ = _e223;
                    phi_1536_ = 0f;
                } else {
                    phi_1545_ = 0f;
                    phi_1540_ = _e223;
                    phi_1537_ = _e217;
                    if (_e244 < _e243) {
                        let _e248 = ((_e244 & 524287u) - 262144u);
                        let _e250 = (f32(_e248) * 0.0009765625f);
                        phi_1545_ = ((min(_e217, 1f) - _e250) / max((1f - (_e250 * _e156.w)), 0.000062f));
                        phi_1540_ = _e248;
                        phi_1537_ = _e250;
                    }
                    let _e258 = phi_1545_;
                    let _e260 = phi_1540_;
                    let _e262 = phi_1537_;
                    phi_1544_ = _e258;
                    phi_1539_ = _e260;
                    phi_1536_ = _e262;
                }
                let _e265 = phi_1544_;
                let _e267 = phi_1539_;
                let _e269 = phi_1536_;
                phi_1541_ = _e265;
                phi_1538_ = _e267;
                phi_1535_ = _e269;
            }
            let _e271 = phi_1541_;
            let _e273 = phi_1538_;
            let _e275 = phi_1535_;
            phi_1546_ = _e271;
            if (_e275 > 0f) {
                let _e277 = atomicAdd((&Y0_.v2_[(_e163 + (((((_e164.y >> bitcast<u32>(5u)) * (_e161 << bitcast<u32>(5u))) + ((_e164.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e164.x & 28u) << bitcast<u32>(5u)) + ((_e164.y & 28u) << bitcast<u32>(2i)))) + (((_e164.y & 3u) << bitcast<u32>(2i)) + (_e164.x & 3u))))]), _e273);
                let _e282 = (f32(bitcast<i32>(((_e277 & 524287u) - 262144u))) * 0.0009765625f);
                let _e284 = clamp(_e282, 0f, 1f);
                phi_1546_ = (_e271 + ((1f - (_e271 * _e156.w)) * ((clamp((_e282 + _e275), 0f, 1f) - _e284) / max((1f - (_e284 * _e156.w)), 0.000062f))));
            }
            let _e296 = phi_1546_;
            phi_1547_ = _e296;
            break;
        }
    }
    let _e298 = phi_1547_;
    phi_1558_ = f32();
    if rj {
        let _e299 = gl_FragCoord_1;
        let _e301 = j.H3_;
        let _e303 = j.I3_;
        if rj {
            phi_1548_ = ((fract((52.982918f * fract(((0.06711056f * _e299.x) + (0.00583715f * _e299.y))))) * _e301) + _e303);
        } else {
            phi_1548_ = 0f;
        }
        let _e315 = phi_1548_;
        phi_1558_ = _e315;
    }
    let _e317 = phi_1558_;
    let _e318 = (_e156 * _e298);
    let _e319 = _e318.xyz;
    if (rj && (_e318.w != 0f)) {
        phi_1581_ = (vec3(_e317) + _e319);
    } else {
        phi_1581_ = _e319;
    }
    let _e326 = phi_1581_;
    let _e332 = vec4<f32>(_e326.x, _e318.y, _e318.z, _e318.w);
    let _e338 = vec4<f32>(_e332.x, _e326.y, _e332.z, _e332.w);
    Q1_ = vec4<f32>(_e338.x, _e338.y, _e326.z, _e338.w);
    return;
}

@fragment
fn main(@location(9) O0_: vec3<f32>, @location(6) @interpolate(flat, either) S0_: f32, @location(0) N0_: vec4<f32>, @location(1) @interpolate(flat, either) p1_: f32, @location(8) P4_: vec2<f32>, @location(7) @interpolate(flat, either) A3_: vec2<u32>, @location(5) W0_: vec4<f32>, @location(4) @interpolate(flat, either) k2_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) I0_: f32) -> @location(0) vec4<f32> {
    O0_1 = O0_;
    S0_1 = S0_;
    N0_1 = N0_;
    p1_1 = p1_;
    P4_1 = P4_;
    A3_1 = A3_;
    W0_1 = W0_;
    k2_1 = k2_;
    gl_FragCoord_1 = gl_FragCoord;
    I0_1 = I0_;
    main_1();
    let _e21 = Q1_;
    return _e21;
}
