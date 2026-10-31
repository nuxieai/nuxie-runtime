struct nd {
    z2_: array<vec4<u32>>,
}

struct n0hf {
    z2_: array<u32>,
}

struct Cg {
    z2_: array<vec4<f32>>,
}

struct RB {
    Kd: f32,
    Ue: f32,
    ah: f32,
    bh: f32,
    V5_: u32,
    Ia: u32,
    Lg: u32,
    Ng: u32,
    M8_: vec4<i32>,
    Ri: vec2<f32>,
    Ve: vec2<f32>,
    y2_: u32,
    Vi: f32,
    z6_: u32,
    n3_: f32,
    We: f32,
    Eg: u32,
    K3_: f32,
    L3_: f32,
    Xe: f32,
    Oi: u32,
    Ha: u32,
    T8_: f32,
    U8_: f32,
}

struct p0hf {
    z2_: array<u32>,
}

struct d5hf {
    z2_: array<u32>,
}

@id(8) override Cj: bool = true;
@id(7) override Bj: bool = true;
@id(9) override Dj: bool = true;
@id(5) override zj: bool = true;
@id(0) override uj: bool = true;
@id(1) override vj: bool = true;
@id(2) override wj: bool = true;

@group(0) @binding(2)
var<storage> JB: nd;
@group(2) @binding(1)
var<storage, read_write> n0_: n0hf;
@group(0) @binding(3)
var<storage> KB: Cg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: RB;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(3) @binding(7)
var V8_: sampler;
@group(2) @binding(0)
var<storage, read_write> p0_: p0hf;
@group(2) @binding(3)
var<storage, read_write> d5_: d5hf;
var<private> J0_1: u32;
@group(0) @binding(9)
var ID: texture_2d<f32>;
@group(3) @binding(9)
var Xa: sampler;
var<private> W2_1: vec2<f32>;
@group(3) @binding(8)
var ib: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(1) @binding(10)
var SB: texture_2d<f32>;
@group(1) @binding(12)
var x4_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1626_: bool;
    var phi_3794_: f32;
    var phi_3793_: f32;
    var phi_3795_: f32;
    var phi_3798_: f32;
    var phi_3797_: f32;
    var phi_1663_: bool;
    var phi_3802_: f32;
    var phi_3799_: f32;
    var phi_3800_: f32;
    var phi_3801_: vec2<f32>;
    var phi_3822_: vec4<f32>;
    var phi_4488_: u32;
    var phi_4487_: u32;
    var phi_3820_: vec4<f32>;
    var phi_3829_: f32;
    var phi_4251_: i32;
    var phi_4315_: vec4<f32>;
    var phi_4472_: vec4<f32>;
    var phi_4485_: vec4<f32>;
    var phi_4512_: vec3<f32>;
    var phi_4514_: vec4<f32>;

    let _e87 = gl_FragCoord_1;
    let _e88 = _e87.xy;
    let _e91 = bitcast<vec2<u32>>(vec2<i32>(floor(_e88)));
    let _e93 = j.V5_;
    let _e122 = bitcast<i32>((((((_e91.y >> bitcast<u32>(5u)) * (((_e93 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e91.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e91.x & 28u) << bitcast<u32>(5u)) + ((_e91.y & 28u) << bitcast<u32>(2i)))) + (((_e91.y & 3u) << bitcast<u32>(2i)) + (_e91.x & 3u))));
    let _e125 = d5_.z2_[_e122];
    let _e127 = (_e125 >> bitcast<u32>(17u));
    let _e128 = J0_1;
    let _e132 = W2_1;
    let _e133 = textureSampleLevel(ID, Xa, _e132, 0f);
    d5_.z2_[_e122] = (((_e128 << bitcast<u32>(17u)) + 65536u) + bitcast<u32>(i32(round((clamp(_e133.x, 0f, 1f) * 2048f)))));
    let _e144 = ((f32((_e125 & 131071u)) * 0.00048828125f) + -32f);
    let _e149 = JB.z2_[((_e127 * 4u) + 1u)];
    phi_3793_ = _e144;
    if ((_e149.z & 768u) != 0u) {
        let _e153 = abs(_e144);
        phi_1626_ = zj;
        if zj {
            phi_1626_ = ((_e149.z & 512u) != 0u);
        }
        let _e157 = phi_1626_;
        phi_3794_ = _e153;
        if _e157 {
            phi_3794_ = (1f - abs(((fract((_e153 * 0.5f)) * 2f) + -1f)));
        }
        let _e165 = phi_3794_;
        phi_3793_ = _e165;
    }
    let _e167 = phi_3793_;
    let _e168 = clamp(_e167, 0f, 1f);
    phi_3797_ = _e168;
    if uj {
        let _e170 = (_e149.z >> bitcast<u32>(16u));
        phi_3798_ = _e168;
        if (_e170 != 0u) {
            let _e174 = n0_.z2_[_e122];
            if (_e170 == (_e174 >> bitcast<u32>(16i))) {
                phi_3795_ = min(_e168, unpack2x16float(_e174).x);
            } else {
                phi_3795_ = 0f;
            }
            let _e182 = phi_3795_;
            phi_3798_ = _e182;
        }
        let _e184 = phi_3798_;
        phi_3797_ = _e184;
    }
    let _e186 = phi_3797_;
    phi_1663_ = vj;
    if vj {
        phi_1663_ = ((_e149.z & 1024u) != 0u);
    }
    let _e190 = phi_1663_;
    phi_3802_ = _e186;
    if _e190 {
        let _e191 = (_e127 * 8u);
        let _e195 = KB.z2_[(_e191 + 2u)];
        let _e206 = KB.z2_[(_e191 + 3u)];
        let _e211 = _e206.zw;
        let _e213 = ((abs(((mat2x2<f32>(vec2<f32>(_e195.x, _e195.y), vec2<f32>(_e195.z, _e195.w)) * _e88) + _e206.xy)) * _e211) - _e211);
        phi_3802_ = min(_e186, clamp((min(_e213.x, _e213.y) + 0.5f), 0f, 1f));
    }
    let _e221 = phi_3802_;
    let _e222 = (_e149.z & 15u);
    let _e225 = ((_e149.z >> bitcast<u32>(4i)) & 15u);
    let _e227 = (wj && (_e225 != 0u));
    if (_e222 <= 1u) {
        let _e304 = (uj && (_e222 == 0u));
        phi_4488_ = 0u;
        if _e304 {
            phi_4488_ = (_e149.w | pack2x16float(vec2<f32>(_e221, 0f)));
        }
        let _e309 = phi_4488_;
        phi_4487_ = _e309;
        phi_3820_ = select(unpack4x8unorm(_e149.w), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e304));
    } else {
        let _e229 = (_e127 * 8u);
        let _e232 = KB.z2_[_e229];
        let _e243 = KB.z2_[(_e229 + 1u)];
        let _e246 = ((mat2x2<f32>(vec2<f32>(_e232.x, _e232.y), vec2<f32>(_e232.z, _e232.w)) * _e88) + _e243.xy);
        let _e250 = f32(_e222);
        let _e252 = j.T8_;
        let _e254 = j.U8_;
        let _e259 = ((_e243.w * _e252) + _e254);
        switch bitcast<i32>(0u) {
            default: {
                if (Dj && (_e250 == 4f)) {
                    let _e273 = normalize(_e246);
                    let _e276 = acos(clamp(_e273.x, -1f, 1f));
                    if (_e273.y >= 0f) {
                        phi_3800_ = _e276;
                    } else {
                        phi_3800_ = -(_e276);
                    }
                    let _e281 = phi_3800_;
                    phi_3801_ = vec2<f32>((_e281 * 0.15915494f), _e259);
                    break;
                } else {
                    if (_e250 == 2f) {
                        phi_3799_ = _e246.x;
                    } else {
                        phi_3799_ = length(_e246);
                    }
                    let _e267 = phi_3799_;
                    phi_3801_ = vec2<f32>(((clamp(_e267, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e243.z < 0f))) + ((max(0f, _e243.z) * 0.001953125f) + 0.0009765625f)), _e259);
                    break;
                }
            }
        }
        let _e285 = phi_3801_;
        let _e286 = textureSampleLevel(YC, V8_, _e285, 0f);
        phi_3822_ = _e286;
        if !(_e227) {
            let _e290 = (_e286.xyz * _e286.w);
            phi_3822_ = vec4<f32>(_e290.x, _e290.y, _e290.z, (_e286.w * abs(bitcast<f32>(_e149.w))));
        }
        let _e300 = phi_3822_;
        phi_4487_ = 0u;
        phi_3820_ = _e300;
    }
    let _e313 = phi_4487_;
    let _e315 = phi_3820_;
    phi_4485_ = _e315;
    if _e227 {
        phi_4472_ = _e315;
        if ((_e315.w * _e221) != 0f) {
            let _e321 = p0_.z2_[_e122];
            let _e322 = unpack4x8unorm(_e321);
            let _e323 = _e315.xyz;
            local_2 = _e323;
            let _e324 = _e322.xyz;
            if (_e322.w != 0f) {
                phi_3829_ = (1f / _e322.w);
            } else {
                phi_3829_ = 0f;
            }
            let _e329 = phi_3829_;
            let _e330 = (_e324 * _e329);
            local = _e330;
            switch bitcast<i32>(_e225) {
                case 11: {
                    let _e614 = local_2;
                    local_1 = (_e614 * _e330);
                    break;
                }
                case 1: {
                    let _e610 = local_2;
                    local_1 = ((_e610 + _e330) - (_e610 * _e330));
                    break;
                }
                case 2: {
                    let _e602 = local_2;
                    let _e603 = (_e602 * _e330);
                    local_1 = (select(_e603, (((_e602 + _e330) - _e603) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e330 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e600 = local_2;
                    local_1 = min(_e600, _e330);
                    break;
                }
                case 4: {
                    let _e598 = local_2;
                    local_1 = max(_e598, _e330);
                    break;
                }
                case 5: {
                    let _e569 = clamp(_e324, vec3<f32>(0f, 0f, 0f), _e322.www);
                    let _e575 = vec4<f32>(_e569.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e581 = vec4<f32>(_e575.x, _e569.y, _e575.z, _e575.w);
                    let _e588 = local_2;
                    let _e591 = (clamp((vec3<f32>(1f, 1f, 1f) - _e588), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e322.w);
                    let _e592 = vec4<f32>(_e581.x, _e581.y, _e569.z, _e581.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e592 / _e591)), sign(_e592), (_e591 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e530 = local_2;
                    local_2 = clamp(_e530, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e533 = clamp(_e324, vec3<f32>(0f, 0f, 0f), _e322.www);
                    let _e539 = vec4<f32>(_e533.x, _e322.y, _e322.z, _e322.w);
                    let _e545 = vec4<f32>(_e539.x, _e533.y, _e539.z, _e539.w);
                    phi_4315_ = vec4<f32>(_e545.x, _e545.y, _e533.z, _e545.w);
                    if (_e322.w == 0f) {
                        phi_4315_ = vec4<f32>(_e533.x, _e533.y, _e533.z, 1f);
                    }
                    let _e555 = phi_4315_;
                    let _e559 = (vec3(_e555.w) - _e555.xyz);
                    let _e560 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e559 / (_e560 * _e555.w))), sign(_e559), (_e560 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e522 = local_2;
                    let _e523 = (_e522 * _e330);
                    local_1 = (select(_e523, (((_e522 + _e330) - _e523) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e522 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_4251_ = 0i;
                    loop {
                        let _e491 = phi_4251_;
                        if (_e491 < 3i) {
                            let _e494 = local_2[_e491];
                            if (_e494 <= 0.5f) {
                                let _e511 = local[_e491];
                                local_1[_e491] = (1f - _e511);
                            } else {
                                let _e497 = local[_e491];
                                if (_e497 <= 0.25f) {
                                    let _e503 = local[_e491];
                                    let _e506 = local[_e491];
                                    local_1[_e491] = ((((16f * _e503) - 12f) * _e506) + 3f);
                                } else {
                                    let _e499 = local[_e491];
                                    local_1[_e491] = (inverseSqrt(_e499) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_4251_ = (_e491 + 1i);
                        }
                    }
                    let _e515 = local_2;
                    let _e519 = local_1;
                    local_1 = (_e330 + ((_e330 * ((_e515 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e519));
                    break;
                }
                case 9: {
                    let _e487 = local_2;
                    local_1 = abs((_e330 - _e487));
                    break;
                }
                case 10: {
                    let _e482 = local_2;
                    local_1 = ((_e482 + _e330) - ((_e482 * 2f) * _e330));
                    break;
                }
                case 12: {
                    if Bj {
                        let _e433 = local_2;
                        let _e434 = clamp(_e433, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e434;
                        let _e449 = (_e434 - vec3(min(min(_e434.x, _e434.y), _e434.z)));
                        let _e457 = (_e449 * ((max(max(_e330.x, _e330.y), _e330.z) - min(min(_e330.x, _e330.y), _e330.z)) / max(0.000062f, max(max(_e449.x, _e449.y), _e449.z))));
                        let _e458 = dot(_e330, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e461 = (_e457 - vec3(dot(_e457, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e474 = (vec2<f32>(_e458, (1f - _e458)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e461.x, _e461.y), _e461.z)), max(max(_e461.x, _e461.y), _e461.z))));
                        local_1 = ((_e461 * min(1f, min(_e474.x, _e474.y))) + vec3(_e458));
                    }
                    break;
                }
                case 13: {
                    if Bj {
                        let _e384 = local_2;
                        let _e385 = clamp(_e384, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e385;
                        let _e400 = (_e330 - vec3(min(min(_e330.x, _e330.y), _e330.z)));
                        let _e408 = (_e400 * ((max(max(_e385.x, _e385.y), _e385.z) - min(min(_e385.x, _e385.y), _e385.z)) / max(0.000062f, max(max(_e400.x, _e400.y), _e400.z))));
                        let _e409 = dot(_e330, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e412 = (_e408 - vec3(dot(_e408, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e425 = (vec2<f32>(_e409, (1f - _e409)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e412.x, _e412.y), _e412.z)), max(max(_e412.x, _e412.y), _e412.z))));
                        local_1 = ((_e412 * min(1f, min(_e425.x, _e425.y))) + vec3(_e409));
                    }
                    break;
                }
                case 14: {
                    if Bj {
                        let _e358 = local_2;
                        let _e359 = clamp(_e358, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e359;
                        let _e360 = dot(_e330, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e363 = (_e359 - vec3(dot(_e359, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e376 = (vec2<f32>(_e360, (1f - _e360)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e363.x, _e363.y), _e363.z)), max(max(_e363.x, _e363.y), _e363.z))));
                        local_1 = ((_e363 * min(1f, min(_e376.x, _e376.y))) + vec3(_e360));
                    }
                    break;
                }
                case 15: {
                    if Bj {
                        let _e332 = local_2;
                        let _e333 = clamp(_e332, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e333;
                        let _e334 = dot(_e333, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e337 = (_e330 - vec3(dot(_e330, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e350 = (vec2<f32>(_e334, (1f - _e334)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e337.x, _e337.y), _e337.z)), max(max(_e337.x, _e337.y), _e337.z))));
                        local_1 = ((_e337 * min(1f, min(_e350.x, _e350.y))) + vec3(_e334));
                    }
                    break;
                }
                default: {
                }
            }
            let _e616 = local_1;
            let _e618 = mix(_e323, _e616, vec3(_e322.w));
            let _e624 = vec4<f32>(_e618.x, _e315.y, _e315.z, _e315.w);
            let _e630 = vec4<f32>(_e624.x, _e618.y, _e624.z, _e624.w);
            phi_4472_ = vec4<f32>(_e630.x, _e630.y, _e618.z, _e630.w);
        }
        let _e638 = phi_4472_;
        let _e641 = (_e638.xyz * _e638.w);
        let _e647 = vec4<f32>(_e641.x, _e638.y, _e638.z, _e638.w);
        let _e653 = vec4<f32>(_e647.x, _e641.y, _e647.z, _e647.w);
        phi_4485_ = vec4<f32>(_e653.x, _e653.y, _e641.z, _e653.w);
    }
    let _e661 = phi_4485_;
    let _e662 = (_e661 * _e221);
    let _e663 = _e662.xyz;
    let _e666 = j.K3_;
    let _e668 = j.L3_;
    if (Cj && (_e662.w != 0f)) {
        phi_4512_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e87.x) + (0.00583715f * _e87.y))))) * _e666) + _e668)) + _e663);
    } else {
        phi_4512_ = _e663;
    }
    let _e684 = phi_4512_;
    let _e690 = vec4<f32>(_e684.x, _e662.y, _e662.z, _e662.w);
    let _e696 = vec4<f32>(_e690.x, _e684.y, _e690.z, _e690.w);
    let _e702 = vec4<f32>(_e696.x, _e696.y, _e684.z, _e696.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e684.x + _e684.y) + _e684.z) + _e662.w) == 0f) {
                break;
            }
            phi_4514_ = _e702;
            if (_e662.w != 1f) {
                let _e711 = p0_.z2_[_e122];
                phi_4514_ = (_e702 + (unpack4x8unorm(_e711) * (1f - _e662.w)));
            }
            let _e717 = phi_4514_;
            p0_.z2_[_e122] = pack4x8unorm(_e717);
            break;
        }
    }
    if (_e313 != 0u) {
        n0_.z2_[_e122] = _e313;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) J0_: u32, @location(0) W2_: vec2<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    J0_1 = J0_;
    W2_1 = W2_;
    main_1();
}
