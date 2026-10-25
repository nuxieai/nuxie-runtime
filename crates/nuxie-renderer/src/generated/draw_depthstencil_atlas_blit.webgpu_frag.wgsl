struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
    P8_: f32,
    Q8_: f32,
}

@id(8) override nj: bool = true;
@id(7) override mj: bool = true;
@id(9) override oj: bool = true;
@id(2) override hj: bool = true;
@id(4) override jj: bool = true;

@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var R8_: sampler;
@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(1) @binding(13)
var W4_: sampler;
@group(0) @binding(10)
var JD: texture_2d<f32>;
@group(3) @binding(10)
var Sa: sampler;
var<private> U2_1: vec2<f32>;
var<private> V0_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> P0_1: vec4<f32>;
@group(0) @binding(12)
var MD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Ni: vec4<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
var<private> h4_1: f32;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_3109_: f32;
    var phi_3110_: f32;
    var phi_3111_: vec2<f32>;
    var phi_3128_: vec4<f32>;
    var phi_3127_: vec4<f32>;
    var phi_1187_: bool;
    var phi_1202_: bool;
    var phi_3112_: f32;
    var phi_3123_: vec4<f32>;
    var phi_3130_: vec4<f32>;
    var phi_3131_: vec4<f32>;
    var phi_3132_: f32;
    var phi_3470_: i32;
    var phi_3522_: vec4<f32>;
    var phi_3652_: vec3<f32>;

    let _e62 = U2_1;
    let _e63 = textureSampleLevel(JD, Sa, _e62, 0f);
    let _e66 = Q0_1;
    let _e67 = u32(_e66);
    let _e68 = V0_1;
    let _e69 = P0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e72 = (hj && (_e67 != 0u));
            if (_e69.w >= 0f) {
                phi_3127_ = _e69;
            } else {
                let _e76 = j.P8_;
                let _e78 = j.Q8_;
                let _e81 = abs(_e69.z);
                let _e82 = floor(_e81);
                let _e85 = (((_e81 - _e82) * 8f) + 0.0009765625f);
                let _e87 = ((_e82 * _e76) + _e78);
                let _e88 = floor(_e85);
                switch bitcast<i32>(0u) {
                    default: {
                        if (oj && (_e88 == 4f)) {
                            let _e107 = normalize(vec2<f32>(_e69.x, -(_e69.y)));
                            let _e110 = acos(clamp(_e107.x, -1f, 1f));
                            if (_e107.y >= 0f) {
                                phi_3110_ = _e110;
                            } else {
                                phi_3110_ = -(_e110);
                            }
                            let _e115 = phi_3110_;
                            phi_3111_ = vec2<f32>((_e115 * 0.15915494f), _e87);
                            break;
                        } else {
                            if (_e88 == 2f) {
                                phi_3109_ = _e69.x;
                            } else {
                                phi_3109_ = length(_e69.xy);
                            }
                            let _e97 = phi_3109_;
                            phi_3111_ = vec2<f32>(((clamp(_e97, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e69.z < 0f))) + _e85), _e87);
                            break;
                        }
                    }
                }
                let _e119 = phi_3111_;
                let _e120 = textureSampleLevel(ZC, R8_, _e119, 0f);
                phi_3128_ = _e120;
                if !(_e72) {
                    let _e126 = (_e120.xyz * _e120.w);
                    phi_3128_ = vec4<f32>(_e126.x, _e126.y, _e126.z, (_e120.w * (ceil(_e69.w) * -0.003921569f)));
                }
                let _e133 = phi_3128_;
                phi_3127_ = _e133;
            }
            let _e135 = phi_3127_;
            phi_1187_ = jj;
            if jj {
                phi_1187_ = (_e68.z < 0f);
            }
            let _e139 = phi_1187_;
            if _e139 {
                let _e141 = textureSampleLevel(UB, W4_, _e68.xy, 0f);
                phi_3131_ = _e141;
                break;
            }
            phi_1202_ = jj;
            if jj {
                phi_1202_ = (_e68.z > 0f);
            }
            let _e145 = phi_1202_;
            phi_3130_ = _e135;
            if _e145 {
                let _e149 = textureSampleLevel(UB, W4_, _e68.xy, (_e68.z - 1f));
                phi_3123_ = _e149;
                if _e72 {
                    if (_e149.w != 0f) {
                        phi_3112_ = (1f / _e149.w);
                    } else {
                        phi_3112_ = 0f;
                    }
                    let _e155 = phi_3112_;
                    let _e156 = (_e149.xyz * _e155);
                    phi_3123_ = vec4<f32>(_e156.x, _e156.y, _e156.z, _e149.w);
                }
                let _e162 = phi_3123_;
                phi_3130_ = (_e135 * _e162);
            }
            let _e165 = phi_3130_;
            phi_3131_ = _e165;
            break;
        }
    }
    let _e167 = phi_3131_;
    let _e168 = gl_FragCoord_1;
    let _e172 = textureLoad(MD, vec2<i32>(floor(_e168.xy)), 0i);
    let _e173 = _e167.xyz;
    local_2 = _e173;
    let _e174 = _e172.xyz;
    if (_e172.w != 0f) {
        phi_3132_ = (1f / _e172.w);
    } else {
        phi_3132_ = 0f;
    }
    let _e179 = phi_3132_;
    let _e180 = (_e174 * _e179);
    local = _e180;
    switch bitcast<i32>(_e67) {
        case 11: {
            let _e464 = local_2;
            local_1 = (_e464 * _e180);
            break;
        }
        case 1: {
            let _e460 = local_2;
            local_1 = ((_e460 + _e180) - (_e460 * _e180));
            break;
        }
        case 2: {
            let _e452 = local_2;
            let _e453 = (_e452 * _e180);
            local_1 = (select(_e453, (((_e452 + _e180) - _e453) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e180 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e450 = local_2;
            local_1 = min(_e450, _e180);
            break;
        }
        case 4: {
            let _e448 = local_2;
            local_1 = max(_e448, _e180);
            break;
        }
        case 5: {
            let _e419 = clamp(_e174, vec3<f32>(0f, 0f, 0f), _e172.www);
            let _e425 = vec4<f32>(_e419.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e431 = vec4<f32>(_e425.x, _e419.y, _e425.z, _e425.w);
            let _e438 = local_2;
            let _e441 = (clamp((vec3<f32>(1f, 1f, 1f) - _e438), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e172.w);
            let _e442 = vec4<f32>(_e431.x, _e431.y, _e419.z, _e431.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e442 / _e441)), sign(_e442), (_e441 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e380 = local_2;
            local_2 = clamp(_e380, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e383 = clamp(_e174, vec3<f32>(0f, 0f, 0f), _e172.www);
            let _e389 = vec4<f32>(_e383.x, _e172.y, _e172.z, _e172.w);
            let _e395 = vec4<f32>(_e389.x, _e383.y, _e389.z, _e389.w);
            phi_3522_ = vec4<f32>(_e395.x, _e395.y, _e383.z, _e395.w);
            if (_e172.w == 0f) {
                phi_3522_ = vec4<f32>(_e383.x, _e383.y, _e383.z, 1f);
            }
            let _e405 = phi_3522_;
            let _e409 = (vec3(_e405.w) - _e405.xyz);
            let _e410 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e409 / (_e410 * _e405.w))), sign(_e409), (_e410 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e372 = local_2;
            let _e373 = (_e372 * _e180);
            local_1 = (select(_e373, (((_e372 + _e180) - _e373) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e372 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3470_ = 0i;
            loop {
                let _e341 = phi_3470_;
                if (_e341 < 3i) {
                    let _e344 = local_2[_e341];
                    if (_e344 <= 0.5f) {
                        let _e361 = local[_e341];
                        local_1[_e341] = (1f - _e361);
                    } else {
                        let _e347 = local[_e341];
                        if (_e347 <= 0.25f) {
                            let _e353 = local[_e341];
                            let _e356 = local[_e341];
                            local_1[_e341] = ((((16f * _e353) - 12f) * _e356) + 3f);
                        } else {
                            let _e349 = local[_e341];
                            local_1[_e341] = (inverseSqrt(_e349) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3470_ = (_e341 + 1i);
                }
            }
            let _e365 = local_2;
            let _e369 = local_1;
            local_1 = (_e180 + ((_e180 * ((_e365 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e369));
            break;
        }
        case 9: {
            let _e337 = local_2;
            local_1 = abs((_e180 - _e337));
            break;
        }
        case 10: {
            let _e332 = local_2;
            local_1 = ((_e332 + _e180) - ((_e332 * 2f) * _e180));
            break;
        }
        case 12: {
            if mj {
                let _e283 = local_2;
                let _e284 = clamp(_e283, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e284;
                let _e299 = (_e284 - vec3(min(min(_e284.x, _e284.y), _e284.z)));
                let _e307 = (_e299 * ((max(max(_e180.x, _e180.y), _e180.z) - min(min(_e180.x, _e180.y), _e180.z)) / max(0.000062f, max(max(_e299.x, _e299.y), _e299.z))));
                let _e308 = dot(_e180, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e311 = (_e307 - vec3(dot(_e307, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e324 = (vec2<f32>(_e308, (1f - _e308)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e311.x, _e311.y), _e311.z)), max(max(_e311.x, _e311.y), _e311.z))));
                local_1 = ((_e311 * min(1f, min(_e324.x, _e324.y))) + vec3(_e308));
            }
            break;
        }
        case 13: {
            if mj {
                let _e234 = local_2;
                let _e235 = clamp(_e234, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e235;
                let _e250 = (_e180 - vec3(min(min(_e180.x, _e180.y), _e180.z)));
                let _e258 = (_e250 * ((max(max(_e235.x, _e235.y), _e235.z) - min(min(_e235.x, _e235.y), _e235.z)) / max(0.000062f, max(max(_e250.x, _e250.y), _e250.z))));
                let _e259 = dot(_e180, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e262 = (_e258 - vec3(dot(_e258, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e275 = (vec2<f32>(_e259, (1f - _e259)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e262.x, _e262.y), _e262.z)), max(max(_e262.x, _e262.y), _e262.z))));
                local_1 = ((_e262 * min(1f, min(_e275.x, _e275.y))) + vec3(_e259));
            }
            break;
        }
        case 14: {
            if mj {
                let _e208 = local_2;
                let _e209 = clamp(_e208, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e209;
                let _e210 = dot(_e180, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e213 = (_e209 - vec3(dot(_e209, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e226 = (vec2<f32>(_e210, (1f - _e210)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e213.x, _e213.y), _e213.z)), max(max(_e213.x, _e213.y), _e213.z))));
                local_1 = ((_e213 * min(1f, min(_e226.x, _e226.y))) + vec3(_e210));
            }
            break;
        }
        case 15: {
            if mj {
                let _e182 = local_2;
                let _e183 = clamp(_e182, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e183;
                let _e184 = dot(_e183, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e187 = (_e180 - vec3(dot(_e180, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e200 = (vec2<f32>(_e184, (1f - _e184)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e187.x, _e187.y), _e187.z)), max(max(_e187.x, _e187.y), _e187.z))));
                local_1 = ((_e187 * min(1f, min(_e200.x, _e200.y))) + vec3(_e184));
            }
            break;
        }
        default: {
        }
    }
    let _e466 = local_1;
    let _e470 = (mix(_e173, _e466, vec3(_e172.w)) * _e167.w);
    let _e476 = vec4<f32>(_e470.x, _e167.y, _e167.z, _e167.w);
    let _e482 = vec4<f32>(_e476.x, _e470.y, _e476.z, _e476.w);
    let _e489 = (vec4<f32>(_e482.x, _e482.y, _e470.z, _e482.w) * clamp(_e63.x, 0f, 1f));
    let _e490 = _e489.xyz;
    let _e492 = gl_FragCoord_1;
    let _e494 = j.G3_;
    let _e496 = j.H3_;
    if (nj && (_e489.w != 0f)) {
        phi_3652_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e492.x) + (0.00583715f * _e492.y))))) * _e494) + _e496)) + _e490);
    } else {
        phi_3652_ = _e490;
    }
    let _e512 = phi_3652_;
    let _e518 = vec4<f32>(_e512.x, _e489.y, _e489.z, _e489.w);
    let _e524 = vec4<f32>(_e518.x, _e512.y, _e518.z, _e518.w);
    Ni = vec4<f32>(_e524.x, _e524.y, _e512.z, _e524.w);
    return;
}

@fragment
fn main(@location(1) U2_: vec2<f32>, @location(9) V0_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) P0_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) h4_: f32) -> @location(0) vec4<f32> {
    U2_1 = U2_;
    V0_1 = V0_;
    Q0_1 = Q0_;
    P0_1 = P0_;
    gl_FragCoord_1 = gl_FragCoord;
    h4_1 = h4_;
    main_1();
    let _e13 = Ni;
    return _e13;
}
