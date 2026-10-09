struct jg {
    r2_: array<vec2<u32>>,
}

struct m0Pe {
    r2_: array<u32>,
}

struct kg {
    r2_: array<vec4<f32>>,
}

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

struct n0Pe {
    r2_: array<u32>,
}

struct R4Pe {
    r2_: array<u32>,
}

struct R4Pe_1 {
    r2_: array<atomic<u32>>,
}

@id(7) override dj: bool = true;
@id(6) override cj: bool = true;
@id(4) override aj: bool = true;
@id(0) override Wi: bool = true;
@id(1) override Xi: bool = true;
@id(2) override Yi: bool = true;
@id(3) override Zi: bool = true;

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(3)
var<storage> VC: jg;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Pe;
@group(0) @binding(4)
var<storage> JB: kg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var N8_: sampler;
@group(2) @binding(0)
var<storage, read_write> n0_: n0Pe;
var<private> S_1: vec4<f32>;
var<private> G0_1: u32;
@group(2) @binding(3)
var<storage, read_write> R4_: R4Pe_1;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1537_: bool;
    var phi_1550_: bool;
    var phi_4195_: f32;
    var phi_4203_: f32;
    var phi_4211_: f32;
    var phi_4210_: f32;
    var phi_2050_: bool;
    var phi_4214_: f32;
    var phi_4213_: f32;
    var phi_4215_: f32;
    var phi_4218_: f32;
    var phi_4217_: f32;
    var phi_2087_: bool;
    var phi_4220_: f32;
    var phi_5080_: u32;
    var phi_4219_: f32;
    var phi_4252_: vec4<f32>;
    var phi_5079_: u32;
    var phi_4250_: vec4<f32>;
    var phi_4257_: f32;
    var phi_4871_: vec4<f32>;
    var phi_4791_: i32;
    var phi_5064_: vec4<f32>;
    var phi_5077_: vec4<f32>;
    var phi_5109_: u32;
    var phi_5102_: vec4<f32>;
    var phi_5104_: vec3<f32>;
    var phi_5106_: vec4<f32>;

    let _e97 = gl_FragCoord_1;
    let _e98 = _e97.xy;
    let _e101 = bitcast<vec2<u32>>(vec2<i32>(floor(_e98)));
    let _e103 = j.P6_;
    let _e132 = bitcast<i32>((((((_e101.y >> bitcast<u32>(5u)) * (((_e103 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e101.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e101.x & 28u) << bitcast<u32>(5u)) + ((_e101.y & 28u) << bitcast<u32>(2i)))) + (((_e101.y & 3u) << bitcast<u32>(2i)) + (_e101.x & 3u))));
    phi_1537_ = Zi;
    if Zi {
        let _e133 = S_1;
        phi_1537_ = (_e133.x < -1.5f);
    }
    let _e137 = phi_1537_;
    if _e137 {
        let _e138 = S_1;
        let _e142 = textureSampleLevel(YC, ab, vec2<f32>((3f + _e138.x), 0f), 0f);
        let _e148 = textureSampleLevel(YC, ab, vec2<f32>((1f - _e138.y), 0f), 0f);
        phi_4210_ = ((1f - _e142.x) - _e148.x);
    } else {
        phi_1550_ = Zi;
        if Zi {
            let _e151 = S_1;
            phi_1550_ = (_e151.y < -1.5f);
        }
        let _e155 = phi_1550_;
        if _e155 {
            let _e156 = S_1;
            let _e159 = max(_e156.w, 0f);
            if (_e156.z >= 0f) {
                let _e162 = textureSampleLevel(YC, ab, vec2<f32>(_e159, 0f), 0f);
                phi_4195_ = _e162.x;
            } else {
                phi_4195_ = 0f;
            }
            let _e165 = phi_4195_;
            phi_4203_ = _e165;
            if (abs(_e156.z) < 1000f) {
                let _e172 = (-2f - _e156.y);
                let _e174 = ((_e172 - _e159) * 0.5984134f);
                let _e177 = (vec4(_e159) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e174));
                let _e183 = ((_e177 * -(_e156.z)) + vec4(((_e172 * _e156.z) + (abs(_e156.x) - 0.25f))));
                let _e186 = textureSampleLevel(YC, ab, vec2<f32>(_e183.x, 0f), 0f);
                let _e189 = textureSampleLevel(YC, ab, vec2<f32>(_e183.y, 0f), 0f);
                let _e192 = textureSampleLevel(YC, ab, vec2<f32>(_e183.z, 0f), 0f);
                let _e195 = textureSampleLevel(YC, ab, vec2<f32>(_e183.w, 0f), 0f);
                let _e201 = (_e177 * 5.0959306f);
                phi_4203_ = (_e165 + (dot(vec4<f32>(_e186.x, _e189.x, _e192.x, _e195.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e201) * (_e201 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e174));
            }
            let _e210 = phi_4203_;
            phi_4211_ = (_e210 * sign(_e156.x));
        } else {
            let _e215 = S_1[0u];
            let _e217 = S_1[1u];
            phi_4211_ = min(min(_e215, abs(_e217)), 1f);
        }
        let _e222 = phi_4211_;
        phi_4210_ = _e222;
    }
    let _e224 = phi_4210_;
    let _e228 = u32(round(((_e224 * 2048f) + 65536f)));
    let _e229 = G0_1;
    let _e232 = ((_e229 << bitcast<u32>(17u)) | _e228);
    let _e235 = atomicMax((&R4_.r2_[_e132]), _e232);
    let _e237 = (_e235 >> bitcast<u32>(17u));
    if (_e237 == _e229) {
        let _e239 = S_1;
        if (_e239.y < 0f) {
            let _e246 = atomicAdd((&R4_.r2_[_e132]), ((_e228 + (_e235 - max(_e232, _e235))) - 65536u));
        }
        phi_5109_ = 0u;
        phi_5102_ = vec4<f32>(0f, 0f, 0f, 0f);
    } else {
        let _e250 = ((f32((_e235 & 131071u)) * 0.00048828125f) + -32f);
        let _e253 = VC.r2_[_e237];
        phi_4213_ = _e250;
        if ((_e253.x & 768u) != 0u) {
            let _e257 = abs(_e250);
            phi_2050_ = aj;
            if aj {
                phi_2050_ = ((_e253.x & 512u) != 0u);
            }
            let _e261 = phi_2050_;
            phi_4214_ = _e257;
            if _e261 {
                phi_4214_ = (1f - abs(((fract((_e257 * 0.5f)) * 2f) + -1f)));
            }
            let _e269 = phi_4214_;
            phi_4213_ = _e269;
        }
        let _e271 = phi_4213_;
        let _e272 = clamp(_e271, 0f, 1f);
        phi_4217_ = _e272;
        if Wi {
            let _e274 = (_e253.x >> bitcast<u32>(16u));
            phi_4218_ = _e272;
            if (_e274 != 0u) {
                let _e278 = m0_.r2_[_e132];
                if (_e274 == (_e278 >> bitcast<u32>(16i))) {
                    phi_4215_ = min(_e272, unpack2x16float(_e278).x);
                } else {
                    phi_4215_ = 0f;
                }
                let _e286 = phi_4215_;
                phi_4218_ = _e286;
            }
            let _e288 = phi_4218_;
            phi_4217_ = _e288;
        }
        let _e290 = phi_4217_;
        phi_2087_ = Xi;
        if Xi {
            phi_2087_ = ((_e253.x & 1024u) != 0u);
        }
        let _e294 = phi_2087_;
        phi_4220_ = _e290;
        if _e294 {
            let _e295 = (_e237 * 8u);
            let _e299 = JB.r2_[(_e295 + 2u)];
            let _e310 = JB.r2_[(_e295 + 3u)];
            let _e315 = _e310.zw;
            let _e317 = ((abs(((mat2x2<f32>(vec2<f32>(_e299.x, _e299.y), vec2<f32>(_e299.z, _e299.w)) * _e98) + _e310.xy)) * _e315) - _e315);
            phi_4220_ = min(_e290, clamp((min(_e317.x, _e317.y) + 0.5f), 0f, 1f));
        }
        let _e325 = phi_4220_;
        let _e326 = (_e253.x & 15u);
        let _e329 = ((_e253.x >> bitcast<u32>(4i)) & 15u);
        let _e331 = (Yi && (_e329 != 0u));
        if (_e326 <= 1u) {
            let _e336 = (Wi && (_e326 == 0u));
            phi_5080_ = 0u;
            if _e336 {
                phi_5080_ = (_e253.y | pack2x16float(vec2<f32>(_e325, 0f)));
            }
            let _e341 = phi_5080_;
            phi_5079_ = _e341;
            phi_4250_ = select(unpack4x8unorm(_e253.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e336));
        } else {
            let _e344 = (_e237 * 8u);
            let _e347 = JB.r2_[_e344];
            let _e358 = JB.r2_[(_e344 + 1u)];
            let _e361 = ((mat2x2<f32>(vec2<f32>(_e347.x, _e347.y), vec2<f32>(_e347.z, _e347.w)) * _e98) + _e358.xy);
            let _e367 = j.L8_;
            let _e369 = j.M8_;
            if (f32(_e326) == 2f) {
                phi_4219_ = _e361.x;
            } else {
                phi_4219_ = length(_e361);
            }
            let _e379 = phi_4219_;
            let _e385 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e379, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e358.z < 0f))) + ((max(0f, _e358.z) * 0.001953125f) + 0.0009765625f)), ((_e358.w * _e367) + _e369)), 0f);
            phi_4252_ = _e385;
            if !(_e331) {
                let _e389 = (_e385.xyz * _e385.w);
                phi_4252_ = vec4<f32>(_e389.x, _e389.y, _e389.z, (_e385.w * abs(bitcast<f32>(_e253.y))));
            }
            let _e399 = phi_4252_;
            phi_5079_ = 0u;
            phi_4250_ = _e399;
        }
        let _e401 = phi_5079_;
        let _e403 = phi_4250_;
        phi_5077_ = _e403;
        if _e331 {
            phi_5064_ = _e403;
            if ((_e403.w * _e325) != 0f) {
                let _e409 = n0_.r2_[_e132];
                let _e410 = unpack4x8unorm(_e409);
                let _e411 = _e403.xyz;
                local_2 = _e411;
                let _e412 = _e410.xyz;
                if (_e410.w != 0f) {
                    phi_4257_ = (1f / _e410.w);
                } else {
                    phi_4257_ = 0f;
                }
                let _e417 = phi_4257_;
                let _e418 = (_e412 * _e417);
                local = _e418;
                switch bitcast<i32>(_e329) {
                    case 11: {
                        let _e420 = local_2;
                        local_1 = (_e420 * _e418);
                        break;
                    }
                    case 1: {
                        let _e422 = local_2;
                        local_1 = ((_e422 + _e418) - (_e422 * _e418));
                        break;
                    }
                    case 2: {
                        let _e426 = local_2;
                        let _e427 = (_e426 * _e418);
                        local_1 = (select(_e427, (((_e426 + _e418) - _e427) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e418 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 3: {
                        let _e434 = local_2;
                        local_1 = min(_e434, _e418);
                        break;
                    }
                    case 4: {
                        let _e436 = local_2;
                        local_1 = max(_e436, _e418);
                        break;
                    }
                    case 5: {
                        let _e439 = clamp(_e412, vec3<f32>(0f, 0f, 0f), _e410.www);
                        let _e445 = vec4<f32>(_e439.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                        let _e451 = vec4<f32>(_e445.x, _e439.y, _e445.z, _e445.w);
                        let _e458 = local_2;
                        let _e461 = (clamp((vec3<f32>(1f, 1f, 1f) - _e458), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e410.w);
                        let _e462 = vec4<f32>(_e451.x, _e451.y, _e439.z, _e451.w).xyz;
                        local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e462 / _e461)), sign(_e462), (_e461 == vec3<f32>(0f, 0f, 0f)));
                        break;
                    }
                    case 6: {
                        let _e468 = local_2;
                        local_2 = clamp(_e468, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        let _e471 = clamp(_e412, vec3<f32>(0f, 0f, 0f), _e410.www);
                        let _e477 = vec4<f32>(_e471.x, _e410.y, _e410.z, _e410.w);
                        let _e483 = vec4<f32>(_e477.x, _e471.y, _e477.z, _e477.w);
                        phi_4871_ = vec4<f32>(_e483.x, _e483.y, _e471.z, _e483.w);
                        if (_e410.w == 0f) {
                            phi_4871_ = vec4<f32>(_e471.x, _e471.y, _e471.z, 1f);
                        }
                        let _e493 = phi_4871_;
                        let _e497 = (vec3(_e493.w) - _e493.xyz);
                        let _e498 = local_2;
                        local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e497 / (_e498 * _e493.w))), sign(_e497), (_e498 == vec3<f32>(0f, 0f, 0f))));
                        break;
                    }
                    case 7: {
                        let _e506 = local_2;
                        let _e507 = (_e506 * _e418);
                        local_1 = (select(_e507, (((_e506 + _e418) - _e507) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e506 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 8: {
                        phi_4791_ = 0i;
                        loop {
                            let _e515 = phi_4791_;
                            if (_e515 < 3i) {
                                let _e518 = local_2[_e515];
                                if (_e518 <= 0.5f) {
                                    let _e521 = local[_e515];
                                    local_1[_e515] = (1f - _e521);
                                } else {
                                    let _e525 = local[_e515];
                                    if (_e525 <= 0.25f) {
                                        let _e527 = local[_e515];
                                        let _e530 = local[_e515];
                                        local_1[_e515] = ((((16f * _e527) - 12f) * _e530) + 3f);
                                    } else {
                                        let _e534 = local[_e515];
                                        local_1[_e515] = (inverseSqrt(_e534) - 1f);
                                    }
                                }
                                continue;
                            } else {
                                break;
                            }
                            continuing {
                                phi_4791_ = (_e515 + 1i);
                            }
                        }
                        let _e539 = local_2;
                        let _e543 = local_1;
                        local_1 = (_e418 + ((_e418 * ((_e539 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e543));
                        break;
                    }
                    case 9: {
                        let _e546 = local_2;
                        local_1 = abs((_e418 - _e546));
                        break;
                    }
                    case 10: {
                        let _e549 = local_2;
                        local_1 = ((_e549 + _e418) - ((_e549 * 2f) * _e418));
                        break;
                    }
                    case 12: {
                        if cj {
                            let _e554 = local_2;
                            let _e555 = clamp(_e554, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e555;
                            let _e570 = (_e555 - vec3(min(min(_e555.x, _e555.y), _e555.z)));
                            let _e578 = (_e570 * ((max(max(_e418.x, _e418.y), _e418.z) - min(min(_e418.x, _e418.y), _e418.z)) / max(0.000062f, max(max(_e570.x, _e570.y), _e570.z))));
                            let _e579 = dot(_e418, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e582 = (_e578 - vec3(dot(_e578, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e595 = (vec2<f32>(_e579, (1f - _e579)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e582.x, _e582.y), _e582.z)), max(max(_e582.x, _e582.y), _e582.z))));
                            local_1 = ((_e582 * min(1f, min(_e595.x, _e595.y))) + vec3(_e579));
                        }
                        break;
                    }
                    case 13: {
                        if cj {
                            let _e603 = local_2;
                            let _e604 = clamp(_e603, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e604;
                            let _e619 = (_e418 - vec3(min(min(_e418.x, _e418.y), _e418.z)));
                            let _e627 = (_e619 * ((max(max(_e604.x, _e604.y), _e604.z) - min(min(_e604.x, _e604.y), _e604.z)) / max(0.000062f, max(max(_e619.x, _e619.y), _e619.z))));
                            let _e628 = dot(_e418, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e631 = (_e627 - vec3(dot(_e627, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e644 = (vec2<f32>(_e628, (1f - _e628)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e631.x, _e631.y), _e631.z)), max(max(_e631.x, _e631.y), _e631.z))));
                            local_1 = ((_e631 * min(1f, min(_e644.x, _e644.y))) + vec3(_e628));
                        }
                        break;
                    }
                    case 14: {
                        if cj {
                            let _e652 = local_2;
                            let _e653 = clamp(_e652, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e653;
                            let _e654 = dot(_e418, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e657 = (_e653 - vec3(dot(_e653, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e670 = (vec2<f32>(_e654, (1f - _e654)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e657.x, _e657.y), _e657.z)), max(max(_e657.x, _e657.y), _e657.z))));
                            local_1 = ((_e657 * min(1f, min(_e670.x, _e670.y))) + vec3(_e654));
                        }
                        break;
                    }
                    case 15: {
                        if cj {
                            let _e678 = local_2;
                            let _e679 = clamp(_e678, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e679;
                            let _e680 = dot(_e679, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e683 = (_e418 - vec3(dot(_e418, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e696 = (vec2<f32>(_e680, (1f - _e680)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e683.x, _e683.y), _e683.z)), max(max(_e683.x, _e683.y), _e683.z))));
                            local_1 = ((_e683 * min(1f, min(_e696.x, _e696.y))) + vec3(_e680));
                        }
                        break;
                    }
                    default: {
                    }
                }
                let _e704 = local_1;
                let _e706 = mix(_e411, _e704, vec3(_e410.w));
                let _e712 = vec4<f32>(_e706.x, _e403.y, _e403.z, _e403.w);
                let _e718 = vec4<f32>(_e712.x, _e706.y, _e712.z, _e712.w);
                phi_5064_ = vec4<f32>(_e718.x, _e718.y, _e706.z, _e718.w);
            }
            let _e726 = phi_5064_;
            let _e729 = (_e726.xyz * _e726.w);
            let _e735 = vec4<f32>(_e729.x, _e726.y, _e726.z, _e726.w);
            let _e741 = vec4<f32>(_e735.x, _e729.y, _e735.z, _e735.w);
            phi_5077_ = vec4<f32>(_e741.x, _e741.y, _e729.z, _e741.w);
        }
        let _e749 = phi_5077_;
        phi_5109_ = _e401;
        phi_5102_ = (_e749 * _e325);
    }
    let _e752 = phi_5109_;
    let _e754 = phi_5102_;
    let _e755 = _e754.xyz;
    let _e758 = j.F3_;
    let _e760 = j.G3_;
    if (dj && (_e754.w != 0f)) {
        phi_5104_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e97.x) + (0.00583715f * _e97.y))))) * _e758) + _e760)) + _e755);
    } else {
        phi_5104_ = _e755;
    }
    let _e776 = phi_5104_;
    let _e782 = vec4<f32>(_e776.x, _e754.y, _e754.z, _e754.w);
    let _e788 = vec4<f32>(_e782.x, _e776.y, _e782.z, _e782.w);
    let _e794 = vec4<f32>(_e788.x, _e788.y, _e776.z, _e788.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e776.x + _e776.y) + _e776.z) + _e754.w) == 0f) {
                break;
            }
            let _e800 = (1f - _e754.w);
            phi_5106_ = _e794;
            if (_e800 != 0f) {
                let _e804 = n0_.r2_[_e132];
                phi_5106_ = (_e794 + (unpack4x8unorm(_e804) * _e800));
            }
            let _e809 = phi_5106_;
            n0_.r2_[_e132] = pack4x8unorm(_e809);
            break;
        }
    }
    if (_e752 != 0u) {
        m0_.r2_[_e132] = _e752;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) S: vec4<f32>, @location(1) @interpolate(flat, either) G0_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    S_1 = S;
    G0_1 = G0_;
    main_1();
}
