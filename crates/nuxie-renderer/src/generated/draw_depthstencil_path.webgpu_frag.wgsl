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
@id(15) override jj: bool = false;
@id(8) override cj: bool = true;

var<private> P0_1: f32;
var<private> O0_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var H8_: sampler;
var<private> V0_1: vec3<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var S4_: sampler;
@group(0) @binding(12)
var JD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Ei: vec4<f32>;
@group(3) @binding(9)
var Ta: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2808_: f32;
    var phi_2804_: f32;
    var phi_2806_: vec2<f32>;
    var phi_2809_: vec4<f32>;
    var phi_2824_: vec4<f32>;
    var phi_2823_: vec4<f32>;
    var phi_891_: bool;
    var phi_2810_: f32;
    var phi_2820_: vec4<f32>;
    var phi_2826_: vec4<f32>;
    var phi_2827_: f32;
    var phi_3153_: vec4<f32>;
    var phi_3109_: i32;
    var phi_3265_: vec3<f32>;

    let _e61 = P0_1;
    let _e62 = u32(_e61);
    let _e64 = (Wi && (_e62 != 0u));
    let _e66 = O0_1[3u];
    if (_e66 >= 0f) {
        let _e68 = O0_1;
        phi_2823_ = _e68;
    } else {
        let _e69 = O0_1;
        let _e71 = j.Bb;
        let _e73 = j.g7_;
        let _e75 = bitcast<u32>(_e69.w);
        let _e79 = ((f32((_e75 & 268304384u)) * _e71) - _e73);
        let _e81 = abs(_e69.z);
        if (_e81 < 1.5f) {
            phi_2808_ = _e81;
            phi_2804_ = _e69.x;
        } else {
            phi_2808_ = (_e81 - 2f);
            phi_2804_ = length(_e69.xy);
        }
        let _e88 = phi_2808_;
        let _e90 = phi_2804_;
        let _e91 = clamp(_e90, 0f, 1f);
        if (_e69.z < 0f) {
            phi_2806_ = vec2<f32>(((_e91 * 0.9980469f) + 0.0009765625f), _e79);
        } else {
            phi_2806_ = vec2<f32>(((_e91 * 0.001953125f) + ((f32((_e75 & 130816u)) * 0.0000076293945f) + 0.0009765625f)), _e79);
        }
        let _e104 = phi_2806_;
        let _e105 = textureSampleLevel(YC, H8_, _e104, 0f);
        phi_2809_ = _e105;
        if jj {
            phi_2809_ = vec4<f32>(_e105.x, _e105.y, _e105.z, (_e105.w * _e88));
        }
        let _e114 = phi_2809_;
        phi_2824_ = _e114;
        if !(_e64) {
            let _e118 = (_e114.xyz * _e114.w);
            phi_2824_ = vec4<f32>(_e118.x, _e118.y, _e118.z, (_e114.w * (f32((_e75 & 255u)) * 0.003921569f)));
        }
        let _e128 = phi_2824_;
        phi_2823_ = _e128;
    }
    let _e130 = phi_2823_;
    phi_891_ = cj;
    if cj {
        let _e132 = V0_1[2u];
        phi_891_ = (_e132 > 0f);
    }
    let _e135 = phi_891_;
    phi_2826_ = _e130;
    if _e135 {
        let _e137 = V0_1[2u];
        let _e139 = V0_1;
        let _e141 = textureSampleLevel(TB, S4_, _e139.xy, (_e137 - 1f));
        phi_2820_ = _e141;
        if _e64 {
            if (_e141.w != 0f) {
                phi_2810_ = (1f / _e141.w);
            } else {
                phi_2810_ = 0f;
            }
            let _e147 = phi_2810_;
            let _e148 = (_e141.xyz * _e147);
            phi_2820_ = vec4<f32>(_e148.x, _e148.y, _e148.z, _e141.w);
        }
        let _e154 = phi_2820_;
        phi_2826_ = (_e130 * _e154);
    }
    let _e157 = phi_2826_;
    let _e158 = gl_FragCoord_1;
    let _e162 = textureLoad(JD, vec2<i32>(floor(_e158.xy)), 0i);
    let _e163 = _e157.xyz;
    local_2 = _e163;
    let _e164 = _e162.xyz;
    if (_e162.w != 0f) {
        phi_2827_ = (1f / _e162.w);
    } else {
        phi_2827_ = 0f;
    }
    let _e169 = phi_2827_;
    let _e170 = (_e164 * _e169);
    local = _e170;
    switch bitcast<i32>(_e62) {
        case 11: {
            let _e172 = local_2;
            local_1 = (_e172 * _e170);
            break;
        }
        case 1: {
            let _e174 = local_2;
            local_1 = ((_e174 + _e170) - (_e174 * _e170));
            break;
        }
        case 2: {
            let _e178 = local_2;
            let _e179 = (_e178 * _e170);
            local_1 = (select(_e179, (((_e178 + _e170) - _e179) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e170 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e186 = local_2;
            local_1 = min(_e186, _e170);
            break;
        }
        case 4: {
            let _e188 = local_2;
            local_1 = max(_e188, _e170);
            break;
        }
        case 5: {
            let _e191 = clamp(_e164, vec3<f32>(0f, 0f, 0f), _e162.www);
            let _e197 = vec4<f32>(_e191.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e203 = vec4<f32>(_e197.x, _e191.y, _e197.z, _e197.w);
            let _e210 = local_2;
            let _e213 = (clamp((vec3<f32>(1f, 1f, 1f) - _e210), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e162.w);
            let _e214 = vec4<f32>(_e203.x, _e203.y, _e191.z, _e203.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e214 / _e213)), sign(_e214), (_e213 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e220 = local_2;
            local_2 = clamp(_e220, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e223 = clamp(_e164, vec3<f32>(0f, 0f, 0f), _e162.www);
            let _e229 = vec4<f32>(_e223.x, _e162.y, _e162.z, _e162.w);
            let _e235 = vec4<f32>(_e229.x, _e223.y, _e229.z, _e229.w);
            phi_3153_ = vec4<f32>(_e235.x, _e235.y, _e223.z, _e235.w);
            if (_e162.w == 0f) {
                phi_3153_ = vec4<f32>(_e223.x, _e223.y, _e223.z, 1f);
            }
            let _e245 = phi_3153_;
            let _e249 = (vec3(_e245.w) - _e245.xyz);
            let _e250 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e249 / (_e250 * _e245.w))), sign(_e249), (_e250 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e258 = local_2;
            let _e259 = (_e258 * _e170);
            local_1 = (select(_e259, (((_e258 + _e170) - _e259) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e258 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3109_ = 0i;
            loop {
                let _e267 = phi_3109_;
                if (_e267 < 3i) {
                    let _e270 = local_2[_e267];
                    if (_e270 <= 0.5f) {
                        let _e273 = local[_e267];
                        local_1[_e267] = (1f - _e273);
                    } else {
                        let _e277 = local[_e267];
                        if (_e277 <= 0.25f) {
                            let _e279 = local[_e267];
                            let _e282 = local[_e267];
                            local_1[_e267] = ((((16f * _e279) - 12f) * _e282) + 3f);
                        } else {
                            let _e286 = local[_e267];
                            local_1[_e267] = (inverseSqrt(_e286) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3109_ = (_e267 + 1i);
                }
            }
            let _e291 = local_2;
            let _e295 = local_1;
            local_1 = (_e170 + ((_e170 * ((_e291 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e295));
            break;
        }
        case 9: {
            let _e298 = local_2;
            local_1 = abs((_e170 - _e298));
            break;
        }
        case 10: {
            let _e301 = local_2;
            local_1 = ((_e301 + _e170) - ((_e301 * 2f) * _e170));
            break;
        }
        case 12: {
            if aj {
                let _e306 = local_2;
                let _e307 = clamp(_e306, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e307;
                let _e322 = (_e307 - vec3(min(min(_e307.x, _e307.y), _e307.z)));
                let _e330 = (_e322 * ((max(max(_e170.x, _e170.y), _e170.z) - min(min(_e170.x, _e170.y), _e170.z)) / max(0.000062f, max(max(_e322.x, _e322.y), _e322.z))));
                let _e331 = dot(_e170, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e334 = (_e330 - vec3(dot(_e330, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e347 = (vec2<f32>(_e331, (1f - _e331)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e334.x, _e334.y), _e334.z)), max(max(_e334.x, _e334.y), _e334.z))));
                local_1 = ((_e334 * min(1f, min(_e347.x, _e347.y))) + vec3(_e331));
            }
            break;
        }
        case 13: {
            if aj {
                let _e355 = local_2;
                let _e356 = clamp(_e355, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e356;
                let _e371 = (_e170 - vec3(min(min(_e170.x, _e170.y), _e170.z)));
                let _e379 = (_e371 * ((max(max(_e356.x, _e356.y), _e356.z) - min(min(_e356.x, _e356.y), _e356.z)) / max(0.000062f, max(max(_e371.x, _e371.y), _e371.z))));
                let _e380 = dot(_e170, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e383 = (_e379 - vec3(dot(_e379, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e396 = (vec2<f32>(_e380, (1f - _e380)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e383.x, _e383.y), _e383.z)), max(max(_e383.x, _e383.y), _e383.z))));
                local_1 = ((_e383 * min(1f, min(_e396.x, _e396.y))) + vec3(_e380));
            }
            break;
        }
        case 14: {
            if aj {
                let _e404 = local_2;
                let _e405 = clamp(_e404, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e405;
                let _e406 = dot(_e170, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e409 = (_e405 - vec3(dot(_e405, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e422 = (vec2<f32>(_e406, (1f - _e406)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e409.x, _e409.y), _e409.z)), max(max(_e409.x, _e409.y), _e409.z))));
                local_1 = ((_e409 * min(1f, min(_e422.x, _e422.y))) + vec3(_e406));
            }
            break;
        }
        case 15: {
            if aj {
                let _e430 = local_2;
                let _e431 = clamp(_e430, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e431;
                let _e432 = dot(_e431, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e435 = (_e170 - vec3(dot(_e170, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e448 = (vec2<f32>(_e432, (1f - _e432)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e435.x, _e435.y), _e435.z)), max(max(_e435.x, _e435.y), _e435.z))));
                local_1 = ((_e435 * min(1f, min(_e448.x, _e448.y))) + vec3(_e432));
            }
            break;
        }
        default: {
        }
    }
    let _e456 = local_1;
    let _e458 = mix(_e163, _e456, vec3(_e162.w));
    let _e464 = vec4<f32>(_e458.x, _e157.y, _e157.z, _e157.w);
    let _e470 = vec4<f32>(_e464.x, _e458.y, _e464.z, _e464.w);
    let _e476 = vec4<f32>(_e470.x, _e470.y, _e458.z, _e470.w);
    let _e479 = (_e476.xyz * _e157.w);
    let _e485 = vec4<f32>(_e479.x, _e476.y, _e476.z, _e476.w);
    let _e491 = vec4<f32>(_e485.x, _e479.y, _e485.z, _e485.w);
    let _e497 = vec4<f32>(_e491.x, _e491.y, _e479.z, _e491.w);
    let _e498 = _e497.xyz;
    let _e499 = gl_FragCoord_1;
    let _e501 = j.E3_;
    let _e503 = j.F3_;
    if (bj && (_e157.w != 0f)) {
        phi_3265_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e499.x) + (0.00583715f * _e499.y))))) * _e501) + _e503)) + _e498);
    } else {
        phi_3265_ = _e498;
    }
    let _e519 = phi_3265_;
    let _e525 = vec4<f32>(_e519.x, _e497.y, _e497.z, _e497.w);
    let _e531 = vec4<f32>(_e525.x, _e519.y, _e525.z, _e525.w);
    Ei = vec4<f32>(_e531.x, _e531.y, _e519.z, _e531.w);
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) P0_: f32, @location(0) O0_: vec4<f32>, @location(2) V0_: vec3<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    P0_1 = P0_;
    O0_1 = O0_;
    V0_1 = V0_;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e9 = Ei;
    return _e9;
}
