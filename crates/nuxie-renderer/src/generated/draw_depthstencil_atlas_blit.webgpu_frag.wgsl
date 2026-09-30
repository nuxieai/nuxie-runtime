struct UB {
    Qc: f32,
    Td: f32,
    Yf: f32,
    Zf: f32,
    z6_: u32,
    X9_: u32,
    Kf: u32,
    Lf: u32,
    i8_: vec4<i32>,
    Ch: vec2<f32>,
    Ud: vec2<f32>,
    i2_: u32,
    Gh: f32,
    T4_: u32,
    c3_: f32,
    Vd: f32,
    Ef: u32,
    M3_: f32,
    N3_: f32,
    Wd: f32,
    zh: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

@id(7) override ii: bool = true;
@id(6) override hi: bool = true;
@id(2) override di: bool = true;
@id(8) override ji: bool = true;

@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var FD: texture_2d<f32>;
@group(3) @binding(8)
var ha: sampler;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var f6_: sampler;
@group(0) @binding(10)
var GD: texture_2d<f32>;
@group(3) @binding(10)
var ma: sampler;
var<private> K2_1: vec2<f32>;
var<private> F1_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> a1_1: vec4<f32>;
@group(0) @binding(12)
var YD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Jh: vec4<f32>;
@group(3) @binding(9)
var wa: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
var<private> Z3_1: f32;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2888_: f32;
    var phi_2889_: f32;
    var phi_2903_: vec4<f32>;
    var phi_2902_: vec4<f32>;
    var phi_1081_: bool;
    var phi_2890_: f32;
    var phi_2899_: vec4<f32>;
    var phi_2905_: vec4<f32>;
    var phi_2906_: f32;
    var phi_3201_: vec4<f32>;
    var phi_3161_: i32;
    var phi_3304_: vec3<f32>;

    let _e57 = K2_1;
    let _e58 = textureSampleLevel(GD, ma, _e57, 0f);
    let _e61 = Q0_1;
    let _e62 = u32(_e61);
    let _e63 = F1_1;
    let _e64 = a1_1;
    let _e66 = (di && (_e62 != 0u));
    if (_e64.w >= 0f) {
        phi_2902_ = _e64;
    } else {
        let _e69 = -(_e64.w);
        let _e74 = j.wc;
        let _e77 = j.xc;
        if (_e64.z > 0f) {
            phi_2888_ = _e64.x;
        } else {
            phi_2888_ = length(_e64.xy);
        }
        let _e85 = phi_2888_;
        let _e86 = clamp(_e85, 0f, 1f);
        let _e87 = abs(_e64.z);
        if (_e87 > 1f) {
            phi_2889_ = ((0.9980469f * _e86) + 0.0009765625f);
        } else {
            phi_2889_ = ((0.001953125f * _e86) + _e87);
        }
        let _e94 = phi_2889_;
        let _e96 = textureSampleLevel(FD, ha, vec2<f32>(_e94, ((floor(_e69) * _e74) + _e77)), 0f);
        phi_2903_ = _e96;
        if !(_e66) {
            let _e100 = (_e96.xyz * _e96.w);
            phi_2903_ = vec4<f32>(_e100.x, _e100.y, _e100.z, (_e96.w * (fract(_e69) * 1.0039216f)));
        }
        let _e107 = phi_2903_;
        phi_2902_ = _e107;
    }
    let _e109 = phi_2902_;
    phi_1081_ = ji;
    if ji {
        phi_1081_ = (_e63.z > 0f);
    }
    let _e113 = phi_1081_;
    phi_2905_ = _e109;
    if _e113 {
        let _e117 = textureSampleLevel(IC, f6_, _e63.xy, (_e63.z - 1f));
        phi_2899_ = _e117;
        if _e66 {
            if (_e117.w != 0f) {
                phi_2890_ = (1f / _e117.w);
            } else {
                phi_2890_ = 0f;
            }
            let _e123 = phi_2890_;
            let _e124 = (_e117.xyz * _e123);
            phi_2899_ = vec4<f32>(_e124.x, _e124.y, _e124.z, _e117.w);
        }
        let _e130 = phi_2899_;
        phi_2905_ = (_e109 * _e130);
    }
    let _e133 = phi_2905_;
    let _e134 = gl_FragCoord_1;
    let _e138 = textureLoad(YD, vec2<i32>(floor(_e134.xy)), 0i);
    let _e139 = _e133.xyz;
    local_2 = _e139;
    let _e140 = _e138.xyz;
    if (_e138.w != 0f) {
        phi_2906_ = (1f / _e138.w);
    } else {
        phi_2906_ = 0f;
    }
    let _e145 = phi_2906_;
    let _e146 = (_e140 * _e145);
    local = _e146;
    switch bitcast<i32>(_e62) {
        case 11: {
            let _e148 = local_2;
            local_1 = (_e148 * _e146);
            break;
        }
        case 1: {
            let _e150 = local_2;
            local_1 = ((_e150 + _e146) - (_e150 * _e146));
            break;
        }
        case 2: {
            let _e154 = local_2;
            let _e155 = (_e154 * _e146);
            local_1 = (select(_e155, (((_e154 + _e146) - _e155) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e146 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e162 = local_2;
            local_1 = min(_e162, _e146);
            break;
        }
        case 4: {
            let _e164 = local_2;
            local_1 = max(_e164, _e146);
            break;
        }
        case 5: {
            let _e167 = clamp(_e140, vec3<f32>(0f, 0f, 0f), _e138.www);
            let _e173 = vec4<f32>(_e167.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e179 = vec4<f32>(_e173.x, _e167.y, _e173.z, _e173.w);
            let _e186 = local_2;
            let _e189 = (clamp((vec3<f32>(1f, 1f, 1f) - _e186), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e138.w);
            let _e190 = vec4<f32>(_e179.x, _e179.y, _e167.z, _e179.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e190 / _e189)), sign(_e190), (_e189 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e196 = local_2;
            local_2 = clamp(_e196, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e199 = clamp(_e140, vec3<f32>(0f, 0f, 0f), _e138.www);
            let _e205 = vec4<f32>(_e199.x, _e138.y, _e138.z, _e138.w);
            let _e211 = vec4<f32>(_e205.x, _e199.y, _e205.z, _e205.w);
            phi_3201_ = vec4<f32>(_e211.x, _e211.y, _e199.z, _e211.w);
            if (_e138.w == 0f) {
                phi_3201_ = vec4<f32>(_e199.x, _e199.y, _e199.z, 1f);
            }
            let _e221 = phi_3201_;
            let _e225 = (vec3(_e221.w) - _e221.xyz);
            let _e226 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e225 / (_e226 * _e221.w))), sign(_e225), (_e226 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e234 = local_2;
            let _e235 = (_e234 * _e146);
            local_1 = (select(_e235, (((_e234 + _e146) - _e235) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e234 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3161_ = 0i;
            loop {
                let _e243 = phi_3161_;
                if (_e243 < 3i) {
                    let _e246 = local_2[_e243];
                    if (_e246 <= 0.5f) {
                        let _e249 = local[_e243];
                        local_1[_e243] = (1f - _e249);
                    } else {
                        let _e253 = local[_e243];
                        if (_e253 <= 0.25f) {
                            let _e255 = local[_e243];
                            let _e258 = local[_e243];
                            local_1[_e243] = ((((16f * _e255) - 12f) * _e258) + 3f);
                        } else {
                            let _e262 = local[_e243];
                            local_1[_e243] = (inverseSqrt(_e262) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3161_ = (_e243 + 1i);
                }
            }
            let _e267 = local_2;
            let _e271 = local_1;
            local_1 = (_e146 + ((_e146 * ((_e267 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e271));
            break;
        }
        case 9: {
            let _e274 = local_2;
            local_1 = abs((_e146 - _e274));
            break;
        }
        case 10: {
            let _e277 = local_2;
            local_1 = ((_e277 + _e146) - ((_e277 * 2f) * _e146));
            break;
        }
        case 12: {
            if hi {
                let _e282 = local_2;
                let _e283 = clamp(_e282, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e283;
                let _e298 = (_e283 - vec3(min(min(_e283.x, _e283.y), _e283.z)));
                let _e306 = (_e298 * ((max(max(_e146.x, _e146.y), _e146.z) - min(min(_e146.x, _e146.y), _e146.z)) / max(0.000062f, max(max(_e298.x, _e298.y), _e298.z))));
                let _e307 = dot(_e146, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e310 = (_e306 - vec3(dot(_e306, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e323 = (vec2<f32>(_e307, (1f - _e307)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e310.x, _e310.y), _e310.z)), max(max(_e310.x, _e310.y), _e310.z))));
                local_1 = ((_e310 * min(1f, min(_e323.x, _e323.y))) + vec3(_e307));
            }
            break;
        }
        case 13: {
            if hi {
                let _e331 = local_2;
                let _e332 = clamp(_e331, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e332;
                let _e347 = (_e146 - vec3(min(min(_e146.x, _e146.y), _e146.z)));
                let _e355 = (_e347 * ((max(max(_e332.x, _e332.y), _e332.z) - min(min(_e332.x, _e332.y), _e332.z)) / max(0.000062f, max(max(_e347.x, _e347.y), _e347.z))));
                let _e356 = dot(_e146, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e359 = (_e355 - vec3(dot(_e355, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e372 = (vec2<f32>(_e356, (1f - _e356)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e359.x, _e359.y), _e359.z)), max(max(_e359.x, _e359.y), _e359.z))));
                local_1 = ((_e359 * min(1f, min(_e372.x, _e372.y))) + vec3(_e356));
            }
            break;
        }
        case 14: {
            if hi {
                let _e380 = local_2;
                let _e381 = clamp(_e380, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e381;
                let _e382 = dot(_e146, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e385 = (_e381 - vec3(dot(_e381, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e398 = (vec2<f32>(_e382, (1f - _e382)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e385.x, _e385.y), _e385.z)), max(max(_e385.x, _e385.y), _e385.z))));
                local_1 = ((_e385 * min(1f, min(_e398.x, _e398.y))) + vec3(_e382));
            }
            break;
        }
        case 15: {
            if hi {
                let _e406 = local_2;
                let _e407 = clamp(_e406, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e407;
                let _e408 = dot(_e407, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e411 = (_e146 - vec3(dot(_e146, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e424 = (vec2<f32>(_e408, (1f - _e408)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e411.x, _e411.y), _e411.z)), max(max(_e411.x, _e411.y), _e411.z))));
                local_1 = ((_e411 * min(1f, min(_e424.x, _e424.y))) + vec3(_e408));
            }
            break;
        }
        default: {
        }
    }
    let _e432 = local_1;
    let _e436 = (mix(_e139, _e432, vec3(_e138.w)) * _e133.w);
    let _e442 = vec4<f32>(_e436.x, _e133.y, _e133.z, _e133.w);
    let _e448 = vec4<f32>(_e442.x, _e436.y, _e442.z, _e442.w);
    let _e455 = (vec4<f32>(_e448.x, _e448.y, _e436.z, _e448.w) * clamp(_e58.x, 0f, 1f));
    let _e456 = _e455.xyz;
    let _e458 = gl_FragCoord_1;
    let _e460 = j.M3_;
    let _e462 = j.N3_;
    if (ii && (_e455.w != 0f)) {
        phi_3304_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e458.x) + (0.00583715f * _e458.y))))) * _e460) + _e462)) + _e456);
    } else {
        phi_3304_ = _e456;
    }
    let _e478 = phi_3304_;
    let _e484 = vec4<f32>(_e478.x, _e455.y, _e455.z, _e455.w);
    let _e490 = vec4<f32>(_e484.x, _e478.y, _e484.z, _e484.w);
    Jh = vec4<f32>(_e490.x, _e490.y, _e478.z, _e490.w);
    return;
}

@fragment
fn main(@location(1) K2_: vec2<f32>, @location(9) F1_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) a1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) Z3_: f32) -> @location(0) vec4<f32> {
    K2_1 = K2_;
    F1_1 = F1_;
    Q0_1 = Q0_;
    a1_1 = a1_;
    gl_FragCoord_1 = gl_FragCoord;
    Z3_1 = Z3_;
    main_1();
    let _e13 = Jh;
    return _e13;
}
