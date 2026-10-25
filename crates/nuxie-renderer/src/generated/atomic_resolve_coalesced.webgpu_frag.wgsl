struct ug {
    v2_: array<vec2<u32>>,
}

struct m0cf {
    v2_: array<u32>,
}

struct vg {
    v2_: array<vec4<f32>>,
}

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

struct n0cf {
    v2_: array<u32>,
}

struct W4cf {
    v2_: array<u32>,
}

@id(7) override qj: bool = true;
@id(9) override sj: bool = true;
@id(5) override oj: bool = true;
@id(0) override jj: bool = true;
@id(1) override kj: bool = true;
@id(2) override lj: bool = true;

@group(0) @binding(3)
var<storage> XC: ug;
@group(2) @binding(1)
var<storage, read_write> m0_: m0cf;
@group(0) @binding(4)
var<storage> JB: vg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var S8_: sampler;
@group(2) @binding(0)
var<storage, read_write> n0_: n0cf;
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf;
var<private> Q1_: vec4<f32>;
@group(3) @binding(9)
var eb: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(1) @binding(11)
var RB: texture_2d<f32>;
@group(1) @binding(13)
var v4_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1394_: bool;
    var phi_3427_: f32;
    var phi_3426_: f32;
    var phi_3428_: f32;
    var phi_3431_: f32;
    var phi_3430_: f32;
    var phi_1431_: bool;
    var phi_3449_: f32;
    var phi_3432_: f32;
    var phi_3433_: f32;
    var phi_3434_: vec2<f32>;
    var phi_3446_: vec4<f32>;
    var phi_3444_: vec4<f32>;
    var phi_3454_: f32;
    var phi_3877_: i32;
    var phi_3941_: vec4<f32>;
    var phi_4098_: vec4<f32>;
    var phi_4111_: vec4<f32>;
    var phi_4112_: vec4<f32>;

    let _e75 = gl_FragCoord_1;
    let _e76 = _e75.xy;
    let _e79 = bitcast<vec2<u32>>(vec2<i32>(floor(_e76)));
    let _e81 = j.T6_;
    let _e110 = bitcast<i32>((((((_e79.y >> bitcast<u32>(5u)) * (((_e81 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e79.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e79.x & 28u) << bitcast<u32>(5u)) + ((_e79.y & 28u) << bitcast<u32>(2i)))) + (((_e79.y & 3u) << bitcast<u32>(2i)) + (_e79.x & 3u))));
    let _e113 = W4_.v2_[_e110];
    let _e117 = ((f32((_e113 & 131071u)) * 0.00048828125f) + -32f);
    let _e119 = (_e113 >> bitcast<u32>(17u));
    let _e122 = XC.v2_[_e119];
    phi_3426_ = _e117;
    if ((_e122.x & 768u) != 0u) {
        let _e126 = abs(_e117);
        phi_1394_ = oj;
        if oj {
            phi_1394_ = ((_e122.x & 512u) != 0u);
        }
        let _e130 = phi_1394_;
        phi_3427_ = _e126;
        if _e130 {
            phi_3427_ = (1f - abs(((fract((_e126 * 0.5f)) * 2f) + -1f)));
        }
        let _e138 = phi_3427_;
        phi_3426_ = _e138;
    }
    let _e140 = phi_3426_;
    let _e141 = clamp(_e140, 0f, 1f);
    phi_3430_ = _e141;
    if jj {
        let _e143 = (_e122.x >> bitcast<u32>(16u));
        phi_3431_ = _e141;
        if (_e143 != 0u) {
            let _e147 = m0_.v2_[_e110];
            if (_e143 == (_e147 >> bitcast<u32>(16i))) {
                phi_3428_ = min(_e141, unpack2x16float(_e147).x);
            } else {
                phi_3428_ = 0f;
            }
            let _e155 = phi_3428_;
            phi_3431_ = _e155;
        }
        let _e157 = phi_3431_;
        phi_3430_ = _e157;
    }
    let _e159 = phi_3430_;
    phi_1431_ = kj;
    if kj {
        phi_1431_ = ((_e122.x & 1024u) != 0u);
    }
    let _e163 = phi_1431_;
    phi_3449_ = _e159;
    if _e163 {
        let _e164 = (_e119 * 8u);
        let _e168 = JB.v2_[(_e164 + 2u)];
        let _e179 = JB.v2_[(_e164 + 3u)];
        let _e184 = _e179.zw;
        let _e186 = ((abs(((mat2x2<f32>(vec2<f32>(_e168.x, _e168.y), vec2<f32>(_e168.z, _e168.w)) * _e76) + _e179.xy)) * _e184) - _e184);
        phi_3449_ = min(_e159, clamp((min(_e186.x, _e186.y) + 0.5f), 0f, 1f));
    }
    let _e194 = phi_3449_;
    let _e195 = (_e122.x & 15u);
    let _e198 = ((_e122.x >> bitcast<u32>(4i)) & 15u);
    let _e200 = (lj && (_e198 != 0u));
    if (_e195 <= 1u) {
        phi_3444_ = select(unpack4x8unorm(_e122.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((jj && (_e195 == 0u))));
    } else {
        let _e202 = (_e119 * 8u);
        let _e205 = JB.v2_[_e202];
        let _e216 = JB.v2_[(_e202 + 1u)];
        let _e219 = ((mat2x2<f32>(vec2<f32>(_e205.x, _e205.y), vec2<f32>(_e205.z, _e205.w)) * _e76) + _e216.xy);
        let _e223 = f32(_e195);
        let _e225 = j.Q8_;
        let _e227 = j.R8_;
        let _e232 = ((_e216.w * _e225) + _e227);
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e223 == 4f)) {
                    let _e246 = normalize(_e219);
                    let _e249 = acos(clamp(_e246.x, -1f, 1f));
                    if (_e246.y >= 0f) {
                        phi_3433_ = _e249;
                    } else {
                        phi_3433_ = -(_e249);
                    }
                    let _e254 = phi_3433_;
                    phi_3434_ = vec2<f32>((_e254 * 0.15915494f), _e232);
                    break;
                } else {
                    if (_e223 == 2f) {
                        phi_3432_ = _e219.x;
                    } else {
                        phi_3432_ = length(_e219);
                    }
                    let _e240 = phi_3432_;
                    phi_3434_ = vec2<f32>(((clamp(_e240, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e216.z < 0f))) + ((max(0f, _e216.z) * 0.001953125f) + 0.0009765625f)), _e232);
                    break;
                }
            }
        }
        let _e258 = phi_3434_;
        let _e259 = textureSampleLevel(ZC, S8_, _e258, 0f);
        phi_3446_ = _e259;
        if !(_e200) {
            let _e263 = (_e259.xyz * _e259.w);
            phi_3446_ = vec4<f32>(_e263.x, _e263.y, _e263.z, (_e259.w * abs(bitcast<f32>(_e122.y))));
        }
        let _e273 = phi_3446_;
        phi_3444_ = _e273;
    }
    let _e281 = phi_3444_;
    phi_4111_ = _e281;
    if _e200 {
        phi_4098_ = _e281;
        if ((_e281.w * _e194) != 0f) {
            let _e287 = n0_.v2_[_e110];
            let _e288 = unpack4x8unorm(_e287);
            let _e289 = _e281.xyz;
            local_2 = _e289;
            let _e290 = _e288.xyz;
            if (_e288.w != 0f) {
                phi_3454_ = (1f / _e288.w);
            } else {
                phi_3454_ = 0f;
            }
            let _e295 = phi_3454_;
            let _e296 = (_e290 * _e295);
            local = _e296;
            switch bitcast<i32>(_e198) {
                case 11: {
                    let _e580 = local_2;
                    local_1 = (_e580 * _e296);
                    break;
                }
                case 1: {
                    let _e576 = local_2;
                    local_1 = ((_e576 + _e296) - (_e576 * _e296));
                    break;
                }
                case 2: {
                    let _e568 = local_2;
                    let _e569 = (_e568 * _e296);
                    local_1 = (select(_e569, (((_e568 + _e296) - _e569) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e296 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e566 = local_2;
                    local_1 = min(_e566, _e296);
                    break;
                }
                case 4: {
                    let _e564 = local_2;
                    local_1 = max(_e564, _e296);
                    break;
                }
                case 5: {
                    let _e535 = clamp(_e290, vec3<f32>(0f, 0f, 0f), _e288.www);
                    let _e541 = vec4<f32>(_e535.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e547 = vec4<f32>(_e541.x, _e535.y, _e541.z, _e541.w);
                    let _e554 = local_2;
                    let _e557 = (clamp((vec3<f32>(1f, 1f, 1f) - _e554), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e288.w);
                    let _e558 = vec4<f32>(_e547.x, _e547.y, _e535.z, _e547.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e558 / _e557)), sign(_e558), (_e557 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e496 = local_2;
                    local_2 = clamp(_e496, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e499 = clamp(_e290, vec3<f32>(0f, 0f, 0f), _e288.www);
                    let _e505 = vec4<f32>(_e499.x, _e288.y, _e288.z, _e288.w);
                    let _e511 = vec4<f32>(_e505.x, _e499.y, _e505.z, _e505.w);
                    phi_3941_ = vec4<f32>(_e511.x, _e511.y, _e499.z, _e511.w);
                    if (_e288.w == 0f) {
                        phi_3941_ = vec4<f32>(_e499.x, _e499.y, _e499.z, 1f);
                    }
                    let _e521 = phi_3941_;
                    let _e525 = (vec3(_e521.w) - _e521.xyz);
                    let _e526 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e525 / (_e526 * _e521.w))), sign(_e525), (_e526 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e488 = local_2;
                    let _e489 = (_e488 * _e296);
                    local_1 = (select(_e489, (((_e488 + _e296) - _e489) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e488 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_3877_ = 0i;
                    loop {
                        let _e457 = phi_3877_;
                        if (_e457 < 3i) {
                            let _e460 = local_2[_e457];
                            if (_e460 <= 0.5f) {
                                let _e477 = local[_e457];
                                local_1[_e457] = (1f - _e477);
                            } else {
                                let _e463 = local[_e457];
                                if (_e463 <= 0.25f) {
                                    let _e469 = local[_e457];
                                    let _e472 = local[_e457];
                                    local_1[_e457] = ((((16f * _e469) - 12f) * _e472) + 3f);
                                } else {
                                    let _e465 = local[_e457];
                                    local_1[_e457] = (inverseSqrt(_e465) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_3877_ = (_e457 + 1i);
                        }
                    }
                    let _e481 = local_2;
                    let _e485 = local_1;
                    local_1 = (_e296 + ((_e296 * ((_e481 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e485));
                    break;
                }
                case 9: {
                    let _e453 = local_2;
                    local_1 = abs((_e296 - _e453));
                    break;
                }
                case 10: {
                    let _e448 = local_2;
                    local_1 = ((_e448 + _e296) - ((_e448 * 2f) * _e296));
                    break;
                }
                case 12: {
                    if qj {
                        let _e399 = local_2;
                        let _e400 = clamp(_e399, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e400;
                        let _e415 = (_e400 - vec3(min(min(_e400.x, _e400.y), _e400.z)));
                        let _e423 = (_e415 * ((max(max(_e296.x, _e296.y), _e296.z) - min(min(_e296.x, _e296.y), _e296.z)) / max(0.000062f, max(max(_e415.x, _e415.y), _e415.z))));
                        let _e424 = dot(_e296, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e427 = (_e423 - vec3(dot(_e423, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e440 = (vec2<f32>(_e424, (1f - _e424)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e427.x, _e427.y), _e427.z)), max(max(_e427.x, _e427.y), _e427.z))));
                        local_1 = ((_e427 * min(1f, min(_e440.x, _e440.y))) + vec3(_e424));
                    }
                    break;
                }
                case 13: {
                    if qj {
                        let _e350 = local_2;
                        let _e351 = clamp(_e350, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e351;
                        let _e366 = (_e296 - vec3(min(min(_e296.x, _e296.y), _e296.z)));
                        let _e374 = (_e366 * ((max(max(_e351.x, _e351.y), _e351.z) - min(min(_e351.x, _e351.y), _e351.z)) / max(0.000062f, max(max(_e366.x, _e366.y), _e366.z))));
                        let _e375 = dot(_e296, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e378 = (_e374 - vec3(dot(_e374, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e391 = (vec2<f32>(_e375, (1f - _e375)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e378.x, _e378.y), _e378.z)), max(max(_e378.x, _e378.y), _e378.z))));
                        local_1 = ((_e378 * min(1f, min(_e391.x, _e391.y))) + vec3(_e375));
                    }
                    break;
                }
                case 14: {
                    if qj {
                        let _e324 = local_2;
                        let _e325 = clamp(_e324, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e325;
                        let _e326 = dot(_e296, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e329 = (_e325 - vec3(dot(_e325, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e342 = (vec2<f32>(_e326, (1f - _e326)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e329.x, _e329.y), _e329.z)), max(max(_e329.x, _e329.y), _e329.z))));
                        local_1 = ((_e329 * min(1f, min(_e342.x, _e342.y))) + vec3(_e326));
                    }
                    break;
                }
                case 15: {
                    if qj {
                        let _e298 = local_2;
                        let _e299 = clamp(_e298, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e299;
                        let _e300 = dot(_e299, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e303 = (_e296 - vec3(dot(_e296, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e316 = (vec2<f32>(_e300, (1f - _e300)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e303.x, _e303.y), _e303.z)), max(max(_e303.x, _e303.y), _e303.z))));
                        local_1 = ((_e303 * min(1f, min(_e316.x, _e316.y))) + vec3(_e300));
                    }
                    break;
                }
                default: {
                }
            }
            let _e582 = local_1;
            let _e584 = mix(_e289, _e582, vec3(_e288.w));
            let _e590 = vec4<f32>(_e584.x, _e281.y, _e281.z, _e281.w);
            let _e596 = vec4<f32>(_e590.x, _e584.y, _e590.z, _e590.w);
            phi_4098_ = vec4<f32>(_e596.x, _e596.y, _e584.z, _e596.w);
        }
        let _e604 = phi_4098_;
        let _e607 = (_e604.xyz * _e604.w);
        let _e613 = vec4<f32>(_e607.x, _e604.y, _e604.z, _e604.w);
        let _e619 = vec4<f32>(_e613.x, _e607.y, _e613.z, _e613.w);
        phi_4111_ = vec4<f32>(_e619.x, _e619.y, _e607.z, _e619.w);
    }
    let _e627 = phi_4111_;
    let _e628 = (_e627 * _e194);
    let _e630 = (1f - _e628.w);
    phi_4112_ = _e628;
    if (_e630 != 0f) {
        let _e634 = n0_.v2_[_e110];
        phi_4112_ = (_e628 + (unpack4x8unorm(_e634) * _e630));
    }
    let _e639 = phi_4112_;
    Q1_ = _e639;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = Q1_;
    return _e3;
}
