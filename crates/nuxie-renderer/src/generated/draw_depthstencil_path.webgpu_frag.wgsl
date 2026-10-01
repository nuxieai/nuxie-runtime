struct UB {
    Qc: f32,
    Td: f32,
    bg: f32,
    cg: f32,
    A6_: u32,
    X9_: u32,
    Nf: u32,
    Of: u32,
    j8_: vec4<i32>,
    Lh: vec2<f32>,
    Ud: vec2<f32>,
    j2_: u32,
    Ph: f32,
    T4_: u32,
    a3_: f32,
    Vd: f32,
    Hf: u32,
    L3_: f32,
    M3_: f32,
    Wd: f32,
    Ih: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

@id(7) override ri: bool = true;
@id(6) override qi: bool = true;
@id(2) override mi: bool = true;
@id(8) override si: bool = true;

@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var ha: sampler;
@group(1) @binding(11)
var CC: texture_2d<f32>;
@group(1) @binding(13)
var r5_: sampler;
var<private> r1_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> a1_1: vec4<f32>;
@group(0) @binding(12)
var XD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Sh: vec4<f32>;
@group(3) @binding(9)
var wa: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> l1_1: vec2<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2918_: f32;
    var phi_2919_: f32;
    var phi_2935_: vec4<f32>;
    var phi_2934_: vec4<f32>;
    var phi_1089_: bool;
    var phi_1104_: bool;
    var phi_2920_: f32;
    var phi_2930_: vec4<f32>;
    var phi_2937_: vec4<f32>;
    var phi_2938_: vec4<f32>;
    var phi_2939_: f32;
    var phi_3298_: vec4<f32>;
    var phi_3250_: i32;
    var phi_3419_: vec3<f32>;

    let _e54 = Q0_1;
    let _e55 = u32(_e54);
    let _e56 = r1_1;
    let _e57 = a1_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e60 = (mi && (_e55 != 0u));
            if (_e57.w >= 0f) {
                phi_2934_ = _e57;
            } else {
                let _e63 = -(_e57.w);
                let _e68 = j.wc;
                let _e71 = j.xc;
                if (_e57.z > 0f) {
                    phi_2918_ = _e57.x;
                } else {
                    phi_2918_ = length(_e57.xy);
                }
                let _e79 = phi_2918_;
                let _e80 = clamp(_e79, 0f, 1f);
                let _e81 = abs(_e57.z);
                if (_e81 > 1f) {
                    phi_2919_ = ((0.9980469f * _e80) + 0.0009765625f);
                } else {
                    phi_2919_ = ((0.001953125f * _e80) + _e81);
                }
                let _e88 = phi_2919_;
                let _e90 = textureSampleLevel(ED, ha, vec2<f32>(_e88, ((floor(_e63) * _e68) + _e71)), 0f);
                phi_2935_ = _e90;
                if !(_e60) {
                    let _e94 = (_e90.xyz * _e90.w);
                    phi_2935_ = vec4<f32>(_e94.x, _e94.y, _e94.z, (_e90.w * (fract(_e63) * 1.0039216f)));
                }
                let _e101 = phi_2935_;
                phi_2934_ = _e101;
            }
            let _e103 = phi_2934_;
            phi_1089_ = si;
            if si {
                phi_1089_ = (_e56.z < 0f);
            }
            let _e107 = phi_1089_;
            if _e107 {
                let _e109 = textureSampleLevel(CC, r5_, _e56.xy, 0f);
                phi_2938_ = _e109;
                break;
            }
            phi_1104_ = si;
            if si {
                phi_1104_ = (_e56.z > 0f);
            }
            let _e113 = phi_1104_;
            phi_2937_ = _e103;
            if _e113 {
                let _e117 = textureSampleLevel(CC, r5_, _e56.xy, (_e56.z - 1f));
                phi_2930_ = _e117;
                if _e60 {
                    if (_e117.w != 0f) {
                        phi_2920_ = (1f / _e117.w);
                    } else {
                        phi_2920_ = 0f;
                    }
                    let _e123 = phi_2920_;
                    let _e124 = (_e117.xyz * _e123);
                    phi_2930_ = vec4<f32>(_e124.x, _e124.y, _e124.z, _e117.w);
                }
                let _e130 = phi_2930_;
                phi_2937_ = (_e103 * _e130);
            }
            let _e133 = phi_2937_;
            phi_2938_ = _e133;
            break;
        }
    }
    let _e135 = phi_2938_;
    let _e136 = gl_FragCoord_1;
    let _e140 = textureLoad(XD, vec2<i32>(floor(_e136.xy)), 0i);
    let _e141 = _e135.xyz;
    local_2 = _e141;
    let _e142 = _e140.xyz;
    if (_e140.w != 0f) {
        phi_2939_ = (1f / _e140.w);
    } else {
        phi_2939_ = 0f;
    }
    let _e147 = phi_2939_;
    let _e148 = (_e142 * _e147);
    local = _e148;
    switch bitcast<i32>(_e55) {
        case 11: {
            let _e150 = local_2;
            local_1 = (_e150 * _e148);
            break;
        }
        case 1: {
            let _e152 = local_2;
            local_1 = ((_e152 + _e148) - (_e152 * _e148));
            break;
        }
        case 2: {
            let _e156 = local_2;
            let _e157 = (_e156 * _e148);
            local_1 = (select(_e157, (((_e156 + _e148) - _e157) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e148 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e164 = local_2;
            local_1 = min(_e164, _e148);
            break;
        }
        case 4: {
            let _e166 = local_2;
            local_1 = max(_e166, _e148);
            break;
        }
        case 5: {
            let _e169 = clamp(_e142, vec3<f32>(0f, 0f, 0f), _e140.www);
            let _e175 = vec4<f32>(_e169.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e181 = vec4<f32>(_e175.x, _e169.y, _e175.z, _e175.w);
            let _e188 = local_2;
            let _e191 = (clamp((vec3<f32>(1f, 1f, 1f) - _e188), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e140.w);
            let _e192 = vec4<f32>(_e181.x, _e181.y, _e169.z, _e181.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e192 / _e191)), sign(_e192), (_e191 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e198 = local_2;
            local_2 = clamp(_e198, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e201 = clamp(_e142, vec3<f32>(0f, 0f, 0f), _e140.www);
            let _e207 = vec4<f32>(_e201.x, _e140.y, _e140.z, _e140.w);
            let _e213 = vec4<f32>(_e207.x, _e201.y, _e207.z, _e207.w);
            phi_3298_ = vec4<f32>(_e213.x, _e213.y, _e201.z, _e213.w);
            if (_e140.w == 0f) {
                phi_3298_ = vec4<f32>(_e201.x, _e201.y, _e201.z, 1f);
            }
            let _e223 = phi_3298_;
            let _e227 = (vec3(_e223.w) - _e223.xyz);
            let _e228 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e227 / (_e228 * _e223.w))), sign(_e227), (_e228 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e236 = local_2;
            let _e237 = (_e236 * _e148);
            local_1 = (select(_e237, (((_e236 + _e148) - _e237) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e236 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3250_ = 0i;
            loop {
                let _e245 = phi_3250_;
                if (_e245 < 3i) {
                    let _e248 = local_2[_e245];
                    if (_e248 <= 0.5f) {
                        let _e251 = local[_e245];
                        local_1[_e245] = (1f - _e251);
                    } else {
                        let _e255 = local[_e245];
                        if (_e255 <= 0.25f) {
                            let _e257 = local[_e245];
                            let _e260 = local[_e245];
                            local_1[_e245] = ((((16f * _e257) - 12f) * _e260) + 3f);
                        } else {
                            let _e264 = local[_e245];
                            local_1[_e245] = (inverseSqrt(_e264) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3250_ = (_e245 + 1i);
                }
            }
            let _e269 = local_2;
            let _e273 = local_1;
            local_1 = (_e148 + ((_e148 * ((_e269 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e273));
            break;
        }
        case 9: {
            let _e276 = local_2;
            local_1 = abs((_e148 - _e276));
            break;
        }
        case 10: {
            let _e279 = local_2;
            local_1 = ((_e279 + _e148) - ((_e279 * 2f) * _e148));
            break;
        }
        case 12: {
            if qi {
                let _e284 = local_2;
                let _e285 = clamp(_e284, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e285;
                let _e300 = (_e285 - vec3(min(min(_e285.x, _e285.y), _e285.z)));
                let _e308 = (_e300 * ((max(max(_e148.x, _e148.y), _e148.z) - min(min(_e148.x, _e148.y), _e148.z)) / max(0.000062f, max(max(_e300.x, _e300.y), _e300.z))));
                let _e309 = dot(_e148, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e312 = (_e308 - vec3(dot(_e308, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e325 = (vec2<f32>(_e309, (1f - _e309)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e312.x, _e312.y), _e312.z)), max(max(_e312.x, _e312.y), _e312.z))));
                local_1 = ((_e312 * min(1f, min(_e325.x, _e325.y))) + vec3(_e309));
            }
            break;
        }
        case 13: {
            if qi {
                let _e333 = local_2;
                let _e334 = clamp(_e333, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e334;
                let _e349 = (_e148 - vec3(min(min(_e148.x, _e148.y), _e148.z)));
                let _e357 = (_e349 * ((max(max(_e334.x, _e334.y), _e334.z) - min(min(_e334.x, _e334.y), _e334.z)) / max(0.000062f, max(max(_e349.x, _e349.y), _e349.z))));
                let _e358 = dot(_e148, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e361 = (_e357 - vec3(dot(_e357, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e374 = (vec2<f32>(_e358, (1f - _e358)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e361.x, _e361.y), _e361.z)), max(max(_e361.x, _e361.y), _e361.z))));
                local_1 = ((_e361 * min(1f, min(_e374.x, _e374.y))) + vec3(_e358));
            }
            break;
        }
        case 14: {
            if qi {
                let _e382 = local_2;
                let _e383 = clamp(_e382, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e383;
                let _e384 = dot(_e148, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e387 = (_e383 - vec3(dot(_e383, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e400 = (vec2<f32>(_e384, (1f - _e384)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e387.x, _e387.y), _e387.z)), max(max(_e387.x, _e387.y), _e387.z))));
                local_1 = ((_e387 * min(1f, min(_e400.x, _e400.y))) + vec3(_e384));
            }
            break;
        }
        case 15: {
            if qi {
                let _e408 = local_2;
                let _e409 = clamp(_e408, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e409;
                let _e410 = dot(_e409, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e413 = (_e148 - vec3(dot(_e148, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e426 = (vec2<f32>(_e410, (1f - _e410)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e413.x, _e413.y), _e413.z)), max(max(_e413.x, _e413.y), _e413.z))));
                local_1 = ((_e413 * min(1f, min(_e426.x, _e426.y))) + vec3(_e410));
            }
            break;
        }
        default: {
        }
    }
    let _e434 = local_1;
    let _e438 = (mix(_e141, _e434, vec3(_e140.w)) * _e135.w);
    let _e444 = vec4<f32>(_e438.x, _e135.y, _e135.z, _e135.w);
    let _e450 = vec4<f32>(_e444.x, _e438.y, _e444.z, _e444.w);
    let _e457 = (vec4<f32>(_e450.x, _e450.y, _e438.z, _e450.w) * 1f);
    let _e458 = _e457.xyz;
    let _e460 = gl_FragCoord_1;
    let _e462 = j.L3_;
    let _e464 = j.M3_;
    if (ri && (_e457.w != 0f)) {
        phi_3419_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e460.x) + (0.00583715f * _e460.y))))) * _e462) + _e464)) + _e458);
    } else {
        phi_3419_ = _e458;
    }
    let _e480 = phi_3419_;
    let _e486 = vec4<f32>(_e480.x, _e457.y, _e457.z, _e457.w);
    let _e492 = vec4<f32>(_e486.x, _e480.y, _e486.z, _e486.w);
    Sh = vec4<f32>(_e492.x, _e492.y, _e480.z, _e492.w);
    return;
}

@fragment
fn main(@location(9) r1_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) a1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) l1_: vec2<f32>) -> @location(0) vec4<f32> {
    r1_1 = r1_;
    Q0_1 = Q0_;
    a1_1 = a1_;
    gl_FragCoord_1 = gl_FragCoord;
    l1_1 = l1_;
    main_1();
    let _e11 = Sh;
    return _e11;
}
