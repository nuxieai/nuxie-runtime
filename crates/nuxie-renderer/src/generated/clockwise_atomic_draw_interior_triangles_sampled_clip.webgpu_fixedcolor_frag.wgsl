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

struct Ef {
    v2_: array<u32>,
}

struct Ef_1 {
    v2_: array<atomic<u32>>,
}

@id(8) override nj: bool = true;
@id(9) override oj: bool = true;
@id(2) override hj: bool = true;
@id(4) override jj: bool = true;
@id(1) override gj: bool = true;
@id(0) override fj: bool = true;

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
@group(0) @binding(6)
var<storage, read_write> Y0_: Ef_1;
var<private> V0_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> P0_1: vec4<f32>;
var<private> p1_1: f32;
var<private> N4_1: vec2<f32>;
var<private> A3_1: vec2<u32>;
var<private> W0_1: vec4<f32>;
var<private> k2_1: vec2<f32>;
@group(2) @binding(1)
var m0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Q1_: vec4<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
var<private> G0_1: f32;

fn main_1() {
    var phi_1511_: f32;
    var phi_1512_: f32;
    var phi_1513_: vec2<f32>;
    var phi_1530_: vec4<f32>;
    var phi_1529_: vec4<f32>;
    var phi_952_: bool;
    var phi_967_: bool;
    var phi_1514_: f32;
    var phi_1525_: vec4<f32>;
    var phi_1532_: vec4<f32>;
    var phi_1533_: vec4<f32>;
    var phi_1535_: f32;
    var phi_774_: bool;
    var phi_1536_: f32;
    var phi_1262_: bool;
    var phi_1264_: bool;
    var phi_1557_: f32;
    var phi_1552_: u32;
    var phi_1549_: f32;
    var phi_1556_: f32;
    var phi_1551_: u32;
    var phi_1548_: f32;
    var phi_1553_: f32;
    var phi_1550_: u32;
    var phi_1547_: f32;
    var phi_1558_: f32;
    var phi_1559_: f32;
    var phi_1560_: f32;
    var phi_1570_: f32;
    var phi_1593_: vec3<f32>;

    let _e59 = Q0_1;
    let _e61 = V0_1;
    let _e62 = P0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e65 = (hj && (u32(_e59) != 0u));
            if (_e62.w >= 0f) {
                phi_1529_ = _e62;
            } else {
                let _e69 = j.P8_;
                let _e71 = j.Q8_;
                let _e74 = abs(_e62.z);
                let _e75 = floor(_e74);
                let _e78 = (((_e74 - _e75) * 8f) + 0.0009765625f);
                let _e80 = ((_e75 * _e69) + _e71);
                let _e81 = floor(_e78);
                switch bitcast<i32>(0u) {
                    default: {
                        if (oj && (_e81 == 4f)) {
                            let _e100 = normalize(vec2<f32>(_e62.x, -(_e62.y)));
                            let _e103 = acos(clamp(_e100.x, -1f, 1f));
                            if (_e100.y >= 0f) {
                                phi_1512_ = _e103;
                            } else {
                                phi_1512_ = -(_e103);
                            }
                            let _e108 = phi_1512_;
                            phi_1513_ = vec2<f32>((_e108 * 0.15915494f), _e80);
                            break;
                        } else {
                            if (_e81 == 2f) {
                                phi_1511_ = _e62.x;
                            } else {
                                phi_1511_ = length(_e62.xy);
                            }
                            let _e90 = phi_1511_;
                            phi_1513_ = vec2<f32>(((clamp(_e90, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e62.z < 0f))) + _e78), _e80);
                            break;
                        }
                    }
                }
                let _e112 = phi_1513_;
                let _e113 = textureSampleLevel(ZC, R8_, _e112, 0f);
                phi_1530_ = _e113;
                if !(_e65) {
                    let _e119 = (_e113.xyz * _e113.w);
                    phi_1530_ = vec4<f32>(_e119.x, _e119.y, _e119.z, (_e113.w * (ceil(_e62.w) * -0.003921569f)));
                }
                let _e126 = phi_1530_;
                phi_1529_ = _e126;
            }
            let _e128 = phi_1529_;
            phi_952_ = jj;
            if jj {
                phi_952_ = (_e61.z < 0f);
            }
            let _e132 = phi_952_;
            if _e132 {
                let _e134 = textureSampleLevel(UB, W4_, _e61.xy, 0f);
                phi_1533_ = _e134;
                break;
            }
            phi_967_ = jj;
            if jj {
                phi_967_ = (_e61.z > 0f);
            }
            let _e138 = phi_967_;
            phi_1532_ = _e128;
            if _e138 {
                let _e142 = textureSampleLevel(UB, W4_, _e61.xy, (_e61.z - 1f));
                phi_1525_ = _e142;
                if _e65 {
                    if (_e142.w != 0f) {
                        phi_1514_ = (1f / _e142.w);
                    } else {
                        phi_1514_ = 0f;
                    }
                    let _e148 = phi_1514_;
                    let _e149 = (_e142.xyz * _e148);
                    phi_1525_ = vec4<f32>(_e149.x, _e149.y, _e149.z, _e142.w);
                }
                let _e155 = phi_1525_;
                phi_1532_ = (_e128 * _e155);
            }
            let _e158 = phi_1532_;
            phi_1533_ = _e158;
            break;
        }
    }
    let _e160 = phi_1533_;
    let _e161 = p1_1;
    let _e162 = N4_1;
    let _e165 = A3_1[1u];
    let _e167 = A3_1[0u];
    let _e168 = vec2<u32>(floor(_e162));
    phi_1535_ = 1f;
    if gj {
        let _e196 = W0_1;
        let _e199 = min(_e196.xy, _e196.zw);
        phi_1535_ = min(min(_e199.x, _e199.y), 1f);
    }
    let _e205 = phi_1535_;
    phi_774_ = fj;
    if fj {
        let _e207 = k2_1[0u];
        phi_774_ = (_e207 != 0f);
    }
    let _e210 = phi_774_;
    phi_1536_ = _e205;
    if _e210 {
        let _e211 = gl_FragCoord_1;
        let _e215 = textureLoad(m0_, vec2<i32>(floor(_e211.xy)), 0i);
        phi_1536_ = min(_e215.x, _e205);
    }
    let _e219 = phi_1536_;
    let _e221 = clamp(_e161, 0f, max(_e219, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e227 = u32(((abs(_e221) * 1024f) + 0.5f));
            let _e230 = atomicLoad((&Y0_.v2_[(_e167 + (((((_e168.y >> bitcast<u32>(5u)) * (_e165 << bitcast<u32>(5u))) + ((_e168.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e168.x & 28u) << bitcast<u32>(5u)) + ((_e168.y & 28u) << bitcast<u32>(2i)))) + (((_e168.y & 3u) << bitcast<u32>(2i)) + (_e168.x & 3u))))]));
            let _e232 = (min(_e160.w, _e221) >= 1f);
            phi_1264_ = _e232;
            if _e232 {
                let _e234 = j.r2_;
                let _e235 = (_e230 < _e234);
                phi_1262_ = _e235;
                if !(_e235) {
                    phi_1262_ = (_e230 >= (_e234 | 262144u));
                }
                let _e240 = phi_1262_;
                phi_1264_ = _e240;
            }
            let _e242 = phi_1264_;
            if _e242 {
                phi_1559_ = 1f;
                break;
            }
            let _e244 = j.r2_;
            phi_1553_ = 0f;
            phi_1550_ = _e227;
            phi_1547_ = _e221;
            if (_e230 < _e244) {
                let _e247 = (_e244 | (262144u + _e227));
                let _e248 = atomicMax((&Y0_.v2_[(_e167 + (((((_e168.y >> bitcast<u32>(5u)) * (_e165 << bitcast<u32>(5u))) + ((_e168.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e168.x & 28u) << bitcast<u32>(5u)) + ((_e168.y & 28u) << bitcast<u32>(2i)))) + (((_e168.y & 3u) << bitcast<u32>(2i)) + (_e168.x & 3u))))]), _e247);
                if (_e248 <= _e244) {
                    phi_1556_ = min(_e221, 1f);
                    phi_1551_ = _e227;
                    phi_1548_ = 0f;
                } else {
                    phi_1557_ = 0f;
                    phi_1552_ = _e227;
                    phi_1549_ = _e221;
                    if (_e248 < _e247) {
                        let _e252 = ((_e248 & 524287u) - 262144u);
                        let _e254 = (f32(_e252) * 0.0009765625f);
                        phi_1557_ = ((min(_e221, 1f) - _e254) / max((1f - (_e254 * _e160.w)), 0.000062f));
                        phi_1552_ = _e252;
                        phi_1549_ = _e254;
                    }
                    let _e262 = phi_1557_;
                    let _e264 = phi_1552_;
                    let _e266 = phi_1549_;
                    phi_1556_ = _e262;
                    phi_1551_ = _e264;
                    phi_1548_ = _e266;
                }
                let _e269 = phi_1556_;
                let _e271 = phi_1551_;
                let _e273 = phi_1548_;
                phi_1553_ = _e269;
                phi_1550_ = _e271;
                phi_1547_ = _e273;
            }
            let _e275 = phi_1553_;
            let _e277 = phi_1550_;
            let _e279 = phi_1547_;
            phi_1558_ = _e275;
            if (_e279 > 0f) {
                let _e281 = atomicAdd((&Y0_.v2_[(_e167 + (((((_e168.y >> bitcast<u32>(5u)) * (_e165 << bitcast<u32>(5u))) + ((_e168.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e168.x & 28u) << bitcast<u32>(5u)) + ((_e168.y & 28u) << bitcast<u32>(2i)))) + (((_e168.y & 3u) << bitcast<u32>(2i)) + (_e168.x & 3u))))]), _e277);
                let _e286 = (f32(bitcast<i32>(((_e281 & 524287u) - 262144u))) * 0.0009765625f);
                let _e288 = clamp(_e286, 0f, 1f);
                phi_1558_ = (_e275 + ((1f - (_e275 * _e160.w)) * ((clamp((_e286 + _e279), 0f, 1f) - _e288) / max((1f - (_e288 * _e160.w)), 0.000062f))));
            }
            let _e300 = phi_1558_;
            phi_1559_ = _e300;
            break;
        }
    }
    let _e302 = phi_1559_;
    phi_1570_ = f32();
    if nj {
        let _e303 = gl_FragCoord_1;
        let _e305 = j.G3_;
        let _e307 = j.H3_;
        if nj {
            phi_1560_ = ((fract((52.982918f * fract(((0.06711056f * _e303.x) + (0.00583715f * _e303.y))))) * _e305) + _e307);
        } else {
            phi_1560_ = 0f;
        }
        let _e319 = phi_1560_;
        phi_1570_ = _e319;
    }
    let _e321 = phi_1570_;
    let _e322 = (_e160 * _e302);
    let _e323 = _e322.xyz;
    if (nj && (_e322.w != 0f)) {
        phi_1593_ = (vec3(_e321) + _e323);
    } else {
        phi_1593_ = _e323;
    }
    let _e330 = phi_1593_;
    let _e336 = vec4<f32>(_e330.x, _e322.y, _e322.z, _e322.w);
    let _e342 = vec4<f32>(_e336.x, _e330.y, _e336.z, _e336.w);
    Q1_ = vec4<f32>(_e342.x, _e342.y, _e330.z, _e342.w);
    return;
}

@fragment
fn main(@location(9) V0_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) P0_: vec4<f32>, @location(1) @interpolate(flat, either) p1_: f32, @location(8) N4_: vec2<f32>, @location(7) @interpolate(flat, either) A3_: vec2<u32>, @location(5) W0_: vec4<f32>, @location(4) @interpolate(flat, either) k2_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32) -> @location(0) vec4<f32> {
    V0_1 = V0_;
    Q0_1 = Q0_;
    P0_1 = P0_;
    p1_1 = p1_;
    N4_1 = N4_;
    A3_1 = A3_;
    W0_1 = W0_;
    k2_1 = k2_;
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    main_1();
    let _e21 = Q1_;
    return _e21;
}
