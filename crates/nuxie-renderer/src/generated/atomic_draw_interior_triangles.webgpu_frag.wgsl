struct lg {
    r2_: array<vec2<u32>>,
}

struct m0Re {
    r2_: array<u32>,
}

struct mg {
    r2_: array<vec4<f32>>,
}

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
    Bi: vec2<f32>,
    Fe: vec2<f32>,
    q2_: u32,
    Fi: f32,
    w6_: u32,
    i3_: f32,
    Ge: f32,
    og: u32,
    F3_: f32,
    G3_: f32,
    He: f32,
    yi: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

struct n0Re {
    r2_: array<u32>,
}

struct R4Re {
    r2_: array<u32>,
}

@id(7) override gj: bool = true;
@id(6) override fj: bool = true;
@id(4) override dj: bool = true;
@id(0) override Zi: bool = true;
@id(1) override aj: bool = true;
@id(2) override bj: bool = true;

@group(0) @binding(3)
var<storage> XC: lg;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Re;
@group(0) @binding(4)
var<storage> JB: mg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var Q8_: sampler;
@group(2) @binding(0)
var<storage, read_write> n0_: n0Re;
@group(2) @binding(3)
var<storage, read_write> R4_: R4Re;
var<private> G0_1: u32;
var<private> o1_1: f32;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_3786_: u32;
    var phi_1615_: bool;
    var phi_3791_: f32;
    var phi_3790_: f32;
    var phi_3792_: f32;
    var phi_3795_: f32;
    var phi_3794_: f32;
    var phi_1652_: bool;
    var phi_3799_: f32;
    var phi_3796_: f32;
    var phi_3797_: f32;
    var phi_3798_: vec2<f32>;
    var phi_3821_: vec4<f32>;
    var phi_4528_: u32;
    var phi_4527_: u32;
    var phi_3819_: vec4<f32>;
    var phi_3828_: f32;
    var phi_4278_: i32;
    var phi_4346_: vec4<f32>;
    var phi_4512_: vec4<f32>;
    var phi_4525_: vec4<f32>;
    var phi_4558_: u32;
    var phi_4552_: vec4<f32>;
    var phi_4553_: vec3<f32>;
    var phi_4555_: vec4<f32>;

    let _e83 = gl_FragCoord_1;
    let _e84 = _e83.xy;
    let _e87 = bitcast<vec2<u32>>(vec2<i32>(floor(_e84)));
    let _e89 = j.S6_;
    let _e118 = bitcast<i32>((((((_e87.y >> bitcast<u32>(5u)) * (((_e89 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e87.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e87.x & 28u) << bitcast<u32>(5u)) + ((_e87.y & 28u) << bitcast<u32>(2i)))) + (((_e87.y & 3u) << bitcast<u32>(2i)) + (_e87.x & 3u))));
    let _e121 = R4_.r2_[_e118];
    let _e123 = (_e121 >> bitcast<u32>(17u));
    let _e124 = G0_1;
    if (_e123 == _e124) {
        phi_3786_ = _e121;
    } else {
        phi_3786_ = ((_e124 << bitcast<u32>(17u)) + 65536u);
    }
    let _e130 = phi_3786_;
    let _e131 = o1_1;
    R4_.r2_[_e118] = (_e130 + bitcast<u32>(i32(round((_e131 * 2048f)))));
    phi_4558_ = 0u;
    phi_4552_ = vec4<f32>(0f, 0f, 0f, 0f);
    if (_e123 != _e124) {
        let _e141 = ((f32((_e121 & 131071u)) * 0.00048828125f) + -32f);
        let _e144 = XC.r2_[_e123];
        phi_3790_ = _e141;
        if ((_e144.x & 768u) != 0u) {
            let _e148 = abs(_e141);
            phi_1615_ = dj;
            if dj {
                phi_1615_ = ((_e144.x & 512u) != 0u);
            }
            let _e152 = phi_1615_;
            phi_3791_ = _e148;
            if _e152 {
                phi_3791_ = (1f - abs(((fract((_e148 * 0.5f)) * 2f) + -1f)));
            }
            let _e160 = phi_3791_;
            phi_3790_ = _e160;
        }
        let _e162 = phi_3790_;
        let _e163 = clamp(_e162, 0f, 1f);
        phi_3794_ = _e163;
        if Zi {
            let _e165 = (_e144.x >> bitcast<u32>(16u));
            phi_3795_ = _e163;
            if (_e165 != 0u) {
                let _e169 = m0_.r2_[_e118];
                if (_e165 == (_e169 >> bitcast<u32>(16i))) {
                    phi_3792_ = min(_e163, unpack2x16float(_e169).x);
                } else {
                    phi_3792_ = 0f;
                }
                let _e177 = phi_3792_;
                phi_3795_ = _e177;
            }
            let _e179 = phi_3795_;
            phi_3794_ = _e179;
        }
        let _e181 = phi_3794_;
        phi_1652_ = aj;
        if aj {
            phi_1652_ = ((_e144.x & 1024u) != 0u);
        }
        let _e185 = phi_1652_;
        phi_3799_ = _e181;
        if _e185 {
            let _e186 = (_e123 * 8u);
            let _e190 = JB.r2_[(_e186 + 2u)];
            let _e201 = JB.r2_[(_e186 + 3u)];
            let _e206 = _e201.zw;
            let _e208 = ((abs(((mat2x2<f32>(vec2<f32>(_e190.x, _e190.y), vec2<f32>(_e190.z, _e190.w)) * _e84) + _e201.xy)) * _e206) - _e206);
            phi_3799_ = min(_e181, clamp((min(_e208.x, _e208.y) + 0.5f), 0f, 1f));
        }
        let _e216 = phi_3799_;
        let _e217 = (_e144.x & 15u);
        let _e220 = ((_e144.x >> bitcast<u32>(4i)) & 15u);
        let _e222 = (bj && (_e220 != 0u));
        if (_e217 <= 1u) {
            let _e302 = (Zi && (_e217 == 0u));
            phi_4528_ = 0u;
            if _e302 {
                phi_4528_ = (_e144.y | pack2x16float(vec2<f32>(_e216, 0f)));
            }
            let _e307 = phi_4528_;
            phi_4527_ = _e307;
            phi_3819_ = select(unpack4x8unorm(_e144.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e302));
        } else {
            let _e224 = (_e123 * 8u);
            let _e227 = JB.r2_[_e224];
            let _e238 = JB.r2_[(_e224 + 1u)];
            let _e241 = ((mat2x2<f32>(vec2<f32>(_e227.x, _e227.y), vec2<f32>(_e227.z, _e227.w)) * _e84) + _e238.xy);
            let _e245 = f32(_e217);
            let _e247 = j.O8_;
            let _e249 = j.P8_;
            let _e254 = ((_e238.w * _e247) + _e249);
            switch bitcast<i32>(0u) {
                default: {
                    if (_e245 == 4f) {
                        let _e271 = normalize(vec2<f32>(_e241.x, -(_e241.y)));
                        let _e274 = acos(clamp(_e271.x, -1f, 1f));
                        if (_e271.y >= 0f) {
                            phi_3797_ = _e274;
                        } else {
                            phi_3797_ = -(_e274);
                        }
                        let _e279 = phi_3797_;
                        phi_3798_ = vec2<f32>((_e279 * 0.15915507f), _e254);
                        break;
                    } else {
                        if (_e245 == 2f) {
                            phi_3796_ = _e241.x;
                        } else {
                            phi_3796_ = length(_e241);
                        }
                        let _e261 = phi_3796_;
                        phi_3798_ = vec2<f32>(((clamp(_e261, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e238.z < 0f))) + ((max(0f, _e238.z) * 0.001953125f) + 0.0009765625f)), _e254);
                        break;
                    }
                }
            }
            let _e283 = phi_3798_;
            let _e284 = textureSampleLevel(ZC, Q8_, _e283, 0f);
            phi_3821_ = _e284;
            if !(_e222) {
                let _e288 = (_e284.xyz * _e284.w);
                phi_3821_ = vec4<f32>(_e288.x, _e288.y, _e288.z, (_e284.w * abs(bitcast<f32>(_e144.y))));
            }
            let _e298 = phi_3821_;
            phi_4527_ = 0u;
            phi_3819_ = _e298;
        }
        let _e311 = phi_4527_;
        let _e313 = phi_3819_;
        phi_4525_ = _e313;
        if _e222 {
            phi_4512_ = _e313;
            if ((_e313.w * _e216) != 0f) {
                let _e319 = n0_.r2_[_e118];
                let _e320 = unpack4x8unorm(_e319);
                let _e321 = _e313.xyz;
                local_2 = _e321;
                let _e322 = _e320.xyz;
                if (_e320.w != 0f) {
                    phi_3828_ = (1f / _e320.w);
                } else {
                    phi_3828_ = 0f;
                }
                let _e327 = phi_3828_;
                let _e328 = (_e322 * _e327);
                local = _e328;
                switch bitcast<i32>(_e220) {
                    case 11: {
                        let _e612 = local_2;
                        local_1 = (_e612 * _e328);
                        break;
                    }
                    case 1: {
                        let _e608 = local_2;
                        local_1 = ((_e608 + _e328) - (_e608 * _e328));
                        break;
                    }
                    case 2: {
                        let _e600 = local_2;
                        let _e601 = (_e600 * _e328);
                        local_1 = (select(_e601, (((_e600 + _e328) - _e601) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e328 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 3: {
                        let _e598 = local_2;
                        local_1 = min(_e598, _e328);
                        break;
                    }
                    case 4: {
                        let _e596 = local_2;
                        local_1 = max(_e596, _e328);
                        break;
                    }
                    case 5: {
                        let _e567 = clamp(_e322, vec3<f32>(0f, 0f, 0f), _e320.www);
                        let _e573 = vec4<f32>(_e567.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                        let _e579 = vec4<f32>(_e573.x, _e567.y, _e573.z, _e573.w);
                        let _e586 = local_2;
                        let _e589 = (clamp((vec3<f32>(1f, 1f, 1f) - _e586), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e320.w);
                        let _e590 = vec4<f32>(_e579.x, _e579.y, _e567.z, _e579.w).xyz;
                        local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e590 / _e589)), sign(_e590), (_e589 == vec3<f32>(0f, 0f, 0f)));
                        break;
                    }
                    case 6: {
                        let _e528 = local_2;
                        local_2 = clamp(_e528, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        let _e531 = clamp(_e322, vec3<f32>(0f, 0f, 0f), _e320.www);
                        let _e537 = vec4<f32>(_e531.x, _e320.y, _e320.z, _e320.w);
                        let _e543 = vec4<f32>(_e537.x, _e531.y, _e537.z, _e537.w);
                        phi_4346_ = vec4<f32>(_e543.x, _e543.y, _e531.z, _e543.w);
                        if (_e320.w == 0f) {
                            phi_4346_ = vec4<f32>(_e531.x, _e531.y, _e531.z, 1f);
                        }
                        let _e553 = phi_4346_;
                        let _e557 = (vec3(_e553.w) - _e553.xyz);
                        let _e558 = local_2;
                        local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e557 / (_e558 * _e553.w))), sign(_e557), (_e558 == vec3<f32>(0f, 0f, 0f))));
                        break;
                    }
                    case 7: {
                        let _e520 = local_2;
                        let _e521 = (_e520 * _e328);
                        local_1 = (select(_e521, (((_e520 + _e328) - _e521) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e520 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 8: {
                        phi_4278_ = 0i;
                        loop {
                            let _e489 = phi_4278_;
                            if (_e489 < 3i) {
                                let _e492 = local_2[_e489];
                                if (_e492 <= 0.5f) {
                                    let _e509 = local[_e489];
                                    local_1[_e489] = (1f - _e509);
                                } else {
                                    let _e495 = local[_e489];
                                    if (_e495 <= 0.25f) {
                                        let _e501 = local[_e489];
                                        let _e504 = local[_e489];
                                        local_1[_e489] = ((((16f * _e501) - 12f) * _e504) + 3f);
                                    } else {
                                        let _e497 = local[_e489];
                                        local_1[_e489] = (inverseSqrt(_e497) - 1f);
                                    }
                                }
                                continue;
                            } else {
                                break;
                            }
                            continuing {
                                phi_4278_ = (_e489 + 1i);
                            }
                        }
                        let _e513 = local_2;
                        let _e517 = local_1;
                        local_1 = (_e328 + ((_e328 * ((_e513 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e517));
                        break;
                    }
                    case 9: {
                        let _e485 = local_2;
                        local_1 = abs((_e328 - _e485));
                        break;
                    }
                    case 10: {
                        let _e480 = local_2;
                        local_1 = ((_e480 + _e328) - ((_e480 * 2f) * _e328));
                        break;
                    }
                    case 12: {
                        if fj {
                            let _e431 = local_2;
                            let _e432 = clamp(_e431, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e432;
                            let _e447 = (_e432 - vec3(min(min(_e432.x, _e432.y), _e432.z)));
                            let _e455 = (_e447 * ((max(max(_e328.x, _e328.y), _e328.z) - min(min(_e328.x, _e328.y), _e328.z)) / max(0.000062f, max(max(_e447.x, _e447.y), _e447.z))));
                            let _e456 = dot(_e328, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e459 = (_e455 - vec3(dot(_e455, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e472 = (vec2<f32>(_e456, (1f - _e456)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e459.x, _e459.y), _e459.z)), max(max(_e459.x, _e459.y), _e459.z))));
                            local_1 = ((_e459 * min(1f, min(_e472.x, _e472.y))) + vec3(_e456));
                        }
                        break;
                    }
                    case 13: {
                        if fj {
                            let _e382 = local_2;
                            let _e383 = clamp(_e382, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e383;
                            let _e398 = (_e328 - vec3(min(min(_e328.x, _e328.y), _e328.z)));
                            let _e406 = (_e398 * ((max(max(_e383.x, _e383.y), _e383.z) - min(min(_e383.x, _e383.y), _e383.z)) / max(0.000062f, max(max(_e398.x, _e398.y), _e398.z))));
                            let _e407 = dot(_e328, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e410 = (_e406 - vec3(dot(_e406, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e423 = (vec2<f32>(_e407, (1f - _e407)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e410.x, _e410.y), _e410.z)), max(max(_e410.x, _e410.y), _e410.z))));
                            local_1 = ((_e410 * min(1f, min(_e423.x, _e423.y))) + vec3(_e407));
                        }
                        break;
                    }
                    case 14: {
                        if fj {
                            let _e356 = local_2;
                            let _e357 = clamp(_e356, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e357;
                            let _e358 = dot(_e328, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e361 = (_e357 - vec3(dot(_e357, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e374 = (vec2<f32>(_e358, (1f - _e358)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e361.x, _e361.y), _e361.z)), max(max(_e361.x, _e361.y), _e361.z))));
                            local_1 = ((_e361 * min(1f, min(_e374.x, _e374.y))) + vec3(_e358));
                        }
                        break;
                    }
                    case 15: {
                        if fj {
                            let _e330 = local_2;
                            let _e331 = clamp(_e330, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e331;
                            let _e332 = dot(_e331, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e335 = (_e328 - vec3(dot(_e328, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e348 = (vec2<f32>(_e332, (1f - _e332)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e335.x, _e335.y), _e335.z)), max(max(_e335.x, _e335.y), _e335.z))));
                            local_1 = ((_e335 * min(1f, min(_e348.x, _e348.y))) + vec3(_e332));
                        }
                        break;
                    }
                    default: {
                    }
                }
                let _e614 = local_1;
                let _e616 = mix(_e321, _e614, vec3(_e320.w));
                let _e622 = vec4<f32>(_e616.x, _e313.y, _e313.z, _e313.w);
                let _e628 = vec4<f32>(_e622.x, _e616.y, _e622.z, _e622.w);
                phi_4512_ = vec4<f32>(_e628.x, _e628.y, _e616.z, _e628.w);
            }
            let _e636 = phi_4512_;
            let _e639 = (_e636.xyz * _e636.w);
            let _e645 = vec4<f32>(_e639.x, _e636.y, _e636.z, _e636.w);
            let _e651 = vec4<f32>(_e645.x, _e639.y, _e645.z, _e645.w);
            phi_4525_ = vec4<f32>(_e651.x, _e651.y, _e639.z, _e651.w);
        }
        let _e659 = phi_4525_;
        phi_4558_ = _e311;
        phi_4552_ = (_e659 * _e216);
    }
    let _e662 = phi_4558_;
    let _e664 = phi_4552_;
    let _e665 = _e664.xyz;
    let _e668 = j.F3_;
    let _e670 = j.G3_;
    if (gj && (_e664.w != 0f)) {
        phi_4553_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e83.x) + (0.00583715f * _e83.y))))) * _e668) + _e670)) + _e665);
    } else {
        phi_4553_ = _e665;
    }
    let _e686 = phi_4553_;
    let _e692 = vec4<f32>(_e686.x, _e664.y, _e664.z, _e664.w);
    let _e698 = vec4<f32>(_e692.x, _e686.y, _e692.z, _e692.w);
    let _e704 = vec4<f32>(_e698.x, _e698.y, _e686.z, _e698.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e686.x + _e686.y) + _e686.z) + _e664.w) == 0f) {
                break;
            }
            phi_4555_ = _e704;
            if (_e664.w != 1f) {
                let _e713 = n0_.r2_[_e118];
                phi_4555_ = (_e704 + (unpack4x8unorm(_e713) * (1f - _e664.w)));
            }
            let _e719 = phi_4555_;
            n0_.r2_[_e118] = pack4x8unorm(_e719);
            break;
        }
    }
    if (_e662 != 0u) {
        m0_.r2_[_e118] = _e662;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) G0_: u32, @location(0) @interpolate(flat, either) o1_: f32) {
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    o1_1 = o1_;
    main_1();
}
