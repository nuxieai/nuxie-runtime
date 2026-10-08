struct VB {
    vd: f32,
    Ce: f32,
    Gg: f32,
    Hg: f32,
    L6_: u32,
    xa: u32,
    sg: u32,
    tg: u32,
    C8_: vec4<i32>,
    Bi: vec2<f32>,
    De: vec2<f32>,
    r2_: u32,
    Fi: f32,
    p6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    E3_: f32,
    F3_: f32,
    Fe: f32,
    yi: u32,
    wa: u32,
    cd: f32,
    g7_: f32,
    Db: f32,
}

@id(7) override fj: bool = true;
@id(6) override ej: bool = true;
@id(15) override nj: bool = false;
@id(16) override oj: bool = false;
@id(2) override aj: bool = true;
@id(8) override gj: bool = true;

var<private> Q0_1: f32;
var<private> P0_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var I8_: sampler;
var<private> V0_1: vec3<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var S4_: sampler;
@group(0) @binding(12)
var KD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Ii: vec4<f32>;
@group(3) @binding(9)
var Va: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2839_: f32;
    var phi_2835_: f32;
    var phi_2837_: vec2<f32>;
    var phi_2840_: bool;
    var phi_2842_: vec4<f32>;
    var phi_2858_: vec4<f32>;
    var phi_2857_: vec4<f32>;
    var phi_906_: bool;
    var phi_2843_: f32;
    var phi_2854_: vec4<f32>;
    var phi_2860_: vec4<f32>;
    var phi_2861_: f32;
    var phi_3219_: vec4<f32>;
    var phi_3171_: i32;
    var phi_3340_: vec3<f32>;

    let _e64 = Q0_1;
    let _e65 = u32(_e64);
    let _e67 = (aj && (_e65 != 0u));
    let _e69 = P0_1[3u];
    if (_e69 >= 0f) {
        let _e71 = P0_1;
        phi_2857_ = _e71;
    } else {
        let _e72 = P0_1;
        let _e74 = j.Db;
        let _e76 = j.g7_;
        let _e78 = bitcast<u32>(_e72.w);
        let _e82 = ((f32((_e78 & 268304384u)) * _e74) - _e76);
        let _e84 = abs(_e72.z);
        if (_e84 < 1.5f) {
            phi_2839_ = _e84;
            phi_2835_ = _e72.x;
        } else {
            phi_2839_ = (_e84 - 2f);
            phi_2835_ = length(_e72.xy);
        }
        let _e91 = phi_2839_;
        let _e93 = phi_2835_;
        let _e94 = clamp(_e93, 0f, 1f);
        if (_e72.z < 0f) {
            phi_2837_ = vec2<f32>(((_e94 * 0.9980469f) + 0.0009765625f), _e82);
        } else {
            phi_2837_ = vec2<f32>(((_e94 * 0.001953125f) + ((f32((_e78 & 130816u)) * 0.0000076293945f) + 0.0009765625f)), _e82);
        }
        let _e107 = phi_2837_;
        let _e108 = textureSampleLevel(YC, I8_, _e107, 0f);
        switch bitcast<i32>(0u) {
            default: {
                if nj {
                    phi_2840_ = true;
                    break;
                }
                if oj {
                    phi_2840_ = true;
                    break;
                }
                phi_2840_ = false;
                break;
            }
        }
        let _e111 = phi_2840_;
        phi_2842_ = _e108;
        if _e111 {
            phi_2842_ = vec4<f32>(_e108.x, _e108.y, _e108.z, (_e108.w * _e91));
        }
        let _e120 = phi_2842_;
        phi_2858_ = _e120;
        if !(_e67) {
            let _e124 = (_e120.xyz * _e120.w);
            phi_2858_ = vec4<f32>(_e124.x, _e124.y, _e124.z, (_e120.w * (f32((_e78 & 255u)) * 0.003921569f)));
        }
        let _e134 = phi_2858_;
        phi_2857_ = _e134;
    }
    let _e136 = phi_2857_;
    phi_906_ = gj;
    if gj {
        let _e138 = V0_1[2u];
        phi_906_ = (_e138 > 0f);
    }
    let _e141 = phi_906_;
    phi_2860_ = _e136;
    if _e141 {
        let _e143 = V0_1[2u];
        let _e145 = V0_1;
        let _e147 = textureSampleLevel(TB, S4_, _e145.xy, (_e143 - 1f));
        phi_2854_ = _e147;
        if _e67 {
            if (_e147.w != 0f) {
                phi_2843_ = (1f / _e147.w);
            } else {
                phi_2843_ = 0f;
            }
            let _e153 = phi_2843_;
            let _e154 = (_e147.xyz * _e153);
            phi_2854_ = vec4<f32>(_e154.x, _e154.y, _e154.z, _e147.w);
        }
        let _e160 = phi_2854_;
        phi_2860_ = (_e136 * _e160);
    }
    let _e163 = phi_2860_;
    let _e164 = gl_FragCoord_1;
    let _e168 = textureLoad(KD, vec2<i32>(floor(_e164.xy)), 0i);
    let _e169 = _e163.xyz;
    local_2 = _e169;
    let _e170 = _e168.xyz;
    if (_e168.w != 0f) {
        phi_2861_ = (1f / _e168.w);
    } else {
        phi_2861_ = 0f;
    }
    let _e175 = phi_2861_;
    let _e176 = (_e170 * _e175);
    local = _e176;
    switch bitcast<i32>(_e65) {
        case 11: {
            let _e178 = local_2;
            local_1 = (_e178 * _e176);
            break;
        }
        case 1: {
            let _e180 = local_2;
            local_1 = ((_e180 + _e176) - (_e180 * _e176));
            break;
        }
        case 2: {
            let _e184 = local_2;
            let _e185 = (_e184 * _e176);
            local_1 = (select(_e185, (((_e184 + _e176) - _e185) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e176 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e192 = local_2;
            local_1 = min(_e192, _e176);
            break;
        }
        case 4: {
            let _e194 = local_2;
            local_1 = max(_e194, _e176);
            break;
        }
        case 5: {
            let _e197 = clamp(_e170, vec3<f32>(0f, 0f, 0f), _e168.www);
            let _e203 = vec4<f32>(_e197.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e209 = vec4<f32>(_e203.x, _e197.y, _e203.z, _e203.w);
            let _e216 = local_2;
            let _e219 = (clamp((vec3<f32>(1f, 1f, 1f) - _e216), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e168.w);
            let _e220 = vec4<f32>(_e209.x, _e209.y, _e197.z, _e209.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e220 / _e219)), sign(_e220), (_e219 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e226 = local_2;
            local_2 = clamp(_e226, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e229 = clamp(_e170, vec3<f32>(0f, 0f, 0f), _e168.www);
            let _e235 = vec4<f32>(_e229.x, _e168.y, _e168.z, _e168.w);
            let _e241 = vec4<f32>(_e235.x, _e229.y, _e235.z, _e235.w);
            phi_3219_ = vec4<f32>(_e241.x, _e241.y, _e229.z, _e241.w);
            if (_e168.w == 0f) {
                phi_3219_ = vec4<f32>(_e229.x, _e229.y, _e229.z, 1f);
            }
            let _e251 = phi_3219_;
            let _e255 = (vec3(_e251.w) - _e251.xyz);
            let _e256 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e255 / (_e256 * _e251.w))), sign(_e255), (_e256 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e264 = local_2;
            let _e265 = (_e264 * _e176);
            local_1 = (select(_e265, (((_e264 + _e176) - _e265) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e264 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3171_ = 0i;
            loop {
                let _e273 = phi_3171_;
                if (_e273 < 3i) {
                    let _e276 = local_2[_e273];
                    if (_e276 <= 0.5f) {
                        let _e279 = local[_e273];
                        local_1[_e273] = (1f - _e279);
                    } else {
                        let _e283 = local[_e273];
                        if (_e283 <= 0.25f) {
                            let _e285 = local[_e273];
                            let _e288 = local[_e273];
                            local_1[_e273] = ((((16f * _e285) - 12f) * _e288) + 3f);
                        } else {
                            let _e292 = local[_e273];
                            local_1[_e273] = (inverseSqrt(_e292) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3171_ = (_e273 + 1i);
                }
            }
            let _e297 = local_2;
            let _e301 = local_1;
            local_1 = (_e176 + ((_e176 * ((_e297 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e301));
            break;
        }
        case 9: {
            let _e304 = local_2;
            local_1 = abs((_e176 - _e304));
            break;
        }
        case 10: {
            let _e307 = local_2;
            local_1 = ((_e307 + _e176) - ((_e307 * 2f) * _e176));
            break;
        }
        case 12: {
            if ej {
                let _e312 = local_2;
                let _e313 = clamp(_e312, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e313;
                let _e328 = (_e313 - vec3(min(min(_e313.x, _e313.y), _e313.z)));
                let _e336 = (_e328 * ((max(max(_e176.x, _e176.y), _e176.z) - min(min(_e176.x, _e176.y), _e176.z)) / max(0.000062f, max(max(_e328.x, _e328.y), _e328.z))));
                let _e337 = dot(_e176, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e340 = (_e336 - vec3(dot(_e336, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e353 = (vec2<f32>(_e337, (1f - _e337)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e340.x, _e340.y), _e340.z)), max(max(_e340.x, _e340.y), _e340.z))));
                local_1 = ((_e340 * min(1f, min(_e353.x, _e353.y))) + vec3(_e337));
            }
            break;
        }
        case 13: {
            if ej {
                let _e361 = local_2;
                let _e362 = clamp(_e361, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e362;
                let _e377 = (_e176 - vec3(min(min(_e176.x, _e176.y), _e176.z)));
                let _e385 = (_e377 * ((max(max(_e362.x, _e362.y), _e362.z) - min(min(_e362.x, _e362.y), _e362.z)) / max(0.000062f, max(max(_e377.x, _e377.y), _e377.z))));
                let _e386 = dot(_e176, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e389 = (_e385 - vec3(dot(_e385, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e402 = (vec2<f32>(_e386, (1f - _e386)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e389.x, _e389.y), _e389.z)), max(max(_e389.x, _e389.y), _e389.z))));
                local_1 = ((_e389 * min(1f, min(_e402.x, _e402.y))) + vec3(_e386));
            }
            break;
        }
        case 14: {
            if ej {
                let _e410 = local_2;
                let _e411 = clamp(_e410, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e411;
                let _e412 = dot(_e176, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e415 = (_e411 - vec3(dot(_e411, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e428 = (vec2<f32>(_e412, (1f - _e412)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e415.x, _e415.y), _e415.z)), max(max(_e415.x, _e415.y), _e415.z))));
                local_1 = ((_e415 * min(1f, min(_e428.x, _e428.y))) + vec3(_e412));
            }
            break;
        }
        case 15: {
            if ej {
                let _e436 = local_2;
                let _e437 = clamp(_e436, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e437;
                let _e438 = dot(_e437, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e441 = (_e176 - vec3(dot(_e176, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e454 = (vec2<f32>(_e438, (1f - _e438)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e441.x, _e441.y), _e441.z)), max(max(_e441.x, _e441.y), _e441.z))));
                local_1 = ((_e441 * min(1f, min(_e454.x, _e454.y))) + vec3(_e438));
            }
            break;
        }
        default: {
        }
    }
    let _e462 = local_1;
    let _e464 = mix(_e169, _e462, vec3(_e168.w));
    let _e470 = vec4<f32>(_e464.x, _e163.y, _e163.z, _e163.w);
    let _e476 = vec4<f32>(_e470.x, _e464.y, _e470.z, _e470.w);
    let _e482 = vec4<f32>(_e476.x, _e476.y, _e464.z, _e476.w);
    let _e485 = (_e482.xyz * _e163.w);
    let _e491 = vec4<f32>(_e485.x, _e482.y, _e482.z, _e482.w);
    let _e497 = vec4<f32>(_e491.x, _e485.y, _e491.z, _e491.w);
    let _e503 = vec4<f32>(_e497.x, _e497.y, _e485.z, _e497.w);
    let _e504 = _e503.xyz;
    let _e505 = gl_FragCoord_1;
    let _e507 = j.E3_;
    let _e509 = j.F3_;
    if (fj && (_e163.w != 0f)) {
        phi_3340_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e505.x) + (0.00583715f * _e505.y))))) * _e507) + _e509)) + _e504);
    } else {
        phi_3340_ = _e504;
    }
    let _e525 = phi_3340_;
    let _e531 = vec4<f32>(_e525.x, _e503.y, _e503.z, _e503.w);
    let _e537 = vec4<f32>(_e531.x, _e525.y, _e531.z, _e531.w);
    Ii = vec4<f32>(_e537.x, _e537.y, _e525.z, _e537.w);
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) Q0_: f32, @location(0) P0_: vec4<f32>, @location(2) V0_: vec3<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    Q0_1 = Q0_;
    P0_1 = P0_;
    V0_1 = V0_;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e9 = Ii;
    return _e9;
}
