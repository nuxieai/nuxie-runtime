struct DC {
    gc: f32,
    qd: f32,
    jf: f32,
    kf: f32,
    o6_: u32,
    Lg: u32,
    Ue: u32,
    Ve: u32,
    T7_: vec4<i32>,
    Hg: vec2<f32>,
    rd: vec2<f32>,
    a2_: u32,
    Mg: f32,
    c6_: u32,
    R2_: f32,
    sd: f32,
    Pe: u32,
    B3_: f32,
    C3_: f32,
    td: f32,
    Eg: u32,
}

@id(7) override lh: bool = true;
@id(6) override kh: bool = true;
@id(2) override gh: bool = true;
@id(8) override mh: bool = true;

@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(3) @binding(8)
var Ob: sampler;
@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(1) @binding(13)
var U5_: sampler;
var<private> f1_1: vec4<f32>;
var<private> A2_1: vec3<f32>;
var<private> e2_1: f32;
@group(0) @binding(12)
var UD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> Pg: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> U1_1: vec2<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2821_: vec4<f32>;
    var phi_2805_: f32;
    var phi_2806_: f32;
    var phi_2822_: vec4<f32>;
    var phi_2820_: vec4<f32>;
    var phi_1045_: bool;
    var phi_2807_: f32;
    var phi_2817_: vec4<f32>;
    var phi_2824_: vec4<f32>;
    var phi_2825_: f32;
    var phi_3152_: vec4<f32>;
    var phi_3108_: i32;
    var phi_3264_: vec3<f32>;

    let _e50 = f1_1;
    let _e51 = A2_1;
    if (_e50.w >= 0f) {
        if gh {
            phi_2821_ = vec4<f32>(_e50.x, _e50.y, _e50.z, _e50.w);
        } else {
            phi_2821_ = (_e50 * 1f);
        }
        let _e62 = phi_2821_;
        phi_2820_ = _e62;
    } else {
        if (_e50.z > 0f) {
            phi_2805_ = _e50.x;
        } else {
            phi_2805_ = length(_e50.xy);
        }
        let _e69 = phi_2805_;
        let _e70 = clamp(_e69, 0f, 1f);
        let _e71 = abs(_e50.z);
        if (_e71 > 1f) {
            phi_2806_ = ((0.9980469f * _e70) + 0.0009765625f);
        } else {
            phi_2806_ = ((0.001953125f * _e70) + _e71);
        }
        let _e78 = phi_2806_;
        let _e81 = textureSampleLevel(MD, Ob, vec2<f32>(_e78, -(_e50.w)), 0f);
        let _e87 = vec4<f32>(_e81.x, _e81.y, _e81.z, _e81.w);
        if gh {
            phi_2822_ = _e87;
        } else {
            let _e89 = (_e87.xyz * _e81.w);
            phi_2822_ = vec4<f32>(_e89.x, _e89.y, _e89.z, _e81.w);
        }
        let _e95 = phi_2822_;
        phi_2820_ = _e95;
    }
    let _e97 = phi_2820_;
    phi_1045_ = mh;
    if mh {
        phi_1045_ = (_e51.z > 0f);
    }
    let _e101 = phi_1045_;
    phi_2824_ = _e97;
    if _e101 {
        let _e105 = textureSampleLevel(JC, U5_, _e51.xy, (_e51.z - 1f));
        phi_2817_ = _e105;
        if gh {
            if (_e105.w != 0f) {
                phi_2807_ = (1f / _e105.w);
            } else {
                phi_2807_ = 0f;
            }
            let _e111 = phi_2807_;
            let _e112 = (_e105.xyz * _e111);
            phi_2817_ = vec4<f32>(_e112.x, _e112.y, _e112.z, _e105.w);
        }
        let _e118 = phi_2817_;
        phi_2824_ = (_e97 * _e118);
    }
    let _e121 = phi_2824_;
    let _e122 = e2_1;
    let _e124 = gl_FragCoord_1;
    let _e128 = textureLoad(UD, vec2<i32>(floor(_e124.xy)), 0i);
    let _e129 = _e121.xyz;
    local_2 = _e129;
    let _e130 = _e128.xyz;
    if (_e128.w != 0f) {
        phi_2825_ = (1f / _e128.w);
    } else {
        phi_2825_ = 0f;
    }
    let _e135 = phi_2825_;
    let _e136 = (_e130 * _e135);
    local = _e136;
    switch bitcast<i32>(u32(_e122)) {
        case 11: {
            let _e138 = local_2;
            local_1 = (_e138 * _e136);
            break;
        }
        case 1: {
            let _e140 = local_2;
            local_1 = ((_e140 + _e136) - (_e140 * _e136));
            break;
        }
        case 2: {
            let _e144 = local_2;
            let _e145 = (_e144 * _e136);
            local_1 = (select(_e145, (((_e144 + _e136) - _e145) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e136 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e152 = local_2;
            local_1 = min(_e152, _e136);
            break;
        }
        case 4: {
            let _e154 = local_2;
            local_1 = max(_e154, _e136);
            break;
        }
        case 5: {
            let _e157 = clamp(_e130, vec3<f32>(0f, 0f, 0f), _e128.www);
            let _e163 = vec4<f32>(_e157.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e169 = vec4<f32>(_e163.x, _e157.y, _e163.z, _e163.w);
            let _e176 = local_2;
            let _e179 = (clamp((vec3<f32>(1f, 1f, 1f) - _e176), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e128.w);
            let _e180 = vec4<f32>(_e169.x, _e169.y, _e157.z, _e169.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e180 / _e179)), sign(_e180), (_e179 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e186 = local_2;
            local_2 = clamp(_e186, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e189 = clamp(_e130, vec3<f32>(0f, 0f, 0f), _e128.www);
            let _e195 = vec4<f32>(_e189.x, _e128.y, _e128.z, _e128.w);
            let _e201 = vec4<f32>(_e195.x, _e189.y, _e195.z, _e195.w);
            phi_3152_ = vec4<f32>(_e201.x, _e201.y, _e189.z, _e201.w);
            if (_e128.w == 0f) {
                phi_3152_ = vec4<f32>(_e189.x, _e189.y, _e189.z, 1f);
            }
            let _e211 = phi_3152_;
            let _e215 = (vec3(_e211.w) - _e211.xyz);
            let _e216 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e215 / (_e216 * _e211.w))), sign(_e215), (_e216 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e224 = local_2;
            let _e225 = (_e224 * _e136);
            local_1 = (select(_e225, (((_e224 + _e136) - _e225) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e224 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3108_ = 0i;
            loop {
                let _e233 = phi_3108_;
                if (_e233 < 3i) {
                    let _e236 = local_2[_e233];
                    if (_e236 <= 0.5f) {
                        let _e239 = local[_e233];
                        local_1[_e233] = (1f - _e239);
                    } else {
                        let _e243 = local[_e233];
                        if (_e243 <= 0.25f) {
                            let _e245 = local[_e233];
                            let _e248 = local[_e233];
                            local_1[_e233] = ((((16f * _e245) - 12f) * _e248) + 3f);
                        } else {
                            let _e252 = local[_e233];
                            local_1[_e233] = (inverseSqrt(_e252) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3108_ = (_e233 + 1i);
                }
            }
            let _e257 = local_2;
            let _e261 = local_1;
            local_1 = (_e136 + ((_e136 * ((_e257 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e261));
            break;
        }
        case 9: {
            let _e264 = local_2;
            local_1 = abs((_e136 - _e264));
            break;
        }
        case 10: {
            let _e267 = local_2;
            local_1 = ((_e267 + _e136) - ((_e267 * 2f) * _e136));
            break;
        }
        case 12: {
            if kh {
                let _e272 = local_2;
                let _e273 = clamp(_e272, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e273;
                let _e288 = (_e273 - vec3(min(min(_e273.x, _e273.y), _e273.z)));
                let _e296 = (_e288 * ((max(max(_e136.x, _e136.y), _e136.z) - min(min(_e136.x, _e136.y), _e136.z)) / max(0.000062f, max(max(_e288.x, _e288.y), _e288.z))));
                let _e297 = dot(_e136, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e300 = (_e296 - vec3(dot(_e296, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e313 = (vec2<f32>(_e297, (1f - _e297)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e300.x, _e300.y), _e300.z)), max(max(_e300.x, _e300.y), _e300.z))));
                local_1 = ((_e300 * min(1f, min(_e313.x, _e313.y))) + vec3(_e297));
            }
            break;
        }
        case 13: {
            if kh {
                let _e321 = local_2;
                let _e322 = clamp(_e321, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e322;
                let _e337 = (_e136 - vec3(min(min(_e136.x, _e136.y), _e136.z)));
                let _e345 = (_e337 * ((max(max(_e322.x, _e322.y), _e322.z) - min(min(_e322.x, _e322.y), _e322.z)) / max(0.000062f, max(max(_e337.x, _e337.y), _e337.z))));
                let _e346 = dot(_e136, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e349 = (_e345 - vec3(dot(_e345, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e362 = (vec2<f32>(_e346, (1f - _e346)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e349.x, _e349.y), _e349.z)), max(max(_e349.x, _e349.y), _e349.z))));
                local_1 = ((_e349 * min(1f, min(_e362.x, _e362.y))) + vec3(_e346));
            }
            break;
        }
        case 14: {
            if kh {
                let _e370 = local_2;
                let _e371 = clamp(_e370, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e371;
                let _e372 = dot(_e136, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e375 = (_e371 - vec3(dot(_e371, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e388 = (vec2<f32>(_e372, (1f - _e372)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e375.x, _e375.y), _e375.z)), max(max(_e375.x, _e375.y), _e375.z))));
                local_1 = ((_e375 * min(1f, min(_e388.x, _e388.y))) + vec3(_e372));
            }
            break;
        }
        case 15: {
            if kh {
                let _e396 = local_2;
                let _e397 = clamp(_e396, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e397;
                let _e398 = dot(_e397, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e401 = (_e136 - vec3(dot(_e136, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e414 = (vec2<f32>(_e398, (1f - _e398)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e401.x, _e401.y), _e401.z)), max(max(_e401.x, _e401.y), _e401.z))));
                local_1 = ((_e401 * min(1f, min(_e414.x, _e414.y))) + vec3(_e398));
            }
            break;
        }
        default: {
        }
    }
    let _e422 = local_1;
    let _e424 = mix(_e129, _e422, vec3(_e128.w));
    let _e430 = vec4<f32>(_e424.x, _e121.y, _e121.z, _e121.w);
    let _e436 = vec4<f32>(_e430.x, _e424.y, _e430.z, _e430.w);
    let _e442 = vec4<f32>(_e436.x, _e436.y, _e424.z, _e436.w);
    let _e445 = (_e442.xyz * _e121.w);
    let _e451 = vec4<f32>(_e445.x, _e442.y, _e442.z, _e442.w);
    let _e457 = vec4<f32>(_e451.x, _e445.y, _e451.z, _e451.w);
    let _e463 = vec4<f32>(_e457.x, _e457.y, _e445.z, _e457.w);
    let _e464 = _e463.xyz;
    let _e465 = gl_FragCoord_1;
    let _e467 = m.B3_;
    let _e469 = m.C3_;
    if (lh && (_e121.w != 0f)) {
        phi_3264_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e465.x) + (0.00583715f * _e465.y))))) * _e467) + _e469)) + _e464);
    } else {
        phi_3264_ = _e464;
    }
    let _e485 = phi_3264_;
    let _e491 = vec4<f32>(_e485.x, _e463.y, _e463.z, _e463.w);
    let _e497 = vec4<f32>(_e491.x, _e485.y, _e491.z, _e491.w);
    Pg = vec4<f32>(_e497.x, _e497.y, _e485.z, _e497.w);
    return;
}

@fragment
fn main(@location(0) f1_: vec4<f32>, @location(9) A2_: vec3<f32>, @location(6) @interpolate(flat, either) e2_: f32, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) U1_: vec2<f32>) -> @location(0) vec4<f32> {
    f1_1 = f1_;
    A2_1 = A2_;
    e2_1 = e2_;
    gl_FragCoord_1 = gl_FragCoord;
    U1_1 = U1_;
    main_1();
    let _e11 = Pg;
    return _e11;
}
