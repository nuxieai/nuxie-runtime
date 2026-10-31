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
    var phi_1524_: bool;
    var phi_3656_: f32;
    var phi_3655_: f32;
    var phi_3657_: f32;
    var phi_3660_: f32;
    var phi_3659_: f32;
    var phi_1561_: bool;
    var phi_3678_: f32;
    var phi_3661_: f32;
    var phi_3662_: f32;
    var phi_3663_: vec2<f32>;
    var phi_3675_: vec4<f32>;
    var phi_3673_: vec4<f32>;
    var phi_3683_: f32;
    var phi_4106_: i32;
    var phi_4170_: vec4<f32>;
    var phi_4327_: vec4<f32>;
    var phi_4340_: vec4<f32>;
    var phi_4341_: vec3<f32>;
    var phi_4343_: vec4<f32>;

    let _e81 = gl_FragCoord_1;
    let _e82 = _e81.xy;
    let _e85 = bitcast<vec2<u32>>(vec2<i32>(floor(_e82)));
    let _e87 = j.V5_;
    let _e116 = bitcast<i32>((((((_e85.y >> bitcast<u32>(5u)) * (((_e87 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e85.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e85.x & 28u) << bitcast<u32>(5u)) + ((_e85.y & 28u) << bitcast<u32>(2i)))) + (((_e85.y & 3u) << bitcast<u32>(2i)) + (_e85.x & 3u))));
    let _e119 = d5_.z2_[_e116];
    let _e123 = ((f32((_e119 & 131071u)) * 0.00048828125f) + -32f);
    let _e125 = (_e119 >> bitcast<u32>(17u));
    let _e130 = JB.z2_[((_e125 * 4u) + 1u)];
    phi_3655_ = _e123;
    if ((_e130.z & 768u) != 0u) {
        let _e134 = abs(_e123);
        phi_1524_ = zj;
        if zj {
            phi_1524_ = ((_e130.z & 512u) != 0u);
        }
        let _e138 = phi_1524_;
        phi_3656_ = _e134;
        if _e138 {
            phi_3656_ = (1f - abs(((fract((_e134 * 0.5f)) * 2f) + -1f)));
        }
        let _e146 = phi_3656_;
        phi_3655_ = _e146;
    }
    let _e148 = phi_3655_;
    let _e149 = clamp(_e148, 0f, 1f);
    phi_3659_ = _e149;
    if uj {
        let _e151 = (_e130.z >> bitcast<u32>(16u));
        phi_3660_ = _e149;
        if (_e151 != 0u) {
            let _e155 = n0_.z2_[_e116];
            if (_e151 == (_e155 >> bitcast<u32>(16i))) {
                phi_3657_ = min(_e149, unpack2x16float(_e155).x);
            } else {
                phi_3657_ = 0f;
            }
            let _e163 = phi_3657_;
            phi_3660_ = _e163;
        }
        let _e165 = phi_3660_;
        phi_3659_ = _e165;
    }
    let _e167 = phi_3659_;
    phi_1561_ = vj;
    if vj {
        phi_1561_ = ((_e130.z & 1024u) != 0u);
    }
    let _e171 = phi_1561_;
    phi_3678_ = _e167;
    if _e171 {
        let _e172 = (_e125 * 8u);
        let _e176 = KB.z2_[(_e172 + 2u)];
        let _e187 = KB.z2_[(_e172 + 3u)];
        let _e192 = _e187.zw;
        let _e194 = ((abs(((mat2x2<f32>(vec2<f32>(_e176.x, _e176.y), vec2<f32>(_e176.z, _e176.w)) * _e82) + _e187.xy)) * _e192) - _e192);
        phi_3678_ = min(_e167, clamp((min(_e194.x, _e194.y) + 0.5f), 0f, 1f));
    }
    let _e202 = phi_3678_;
    let _e203 = (_e130.z & 15u);
    let _e206 = ((_e130.z >> bitcast<u32>(4i)) & 15u);
    let _e208 = (wj && (_e206 != 0u));
    if (_e203 <= 1u) {
        phi_3673_ = select(unpack4x8unorm(_e130.w), vec4<f32>(0f, 0f, 0f, 0f), vec4((uj && (_e203 == 0u))));
    } else {
        let _e210 = (_e125 * 8u);
        let _e213 = KB.z2_[_e210];
        let _e224 = KB.z2_[(_e210 + 1u)];
        let _e227 = ((mat2x2<f32>(vec2<f32>(_e213.x, _e213.y), vec2<f32>(_e213.z, _e213.w)) * _e82) + _e224.xy);
        let _e231 = f32(_e203);
        let _e233 = j.T8_;
        let _e235 = j.U8_;
        let _e240 = ((_e224.w * _e233) + _e235);
        switch bitcast<i32>(0u) {
            default: {
                if (Dj && (_e231 == 4f)) {
                    let _e254 = normalize(_e227);
                    let _e257 = acos(clamp(_e254.x, -1f, 1f));
                    if (_e254.y >= 0f) {
                        phi_3662_ = _e257;
                    } else {
                        phi_3662_ = -(_e257);
                    }
                    let _e262 = phi_3662_;
                    phi_3663_ = vec2<f32>((_e262 * 0.15915494f), _e240);
                    break;
                } else {
                    if (_e231 == 2f) {
                        phi_3661_ = _e227.x;
                    } else {
                        phi_3661_ = length(_e227);
                    }
                    let _e248 = phi_3661_;
                    phi_3663_ = vec2<f32>(((clamp(_e248, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e224.z < 0f))) + ((max(0f, _e224.z) * 0.001953125f) + 0.0009765625f)), _e240);
                    break;
                }
            }
        }
        let _e266 = phi_3663_;
        let _e267 = textureSampleLevel(YC, V8_, _e266, 0f);
        phi_3675_ = _e267;
        if !(_e208) {
            let _e271 = (_e267.xyz * _e267.w);
            phi_3675_ = vec4<f32>(_e271.x, _e271.y, _e271.z, (_e267.w * abs(bitcast<f32>(_e130.w))));
        }
        let _e281 = phi_3675_;
        phi_3673_ = _e281;
    }
    let _e289 = phi_3673_;
    phi_4340_ = _e289;
    if _e208 {
        phi_4327_ = _e289;
        if ((_e289.w * _e202) != 0f) {
            let _e295 = p0_.z2_[_e116];
            let _e296 = unpack4x8unorm(_e295);
            let _e297 = _e289.xyz;
            local_2 = _e297;
            let _e298 = _e296.xyz;
            if (_e296.w != 0f) {
                phi_3683_ = (1f / _e296.w);
            } else {
                phi_3683_ = 0f;
            }
            let _e303 = phi_3683_;
            let _e304 = (_e298 * _e303);
            local = _e304;
            switch bitcast<i32>(_e206) {
                case 11: {
                    let _e588 = local_2;
                    local_1 = (_e588 * _e304);
                    break;
                }
                case 1: {
                    let _e584 = local_2;
                    local_1 = ((_e584 + _e304) - (_e584 * _e304));
                    break;
                }
                case 2: {
                    let _e576 = local_2;
                    let _e577 = (_e576 * _e304);
                    local_1 = (select(_e577, (((_e576 + _e304) - _e577) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e304 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e574 = local_2;
                    local_1 = min(_e574, _e304);
                    break;
                }
                case 4: {
                    let _e572 = local_2;
                    local_1 = max(_e572, _e304);
                    break;
                }
                case 5: {
                    let _e543 = clamp(_e298, vec3<f32>(0f, 0f, 0f), _e296.www);
                    let _e549 = vec4<f32>(_e543.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e555 = vec4<f32>(_e549.x, _e543.y, _e549.z, _e549.w);
                    let _e562 = local_2;
                    let _e565 = (clamp((vec3<f32>(1f, 1f, 1f) - _e562), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e296.w);
                    let _e566 = vec4<f32>(_e555.x, _e555.y, _e543.z, _e555.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e566 / _e565)), sign(_e566), (_e565 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e504 = local_2;
                    local_2 = clamp(_e504, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e507 = clamp(_e298, vec3<f32>(0f, 0f, 0f), _e296.www);
                    let _e513 = vec4<f32>(_e507.x, _e296.y, _e296.z, _e296.w);
                    let _e519 = vec4<f32>(_e513.x, _e507.y, _e513.z, _e513.w);
                    phi_4170_ = vec4<f32>(_e519.x, _e519.y, _e507.z, _e519.w);
                    if (_e296.w == 0f) {
                        phi_4170_ = vec4<f32>(_e507.x, _e507.y, _e507.z, 1f);
                    }
                    let _e529 = phi_4170_;
                    let _e533 = (vec3(_e529.w) - _e529.xyz);
                    let _e534 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e533 / (_e534 * _e529.w))), sign(_e533), (_e534 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e496 = local_2;
                    let _e497 = (_e496 * _e304);
                    local_1 = (select(_e497, (((_e496 + _e304) - _e497) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e496 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_4106_ = 0i;
                    loop {
                        let _e465 = phi_4106_;
                        if (_e465 < 3i) {
                            let _e468 = local_2[_e465];
                            if (_e468 <= 0.5f) {
                                let _e485 = local[_e465];
                                local_1[_e465] = (1f - _e485);
                            } else {
                                let _e471 = local[_e465];
                                if (_e471 <= 0.25f) {
                                    let _e477 = local[_e465];
                                    let _e480 = local[_e465];
                                    local_1[_e465] = ((((16f * _e477) - 12f) * _e480) + 3f);
                                } else {
                                    let _e473 = local[_e465];
                                    local_1[_e465] = (inverseSqrt(_e473) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_4106_ = (_e465 + 1i);
                        }
                    }
                    let _e489 = local_2;
                    let _e493 = local_1;
                    local_1 = (_e304 + ((_e304 * ((_e489 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e493));
                    break;
                }
                case 9: {
                    let _e461 = local_2;
                    local_1 = abs((_e304 - _e461));
                    break;
                }
                case 10: {
                    let _e456 = local_2;
                    local_1 = ((_e456 + _e304) - ((_e456 * 2f) * _e304));
                    break;
                }
                case 12: {
                    if Bj {
                        let _e407 = local_2;
                        let _e408 = clamp(_e407, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e408;
                        let _e423 = (_e408 - vec3(min(min(_e408.x, _e408.y), _e408.z)));
                        let _e431 = (_e423 * ((max(max(_e304.x, _e304.y), _e304.z) - min(min(_e304.x, _e304.y), _e304.z)) / max(0.000062f, max(max(_e423.x, _e423.y), _e423.z))));
                        let _e432 = dot(_e304, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e435 = (_e431 - vec3(dot(_e431, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e448 = (vec2<f32>(_e432, (1f - _e432)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e435.x, _e435.y), _e435.z)), max(max(_e435.x, _e435.y), _e435.z))));
                        local_1 = ((_e435 * min(1f, min(_e448.x, _e448.y))) + vec3(_e432));
                    }
                    break;
                }
                case 13: {
                    if Bj {
                        let _e358 = local_2;
                        let _e359 = clamp(_e358, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e359;
                        let _e374 = (_e304 - vec3(min(min(_e304.x, _e304.y), _e304.z)));
                        let _e382 = (_e374 * ((max(max(_e359.x, _e359.y), _e359.z) - min(min(_e359.x, _e359.y), _e359.z)) / max(0.000062f, max(max(_e374.x, _e374.y), _e374.z))));
                        let _e383 = dot(_e304, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e386 = (_e382 - vec3(dot(_e382, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e399 = (vec2<f32>(_e383, (1f - _e383)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e386.x, _e386.y), _e386.z)), max(max(_e386.x, _e386.y), _e386.z))));
                        local_1 = ((_e386 * min(1f, min(_e399.x, _e399.y))) + vec3(_e383));
                    }
                    break;
                }
                case 14: {
                    if Bj {
                        let _e332 = local_2;
                        let _e333 = clamp(_e332, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e333;
                        let _e334 = dot(_e304, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e337 = (_e333 - vec3(dot(_e333, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e350 = (vec2<f32>(_e334, (1f - _e334)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e337.x, _e337.y), _e337.z)), max(max(_e337.x, _e337.y), _e337.z))));
                        local_1 = ((_e337 * min(1f, min(_e350.x, _e350.y))) + vec3(_e334));
                    }
                    break;
                }
                case 15: {
                    if Bj {
                        let _e306 = local_2;
                        let _e307 = clamp(_e306, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e307;
                        let _e308 = dot(_e307, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e311 = (_e304 - vec3(dot(_e304, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e324 = (vec2<f32>(_e308, (1f - _e308)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e311.x, _e311.y), _e311.z)), max(max(_e311.x, _e311.y), _e311.z))));
                        local_1 = ((_e311 * min(1f, min(_e324.x, _e324.y))) + vec3(_e308));
                    }
                    break;
                }
                default: {
                }
            }
            let _e590 = local_1;
            let _e592 = mix(_e297, _e590, vec3(_e296.w));
            let _e598 = vec4<f32>(_e592.x, _e289.y, _e289.z, _e289.w);
            let _e604 = vec4<f32>(_e598.x, _e592.y, _e598.z, _e598.w);
            phi_4327_ = vec4<f32>(_e604.x, _e604.y, _e592.z, _e604.w);
        }
        let _e612 = phi_4327_;
        let _e615 = (_e612.xyz * _e612.w);
        let _e621 = vec4<f32>(_e615.x, _e612.y, _e612.z, _e612.w);
        let _e627 = vec4<f32>(_e621.x, _e615.y, _e621.z, _e621.w);
        phi_4340_ = vec4<f32>(_e627.x, _e627.y, _e615.z, _e627.w);
    }
    let _e635 = phi_4340_;
    let _e636 = (_e635 * _e202);
    let _e637 = _e636.xyz;
    let _e640 = j.K3_;
    let _e642 = j.L3_;
    if (Cj && (_e636.w != 0f)) {
        phi_4341_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e81.x) + (0.00583715f * _e81.y))))) * _e640) + _e642)) + _e637);
    } else {
        phi_4341_ = _e637;
    }
    let _e658 = phi_4341_;
    let _e664 = vec4<f32>(_e658.x, _e636.y, _e636.z, _e636.w);
    let _e670 = vec4<f32>(_e664.x, _e658.y, _e664.z, _e664.w);
    let _e676 = vec4<f32>(_e670.x, _e670.y, _e658.z, _e670.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e658.x + _e658.y) + _e658.z) + _e636.w) == 0f) {
                break;
            }
            phi_4343_ = _e676;
            if (_e636.w != 1f) {
                let _e685 = p0_.z2_[_e116];
                phi_4343_ = (_e676 + (unpack4x8unorm(_e685) * (1f - _e636.w)));
            }
            let _e691 = phi_4343_;
            p0_.z2_[_e116] = pack4x8unorm(_e691);
            break;
        }
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
