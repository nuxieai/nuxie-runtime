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

struct n0Ue {
    v2_: array<u32>,
}

struct T4Ue {
    v2_: array<u32>,
}

@id(8) override nj: bool = true;
@id(7) override mj: bool = true;
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
@group(2) @binding(0)
var<storage, read_write> n0_: n0Ue;
@group(2) @binding(3)
var<storage, read_write> T4_: T4Ue;
var<private> G0_1: u32;
@group(0) @binding(10)
var JD: texture_2d<f32>;
@group(3) @binding(10)
var Sa: sampler;
var<private> U2_1: vec2<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(1) @binding(13)
var W4_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1623_: bool;
    var phi_3798_: f32;
    var phi_3797_: f32;
    var phi_3799_: f32;
    var phi_3802_: f32;
    var phi_3801_: f32;
    var phi_1660_: bool;
    var phi_3806_: f32;
    var phi_3803_: f32;
    var phi_3804_: f32;
    var phi_3805_: vec2<f32>;
    var phi_3826_: vec4<f32>;
    var phi_4492_: u32;
    var phi_4491_: u32;
    var phi_3824_: vec4<f32>;
    var phi_3833_: f32;
    var phi_4255_: i32;
    var phi_4319_: vec4<f32>;
    var phi_4476_: vec4<f32>;
    var phi_4489_: vec4<f32>;
    var phi_4516_: vec3<f32>;
    var phi_4518_: vec4<f32>;

    let _e86 = gl_FragCoord_1;
    let _e87 = _e86.xy;
    let _e90 = bitcast<vec2<u32>>(vec2<i32>(floor(_e87)));
    let _e92 = j.T6_;
    let _e121 = bitcast<i32>((((((_e90.y >> bitcast<u32>(5u)) * (((_e92 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e90.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e90.x & 28u) << bitcast<u32>(5u)) + ((_e90.y & 28u) << bitcast<u32>(2i)))) + (((_e90.y & 3u) << bitcast<u32>(2i)) + (_e90.x & 3u))));
    let _e124 = T4_.v2_[_e121];
    let _e126 = (_e124 >> bitcast<u32>(17u));
    let _e127 = G0_1;
    let _e131 = U2_1;
    let _e132 = textureSampleLevel(JD, Sa, _e131, 0f);
    T4_.v2_[_e121] = (((_e127 << bitcast<u32>(17u)) + 65536u) + bitcast<u32>(i32(round((clamp(_e132.x, 0f, 1f) * 2048f)))));
    let _e143 = ((f32((_e124 & 131071u)) * 0.00048828125f) + -32f);
    let _e146 = XC.v2_[_e126];
    phi_3797_ = _e143;
    if ((_e146.x & 768u) != 0u) {
        let _e150 = abs(_e143);
        phi_1623_ = kj;
        if kj {
            phi_1623_ = ((_e146.x & 512u) != 0u);
        }
        let _e154 = phi_1623_;
        phi_3798_ = _e150;
        if _e154 {
            phi_3798_ = (1f - abs(((fract((_e150 * 0.5f)) * 2f) + -1f)));
        }
        let _e162 = phi_3798_;
        phi_3797_ = _e162;
    }
    let _e164 = phi_3797_;
    let _e165 = clamp(_e164, 0f, 1f);
    phi_3801_ = _e165;
    if fj {
        let _e167 = (_e146.x >> bitcast<u32>(16u));
        phi_3802_ = _e165;
        if (_e167 != 0u) {
            let _e171 = m0_.v2_[_e121];
            if (_e167 == (_e171 >> bitcast<u32>(16i))) {
                phi_3799_ = min(_e165, unpack2x16float(_e171).x);
            } else {
                phi_3799_ = 0f;
            }
            let _e179 = phi_3799_;
            phi_3802_ = _e179;
        }
        let _e181 = phi_3802_;
        phi_3801_ = _e181;
    }
    let _e183 = phi_3801_;
    phi_1660_ = gj;
    if gj {
        phi_1660_ = ((_e146.x & 1024u) != 0u);
    }
    let _e187 = phi_1660_;
    phi_3806_ = _e183;
    if _e187 {
        let _e188 = (_e126 * 8u);
        let _e192 = JB.v2_[(_e188 + 2u)];
        let _e203 = JB.v2_[(_e188 + 3u)];
        let _e208 = _e203.zw;
        let _e210 = ((abs(((mat2x2<f32>(vec2<f32>(_e192.x, _e192.y), vec2<f32>(_e192.z, _e192.w)) * _e87) + _e203.xy)) * _e208) - _e208);
        phi_3806_ = min(_e183, clamp((min(_e210.x, _e210.y) + 0.5f), 0f, 1f));
    }
    let _e218 = phi_3806_;
    let _e219 = (_e146.x & 15u);
    let _e222 = ((_e146.x >> bitcast<u32>(4i)) & 15u);
    let _e224 = (hj && (_e222 != 0u));
    if (_e219 <= 1u) {
        let _e305 = (fj && (_e219 == 0u));
        phi_4492_ = 0u;
        if _e305 {
            phi_4492_ = (_e146.y | pack2x16float(vec2<f32>(_e218, 0f)));
        }
        let _e310 = phi_4492_;
        phi_4491_ = _e310;
        phi_3824_ = select(unpack4x8unorm(_e146.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e305));
    } else {
        let _e226 = (_e126 * 8u);
        let _e229 = JB.v2_[_e226];
        let _e240 = JB.v2_[(_e226 + 1u)];
        let _e243 = ((mat2x2<f32>(vec2<f32>(_e229.x, _e229.y), vec2<f32>(_e229.z, _e229.w)) * _e87) + _e240.xy);
        let _e247 = f32(_e219);
        let _e249 = j.P8_;
        let _e251 = j.Q8_;
        let _e256 = ((_e240.w * _e249) + _e251);
        switch bitcast<i32>(0u) {
            default: {
                if (oj && (_e247 == 4f)) {
                    let _e274 = normalize(vec2<f32>(_e243.x, -(_e243.y)));
                    let _e277 = acos(clamp(_e274.x, -1f, 1f));
                    if (_e274.y >= 0f) {
                        phi_3804_ = _e277;
                    } else {
                        phi_3804_ = -(_e277);
                    }
                    let _e282 = phi_3804_;
                    phi_3805_ = vec2<f32>((_e282 * 0.15915494f), _e256);
                    break;
                } else {
                    if (_e247 == 2f) {
                        phi_3803_ = _e243.x;
                    } else {
                        phi_3803_ = length(_e243);
                    }
                    let _e264 = phi_3803_;
                    phi_3805_ = vec2<f32>(((clamp(_e264, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e240.z < 0f))) + ((max(0f, _e240.z) * 0.001953125f) + 0.0009765625f)), _e256);
                    break;
                }
            }
        }
        let _e286 = phi_3805_;
        let _e287 = textureSampleLevel(ZC, R8_, _e286, 0f);
        phi_3826_ = _e287;
        if !(_e224) {
            let _e291 = (_e287.xyz * _e287.w);
            phi_3826_ = vec4<f32>(_e291.x, _e291.y, _e291.z, (_e287.w * abs(bitcast<f32>(_e146.y))));
        }
        let _e301 = phi_3826_;
        phi_4491_ = 0u;
        phi_3824_ = _e301;
    }
    let _e314 = phi_4491_;
    let _e316 = phi_3824_;
    phi_4489_ = _e316;
    if _e224 {
        phi_4476_ = _e316;
        if ((_e316.w * _e218) != 0f) {
            let _e322 = n0_.v2_[_e121];
            let _e323 = unpack4x8unorm(_e322);
            let _e324 = _e316.xyz;
            local_2 = _e324;
            let _e325 = _e323.xyz;
            if (_e323.w != 0f) {
                phi_3833_ = (1f / _e323.w);
            } else {
                phi_3833_ = 0f;
            }
            let _e330 = phi_3833_;
            let _e331 = (_e325 * _e330);
            local = _e331;
            switch bitcast<i32>(_e222) {
                case 11: {
                    let _e615 = local_2;
                    local_1 = (_e615 * _e331);
                    break;
                }
                case 1: {
                    let _e611 = local_2;
                    local_1 = ((_e611 + _e331) - (_e611 * _e331));
                    break;
                }
                case 2: {
                    let _e603 = local_2;
                    let _e604 = (_e603 * _e331);
                    local_1 = (select(_e604, (((_e603 + _e331) - _e604) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e331 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e601 = local_2;
                    local_1 = min(_e601, _e331);
                    break;
                }
                case 4: {
                    let _e599 = local_2;
                    local_1 = max(_e599, _e331);
                    break;
                }
                case 5: {
                    let _e570 = clamp(_e325, vec3<f32>(0f, 0f, 0f), _e323.www);
                    let _e576 = vec4<f32>(_e570.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e582 = vec4<f32>(_e576.x, _e570.y, _e576.z, _e576.w);
                    let _e589 = local_2;
                    let _e592 = (clamp((vec3<f32>(1f, 1f, 1f) - _e589), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e323.w);
                    let _e593 = vec4<f32>(_e582.x, _e582.y, _e570.z, _e582.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e593 / _e592)), sign(_e593), (_e592 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e531 = local_2;
                    local_2 = clamp(_e531, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e534 = clamp(_e325, vec3<f32>(0f, 0f, 0f), _e323.www);
                    let _e540 = vec4<f32>(_e534.x, _e323.y, _e323.z, _e323.w);
                    let _e546 = vec4<f32>(_e540.x, _e534.y, _e540.z, _e540.w);
                    phi_4319_ = vec4<f32>(_e546.x, _e546.y, _e534.z, _e546.w);
                    if (_e323.w == 0f) {
                        phi_4319_ = vec4<f32>(_e534.x, _e534.y, _e534.z, 1f);
                    }
                    let _e556 = phi_4319_;
                    let _e560 = (vec3(_e556.w) - _e556.xyz);
                    let _e561 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e560 / (_e561 * _e556.w))), sign(_e560), (_e561 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e523 = local_2;
                    let _e524 = (_e523 * _e331);
                    local_1 = (select(_e524, (((_e523 + _e331) - _e524) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e523 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_4255_ = 0i;
                    loop {
                        let _e492 = phi_4255_;
                        if (_e492 < 3i) {
                            let _e495 = local_2[_e492];
                            if (_e495 <= 0.5f) {
                                let _e512 = local[_e492];
                                local_1[_e492] = (1f - _e512);
                            } else {
                                let _e498 = local[_e492];
                                if (_e498 <= 0.25f) {
                                    let _e504 = local[_e492];
                                    let _e507 = local[_e492];
                                    local_1[_e492] = ((((16f * _e504) - 12f) * _e507) + 3f);
                                } else {
                                    let _e500 = local[_e492];
                                    local_1[_e492] = (inverseSqrt(_e500) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_4255_ = (_e492 + 1i);
                        }
                    }
                    let _e516 = local_2;
                    let _e520 = local_1;
                    local_1 = (_e331 + ((_e331 * ((_e516 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e520));
                    break;
                }
                case 9: {
                    let _e488 = local_2;
                    local_1 = abs((_e331 - _e488));
                    break;
                }
                case 10: {
                    let _e483 = local_2;
                    local_1 = ((_e483 + _e331) - ((_e483 * 2f) * _e331));
                    break;
                }
                case 12: {
                    if mj {
                        let _e434 = local_2;
                        let _e435 = clamp(_e434, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e435;
                        let _e450 = (_e435 - vec3(min(min(_e435.x, _e435.y), _e435.z)));
                        let _e458 = (_e450 * ((max(max(_e331.x, _e331.y), _e331.z) - min(min(_e331.x, _e331.y), _e331.z)) / max(0.000062f, max(max(_e450.x, _e450.y), _e450.z))));
                        let _e459 = dot(_e331, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e462 = (_e458 - vec3(dot(_e458, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e475 = (vec2<f32>(_e459, (1f - _e459)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e462.x, _e462.y), _e462.z)), max(max(_e462.x, _e462.y), _e462.z))));
                        local_1 = ((_e462 * min(1f, min(_e475.x, _e475.y))) + vec3(_e459));
                    }
                    break;
                }
                case 13: {
                    if mj {
                        let _e385 = local_2;
                        let _e386 = clamp(_e385, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e386;
                        let _e401 = (_e331 - vec3(min(min(_e331.x, _e331.y), _e331.z)));
                        let _e409 = (_e401 * ((max(max(_e386.x, _e386.y), _e386.z) - min(min(_e386.x, _e386.y), _e386.z)) / max(0.000062f, max(max(_e401.x, _e401.y), _e401.z))));
                        let _e410 = dot(_e331, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e413 = (_e409 - vec3(dot(_e409, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e426 = (vec2<f32>(_e410, (1f - _e410)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e413.x, _e413.y), _e413.z)), max(max(_e413.x, _e413.y), _e413.z))));
                        local_1 = ((_e413 * min(1f, min(_e426.x, _e426.y))) + vec3(_e410));
                    }
                    break;
                }
                case 14: {
                    if mj {
                        let _e359 = local_2;
                        let _e360 = clamp(_e359, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e360;
                        let _e361 = dot(_e331, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e364 = (_e360 - vec3(dot(_e360, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e377 = (vec2<f32>(_e361, (1f - _e361)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e364.x, _e364.y), _e364.z)), max(max(_e364.x, _e364.y), _e364.z))));
                        local_1 = ((_e364 * min(1f, min(_e377.x, _e377.y))) + vec3(_e361));
                    }
                    break;
                }
                case 15: {
                    if mj {
                        let _e333 = local_2;
                        let _e334 = clamp(_e333, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e334;
                        let _e335 = dot(_e334, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e338 = (_e331 - vec3(dot(_e331, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e351 = (vec2<f32>(_e335, (1f - _e335)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e338.x, _e338.y), _e338.z)), max(max(_e338.x, _e338.y), _e338.z))));
                        local_1 = ((_e338 * min(1f, min(_e351.x, _e351.y))) + vec3(_e335));
                    }
                    break;
                }
                default: {
                }
            }
            let _e617 = local_1;
            let _e619 = mix(_e324, _e617, vec3(_e323.w));
            let _e625 = vec4<f32>(_e619.x, _e316.y, _e316.z, _e316.w);
            let _e631 = vec4<f32>(_e625.x, _e619.y, _e625.z, _e625.w);
            phi_4476_ = vec4<f32>(_e631.x, _e631.y, _e619.z, _e631.w);
        }
        let _e639 = phi_4476_;
        let _e642 = (_e639.xyz * _e639.w);
        let _e648 = vec4<f32>(_e642.x, _e639.y, _e639.z, _e639.w);
        let _e654 = vec4<f32>(_e648.x, _e642.y, _e648.z, _e648.w);
        phi_4489_ = vec4<f32>(_e654.x, _e654.y, _e642.z, _e654.w);
    }
    let _e662 = phi_4489_;
    let _e663 = (_e662 * _e218);
    let _e664 = _e663.xyz;
    let _e667 = j.G3_;
    let _e669 = j.H3_;
    if (nj && (_e663.w != 0f)) {
        phi_4516_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e86.x) + (0.00583715f * _e86.y))))) * _e667) + _e669)) + _e664);
    } else {
        phi_4516_ = _e664;
    }
    let _e685 = phi_4516_;
    let _e691 = vec4<f32>(_e685.x, _e663.y, _e663.z, _e663.w);
    let _e697 = vec4<f32>(_e691.x, _e685.y, _e691.z, _e691.w);
    let _e703 = vec4<f32>(_e697.x, _e697.y, _e685.z, _e697.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e685.x + _e685.y) + _e685.z) + _e663.w) == 0f) {
                break;
            }
            phi_4518_ = _e703;
            if (_e663.w != 1f) {
                let _e712 = n0_.v2_[_e121];
                phi_4518_ = (_e703 + (unpack4x8unorm(_e712) * (1f - _e663.w)));
            }
            let _e718 = phi_4518_;
            n0_.v2_[_e121] = pack4x8unorm(_e718);
            break;
        }
    }
    if (_e314 != 0u) {
        m0_.v2_[_e121] = _e314;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) G0_: u32, @location(0) U2_: vec2<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    U2_1 = U2_;
    main_1();
}
