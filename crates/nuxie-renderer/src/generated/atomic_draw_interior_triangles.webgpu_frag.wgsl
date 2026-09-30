struct lf {
    g2_: array<vec2<u32>>,
}

struct i0Td {
    g2_: array<u32>,
}

struct mf {
    g2_: array<vec4<f32>>,
}

struct l0Td {
    g2_: array<u32>,
}

struct AC {
    tc: f32,
    Dd: f32,
    Hf: f32,
    If: f32,
    q6_: u32,
    Qb: u32,
    tf: u32,
    uf: u32,
    X7_: vec4<i32>,
    eh: vec2<f32>,
    Ed: vec2<f32>,
    f2_: u32,
    ih: f32,
    f6_: u32,
    U2_: f32,
    Fd: f32,
    of_: u32,
    F3_: f32,
    G3_: f32,
    Gd: f32,
    bh: u32,
    Pb: u32,
}

struct z4Td {
    g2_: array<u32>,
}

@id(7) override Lh: bool = true;
@id(6) override Kh: bool = true;
@id(4) override Ih: bool = true;
@id(0) override Eh: bool = true;
@id(1) override Fh: bool = true;
@id(2) override Gh: bool = true;

@group(0) @binding(3)
var<storage> CD: lf;
@group(2) @binding(1)
var<storage, read_write> i0_: i0Td;
@group(0) @binding(4)
var<storage> PB: mf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var P9_: sampler;
@group(2) @binding(0)
var<storage, read_write> l0_: l0Td;
@group(0) @binding(0)
var<uniform> j: AC;
@group(2) @binding(3)
var<storage, read_write> z4_: z4Td;
var<private> D0_1: u32;
var<private> j1_1: f32;
@group(3) @binding(9)
var ga: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var Y5_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_3491_: u32;
    var phi_1454_: bool;
    var phi_3496_: f32;
    var phi_3495_: f32;
    var phi_3497_: f32;
    var phi_3500_: f32;
    var phi_3499_: f32;
    var phi_1491_: bool;
    var phi_3502_: f32;
    var phi_4147_: u32;
    var phi_3501_: f32;
    var phi_3524_: vec4<f32>;
    var phi_4146_: u32;
    var phi_3522_: vec4<f32>;
    var phi_3529_: f32;
    var phi_3983_: vec4<f32>;
    var phi_3923_: i32;
    var phi_4131_: vec4<f32>;
    var phi_4144_: vec4<f32>;
    var phi_4175_: u32;
    var phi_4169_: vec4<f32>;
    var phi_4170_: vec3<f32>;
    var phi_4172_: vec4<f32>;

    let _e76 = gl_FragCoord_1;
    let _e77 = _e76.xy;
    let _e80 = bitcast<vec2<u32>>(vec2<i32>(floor(_e77)));
    let _e82 = j.q6_;
    let _e111 = bitcast<i32>((((((_e80.y >> bitcast<u32>(5u)) * (((_e82 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e80.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e80.x & 28u) << bitcast<u32>(5u)) + ((_e80.y & 28u) << bitcast<u32>(2i)))) + (((_e80.y & 3u) << bitcast<u32>(2i)) + (_e80.x & 3u))));
    let _e114 = z4_.g2_[_e111];
    let _e116 = (_e114 >> bitcast<u32>(17u));
    let _e117 = D0_1;
    if (_e116 == _e117) {
        phi_3491_ = _e114;
    } else {
        phi_3491_ = ((_e117 << bitcast<u32>(17u)) + 65536u);
    }
    let _e123 = phi_3491_;
    let _e124 = j1_1;
    z4_.g2_[_e111] = (_e123 + bitcast<u32>(i32(round((_e124 * 2048f)))));
    phi_4175_ = 0u;
    phi_4169_ = vec4<f32>(0f, 0f, 0f, 0f);
    if (_e116 != _e117) {
        let _e134 = ((f32((_e114 & 131071u)) * 0.00048828125f) + -32f);
        let _e137 = CD.g2_[_e116];
        phi_3495_ = _e134;
        if ((_e137.x & 768u) != 0u) {
            let _e141 = abs(_e134);
            phi_1454_ = Ih;
            if Ih {
                phi_1454_ = ((_e137.x & 512u) != 0u);
            }
            let _e145 = phi_1454_;
            phi_3496_ = _e141;
            if _e145 {
                phi_3496_ = (1f - abs(((fract((_e141 * 0.5f)) * 2f) + -1f)));
            }
            let _e153 = phi_3496_;
            phi_3495_ = _e153;
        }
        let _e155 = phi_3495_;
        let _e156 = clamp(_e155, 0f, 1f);
        phi_3499_ = _e156;
        if Eh {
            let _e158 = (_e137.x >> bitcast<u32>(16u));
            phi_3500_ = _e156;
            if (_e158 != 0u) {
                let _e162 = i0_.g2_[_e111];
                if (_e158 == (_e162 >> bitcast<u32>(16i))) {
                    phi_3497_ = min(_e156, unpack2x16float(_e162).x);
                } else {
                    phi_3497_ = 0f;
                }
                let _e170 = phi_3497_;
                phi_3500_ = _e170;
            }
            let _e172 = phi_3500_;
            phi_3499_ = _e172;
        }
        let _e174 = phi_3499_;
        phi_1491_ = Fh;
        if Fh {
            phi_1491_ = ((_e137.x & 1024u) != 0u);
        }
        let _e178 = phi_1491_;
        phi_3502_ = _e174;
        if _e178 {
            let _e179 = (_e116 * 8u);
            let _e183 = PB.g2_[(_e179 + 2u)];
            let _e194 = PB.g2_[(_e179 + 3u)];
            let _e199 = _e194.zw;
            let _e201 = ((abs(((mat2x2<f32>(vec2<f32>(_e183.x, _e183.y), vec2<f32>(_e183.z, _e183.w)) * _e77) + _e194.xy)) * _e199) - _e199);
            phi_3502_ = min(_e174, clamp((min(_e201.x, _e201.y) + 0.5f), 0f, 1f));
        }
        let _e209 = phi_3502_;
        let _e210 = (_e137.x & 15u);
        let _e213 = ((_e137.x >> bitcast<u32>(4i)) & 15u);
        let _e215 = (Gh && (_e213 != 0u));
        if (_e210 <= 1u) {
            let _e220 = (Eh && (_e210 == 0u));
            phi_4147_ = 0u;
            if _e220 {
                phi_4147_ = (_e137.y | pack2x16float(vec2<f32>(_e209, 0f)));
            }
            let _e225 = phi_4147_;
            phi_4146_ = _e225;
            phi_3522_ = select(unpack4x8unorm(_e137.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e220));
        } else {
            let _e228 = (_e116 * 8u);
            let _e231 = PB.g2_[_e228];
            let _e242 = PB.g2_[(_e228 + 1u)];
            let _e245 = ((mat2x2<f32>(vec2<f32>(_e231.x, _e231.y), vec2<f32>(_e231.z, _e231.w)) * _e77) + _e242.xy);
            if (_e210 == 2u) {
                phi_3501_ = _e245.x;
            } else {
                phi_3501_ = length(_e245);
            }
            let _e250 = phi_3501_;
            let _e259 = textureSampleLevel(DD, P9_, vec2<f32>(((clamp(_e250, 0f, 1f) * _e242.z) + _e242.w), bitcast<f32>(_e137.y)), 0f);
            phi_3524_ = _e259;
            if !(_e215) {
                let _e263 = (_e259.xyz * _e259.w);
                let _e269 = vec4<f32>(_e263.x, _e259.y, _e259.z, _e259.w);
                let _e275 = vec4<f32>(_e269.x, _e263.y, _e269.z, _e269.w);
                phi_3524_ = vec4<f32>(_e275.x, _e275.y, _e263.z, _e275.w);
            }
            let _e283 = phi_3524_;
            phi_4146_ = 0u;
            phi_3522_ = _e283;
        }
        let _e285 = phi_4146_;
        let _e287 = phi_3522_;
        phi_4144_ = _e287;
        if _e215 {
            phi_4131_ = _e287;
            if ((_e287.w * _e209) != 0f) {
                let _e293 = l0_.g2_[_e111];
                let _e294 = unpack4x8unorm(_e293);
                let _e295 = _e287.xyz;
                local_2 = _e295;
                let _e296 = _e294.xyz;
                if (_e294.w != 0f) {
                    phi_3529_ = (1f / _e294.w);
                } else {
                    phi_3529_ = 0f;
                }
                let _e301 = phi_3529_;
                let _e302 = (_e296 * _e301);
                local = _e302;
                switch bitcast<i32>(_e213) {
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
                        phi_3983_ = vec4<f32>(_e367.x, _e367.y, _e355.z, _e367.w);
                        if (_e294.w == 0f) {
                            phi_3983_ = vec4<f32>(_e355.x, _e355.y, _e355.z, 1f);
                        }
                        let _e377 = phi_3983_;
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
                        phi_3923_ = 0i;
                        loop {
                            let _e399 = phi_3923_;
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
                                phi_3923_ = (_e399 + 1i);
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
                        if Kh {
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
                        if Kh {
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
                        if Kh {
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
                        if Kh {
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
                phi_4131_ = vec4<f32>(_e602.x, _e602.y, _e590.z, _e602.w);
            }
            let _e610 = phi_4131_;
            let _e613 = (_e610.xyz * _e610.w);
            let _e619 = vec4<f32>(_e613.x, _e610.y, _e610.z, _e610.w);
            let _e625 = vec4<f32>(_e619.x, _e613.y, _e619.z, _e619.w);
            phi_4144_ = vec4<f32>(_e625.x, _e625.y, _e613.z, _e625.w);
        }
        let _e633 = phi_4144_;
        phi_4175_ = _e285;
        phi_4169_ = (_e633 * _e209);
    }
    let _e636 = phi_4175_;
    let _e638 = phi_4169_;
    let _e639 = _e638.xyz;
    let _e642 = j.F3_;
    let _e644 = j.G3_;
    if (Lh && (_e638.w != 0f)) {
        phi_4170_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e76.x) + (0.00583715f * _e76.y))))) * _e642) + _e644)) + _e639);
    } else {
        phi_4170_ = _e639;
    }
    let _e660 = phi_4170_;
    let _e666 = vec4<f32>(_e660.x, _e638.y, _e638.z, _e638.w);
    let _e672 = vec4<f32>(_e666.x, _e660.y, _e666.z, _e666.w);
    let _e678 = vec4<f32>(_e672.x, _e672.y, _e660.z, _e672.w);
    switch bitcast<i32>(0u) {
        default: {
            if (_e638.w == 0f) {
                break;
            }
            let _e681 = (1f - _e638.w);
            phi_4172_ = _e678;
            if (_e681 != 0f) {
                let _e685 = l0_.g2_[_e111];
                phi_4172_ = (_e678 + (unpack4x8unorm(_e685) * _e681));
            }
            let _e690 = phi_4172_;
            l0_.g2_[_e111] = pack4x8unorm(_e690);
            break;
        }
    }
    if (_e636 != 0u) {
        i0_.g2_[_e111] = _e636;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) D0_: u32, @location(0) @interpolate(flat, either) j1_: f32) {
    gl_FragCoord_1 = gl_FragCoord;
    D0_1 = D0_;
    j1_1 = j1_;
    main_1();
}
