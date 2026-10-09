struct VB {
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
}

@id(7) override dj: bool = true;
@id(6) override cj: bool = true;
@id(15) override lj: bool = false;
@id(16) override mj: bool = false;
@id(2) override Yi: bool = true;
@id(8) override ej: bool = true;

var<private> P0_1: f32;
var<private> O0_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var N8_: sampler;
var<private> U0_1: vec3<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;
@group(0) @binding(12)
var KD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Fi: vec4<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2892_: f32;
    var phi_2893_: bool;
    var phi_2895_: vec4<f32>;
    var phi_2910_: vec4<f32>;
    var phi_2909_: vec4<f32>;
    var phi_929_: bool;
    var phi_2896_: f32;
    var phi_2906_: vec4<f32>;
    var phi_2912_: vec4<f32>;
    var phi_2913_: f32;
    var phi_3239_: vec4<f32>;
    var phi_3195_: i32;
    var phi_3351_: vec3<f32>;

    let _e62 = P0_1;
    let _e63 = u32(_e62);
    let _e65 = (Yi && (_e63 != 0u));
    let _e67 = O0_1[3u];
    if (_e67 >= 0f) {
        let _e69 = O0_1;
        phi_2909_ = _e69;
    } else {
        let _e70 = O0_1;
        let _e72 = j.L8_;
        let _e74 = j.M8_;
        let _e77 = abs(_e70.z);
        let _e78 = floor(_e77);
        let _e81 = (((_e77 - _e78) * 8f) + 0.0009765625f);
        if (floor(_e81) == 2f) {
            phi_2892_ = _e70.x;
        } else {
            phi_2892_ = length(_e70.xy);
        }
        let _e90 = phi_2892_;
        let _e96 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e90, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e70.z < 0f))) + _e81), ((_e78 * _e72) + _e74)), 0f);
        switch bitcast<i32>(0u) {
            default: {
                if lj {
                    phi_2893_ = true;
                    break;
                }
                if mj {
                    phi_2893_ = true;
                    break;
                }
                phi_2893_ = false;
                break;
            }
        }
        let _e99 = phi_2893_;
        phi_2895_ = _e96;
        if _e99 {
            phi_2895_ = vec4<f32>(_e96.x, _e96.y, _e96.z, (_e96.w * ((fract(_e70.w) * -2f) + 1.5f)));
        }
        let _e112 = phi_2895_;
        phi_2910_ = _e112;
        if !(_e65) {
            let _e116 = (_e112.xyz * _e112.w);
            phi_2910_ = vec4<f32>(_e116.x, _e116.y, _e116.z, (_e112.w * (ceil(_e70.w) * -0.003921569f)));
        }
        let _e126 = phi_2910_;
        phi_2909_ = _e126;
    }
    let _e128 = phi_2909_;
    phi_929_ = ej;
    if ej {
        let _e130 = U0_1[2u];
        phi_929_ = (_e130 > 0f);
    }
    let _e133 = phi_929_;
    phi_2912_ = _e128;
    if _e133 {
        let _e135 = U0_1[2u];
        let _e137 = U0_1;
        let _e139 = textureSampleLevel(TB, U4_, _e137.xy, (_e135 - 1f));
        phi_2906_ = _e139;
        if _e65 {
            if (_e139.w != 0f) {
                phi_2896_ = (1f / _e139.w);
            } else {
                phi_2896_ = 0f;
            }
            let _e145 = phi_2896_;
            let _e146 = (_e139.xyz * _e145);
            phi_2906_ = vec4<f32>(_e146.x, _e146.y, _e146.z, _e139.w);
        }
        let _e152 = phi_2906_;
        phi_2912_ = (_e128 * _e152);
    }
    let _e155 = phi_2912_;
    let _e156 = gl_FragCoord_1;
    let _e160 = textureLoad(KD, vec2<i32>(floor(_e156.xy)), 0i);
    let _e161 = _e155.xyz;
    local_2 = _e161;
    let _e162 = _e160.xyz;
    if (_e160.w != 0f) {
        phi_2913_ = (1f / _e160.w);
    } else {
        phi_2913_ = 0f;
    }
    let _e167 = phi_2913_;
    let _e168 = (_e162 * _e167);
    local = _e168;
    switch bitcast<i32>(_e63) {
        case 11: {
            let _e170 = local_2;
            local_1 = (_e170 * _e168);
            break;
        }
        case 1: {
            let _e172 = local_2;
            local_1 = ((_e172 + _e168) - (_e172 * _e168));
            break;
        }
        case 2: {
            let _e176 = local_2;
            let _e177 = (_e176 * _e168);
            local_1 = (select(_e177, (((_e176 + _e168) - _e177) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e168 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e184 = local_2;
            local_1 = min(_e184, _e168);
            break;
        }
        case 4: {
            let _e186 = local_2;
            local_1 = max(_e186, _e168);
            break;
        }
        case 5: {
            let _e189 = clamp(_e162, vec3<f32>(0f, 0f, 0f), _e160.www);
            let _e195 = vec4<f32>(_e189.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e201 = vec4<f32>(_e195.x, _e189.y, _e195.z, _e195.w);
            let _e208 = local_2;
            let _e211 = (clamp((vec3<f32>(1f, 1f, 1f) - _e208), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e160.w);
            let _e212 = vec4<f32>(_e201.x, _e201.y, _e189.z, _e201.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e212 / _e211)), sign(_e212), (_e211 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e218 = local_2;
            local_2 = clamp(_e218, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e221 = clamp(_e162, vec3<f32>(0f, 0f, 0f), _e160.www);
            let _e227 = vec4<f32>(_e221.x, _e160.y, _e160.z, _e160.w);
            let _e233 = vec4<f32>(_e227.x, _e221.y, _e227.z, _e227.w);
            phi_3239_ = vec4<f32>(_e233.x, _e233.y, _e221.z, _e233.w);
            if (_e160.w == 0f) {
                phi_3239_ = vec4<f32>(_e221.x, _e221.y, _e221.z, 1f);
            }
            let _e243 = phi_3239_;
            let _e247 = (vec3(_e243.w) - _e243.xyz);
            let _e248 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e247 / (_e248 * _e243.w))), sign(_e247), (_e248 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e256 = local_2;
            let _e257 = (_e256 * _e168);
            local_1 = (select(_e257, (((_e256 + _e168) - _e257) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e256 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3195_ = 0i;
            loop {
                let _e265 = phi_3195_;
                if (_e265 < 3i) {
                    let _e268 = local_2[_e265];
                    if (_e268 <= 0.5f) {
                        let _e271 = local[_e265];
                        local_1[_e265] = (1f - _e271);
                    } else {
                        let _e275 = local[_e265];
                        if (_e275 <= 0.25f) {
                            let _e277 = local[_e265];
                            let _e280 = local[_e265];
                            local_1[_e265] = ((((16f * _e277) - 12f) * _e280) + 3f);
                        } else {
                            let _e284 = local[_e265];
                            local_1[_e265] = (inverseSqrt(_e284) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3195_ = (_e265 + 1i);
                }
            }
            let _e289 = local_2;
            let _e293 = local_1;
            local_1 = (_e168 + ((_e168 * ((_e289 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e293));
            break;
        }
        case 9: {
            let _e296 = local_2;
            local_1 = abs((_e168 - _e296));
            break;
        }
        case 10: {
            let _e299 = local_2;
            local_1 = ((_e299 + _e168) - ((_e299 * 2f) * _e168));
            break;
        }
        case 12: {
            if cj {
                let _e304 = local_2;
                let _e305 = clamp(_e304, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e305;
                let _e320 = (_e305 - vec3(min(min(_e305.x, _e305.y), _e305.z)));
                let _e328 = (_e320 * ((max(max(_e168.x, _e168.y), _e168.z) - min(min(_e168.x, _e168.y), _e168.z)) / max(0.000062f, max(max(_e320.x, _e320.y), _e320.z))));
                let _e329 = dot(_e168, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e332 = (_e328 - vec3(dot(_e328, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e345 = (vec2<f32>(_e329, (1f - _e329)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e332.x, _e332.y), _e332.z)), max(max(_e332.x, _e332.y), _e332.z))));
                local_1 = ((_e332 * min(1f, min(_e345.x, _e345.y))) + vec3(_e329));
            }
            break;
        }
        case 13: {
            if cj {
                let _e353 = local_2;
                let _e354 = clamp(_e353, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e354;
                let _e369 = (_e168 - vec3(min(min(_e168.x, _e168.y), _e168.z)));
                let _e377 = (_e369 * ((max(max(_e354.x, _e354.y), _e354.z) - min(min(_e354.x, _e354.y), _e354.z)) / max(0.000062f, max(max(_e369.x, _e369.y), _e369.z))));
                let _e378 = dot(_e168, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e381 = (_e377 - vec3(dot(_e377, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e394 = (vec2<f32>(_e378, (1f - _e378)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e381.x, _e381.y), _e381.z)), max(max(_e381.x, _e381.y), _e381.z))));
                local_1 = ((_e381 * min(1f, min(_e394.x, _e394.y))) + vec3(_e378));
            }
            break;
        }
        case 14: {
            if cj {
                let _e402 = local_2;
                let _e403 = clamp(_e402, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e403;
                let _e404 = dot(_e168, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e407 = (_e403 - vec3(dot(_e403, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e420 = (vec2<f32>(_e404, (1f - _e404)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e407.x, _e407.y), _e407.z)), max(max(_e407.x, _e407.y), _e407.z))));
                local_1 = ((_e407 * min(1f, min(_e420.x, _e420.y))) + vec3(_e404));
            }
            break;
        }
        case 15: {
            if cj {
                let _e428 = local_2;
                let _e429 = clamp(_e428, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e429;
                let _e430 = dot(_e429, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e433 = (_e168 - vec3(dot(_e168, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e446 = (vec2<f32>(_e430, (1f - _e430)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e433.x, _e433.y), _e433.z)), max(max(_e433.x, _e433.y), _e433.z))));
                local_1 = ((_e433 * min(1f, min(_e446.x, _e446.y))) + vec3(_e430));
            }
            break;
        }
        default: {
        }
    }
    let _e454 = local_1;
    let _e456 = mix(_e161, _e454, vec3(_e160.w));
    let _e462 = vec4<f32>(_e456.x, _e155.y, _e155.z, _e155.w);
    let _e468 = vec4<f32>(_e462.x, _e456.y, _e462.z, _e462.w);
    let _e474 = vec4<f32>(_e468.x, _e468.y, _e456.z, _e468.w);
    let _e477 = (_e474.xyz * _e155.w);
    let _e483 = vec4<f32>(_e477.x, _e474.y, _e474.z, _e474.w);
    let _e489 = vec4<f32>(_e483.x, _e477.y, _e483.z, _e483.w);
    let _e495 = vec4<f32>(_e489.x, _e489.y, _e477.z, _e489.w);
    let _e496 = _e495.xyz;
    let _e497 = gl_FragCoord_1;
    let _e499 = j.F3_;
    let _e501 = j.G3_;
    if (dj && (_e155.w != 0f)) {
        phi_3351_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e497.x) + (0.00583715f * _e497.y))))) * _e499) + _e501)) + _e496);
    } else {
        phi_3351_ = _e496;
    }
    let _e517 = phi_3351_;
    let _e523 = vec4<f32>(_e517.x, _e495.y, _e495.z, _e495.w);
    let _e529 = vec4<f32>(_e523.x, _e517.y, _e523.z, _e523.w);
    Fi = vec4<f32>(_e529.x, _e529.y, _e517.z, _e529.w);
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) P0_: f32, @location(0) O0_: vec4<f32>, @location(2) U0_: vec3<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    P0_1 = P0_;
    O0_1 = O0_;
    U0_1 = U0_;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e9 = Fi;
    return _e9;
}
