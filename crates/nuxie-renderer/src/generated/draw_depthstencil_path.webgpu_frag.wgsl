struct VB {
    yd: f32,
    Ee: f32,
    Jg: f32,
    Kg: f32,
    S6_: u32,
    Ea: u32,
    vg: u32,
    wg: u32,
    H8_: vec4<i32>,
    Bi: vec2<f32>,
    Fe: vec2<f32>,
    q2_: u32,
    Fi: f32,
    w6_: u32,
    i3_: f32,
    Ge: f32,
    og: u32,
    F3_: f32,
    G3_: f32,
    He: f32,
    yi: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

@id(7) override gj: bool = true;
@id(6) override fj: bool = true;
@id(15) override oj: bool = false;
@id(16) override pj: bool = false;
@id(2) override bj: bool = true;
@id(8) override hj: bool = true;

var<private> Q0_1: f32;
var<private> P0_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var Q8_: sampler;
var<private> U0_1: vec3<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;
@group(0) @binding(12)
var MD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Ii: vec4<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2992_: f32;
    var phi_2993_: f32;
    var phi_2994_: vec2<f32>;
    var phi_2995_: bool;
    var phi_2997_: vec4<f32>;
    var phi_3014_: vec4<f32>;
    var phi_3013_: vec4<f32>;
    var phi_975_: bool;
    var phi_2998_: f32;
    var phi_3010_: vec4<f32>;
    var phi_3016_: vec4<f32>;
    var phi_3017_: f32;
    var phi_3355_: i32;
    var phi_3407_: vec4<f32>;
    var phi_3537_: vec3<f32>;

    let _e65 = Q0_1;
    let _e66 = u32(_e65);
    let _e68 = (bj && (_e66 != 0u));
    let _e70 = P0_1[3u];
    if (_e70 >= 0f) {
        let _e148 = P0_1;
        phi_3013_ = _e148;
    } else {
        let _e72 = P0_1;
        let _e74 = j.O8_;
        let _e76 = j.P8_;
        let _e79 = abs(_e72.z);
        let _e80 = floor(_e79);
        let _e83 = (((_e79 - _e80) * 8f) + 0.0009765625f);
        let _e85 = ((_e80 * _e74) + _e76);
        let _e86 = floor(_e83);
        switch bitcast<i32>(0u) {
            default: {
                if (_e86 == 4f) {
                    let _e104 = normalize(vec2<f32>(_e72.x, -(_e72.y)));
                    let _e107 = acos(clamp(_e104.x, -1f, 1f));
                    if (_e104.y >= 0f) {
                        phi_2993_ = _e107;
                    } else {
                        phi_2993_ = -(_e107);
                    }
                    let _e112 = phi_2993_;
                    phi_2994_ = vec2<f32>((_e112 * 0.15915507f), _e85);
                    break;
                } else {
                    if (_e86 == 2f) {
                        phi_2992_ = _e72.x;
                    } else {
                        phi_2992_ = length(_e72.xy);
                    }
                    let _e94 = phi_2992_;
                    phi_2994_ = vec2<f32>(((clamp(_e94, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e72.z < 0f))) + _e83), _e85);
                    break;
                }
            }
        }
        let _e116 = phi_2994_;
        let _e117 = textureSampleLevel(ZC, Q8_, _e116, 0f);
        switch bitcast<i32>(0u) {
            default: {
                if oj {
                    phi_2995_ = true;
                    break;
                }
                if pj {
                    phi_2995_ = true;
                    break;
                }
                phi_2995_ = false;
                break;
            }
        }
        let _e120 = phi_2995_;
        phi_2997_ = _e117;
        if _e120 {
            phi_2997_ = vec4<f32>(_e117.x, _e117.y, _e117.z, (_e117.w * ((fract(_e72.w) * -2f) + 1.5f)));
        }
        let _e133 = phi_2997_;
        phi_3014_ = _e133;
        if !(_e68) {
            let _e137 = (_e133.xyz * _e133.w);
            phi_3014_ = vec4<f32>(_e137.x, _e137.y, _e137.z, (_e133.w * (ceil(_e72.w) * -0.003921569f)));
        }
        let _e147 = phi_3014_;
        phi_3013_ = _e147;
    }
    let _e150 = phi_3013_;
    phi_975_ = hj;
    if hj {
        let _e152 = U0_1[2u];
        phi_975_ = (_e152 > 0f);
    }
    let _e155 = phi_975_;
    phi_3016_ = _e150;
    if _e155 {
        let _e157 = U0_1[2u];
        let _e159 = U0_1;
        let _e161 = textureSampleLevel(TB, U4_, _e159.xy, (_e157 - 1f));
        phi_3010_ = _e161;
        if _e68 {
            if (_e161.w != 0f) {
                phi_2998_ = (1f / _e161.w);
            } else {
                phi_2998_ = 0f;
            }
            let _e167 = phi_2998_;
            let _e168 = (_e161.xyz * _e167);
            phi_3010_ = vec4<f32>(_e168.x, _e168.y, _e168.z, _e161.w);
        }
        let _e174 = phi_3010_;
        phi_3016_ = (_e150 * _e174);
    }
    let _e177 = phi_3016_;
    let _e178 = gl_FragCoord_1;
    let _e182 = textureLoad(MD, vec2<i32>(floor(_e178.xy)), 0i);
    let _e183 = _e177.xyz;
    local_2 = _e183;
    let _e184 = _e182.xyz;
    if (_e182.w != 0f) {
        phi_3017_ = (1f / _e182.w);
    } else {
        phi_3017_ = 0f;
    }
    let _e189 = phi_3017_;
    let _e190 = (_e184 * _e189);
    local = _e190;
    switch bitcast<i32>(_e66) {
        case 11: {
            let _e474 = local_2;
            local_1 = (_e474 * _e190);
            break;
        }
        case 1: {
            let _e470 = local_2;
            local_1 = ((_e470 + _e190) - (_e470 * _e190));
            break;
        }
        case 2: {
            let _e462 = local_2;
            let _e463 = (_e462 * _e190);
            local_1 = (select(_e463, (((_e462 + _e190) - _e463) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e190 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e460 = local_2;
            local_1 = min(_e460, _e190);
            break;
        }
        case 4: {
            let _e458 = local_2;
            local_1 = max(_e458, _e190);
            break;
        }
        case 5: {
            let _e429 = clamp(_e184, vec3<f32>(0f, 0f, 0f), _e182.www);
            let _e435 = vec4<f32>(_e429.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e441 = vec4<f32>(_e435.x, _e429.y, _e435.z, _e435.w);
            let _e448 = local_2;
            let _e451 = (clamp((vec3<f32>(1f, 1f, 1f) - _e448), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e182.w);
            let _e452 = vec4<f32>(_e441.x, _e441.y, _e429.z, _e441.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e452 / _e451)), sign(_e452), (_e451 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e390 = local_2;
            local_2 = clamp(_e390, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e393 = clamp(_e184, vec3<f32>(0f, 0f, 0f), _e182.www);
            let _e399 = vec4<f32>(_e393.x, _e182.y, _e182.z, _e182.w);
            let _e405 = vec4<f32>(_e399.x, _e393.y, _e399.z, _e399.w);
            phi_3407_ = vec4<f32>(_e405.x, _e405.y, _e393.z, _e405.w);
            if (_e182.w == 0f) {
                phi_3407_ = vec4<f32>(_e393.x, _e393.y, _e393.z, 1f);
            }
            let _e415 = phi_3407_;
            let _e419 = (vec3(_e415.w) - _e415.xyz);
            let _e420 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e419 / (_e420 * _e415.w))), sign(_e419), (_e420 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e382 = local_2;
            let _e383 = (_e382 * _e190);
            local_1 = (select(_e383, (((_e382 + _e190) - _e383) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e382 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3355_ = 0i;
            loop {
                let _e351 = phi_3355_;
                if (_e351 < 3i) {
                    let _e354 = local_2[_e351];
                    if (_e354 <= 0.5f) {
                        let _e371 = local[_e351];
                        local_1[_e351] = (1f - _e371);
                    } else {
                        let _e357 = local[_e351];
                        if (_e357 <= 0.25f) {
                            let _e363 = local[_e351];
                            let _e366 = local[_e351];
                            local_1[_e351] = ((((16f * _e363) - 12f) * _e366) + 3f);
                        } else {
                            let _e359 = local[_e351];
                            local_1[_e351] = (inverseSqrt(_e359) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3355_ = (_e351 + 1i);
                }
            }
            let _e375 = local_2;
            let _e379 = local_1;
            local_1 = (_e190 + ((_e190 * ((_e375 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e379));
            break;
        }
        case 9: {
            let _e347 = local_2;
            local_1 = abs((_e190 - _e347));
            break;
        }
        case 10: {
            let _e342 = local_2;
            local_1 = ((_e342 + _e190) - ((_e342 * 2f) * _e190));
            break;
        }
        case 12: {
            if fj {
                let _e293 = local_2;
                let _e294 = clamp(_e293, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e294;
                let _e309 = (_e294 - vec3(min(min(_e294.x, _e294.y), _e294.z)));
                let _e317 = (_e309 * ((max(max(_e190.x, _e190.y), _e190.z) - min(min(_e190.x, _e190.y), _e190.z)) / max(0.000062f, max(max(_e309.x, _e309.y), _e309.z))));
                let _e318 = dot(_e190, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e321 = (_e317 - vec3(dot(_e317, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e334 = (vec2<f32>(_e318, (1f - _e318)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e321.x, _e321.y), _e321.z)), max(max(_e321.x, _e321.y), _e321.z))));
                local_1 = ((_e321 * min(1f, min(_e334.x, _e334.y))) + vec3(_e318));
            }
            break;
        }
        case 13: {
            if fj {
                let _e244 = local_2;
                let _e245 = clamp(_e244, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e245;
                let _e260 = (_e190 - vec3(min(min(_e190.x, _e190.y), _e190.z)));
                let _e268 = (_e260 * ((max(max(_e245.x, _e245.y), _e245.z) - min(min(_e245.x, _e245.y), _e245.z)) / max(0.000062f, max(max(_e260.x, _e260.y), _e260.z))));
                let _e269 = dot(_e190, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e272 = (_e268 - vec3(dot(_e268, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e285 = (vec2<f32>(_e269, (1f - _e269)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e272.x, _e272.y), _e272.z)), max(max(_e272.x, _e272.y), _e272.z))));
                local_1 = ((_e272 * min(1f, min(_e285.x, _e285.y))) + vec3(_e269));
            }
            break;
        }
        case 14: {
            if fj {
                let _e218 = local_2;
                let _e219 = clamp(_e218, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e219;
                let _e220 = dot(_e190, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e223 = (_e219 - vec3(dot(_e219, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e236 = (vec2<f32>(_e220, (1f - _e220)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e223.x, _e223.y), _e223.z)), max(max(_e223.x, _e223.y), _e223.z))));
                local_1 = ((_e223 * min(1f, min(_e236.x, _e236.y))) + vec3(_e220));
            }
            break;
        }
        case 15: {
            if fj {
                let _e192 = local_2;
                let _e193 = clamp(_e192, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e193;
                let _e194 = dot(_e193, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e197 = (_e190 - vec3(dot(_e190, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e210 = (vec2<f32>(_e194, (1f - _e194)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e197.x, _e197.y), _e197.z)), max(max(_e197.x, _e197.y), _e197.z))));
                local_1 = ((_e197 * min(1f, min(_e210.x, _e210.y))) + vec3(_e194));
            }
            break;
        }
        default: {
        }
    }
    let _e476 = local_1;
    let _e478 = mix(_e183, _e476, vec3(_e182.w));
    let _e484 = vec4<f32>(_e478.x, _e177.y, _e177.z, _e177.w);
    let _e490 = vec4<f32>(_e484.x, _e478.y, _e484.z, _e484.w);
    let _e496 = vec4<f32>(_e490.x, _e490.y, _e478.z, _e490.w);
    let _e499 = (_e496.xyz * _e177.w);
    let _e505 = vec4<f32>(_e499.x, _e496.y, _e496.z, _e496.w);
    let _e511 = vec4<f32>(_e505.x, _e499.y, _e505.z, _e505.w);
    let _e517 = vec4<f32>(_e511.x, _e511.y, _e499.z, _e511.w);
    let _e518 = _e517.xyz;
    let _e519 = gl_FragCoord_1;
    let _e521 = j.F3_;
    let _e523 = j.G3_;
    if (gj && (_e177.w != 0f)) {
        phi_3537_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e519.x) + (0.00583715f * _e519.y))))) * _e521) + _e523)) + _e518);
    } else {
        phi_3537_ = _e518;
    }
    let _e539 = phi_3537_;
    let _e545 = vec4<f32>(_e539.x, _e517.y, _e517.z, _e517.w);
    let _e551 = vec4<f32>(_e545.x, _e539.y, _e545.z, _e545.w);
    Ii = vec4<f32>(_e551.x, _e551.y, _e539.z, _e551.w);
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) Q0_: f32, @location(0) P0_: vec4<f32>, @location(2) U0_: vec3<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    Q0_1 = Q0_;
    P0_1 = P0_;
    U0_1 = U0_;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e9 = Ii;
    return _e9;
}
