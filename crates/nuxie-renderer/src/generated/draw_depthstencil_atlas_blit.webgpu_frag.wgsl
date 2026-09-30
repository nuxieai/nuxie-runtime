struct TB {
    tc: f32,
    Bd: f32,
    Hf: f32,
    If: f32,
    o6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    T7_: vec4<i32>,
    hh: vec2<f32>,
    Cd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    X2_: f32,
    Dd: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Ed: f32,
    eh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

@id(7) override Oh: bool = true;
@id(6) override Nh: bool = true;
@id(2) override Jh: bool = true;
@id(8) override Ph: bool = true;

@group(0) @binding(0)
var<uniform> j: TB;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var W5_: sampler;
@group(0) @binding(10)
var FD: texture_2d<f32>;
@group(3) @binding(10)
var S9_: sampler;
var<private> G2_1: vec2<f32>;
var<private> D2_1: vec3<f32>;
var<private> f1_1: f32;
var<private> X1_1: vec4<f32>;
@group(0) @binding(12)
var YD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> oh: vec4<f32>;
@group(3) @binding(9)
var ca: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> N3_1: f32;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2889_: f32;
    var phi_2890_: f32;
    var phi_2904_: vec4<f32>;
    var phi_2903_: vec4<f32>;
    var phi_1082_: bool;
    var phi_2891_: f32;
    var phi_2900_: vec4<f32>;
    var phi_2906_: vec4<f32>;
    var phi_2907_: f32;
    var phi_3202_: vec4<f32>;
    var phi_3162_: i32;
    var phi_3305_: vec3<f32>;

    let _e57 = G2_1;
    let _e58 = textureSampleLevel(FD, S9_, _e57, 0f);
    let _e61 = f1_1;
    let _e62 = u32(_e61);
    let _e63 = D2_1;
    let _e64 = X1_1;
    let _e66 = (Jh && (_e62 != 0u));
    if (_e64.w >= 0f) {
        phi_2903_ = _e64;
    } else {
        let _e69 = -(_e64.w);
        let _e74 = j.Zb;
        let _e77 = j.ac;
        if (_e64.z > 0f) {
            phi_2889_ = _e64.x;
        } else {
            phi_2889_ = length(_e64.xy);
        }
        let _e85 = phi_2889_;
        let _e86 = clamp(_e85, 0f, 1f);
        let _e87 = abs(_e64.z);
        if (_e87 > 1f) {
            phi_2890_ = ((0.9980469f * _e86) + 0.0009765625f);
        } else {
            phi_2890_ = ((0.001953125f * _e86) + _e87);
        }
        let _e94 = phi_2890_;
        let _e96 = textureSampleLevel(ED, N9_, vec2<f32>(_e94, ((floor(_e69) * _e74) + _e77)), 0f);
        phi_2904_ = _e96;
        if !(_e66) {
            let _e100 = (_e96.xyz * _e96.w);
            phi_2904_ = vec4<f32>(_e100.x, _e100.y, _e100.z, (_e96.w * (fract(_e69) * 1.0039216f)));
        }
        let _e107 = phi_2904_;
        phi_2903_ = _e107;
    }
    let _e109 = phi_2903_;
    phi_1082_ = Ph;
    if Ph {
        phi_1082_ = (_e63.z > 0f);
    }
    let _e113 = phi_1082_;
    phi_2906_ = _e109;
    if _e113 {
        let _e117 = textureSampleLevel(HC, W5_, _e63.xy, (_e63.z - 1f));
        phi_2900_ = _e117;
        if _e66 {
            if (_e117.w != 0f) {
                phi_2891_ = (1f / _e117.w);
            } else {
                phi_2891_ = 0f;
            }
            let _e123 = phi_2891_;
            let _e124 = (_e117.xyz * _e123);
            phi_2900_ = vec4<f32>(_e124.x, _e124.y, _e124.z, _e117.w);
        }
        let _e130 = phi_2900_;
        phi_2906_ = (_e109 * _e130);
    }
    let _e133 = phi_2906_;
    let _e134 = gl_FragCoord_1;
    let _e138 = textureLoad(YD, vec2<i32>(floor(_e134.xy)), 0i);
    let _e139 = _e133.xyz;
    local_2 = _e139;
    let _e140 = _e138.xyz;
    if (_e138.w != 0f) {
        phi_2907_ = (1f / _e138.w);
    } else {
        phi_2907_ = 0f;
    }
    let _e145 = phi_2907_;
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
            phi_3202_ = vec4<f32>(_e211.x, _e211.y, _e199.z, _e211.w);
            if (_e138.w == 0f) {
                phi_3202_ = vec4<f32>(_e199.x, _e199.y, _e199.z, 1f);
            }
            let _e221 = phi_3202_;
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
            phi_3162_ = 0i;
            loop {
                let _e243 = phi_3162_;
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
                    phi_3162_ = (_e243 + 1i);
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
            if Nh {
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
            if Nh {
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
            if Nh {
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
            if Nh {
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
    let _e460 = j.F3_;
    let _e462 = j.G3_;
    if (Oh && (_e455.w != 0f)) {
        phi_3305_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e458.x) + (0.00583715f * _e458.y))))) * _e460) + _e462)) + _e456);
    } else {
        phi_3305_ = _e456;
    }
    let _e478 = phi_3305_;
    let _e484 = vec4<f32>(_e478.x, _e455.y, _e455.z, _e455.w);
    let _e490 = vec4<f32>(_e484.x, _e478.y, _e484.z, _e484.w);
    oh = vec4<f32>(_e490.x, _e490.y, _e478.z, _e490.w);
    return;
}

@fragment
fn main(@location(1) G2_: vec2<f32>, @location(9) D2_: vec3<f32>, @location(6) @interpolate(flat, either) f1_: f32, @location(0) X1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) N3_: f32) -> @location(0) vec4<f32> {
    G2_1 = G2_;
    D2_1 = D2_;
    f1_1 = f1_;
    X1_1 = X1_;
    gl_FragCoord_1 = gl_FragCoord;
    N3_1 = N3_;
    main_1();
    let _e13 = oh;
    return _e13;
}
