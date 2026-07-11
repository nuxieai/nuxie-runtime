struct NB {
    Ub: f32,
    dd: f32,
    Xe: f32,
    Ye: f32,
    q5_: u32,
    ug: u32,
    Je: u32,
    Ke: u32,
    U7_: vec4<i32>,
    rg: vec2<f32>,
    ed: vec2<f32>,
    W1_: u32,
    vg: f32,
    Y5_: u32,
    P2_: f32,
    fd: f32,
    Ee: u32,
    y3_: f32,
    z3_: f32,
    gd: f32,
    og: u32,
}

struct LC {
    r9_: vec4<f32>,
    c2_: vec2<f32>,
    x4_: f32,
    ki: f32,
    k2_: vec4<f32>,
    D2_: vec2<f32>,
    V0_: u32,
    n2_: u32,
    Z6_: u32,
}

@id(7) override Tg: bool = true;
@id(6) override Sg: bool = true;

@group(1) @binding(12) 
var AC: texture_2d<f32>;
@group(1) @binding(14) 
var R5_: sampler;
var<private> U0_1: vec2<f32>;
@group(0) @binding(0) 
var<uniform> k: NB;
@group(0) @binding(2) 
var<uniform> A0_: LC;
@group(0) @binding(13) 
var LD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> yg: vec4<f32>;
var<private> I3_1: f32;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2485_: f32;
    var phi_2487_: f32;
    var phi_2558_: vec4<f32>;
    var phi_2546_: i32;
    var phi_2598_: vec3<f32>;

    let _e43 = U0_1;
    let _e45 = k.fd;
    let _e46 = textureSampleBias(AC, R5_, _e43, _e45);
    let _e48 = A0_.x4_;
    let _e49 = (_e46 * _e48);
    if (_e49.w != 0f) {
        phi_2485_ = (1f / _e49.w);
    } else {
        phi_2485_ = 0f;
    }
    let _e55 = phi_2485_;
    let _e56 = (_e49.xyz * _e55);
    let _e62 = vec4<f32>(_e56.x, _e49.y, _e49.z, _e49.w);
    let _e68 = vec4<f32>(_e62.x, _e56.y, _e62.z, _e62.w);
    let _e74 = vec4<f32>(_e68.x, _e68.y, _e56.z, _e68.w);
    let _e76 = A0_.n2_;
    let _e77 = gl_FragCoord_1;
    let _e81 = textureLoad(LD, vec2<i32>(floor(_e77.xy)), 0i);
    let _e82 = _e74.xyz;
    local_2 = _e82;
    let _e83 = _e81.xyz;
    if (_e81.w != 0f) {
        phi_2487_ = (1f / _e81.w);
    } else {
        phi_2487_ = 0f;
    }
    let _e88 = phi_2487_;
    let _e89 = (_e83 * _e88);
    local = _e89;
    switch bitcast<i32>(_e76) {
        case 11: {
            let _e91 = local_2;
            local_1 = (_e91 * _e89);
            break;
        }
        case 1: {
            let _e93 = local_2;
            local_1 = ((_e93 + _e89) - (_e93 * _e89));
            break;
        }
        case 2: {
            let _e97 = local_2;
            let _e98 = (_e97 * _e89);
            local_1 = (select(_e98, (((_e97 + _e89) - _e98) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e89 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e105 = local_2;
            local_1 = min(_e105, _e89);
            break;
        }
        case 4: {
            let _e107 = local_2;
            local_1 = max(_e107, _e89);
            break;
        }
        case 5: {
            let _e110 = clamp(_e83, vec3<f32>(0f, 0f, 0f), _e81.www);
            let _e116 = vec4<f32>(_e110.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e122 = vec4<f32>(_e116.x, _e110.y, _e116.z, _e116.w);
            let _e129 = local_2;
            let _e132 = (clamp((vec3<f32>(1f, 1f, 1f) - _e129), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e81.w);
            let _e133 = vec4<f32>(_e122.x, _e122.y, _e110.z, _e122.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e133 / _e132)), sign(_e133), (_e132 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e139 = local_2;
            local_2 = clamp(_e139, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e142 = clamp(_e83, vec3<f32>(0f, 0f, 0f), _e81.www);
            let _e148 = vec4<f32>(_e142.x, _e81.y, _e81.z, _e81.w);
            let _e154 = vec4<f32>(_e148.x, _e142.y, _e148.z, _e148.w);
            phi_2558_ = vec4<f32>(_e154.x, _e154.y, _e142.z, _e154.w);
            if (_e81.w == 0f) {
                phi_2558_ = vec4<f32>(_e142.x, _e142.y, _e142.z, 1f);
            }
            let _e164 = phi_2558_;
            let _e168 = (vec3(_e164.w) - _e164.xyz);
            let _e169 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e168 / (_e169 * _e164.w))), sign(_e168), (_e169 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e177 = local_2;
            let _e178 = (_e177 * _e89);
            local_1 = (select(_e178, (((_e177 + _e89) - _e178) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e177 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_2546_ = 0i;
            loop {
                let _e186 = phi_2546_;
                if (_e186 < 3i) {
                    let _e189 = local_2[_e186];
                    if (_e189 <= 0.5f) {
                        let _e192 = local[_e186];
                        local_1[_e186] = (1f - _e192);
                    } else {
                        let _e196 = local[_e186];
                        if (_e196 <= 0.25f) {
                            let _e198 = local[_e186];
                            let _e201 = local[_e186];
                            local_1[_e186] = ((((16f * _e198) - 12f) * _e201) + 3f);
                        } else {
                            let _e205 = local[_e186];
                            local_1[_e186] = (inverseSqrt(_e205) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_2546_ = (_e186 + 1i);
                }
            }
            let _e210 = local_2;
            let _e214 = local_1;
            local_1 = (_e89 + ((_e89 * ((_e210 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e214));
            break;
        }
        case 9: {
            let _e217 = local_2;
            local_1 = abs((_e89 - _e217));
            break;
        }
        case 10: {
            let _e220 = local_2;
            local_1 = ((_e220 + _e89) - ((_e220 * 2f) * _e89));
            break;
        }
        case 12: {
            if Sg {
                let _e225 = local_2;
                let _e226 = clamp(_e225, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e226;
                let _e241 = (_e226 - vec3(min(min(_e226.x, _e226.y), _e226.z)));
                let _e249 = (_e241 * ((max(max(_e89.x, _e89.y), _e89.z) - min(min(_e89.x, _e89.y), _e89.z)) / max(0.000062f, max(max(_e241.x, _e241.y), _e241.z))));
                let _e250 = dot(_e89, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e253 = (_e249 - vec3(dot(_e249, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e266 = (vec2<f32>(_e250, (1f - _e250)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e253.x, _e253.y), _e253.z)), max(max(_e253.x, _e253.y), _e253.z))));
                local_1 = ((_e253 * min(1f, min(_e266.x, _e266.y))) + vec3(_e250));
            }
            break;
        }
        case 13: {
            if Sg {
                let _e274 = local_2;
                let _e275 = clamp(_e274, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e275;
                let _e290 = (_e89 - vec3(min(min(_e89.x, _e89.y), _e89.z)));
                let _e298 = (_e290 * ((max(max(_e275.x, _e275.y), _e275.z) - min(min(_e275.x, _e275.y), _e275.z)) / max(0.000062f, max(max(_e290.x, _e290.y), _e290.z))));
                let _e299 = dot(_e89, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e302 = (_e298 - vec3(dot(_e298, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e315 = (vec2<f32>(_e299, (1f - _e299)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e302.x, _e302.y), _e302.z)), max(max(_e302.x, _e302.y), _e302.z))));
                local_1 = ((_e302 * min(1f, min(_e315.x, _e315.y))) + vec3(_e299));
            }
            break;
        }
        case 14: {
            if Sg {
                let _e323 = local_2;
                let _e324 = clamp(_e323, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e324;
                let _e325 = dot(_e89, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e328 = (_e324 - vec3(dot(_e324, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e341 = (vec2<f32>(_e325, (1f - _e325)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e328.x, _e328.y), _e328.z)), max(max(_e328.x, _e328.y), _e328.z))));
                local_1 = ((_e328 * min(1f, min(_e341.x, _e341.y))) + vec3(_e325));
            }
            break;
        }
        case 15: {
            if Sg {
                let _e349 = local_2;
                let _e350 = clamp(_e349, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e350;
                let _e351 = dot(_e350, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e354 = (_e89 - vec3(dot(_e89, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e367 = (vec2<f32>(_e351, (1f - _e351)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e354.x, _e354.y), _e354.z)), max(max(_e354.x, _e354.y), _e354.z))));
                local_1 = ((_e354 * min(1f, min(_e367.x, _e367.y))) + vec3(_e351));
            }
            break;
        }
        default: {
        }
    }
    let _e375 = local_1;
    let _e377 = mix(_e82, _e375, vec3(_e81.w));
    let _e383 = vec4<f32>(_e377.x, _e74.y, _e74.z, _e74.w);
    let _e389 = vec4<f32>(_e383.x, _e377.y, _e383.z, _e383.w);
    let _e395 = vec4<f32>(_e389.x, _e389.y, _e377.z, _e389.w);
    let _e397 = (_e395.xyz * _e49.w);
    let _e403 = vec4<f32>(_e397.x, _e395.y, _e395.z, _e395.w);
    let _e409 = vec4<f32>(_e403.x, _e397.y, _e403.z, _e403.w);
    let _e415 = vec4<f32>(_e409.x, _e409.y, _e397.z, _e409.w);
    let _e416 = _e415.xyz;
    let _e417 = gl_FragCoord_1;
    let _e419 = k.y3_;
    let _e421 = k.z3_;
    if Tg {
        phi_2598_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e417.x) + (0.00583715f * _e417.y))))) * _e419) + _e421)) + _e416);
    } else {
        phi_2598_ = _e416;
    }
    let _e435 = phi_2598_;
    let _e441 = vec4<f32>(_e435.x, _e415.y, _e415.z, _e415.w);
    let _e447 = vec4<f32>(_e441.x, _e435.y, _e441.z, _e441.w);
    yg = vec4<f32>(_e447.x, _e447.y, _e435.z, _e447.w);
    return;
}

@fragment 
fn main(@location(0) U0_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat) I3_: f32) -> @location(0) vec4<f32> {
    U0_1 = U0_;
    gl_FragCoord_1 = gl_FragCoord;
    I3_1 = I3_;
    main_1();
    let _e7 = yg;
    return _e7;
}
