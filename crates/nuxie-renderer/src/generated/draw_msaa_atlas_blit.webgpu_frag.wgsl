struct CC {
    ec: f32,
    od: f32,
    ff: f32,
    gf: f32,
    m6_: u32,
    Fg: u32,
    Re: u32,
    Se: u32,
    R7_: vec4<i32>,
    Bg: vec2<f32>,
    pd: vec2<f32>,
    a2_: u32,
    Gg: f32,
    Z5_: u32,
    P2_: f32,
    qd: f32,
    Me: u32,
    z3_: f32,
    A3_: f32,
    rd: f32,
    yg: u32,
}

@id(7) override fh: bool = true;
@id(6) override eh: bool = true;
@id(2) override ah: bool = true;

@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(3) @binding(8)
var Mb: sampler;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;
@group(0) @binding(10)
var BD: texture_2d<f32>;
@group(3) @binding(10)
var Q9_: sampler;
var<private> C2_1: vec2<f32>;
var<private> f1_1: vec4<f32>;
var<private> e2_1: f32;
@group(0) @binding(12)
var SD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> n: CC;
var<private> Jg: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> I3_1: f32;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2839_: vec4<f32>;
    var phi_2836_: f32;
    var phi_2837_: f32;
    var phi_2841_: vec4<f32>;
    var phi_2832_: f32;
    var phi_2842_: vec4<f32>;
    var phi_2840_: vec4<f32>;
    var phi_2838_: vec4<f32>;
    var phi_2843_: f32;
    var phi_3138_: vec4<f32>;
    var phi_3098_: i32;
    var phi_3241_: vec3<f32>;

    let _e53 = C2_1;
    let _e54 = textureSampleLevel(BD, Q9_, _e53, 0f);
    let _e56 = clamp(_e54.x, 0f, 1f);
    let _e57 = f1_1;
    if (_e57.w >= 0f) {
        if ah {
            phi_2839_ = vec4<f32>(_e57.x, _e57.y, _e57.z, (_e57.w * _e56));
        } else {
            phi_2839_ = (_e57 * _e56);
        }
        let _e69 = phi_2839_;
        phi_2838_ = _e69;
    } else {
        if (_e57.w > -1f) {
            if (_e57.z > 0f) {
                phi_2836_ = _e57.x;
            } else {
                phi_2836_ = length(_e57.xy);
            }
            let _e77 = phi_2836_;
            let _e78 = clamp(_e77, 0f, 1f);
            let _e79 = abs(_e57.z);
            if (_e79 > 1f) {
                phi_2837_ = ((0.9980469f * _e78) + 0.0009765625f);
            } else {
                phi_2837_ = ((0.001953125f * _e78) + _e79);
            }
            let _e86 = phi_2837_;
            let _e89 = textureSampleLevel(KD, Mb, vec2<f32>(_e86, -(_e57.w)), 0f);
            let _e91 = (_e89.w * _e56);
            let _e96 = vec4<f32>(_e89.x, _e89.y, _e89.z, _e91);
            if ah {
                phi_2841_ = _e96;
            } else {
                let _e98 = (_e96.xyz * _e91);
                phi_2841_ = vec4<f32>(_e98.x, _e98.y, _e98.z, _e91);
            }
            let _e104 = phi_2841_;
            phi_2840_ = _e104;
        } else {
            let _e107 = textureSampleLevel(IC, S5_, _e57.xy, (-2f - _e57.w));
            let _e109 = (_e57.z * _e56);
            if ah {
                if (_e107.w != 0f) {
                    phi_2832_ = (1f / _e107.w);
                } else {
                    phi_2832_ = 0f;
                }
                let _e115 = phi_2832_;
                let _e116 = (_e107.xyz * _e115);
                phi_2842_ = vec4<f32>(_e116.x, _e116.y, _e116.z, (_e107.w * _e109));
            } else {
                phi_2842_ = (_e107 * _e109);
            }
            let _e124 = phi_2842_;
            phi_2840_ = _e124;
        }
        let _e126 = phi_2840_;
        phi_2838_ = _e126;
    }
    let _e128 = phi_2838_;
    let _e129 = e2_1;
    let _e131 = gl_FragCoord_1;
    let _e135 = textureLoad(SD, vec2<i32>(floor(_e131.xy)), 0i);
    let _e136 = _e128.xyz;
    local_2 = _e136;
    let _e137 = _e135.xyz;
    if (_e135.w != 0f) {
        phi_2843_ = (1f / _e135.w);
    } else {
        phi_2843_ = 0f;
    }
    let _e142 = phi_2843_;
    let _e143 = (_e137 * _e142);
    local = _e143;
    switch bitcast<i32>(u32(_e129)) {
        case 11: {
            let _e145 = local_2;
            local_1 = (_e145 * _e143);
            break;
        }
        case 1: {
            let _e147 = local_2;
            local_1 = ((_e147 + _e143) - (_e147 * _e143));
            break;
        }
        case 2: {
            let _e151 = local_2;
            let _e152 = (_e151 * _e143);
            local_1 = (select(_e152, (((_e151 + _e143) - _e152) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e143 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e159 = local_2;
            local_1 = min(_e159, _e143);
            break;
        }
        case 4: {
            let _e161 = local_2;
            local_1 = max(_e161, _e143);
            break;
        }
        case 5: {
            let _e164 = clamp(_e137, vec3<f32>(0f, 0f, 0f), _e135.www);
            let _e170 = vec4<f32>(_e164.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e176 = vec4<f32>(_e170.x, _e164.y, _e170.z, _e170.w);
            let _e183 = local_2;
            let _e186 = (clamp((vec3<f32>(1f, 1f, 1f) - _e183), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e135.w);
            let _e187 = vec4<f32>(_e176.x, _e176.y, _e164.z, _e176.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e187 / _e186)), sign(_e187), (_e186 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e193 = local_2;
            local_2 = clamp(_e193, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e196 = clamp(_e137, vec3<f32>(0f, 0f, 0f), _e135.www);
            let _e202 = vec4<f32>(_e196.x, _e135.y, _e135.z, _e135.w);
            let _e208 = vec4<f32>(_e202.x, _e196.y, _e202.z, _e202.w);
            phi_3138_ = vec4<f32>(_e208.x, _e208.y, _e196.z, _e208.w);
            if (_e135.w == 0f) {
                phi_3138_ = vec4<f32>(_e196.x, _e196.y, _e196.z, 1f);
            }
            let _e218 = phi_3138_;
            let _e222 = (vec3(_e218.w) - _e218.xyz);
            let _e223 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e222 / (_e223 * _e218.w))), sign(_e222), (_e223 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e231 = local_2;
            let _e232 = (_e231 * _e143);
            local_1 = (select(_e232, (((_e231 + _e143) - _e232) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e231 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3098_ = 0i;
            loop {
                let _e240 = phi_3098_;
                if (_e240 < 3i) {
                    let _e243 = local_2[_e240];
                    if (_e243 <= 0.5f) {
                        let _e246 = local[_e240];
                        local_1[_e240] = (1f - _e246);
                    } else {
                        let _e250 = local[_e240];
                        if (_e250 <= 0.25f) {
                            let _e252 = local[_e240];
                            let _e255 = local[_e240];
                            local_1[_e240] = ((((16f * _e252) - 12f) * _e255) + 3f);
                        } else {
                            let _e259 = local[_e240];
                            local_1[_e240] = (inverseSqrt(_e259) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3098_ = (_e240 + 1i);
                }
            }
            let _e264 = local_2;
            let _e268 = local_1;
            local_1 = (_e143 + ((_e143 * ((_e264 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e268));
            break;
        }
        case 9: {
            let _e271 = local_2;
            local_1 = abs((_e143 - _e271));
            break;
        }
        case 10: {
            let _e274 = local_2;
            local_1 = ((_e274 + _e143) - ((_e274 * 2f) * _e143));
            break;
        }
        case 12: {
            if eh {
                let _e279 = local_2;
                let _e280 = clamp(_e279, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e280;
                let _e295 = (_e280 - vec3(min(min(_e280.x, _e280.y), _e280.z)));
                let _e303 = (_e295 * ((max(max(_e143.x, _e143.y), _e143.z) - min(min(_e143.x, _e143.y), _e143.z)) / max(0.000062f, max(max(_e295.x, _e295.y), _e295.z))));
                let _e304 = dot(_e143, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e307 = (_e303 - vec3(dot(_e303, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e320 = (vec2<f32>(_e304, (1f - _e304)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e307.x, _e307.y), _e307.z)), max(max(_e307.x, _e307.y), _e307.z))));
                local_1 = ((_e307 * min(1f, min(_e320.x, _e320.y))) + vec3(_e304));
            }
            break;
        }
        case 13: {
            if eh {
                let _e328 = local_2;
                let _e329 = clamp(_e328, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e329;
                let _e344 = (_e143 - vec3(min(min(_e143.x, _e143.y), _e143.z)));
                let _e352 = (_e344 * ((max(max(_e329.x, _e329.y), _e329.z) - min(min(_e329.x, _e329.y), _e329.z)) / max(0.000062f, max(max(_e344.x, _e344.y), _e344.z))));
                let _e353 = dot(_e143, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e356 = (_e352 - vec3(dot(_e352, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e369 = (vec2<f32>(_e353, (1f - _e353)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e356.x, _e356.y), _e356.z)), max(max(_e356.x, _e356.y), _e356.z))));
                local_1 = ((_e356 * min(1f, min(_e369.x, _e369.y))) + vec3(_e353));
            }
            break;
        }
        case 14: {
            if eh {
                let _e377 = local_2;
                let _e378 = clamp(_e377, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e378;
                let _e379 = dot(_e143, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e382 = (_e378 - vec3(dot(_e378, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e395 = (vec2<f32>(_e379, (1f - _e379)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e382.x, _e382.y), _e382.z)), max(max(_e382.x, _e382.y), _e382.z))));
                local_1 = ((_e382 * min(1f, min(_e395.x, _e395.y))) + vec3(_e379));
            }
            break;
        }
        case 15: {
            if eh {
                let _e403 = local_2;
                let _e404 = clamp(_e403, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e404;
                let _e405 = dot(_e404, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e408 = (_e143 - vec3(dot(_e143, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e421 = (vec2<f32>(_e405, (1f - _e405)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e408.x, _e408.y), _e408.z)), max(max(_e408.x, _e408.y), _e408.z))));
                local_1 = ((_e408 * min(1f, min(_e421.x, _e421.y))) + vec3(_e405));
            }
            break;
        }
        default: {
        }
    }
    let _e429 = local_1;
    let _e431 = mix(_e136, _e429, vec3(_e135.w));
    let _e437 = vec4<f32>(_e431.x, _e128.y, _e128.z, _e128.w);
    let _e443 = vec4<f32>(_e437.x, _e431.y, _e437.z, _e437.w);
    let _e449 = vec4<f32>(_e443.x, _e443.y, _e431.z, _e443.w);
    let _e452 = (_e449.xyz * _e128.w);
    let _e458 = vec4<f32>(_e452.x, _e449.y, _e449.z, _e449.w);
    let _e464 = vec4<f32>(_e458.x, _e452.y, _e458.z, _e458.w);
    let _e470 = vec4<f32>(_e464.x, _e464.y, _e452.z, _e464.w);
    let _e471 = _e470.xyz;
    let _e472 = gl_FragCoord_1;
    let _e474 = n.z3_;
    let _e476 = n.A3_;
    if (fh && (_e128.w != 0f)) {
        phi_3241_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e472.x) + (0.00583715f * _e472.y))))) * _e474) + _e476)) + _e471);
    } else {
        phi_3241_ = _e471;
    }
    let _e492 = phi_3241_;
    let _e498 = vec4<f32>(_e492.x, _e470.y, _e470.z, _e470.w);
    let _e504 = vec4<f32>(_e498.x, _e492.y, _e498.z, _e498.w);
    Jg = vec4<f32>(_e504.x, _e504.y, _e492.z, _e504.w);
    return;
}

@fragment
fn main(@location(1) C2_: vec2<f32>, @location(0) f1_: vec4<f32>, @location(6) @interpolate(flat, either) e2_: f32, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) I3_: f32) -> @location(0) vec4<f32> {
    C2_1 = C2_;
    f1_1 = f1_;
    e2_1 = e2_;
    gl_FragCoord_1 = gl_FragCoord;
    I3_1 = I3_;
    main_1();
    let _e11 = Jg;
    return _e11;
}
