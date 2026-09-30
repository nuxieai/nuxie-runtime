struct AC {
    tc: f32,
    Dd: f32,
    Hf: f32,
    If: f32,
    q6_: u32,
    Qb: u32,
    tf: u32,
    uf: u32,
    X7_: vec4<i32>,
    eh: vec2<f32>,
    Ed: vec2<f32>,
    f2_: u32,
    ih: f32,
    f6_: u32,
    U2_: f32,
    Fd: f32,
    of_: u32,
    F3_: f32,
    G3_: f32,
    Gd: f32,
    bh: u32,
    Pb: u32,
}

@id(7) override Lh: bool = true;
@id(6) override Kh: bool = true;
@id(2) override Gh: bool = true;
@id(8) override Mh: bool = true;

@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var P9_: sampler;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var Y5_: sampler;
@group(0) @binding(10)
var ED: texture_2d<f32>;
@group(3) @binding(10)
var T9_: sampler;
var<private> F2_1: vec2<f32>;
var<private> C2_1: vec3<f32>;
var<private> g1_1: f32;
var<private> X1_1: vec4<f32>;
@group(0) @binding(12)
var XD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: AC;
var<private> lh: vec4<f32>;
@group(3) @binding(9)
var ga: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> O3_1: f32;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2822_: f32;
    var phi_2823_: f32;
    var phi_2837_: vec4<f32>;
    var phi_2836_: vec4<f32>;
    var phi_1027_: bool;
    var phi_2824_: f32;
    var phi_2833_: vec4<f32>;
    var phi_2839_: vec4<f32>;
    var phi_2840_: f32;
    var phi_3135_: vec4<f32>;
    var phi_3095_: i32;
    var phi_3238_: vec3<f32>;

    let _e54 = F2_1;
    let _e55 = textureSampleLevel(ED, T9_, _e54, 0f);
    let _e58 = g1_1;
    let _e59 = u32(_e58);
    let _e60 = C2_1;
    let _e61 = X1_1;
    let _e63 = (Gh && (_e59 != 0u));
    if (_e61.w >= 0f) {
        phi_2836_ = _e61;
    } else {
        if (_e61.z > 0f) {
            phi_2822_ = _e61.x;
        } else {
            phi_2822_ = length(_e61.xy);
        }
        let _e73 = phi_2822_;
        let _e74 = clamp(_e73, 0f, 1f);
        let _e75 = abs(_e61.z);
        if (_e75 > 1f) {
            phi_2823_ = ((0.9980469f * _e74) + 0.0009765625f);
        } else {
            phi_2823_ = ((0.001953125f * _e74) + _e75);
        }
        let _e82 = phi_2823_;
        let _e84 = textureSampleLevel(DD, P9_, vec2<f32>(_e82, -(_e61.w)), 0f);
        phi_2837_ = _e84;
        if !(_e63) {
            let _e88 = (_e84.xyz * _e84.w);
            let _e94 = vec4<f32>(_e88.x, _e84.y, _e84.z, _e84.w);
            let _e100 = vec4<f32>(_e94.x, _e88.y, _e94.z, _e94.w);
            phi_2837_ = vec4<f32>(_e100.x, _e100.y, _e88.z, _e100.w);
        }
        let _e108 = phi_2837_;
        phi_2836_ = _e108;
    }
    let _e110 = phi_2836_;
    phi_1027_ = Mh;
    if Mh {
        phi_1027_ = (_e60.z > 0f);
    }
    let _e114 = phi_1027_;
    phi_2839_ = _e110;
    if _e114 {
        let _e118 = textureSampleLevel(GC, Y5_, _e60.xy, (_e60.z - 1f));
        phi_2833_ = _e118;
        if _e63 {
            if (_e118.w != 0f) {
                phi_2824_ = (1f / _e118.w);
            } else {
                phi_2824_ = 0f;
            }
            let _e124 = phi_2824_;
            let _e125 = (_e118.xyz * _e124);
            phi_2833_ = vec4<f32>(_e125.x, _e125.y, _e125.z, _e118.w);
        }
        let _e131 = phi_2833_;
        phi_2839_ = (_e110 * _e131);
    }
    let _e134 = phi_2839_;
    let _e135 = gl_FragCoord_1;
    let _e139 = textureLoad(XD, vec2<i32>(floor(_e135.xy)), 0i);
    let _e140 = _e134.xyz;
    local_2 = _e140;
    let _e141 = _e139.xyz;
    if (_e139.w != 0f) {
        phi_2840_ = (1f / _e139.w);
    } else {
        phi_2840_ = 0f;
    }
    let _e146 = phi_2840_;
    let _e147 = (_e141 * _e146);
    local = _e147;
    switch bitcast<i32>(_e59) {
        case 11: {
            let _e149 = local_2;
            local_1 = (_e149 * _e147);
            break;
        }
        case 1: {
            let _e151 = local_2;
            local_1 = ((_e151 + _e147) - (_e151 * _e147));
            break;
        }
        case 2: {
            let _e155 = local_2;
            let _e156 = (_e155 * _e147);
            local_1 = (select(_e156, (((_e155 + _e147) - _e156) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e147 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e163 = local_2;
            local_1 = min(_e163, _e147);
            break;
        }
        case 4: {
            let _e165 = local_2;
            local_1 = max(_e165, _e147);
            break;
        }
        case 5: {
            let _e168 = clamp(_e141, vec3<f32>(0f, 0f, 0f), _e139.www);
            let _e174 = vec4<f32>(_e168.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e180 = vec4<f32>(_e174.x, _e168.y, _e174.z, _e174.w);
            let _e187 = local_2;
            let _e190 = (clamp((vec3<f32>(1f, 1f, 1f) - _e187), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e139.w);
            let _e191 = vec4<f32>(_e180.x, _e180.y, _e168.z, _e180.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e191 / _e190)), sign(_e191), (_e190 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e197 = local_2;
            local_2 = clamp(_e197, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e200 = clamp(_e141, vec3<f32>(0f, 0f, 0f), _e139.www);
            let _e206 = vec4<f32>(_e200.x, _e139.y, _e139.z, _e139.w);
            let _e212 = vec4<f32>(_e206.x, _e200.y, _e206.z, _e206.w);
            phi_3135_ = vec4<f32>(_e212.x, _e212.y, _e200.z, _e212.w);
            if (_e139.w == 0f) {
                phi_3135_ = vec4<f32>(_e200.x, _e200.y, _e200.z, 1f);
            }
            let _e222 = phi_3135_;
            let _e226 = (vec3(_e222.w) - _e222.xyz);
            let _e227 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e226 / (_e227 * _e222.w))), sign(_e226), (_e227 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e235 = local_2;
            let _e236 = (_e235 * _e147);
            local_1 = (select(_e236, (((_e235 + _e147) - _e236) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e235 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3095_ = 0i;
            loop {
                let _e244 = phi_3095_;
                if (_e244 < 3i) {
                    let _e247 = local_2[_e244];
                    if (_e247 <= 0.5f) {
                        let _e250 = local[_e244];
                        local_1[_e244] = (1f - _e250);
                    } else {
                        let _e254 = local[_e244];
                        if (_e254 <= 0.25f) {
                            let _e256 = local[_e244];
                            let _e259 = local[_e244];
                            local_1[_e244] = ((((16f * _e256) - 12f) * _e259) + 3f);
                        } else {
                            let _e263 = local[_e244];
                            local_1[_e244] = (inverseSqrt(_e263) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3095_ = (_e244 + 1i);
                }
            }
            let _e268 = local_2;
            let _e272 = local_1;
            local_1 = (_e147 + ((_e147 * ((_e268 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e272));
            break;
        }
        case 9: {
            let _e275 = local_2;
            local_1 = abs((_e147 - _e275));
            break;
        }
        case 10: {
            let _e278 = local_2;
            local_1 = ((_e278 + _e147) - ((_e278 * 2f) * _e147));
            break;
        }
        case 12: {
            if Kh {
                let _e283 = local_2;
                let _e284 = clamp(_e283, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e284;
                let _e299 = (_e284 - vec3(min(min(_e284.x, _e284.y), _e284.z)));
                let _e307 = (_e299 * ((max(max(_e147.x, _e147.y), _e147.z) - min(min(_e147.x, _e147.y), _e147.z)) / max(0.000062f, max(max(_e299.x, _e299.y), _e299.z))));
                let _e308 = dot(_e147, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e311 = (_e307 - vec3(dot(_e307, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e324 = (vec2<f32>(_e308, (1f - _e308)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e311.x, _e311.y), _e311.z)), max(max(_e311.x, _e311.y), _e311.z))));
                local_1 = ((_e311 * min(1f, min(_e324.x, _e324.y))) + vec3(_e308));
            }
            break;
        }
        case 13: {
            if Kh {
                let _e332 = local_2;
                let _e333 = clamp(_e332, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e333;
                let _e348 = (_e147 - vec3(min(min(_e147.x, _e147.y), _e147.z)));
                let _e356 = (_e348 * ((max(max(_e333.x, _e333.y), _e333.z) - min(min(_e333.x, _e333.y), _e333.z)) / max(0.000062f, max(max(_e348.x, _e348.y), _e348.z))));
                let _e357 = dot(_e147, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e360 = (_e356 - vec3(dot(_e356, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e373 = (vec2<f32>(_e357, (1f - _e357)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e360.x, _e360.y), _e360.z)), max(max(_e360.x, _e360.y), _e360.z))));
                local_1 = ((_e360 * min(1f, min(_e373.x, _e373.y))) + vec3(_e357));
            }
            break;
        }
        case 14: {
            if Kh {
                let _e381 = local_2;
                let _e382 = clamp(_e381, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e382;
                let _e383 = dot(_e147, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e386 = (_e382 - vec3(dot(_e382, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e399 = (vec2<f32>(_e383, (1f - _e383)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e386.x, _e386.y), _e386.z)), max(max(_e386.x, _e386.y), _e386.z))));
                local_1 = ((_e386 * min(1f, min(_e399.x, _e399.y))) + vec3(_e383));
            }
            break;
        }
        case 15: {
            if Kh {
                let _e407 = local_2;
                let _e408 = clamp(_e407, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e408;
                let _e409 = dot(_e408, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e412 = (_e147 - vec3(dot(_e147, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e425 = (vec2<f32>(_e409, (1f - _e409)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e412.x, _e412.y), _e412.z)), max(max(_e412.x, _e412.y), _e412.z))));
                local_1 = ((_e412 * min(1f, min(_e425.x, _e425.y))) + vec3(_e409));
            }
            break;
        }
        default: {
        }
    }
    let _e433 = local_1;
    let _e437 = (mix(_e140, _e433, vec3(_e139.w)) * _e134.w);
    let _e443 = vec4<f32>(_e437.x, _e134.y, _e134.z, _e134.w);
    let _e449 = vec4<f32>(_e443.x, _e437.y, _e443.z, _e443.w);
    let _e456 = (vec4<f32>(_e449.x, _e449.y, _e437.z, _e449.w) * clamp(_e55.x, 0f, 1f));
    let _e457 = _e456.xyz;
    let _e459 = gl_FragCoord_1;
    let _e461 = j.F3_;
    let _e463 = j.G3_;
    if (Lh && (_e456.w != 0f)) {
        phi_3238_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e459.x) + (0.00583715f * _e459.y))))) * _e461) + _e463)) + _e457);
    } else {
        phi_3238_ = _e457;
    }
    let _e479 = phi_3238_;
    let _e485 = vec4<f32>(_e479.x, _e456.y, _e456.z, _e456.w);
    let _e491 = vec4<f32>(_e485.x, _e479.y, _e485.z, _e485.w);
    lh = vec4<f32>(_e491.x, _e491.y, _e479.z, _e491.w);
    return;
}

@fragment
fn main(@location(1) F2_: vec2<f32>, @location(9) C2_: vec3<f32>, @location(6) @interpolate(flat, either) g1_: f32, @location(0) X1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) O3_: f32) -> @location(0) vec4<f32> {
    F2_1 = F2_;
    C2_1 = C2_;
    g1_1 = g1_;
    X1_1 = X1_;
    gl_FragCoord_1 = gl_FragCoord;
    O3_1 = O3_;
    main_1();
    let _e13 = lh;
    return _e13;
}
