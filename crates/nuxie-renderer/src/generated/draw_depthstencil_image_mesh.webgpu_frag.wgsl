struct UB {
    Rc: f32,
    Ud: f32,
    dg: f32,
    eg: f32,
    A6_: u32,
    Y9_: u32,
    Pf: u32,
    Qf: u32,
    i8_: vec4<i32>,
    Nh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Rh: f32,
    U4_: u32,
    a3_: f32,
    Wd: f32,
    Jf: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Kh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

@id(7) override ti: bool = true;
@id(6) override si: bool = true;

@group(1) @binding(11)
var CC: texture_2d<f32>;
@group(1) @binding(13)
var r5_: sampler;
var<private> V5_1: vec2<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> R1_1: vec4<f32>;
var<private> I1_1: u32;
@group(0) @binding(12)
var XD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Uh: vec4<f32>;
var<private> Z3_1: f32;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2462_: f32;
    var phi_2464_: f32;
    var phi_2535_: vec4<f32>;
    var phi_2523_: i32;
    var phi_2575_: vec3<f32>;

    let _e42 = V5_1;
    let _e44 = j.Wd;
    let _e45 = textureSampleBias(CC, r5_, _e42, _e44);
    let _e46 = R1_1;
    let _e47 = (_e45 * _e46);
    let _e50 = (_e47.w != 0f);
    if _e50 {
        phi_2462_ = (1f / _e47.w);
    } else {
        phi_2462_ = 0f;
    }
    let _e53 = phi_2462_;
    let _e54 = (_e47.xyz * _e53);
    let _e60 = vec4<f32>(_e54.x, _e47.y, _e47.z, _e47.w);
    let _e66 = vec4<f32>(_e60.x, _e54.y, _e60.z, _e60.w);
    let _e72 = vec4<f32>(_e66.x, _e66.y, _e54.z, _e66.w);
    let _e73 = I1_1;
    let _e74 = gl_FragCoord_1;
    let _e78 = textureLoad(XD, vec2<i32>(floor(_e74.xy)), 0i);
    let _e79 = _e72.xyz;
    local_2 = _e79;
    let _e80 = _e78.xyz;
    if (_e78.w != 0f) {
        phi_2464_ = (1f / _e78.w);
    } else {
        phi_2464_ = 0f;
    }
    let _e85 = phi_2464_;
    let _e86 = (_e80 * _e85);
    local = _e86;
    switch bitcast<i32>(_e73) {
        case 11: {
            let _e88 = local_2;
            local_1 = (_e88 * _e86);
            break;
        }
        case 1: {
            let _e90 = local_2;
            local_1 = ((_e90 + _e86) - (_e90 * _e86));
            break;
        }
        case 2: {
            let _e94 = local_2;
            let _e95 = (_e94 * _e86);
            local_1 = (select(_e95, (((_e94 + _e86) - _e95) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e86 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e102 = local_2;
            local_1 = min(_e102, _e86);
            break;
        }
        case 4: {
            let _e104 = local_2;
            local_1 = max(_e104, _e86);
            break;
        }
        case 5: {
            let _e107 = clamp(_e80, vec3<f32>(0f, 0f, 0f), _e78.www);
            let _e113 = vec4<f32>(_e107.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e119 = vec4<f32>(_e113.x, _e107.y, _e113.z, _e113.w);
            let _e126 = local_2;
            let _e129 = (clamp((vec3<f32>(1f, 1f, 1f) - _e126), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e78.w);
            let _e130 = vec4<f32>(_e119.x, _e119.y, _e107.z, _e119.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e130 / _e129)), sign(_e130), (_e129 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e136 = local_2;
            local_2 = clamp(_e136, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e139 = clamp(_e80, vec3<f32>(0f, 0f, 0f), _e78.www);
            let _e145 = vec4<f32>(_e139.x, _e78.y, _e78.z, _e78.w);
            let _e151 = vec4<f32>(_e145.x, _e139.y, _e145.z, _e145.w);
            phi_2535_ = vec4<f32>(_e151.x, _e151.y, _e139.z, _e151.w);
            if (_e78.w == 0f) {
                phi_2535_ = vec4<f32>(_e139.x, _e139.y, _e139.z, 1f);
            }
            let _e161 = phi_2535_;
            let _e165 = (vec3(_e161.w) - _e161.xyz);
            let _e166 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e165 / (_e166 * _e161.w))), sign(_e165), (_e166 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e174 = local_2;
            let _e175 = (_e174 * _e86);
            local_1 = (select(_e175, (((_e174 + _e86) - _e175) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e174 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_2523_ = 0i;
            loop {
                let _e183 = phi_2523_;
                if (_e183 < 3i) {
                    let _e186 = local_2[_e183];
                    if (_e186 <= 0.5f) {
                        let _e189 = local[_e183];
                        local_1[_e183] = (1f - _e189);
                    } else {
                        let _e193 = local[_e183];
                        if (_e193 <= 0.25f) {
                            let _e195 = local[_e183];
                            let _e198 = local[_e183];
                            local_1[_e183] = ((((16f * _e195) - 12f) * _e198) + 3f);
                        } else {
                            let _e202 = local[_e183];
                            local_1[_e183] = (inverseSqrt(_e202) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_2523_ = (_e183 + 1i);
                }
            }
            let _e207 = local_2;
            let _e211 = local_1;
            local_1 = (_e86 + ((_e86 * ((_e207 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e211));
            break;
        }
        case 9: {
            let _e214 = local_2;
            local_1 = abs((_e86 - _e214));
            break;
        }
        case 10: {
            let _e217 = local_2;
            local_1 = ((_e217 + _e86) - ((_e217 * 2f) * _e86));
            break;
        }
        case 12: {
            if si {
                let _e222 = local_2;
                let _e223 = clamp(_e222, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e223;
                let _e238 = (_e223 - vec3(min(min(_e223.x, _e223.y), _e223.z)));
                let _e246 = (_e238 * ((max(max(_e86.x, _e86.y), _e86.z) - min(min(_e86.x, _e86.y), _e86.z)) / max(0.000062f, max(max(_e238.x, _e238.y), _e238.z))));
                let _e247 = dot(_e86, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e250 = (_e246 - vec3(dot(_e246, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e263 = (vec2<f32>(_e247, (1f - _e247)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e250.x, _e250.y), _e250.z)), max(max(_e250.x, _e250.y), _e250.z))));
                local_1 = ((_e250 * min(1f, min(_e263.x, _e263.y))) + vec3(_e247));
            }
            break;
        }
        case 13: {
            if si {
                let _e271 = local_2;
                let _e272 = clamp(_e271, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e272;
                let _e287 = (_e86 - vec3(min(min(_e86.x, _e86.y), _e86.z)));
                let _e295 = (_e287 * ((max(max(_e272.x, _e272.y), _e272.z) - min(min(_e272.x, _e272.y), _e272.z)) / max(0.000062f, max(max(_e287.x, _e287.y), _e287.z))));
                let _e296 = dot(_e86, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e299 = (_e295 - vec3(dot(_e295, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e312 = (vec2<f32>(_e296, (1f - _e296)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e299.x, _e299.y), _e299.z)), max(max(_e299.x, _e299.y), _e299.z))));
                local_1 = ((_e299 * min(1f, min(_e312.x, _e312.y))) + vec3(_e296));
            }
            break;
        }
        case 14: {
            if si {
                let _e320 = local_2;
                let _e321 = clamp(_e320, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e321;
                let _e322 = dot(_e86, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e325 = (_e321 - vec3(dot(_e321, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e338 = (vec2<f32>(_e322, (1f - _e322)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e325.x, _e325.y), _e325.z)), max(max(_e325.x, _e325.y), _e325.z))));
                local_1 = ((_e325 * min(1f, min(_e338.x, _e338.y))) + vec3(_e322));
            }
            break;
        }
        case 15: {
            if si {
                let _e346 = local_2;
                let _e347 = clamp(_e346, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e347;
                let _e348 = dot(_e347, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e351 = (_e86 - vec3(dot(_e86, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e364 = (vec2<f32>(_e348, (1f - _e348)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e351.x, _e351.y), _e351.z)), max(max(_e351.x, _e351.y), _e351.z))));
                local_1 = ((_e351 * min(1f, min(_e364.x, _e364.y))) + vec3(_e348));
            }
            break;
        }
        default: {
        }
    }
    let _e372 = local_1;
    let _e375 = (mix(_e79, _e372, vec3(_e78.w)) * _e47.w);
    let _e381 = vec4<f32>(_e375.x, _e72.y, _e72.z, _e72.w);
    let _e387 = vec4<f32>(_e381.x, _e375.y, _e381.z, _e381.w);
    let _e393 = vec4<f32>(_e387.x, _e387.y, _e375.z, _e387.w);
    let _e394 = _e393.xyz;
    let _e395 = gl_FragCoord_1;
    let _e397 = j.M3_;
    let _e399 = j.N3_;
    if (ti && _e50) {
        phi_2575_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e395.x) + (0.00583715f * _e395.y))))) * _e397) + _e399)) + _e394);
    } else {
        phi_2575_ = _e394;
    }
    let _e414 = phi_2575_;
    let _e420 = vec4<f32>(_e414.x, _e393.y, _e393.z, _e393.w);
    let _e426 = vec4<f32>(_e420.x, _e414.y, _e420.z, _e420.w);
    Uh = vec4<f32>(_e426.x, _e426.y, _e414.z, _e426.w);
    return;
}

@fragment
fn main(@location(0) V5_: vec2<f32>, @location(3) @interpolate(flat, either) R1_: vec4<f32>, @location(4) @interpolate(flat, either) I1_: u32, @builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) Z3_: f32) -> @location(0) vec4<f32> {
    V5_1 = V5_;
    R1_1 = R1_;
    I1_1 = I1_;
    gl_FragCoord_1 = gl_FragCoord;
    Z3_1 = Z3_;
    main_1();
    let _e11 = Uh;
    return _e11;
}
