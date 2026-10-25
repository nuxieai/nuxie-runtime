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

@id(8) override rj: bool = true;
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
var<private> I0_1: u32;
var<private> p1_1: f32;
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
    var phi_3777_: u32;
    var phi_1612_: bool;
    var phi_3782_: f32;
    var phi_3781_: f32;
    var phi_3783_: f32;
    var phi_3786_: f32;
    var phi_3785_: f32;
    var phi_1649_: bool;
    var phi_3790_: f32;
    var phi_3787_: f32;
    var phi_3788_: f32;
    var phi_3789_: vec2<f32>;
    var phi_3812_: vec4<f32>;
    var phi_4519_: u32;
    var phi_4518_: u32;
    var phi_3810_: vec4<f32>;
    var phi_3819_: f32;
    var phi_4269_: i32;
    var phi_4337_: vec4<f32>;
    var phi_4503_: vec4<f32>;
    var phi_4516_: vec4<f32>;
    var phi_4549_: u32;
    var phi_4543_: vec4<f32>;
    var phi_4544_: vec3<f32>;
    var phi_4546_: vec4<f32>;

    let _e84 = gl_FragCoord_1;
    let _e85 = _e84.xy;
    let _e88 = bitcast<vec2<u32>>(vec2<i32>(floor(_e85)));
    let _e90 = j.T6_;
    let _e119 = bitcast<i32>((((((_e88.y >> bitcast<u32>(5u)) * (((_e90 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e88.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e88.x & 28u) << bitcast<u32>(5u)) + ((_e88.y & 28u) << bitcast<u32>(2i)))) + (((_e88.y & 3u) << bitcast<u32>(2i)) + (_e88.x & 3u))));
    let _e122 = W4_.v2_[_e119];
    let _e124 = (_e122 >> bitcast<u32>(17u));
    let _e125 = I0_1;
    if (_e124 == _e125) {
        phi_3777_ = _e122;
    } else {
        phi_3777_ = ((_e125 << bitcast<u32>(17u)) + 65536u);
    }
    let _e131 = phi_3777_;
    let _e132 = p1_1;
    W4_.v2_[_e119] = (_e131 + bitcast<u32>(i32(round((_e132 * 2048f)))));
    phi_4549_ = 0u;
    phi_4543_ = vec4<f32>(0f, 0f, 0f, 0f);
    if (_e124 != _e125) {
        let _e142 = ((f32((_e122 & 131071u)) * 0.00048828125f) + -32f);
        let _e145 = XC.v2_[_e124];
        phi_3781_ = _e142;
        if ((_e145.x & 768u) != 0u) {
            let _e149 = abs(_e142);
            phi_1612_ = oj;
            if oj {
                phi_1612_ = ((_e145.x & 512u) != 0u);
            }
            let _e153 = phi_1612_;
            phi_3782_ = _e149;
            if _e153 {
                phi_3782_ = (1f - abs(((fract((_e149 * 0.5f)) * 2f) + -1f)));
            }
            let _e161 = phi_3782_;
            phi_3781_ = _e161;
        }
        let _e163 = phi_3781_;
        let _e164 = clamp(_e163, 0f, 1f);
        phi_3785_ = _e164;
        if jj {
            let _e166 = (_e145.x >> bitcast<u32>(16u));
            phi_3786_ = _e164;
            if (_e166 != 0u) {
                let _e170 = m0_.v2_[_e119];
                if (_e166 == (_e170 >> bitcast<u32>(16i))) {
                    phi_3783_ = min(_e164, unpack2x16float(_e170).x);
                } else {
                    phi_3783_ = 0f;
                }
                let _e178 = phi_3783_;
                phi_3786_ = _e178;
            }
            let _e180 = phi_3786_;
            phi_3785_ = _e180;
        }
        let _e182 = phi_3785_;
        phi_1649_ = kj;
        if kj {
            phi_1649_ = ((_e145.x & 1024u) != 0u);
        }
        let _e186 = phi_1649_;
        phi_3790_ = _e182;
        if _e186 {
            let _e187 = (_e124 * 8u);
            let _e191 = JB.v2_[(_e187 + 2u)];
            let _e202 = JB.v2_[(_e187 + 3u)];
            let _e207 = _e202.zw;
            let _e209 = ((abs(((mat2x2<f32>(vec2<f32>(_e191.x, _e191.y), vec2<f32>(_e191.z, _e191.w)) * _e85) + _e202.xy)) * _e207) - _e207);
            phi_3790_ = min(_e182, clamp((min(_e209.x, _e209.y) + 0.5f), 0f, 1f));
        }
        let _e217 = phi_3790_;
        let _e218 = (_e145.x & 15u);
        let _e221 = ((_e145.x >> bitcast<u32>(4i)) & 15u);
        let _e223 = (lj && (_e221 != 0u));
        if (_e218 <= 1u) {
            let _e300 = (jj && (_e218 == 0u));
            phi_4519_ = 0u;
            if _e300 {
                phi_4519_ = (_e145.y | pack2x16float(vec2<f32>(_e217, 0f)));
            }
            let _e305 = phi_4519_;
            phi_4518_ = _e305;
            phi_3810_ = select(unpack4x8unorm(_e145.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e300));
        } else {
            let _e225 = (_e124 * 8u);
            let _e228 = JB.v2_[_e225];
            let _e239 = JB.v2_[(_e225 + 1u)];
            let _e242 = ((mat2x2<f32>(vec2<f32>(_e228.x, _e228.y), vec2<f32>(_e228.z, _e228.w)) * _e85) + _e239.xy);
            let _e246 = f32(_e218);
            let _e248 = j.Q8_;
            let _e250 = j.R8_;
            let _e255 = ((_e239.w * _e248) + _e250);
            switch bitcast<i32>(0u) {
                default: {
                    if (sj && (_e246 == 4f)) {
                        let _e269 = normalize(_e242);
                        let _e272 = acos(clamp(_e269.x, -1f, 1f));
                        if (_e269.y >= 0f) {
                            phi_3788_ = _e272;
                        } else {
                            phi_3788_ = -(_e272);
                        }
                        let _e277 = phi_3788_;
                        phi_3789_ = vec2<f32>((_e277 * 0.15915494f), _e255);
                        break;
                    } else {
                        if (_e246 == 2f) {
                            phi_3787_ = _e242.x;
                        } else {
                            phi_3787_ = length(_e242);
                        }
                        let _e263 = phi_3787_;
                        phi_3789_ = vec2<f32>(((clamp(_e263, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e239.z < 0f))) + ((max(0f, _e239.z) * 0.001953125f) + 0.0009765625f)), _e255);
                        break;
                    }
                }
            }
            let _e281 = phi_3789_;
            let _e282 = textureSampleLevel(ZC, S8_, _e281, 0f);
            phi_3812_ = _e282;
            if !(_e223) {
                let _e286 = (_e282.xyz * _e282.w);
                phi_3812_ = vec4<f32>(_e286.x, _e286.y, _e286.z, (_e282.w * abs(bitcast<f32>(_e145.y))));
            }
            let _e296 = phi_3812_;
            phi_4518_ = 0u;
            phi_3810_ = _e296;
        }
        let _e309 = phi_4518_;
        let _e311 = phi_3810_;
        phi_4516_ = _e311;
        if _e223 {
            phi_4503_ = _e311;
            if ((_e311.w * _e217) != 0f) {
                let _e317 = n0_.v2_[_e119];
                let _e318 = unpack4x8unorm(_e317);
                let _e319 = _e311.xyz;
                local_2 = _e319;
                let _e320 = _e318.xyz;
                if (_e318.w != 0f) {
                    phi_3819_ = (1f / _e318.w);
                } else {
                    phi_3819_ = 0f;
                }
                let _e325 = phi_3819_;
                let _e326 = (_e320 * _e325);
                local = _e326;
                switch bitcast<i32>(_e221) {
                    case 11: {
                        let _e610 = local_2;
                        local_1 = (_e610 * _e326);
                        break;
                    }
                    case 1: {
                        let _e606 = local_2;
                        local_1 = ((_e606 + _e326) - (_e606 * _e326));
                        break;
                    }
                    case 2: {
                        let _e598 = local_2;
                        let _e599 = (_e598 * _e326);
                        local_1 = (select(_e599, (((_e598 + _e326) - _e599) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e326 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 3: {
                        let _e596 = local_2;
                        local_1 = min(_e596, _e326);
                        break;
                    }
                    case 4: {
                        let _e594 = local_2;
                        local_1 = max(_e594, _e326);
                        break;
                    }
                    case 5: {
                        let _e565 = clamp(_e320, vec3<f32>(0f, 0f, 0f), _e318.www);
                        let _e571 = vec4<f32>(_e565.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                        let _e577 = vec4<f32>(_e571.x, _e565.y, _e571.z, _e571.w);
                        let _e584 = local_2;
                        let _e587 = (clamp((vec3<f32>(1f, 1f, 1f) - _e584), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e318.w);
                        let _e588 = vec4<f32>(_e577.x, _e577.y, _e565.z, _e577.w).xyz;
                        local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e588 / _e587)), sign(_e588), (_e587 == vec3<f32>(0f, 0f, 0f)));
                        break;
                    }
                    case 6: {
                        let _e526 = local_2;
                        local_2 = clamp(_e526, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        let _e529 = clamp(_e320, vec3<f32>(0f, 0f, 0f), _e318.www);
                        let _e535 = vec4<f32>(_e529.x, _e318.y, _e318.z, _e318.w);
                        let _e541 = vec4<f32>(_e535.x, _e529.y, _e535.z, _e535.w);
                        phi_4337_ = vec4<f32>(_e541.x, _e541.y, _e529.z, _e541.w);
                        if (_e318.w == 0f) {
                            phi_4337_ = vec4<f32>(_e529.x, _e529.y, _e529.z, 1f);
                        }
                        let _e551 = phi_4337_;
                        let _e555 = (vec3(_e551.w) - _e551.xyz);
                        let _e556 = local_2;
                        local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e555 / (_e556 * _e551.w))), sign(_e555), (_e556 == vec3<f32>(0f, 0f, 0f))));
                        break;
                    }
                    case 7: {
                        let _e518 = local_2;
                        let _e519 = (_e518 * _e326);
                        local_1 = (select(_e519, (((_e518 + _e326) - _e519) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e518 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 8: {
                        phi_4269_ = 0i;
                        loop {
                            let _e487 = phi_4269_;
                            if (_e487 < 3i) {
                                let _e490 = local_2[_e487];
                                if (_e490 <= 0.5f) {
                                    let _e507 = local[_e487];
                                    local_1[_e487] = (1f - _e507);
                                } else {
                                    let _e493 = local[_e487];
                                    if (_e493 <= 0.25f) {
                                        let _e499 = local[_e487];
                                        let _e502 = local[_e487];
                                        local_1[_e487] = ((((16f * _e499) - 12f) * _e502) + 3f);
                                    } else {
                                        let _e495 = local[_e487];
                                        local_1[_e487] = (inverseSqrt(_e495) - 1f);
                                    }
                                }
                                continue;
                            } else {
                                break;
                            }
                            continuing {
                                phi_4269_ = (_e487 + 1i);
                            }
                        }
                        let _e511 = local_2;
                        let _e515 = local_1;
                        local_1 = (_e326 + ((_e326 * ((_e511 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e515));
                        break;
                    }
                    case 9: {
                        let _e483 = local_2;
                        local_1 = abs((_e326 - _e483));
                        break;
                    }
                    case 10: {
                        let _e478 = local_2;
                        local_1 = ((_e478 + _e326) - ((_e478 * 2f) * _e326));
                        break;
                    }
                    case 12: {
                        if qj {
                            let _e429 = local_2;
                            let _e430 = clamp(_e429, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e430;
                            let _e445 = (_e430 - vec3(min(min(_e430.x, _e430.y), _e430.z)));
                            let _e453 = (_e445 * ((max(max(_e326.x, _e326.y), _e326.z) - min(min(_e326.x, _e326.y), _e326.z)) / max(0.000062f, max(max(_e445.x, _e445.y), _e445.z))));
                            let _e454 = dot(_e326, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e457 = (_e453 - vec3(dot(_e453, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e470 = (vec2<f32>(_e454, (1f - _e454)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e457.x, _e457.y), _e457.z)), max(max(_e457.x, _e457.y), _e457.z))));
                            local_1 = ((_e457 * min(1f, min(_e470.x, _e470.y))) + vec3(_e454));
                        }
                        break;
                    }
                    case 13: {
                        if qj {
                            let _e380 = local_2;
                            let _e381 = clamp(_e380, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e381;
                            let _e396 = (_e326 - vec3(min(min(_e326.x, _e326.y), _e326.z)));
                            let _e404 = (_e396 * ((max(max(_e381.x, _e381.y), _e381.z) - min(min(_e381.x, _e381.y), _e381.z)) / max(0.000062f, max(max(_e396.x, _e396.y), _e396.z))));
                            let _e405 = dot(_e326, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e408 = (_e404 - vec3(dot(_e404, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e421 = (vec2<f32>(_e405, (1f - _e405)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e408.x, _e408.y), _e408.z)), max(max(_e408.x, _e408.y), _e408.z))));
                            local_1 = ((_e408 * min(1f, min(_e421.x, _e421.y))) + vec3(_e405));
                        }
                        break;
                    }
                    case 14: {
                        if qj {
                            let _e354 = local_2;
                            let _e355 = clamp(_e354, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e355;
                            let _e356 = dot(_e326, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e359 = (_e355 - vec3(dot(_e355, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e372 = (vec2<f32>(_e356, (1f - _e356)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e359.x, _e359.y), _e359.z)), max(max(_e359.x, _e359.y), _e359.z))));
                            local_1 = ((_e359 * min(1f, min(_e372.x, _e372.y))) + vec3(_e356));
                        }
                        break;
                    }
                    case 15: {
                        if qj {
                            let _e328 = local_2;
                            let _e329 = clamp(_e328, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e329;
                            let _e330 = dot(_e329, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e333 = (_e326 - vec3(dot(_e326, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e346 = (vec2<f32>(_e330, (1f - _e330)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e333.x, _e333.y), _e333.z)), max(max(_e333.x, _e333.y), _e333.z))));
                            local_1 = ((_e333 * min(1f, min(_e346.x, _e346.y))) + vec3(_e330));
                        }
                        break;
                    }
                    default: {
                    }
                }
                let _e612 = local_1;
                let _e614 = mix(_e319, _e612, vec3(_e318.w));
                let _e620 = vec4<f32>(_e614.x, _e311.y, _e311.z, _e311.w);
                let _e626 = vec4<f32>(_e620.x, _e614.y, _e620.z, _e620.w);
                phi_4503_ = vec4<f32>(_e626.x, _e626.y, _e614.z, _e626.w);
            }
            let _e634 = phi_4503_;
            let _e637 = (_e634.xyz * _e634.w);
            let _e643 = vec4<f32>(_e637.x, _e634.y, _e634.z, _e634.w);
            let _e649 = vec4<f32>(_e643.x, _e637.y, _e643.z, _e643.w);
            phi_4516_ = vec4<f32>(_e649.x, _e649.y, _e637.z, _e649.w);
        }
        let _e657 = phi_4516_;
        phi_4549_ = _e309;
        phi_4543_ = (_e657 * _e217);
    }
    let _e660 = phi_4549_;
    let _e662 = phi_4543_;
    let _e663 = _e662.xyz;
    let _e666 = j.H3_;
    let _e668 = j.I3_;
    if (rj && (_e662.w != 0f)) {
        phi_4544_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e84.x) + (0.00583715f * _e84.y))))) * _e666) + _e668)) + _e663);
    } else {
        phi_4544_ = _e663;
    }
    let _e684 = phi_4544_;
    let _e690 = vec4<f32>(_e684.x, _e662.y, _e662.z, _e662.w);
    let _e696 = vec4<f32>(_e690.x, _e684.y, _e690.z, _e690.w);
    let _e702 = vec4<f32>(_e696.x, _e696.y, _e684.z, _e696.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e684.x + _e684.y) + _e684.z) + _e662.w) == 0f) {
                break;
            }
            phi_4546_ = _e702;
            if (_e662.w != 1f) {
                let _e711 = n0_.v2_[_e119];
                phi_4546_ = (_e702 + (unpack4x8unorm(_e711) * (1f - _e662.w)));
            }
            let _e717 = phi_4546_;
            n0_.v2_[_e119] = pack4x8unorm(_e717);
            break;
        }
    }
    if (_e660 != 0u) {
        m0_.v2_[_e119] = _e660;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) I0_: u32, @location(0) @interpolate(flat, either) p1_: f32) {
    gl_FragCoord_1 = gl_FragCoord;
    I0_1 = I0_;
    p1_1 = p1_;
    main_1();
}
