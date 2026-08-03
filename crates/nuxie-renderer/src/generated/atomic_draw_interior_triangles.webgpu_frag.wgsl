struct He {
    c2_: array<vec2<u32>>,
}

struct h0zd {
    c2_: array<u32>,
}

struct Ie {
    c2_: array<vec4<f32>>,
}

struct j0zd {
    c2_: array<u32>,
}

struct CC {
    cc: f32,
    md: f32,
    df: f32,
    ef: f32,
    m6_: u32,
    Dg: u32,
    Pe: u32,
    Qe: u32,
    R7_: vec4<i32>,
    zg: vec2<f32>,
    nd: vec2<f32>,
    a2_: u32,
    Eg: f32,
    Z5_: u32,
    P2_: f32,
    od: f32,
    Ke: u32,
    z3_: f32,
    A3_: f32,
    pd: f32,
    wg: u32,
}

struct q4zd {
    c2_: array<u32>,
}

@id(7) override dh: bool = true;
@id(6) override ch: bool = true;
@id(4) override ah: bool = true;
@id(0) override Wg: bool = true;
@id(1) override Xg: bool = true;
@id(2) override Yg: bool = true;

@group(0) @binding(3)
var<storage> AD: He;
@group(2) @binding(1)
var<storage, read_write> h0_: h0zd;
@group(0) @binding(4)
var<storage> RB: Ie;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(3) @binding(8)
var Kb: sampler;
@group(2) @binding(0)
var<storage, read_write> j0_: j0zd;
@group(0) @binding(0)
var<uniform> n: CC;
@group(2) @binding(3)
var<storage, read_write> q4_: q4zd;
var<private> B0_1: u32;
var<private> i1_1: f32;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_3455_: u32;
    var phi_1436_: bool;
    var phi_3460_: f32;
    var phi_3459_: f32;
    var phi_3461_: f32;
    var phi_3464_: f32;
    var phi_3463_: f32;
    var phi_1473_: bool;
    var phi_3466_: f32;
    var phi_4156_: u32;
    var phi_3465_: f32;
    var phi_4155_: u32;
    var phi_3489_: vec4<f32>;
    var phi_1592_: bool;
    var phi_3493_: u32;
    var phi_1601_: bool;
    var phi_3508_: f32;
    var phi_3994_: vec4<f32>;
    var phi_3930_: i32;
    var phi_4151_: vec4<f32>;
    var phi_4182_: u32;
    var phi_4176_: vec4<f32>;
    var phi_4177_: vec3<f32>;
    var phi_4179_: vec4<f32>;

    let _e77 = gl_FragCoord_1;
    let _e78 = _e77.xy;
    let _e81 = bitcast<vec2<u32>>(vec2<i32>(floor(_e78)));
    let _e83 = n.m6_;
    let _e112 = bitcast<i32>((((((_e81.y >> bitcast<u32>(5u)) * (((_e83 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e81.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e81.x & 28u) << bitcast<u32>(5u)) + ((_e81.y & 28u) << bitcast<u32>(2i)))) + (((_e81.y & 3u) << bitcast<u32>(2i)) + (_e81.x & 3u))));
    let _e115 = q4_.c2_[_e112];
    let _e117 = (_e115 >> bitcast<u32>(17u));
    let _e118 = B0_1;
    if (_e117 == _e118) {
        phi_3455_ = _e115;
    } else {
        phi_3455_ = ((_e118 << bitcast<u32>(17u)) + 65536u);
    }
    let _e124 = phi_3455_;
    let _e125 = i1_1;
    q4_.c2_[_e112] = (_e124 + bitcast<u32>(i32(round((_e125 * 2048f)))));
    phi_4182_ = 0u;
    phi_4176_ = vec4<f32>(0f, 0f, 0f, 0f);
    if (_e117 != _e118) {
        let _e135 = ((f32((_e115 & 131071u)) * 0.00048828125f) + -32f);
        let _e138 = AD.c2_[_e117];
        phi_3459_ = _e135;
        if ((_e138.x & 768u) != 0u) {
            let _e142 = abs(_e135);
            phi_1436_ = ah;
            if ah {
                phi_1436_ = ((_e138.x & 512u) != 0u);
            }
            let _e146 = phi_1436_;
            phi_3460_ = _e142;
            if _e146 {
                phi_3460_ = (1f - abs(((fract((_e142 * 0.5f)) * 2f) + -1f)));
            }
            let _e154 = phi_3460_;
            phi_3459_ = _e154;
        }
        let _e156 = phi_3459_;
        let _e157 = clamp(_e156, 0f, 1f);
        phi_3463_ = _e157;
        if Wg {
            let _e159 = (_e138.x >> bitcast<u32>(16u));
            phi_3464_ = _e157;
            if (_e159 != 0u) {
                let _e163 = h0_.c2_[_e112];
                if (_e159 == (_e163 >> bitcast<u32>(16i))) {
                    phi_3461_ = min(_e157, unpack2x16float(_e163).x);
                } else {
                    phi_3461_ = 0f;
                }
                let _e171 = phi_3461_;
                phi_3464_ = _e171;
            }
            let _e173 = phi_3464_;
            phi_3463_ = _e173;
        }
        let _e175 = phi_3463_;
        phi_1473_ = Xg;
        if Xg {
            phi_1473_ = ((_e138.x & 1024u) != 0u);
        }
        let _e179 = phi_1473_;
        phi_3466_ = _e175;
        if _e179 {
            let _e180 = (_e117 * 4u);
            let _e184 = RB.c2_[(_e180 + 2u)];
            let _e195 = RB.c2_[(_e180 + 3u)];
            let _e200 = _e195.zw;
            let _e202 = ((abs(((mat2x2<f32>(vec2<f32>(_e184.x, _e184.y), vec2<f32>(_e184.z, _e184.w)) * _e78) + _e195.xy)) * _e200) - _e200);
            phi_3466_ = min(_e175, clamp((min(_e202.x, _e202.y) + 0.5f), 0f, 1f));
        }
        let _e210 = phi_3466_;
        let _e211 = (_e138.x & 15u);
        if (_e211 <= 1u) {
            let _e216 = (Wg && (_e211 == 0u));
            phi_4156_ = 0u;
            if _e216 {
                phi_4156_ = (_e138.y | pack2x16float(vec2<f32>(_e210, 0f)));
            }
            let _e221 = phi_4156_;
            phi_4155_ = _e221;
            phi_3489_ = select(unpack4x8unorm(_e138.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e216));
        } else {
            let _e224 = (_e117 * 4u);
            let _e227 = RB.c2_[_e224];
            let _e238 = RB.c2_[(_e224 + 1u)];
            let _e241 = ((mat2x2<f32>(vec2<f32>(_e227.x, _e227.y), vec2<f32>(_e227.z, _e227.w)) * _e78) + _e238.xy);
            if (_e211 == 2u) {
                phi_3465_ = _e241.x;
            } else {
                phi_3465_ = length(_e241);
            }
            let _e246 = phi_3465_;
            let _e255 = textureSampleLevel(KD, Kb, vec2<f32>(((clamp(_e246, 0f, 1f) * _e238.z) + _e238.w), bitcast<f32>(_e138.y)), 0f);
            phi_4155_ = 0u;
            phi_3489_ = _e255;
        }
        let _e257 = phi_4155_;
        let _e259 = phi_3489_;
        let _e261 = (_e259.w * _e210);
        let _e266 = vec4<f32>(_e259.x, _e259.y, _e259.z, _e261);
        phi_1592_ = Yg;
        if Yg {
            phi_1592_ = (_e261 != 0f);
        }
        let _e269 = phi_1592_;
        phi_3493_ = u32();
        phi_1601_ = _e269;
        if _e269 {
            let _e272 = ((_e138.x >> bitcast<u32>(4i)) & 15u);
            phi_3493_ = _e272;
            phi_1601_ = (_e272 != 0u);
        }
        let _e275 = phi_3493_;
        let _e277 = phi_1601_;
        phi_4151_ = _e266;
        if _e277 {
            let _e280 = j0_.c2_[_e112];
            let _e281 = unpack4x8unorm(_e280);
            let _e282 = _e266.xyz;
            local_2 = _e282;
            let _e283 = _e281.xyz;
            if (_e281.w != 0f) {
                phi_3508_ = (1f / _e281.w);
            } else {
                phi_3508_ = 0f;
            }
            let _e288 = phi_3508_;
            let _e289 = (_e283 * _e288);
            local = _e289;
            switch bitcast<i32>(_e275) {
                case 11: {
                    let _e291 = local_2;
                    local_1 = (_e291 * _e289);
                    break;
                }
                case 1: {
                    let _e293 = local_2;
                    local_1 = ((_e293 + _e289) - (_e293 * _e289));
                    break;
                }
                case 2: {
                    let _e297 = local_2;
                    let _e298 = (_e297 * _e289);
                    local_1 = (select(_e298, (((_e297 + _e289) - _e298) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e289 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e305 = local_2;
                    local_1 = min(_e305, _e289);
                    break;
                }
                case 4: {
                    let _e307 = local_2;
                    local_1 = max(_e307, _e289);
                    break;
                }
                case 5: {
                    let _e310 = clamp(_e283, vec3<f32>(0f, 0f, 0f), _e281.www);
                    let _e316 = vec4<f32>(_e310.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e322 = vec4<f32>(_e316.x, _e310.y, _e316.z, _e316.w);
                    let _e329 = local_2;
                    let _e332 = (clamp((vec3<f32>(1f, 1f, 1f) - _e329), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e281.w);
                    let _e333 = vec4<f32>(_e322.x, _e322.y, _e310.z, _e322.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e333 / _e332)), sign(_e333), (_e332 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e339 = local_2;
                    local_2 = clamp(_e339, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e342 = clamp(_e283, vec3<f32>(0f, 0f, 0f), _e281.www);
                    let _e348 = vec4<f32>(_e342.x, _e281.y, _e281.z, _e281.w);
                    let _e354 = vec4<f32>(_e348.x, _e342.y, _e348.z, _e348.w);
                    phi_3994_ = vec4<f32>(_e354.x, _e354.y, _e342.z, _e354.w);
                    if (_e281.w == 0f) {
                        phi_3994_ = vec4<f32>(_e342.x, _e342.y, _e342.z, 1f);
                    }
                    let _e364 = phi_3994_;
                    let _e368 = (vec3(_e364.w) - _e364.xyz);
                    let _e369 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e368 / (_e369 * _e364.w))), sign(_e368), (_e369 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e377 = local_2;
                    let _e378 = (_e377 * _e289);
                    local_1 = (select(_e378, (((_e377 + _e289) - _e378) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e377 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_3930_ = 0i;
                    loop {
                        let _e386 = phi_3930_;
                        if (_e386 < 3i) {
                            let _e389 = local_2[_e386];
                            if (_e389 <= 0.5f) {
                                let _e392 = local[_e386];
                                local_1[_e386] = (1f - _e392);
                            } else {
                                let _e396 = local[_e386];
                                if (_e396 <= 0.25f) {
                                    let _e398 = local[_e386];
                                    let _e401 = local[_e386];
                                    local_1[_e386] = ((((16f * _e398) - 12f) * _e401) + 3f);
                                } else {
                                    let _e405 = local[_e386];
                                    local_1[_e386] = (inverseSqrt(_e405) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_3930_ = (_e386 + 1i);
                        }
                    }
                    let _e410 = local_2;
                    let _e414 = local_1;
                    local_1 = (_e289 + ((_e289 * ((_e410 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e414));
                    break;
                }
                case 9: {
                    let _e417 = local_2;
                    local_1 = abs((_e289 - _e417));
                    break;
                }
                case 10: {
                    let _e420 = local_2;
                    local_1 = ((_e420 + _e289) - ((_e420 * 2f) * _e289));
                    break;
                }
                case 12: {
                    if ch {
                        let _e425 = local_2;
                        let _e426 = clamp(_e425, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e426;
                        let _e441 = (_e426 - vec3(min(min(_e426.x, _e426.y), _e426.z)));
                        let _e449 = (_e441 * ((max(max(_e289.x, _e289.y), _e289.z) - min(min(_e289.x, _e289.y), _e289.z)) / max(0.000062f, max(max(_e441.x, _e441.y), _e441.z))));
                        let _e450 = dot(_e289, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e453 = (_e449 - vec3(dot(_e449, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e466 = (vec2<f32>(_e450, (1f - _e450)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e453.x, _e453.y), _e453.z)), max(max(_e453.x, _e453.y), _e453.z))));
                        local_1 = ((_e453 * min(1f, min(_e466.x, _e466.y))) + vec3(_e450));
                    }
                    break;
                }
                case 13: {
                    if ch {
                        let _e474 = local_2;
                        let _e475 = clamp(_e474, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e475;
                        let _e490 = (_e289 - vec3(min(min(_e289.x, _e289.y), _e289.z)));
                        let _e498 = (_e490 * ((max(max(_e475.x, _e475.y), _e475.z) - min(min(_e475.x, _e475.y), _e475.z)) / max(0.000062f, max(max(_e490.x, _e490.y), _e490.z))));
                        let _e499 = dot(_e289, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e502 = (_e498 - vec3(dot(_e498, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e515 = (vec2<f32>(_e499, (1f - _e499)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e502.x, _e502.y), _e502.z)), max(max(_e502.x, _e502.y), _e502.z))));
                        local_1 = ((_e502 * min(1f, min(_e515.x, _e515.y))) + vec3(_e499));
                    }
                    break;
                }
                case 14: {
                    if ch {
                        let _e523 = local_2;
                        let _e524 = clamp(_e523, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e524;
                        let _e525 = dot(_e289, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e528 = (_e524 - vec3(dot(_e524, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e541 = (vec2<f32>(_e525, (1f - _e525)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e528.x, _e528.y), _e528.z)), max(max(_e528.x, _e528.y), _e528.z))));
                        local_1 = ((_e528 * min(1f, min(_e541.x, _e541.y))) + vec3(_e525));
                    }
                    break;
                }
                case 15: {
                    if ch {
                        let _e549 = local_2;
                        let _e550 = clamp(_e549, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e550;
                        let _e551 = dot(_e550, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e554 = (_e289 - vec3(dot(_e289, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e567 = (vec2<f32>(_e551, (1f - _e551)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e554.x, _e554.y), _e554.z)), max(max(_e554.x, _e554.y), _e554.z))));
                        local_1 = ((_e554 * min(1f, min(_e567.x, _e567.y))) + vec3(_e551));
                    }
                    break;
                }
                default: {
                }
            }
            let _e575 = local_1;
            let _e577 = mix(_e282, _e575, vec3(_e281.w));
            phi_4151_ = vec4<f32>(_e577.x, _e577.y, _e577.z, _e261);
        }
        let _e583 = phi_4151_;
        let _e586 = (_e583.xyz * _e583.w);
        let _e592 = vec4<f32>(_e586.x, _e583.y, _e583.z, _e583.w);
        let _e598 = vec4<f32>(_e592.x, _e586.y, _e592.z, _e592.w);
        phi_4182_ = _e257;
        phi_4176_ = vec4<f32>(_e598.x, _e598.y, _e586.z, _e598.w);
    }
    let _e606 = phi_4182_;
    let _e608 = phi_4176_;
    let _e609 = _e608.xyz;
    let _e612 = n.z3_;
    let _e614 = n.A3_;
    if (dh && (_e608.w != 0f)) {
        phi_4177_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e77.x) + (0.00583715f * _e77.y))))) * _e612) + _e614)) + _e609);
    } else {
        phi_4177_ = _e609;
    }
    let _e630 = phi_4177_;
    let _e636 = vec4<f32>(_e630.x, _e608.y, _e608.z, _e608.w);
    let _e642 = vec4<f32>(_e636.x, _e630.y, _e636.z, _e636.w);
    let _e648 = vec4<f32>(_e642.x, _e642.y, _e630.z, _e642.w);
    switch bitcast<i32>(0u) {
        default: {
            if (_e608.w == 0f) {
                break;
            }
            let _e651 = (1f - _e608.w);
            phi_4179_ = _e648;
            if (_e651 != 0f) {
                let _e655 = j0_.c2_[_e112];
                phi_4179_ = (_e648 + (unpack4x8unorm(_e655) * _e651));
            }
            let _e660 = phi_4179_;
            j0_.c2_[_e112] = pack4x8unorm(_e660);
            break;
        }
    }
    if (_e606 != 0u) {
        h0_.c2_[_e112] = _e606;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) B0_: u32, @location(0) @interpolate(flat, either) i1_: f32) {
    gl_FragCoord_1 = gl_FragCoord;
    B0_1 = B0_;
    i1_1 = i1_;
    main_1();
}
