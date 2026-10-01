struct UB {
    Qc: f32,
    Td: f32,
    bg: f32,
    cg: f32,
    A6_: u32,
    X9_: u32,
    Nf: u32,
    Of: u32,
    j8_: vec4<i32>,
    Lh: vec2<f32>,
    Ud: vec2<f32>,
    j2_: u32,
    Ph: f32,
    T4_: u32,
    a3_: f32,
    Vd: f32,
    Hf: u32,
    L3_: f32,
    M3_: f32,
    Wd: f32,
    Ih: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

struct Te {
    k2_: array<u32>,
}

struct m0ge {
    k2_: array<u32>,
}

struct Te_1 {
    k2_: array<atomic<u32>>,
}

@id(7) override ri: bool = true;
@id(2) override mi: bool = true;
@id(8) override si: bool = true;
@id(1) override li: bool = true;
@id(0) override ki: bool = true;

@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var ha: sampler;
@group(1) @binding(11)
var CC: texture_2d<f32>;
@group(1) @binding(13)
var r5_: sampler;
@group(0) @binding(6)
var<storage, read_write> V0_: Te_1;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> r1_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> a1_1: vec4<f32>;
var<private> m1_1: f32;
var<private> F4_1: vec2<f32>;
var<private> q3_1: vec2<u32>;
var<private> R0_1: vec4<f32>;
var<private> l1_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> m0_: m0ge;
var<private> K1_: vec4<f32>;
@group(3) @binding(9)
var wa: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> F0_1: f32;

fn main_1() {
    var phi_1443_: f32;
    var phi_1444_: f32;
    var phi_1460_: vec4<f32>;
    var phi_1459_: vec4<f32>;
    var phi_951_: bool;
    var phi_966_: bool;
    var phi_1445_: f32;
    var phi_1455_: vec4<f32>;
    var phi_1462_: vec4<f32>;
    var phi_1463_: vec4<f32>;
    var phi_1465_: f32;
    var phi_729_: bool;
    var phi_1466_: f32;
    var phi_1182_: bool;
    var phi_1184_: bool;
    var phi_1487_: f32;
    var phi_1482_: u32;
    var phi_1479_: f32;
    var phi_1486_: f32;
    var phi_1481_: u32;
    var phi_1478_: f32;
    var phi_1483_: f32;
    var phi_1480_: u32;
    var phi_1477_: f32;
    var phi_1488_: f32;
    var phi_1489_: f32;
    var phi_1490_: f32;
    var phi_1500_: f32;
    var phi_1522_: vec3<f32>;

    let _e57 = gl_FragCoord_1;
    let _e61 = bitcast<vec2<u32>>(vec2<i32>(floor(_e57.xy)));
    let _e63 = j.A6_;
    let _e92 = bitcast<i32>((((((_e61.y >> bitcast<u32>(5u)) * (((_e63 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e61.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e61.x & 28u) << bitcast<u32>(5u)) + ((_e61.y & 28u) << bitcast<u32>(2i)))) + (((_e61.y & 3u) << bitcast<u32>(2i)) + (_e61.x & 3u))));
    let _e93 = Q0_1;
    let _e95 = r1_1;
    let _e96 = a1_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e99 = (mi && (u32(_e93) != 0u));
            if (_e96.w >= 0f) {
                phi_1459_ = _e96;
            } else {
                let _e102 = -(_e96.w);
                let _e107 = j.wc;
                let _e110 = j.xc;
                if (_e96.z > 0f) {
                    phi_1443_ = _e96.x;
                } else {
                    phi_1443_ = length(_e96.xy);
                }
                let _e118 = phi_1443_;
                let _e119 = clamp(_e118, 0f, 1f);
                let _e120 = abs(_e96.z);
                if (_e120 > 1f) {
                    phi_1444_ = ((0.9980469f * _e119) + 0.0009765625f);
                } else {
                    phi_1444_ = ((0.001953125f * _e119) + _e120);
                }
                let _e127 = phi_1444_;
                let _e129 = textureSampleLevel(ED, ha, vec2<f32>(_e127, ((floor(_e102) * _e107) + _e110)), 0f);
                phi_1460_ = _e129;
                if !(_e99) {
                    let _e133 = (_e129.xyz * _e129.w);
                    phi_1460_ = vec4<f32>(_e133.x, _e133.y, _e133.z, (_e129.w * (fract(_e102) * 1.0039216f)));
                }
                let _e140 = phi_1460_;
                phi_1459_ = _e140;
            }
            let _e142 = phi_1459_;
            phi_951_ = si;
            if si {
                phi_951_ = (_e95.z < 0f);
            }
            let _e146 = phi_951_;
            if _e146 {
                let _e148 = textureSampleLevel(CC, r5_, _e95.xy, 0f);
                phi_1463_ = _e148;
                break;
            }
            phi_966_ = si;
            if si {
                phi_966_ = (_e95.z > 0f);
            }
            let _e152 = phi_966_;
            phi_1462_ = _e142;
            if _e152 {
                let _e156 = textureSampleLevel(CC, r5_, _e95.xy, (_e95.z - 1f));
                phi_1455_ = _e156;
                if _e99 {
                    if (_e156.w != 0f) {
                        phi_1445_ = (1f / _e156.w);
                    } else {
                        phi_1445_ = 0f;
                    }
                    let _e162 = phi_1445_;
                    let _e163 = (_e156.xyz * _e162);
                    phi_1455_ = vec4<f32>(_e163.x, _e163.y, _e163.z, _e156.w);
                }
                let _e169 = phi_1455_;
                phi_1462_ = (_e142 * _e169);
            }
            let _e172 = phi_1462_;
            phi_1463_ = _e172;
            break;
        }
    }
    let _e174 = phi_1463_;
    let _e175 = m1_1;
    let _e176 = F4_1;
    let _e179 = q3_1[1u];
    let _e181 = q3_1[0u];
    let _e182 = vec2<u32>(floor(_e176));
    phi_1465_ = 1f;
    if li {
        let _e210 = R0_1;
        let _e213 = min(_e210.xy, _e210.zw);
        phi_1465_ = min(min(_e213.x, _e213.y), 1f);
    }
    let _e219 = phi_1465_;
    phi_729_ = ki;
    if ki {
        let _e221 = l1_1[0u];
        phi_729_ = (_e221 != 0f);
    }
    let _e224 = phi_729_;
    phi_1466_ = _e219;
    if _e224 {
        let _e227 = m0_.k2_[_e92];
        phi_1466_ = min(unpack4x8unorm(_e227).x, _e219);
    }
    let _e232 = phi_1466_;
    let _e234 = clamp(_e175, 0f, max(_e232, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e240 = u32(((abs(_e234) * 1024f) + 0.5f));
            let _e243 = atomicLoad((&V0_.k2_[(_e181 + (((((_e182.y >> bitcast<u32>(5u)) * (_e179 << bitcast<u32>(5u))) + ((_e182.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e182.x & 28u) << bitcast<u32>(5u)) + ((_e182.y & 28u) << bitcast<u32>(2i)))) + (((_e182.y & 3u) << bitcast<u32>(2i)) + (_e182.x & 3u))))]));
            let _e245 = (min(_e174.w, _e234) >= 1f);
            phi_1184_ = _e245;
            if _e245 {
                let _e247 = j.j2_;
                let _e248 = (_e243 < _e247);
                phi_1182_ = _e248;
                if !(_e248) {
                    phi_1182_ = (_e243 >= (_e247 | 262144u));
                }
                let _e253 = phi_1182_;
                phi_1184_ = _e253;
            }
            let _e255 = phi_1184_;
            if _e255 {
                phi_1489_ = 1f;
                break;
            }
            let _e257 = j.j2_;
            phi_1483_ = 0f;
            phi_1480_ = _e240;
            phi_1477_ = _e234;
            if (_e243 < _e257) {
                let _e260 = (_e257 | (262144u + _e240));
                let _e261 = atomicMax((&V0_.k2_[(_e181 + (((((_e182.y >> bitcast<u32>(5u)) * (_e179 << bitcast<u32>(5u))) + ((_e182.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e182.x & 28u) << bitcast<u32>(5u)) + ((_e182.y & 28u) << bitcast<u32>(2i)))) + (((_e182.y & 3u) << bitcast<u32>(2i)) + (_e182.x & 3u))))]), _e260);
                if (_e261 <= _e257) {
                    phi_1486_ = min(_e234, 1f);
                    phi_1481_ = _e240;
                    phi_1478_ = 0f;
                } else {
                    phi_1487_ = 0f;
                    phi_1482_ = _e240;
                    phi_1479_ = _e234;
                    if (_e261 < _e260) {
                        let _e266 = ((_e261 & 524287u) - 262144u);
                        let _e268 = (f32(_e266) * 0.0009765625f);
                        phi_1487_ = ((min(_e234, 1f) - _e268) / max((1f - (_e268 * _e174.w)), 0.000062f));
                        phi_1482_ = _e266;
                        phi_1479_ = _e268;
                    }
                    let _e276 = phi_1487_;
                    let _e278 = phi_1482_;
                    let _e280 = phi_1479_;
                    phi_1486_ = _e276;
                    phi_1481_ = _e278;
                    phi_1478_ = _e280;
                }
                let _e282 = phi_1486_;
                let _e284 = phi_1481_;
                let _e286 = phi_1478_;
                phi_1483_ = _e282;
                phi_1480_ = _e284;
                phi_1477_ = _e286;
            }
            let _e288 = phi_1483_;
            let _e290 = phi_1480_;
            let _e292 = phi_1477_;
            phi_1488_ = _e288;
            if (_e292 > 0f) {
                let _e294 = atomicAdd((&V0_.k2_[(_e181 + (((((_e182.y >> bitcast<u32>(5u)) * (_e179 << bitcast<u32>(5u))) + ((_e182.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e182.x & 28u) << bitcast<u32>(5u)) + ((_e182.y & 28u) << bitcast<u32>(2i)))) + (((_e182.y & 3u) << bitcast<u32>(2i)) + (_e182.x & 3u))))]), _e290);
                let _e299 = (f32(bitcast<i32>(((_e294 & 524287u) - 262144u))) * 0.0009765625f);
                let _e301 = clamp(_e299, 0f, 1f);
                phi_1488_ = (_e288 + ((1f - (_e288 * _e174.w)) * ((clamp((_e299 + _e292), 0f, 1f) - _e301) / max((1f - (_e301 * _e174.w)), 0.000062f))));
            }
            let _e313 = phi_1488_;
            phi_1489_ = _e313;
            break;
        }
    }
    let _e315 = phi_1489_;
    phi_1500_ = f32();
    if ri {
        let _e317 = j.L3_;
        let _e319 = j.M3_;
        if ri {
            phi_1490_ = ((fract((52.982918f * fract(((0.06711056f * _e57.x) + (0.00583715f * _e57.y))))) * _e317) + _e319);
        } else {
            phi_1490_ = 0f;
        }
        let _e331 = phi_1490_;
        phi_1500_ = _e331;
    }
    let _e333 = phi_1500_;
    let _e334 = (_e174 * _e315);
    let _e335 = _e334.xyz;
    if (ri && (_e334.w != 0f)) {
        phi_1522_ = (vec3(_e333) + _e335);
    } else {
        phi_1522_ = _e335;
    }
    let _e342 = phi_1522_;
    let _e348 = vec4<f32>(_e342.x, _e334.y, _e334.z, _e334.w);
    let _e354 = vec4<f32>(_e348.x, _e342.y, _e348.z, _e348.w);
    m0_.k2_[_e92] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    K1_ = vec4<f32>(_e354.x, _e354.y, _e342.z, _e354.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(9) r1_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) a1_: vec4<f32>, @location(1) @interpolate(flat, either) m1_: f32, @location(8) F4_: vec2<f32>, @location(7) @interpolate(flat, either) q3_: vec2<u32>, @location(5) R0_: vec4<f32>, @location(4) @interpolate(flat, either) l1_: vec2<f32>, @location(3) @interpolate(flat, either) F0_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    r1_1 = r1_;
    Q0_1 = Q0_;
    a1_1 = a1_;
    m1_1 = m1_;
    F4_1 = F4_;
    q3_1 = q3_;
    R0_1 = R0_;
    l1_1 = l1_;
    F0_1 = F0_;
    main_1();
    let _e21 = K1_;
    return _e21;
}
