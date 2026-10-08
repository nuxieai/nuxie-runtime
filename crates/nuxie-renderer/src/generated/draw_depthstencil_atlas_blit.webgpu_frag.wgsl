struct VB {
    td: f32,
    ze: f32,
    Dg: f32,
    Eg: f32,
    L6_: u32,
    va: u32,
    pg: u32,
    qg: u32,
    B8_: vec4<i32>,
    xi: vec2<f32>,
    Ae: vec2<f32>,
    r2_: u32,
    Bi: f32,
    p6_: u32,
    h3_: f32,
    Be: f32,
    jg: u32,
    E3_: f32,
    F3_: f32,
    Ce: f32,
    ui: u32,
    ua: u32,
    ad: f32,
    g7_: f32,
    Bb: f32,
}

@id(7) override bj: bool = true;
@id(6) override aj: bool = true;
@id(2) override Wi: bool = true;
@id(8) override cj: bool = true;

@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var H8_: sampler;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var S4_: sampler;
@group(0) @binding(10)
var GD: texture_2d<f32>;
@group(3) @binding(10)
var Ja: sampler;
var<private> T2_1: vec2<f32>;
var<private> V0_1: vec3<f32>;
var<private> P0_1: f32;
var<private> O0_1: vec4<f32>;
@group(0) @binding(12)
var JD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Ei: vec4<f32>;
@group(3) @binding(9)
var Ta: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
var<private> e4_1: f32;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2939_: f32;
    var phi_2940_: f32;
    var phi_2956_: vec4<f32>;
    var phi_2955_: vec4<f32>;
    var phi_1110_: bool;
    var phi_1125_: bool;
    var phi_2941_: f32;
    var phi_2951_: vec4<f32>;
    var phi_2958_: vec4<f32>;
    var phi_2959_: vec4<f32>;
    var phi_2960_: f32;
    var phi_3319_: vec4<f32>;
    var phi_3271_: i32;
    var phi_3440_: vec3<f32>;

    let _e57 = T2_1;
    let _e58 = textureSampleLevel(GD, Ja, _e57, 0f);
    let _e61 = P0_1;
    let _e62 = u32(_e61);
    let _e63 = V0_1;
    let _e64 = O0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e67 = (Wi && (_e62 != 0u));
            if (_e64.w >= 0f) {
                phi_2955_ = _e64;
            } else {
                let _e70 = -(_e64.w);
                let _e75 = j.ad;
                let _e78 = j.g7_;
                if (_e64.z > 0f) {
                    phi_2939_ = _e64.x;
                } else {
                    phi_2939_ = length(_e64.xy);
                }
                let _e86 = phi_2939_;
                let _e87 = clamp(_e86, 0f, 1f);
                let _e88 = abs(_e64.z);
                if (_e88 > 1f) {
                    phi_2940_ = ((0.9980469f * _e87) + 0.0009765625f);
                } else {
                    phi_2940_ = ((0.001953125f * _e87) + _e88);
                }
                let _e95 = phi_2940_;
                let _e97 = textureSampleLevel(YC, H8_, vec2<f32>(_e95, ((floor(_e70) * _e75) + _e78)), 0f);
                phi_2956_ = _e97;
                if !(_e67) {
                    let _e101 = (_e97.xyz * _e97.w);
                    phi_2956_ = vec4<f32>(_e101.x, _e101.y, _e101.z, (_e97.w * (fract(_e70) * 1.0039216f)));
                }
                let _e108 = phi_2956_;
                phi_2955_ = _e108;
            }
            let _e110 = phi_2955_;
            phi_1110_ = cj;
            if cj {
                phi_1110_ = (_e63.z < 0f);
            }
            let _e114 = phi_1110_;
            if _e114 {
                let _e116 = textureSampleLevel(TB, S4_, _e63.xy, 0f);
                phi_2959_ = _e116;
                break;
            }
            phi_1125_ = cj;
            if cj {
                phi_1125_ = (_e63.z > 0f);
            }
            let _e120 = phi_1125_;
            phi_2958_ = _e110;
            if _e120 {
                let _e124 = textureSampleLevel(TB, S4_, _e63.xy, (_e63.z - 1f));
                phi_2951_ = _e124;
                if _e67 {
                    if (_e124.w != 0f) {
                        phi_2941_ = (1f / _e124.w);
                    } else {
                        phi_2941_ = 0f;
                    }
                    let _e130 = phi_2941_;
                    let _e131 = (_e124.xyz * _e130);
                    phi_2951_ = vec4<f32>(_e131.x, _e131.y, _e131.z, _e124.w);
                }
                let _e137 = phi_2951_;
                phi_2958_ = (_e110 * _e137);
            }
            let _e140 = phi_2958_;
            phi_2959_ = _e140;
            break;
        }
    }
    let _e142 = phi_2959_;
    let _e143 = gl_FragCoord_1;
    let _e147 = textureLoad(JD, vec2<i32>(floor(_e143.xy)), 0i);
    let _e148 = _e142.xyz;
    local_2 = _e148;
    let _e149 = _e147.xyz;
    if (_e147.w != 0f) {
        phi_2960_ = (1f / _e147.w);
    } else {
        phi_2960_ = 0f;
    }
    let _e154 = phi_2960_;
    let _e155 = (_e149 * _e154);
    local = _e155;
    switch bitcast<i32>(_e62) {
        case 11: {
            let _e157 = local_2;
            local_1 = (_e157 * _e155);
            break;
        }
        case 1: {
            let _e159 = local_2;
            local_1 = ((_e159 + _e155) - (_e159 * _e155));
            break;
        }
        case 2: {
            let _e163 = local_2;
            let _e164 = (_e163 * _e155);
            local_1 = (select(_e164, (((_e163 + _e155) - _e164) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e155 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e171 = local_2;
            local_1 = min(_e171, _e155);
            break;
        }
        case 4: {
            let _e173 = local_2;
            local_1 = max(_e173, _e155);
            break;
        }
        case 5: {
            let _e176 = clamp(_e149, vec3<f32>(0f, 0f, 0f), _e147.www);
            let _e182 = vec4<f32>(_e176.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e188 = vec4<f32>(_e182.x, _e176.y, _e182.z, _e182.w);
            let _e195 = local_2;
            let _e198 = (clamp((vec3<f32>(1f, 1f, 1f) - _e195), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e147.w);
            let _e199 = vec4<f32>(_e188.x, _e188.y, _e176.z, _e188.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e199 / _e198)), sign(_e199), (_e198 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e205 = local_2;
            local_2 = clamp(_e205, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e208 = clamp(_e149, vec3<f32>(0f, 0f, 0f), _e147.www);
            let _e214 = vec4<f32>(_e208.x, _e147.y, _e147.z, _e147.w);
            let _e220 = vec4<f32>(_e214.x, _e208.y, _e214.z, _e214.w);
            phi_3319_ = vec4<f32>(_e220.x, _e220.y, _e208.z, _e220.w);
            if (_e147.w == 0f) {
                phi_3319_ = vec4<f32>(_e208.x, _e208.y, _e208.z, 1f);
            }
            let _e230 = phi_3319_;
            let _e234 = (vec3(_e230.w) - _e230.xyz);
            let _e235 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e234 / (_e235 * _e230.w))), sign(_e234), (_e235 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e243 = local_2;
            let _e244 = (_e243 * _e155);
            local_1 = (select(_e244, (((_e243 + _e155) - _e244) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e243 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3271_ = 0i;
            loop {
                let _e252 = phi_3271_;
                if (_e252 < 3i) {
                    let _e255 = local_2[_e252];
                    if (_e255 <= 0.5f) {
                        let _e258 = local[_e252];
                        local_1[_e252] = (1f - _e258);
                    } else {
                        let _e262 = local[_e252];
                        if (_e262 <= 0.25f) {
                            let _e264 = local[_e252];
                            let _e267 = local[_e252];
                            local_1[_e252] = ((((16f * _e264) - 12f) * _e267) + 3f);
                        } else {
                            let _e271 = local[_e252];
                            local_1[_e252] = (inverseSqrt(_e271) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3271_ = (_e252 + 1i);
                }
            }
            let _e276 = local_2;
            let _e280 = local_1;
            local_1 = (_e155 + ((_e155 * ((_e276 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e280));
            break;
        }
        case 9: {
            let _e283 = local_2;
            local_1 = abs((_e155 - _e283));
            break;
        }
        case 10: {
            let _e286 = local_2;
            local_1 = ((_e286 + _e155) - ((_e286 * 2f) * _e155));
            break;
        }
        case 12: {
            if aj {
                let _e291 = local_2;
                let _e292 = clamp(_e291, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e292;
                let _e307 = (_e292 - vec3(min(min(_e292.x, _e292.y), _e292.z)));
                let _e315 = (_e307 * ((max(max(_e155.x, _e155.y), _e155.z) - min(min(_e155.x, _e155.y), _e155.z)) / max(0.000062f, max(max(_e307.x, _e307.y), _e307.z))));
                let _e316 = dot(_e155, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e319 = (_e315 - vec3(dot(_e315, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e332 = (vec2<f32>(_e316, (1f - _e316)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e319.x, _e319.y), _e319.z)), max(max(_e319.x, _e319.y), _e319.z))));
                local_1 = ((_e319 * min(1f, min(_e332.x, _e332.y))) + vec3(_e316));
            }
            break;
        }
        case 13: {
            if aj {
                let _e340 = local_2;
                let _e341 = clamp(_e340, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e341;
                let _e356 = (_e155 - vec3(min(min(_e155.x, _e155.y), _e155.z)));
                let _e364 = (_e356 * ((max(max(_e341.x, _e341.y), _e341.z) - min(min(_e341.x, _e341.y), _e341.z)) / max(0.000062f, max(max(_e356.x, _e356.y), _e356.z))));
                let _e365 = dot(_e155, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e368 = (_e364 - vec3(dot(_e364, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e381 = (vec2<f32>(_e365, (1f - _e365)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e368.x, _e368.y), _e368.z)), max(max(_e368.x, _e368.y), _e368.z))));
                local_1 = ((_e368 * min(1f, min(_e381.x, _e381.y))) + vec3(_e365));
            }
            break;
        }
        case 14: {
            if aj {
                let _e389 = local_2;
                let _e390 = clamp(_e389, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e390;
                let _e391 = dot(_e155, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e394 = (_e390 - vec3(dot(_e390, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e407 = (vec2<f32>(_e391, (1f - _e391)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e394.x, _e394.y), _e394.z)), max(max(_e394.x, _e394.y), _e394.z))));
                local_1 = ((_e394 * min(1f, min(_e407.x, _e407.y))) + vec3(_e391));
            }
            break;
        }
        case 15: {
            if aj {
                let _e415 = local_2;
                let _e416 = clamp(_e415, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e416;
                let _e417 = dot(_e416, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e420 = (_e155 - vec3(dot(_e155, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e433 = (vec2<f32>(_e417, (1f - _e417)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e420.x, _e420.y), _e420.z)), max(max(_e420.x, _e420.y), _e420.z))));
                local_1 = ((_e420 * min(1f, min(_e433.x, _e433.y))) + vec3(_e417));
            }
            break;
        }
        default: {
        }
    }
    let _e441 = local_1;
    let _e445 = (mix(_e148, _e441, vec3(_e147.w)) * _e142.w);
    let _e451 = vec4<f32>(_e445.x, _e142.y, _e142.z, _e142.w);
    let _e457 = vec4<f32>(_e451.x, _e445.y, _e451.z, _e451.w);
    let _e464 = (vec4<f32>(_e457.x, _e457.y, _e445.z, _e457.w) * clamp(_e58.x, 0f, 1f));
    let _e465 = _e464.xyz;
    let _e467 = gl_FragCoord_1;
    let _e469 = j.E3_;
    let _e471 = j.F3_;
    if (bj && (_e464.w != 0f)) {
        phi_3440_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e467.x) + (0.00583715f * _e467.y))))) * _e469) + _e471)) + _e465);
    } else {
        phi_3440_ = _e465;
    }
    let _e487 = phi_3440_;
    let _e493 = vec4<f32>(_e487.x, _e464.y, _e464.z, _e464.w);
    let _e499 = vec4<f32>(_e493.x, _e487.y, _e493.z, _e493.w);
    Ei = vec4<f32>(_e499.x, _e499.y, _e487.z, _e499.w);
    return;
}

@fragment
fn main(@location(1) T2_: vec2<f32>, @location(9) V0_: vec3<f32>, @location(6) @interpolate(flat, either) P0_: f32, @location(0) O0_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) e4_: f32) -> @location(0) vec4<f32> {
    T2_1 = T2_;
    V0_1 = V0_;
    P0_1 = P0_;
    O0_1 = O0_;
    gl_FragCoord_1 = gl_FragCoord;
    e4_1 = e4_;
    main_1();
    let _e13 = Ei;
    return _e13;
}
