struct Bf {
    j2_: array<vec2<u32>>,
}

struct m0ge {
    j2_: array<u32>,
}

struct Cf {
    j2_: array<vec4<f32>>,
}

struct UB {
    Qc: f32,
    Td: f32,
    Yf: f32,
    Zf: f32,
    z6_: u32,
    X9_: u32,
    Kf: u32,
    Lf: u32,
    i8_: vec4<i32>,
    Ch: vec2<f32>,
    Ud: vec2<f32>,
    i2_: u32,
    Gh: f32,
    T4_: u32,
    c3_: f32,
    Vd: f32,
    Ef: u32,
    M3_: f32,
    N3_: f32,
    Wd: f32,
    zh: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

struct o0ge {
    j2_: array<u32>,
}

struct K4ge {
    j2_: array<u32>,
}

@id(7) override ii: bool = true;
@id(6) override hi: bool = true;
@id(4) override fi: bool = true;
@id(0) override bi: bool = true;
@id(1) override ci: bool = true;
@id(2) override di: bool = true;

@group(0) @binding(3)
var<storage> XC: Bf;
@group(2) @binding(1)
var<storage, read_write> m0_: m0ge;
@group(0) @binding(4)
var<storage> JB: Cf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var FD: texture_2d<f32>;
@group(3) @binding(8)
var ha: sampler;
@group(2) @binding(0)
var<storage, read_write> o0_: o0ge;
@group(2) @binding(3)
var<storage, read_write> K4_: K4ge;
var<private> F0_1: u32;
@group(0) @binding(10)
var GD: texture_2d<f32>;
@group(3) @binding(10)
var ma: sampler;
var<private> K2_1: vec2<f32>;
@group(3) @binding(9)
var wa: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var f6_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1494_: bool;
    var phi_3571_: f32;
    var phi_3570_: f32;
    var phi_3572_: f32;
    var phi_3575_: f32;
    var phi_3574_: f32;
    var phi_1531_: bool;
    var phi_3577_: f32;
    var phi_4179_: u32;
    var phi_3576_: f32;
    var phi_3597_: vec4<f32>;
    var phi_4178_: u32;
    var phi_3595_: vec4<f32>;
    var phi_3602_: f32;
    var phi_4024_: vec4<f32>;
    var phi_3968_: i32;
    var phi_4163_: vec4<f32>;
    var phi_4176_: vec4<f32>;
    var phi_4201_: vec3<f32>;
    var phi_4203_: vec4<f32>;

    let _e81 = gl_FragCoord_1;
    let _e82 = _e81.xy;
    let _e85 = bitcast<vec2<u32>>(vec2<i32>(floor(_e82)));
    let _e87 = j.z6_;
    let _e116 = bitcast<i32>((((((_e85.y >> bitcast<u32>(5u)) * (((_e87 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e85.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e85.x & 28u) << bitcast<u32>(5u)) + ((_e85.y & 28u) << bitcast<u32>(2i)))) + (((_e85.y & 3u) << bitcast<u32>(2i)) + (_e85.x & 3u))));
    let _e119 = K4_.j2_[_e116];
    let _e121 = (_e119 >> bitcast<u32>(17u));
    let _e122 = F0_1;
    let _e126 = K2_1;
    let _e127 = textureSampleLevel(GD, ma, _e126, 0f);
    K4_.j2_[_e116] = (((_e122 << bitcast<u32>(17u)) + 65536u) + bitcast<u32>(i32(round((clamp(_e127.x, 0f, 1f) * 2048f)))));
    let _e138 = ((f32((_e119 & 131071u)) * 0.00048828125f) + -32f);
    let _e141 = XC.j2_[_e121];
    phi_3570_ = _e138;
    if ((_e141.x & 768u) != 0u) {
        let _e145 = abs(_e138);
        phi_1494_ = fi;
        if fi {
            phi_1494_ = ((_e141.x & 512u) != 0u);
        }
        let _e149 = phi_1494_;
        phi_3571_ = _e145;
        if _e149 {
            phi_3571_ = (1f - abs(((fract((_e145 * 0.5f)) * 2f) + -1f)));
        }
        let _e157 = phi_3571_;
        phi_3570_ = _e157;
    }
    let _e159 = phi_3570_;
    let _e160 = clamp(_e159, 0f, 1f);
    phi_3574_ = _e160;
    if bi {
        let _e162 = (_e141.x >> bitcast<u32>(16u));
        phi_3575_ = _e160;
        if (_e162 != 0u) {
            let _e166 = m0_.j2_[_e116];
            if (_e162 == (_e166 >> bitcast<u32>(16i))) {
                phi_3572_ = min(_e160, unpack2x16float(_e166).x);
            } else {
                phi_3572_ = 0f;
            }
            let _e174 = phi_3572_;
            phi_3575_ = _e174;
        }
        let _e176 = phi_3575_;
        phi_3574_ = _e176;
    }
    let _e178 = phi_3574_;
    phi_1531_ = ci;
    if ci {
        phi_1531_ = ((_e141.x & 1024u) != 0u);
    }
    let _e182 = phi_1531_;
    phi_3577_ = _e178;
    if _e182 {
        let _e183 = (_e121 * 8u);
        let _e187 = JB.j2_[(_e183 + 2u)];
        let _e198 = JB.j2_[(_e183 + 3u)];
        let _e203 = _e198.zw;
        let _e205 = ((abs(((mat2x2<f32>(vec2<f32>(_e187.x, _e187.y), vec2<f32>(_e187.z, _e187.w)) * _e82) + _e198.xy)) * _e203) - _e203);
        phi_3577_ = min(_e178, clamp((min(_e205.x, _e205.y) + 0.5f), 0f, 1f));
    }
    let _e213 = phi_3577_;
    let _e214 = (_e141.x & 15u);
    let _e217 = ((_e141.x >> bitcast<u32>(4i)) & 15u);
    let _e219 = (di && (_e217 != 0u));
    if (_e214 <= 1u) {
        let _e224 = (bi && (_e214 == 0u));
        phi_4179_ = 0u;
        if _e224 {
            phi_4179_ = (_e141.y | pack2x16float(vec2<f32>(_e213, 0f)));
        }
        let _e229 = phi_4179_;
        phi_4178_ = _e229;
        phi_3595_ = select(unpack4x8unorm(_e141.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e224));
    } else {
        let _e232 = (_e121 * 8u);
        let _e235 = JB.j2_[_e232];
        let _e246 = JB.j2_[(_e232 + 1u)];
        let _e249 = ((mat2x2<f32>(vec2<f32>(_e235.x, _e235.y), vec2<f32>(_e235.z, _e235.w)) * _e82) + _e246.xy);
        if (_e214 == 2u) {
            phi_3576_ = _e249.x;
        } else {
            phi_3576_ = length(_e249);
        }
        let _e254 = phi_3576_;
        let _e261 = bitcast<f32>(_e141.y);
        let _e264 = j.wc;
        let _e267 = j.xc;
        let _e270 = textureSampleLevel(FD, ha, vec2<f32>(((clamp(_e254, 0f, 1f) * _e246.z) + _e246.w), ((floor(_e261) * _e264) + _e267)), 0f);
        phi_3597_ = _e270;
        if !(_e219) {
            let _e274 = (_e270.xyz * _e270.w);
            phi_3597_ = vec4<f32>(_e274.x, _e274.y, _e274.z, (_e270.w * (fract(_e261) * 1.0039216f)));
        }
        let _e283 = phi_3597_;
        phi_4178_ = 0u;
        phi_3595_ = _e283;
    }
    let _e285 = phi_4178_;
    let _e287 = phi_3595_;
    phi_4176_ = _e287;
    if _e219 {
        phi_4163_ = _e287;
        if ((_e287.w * _e213) != 0f) {
            let _e293 = o0_.j2_[_e116];
            let _e294 = unpack4x8unorm(_e293);
            let _e295 = _e287.xyz;
            local_2 = _e295;
            let _e296 = _e294.xyz;
            if (_e294.w != 0f) {
                phi_3602_ = (1f / _e294.w);
            } else {
                phi_3602_ = 0f;
            }
            let _e301 = phi_3602_;
            let _e302 = (_e296 * _e301);
            local = _e302;
            switch bitcast<i32>(_e217) {
                case 11: {
                    let _e304 = local_2;
                    local_1 = (_e304 * _e302);
                    break;
                }
                case 1: {
                    let _e306 = local_2;
                    local_1 = ((_e306 + _e302) - (_e306 * _e302));
                    break;
                }
                case 2: {
                    let _e310 = local_2;
                    let _e311 = (_e310 * _e302);
                    local_1 = (select(_e311, (((_e310 + _e302) - _e311) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e302 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e318 = local_2;
                    local_1 = min(_e318, _e302);
                    break;
                }
                case 4: {
                    let _e320 = local_2;
                    local_1 = max(_e320, _e302);
                    break;
                }
                case 5: {
                    let _e323 = clamp(_e296, vec3<f32>(0f, 0f, 0f), _e294.www);
                    let _e329 = vec4<f32>(_e323.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e335 = vec4<f32>(_e329.x, _e323.y, _e329.z, _e329.w);
                    let _e342 = local_2;
                    let _e345 = (clamp((vec3<f32>(1f, 1f, 1f) - _e342), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e294.w);
                    let _e346 = vec4<f32>(_e335.x, _e335.y, _e323.z, _e335.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e346 / _e345)), sign(_e346), (_e345 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e352 = local_2;
                    local_2 = clamp(_e352, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e355 = clamp(_e296, vec3<f32>(0f, 0f, 0f), _e294.www);
                    let _e361 = vec4<f32>(_e355.x, _e294.y, _e294.z, _e294.w);
                    let _e367 = vec4<f32>(_e361.x, _e355.y, _e361.z, _e361.w);
                    phi_4024_ = vec4<f32>(_e367.x, _e367.y, _e355.z, _e367.w);
                    if (_e294.w == 0f) {
                        phi_4024_ = vec4<f32>(_e355.x, _e355.y, _e355.z, 1f);
                    }
                    let _e377 = phi_4024_;
                    let _e381 = (vec3(_e377.w) - _e377.xyz);
                    let _e382 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e381 / (_e382 * _e377.w))), sign(_e381), (_e382 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e390 = local_2;
                    let _e391 = (_e390 * _e302);
                    local_1 = (select(_e391, (((_e390 + _e302) - _e391) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e390 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_3968_ = 0i;
                    loop {
                        let _e399 = phi_3968_;
                        if (_e399 < 3i) {
                            let _e402 = local_2[_e399];
                            if (_e402 <= 0.5f) {
                                let _e405 = local[_e399];
                                local_1[_e399] = (1f - _e405);
                            } else {
                                let _e409 = local[_e399];
                                if (_e409 <= 0.25f) {
                                    let _e411 = local[_e399];
                                    let _e414 = local[_e399];
                                    local_1[_e399] = ((((16f * _e411) - 12f) * _e414) + 3f);
                                } else {
                                    let _e418 = local[_e399];
                                    local_1[_e399] = (inverseSqrt(_e418) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_3968_ = (_e399 + 1i);
                        }
                    }
                    let _e423 = local_2;
                    let _e427 = local_1;
                    local_1 = (_e302 + ((_e302 * ((_e423 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e427));
                    break;
                }
                case 9: {
                    let _e430 = local_2;
                    local_1 = abs((_e302 - _e430));
                    break;
                }
                case 10: {
                    let _e433 = local_2;
                    local_1 = ((_e433 + _e302) - ((_e433 * 2f) * _e302));
                    break;
                }
                case 12: {
                    if hi {
                        let _e438 = local_2;
                        let _e439 = clamp(_e438, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e439;
                        let _e454 = (_e439 - vec3(min(min(_e439.x, _e439.y), _e439.z)));
                        let _e462 = (_e454 * ((max(max(_e302.x, _e302.y), _e302.z) - min(min(_e302.x, _e302.y), _e302.z)) / max(0.000062f, max(max(_e454.x, _e454.y), _e454.z))));
                        let _e463 = dot(_e302, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e466 = (_e462 - vec3(dot(_e462, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e479 = (vec2<f32>(_e463, (1f - _e463)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e466.x, _e466.y), _e466.z)), max(max(_e466.x, _e466.y), _e466.z))));
                        local_1 = ((_e466 * min(1f, min(_e479.x, _e479.y))) + vec3(_e463));
                    }
                    break;
                }
                case 13: {
                    if hi {
                        let _e487 = local_2;
                        let _e488 = clamp(_e487, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e488;
                        let _e503 = (_e302 - vec3(min(min(_e302.x, _e302.y), _e302.z)));
                        let _e511 = (_e503 * ((max(max(_e488.x, _e488.y), _e488.z) - min(min(_e488.x, _e488.y), _e488.z)) / max(0.000062f, max(max(_e503.x, _e503.y), _e503.z))));
                        let _e512 = dot(_e302, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e515 = (_e511 - vec3(dot(_e511, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e528 = (vec2<f32>(_e512, (1f - _e512)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e515.x, _e515.y), _e515.z)), max(max(_e515.x, _e515.y), _e515.z))));
                        local_1 = ((_e515 * min(1f, min(_e528.x, _e528.y))) + vec3(_e512));
                    }
                    break;
                }
                case 14: {
                    if hi {
                        let _e536 = local_2;
                        let _e537 = clamp(_e536, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e537;
                        let _e538 = dot(_e302, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e541 = (_e537 - vec3(dot(_e537, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e554 = (vec2<f32>(_e538, (1f - _e538)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e541.x, _e541.y), _e541.z)), max(max(_e541.x, _e541.y), _e541.z))));
                        local_1 = ((_e541 * min(1f, min(_e554.x, _e554.y))) + vec3(_e538));
                    }
                    break;
                }
                case 15: {
                    if hi {
                        let _e562 = local_2;
                        let _e563 = clamp(_e562, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e563;
                        let _e564 = dot(_e563, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e567 = (_e302 - vec3(dot(_e302, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e580 = (vec2<f32>(_e564, (1f - _e564)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e567.x, _e567.y), _e567.z)), max(max(_e567.x, _e567.y), _e567.z))));
                        local_1 = ((_e567 * min(1f, min(_e580.x, _e580.y))) + vec3(_e564));
                    }
                    break;
                }
                default: {
                }
            }
            let _e588 = local_1;
            let _e590 = mix(_e295, _e588, vec3(_e294.w));
            let _e596 = vec4<f32>(_e590.x, _e287.y, _e287.z, _e287.w);
            let _e602 = vec4<f32>(_e596.x, _e590.y, _e596.z, _e596.w);
            phi_4163_ = vec4<f32>(_e602.x, _e602.y, _e590.z, _e602.w);
        }
        let _e610 = phi_4163_;
        let _e613 = (_e610.xyz * _e610.w);
        let _e619 = vec4<f32>(_e613.x, _e610.y, _e610.z, _e610.w);
        let _e625 = vec4<f32>(_e619.x, _e613.y, _e619.z, _e619.w);
        phi_4176_ = vec4<f32>(_e625.x, _e625.y, _e613.z, _e625.w);
    }
    let _e633 = phi_4176_;
    let _e634 = (_e633 * _e213);
    let _e635 = _e634.xyz;
    let _e638 = j.M3_;
    let _e640 = j.N3_;
    if (ii && (_e634.w != 0f)) {
        phi_4201_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e81.x) + (0.00583715f * _e81.y))))) * _e638) + _e640)) + _e635);
    } else {
        phi_4201_ = _e635;
    }
    let _e656 = phi_4201_;
    let _e662 = vec4<f32>(_e656.x, _e634.y, _e634.z, _e634.w);
    let _e668 = vec4<f32>(_e662.x, _e656.y, _e662.z, _e662.w);
    let _e674 = vec4<f32>(_e668.x, _e668.y, _e656.z, _e668.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e656.x + _e656.y) + _e656.z) + _e634.w) == 0f) {
                break;
            }
            let _e680 = (1f - _e634.w);
            phi_4203_ = _e674;
            if (_e680 != 0f) {
                let _e684 = o0_.j2_[_e116];
                phi_4203_ = (_e674 + (unpack4x8unorm(_e684) * _e680));
            }
            let _e689 = phi_4203_;
            o0_.j2_[_e116] = pack4x8unorm(_e689);
            break;
        }
    }
    if (_e285 != 0u) {
        m0_.j2_[_e116] = _e285;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) F0_: u32, @location(0) K2_: vec2<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    F0_1 = F0_;
    K2_1 = K2_;
    main_1();
}
