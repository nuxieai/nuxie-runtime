struct VB {
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
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

@id(7) override dj: bool = true;
@id(2) override Yi: bool = true;
@id(8) override ej: bool = true;
@id(1) override Xi: bool = true;
@id(0) override Wi: bool = true;

@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var N8_: sampler;
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
    var phi_1511_: f32;
    var phi_1526_: vec4<f32>;
    var phi_1525_: vec4<f32>;
    var phi_976_: bool;
    var phi_991_: bool;
    var phi_1512_: f32;
    var phi_1521_: vec4<f32>;
    var phi_1528_: vec4<f32>;
    var phi_1529_: vec4<f32>;
    var phi_1531_: f32;
    var phi_763_: bool;
    var phi_1532_: f32;
    var phi_1241_: bool;
    var phi_1243_: bool;
    var phi_1553_: f32;
    var phi_1548_: u32;
    var phi_1545_: f32;
    var phi_1552_: f32;
    var phi_1547_: u32;
    var phi_1544_: f32;
    var phi_1549_: f32;
    var phi_1546_: u32;
    var phi_1543_: f32;
    var phi_1554_: f32;
    var phi_1555_: f32;
    var phi_1556_: f32;
    var phi_1566_: f32;
    var phi_1587_: vec3<f32>;

    let _e59 = gl_FragCoord_1;
    let _e63 = bitcast<vec2<u32>>(vec2<i32>(floor(_e59.xy)));
    let _e65 = j.P6_;
    let _e94 = bitcast<i32>((((((_e63.y >> bitcast<u32>(5u)) * (((_e65 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e63.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e63.x & 28u) << bitcast<u32>(5u)) + ((_e63.y & 28u) << bitcast<u32>(2i)))) + (((_e63.y & 3u) << bitcast<u32>(2i)) + (_e63.x & 3u))));
    let _e95 = P0_1;
    let _e97 = U0_1;
    let _e98 = O0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e101 = (Yi && (u32(_e95) != 0u));
            if (_e98.w >= 0f) {
                phi_1525_ = _e98;
            } else {
                let _e105 = j.L8_;
                let _e107 = j.M8_;
                let _e110 = abs(_e98.z);
                let _e111 = floor(_e110);
                let _e114 = (((_e110 - _e111) * 8f) + 0.0009765625f);
                if (floor(_e114) == 2f) {
                    phi_1511_ = _e98.x;
                } else {
                    phi_1511_ = length(_e98.xy);
                }
                let _e123 = phi_1511_;
                let _e129 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e123, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e98.z < 0f))) + _e114), ((_e111 * _e105) + _e107)), 0f);
                phi_1526_ = _e129;
                if !(_e101) {
                    let _e135 = (_e129.xyz * _e129.w);
                    phi_1526_ = vec4<f32>(_e135.x, _e135.y, _e135.z, (_e129.w * (ceil(_e98.w) * -0.003921569f)));
                }
                let _e142 = phi_1526_;
                phi_1525_ = _e142;
            }
            let _e144 = phi_1525_;
            phi_976_ = ej;
            if ej {
                phi_976_ = (_e97.z < 0f);
            }
            let _e148 = phi_976_;
            if _e148 {
                let _e150 = textureSampleLevel(TB, U4_, _e97.xy, 0f);
                phi_1529_ = _e150;
                break;
            }
            phi_991_ = ej;
            if ej {
                phi_991_ = (_e97.z > 0f);
            }
            let _e154 = phi_991_;
            phi_1528_ = _e144;
            if _e154 {
                let _e158 = textureSampleLevel(TB, U4_, _e97.xy, (_e97.z - 1f));
                phi_1521_ = _e158;
                if _e101 {
                    if (_e158.w != 0f) {
                        phi_1512_ = (1f / _e158.w);
                    } else {
                        phi_1512_ = 0f;
                    }
                    let _e164 = phi_1512_;
                    let _e165 = (_e158.xyz * _e164);
                    phi_1521_ = vec4<f32>(_e165.x, _e165.y, _e165.z, _e158.w);
                }
                let _e171 = phi_1521_;
                phi_1528_ = (_e144 * _e171);
            }
            let _e174 = phi_1528_;
            phi_1529_ = _e174;
            break;
        }
    }
    let _e176 = phi_1529_;
    let _e177 = n1_1;
    let _e178 = L4_1;
    let _e181 = z3_1[1u];
    let _e183 = z3_1[0u];
    let _e184 = vec2<u32>(floor(_e178));
    phi_1531_ = 1f;
    if Xi {
        let _e212 = V0_1;
        let _e215 = min(_e212.xy, _e212.zw);
        phi_1531_ = min(min(_e215.x, _e215.y), 1f);
    }
    let _e221 = phi_1531_;
    phi_763_ = Wi;
    if Wi {
        let _e223 = i2_1[0u];
        phi_763_ = (_e223 != 0f);
    }
    let _e226 = phi_763_;
    phi_1532_ = _e221;
    if _e226 {
        let _e229 = m0_.r2_[_e94];
        phi_1532_ = min(unpack4x8unorm(_e229).x, _e221);
    }
    let _e234 = phi_1532_;
    let _e236 = clamp(_e177, 0f, max(_e234, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e242 = u32(((abs(_e236) * 1024f) + 0.5f));
            let _e245 = atomicLoad((&Y0_.r2_[(_e183 + (((((_e184.y >> bitcast<u32>(5u)) * (_e181 << bitcast<u32>(5u))) + ((_e184.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e184.x & 28u) << bitcast<u32>(5u)) + ((_e184.y & 28u) << bitcast<u32>(2i)))) + (((_e184.y & 3u) << bitcast<u32>(2i)) + (_e184.x & 3u))))]));
            let _e247 = (min(_e176.w, _e236) >= 1f);
            phi_1243_ = _e247;
            if _e247 {
                let _e249 = j.q2_;
                let _e250 = (_e245 < _e249);
                phi_1241_ = _e250;
                if !(_e250) {
                    phi_1241_ = (_e245 >= (_e249 | 262144u));
                }
                let _e255 = phi_1241_;
                phi_1243_ = _e255;
            }
            let _e257 = phi_1243_;
            if _e257 {
                phi_1555_ = 1f;
                break;
            }
            let _e259 = j.q2_;
            phi_1549_ = 0f;
            phi_1546_ = _e242;
            phi_1543_ = _e236;
            if (_e245 < _e259) {
                let _e262 = (_e259 | (262144u + _e242));
                let _e263 = atomicMax((&Y0_.r2_[(_e183 + (((((_e184.y >> bitcast<u32>(5u)) * (_e181 << bitcast<u32>(5u))) + ((_e184.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e184.x & 28u) << bitcast<u32>(5u)) + ((_e184.y & 28u) << bitcast<u32>(2i)))) + (((_e184.y & 3u) << bitcast<u32>(2i)) + (_e184.x & 3u))))]), _e262);
                if (_e263 <= _e259) {
                    phi_1552_ = min(_e236, 1f);
                    phi_1547_ = _e242;
                    phi_1544_ = 0f;
                } else {
                    phi_1553_ = 0f;
                    phi_1548_ = _e242;
                    phi_1545_ = _e236;
                    if (_e263 < _e262) {
                        let _e268 = ((_e263 & 524287u) - 262144u);
                        let _e270 = (f32(_e268) * 0.0009765625f);
                        phi_1553_ = ((min(_e236, 1f) - _e270) / max((1f - (_e270 * _e176.w)), 0.000062f));
                        phi_1548_ = _e268;
                        phi_1545_ = _e270;
                    }
                    let _e278 = phi_1553_;
                    let _e280 = phi_1548_;
                    let _e282 = phi_1545_;
                    phi_1552_ = _e278;
                    phi_1547_ = _e280;
                    phi_1544_ = _e282;
                }
                let _e284 = phi_1552_;
                let _e286 = phi_1547_;
                let _e288 = phi_1544_;
                phi_1549_ = _e284;
                phi_1546_ = _e286;
                phi_1543_ = _e288;
            }
            let _e290 = phi_1549_;
            let _e292 = phi_1546_;
            let _e294 = phi_1543_;
            phi_1554_ = _e290;
            if (_e294 > 0f) {
                let _e296 = atomicAdd((&Y0_.r2_[(_e183 + (((((_e184.y >> bitcast<u32>(5u)) * (_e181 << bitcast<u32>(5u))) + ((_e184.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e184.x & 28u) << bitcast<u32>(5u)) + ((_e184.y & 28u) << bitcast<u32>(2i)))) + (((_e184.y & 3u) << bitcast<u32>(2i)) + (_e184.x & 3u))))]), _e292);
                let _e301 = (f32(bitcast<i32>(((_e296 & 524287u) - 262144u))) * 0.0009765625f);
                let _e303 = clamp(_e301, 0f, 1f);
                phi_1554_ = (_e290 + ((1f - (_e290 * _e176.w)) * ((clamp((_e301 + _e294), 0f, 1f) - _e303) / max((1f - (_e303 * _e176.w)), 0.000062f))));
            }
            let _e315 = phi_1554_;
            phi_1555_ = _e315;
            break;
        }
    }
    let _e317 = phi_1555_;
    phi_1566_ = f32();
    if dj {
        let _e319 = j.F3_;
        let _e321 = j.G3_;
        if dj {
            phi_1556_ = ((fract((52.982918f * fract(((0.06711056f * _e59.x) + (0.00583715f * _e59.y))))) * _e319) + _e321);
        } else {
            phi_1556_ = 0f;
        }
        let _e333 = phi_1556_;
        phi_1566_ = _e333;
    }
    let _e335 = phi_1566_;
    let _e336 = (_e176 * _e317);
    let _e337 = _e336.xyz;
    if (dj && (_e336.w != 0f)) {
        phi_1587_ = (vec3(_e335) + _e337);
    } else {
        phi_1587_ = _e337;
    }
    let _e344 = phi_1587_;
    let _e350 = vec4<f32>(_e344.x, _e336.y, _e336.z, _e336.w);
    let _e356 = vec4<f32>(_e350.x, _e344.y, _e350.z, _e350.w);
    m0_.r2_[_e94] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    L1_ = vec4<f32>(_e356.x, _e356.y, _e344.z, _e356.w);
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
