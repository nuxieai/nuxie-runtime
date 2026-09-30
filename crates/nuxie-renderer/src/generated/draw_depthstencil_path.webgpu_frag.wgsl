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
var<private> Y1_1: vec2<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2802_: f32;
    var phi_2803_: f32;
    var phi_2817_: vec4<f32>;
    var phi_2816_: vec4<f32>;
    var phi_1007_: bool;
    var phi_2804_: f32;
    var phi_2813_: vec4<f32>;
    var phi_2819_: vec4<f32>;
    var phi_2820_: f32;
    var phi_3115_: vec4<f32>;
    var phi_3075_: i32;
    var phi_3218_: vec3<f32>;

    let _e51 = g1_1;
    let _e52 = u32(_e51);
    let _e53 = C2_1;
    let _e54 = X1_1;
    let _e56 = (Gh && (_e52 != 0u));
    if (_e54.w >= 0f) {
        phi_2816_ = _e54;
    } else {
        if (_e54.z > 0f) {
            phi_2802_ = _e54.x;
        } else {
            phi_2802_ = length(_e54.xy);
        }
        let _e66 = phi_2802_;
        let _e67 = clamp(_e66, 0f, 1f);
        let _e68 = abs(_e54.z);
        if (_e68 > 1f) {
            phi_2803_ = ((0.9980469f * _e67) + 0.0009765625f);
        } else {
            phi_2803_ = ((0.001953125f * _e67) + _e68);
        }
        let _e75 = phi_2803_;
        let _e77 = textureSampleLevel(DD, P9_, vec2<f32>(_e75, -(_e54.w)), 0f);
        phi_2817_ = _e77;
        if !(_e56) {
            let _e81 = (_e77.xyz * _e77.w);
            let _e87 = vec4<f32>(_e81.x, _e77.y, _e77.z, _e77.w);
            let _e93 = vec4<f32>(_e87.x, _e81.y, _e87.z, _e87.w);
            phi_2817_ = vec4<f32>(_e93.x, _e93.y, _e81.z, _e93.w);
        }
        let _e101 = phi_2817_;
        phi_2816_ = _e101;
    }
    let _e103 = phi_2816_;
    phi_1007_ = Mh;
    if Mh {
        phi_1007_ = (_e53.z > 0f);
    }
    let _e107 = phi_1007_;
    phi_2819_ = _e103;
    if _e107 {
        let _e111 = textureSampleLevel(GC, Y5_, _e53.xy, (_e53.z - 1f));
        phi_2813_ = _e111;
        if _e56 {
            if (_e111.w != 0f) {
                phi_2804_ = (1f / _e111.w);
            } else {
                phi_2804_ = 0f;
            }
            let _e117 = phi_2804_;
            let _e118 = (_e111.xyz * _e117);
            phi_2813_ = vec4<f32>(_e118.x, _e118.y, _e118.z, _e111.w);
        }
        let _e124 = phi_2813_;
        phi_2819_ = (_e103 * _e124);
    }
    let _e127 = phi_2819_;
    let _e128 = gl_FragCoord_1;
    let _e132 = textureLoad(XD, vec2<i32>(floor(_e128.xy)), 0i);
    let _e133 = _e127.xyz;
    local_2 = _e133;
    let _e134 = _e132.xyz;
    if (_e132.w != 0f) {
        phi_2820_ = (1f / _e132.w);
    } else {
        phi_2820_ = 0f;
    }
    let _e139 = phi_2820_;
    let _e140 = (_e134 * _e139);
    local = _e140;
    switch bitcast<i32>(_e52) {
        case 11: {
            let _e142 = local_2;
            local_1 = (_e142 * _e140);
            break;
        }
        case 1: {
            let _e144 = local_2;
            local_1 = ((_e144 + _e140) - (_e144 * _e140));
            break;
        }
        case 2: {
            let _e148 = local_2;
            let _e149 = (_e148 * _e140);
            local_1 = (select(_e149, (((_e148 + _e140) - _e149) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e140 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e156 = local_2;
            local_1 = min(_e156, _e140);
            break;
        }
        case 4: {
            let _e158 = local_2;
            local_1 = max(_e158, _e140);
            break;
        }
        case 5: {
            let _e161 = clamp(_e134, vec3<f32>(0f, 0f, 0f), _e132.www);
            let _e167 = vec4<f32>(_e161.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e173 = vec4<f32>(_e167.x, _e161.y, _e167.z, _e167.w);
            let _e180 = local_2;
            let _e183 = (clamp((vec3<f32>(1f, 1f, 1f) - _e180), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e132.w);
            let _e184 = vec4<f32>(_e173.x, _e173.y, _e161.z, _e173.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e184 / _e183)), sign(_e184), (_e183 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e190 = local_2;
            local_2 = clamp(_e190, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e193 = clamp(_e134, vec3<f32>(0f, 0f, 0f), _e132.www);
            let _e199 = vec4<f32>(_e193.x, _e132.y, _e132.z, _e132.w);
            let _e205 = vec4<f32>(_e199.x, _e193.y, _e199.z, _e199.w);
            phi_3115_ = vec4<f32>(_e205.x, _e205.y, _e193.z, _e205.w);
            if (_e132.w == 0f) {
                phi_3115_ = vec4<f32>(_e193.x, _e193.y, _e193.z, 1f);
            }
            let _e215 = phi_3115_;
            let _e219 = (vec3(_e215.w) - _e215.xyz);
            let _e220 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e219 / (_e220 * _e215.w))), sign(_e219), (_e220 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e228 = local_2;
            let _e229 = (_e228 * _e140);
            local_1 = (select(_e229, (((_e228 + _e140) - _e229) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e228 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3075_ = 0i;
            loop {
                let _e237 = phi_3075_;
                if (_e237 < 3i) {
                    let _e240 = local_2[_e237];
                    if (_e240 <= 0.5f) {
                        let _e243 = local[_e237];
                        local_1[_e237] = (1f - _e243);
                    } else {
                        let _e247 = local[_e237];
                        if (_e247 <= 0.25f) {
                            let _e249 = local[_e237];
                            let _e252 = local[_e237];
                            local_1[_e237] = ((((16f * _e249) - 12f) * _e252) + 3f);
                        } else {
                            let _e256 = local[_e237];
                            local_1[_e237] = (inverseSqrt(_e256) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3075_ = (_e237 + 1i);
                }
            }
            let _e261 = local_2;
            let _e265 = local_1;
            local_1 = (_e140 + ((_e140 * ((_e261 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e265));
            break;
        }
        case 9: {
            let _e268 = local_2;
            local_1 = abs((_e140 - _e268));
            break;
        }
        case 10: {
            let _e271 = local_2;
            local_1 = ((_e271 + _e140) - ((_e271 * 2f) * _e140));
            break;
        }
        case 12: {
            if Kh {
                let _e276 = local_2;
                let _e277 = clamp(_e276, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e277;
                let _e292 = (_e277 - vec3(min(min(_e277.x, _e277.y), _e277.z)));
                let _e300 = (_e292 * ((max(max(_e140.x, _e140.y), _e140.z) - min(min(_e140.x, _e140.y), _e140.z)) / max(0.000062f, max(max(_e292.x, _e292.y), _e292.z))));
                let _e301 = dot(_e140, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e304 = (_e300 - vec3(dot(_e300, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e317 = (vec2<f32>(_e301, (1f - _e301)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e304.x, _e304.y), _e304.z)), max(max(_e304.x, _e304.y), _e304.z))));
                local_1 = ((_e304 * min(1f, min(_e317.x, _e317.y))) + vec3(_e301));
            }
            break;
        }
        case 13: {
            if Kh {
                let _e325 = local_2;
                let _e326 = clamp(_e325, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e326;
                let _e341 = (_e140 - vec3(min(min(_e140.x, _e140.y), _e140.z)));
                let _e349 = (_e341 * ((max(max(_e326.x, _e326.y), _e326.z) - min(min(_e326.x, _e326.y), _e326.z)) / max(0.000062f, max(max(_e341.x, _e341.y), _e341.z))));
                let _e350 = dot(_e140, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e353 = (_e349 - vec3(dot(_e349, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e366 = (vec2<f32>(_e350, (1f - _e350)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e353.x, _e353.y), _e353.z)), max(max(_e353.x, _e353.y), _e353.z))));
                local_1 = ((_e353 * min(1f, min(_e366.x, _e366.y))) + vec3(_e350));
            }
            break;
        }
        case 14: {
            if Kh {
                let _e374 = local_2;
                let _e375 = clamp(_e374, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e375;
                let _e376 = dot(_e140, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e379 = (_e375 - vec3(dot(_e375, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e392 = (vec2<f32>(_e376, (1f - _e376)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e379.x, _e379.y), _e379.z)), max(max(_e379.x, _e379.y), _e379.z))));
                local_1 = ((_e379 * min(1f, min(_e392.x, _e392.y))) + vec3(_e376));
            }
            break;
        }
        case 15: {
            if Kh {
                let _e400 = local_2;
                let _e401 = clamp(_e400, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e401;
                let _e402 = dot(_e401, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e405 = (_e140 - vec3(dot(_e140, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e418 = (vec2<f32>(_e402, (1f - _e402)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e405.x, _e405.y), _e405.z)), max(max(_e405.x, _e405.y), _e405.z))));
                local_1 = ((_e405 * min(1f, min(_e418.x, _e418.y))) + vec3(_e402));
            }
            break;
        }
        default: {
        }
    }
    let _e426 = local_1;
    let _e430 = (mix(_e133, _e426, vec3(_e132.w)) * _e127.w);
    let _e436 = vec4<f32>(_e430.x, _e127.y, _e127.z, _e127.w);
    let _e442 = vec4<f32>(_e436.x, _e430.y, _e436.z, _e436.w);
    let _e449 = (vec4<f32>(_e442.x, _e442.y, _e430.z, _e442.w) * 1f);
    let _e450 = _e449.xyz;
    let _e452 = gl_FragCoord_1;
    let _e454 = j.F3_;
    let _e456 = j.G3_;
    if (Lh && (_e449.w != 0f)) {
        phi_3218_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e452.x) + (0.00583715f * _e452.y))))) * _e454) + _e456)) + _e450);
    } else {
        phi_3218_ = _e450;
    }
    let _e472 = phi_3218_;
    let _e478 = vec4<f32>(_e472.x, _e449.y, _e449.z, _e449.w);
    let _e484 = vec4<f32>(_e478.x, _e472.y, _e478.z, _e478.w);
    lh = vec4<f32>(_e484.x, _e484.y, _e472.z, _e484.w);
    return;
}

@fragment
fn main(@location(9) C2_: vec3<f32>, @location(6) @interpolate(flat, either) g1_: f32, @location(0) X1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) Y1_: vec2<f32>) -> @location(0) vec4<f32> {
    C2_1 = C2_;
    g1_1 = g1_;
    X1_1 = X1_;
    gl_FragCoord_1 = gl_FragCoord;
    Y1_1 = Y1_;
    main_1();
    let _e11 = lh;
    return _e11;
}
