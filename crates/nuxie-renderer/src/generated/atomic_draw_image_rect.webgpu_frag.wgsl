struct jg {
    r2_: array<vec2<u32>>,
}

struct m0Pe {
    r2_: array<u32>,
}

struct kg {
    r2_: array<vec4<f32>>,
}

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

struct n0Pe {
    r2_: array<u32>,
}

struct R4Pe {
    r2_: array<u32>,
}

@id(7) override dj: bool = true;
@id(6) override cj: bool = true;
@id(4) override aj: bool = true;
@id(0) override Wi: bool = true;
@id(1) override Xi: bool = true;
@id(2) override Yi: bool = true;

@group(0) @binding(3)
var<storage> VC: jg;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Pe;
@group(0) @binding(4)
var<storage> JB: kg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var N8_: sampler;
@group(2) @binding(0)
var<storage, read_write> n0_: n0Pe;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;
var<private> l2_1: vec2<f32>;
var<private> q5_1: f32;
var<private> V0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> R4_: R4Pe;
var<private> S3_1: u32;
var<private> r5_1: vec4<f32>;
var<private> T1_1: vec4<f32>;
var<private> J1_1: u32;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var local_3: vec3<f32>;
    var local_4: vec3<f32>;
    var local_5: vec3<f32>;
    var phi_6278_: f32;
    var phi_1739_: bool;
    var phi_5593_: f32;
    var phi_5592_: f32;
    var phi_5594_: f32;
    var phi_5597_: f32;
    var phi_5596_: f32;
    var phi_1776_: bool;
    var phi_5599_: f32;
    var phi_6245_: u32;
    var phi_5598_: f32;
    var phi_5622_: vec4<f32>;
    var phi_6244_: u32;
    var phi_5620_: vec4<f32>;
    var phi_5627_: f32;
    var phi_6081_: vec4<f32>;
    var phi_6021_: i32;
    var phi_6229_: vec4<f32>;
    var phi_6242_: vec4<f32>;
    var phi_1436_: bool;
    var phi_6268_: u32;
    var phi_6296_: f32;
    var phi_7792_: f32;
    var phi_6326_: f32;
    var phi_6357_: vec4<f32>;
    var phi_1504_: bool;
    var phi_6366_: f32;
    var phi_6367_: f32;
    var phi_7460_: vec4<f32>;
    var phi_7320_: i32;
    var phi_7805_: vec4<f32>;
    var phi_7818_: vec3<f32>;
    var phi_7820_: vec4<f32>;

    let _e89 = gl_FragCoord_1;
    let _e90 = _e89.xy;
    let _e93 = bitcast<vec2<u32>>(vec2<i32>(floor(_e90)));
    let _e95 = j.P6_;
    let _e124 = bitcast<i32>((((((_e93.y >> bitcast<u32>(5u)) * (((_e95 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e93.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e93.x & 28u) << bitcast<u32>(5u)) + ((_e93.y & 28u) << bitcast<u32>(2i)))) + (((_e93.y & 3u) << bitcast<u32>(2i)) + (_e93.x & 3u))));
    let _e125 = l2_1;
    let _e126 = textureSample(TB, U4_, _e125);
    let _e127 = q5_1;
    let _e128 = min(_e127, 1f);
    phi_6278_ = _e128;
    if Xi {
        let _e129 = V0_1;
        let _e132 = min(_e129.xy, _e129.zw);
        phi_6278_ = clamp(min(_e132.x, _e132.y), 0f, _e128);
    }
    let _e138 = phi_6278_;
    let _e141 = R4_.r2_[_e124];
    let _e143 = (_e141 >> bitcast<u32>(17u));
    let _e147 = ((f32((_e141 & 131071u)) * 0.00048828125f) + -32f);
    let _e150 = VC.r2_[_e143];
    phi_5592_ = _e147;
    if ((_e150.x & 768u) != 0u) {
        let _e154 = abs(_e147);
        phi_1739_ = aj;
        if aj {
            phi_1739_ = ((_e150.x & 512u) != 0u);
        }
        let _e158 = phi_1739_;
        phi_5593_ = _e154;
        if _e158 {
            phi_5593_ = (1f - abs(((fract((_e154 * 0.5f)) * 2f) + -1f)));
        }
        let _e166 = phi_5593_;
        phi_5592_ = _e166;
    }
    let _e168 = phi_5592_;
    let _e169 = clamp(_e168, 0f, 1f);
    phi_5596_ = _e169;
    if Wi {
        let _e171 = (_e150.x >> bitcast<u32>(16u));
        phi_5597_ = _e169;
        if (_e171 != 0u) {
            let _e175 = m0_.r2_[_e124];
            if (_e171 == (_e175 >> bitcast<u32>(16i))) {
                phi_5594_ = min(_e169, unpack2x16float(_e175).x);
            } else {
                phi_5594_ = 0f;
            }
            let _e183 = phi_5594_;
            phi_5597_ = _e183;
        }
        let _e185 = phi_5597_;
        phi_5596_ = _e185;
    }
    let _e187 = phi_5596_;
    phi_1776_ = Xi;
    if Xi {
        phi_1776_ = ((_e150.x & 1024u) != 0u);
    }
    let _e191 = phi_1776_;
    phi_5599_ = _e187;
    if _e191 {
        let _e192 = (_e143 * 8u);
        let _e196 = JB.r2_[(_e192 + 2u)];
        let _e207 = JB.r2_[(_e192 + 3u)];
        let _e212 = _e207.zw;
        let _e214 = ((abs(((mat2x2<f32>(vec2<f32>(_e196.x, _e196.y), vec2<f32>(_e196.z, _e196.w)) * _e90) + _e207.xy)) * _e212) - _e212);
        phi_5599_ = min(_e187, clamp((min(_e214.x, _e214.y) + 0.5f), 0f, 1f));
    }
    let _e222 = phi_5599_;
    let _e223 = (_e150.x & 15u);
    let _e226 = ((_e150.x >> bitcast<u32>(4i)) & 15u);
    let _e228 = (Yi && (_e226 != 0u));
    if (_e223 <= 1u) {
        let _e233 = (Wi && (_e223 == 0u));
        phi_6245_ = 0u;
        if _e233 {
            phi_6245_ = (_e150.y | pack2x16float(vec2<f32>(_e222, 0f)));
        }
        let _e238 = phi_6245_;
        phi_6244_ = _e238;
        phi_5620_ = select(unpack4x8unorm(_e150.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e233));
    } else {
        let _e241 = (_e143 * 8u);
        let _e244 = JB.r2_[_e241];
        let _e255 = JB.r2_[(_e241 + 1u)];
        let _e258 = ((mat2x2<f32>(vec2<f32>(_e244.x, _e244.y), vec2<f32>(_e244.z, _e244.w)) * _e90) + _e255.xy);
        let _e264 = j.L8_;
        let _e266 = j.M8_;
        if (f32(_e223) == 2f) {
            phi_5598_ = _e258.x;
        } else {
            phi_5598_ = length(_e258);
        }
        let _e276 = phi_5598_;
        let _e282 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e276, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e255.z < 0f))) + ((max(0f, _e255.z) * 0.001953125f) + 0.0009765625f)), ((_e255.w * _e264) + _e266)), 0f);
        phi_5622_ = _e282;
        if !(_e228) {
            let _e286 = (_e282.xyz * _e282.w);
            phi_5622_ = vec4<f32>(_e286.x, _e286.y, _e286.z, (_e282.w * abs(bitcast<f32>(_e150.y))));
        }
        let _e296 = phi_5622_;
        phi_6244_ = 0u;
        phi_5620_ = _e296;
    }
    let _e298 = phi_6244_;
    let _e300 = phi_5620_;
    phi_6242_ = _e300;
    if _e228 {
        phi_6229_ = _e300;
        if ((_e300.w * _e222) != 0f) {
            let _e306 = n0_.r2_[_e124];
            let _e307 = unpack4x8unorm(_e306);
            let _e308 = _e300.xyz;
            local_5 = _e308;
            let _e309 = _e307.xyz;
            if (_e307.w != 0f) {
                phi_5627_ = (1f / _e307.w);
            } else {
                phi_5627_ = 0f;
            }
            let _e314 = phi_5627_;
            let _e315 = (_e309 * _e314);
            local_3 = _e315;
            switch bitcast<i32>(_e226) {
                case 11: {
                    let _e317 = local_5;
                    local_4 = (_e317 * _e315);
                    break;
                }
                case 1: {
                    let _e319 = local_5;
                    local_4 = ((_e319 + _e315) - (_e319 * _e315));
                    break;
                }
                case 2: {
                    let _e323 = local_5;
                    let _e324 = (_e323 * _e315);
                    local_4 = (select(_e324, (((_e323 + _e315) - _e324) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e315 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e331 = local_5;
                    local_4 = min(_e331, _e315);
                    break;
                }
                case 4: {
                    let _e333 = local_5;
                    local_4 = max(_e333, _e315);
                    break;
                }
                case 5: {
                    let _e336 = clamp(_e309, vec3<f32>(0f, 0f, 0f), _e307.www);
                    let _e342 = vec4<f32>(_e336.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e348 = vec4<f32>(_e342.x, _e336.y, _e342.z, _e342.w);
                    let _e355 = local_5;
                    let _e358 = (clamp((vec3<f32>(1f, 1f, 1f) - _e355), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e307.w);
                    let _e359 = vec4<f32>(_e348.x, _e348.y, _e336.z, _e348.w).xyz;
                    local_4 = select(min(vec3<f32>(1f, 1f, 1f), (_e359 / _e358)), sign(_e359), (_e358 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e365 = local_5;
                    local_5 = clamp(_e365, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e368 = clamp(_e309, vec3<f32>(0f, 0f, 0f), _e307.www);
                    let _e374 = vec4<f32>(_e368.x, _e307.y, _e307.z, _e307.w);
                    let _e380 = vec4<f32>(_e374.x, _e368.y, _e374.z, _e374.w);
                    phi_6081_ = vec4<f32>(_e380.x, _e380.y, _e368.z, _e380.w);
                    if (_e307.w == 0f) {
                        phi_6081_ = vec4<f32>(_e368.x, _e368.y, _e368.z, 1f);
                    }
                    let _e390 = phi_6081_;
                    let _e394 = (vec3(_e390.w) - _e390.xyz);
                    let _e395 = local_5;
                    local_4 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e394 / (_e395 * _e390.w))), sign(_e394), (_e395 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e403 = local_5;
                    let _e404 = (_e403 * _e315);
                    local_4 = (select(_e404, (((_e403 + _e315) - _e404) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e403 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_6021_ = 0i;
                    loop {
                        let _e412 = phi_6021_;
                        if (_e412 < 3i) {
                            let _e415 = local_5[_e412];
                            if (_e415 <= 0.5f) {
                                let _e418 = local_3[_e412];
                                local_4[_e412] = (1f - _e418);
                            } else {
                                let _e422 = local_3[_e412];
                                if (_e422 <= 0.25f) {
                                    let _e424 = local_3[_e412];
                                    let _e427 = local_3[_e412];
                                    local_4[_e412] = ((((16f * _e424) - 12f) * _e427) + 3f);
                                } else {
                                    let _e431 = local_3[_e412];
                                    local_4[_e412] = (inverseSqrt(_e431) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_6021_ = (_e412 + 1i);
                        }
                    }
                    let _e436 = local_5;
                    let _e440 = local_4;
                    local_4 = (_e315 + ((_e315 * ((_e436 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e440));
                    break;
                }
                case 9: {
                    let _e443 = local_5;
                    local_4 = abs((_e315 - _e443));
                    break;
                }
                case 10: {
                    let _e446 = local_5;
                    local_4 = ((_e446 + _e315) - ((_e446 * 2f) * _e315));
                    break;
                }
                case 12: {
                    if cj {
                        let _e451 = local_5;
                        let _e452 = clamp(_e451, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e452;
                        let _e467 = (_e452 - vec3(min(min(_e452.x, _e452.y), _e452.z)));
                        let _e475 = (_e467 * ((max(max(_e315.x, _e315.y), _e315.z) - min(min(_e315.x, _e315.y), _e315.z)) / max(0.000062f, max(max(_e467.x, _e467.y), _e467.z))));
                        let _e476 = dot(_e315, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e479 = (_e475 - vec3(dot(_e475, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e492 = (vec2<f32>(_e476, (1f - _e476)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e479.x, _e479.y), _e479.z)), max(max(_e479.x, _e479.y), _e479.z))));
                        local_4 = ((_e479 * min(1f, min(_e492.x, _e492.y))) + vec3(_e476));
                    }
                    break;
                }
                case 13: {
                    if cj {
                        let _e500 = local_5;
                        let _e501 = clamp(_e500, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e501;
                        let _e516 = (_e315 - vec3(min(min(_e315.x, _e315.y), _e315.z)));
                        let _e524 = (_e516 * ((max(max(_e501.x, _e501.y), _e501.z) - min(min(_e501.x, _e501.y), _e501.z)) / max(0.000062f, max(max(_e516.x, _e516.y), _e516.z))));
                        let _e525 = dot(_e315, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e528 = (_e524 - vec3(dot(_e524, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e541 = (vec2<f32>(_e525, (1f - _e525)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e528.x, _e528.y), _e528.z)), max(max(_e528.x, _e528.y), _e528.z))));
                        local_4 = ((_e528 * min(1f, min(_e541.x, _e541.y))) + vec3(_e525));
                    }
                    break;
                }
                case 14: {
                    if cj {
                        let _e549 = local_5;
                        let _e550 = clamp(_e549, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e550;
                        let _e551 = dot(_e315, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e554 = (_e550 - vec3(dot(_e550, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e567 = (vec2<f32>(_e551, (1f - _e551)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e554.x, _e554.y), _e554.z)), max(max(_e554.x, _e554.y), _e554.z))));
                        local_4 = ((_e554 * min(1f, min(_e567.x, _e567.y))) + vec3(_e551));
                    }
                    break;
                }
                case 15: {
                    if cj {
                        let _e575 = local_5;
                        let _e576 = clamp(_e575, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e576;
                        let _e577 = dot(_e576, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e580 = (_e315 - vec3(dot(_e315, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e593 = (vec2<f32>(_e577, (1f - _e577)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e580.x, _e580.y), _e580.z)), max(max(_e580.x, _e580.y), _e580.z))));
                        local_4 = ((_e580 * min(1f, min(_e593.x, _e593.y))) + vec3(_e577));
                    }
                    break;
                }
                default: {
                }
            }
            let _e601 = local_4;
            let _e603 = mix(_e308, _e601, vec3(_e307.w));
            let _e609 = vec4<f32>(_e603.x, _e300.y, _e300.z, _e300.w);
            let _e615 = vec4<f32>(_e609.x, _e603.y, _e609.z, _e609.w);
            phi_6229_ = vec4<f32>(_e615.x, _e615.y, _e603.z, _e615.w);
        }
        let _e623 = phi_6229_;
        let _e626 = (_e623.xyz * _e623.w);
        let _e632 = vec4<f32>(_e626.x, _e623.y, _e623.z, _e623.w);
        let _e638 = vec4<f32>(_e632.x, _e626.y, _e632.z, _e632.w);
        phi_6242_ = vec4<f32>(_e638.x, _e638.y, _e626.z, _e638.w);
    }
    let _e646 = phi_6242_;
    let _e647 = (_e646 * _e222);
    phi_1436_ = Wi;
    if Wi {
        let _e648 = S3_1;
        phi_1436_ = (_e648 != 0u);
    }
    let _e651 = phi_1436_;
    phi_7792_ = _e138;
    if _e651 {
        if (_e298 != 0u) {
            phi_6268_ = _e298;
        } else {
            let _e655 = m0_.r2_[_e124];
            phi_6268_ = _e655;
        }
        let _e657 = phi_6268_;
        let _e658 = S3_1;
        if (_e658 == (_e657 >> bitcast<u32>(16i))) {
            phi_6296_ = min(_e138, unpack2x16float(_e657).x);
        } else {
            phi_6296_ = 0f;
        }
        let _e666 = phi_6296_;
        phi_7792_ = _e666;
    }
    let _e668 = phi_7792_;
    let _e670 = r5_1[3u];
    phi_6357_ = _e126;
    if (_e670 != 0f) {
        let _e672 = r5_1;
        let _e674 = j.L8_;
        let _e676 = j.M8_;
        let _e679 = abs(_e672.z);
        let _e680 = floor(_e679);
        let _e683 = (((_e679 - _e680) * 8f) + 0.0009765625f);
        if (floor(_e683) == 2f) {
            phi_6326_ = _e672.x;
        } else {
            phi_6326_ = length(_e672.xy);
        }
        let _e692 = phi_6326_;
        let _e698 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e692, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e672.z < 0f))) + _e683), ((_e680 * _e674) + _e676)), 0f);
        let _e701 = (_e698.xyz * _e698.w);
        let _e707 = vec4<f32>(_e701.x, _e698.y, _e698.z, _e698.w);
        let _e713 = vec4<f32>(_e707.x, _e701.y, _e707.z, _e707.w);
        phi_6357_ = (_e126 * vec4<f32>(_e713.x, _e713.y, _e701.z, _e713.w));
    }
    let _e722 = phi_6357_;
    let _e723 = T1_1;
    let _e724 = (_e722 * _e723);
    phi_1504_ = Yi;
    if Yi {
        let _e725 = J1_1;
        phi_1504_ = (_e725 != 0u);
    }
    let _e728 = phi_1504_;
    phi_7805_ = _e724;
    if _e728 {
        let _e731 = n0_.r2_[_e124];
        let _e736 = ((unpack4x8unorm(_e731) * (1f - _e647.w)) + _e647);
        if (_e724.w != 0f) {
            phi_6366_ = (1f / _e724.w);
        } else {
            phi_6366_ = 0f;
        }
        let _e742 = phi_6366_;
        let _e743 = (_e724.xyz * _e742);
        let _e744 = J1_1;
        local_2 = _e743;
        let _e745 = _e736.xyz;
        if (_e736.w != 0f) {
            phi_6367_ = (1f / _e736.w);
        } else {
            phi_6367_ = 0f;
        }
        let _e750 = phi_6367_;
        let _e751 = (_e745 * _e750);
        local = _e751;
        switch bitcast<i32>(_e744) {
            case 11: {
                let _e753 = local_2;
                local_1 = (_e753 * _e751);
                break;
            }
            case 1: {
                let _e755 = local_2;
                local_1 = ((_e755 + _e751) - (_e755 * _e751));
                break;
            }
            case 2: {
                let _e759 = local_2;
                let _e760 = (_e759 * _e751);
                local_1 = (select(_e760, (((_e759 + _e751) - _e760) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e751 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e767 = local_2;
                local_1 = min(_e767, _e751);
                break;
            }
            case 4: {
                let _e769 = local_2;
                local_1 = max(_e769, _e751);
                break;
            }
            case 5: {
                let _e772 = clamp(_e745, vec3<f32>(0f, 0f, 0f), _e736.www);
                let _e778 = vec4<f32>(_e772.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e784 = vec4<f32>(_e778.x, _e772.y, _e778.z, _e778.w);
                let _e791 = local_2;
                let _e794 = (clamp((vec3<f32>(1f, 1f, 1f) - _e791), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e736.w);
                let _e795 = vec4<f32>(_e784.x, _e784.y, _e772.z, _e784.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e795 / _e794)), sign(_e795), (_e794 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e801 = local_2;
                local_2 = clamp(_e801, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e804 = clamp(_e745, vec3<f32>(0f, 0f, 0f), _e736.www);
                let _e810 = vec4<f32>(_e804.x, _e736.y, _e736.z, _e736.w);
                let _e816 = vec4<f32>(_e810.x, _e804.y, _e810.z, _e810.w);
                phi_7460_ = vec4<f32>(_e816.x, _e816.y, _e804.z, _e816.w);
                if (_e736.w == 0f) {
                    phi_7460_ = vec4<f32>(_e804.x, _e804.y, _e804.z, 1f);
                }
                let _e826 = phi_7460_;
                let _e830 = (vec3(_e826.w) - _e826.xyz);
                let _e831 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e830 / (_e831 * _e826.w))), sign(_e830), (_e831 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e839 = local_2;
                let _e840 = (_e839 * _e751);
                local_1 = (select(_e840, (((_e839 + _e751) - _e840) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e839 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_7320_ = 0i;
                loop {
                    let _e848 = phi_7320_;
                    if (_e848 < 3i) {
                        let _e851 = local_2[_e848];
                        if (_e851 <= 0.5f) {
                            let _e854 = local[_e848];
                            local_1[_e848] = (1f - _e854);
                        } else {
                            let _e858 = local[_e848];
                            if (_e858 <= 0.25f) {
                                let _e860 = local[_e848];
                                let _e863 = local[_e848];
                                local_1[_e848] = ((((16f * _e860) - 12f) * _e863) + 3f);
                            } else {
                                let _e867 = local[_e848];
                                local_1[_e848] = (inverseSqrt(_e867) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_7320_ = (_e848 + 1i);
                    }
                }
                let _e872 = local_2;
                let _e876 = local_1;
                local_1 = (_e751 + ((_e751 * ((_e872 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e876));
                break;
            }
            case 9: {
                let _e879 = local_2;
                local_1 = abs((_e751 - _e879));
                break;
            }
            case 10: {
                let _e882 = local_2;
                local_1 = ((_e882 + _e751) - ((_e882 * 2f) * _e751));
                break;
            }
            case 12: {
                if cj {
                    let _e887 = local_2;
                    let _e888 = clamp(_e887, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e888;
                    let _e903 = (_e888 - vec3(min(min(_e888.x, _e888.y), _e888.z)));
                    let _e911 = (_e903 * ((max(max(_e751.x, _e751.y), _e751.z) - min(min(_e751.x, _e751.y), _e751.z)) / max(0.000062f, max(max(_e903.x, _e903.y), _e903.z))));
                    let _e912 = dot(_e751, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e915 = (_e911 - vec3(dot(_e911, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e928 = (vec2<f32>(_e912, (1f - _e912)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e915.x, _e915.y), _e915.z)), max(max(_e915.x, _e915.y), _e915.z))));
                    local_1 = ((_e915 * min(1f, min(_e928.x, _e928.y))) + vec3(_e912));
                }
                break;
            }
            case 13: {
                if cj {
                    let _e936 = local_2;
                    let _e937 = clamp(_e936, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e937;
                    let _e952 = (_e751 - vec3(min(min(_e751.x, _e751.y), _e751.z)));
                    let _e960 = (_e952 * ((max(max(_e937.x, _e937.y), _e937.z) - min(min(_e937.x, _e937.y), _e937.z)) / max(0.000062f, max(max(_e952.x, _e952.y), _e952.z))));
                    let _e961 = dot(_e751, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e964 = (_e960 - vec3(dot(_e960, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e977 = (vec2<f32>(_e961, (1f - _e961)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e964.x, _e964.y), _e964.z)), max(max(_e964.x, _e964.y), _e964.z))));
                    local_1 = ((_e964 * min(1f, min(_e977.x, _e977.y))) + vec3(_e961));
                }
                break;
            }
            case 14: {
                if cj {
                    let _e985 = local_2;
                    let _e986 = clamp(_e985, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e986;
                    let _e987 = dot(_e751, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e990 = (_e986 - vec3(dot(_e986, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e1003 = (vec2<f32>(_e987, (1f - _e987)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e990.x, _e990.y), _e990.z)), max(max(_e990.x, _e990.y), _e990.z))));
                    local_1 = ((_e990 * min(1f, min(_e1003.x, _e1003.y))) + vec3(_e987));
                }
                break;
            }
            case 15: {
                if cj {
                    let _e1011 = local_2;
                    let _e1012 = clamp(_e1011, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e1012;
                    let _e1013 = dot(_e1012, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e1016 = (_e751 - vec3(dot(_e751, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e1029 = (vec2<f32>(_e1013, (1f - _e1013)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e1016.x, _e1016.y), _e1016.z)), max(max(_e1016.x, _e1016.y), _e1016.z))));
                    local_1 = ((_e1016 * min(1f, min(_e1029.x, _e1029.y))) + vec3(_e1013));
                }
                break;
            }
            default: {
            }
        }
        let _e1037 = local_1;
        let _e1040 = (mix(_e743, _e1037, vec3(_e736.w)) * _e724.w);
        let _e1046 = vec4<f32>(_e1040.x, _e724.y, _e724.z, _e724.w);
        let _e1052 = vec4<f32>(_e1046.x, _e1040.y, _e1046.z, _e1046.w);
        phi_7805_ = vec4<f32>(_e1052.x, _e1052.y, _e1040.z, _e1052.w);
    }
    let _e1060 = phi_7805_;
    let _e1061 = (_e1060 * _e668);
    let _e1065 = ((_e647 * (1f - _e1061.w)) + _e1061);
    let _e1066 = _e1065.xyz;
    let _e1069 = j.F3_;
    let _e1071 = j.G3_;
    if (dj && (_e1065.w != 0f)) {
        phi_7818_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e89.x) + (0.00583715f * _e89.y))))) * _e1069) + _e1071)) + _e1066);
    } else {
        phi_7818_ = _e1066;
    }
    let _e1087 = phi_7818_;
    let _e1093 = vec4<f32>(_e1087.x, _e1065.y, _e1065.z, _e1065.w);
    let _e1099 = vec4<f32>(_e1093.x, _e1087.y, _e1093.z, _e1093.w);
    let _e1105 = vec4<f32>(_e1099.x, _e1099.y, _e1087.z, _e1099.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e1087.x + _e1087.y) + _e1087.z) + _e1065.w) == 0f) {
                break;
            }
            let _e1111 = (1f - _e1065.w);
            phi_7820_ = _e1105;
            if (_e1111 != 0f) {
                let _e1115 = n0_.r2_[_e124];
                phi_7820_ = (_e1105 + (unpack4x8unorm(_e1115) * _e1111));
            }
            let _e1120 = phi_7820_;
            n0_.r2_[_e124] = pack4x8unorm(_e1120);
            break;
        }
    }
    if (_e298 != 0u) {
        m0_.r2_[_e124] = _e298;
    }
    R4_.r2_[_e124] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) l2_: vec2<f32>, @location(1) q5_: f32, @location(3) V0_: vec4<f32>, @location(5) @interpolate(flat, either) S3_: u32, @location(2) r5_: vec4<f32>, @location(4) @interpolate(flat, either) T1_: vec4<f32>, @location(6) @interpolate(flat, either) J1_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    l2_1 = l2_;
    q5_1 = q5_;
    V0_1 = V0_;
    S3_1 = S3_;
    r5_1 = r5_;
    T1_1 = T1_;
    J1_1 = J1_;
    main_1();
}
