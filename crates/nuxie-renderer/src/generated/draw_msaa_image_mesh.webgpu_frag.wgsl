struct CC {
    bc: f32,
    kd: f32,
    bf: f32,
    cf: f32,
    m6_: u32,
    Bg: u32,
    Ne: u32,
    Oe: u32,
    Q7_: vec4<i32>,
    xg: vec2<f32>,
    ld: vec2<f32>,
    a2_: u32,
    Cg: f32,
    Z5_: u32,
    N2_: f32,
    md: f32,
    Ie: u32,
    y3_: f32,
    z3_: f32,
    nd: f32,
    ug: u32,
}

@id(7) override bh: bool = true;
@id(6) override ah: bool = true;

@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;
var<private> E5_1: vec2<f32>;
@group(0) @binding(0)
var<uniform> m: CC;
var<private> H1_1: f32;
var<private> A1_1: u32;
@group(0) @binding(12)
var SD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Fg: vec4<f32>;
var<private> H3_1: f32;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2464_: f32;
    var phi_2466_: f32;
    var phi_2537_: vec4<f32>;
    var phi_2525_: i32;
    var phi_2577_: vec3<f32>;

    let _e42 = E5_1;
    let _e44 = m.md;
    let _e45 = textureSampleBias(IC, S5_, _e42, _e44);
    let _e46 = H1_1;
    let _e47 = (_e45 * _e46);
    if (_e47.w != 0f) {
        phi_2464_ = (1f / _e47.w);
    } else {
        phi_2464_ = 0f;
    }
    let _e53 = phi_2464_;
    let _e54 = (_e47.xyz * _e53);
    let _e60 = vec4<f32>(_e54.x, _e47.y, _e47.z, _e47.w);
    let _e66 = vec4<f32>(_e60.x, _e54.y, _e60.z, _e60.w);
    let _e72 = vec4<f32>(_e66.x, _e66.y, _e54.z, _e66.w);
    let _e73 = A1_1;
    let _e74 = gl_FragCoord_1;
    let _e78 = textureLoad(SD, vec2<i32>(floor(_e74.xy)), 0i);
    let _e79 = _e72.xyz;
    local_2 = _e79;
    let _e80 = _e78.xyz;
    if (_e78.w != 0f) {
        phi_2466_ = (1f / _e78.w);
    } else {
        phi_2466_ = 0f;
    }
    let _e85 = phi_2466_;
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
            phi_2537_ = vec4<f32>(_e151.x, _e151.y, _e139.z, _e151.w);
            if (_e78.w == 0f) {
                phi_2537_ = vec4<f32>(_e139.x, _e139.y, _e139.z, 1f);
            }
            let _e161 = phi_2537_;
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
            phi_2525_ = 0i;
            loop {
                let _e183 = phi_2525_;
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
                    phi_2525_ = (_e183 + 1i);
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
            if ah {
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
            if ah {
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
            if ah {
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
            if ah {
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
    let _e374 = mix(_e79, _e372, vec3(_e78.w));
    let _e380 = vec4<f32>(_e374.x, _e72.y, _e72.z, _e72.w);
    let _e386 = vec4<f32>(_e380.x, _e374.y, _e380.z, _e380.w);
    let _e392 = vec4<f32>(_e386.x, _e386.y, _e374.z, _e386.w);
    let _e394 = (_e392.xyz * _e47.w);
    let _e400 = vec4<f32>(_e394.x, _e392.y, _e392.z, _e392.w);
    let _e406 = vec4<f32>(_e400.x, _e394.y, _e400.z, _e400.w);
    let _e412 = vec4<f32>(_e406.x, _e406.y, _e394.z, _e406.w);
    let _e413 = _e412.xyz;
    let _e414 = gl_FragCoord_1;
    let _e416 = m.y3_;
    let _e418 = m.z3_;
    if bh {
        phi_2577_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e414.x) + (0.00583715f * _e414.y))))) * _e416) + _e418)) + _e413);
    } else {
        phi_2577_ = _e413;
    }
    let _e432 = phi_2577_;
    let _e438 = vec4<f32>(_e432.x, _e412.y, _e412.z, _e412.w);
    let _e444 = vec4<f32>(_e438.x, _e432.y, _e438.z, _e438.w);
    Fg = vec4<f32>(_e444.x, _e444.y, _e432.z, _e444.w);
    return;
}

@fragment
fn main(@location(0) E5_: vec2<f32>, @location(3) @interpolate(flat, either) H1_: f32, @location(4) @interpolate(flat, either) A1_: u32, @builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) H3_: f32) -> @location(0) vec4<f32> {
    E5_1 = E5_;
    H1_1 = H1_;
    A1_1 = A1_;
    gl_FragCoord_1 = gl_FragCoord;
    H3_1 = H3_;
    main_1();
    let _e11 = Fg;
    return _e11;
}
