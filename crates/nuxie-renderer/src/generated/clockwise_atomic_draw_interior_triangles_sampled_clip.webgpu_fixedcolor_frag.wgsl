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
    Ci: vec2<f32>,
    Fe: vec2<f32>,
    q2_: u32,
    Gi: f32,
    w6_: u32,
    i3_: f32,
    Ge: f32,
    og: u32,
    G3_: f32,
    H3_: f32,
    He: f32,
    zi: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

struct Bf {
    r2_: array<u32>,
}

struct Bf_1 {
    r2_: array<atomic<u32>>,
}

@id(7) override hj: bool = true;
@id(2) override cj: bool = true;
@id(8) override ij: bool = true;
@id(1) override bj: bool = true;
@id(0) override aj: bool = true;

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
@group(0) @binding(6)
var<storage, read_write> Y0_: Bf_1;
var<private> U0_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> P0_1: vec4<f32>;
var<private> o1_1: f32;
var<private> L4_1: vec2<f32>;
var<private> A3_1: vec2<u32>;
var<private> V0_1: vec4<f32>;
var<private> i2_1: vec2<f32>;
@group(2) @binding(1)
var m0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> N1_: vec4<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
var<private> G0_1: f32;

fn main_1() {
    var phi_1508_: f32;
    var phi_1509_: f32;
    var phi_1510_: vec2<f32>;
    var phi_1527_: vec4<f32>;
    var phi_1526_: vec4<f32>;
    var phi_950_: bool;
    var phi_965_: bool;
    var phi_1511_: f32;
    var phi_1522_: vec4<f32>;
    var phi_1529_: vec4<f32>;
    var phi_1530_: vec4<f32>;
    var phi_1532_: f32;
    var phi_772_: bool;
    var phi_1533_: f32;
    var phi_1259_: bool;
    var phi_1261_: bool;
    var phi_1554_: f32;
    var phi_1549_: u32;
    var phi_1546_: f32;
    var phi_1553_: f32;
    var phi_1548_: u32;
    var phi_1545_: f32;
    var phi_1550_: f32;
    var phi_1547_: u32;
    var phi_1544_: f32;
    var phi_1555_: f32;
    var phi_1556_: f32;
    var phi_1557_: f32;
    var phi_1567_: f32;
    var phi_1590_: vec3<f32>;

    let _e58 = Q0_1;
    let _e60 = U0_1;
    let _e61 = P0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e64 = (cj && (u32(_e58) != 0u));
            if (_e61.w >= 0f) {
                phi_1526_ = _e61;
            } else {
                let _e68 = j.O8_;
                let _e70 = j.P8_;
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
                                phi_1509_ = _e101;
                            } else {
                                phi_1509_ = -(_e101);
                            }
                            let _e106 = phi_1509_;
                            phi_1510_ = vec2<f32>((_e106 * 0.15915507f), _e79);
                            break;
                        } else {
                            if (_e80 == 2f) {
                                phi_1508_ = _e61.x;
                            } else {
                                phi_1508_ = length(_e61.xy);
                            }
                            let _e88 = phi_1508_;
                            phi_1510_ = vec2<f32>(((clamp(_e88, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e61.z < 0f))) + _e77), _e79);
                            break;
                        }
                    }
                }
                let _e110 = phi_1510_;
                let _e111 = textureSampleLevel(ZC, Q8_, _e110, 0f);
                phi_1527_ = _e111;
                if !(_e64) {
                    let _e117 = (_e111.xyz * _e111.w);
                    phi_1527_ = vec4<f32>(_e117.x, _e117.y, _e117.z, (_e111.w * (ceil(_e61.w) * -0.003921569f)));
                }
                let _e124 = phi_1527_;
                phi_1526_ = _e124;
            }
            let _e126 = phi_1526_;
            phi_950_ = ij;
            if ij {
                phi_950_ = (_e60.z < 0f);
            }
            let _e130 = phi_950_;
            if _e130 {
                let _e132 = textureSampleLevel(TB, U4_, _e60.xy, 0f);
                phi_1530_ = _e132;
                break;
            }
            phi_965_ = ij;
            if ij {
                phi_965_ = (_e60.z > 0f);
            }
            let _e136 = phi_965_;
            phi_1529_ = _e126;
            if _e136 {
                let _e140 = textureSampleLevel(TB, U4_, _e60.xy, (_e60.z - 1f));
                phi_1522_ = _e140;
                if _e64 {
                    if (_e140.w != 0f) {
                        phi_1511_ = (1f / _e140.w);
                    } else {
                        phi_1511_ = 0f;
                    }
                    let _e146 = phi_1511_;
                    let _e147 = (_e140.xyz * _e146);
                    phi_1522_ = vec4<f32>(_e147.x, _e147.y, _e147.z, _e140.w);
                }
                let _e153 = phi_1522_;
                phi_1529_ = (_e126 * _e153);
            }
            let _e156 = phi_1529_;
            phi_1530_ = _e156;
            break;
        }
    }
    let _e158 = phi_1530_;
    let _e159 = o1_1;
    let _e160 = L4_1;
    let _e163 = A3_1[1u];
    let _e165 = A3_1[0u];
    let _e166 = vec2<u32>(floor(_e160));
    phi_1532_ = 1f;
    if bj {
        let _e194 = V0_1;
        let _e197 = min(_e194.xy, _e194.zw);
        phi_1532_ = min(min(_e197.x, _e197.y), 1f);
    }
    let _e203 = phi_1532_;
    phi_772_ = aj;
    if aj {
        let _e205 = i2_1[0u];
        phi_772_ = (_e205 != 0f);
    }
    let _e208 = phi_772_;
    phi_1533_ = _e203;
    if _e208 {
        let _e209 = gl_FragCoord_1;
        let _e213 = textureLoad(m0_, vec2<i32>(floor(_e209.xy)), 0i);
        phi_1533_ = min(_e213.x, _e203);
    }
    let _e217 = phi_1533_;
    let _e219 = clamp(_e159, 0f, max(_e217, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e225 = u32(((abs(_e219) * 1024f) + 0.5f));
            let _e228 = atomicLoad((&Y0_.r2_[(_e165 + (((((_e166.y >> bitcast<u32>(5u)) * (_e163 << bitcast<u32>(5u))) + ((_e166.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e166.x & 28u) << bitcast<u32>(5u)) + ((_e166.y & 28u) << bitcast<u32>(2i)))) + (((_e166.y & 3u) << bitcast<u32>(2i)) + (_e166.x & 3u))))]));
            let _e230 = (min(_e158.w, _e219) >= 1f);
            phi_1261_ = _e230;
            if _e230 {
                let _e232 = j.q2_;
                let _e233 = (_e228 < _e232);
                phi_1259_ = _e233;
                if !(_e233) {
                    phi_1259_ = (_e228 >= (_e232 | 262144u));
                }
                let _e238 = phi_1259_;
                phi_1261_ = _e238;
            }
            let _e240 = phi_1261_;
            if _e240 {
                phi_1556_ = 1f;
                break;
            }
            let _e242 = j.q2_;
            phi_1550_ = 0f;
            phi_1547_ = _e225;
            phi_1544_ = _e219;
            if (_e228 < _e242) {
                let _e245 = (_e242 | (262144u + _e225));
                let _e246 = atomicMax((&Y0_.r2_[(_e165 + (((((_e166.y >> bitcast<u32>(5u)) * (_e163 << bitcast<u32>(5u))) + ((_e166.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e166.x & 28u) << bitcast<u32>(5u)) + ((_e166.y & 28u) << bitcast<u32>(2i)))) + (((_e166.y & 3u) << bitcast<u32>(2i)) + (_e166.x & 3u))))]), _e245);
                if (_e246 <= _e242) {
                    phi_1553_ = min(_e219, 1f);
                    phi_1548_ = _e225;
                    phi_1545_ = 0f;
                } else {
                    phi_1554_ = 0f;
                    phi_1549_ = _e225;
                    phi_1546_ = _e219;
                    if (_e246 < _e245) {
                        let _e250 = ((_e246 & 524287u) - 262144u);
                        let _e252 = (f32(_e250) * 0.0009765625f);
                        phi_1554_ = ((min(_e219, 1f) - _e252) / max((1f - (_e252 * _e158.w)), 0.000062f));
                        phi_1549_ = _e250;
                        phi_1546_ = _e252;
                    }
                    let _e260 = phi_1554_;
                    let _e262 = phi_1549_;
                    let _e264 = phi_1546_;
                    phi_1553_ = _e260;
                    phi_1548_ = _e262;
                    phi_1545_ = _e264;
                }
                let _e267 = phi_1553_;
                let _e269 = phi_1548_;
                let _e271 = phi_1545_;
                phi_1550_ = _e267;
                phi_1547_ = _e269;
                phi_1544_ = _e271;
            }
            let _e273 = phi_1550_;
            let _e275 = phi_1547_;
            let _e277 = phi_1544_;
            phi_1555_ = _e273;
            if (_e277 > 0f) {
                let _e279 = atomicAdd((&Y0_.r2_[(_e165 + (((((_e166.y >> bitcast<u32>(5u)) * (_e163 << bitcast<u32>(5u))) + ((_e166.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e166.x & 28u) << bitcast<u32>(5u)) + ((_e166.y & 28u) << bitcast<u32>(2i)))) + (((_e166.y & 3u) << bitcast<u32>(2i)) + (_e166.x & 3u))))]), _e275);
                let _e284 = (f32(bitcast<i32>(((_e279 & 524287u) - 262144u))) * 0.0009765625f);
                let _e286 = clamp(_e284, 0f, 1f);
                phi_1555_ = (_e273 + ((1f - (_e273 * _e158.w)) * ((clamp((_e284 + _e277), 0f, 1f) - _e286) / max((1f - (_e286 * _e158.w)), 0.000062f))));
            }
            let _e298 = phi_1555_;
            phi_1556_ = _e298;
            break;
        }
    }
    let _e300 = phi_1556_;
    phi_1567_ = f32();
    if hj {
        let _e301 = gl_FragCoord_1;
        let _e303 = j.G3_;
        let _e305 = j.H3_;
        if hj {
            phi_1557_ = ((fract((52.982918f * fract(((0.06711056f * _e301.x) + (0.00583715f * _e301.y))))) * _e303) + _e305);
        } else {
            phi_1557_ = 0f;
        }
        let _e317 = phi_1557_;
        phi_1567_ = _e317;
    }
    let _e319 = phi_1567_;
    let _e320 = (_e158 * _e300);
    let _e321 = _e320.xyz;
    if (hj && (_e320.w != 0f)) {
        phi_1590_ = (vec3(_e319) + _e321);
    } else {
        phi_1590_ = _e321;
    }
    let _e328 = phi_1590_;
    let _e334 = vec4<f32>(_e328.x, _e320.y, _e320.z, _e320.w);
    let _e340 = vec4<f32>(_e334.x, _e328.y, _e334.z, _e334.w);
    N1_ = vec4<f32>(_e340.x, _e340.y, _e328.z, _e340.w);
    return;
}

@fragment
fn main(@location(9) U0_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) P0_: vec4<f32>, @location(1) @interpolate(flat, either) o1_: f32, @location(8) L4_: vec2<f32>, @location(7) @interpolate(flat, either) A3_: vec2<u32>, @location(5) V0_: vec4<f32>, @location(4) @interpolate(flat, either) i2_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32) -> @location(0) vec4<f32> {
    U0_1 = U0_;
    Q0_1 = Q0_;
    P0_1 = P0_;
    o1_1 = o1_;
    L4_1 = L4_;
    A3_1 = A3_;
    V0_1 = V0_;
    i2_1 = i2_;
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    main_1();
    let _e21 = N1_;
    return _e21;
}
