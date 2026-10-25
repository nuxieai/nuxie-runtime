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

struct W4cf_1 {
    v2_: array<atomic<u32>>,
}

@id(8) override rj: bool = true;
@id(7) override qj: bool = true;
@id(9) override sj: bool = true;
@id(5) override oj: bool = true;
@id(0) override jj: bool = true;
@id(1) override kj: bool = true;
@id(2) override lj: bool = true;
@id(3) override mj: bool = true;

@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(9)
var eb: sampler;
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
var<private> T_1: vec4<f32>;
var<private> I0_1: u32;
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf_1;
@group(1) @binding(11)
var RB: texture_2d<f32>;
@group(1) @binding(13)
var v4_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1590_: bool;
    var phi_1603_: bool;
    var phi_4303_: f32;
    var phi_4311_: f32;
    var phi_4319_: f32;
    var phi_4318_: f32;
    var phi_2108_: bool;
    var phi_4322_: f32;
    var phi_4321_: f32;
    var phi_4323_: f32;
    var phi_4326_: f32;
    var phi_4325_: f32;
    var phi_2145_: bool;
    var phi_4330_: f32;
    var phi_4327_: f32;
    var phi_4328_: f32;
    var phi_4329_: vec2<f32>;
    var phi_4362_: vec4<f32>;
    var phi_5274_: u32;
    var phi_5273_: u32;
    var phi_4360_: vec4<f32>;
    var phi_4369_: f32;
    var phi_4959_: i32;
    var phi_5047_: vec4<f32>;
    var phi_5258_: vec4<f32>;
    var phi_5271_: vec4<f32>;
    var phi_5305_: u32;
    var phi_5298_: vec4<f32>;
    var phi_5300_: vec3<f32>;
    var phi_5302_: vec4<f32>;

    let _e100 = gl_FragCoord_1;
    let _e101 = _e100.xy;
    let _e104 = bitcast<vec2<u32>>(vec2<i32>(floor(_e101)));
    let _e106 = j.T6_;
    let _e135 = bitcast<i32>((((((_e104.y >> bitcast<u32>(5u)) * (((_e106 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e104.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e104.x & 28u) << bitcast<u32>(5u)) + ((_e104.y & 28u) << bitcast<u32>(2i)))) + (((_e104.y & 3u) << bitcast<u32>(2i)) + (_e104.x & 3u))));
    phi_1590_ = mj;
    if mj {
        let _e136 = T_1;
        phi_1590_ = (_e136.x < -1.5f);
    }
    let _e140 = phi_1590_;
    if _e140 {
        let _e213 = T_1;
        let _e217 = textureSampleLevel(AD, eb, vec2<f32>((3f + _e213.x), 0f), 0f);
        let _e223 = textureSampleLevel(AD, eb, vec2<f32>((1f - _e213.y), 0f), 0f);
        phi_4318_ = ((1f - _e217.x) - _e223.x);
    } else {
        phi_1603_ = mj;
        if mj {
            let _e141 = T_1;
            phi_1603_ = (_e141.y < -1.5f);
        }
        let _e145 = phi_1603_;
        if _e145 {
            let _e153 = T_1;
            let _e156 = max(_e153.w, 0f);
            if (_e153.z >= 0f) {
                let _e159 = textureSampleLevel(AD, eb, vec2<f32>(_e156, 0f), 0f);
                phi_4303_ = _e159.x;
            } else {
                phi_4303_ = 0f;
            }
            let _e162 = phi_4303_;
            phi_4311_ = _e162;
            if (abs(_e153.z) < 1000f) {
                let _e169 = (-2f - _e153.y);
                let _e171 = ((_e169 - _e156) * 0.5984134f);
                let _e174 = (vec4(_e156) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e171));
                let _e180 = ((_e174 * -(_e153.z)) + vec4(((_e169 * _e153.z) + (abs(_e153.x) - 0.25f))));
                let _e183 = textureSampleLevel(AD, eb, vec2<f32>(_e180.x, 0f), 0f);
                let _e186 = textureSampleLevel(AD, eb, vec2<f32>(_e180.y, 0f), 0f);
                let _e189 = textureSampleLevel(AD, eb, vec2<f32>(_e180.z, 0f), 0f);
                let _e192 = textureSampleLevel(AD, eb, vec2<f32>(_e180.w, 0f), 0f);
                let _e198 = (_e174 * 5.0959306f);
                phi_4311_ = (_e162 + (dot(vec4<f32>(_e183.x, _e186.x, _e189.x, _e192.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e198) * (_e198 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e171));
            }
            let _e207 = phi_4311_;
            phi_4319_ = (_e207 * sign(_e153.x));
        } else {
            let _e147 = T_1[0u];
            let _e149 = T_1[1u];
            phi_4319_ = min(min(_e147, abs(_e149)), 1f);
        }
        let _e212 = phi_4319_;
        phi_4318_ = _e212;
    }
    let _e227 = phi_4318_;
    let _e231 = u32(round(((_e227 * 2048f) + 65536f)));
    let _e232 = I0_1;
    let _e235 = ((_e232 << bitcast<u32>(17u)) | _e231);
    let _e238 = atomicMax((&W4_.v2_[_e135]), _e235);
    let _e240 = (_e238 >> bitcast<u32>(17u));
    if (_e240 == _e232) {
        let _e762 = T_1;
        if (_e762.y < 0f) {
            let _e769 = atomicAdd((&W4_.v2_[_e135]), ((_e231 + (_e238 - max(_e235, _e238))) - 65536u));
        }
        phi_5305_ = 0u;
        phi_5298_ = vec4<f32>(0f, 0f, 0f, 0f);
    } else {
        let _e245 = ((f32((_e238 & 131071u)) * 0.00048828125f) + -32f);
        let _e248 = XC.v2_[_e240];
        phi_4321_ = _e245;
        if ((_e248.x & 768u) != 0u) {
            let _e252 = abs(_e245);
            phi_2108_ = oj;
            if oj {
                phi_2108_ = ((_e248.x & 512u) != 0u);
            }
            let _e256 = phi_2108_;
            phi_4322_ = _e252;
            if _e256 {
                phi_4322_ = (1f - abs(((fract((_e252 * 0.5f)) * 2f) + -1f)));
            }
            let _e264 = phi_4322_;
            phi_4321_ = _e264;
        }
        let _e266 = phi_4321_;
        let _e267 = clamp(_e266, 0f, 1f);
        phi_4325_ = _e267;
        if jj {
            let _e269 = (_e248.x >> bitcast<u32>(16u));
            phi_4326_ = _e267;
            if (_e269 != 0u) {
                let _e273 = m0_.v2_[_e135];
                if (_e269 == (_e273 >> bitcast<u32>(16i))) {
                    phi_4323_ = min(_e267, unpack2x16float(_e273).x);
                } else {
                    phi_4323_ = 0f;
                }
                let _e281 = phi_4323_;
                phi_4326_ = _e281;
            }
            let _e283 = phi_4326_;
            phi_4325_ = _e283;
        }
        let _e285 = phi_4325_;
        phi_2145_ = kj;
        if kj {
            phi_2145_ = ((_e248.x & 1024u) != 0u);
        }
        let _e289 = phi_2145_;
        phi_4330_ = _e285;
        if _e289 {
            let _e290 = (_e240 * 8u);
            let _e294 = JB.v2_[(_e290 + 2u)];
            let _e305 = JB.v2_[(_e290 + 3u)];
            let _e310 = _e305.zw;
            let _e312 = ((abs(((mat2x2<f32>(vec2<f32>(_e294.x, _e294.y), vec2<f32>(_e294.z, _e294.w)) * _e101) + _e305.xy)) * _e310) - _e310);
            phi_4330_ = min(_e285, clamp((min(_e312.x, _e312.y) + 0.5f), 0f, 1f));
        }
        let _e320 = phi_4330_;
        let _e321 = (_e248.x & 15u);
        let _e324 = ((_e248.x >> bitcast<u32>(4i)) & 15u);
        let _e326 = (lj && (_e324 != 0u));
        if (_e321 <= 1u) {
            let _e403 = (jj && (_e321 == 0u));
            phi_5274_ = 0u;
            if _e403 {
                phi_5274_ = (_e248.y | pack2x16float(vec2<f32>(_e320, 0f)));
            }
            let _e408 = phi_5274_;
            phi_5273_ = _e408;
            phi_4360_ = select(unpack4x8unorm(_e248.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e403));
        } else {
            let _e328 = (_e240 * 8u);
            let _e331 = JB.v2_[_e328];
            let _e342 = JB.v2_[(_e328 + 1u)];
            let _e345 = ((mat2x2<f32>(vec2<f32>(_e331.x, _e331.y), vec2<f32>(_e331.z, _e331.w)) * _e101) + _e342.xy);
            let _e349 = f32(_e321);
            let _e351 = j.Q8_;
            let _e353 = j.R8_;
            let _e358 = ((_e342.w * _e351) + _e353);
            switch bitcast<i32>(0u) {
                default: {
                    if (sj && (_e349 == 4f)) {
                        let _e372 = normalize(_e345);
                        let _e375 = acos(clamp(_e372.x, -1f, 1f));
                        if (_e372.y >= 0f) {
                            phi_4328_ = _e375;
                        } else {
                            phi_4328_ = -(_e375);
                        }
                        let _e380 = phi_4328_;
                        phi_4329_ = vec2<f32>((_e380 * 0.15915494f), _e358);
                        break;
                    } else {
                        if (_e349 == 2f) {
                            phi_4327_ = _e345.x;
                        } else {
                            phi_4327_ = length(_e345);
                        }
                        let _e366 = phi_4327_;
                        phi_4329_ = vec2<f32>(((clamp(_e366, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e342.z < 0f))) + ((max(0f, _e342.z) * 0.001953125f) + 0.0009765625f)), _e358);
                        break;
                    }
                }
            }
            let _e384 = phi_4329_;
            let _e385 = textureSampleLevel(ZC, S8_, _e384, 0f);
            phi_4362_ = _e385;
            if !(_e326) {
                let _e389 = (_e385.xyz * _e385.w);
                phi_4362_ = vec4<f32>(_e389.x, _e389.y, _e389.z, (_e385.w * abs(bitcast<f32>(_e248.y))));
            }
            let _e399 = phi_4362_;
            phi_5273_ = 0u;
            phi_4360_ = _e399;
        }
        let _e412 = phi_5273_;
        let _e414 = phi_4360_;
        phi_5271_ = _e414;
        if _e326 {
            phi_5258_ = _e414;
            if ((_e414.w * _e320) != 0f) {
                let _e420 = n0_.v2_[_e135];
                let _e421 = unpack4x8unorm(_e420);
                let _e422 = _e414.xyz;
                local_2 = _e422;
                let _e423 = _e421.xyz;
                if (_e421.w != 0f) {
                    phi_4369_ = (1f / _e421.w);
                } else {
                    phi_4369_ = 0f;
                }
                let _e428 = phi_4369_;
                let _e429 = (_e423 * _e428);
                local = _e429;
                switch bitcast<i32>(_e324) {
                    case 11: {
                        let _e713 = local_2;
                        local_1 = (_e713 * _e429);
                        break;
                    }
                    case 1: {
                        let _e709 = local_2;
                        local_1 = ((_e709 + _e429) - (_e709 * _e429));
                        break;
                    }
                    case 2: {
                        let _e701 = local_2;
                        let _e702 = (_e701 * _e429);
                        local_1 = (select(_e702, (((_e701 + _e429) - _e702) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e429 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 3: {
                        let _e699 = local_2;
                        local_1 = min(_e699, _e429);
                        break;
                    }
                    case 4: {
                        let _e697 = local_2;
                        local_1 = max(_e697, _e429);
                        break;
                    }
                    case 5: {
                        let _e668 = clamp(_e423, vec3<f32>(0f, 0f, 0f), _e421.www);
                        let _e674 = vec4<f32>(_e668.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                        let _e680 = vec4<f32>(_e674.x, _e668.y, _e674.z, _e674.w);
                        let _e687 = local_2;
                        let _e690 = (clamp((vec3<f32>(1f, 1f, 1f) - _e687), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e421.w);
                        let _e691 = vec4<f32>(_e680.x, _e680.y, _e668.z, _e680.w).xyz;
                        local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e691 / _e690)), sign(_e691), (_e690 == vec3<f32>(0f, 0f, 0f)));
                        break;
                    }
                    case 6: {
                        let _e629 = local_2;
                        local_2 = clamp(_e629, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        let _e632 = clamp(_e423, vec3<f32>(0f, 0f, 0f), _e421.www);
                        let _e638 = vec4<f32>(_e632.x, _e421.y, _e421.z, _e421.w);
                        let _e644 = vec4<f32>(_e638.x, _e632.y, _e638.z, _e638.w);
                        phi_5047_ = vec4<f32>(_e644.x, _e644.y, _e632.z, _e644.w);
                        if (_e421.w == 0f) {
                            phi_5047_ = vec4<f32>(_e632.x, _e632.y, _e632.z, 1f);
                        }
                        let _e654 = phi_5047_;
                        let _e658 = (vec3(_e654.w) - _e654.xyz);
                        let _e659 = local_2;
                        local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e658 / (_e659 * _e654.w))), sign(_e658), (_e659 == vec3<f32>(0f, 0f, 0f))));
                        break;
                    }
                    case 7: {
                        let _e621 = local_2;
                        let _e622 = (_e621 * _e429);
                        local_1 = (select(_e622, (((_e621 + _e429) - _e622) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e621 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 8: {
                        phi_4959_ = 0i;
                        loop {
                            let _e590 = phi_4959_;
                            if (_e590 < 3i) {
                                let _e593 = local_2[_e590];
                                if (_e593 <= 0.5f) {
                                    let _e610 = local[_e590];
                                    local_1[_e590] = (1f - _e610);
                                } else {
                                    let _e596 = local[_e590];
                                    if (_e596 <= 0.25f) {
                                        let _e602 = local[_e590];
                                        let _e605 = local[_e590];
                                        local_1[_e590] = ((((16f * _e602) - 12f) * _e605) + 3f);
                                    } else {
                                        let _e598 = local[_e590];
                                        local_1[_e590] = (inverseSqrt(_e598) - 1f);
                                    }
                                }
                                continue;
                            } else {
                                break;
                            }
                            continuing {
                                phi_4959_ = (_e590 + 1i);
                            }
                        }
                        let _e614 = local_2;
                        let _e618 = local_1;
                        local_1 = (_e429 + ((_e429 * ((_e614 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e618));
                        break;
                    }
                    case 9: {
                        let _e586 = local_2;
                        local_1 = abs((_e429 - _e586));
                        break;
                    }
                    case 10: {
                        let _e581 = local_2;
                        local_1 = ((_e581 + _e429) - ((_e581 * 2f) * _e429));
                        break;
                    }
                    case 12: {
                        if qj {
                            let _e532 = local_2;
                            let _e533 = clamp(_e532, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e533;
                            let _e548 = (_e533 - vec3(min(min(_e533.x, _e533.y), _e533.z)));
                            let _e556 = (_e548 * ((max(max(_e429.x, _e429.y), _e429.z) - min(min(_e429.x, _e429.y), _e429.z)) / max(0.000062f, max(max(_e548.x, _e548.y), _e548.z))));
                            let _e557 = dot(_e429, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e560 = (_e556 - vec3(dot(_e556, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e573 = (vec2<f32>(_e557, (1f - _e557)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e560.x, _e560.y), _e560.z)), max(max(_e560.x, _e560.y), _e560.z))));
                            local_1 = ((_e560 * min(1f, min(_e573.x, _e573.y))) + vec3(_e557));
                        }
                        break;
                    }
                    case 13: {
                        if qj {
                            let _e483 = local_2;
                            let _e484 = clamp(_e483, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e484;
                            let _e499 = (_e429 - vec3(min(min(_e429.x, _e429.y), _e429.z)));
                            let _e507 = (_e499 * ((max(max(_e484.x, _e484.y), _e484.z) - min(min(_e484.x, _e484.y), _e484.z)) / max(0.000062f, max(max(_e499.x, _e499.y), _e499.z))));
                            let _e508 = dot(_e429, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e511 = (_e507 - vec3(dot(_e507, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e524 = (vec2<f32>(_e508, (1f - _e508)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e511.x, _e511.y), _e511.z)), max(max(_e511.x, _e511.y), _e511.z))));
                            local_1 = ((_e511 * min(1f, min(_e524.x, _e524.y))) + vec3(_e508));
                        }
                        break;
                    }
                    case 14: {
                        if qj {
                            let _e457 = local_2;
                            let _e458 = clamp(_e457, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e458;
                            let _e459 = dot(_e429, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e462 = (_e458 - vec3(dot(_e458, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e475 = (vec2<f32>(_e459, (1f - _e459)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e462.x, _e462.y), _e462.z)), max(max(_e462.x, _e462.y), _e462.z))));
                            local_1 = ((_e462 * min(1f, min(_e475.x, _e475.y))) + vec3(_e459));
                        }
                        break;
                    }
                    case 15: {
                        if qj {
                            let _e431 = local_2;
                            let _e432 = clamp(_e431, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e432;
                            let _e433 = dot(_e432, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e436 = (_e429 - vec3(dot(_e429, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e449 = (vec2<f32>(_e433, (1f - _e433)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e436.x, _e436.y), _e436.z)), max(max(_e436.x, _e436.y), _e436.z))));
                            local_1 = ((_e436 * min(1f, min(_e449.x, _e449.y))) + vec3(_e433));
                        }
                        break;
                    }
                    default: {
                    }
                }
                let _e715 = local_1;
                let _e717 = mix(_e422, _e715, vec3(_e421.w));
                let _e723 = vec4<f32>(_e717.x, _e414.y, _e414.z, _e414.w);
                let _e729 = vec4<f32>(_e723.x, _e717.y, _e723.z, _e723.w);
                phi_5258_ = vec4<f32>(_e729.x, _e729.y, _e717.z, _e729.w);
            }
            let _e737 = phi_5258_;
            let _e740 = (_e737.xyz * _e737.w);
            let _e746 = vec4<f32>(_e740.x, _e737.y, _e737.z, _e737.w);
            let _e752 = vec4<f32>(_e746.x, _e740.y, _e746.z, _e746.w);
            phi_5271_ = vec4<f32>(_e752.x, _e752.y, _e740.z, _e752.w);
        }
        let _e760 = phi_5271_;
        phi_5305_ = _e412;
        phi_5298_ = (_e760 * _e320);
    }
    let _e771 = phi_5305_;
    let _e773 = phi_5298_;
    let _e774 = _e773.xyz;
    let _e777 = j.H3_;
    let _e779 = j.I3_;
    if (rj && (_e773.w != 0f)) {
        phi_5300_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e100.x) + (0.00583715f * _e100.y))))) * _e777) + _e779)) + _e774);
    } else {
        phi_5300_ = _e774;
    }
    let _e795 = phi_5300_;
    let _e801 = vec4<f32>(_e795.x, _e773.y, _e773.z, _e773.w);
    let _e807 = vec4<f32>(_e801.x, _e795.y, _e801.z, _e801.w);
    let _e813 = vec4<f32>(_e807.x, _e807.y, _e795.z, _e807.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e795.x + _e795.y) + _e795.z) + _e773.w) == 0f) {
                break;
            }
            phi_5302_ = _e813;
            if (_e773.w != 1f) {
                let _e822 = n0_.v2_[_e135];
                phi_5302_ = (_e813 + (unpack4x8unorm(_e822) * (1f - _e773.w)));
            }
            let _e828 = phi_5302_;
            n0_.v2_[_e135] = pack4x8unorm(_e828);
            break;
        }
    }
    if (_e771 != 0u) {
        m0_.v2_[_e135] = _e771;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) T: vec4<f32>, @location(1) @interpolate(flat, either) I0_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    T_1 = T;
    I0_1 = I0_;
    main_1();
}
