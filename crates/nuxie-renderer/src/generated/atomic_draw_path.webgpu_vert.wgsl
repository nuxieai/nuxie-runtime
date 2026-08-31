struct dg {
    c2_: array<vec4<u32>>,
}

struct cg {
    c2_: array<vec4<u32>>,
}

struct CC {
    ec: f32,
    od: f32,
    ff: f32,
    gf: f32,
    m6_: u32,
    Gg: u32,
    Re: u32,
    Se: u32,
    R7_: vec4<i32>,
    Cg: vec2<f32>,
    pd: vec2<f32>,
    a2_: u32,
    Hg: f32,
    Z5_: u32,
    P2_: f32,
    qd: f32,
    Me: u32,
    z3_: f32,
    A3_: f32,
    rd: f32,
    zg: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Je {
    c2_: array<vec2<u32>>,
}

struct Ke {
    c2_: array<vec4<f32>>,
}

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @location(1) @interpolate(flat, either) member_1: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(7)
var LC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> ED: dg;
@group(0) @binding(2)
var<storage> PB: cg;
@group(0) @binding(0)
var<uniform> n: CC;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> UB_1: vec4<f32>;
var<private> VB_1: vec4<f32>;
var<private> O: vec4<f32>;
var<private> B0_: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(3)
var<storage> AD: Je;
@group(0) @binding(4)
var<storage> RB: Ke;
@group(3) @binding(9)
var aa: sampler;

fn main_1() {
    var phi_2304_: f32;
    var phi_2242_: f32;
    var phi_2214_: i32;
    var phi_1352_: bool;
    var phi_2227_: i32;
    var phi_2219_: vec4<u32>;
    var phi_2226_: i32;
    var phi_2218_: vec4<u32>;
    var phi_2225_: i32;
    var phi_2223_: vec4<u32>;
    var phi_2222_: u32;
    var phi_2229_: vec2<i32>;
    var phi_2230_: vec4<u32>;
    var phi_2234_: f32;
    var phi_2314_: f32;
    var phi_2248_: f32;
    var phi_2313_: f32;
    var phi_2256_: f32;
    var phi_2249_: f32;
    var phi_2246_: f32;
    var phi_2260_: f32;
    var phi_2335_: f32;
    var phi_2326_: f32;
    var phi_2311_: f32;
    var phi_2259_: f32;
    var phi_2309_: f32;
    var phi_2392_: f32;
    var phi_2403_: f32;
    var phi_2395_: f32;
    var phi_2484_: f32;
    var phi_2441_: i32;
    var phi_2450_: f32;
    var phi_1691_: bool;
    var phi_2457_: f32;
    var phi_2473_: vec2<f32>;
    var phi_2472_: vec2<f32>;
    var phi_2494_: vec4<f32>;
    var phi_2509_: vec2<f32>;
    var phi_2493_: vec4<f32>;
    var phi_2540_: vec4<f32>;
    var phi_2345_: f32;
    var phi_2344_: f32;
    var phi_2346_: f32;
    var phi_2350_: f32;
    var phi_2372_: f32;
    var phi_2370_: f32;
    var phi_2388_: vec4<f32>;
    var phi_2538_: vec2<f32>;
    var phi_2387_: vec4<f32>;
    var phi_2542_: vec4<f32>;
    var phi_2539_: vec4<f32>;
    var phi_2535_: vec2<f32>;
    var phi_2511_: vec2<f32>;
    var phi_2582_: vec4<f32>;
    var phi_2544_: vec2<f32>;
    var phi_2543_: bool;
    var local: u32;
    var phi_2583_: vec4<f32>;

    let _e70 = gl_InstanceIndex_1;
    let _e71 = UB_1;
    let _e72 = VB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e75 = i32(_e71.x);
            let _e79 = bitcast<i32>(_e71.w);
            let _e81 = (_e79 >> bitcast<u32>(2i));
            let _e82 = (_e79 & 3i);
            let _e84 = min(_e75, (_e81 - 1i));
            let _e86 = ((_e70 * _e81) + _e84);
            let _e91 = textureLoad(LC, vec2<i32>((_e86 & 2047i), (_e86 >> bitcast<u32>(11i))), 0i);
            let _e98 = ED.c2_[(max((_e91.w & 65535u), 1u) - 1u)];
            let _e100 = bitcast<vec2<f32>>(_e98.xy);
            let _e102 = (_e98.z & 65535u);
            let _e104 = (_e102 * 4u);
            let _e107 = PB.c2_[_e104];
            let _e108 = bitcast<vec4<f32>>(_e107);
            let _e115 = mat2x2<f32>(vec2<f32>(_e108.x, _e108.y), vec2<f32>(_e108.z, _e108.w));
            let _e119 = PB.c2_[(_e104 + 1u)];
            let _e123 = bitcast<f32>(_e119.z);
            let _e125 = bitcast<f32>(_e119.w);
            let _e126 = (_e91.w & 8388608u);
            phi_2304_ = _e71.z;
            phi_2242_ = _e71.y;
            phi_2214_ = _e75;
            local = _e102;
            if (_e126 != 0u) {
                phi_2304_ = _e72.z;
                phi_2242_ = _e72.y;
                phi_2214_ = i32(_e72.x);
            }
            let _e133 = phi_2304_;
            let _e135 = phi_2242_;
            let _e137 = phi_2214_;
            phi_2225_ = _e86;
            phi_2223_ = _e91;
            phi_2222_ = _e91.w;
            if (_e137 != _e84) {
                let _e140 = ((_e86 + _e137) - _e84);
                let _e145 = textureLoad(LC, vec2<i32>((_e140 & 2047i), (_e140 >> bitcast<u32>(11i))), 0i);
                if ((_e145.w & 8454143u) != (_e91.w & 8454143u)) {
                    let _e150 = (_e123 == 0f);
                    phi_1352_ = _e150;
                    if !(_e150) {
                        phi_1352_ = (_e100.x != 0f);
                    }
                    let _e155 = phi_1352_;
                    phi_2227_ = _e86;
                    phi_2219_ = _e91;
                    if _e155 {
                        let _e156 = bitcast<i32>(_e98.w);
                        let _e161 = textureLoad(LC, vec2<i32>((_e156 & 2047i), (_e156 >> bitcast<u32>(11i))), 0i);
                        phi_2227_ = _e156;
                        phi_2219_ = _e161;
                    }
                    let _e163 = phi_2227_;
                    let _e165 = phi_2219_;
                    phi_2226_ = _e163;
                    phi_2218_ = _e165;
                } else {
                    phi_2226_ = _e140;
                    phi_2218_ = _e145;
                }
                let _e167 = phi_2226_;
                let _e169 = phi_2218_;
                phi_2225_ = _e167;
                phi_2223_ = _e169;
                phi_2222_ = ((_e169.w & 4286578687u) | _e126);
            }
            let _e174 = phi_2225_;
            let _e176 = phi_2223_;
            let _e178 = phi_2222_;
            let _e179 = (_e178 & 469762048u);
            let _e182 = ((_e179 == 67108864u) && (_e82 == 0i));
            if _e182 {
                let _e185 = f32((_e176.z & 65535u));
                let _e188 = f32((_e176.z >> bitcast<u32>(16i)));
                let _e194 = vec2<i32>(i32((-1f - _e185)), i32(((_e188 - _e185) + 1f)));
                phi_2229_ = _e194;
                if ((_e178 & 8388608u) != 0u) {
                    phi_2229_ = -(_e194);
                }
                let _e199 = phi_2229_;
                let _e201 = (_e174 + _e199.x);
                let _e206 = textureLoad(LC, vec2<i32>((_e201 & 2047i), (_e201 >> bitcast<u32>(11i))), 0i);
                let _e208 = (_e174 + _e199.y);
                let _e213 = textureLoad(LC, vec2<i32>((_e208 & 2047i), (_e208 >> bitcast<u32>(11i))), 0i);
                phi_2230_ = _e213;
                if ((_e213.w & 8454143u) != (_e206.w & 8454143u)) {
                    let _e219 = bitcast<i32>(_e98.w);
                    let _e224 = textureLoad(LC, vec2<i32>((_e219 & 2047i), (_e219 >> bitcast<u32>(11i))), 0i);
                    phi_2230_ = _e224;
                }
                let _e226 = phi_2230_;
                let _e228 = bitcast<f32>(_e206.z);
                let _e230 = bitcast<f32>(_e226.z);
                let _e231 = (_e230 - _e228);
                phi_2234_ = _e231;
                if (abs(_e231) > 3.1415927f) {
                    phi_2234_ = (_e231 - (6.2831855f * sign(_e231)));
                }
                let _e238 = phi_2234_;
                let _e239 = (_e188 + -2f);
                let _e245 = clamp(round(((abs(_e238) * 0.31830987f) * _e239)), 1f, (_e188 + -3f));
                let _e246 = (_e239 - _e245);
                if (_e185 <= _e246) {
                    phi_2314_ = _e135;
                    if (_e185 == _e246) {
                        phi_2314_ = -(_e135);
                    }
                    let _e255 = phi_2314_;
                    phi_2313_ = _e255;
                    phi_2256_ = -(((3.1415927f * sign(_e238)) - _e238));
                    phi_2249_ = _e246;
                    phi_2246_ = _e185;
                } else {
                    let _e257 = (_e185 == (_e246 + 1f));
                    if _e257 {
                        phi_2248_ = 0f;
                    } else {
                        phi_2248_ = (_e185 - (_e246 + 2f));
                    }
                    let _e261 = phi_2248_;
                    phi_2313_ = select(_e135, 0f, _e257);
                    phi_2256_ = _e238;
                    phi_2249_ = select(_e245, 0f, _e257);
                    phi_2246_ = _e261;
                }
                let _e265 = phi_2313_;
                let _e267 = phi_2256_;
                let _e269 = phi_2249_;
                let _e271 = phi_2246_;
                if (_e271 == _e269) {
                    phi_2260_ = _e230;
                } else {
                    phi_2260_ = (_e228 + (_e267 * (_e271 / _e269)));
                }
                let _e277 = phi_2260_;
                phi_2335_ = _e228;
                phi_2326_ = _e267;
                phi_2311_ = _e265;
                phi_2259_ = _e277;
            } else {
                phi_2335_ = f32();
                phi_2326_ = f32();
                phi_2311_ = _e135;
                phi_2259_ = bitcast<f32>(_e176.z);
            }
            let _e281 = phi_2335_;
            let _e283 = phi_2326_;
            let _e285 = phi_2311_;
            let _e287 = phi_2259_;
            let _e291 = vec2<f32>(sin(_e287), -(cos(_e287)));
            let _e293 = bitcast<vec2<f32>>(_e176.xy);
            phi_2309_ = _e125;
            if (_e125 != 0f) {
                phi_2309_ = max(_e125, (1f / length((_e115 * _e291))));
            }
            let _e300 = phi_2309_;
            if (_e123 != 0f) {
                let _e304 = (_e285 * sign(determinant(_e115)));
                let _e306 = ((_e178 & 1048576u) != 0u);
                phi_2392_ = _e304;
                if _e306 {
                    phi_2392_ = min(_e304, 0f);
                }
                let _e309 = phi_2392_;
                phi_2403_ = _e309;
                if ((_e178 & 524288u) != 0u) {
                    phi_2403_ = max(_e309, 0f);
                }
                let _e314 = phi_2403_;
                let _e315 = (_e300 != 0f);
                if _e315 {
                    phi_2395_ = _e300;
                } else {
                    let _e316 = (_e115 * _e291);
                    phi_2395_ = (((abs(_e316.x) + abs(_e316.y)) * (1f / dot(_e316, _e316))) * 0.5f);
                }
                let _e327 = phi_2395_;
                let _e330 = ((_e327 > _e123) && (_e300 == 0f));
                phi_2484_ = 1f;
                if _e330 {
                    phi_2484_ = (_e123 / _e327);
                }
                let _e333 = phi_2484_;
                let _e334 = select(_e123, _e327, _e330);
                let _e335 = (_e334 + _e327);
                let _e336 = (_e291 * _e335);
                let _e337 = (_e314 * _e335);
                let _e344 = (((vec2<f32>(_e337, -(_e337)) + vec2(_e334)) * (0.5f / _e327)) + vec2<f32>(0.5f, 0.5f));
                let _e347 = vec4<f32>(_e344.x, _e344.y, 0f, 0f);
                phi_2509_ = _e336;
                phi_2493_ = _e347;
                if (_e179 > 134217728u) {
                    let _e349 = (_e178 & 4194304u);
                    let _e351 = select(2i, -2i, (_e349 == 0u));
                    phi_2441_ = _e351;
                    if ((_e178 & 8388608u) != 0u) {
                        phi_2441_ = -(_e351);
                    }
                    let _e356 = phi_2441_;
                    let _e357 = (_e174 + _e356);
                    let _e362 = textureLoad(LC, vec2<i32>((_e357 & 2047i), (_e357 >> bitcast<u32>(11i))), 0i);
                    let _e366 = abs((bitcast<f32>(_e362.z) - _e287));
                    phi_2450_ = _e366;
                    if (_e366 > 3.1415927f) {
                        phi_2450_ = (6.2831855f - _e366);
                    }
                    let _e370 = phi_2450_;
                    let _e375 = ((_e370 * select(0.5f, -0.5f, ((_e349 != 0u) == _e306))) + _e287);
                    let _e379 = vec2<f32>(sin(_e375), -(cos(_e375)));
                    let _e380 = (_e115 * _e379);
                    let _e388 = ((abs(_e380.x) + abs(_e380.y)) * (1f / dot(_e380, _e380)));
                    let _e390 = cos((_e370 * 0.5f));
                    let _e391 = (_e179 == 335544320u);
                    phi_1691_ = _e391;
                    if !(_e391) {
                        phi_1691_ = ((_e179 == 268435456u) && (_e390 >= 0.25f));
                    }
                    let _e397 = phi_1691_;
                    if _e397 {
                        phi_2457_ = (_e334 * (1f / max(_e390, select(0.25f, 1f, ((_e178 & 33554432u) != 0u)))));
                    } else {
                        phi_2457_ = ((_e334 * _e390) + (_e388 * 0.5f));
                    }
                    let _e408 = phi_2457_;
                    let _e410 = (_e408 + (_e388 * 0.5f));
                    phi_2472_ = _e336;
                    if ((_e178 & 2097152u) != 0u) {
                        if (_e335 <= ((_e410 * _e390) + (_e327 * 0.125f))) {
                            phi_2473_ = (_e379 * (_e335 * (1f / _e390)));
                        } else {
                            let _e420 = (_e379 * _e410);
                            phi_2473_ = (vec2<f32>(dot(_e336, _e336), dot(_e420, _e420)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e336, _e420)));
                        }
                        let _e428 = phi_2473_;
                        phi_2472_ = _e428;
                    }
                    let _e430 = phi_2472_;
                    let _e435 = ((_e410 - dot((_e430 * abs(_e314)), _e379)) / _e388);
                    if _e306 {
                        phi_2494_ = vec4<f32>(_e347.x, _e435, _e347.z, _e347.w);
                    } else {
                        phi_2494_ = vec4<f32>(_e435, _e347.y, _e347.z, _e347.w);
                    }
                    let _e447 = phi_2494_;
                    phi_2509_ = _e430;
                    phi_2493_ = _e447;
                }
                let _e449 = phi_2509_;
                let _e451 = phi_2493_;
                let _e453 = (_e451.xy * _e333);
                let _e459 = vec4<f32>(_e453.x, _e451.y, _e451.z, _e451.w);
                let _e466 = vec4<f32>(_e459.x, max(_e453.y, 0.0001f), _e459.z, _e459.w);
                phi_2540_ = _e466;
                if _e315 {
                    phi_2540_ = vec4<f32>((-2f - _e453.x), _e466.y, _e466.z, _e466.w);
                }
                let _e474 = phi_2540_;
                if (_e82 != 0i) {
                    phi_2582_ = _e474;
                    phi_2544_ = vec2<f32>();
                    phi_2543_ = false;
                    break;
                }
                phi_2539_ = _e474;
                phi_2535_ = (_e115 * (_e449 * _e314));
                phi_2511_ = _e293;
            } else {
                let _e478 = vec4<f32>(_e133, -1f, 0f, 0f);
                if (_e300 != 0f) {
                    let _e484 = vec4<f32>(_e478.x, -2f, _e478.z, _e478.w);
                    let _e489 = vec4<f32>(_e484.x, _e484.y, 1000000f, _e484.w);
                    phi_2388_ = vec4<f32>(_e489.x, _e489.y, _e489.z, _e133);
                    if _e182 {
                        phi_2345_ = _e283;
                        phi_2344_ = _e281;
                        if (_e283 < 0f) {
                            phi_2345_ = -(_e283);
                            phi_2344_ = (_e281 + _e283);
                        }
                        let _e499 = phi_2345_;
                        let _e501 = phi_2344_;
                        let _e503 = ((_e287 - _e501) + 1.5707964f);
                        let _e509 = clamp(((_e503 - (floor((_e503 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e499);
                        phi_2346_ = _e509;
                        if (_e509 > (_e499 * 0.5f)) {
                            phi_2346_ = (_e499 - _e509);
                        }
                        let _e514 = phi_2346_;
                        let _e521 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e514), cos(_e514)) * abs(_e285))) * 0.5f);
                        if (abs((_e499 - 1.5707964f)) < 0.001f) {
                            phi_2372_ = 0f;
                            phi_2370_ = 0f;
                        } else {
                            let _e525 = tan(_e499);
                            let _e530 = (sign((1.5707964f - _e499)) / max(abs(_e525), 0.000001f));
                            if (_e530 >= 0f) {
                                phi_2350_ = (_e521.y - ((1f - _e521.x) * _e525));
                            } else {
                                phi_2350_ = (_e521.y + (_e521.x * _e525));
                            }
                            let _e542 = phi_2350_;
                            phi_2372_ = _e542;
                            phi_2370_ = _e530;
                        }
                        let _e544 = phi_2372_;
                        let _e546 = phi_2370_;
                        phi_2388_ = vec4<f32>((max(_e521.x, 0f) + 0.25f), (-2f - _e521.y), _e546, _e544);
                    }
                    let _e554 = phi_2388_;
                    phi_2538_ = (_e115 * (_e291 * (_e285 * _e300)));
                    phi_2387_ = _e554;
                } else {
                    phi_2538_ = (sign(((_e291 * _e285) * _naga_inverse_2x2_f32(_e115))) * 0.5f);
                    phi_2387_ = _e478;
                }
                let _e564 = phi_2538_;
                let _e566 = phi_2387_;
                phi_2542_ = _e566;
                if (((_e178 & 8388608u) != 0u) != ((_e178 & 16777216u) != 0u)) {
                    phi_2542_ = (_e566 * vec4<f32>(-1f, 1f, 1f, 1f));
                }
                let _e574 = phi_2542_;
                if (((_e178 & 2147483648u) != 0u) && (_e82 != 1i)) {
                    phi_2582_ = _e574;
                    phi_2544_ = vec2<f32>();
                    phi_2543_ = false;
                    break;
                }
                phi_2539_ = _e574;
                phi_2535_ = _e564;
                phi_2511_ = select(_e293, _e100, vec2((_e82 == 2i)));
            }
            let _e583 = phi_2539_;
            let _e585 = phi_2535_;
            let _e587 = phi_2511_;
            let _e593 = n.zg;
            let _e596 = select(_e583.xy, vec2<f32>(1f, -1f), vec2((_e593 != 0u)));
            let _e602 = vec4<f32>(_e596.x, _e583.y, _e583.z, _e583.w);
            phi_2582_ = vec4<f32>(_e602.x, _e596.y, _e602.z, _e602.w);
            phi_2544_ = (((_e115 * _e587) + _e585) + bitcast<vec2<f32>>(_e119.xy));
            phi_2543_ = true;
            break;
        }
    }
    let _e610 = phi_2582_;
    let _e612 = phi_2544_;
    let _e614 = phi_2543_;
    if _e614 {
        O = _e610;
        let _e616 = local;
        B0_ = _e616;
        let _e618 = n.ff;
        let _e620 = n.gf;
        phi_2583_ = vec4<f32>(((_e612.x * _e618) - 1f), ((_e612.y * _e620) - sign(_e620)), 0f, 1f);
    } else {
        let _e630 = n.P2_;
        phi_2583_ = vec4(_e630);
    }
    let _e633 = phi_2583_;
    unnamed.gl_Position = _e633;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) UB: vec4<f32>, @location(1) VB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    UB_1 = UB;
    VB_1 = VB;
    main_1();
    let _e14 = O;
    let _e15 = B0_;
    let _e16 = unnamed.gl_Position;
    return VertexOutput(_e14, _e15, _e16);
}

fn _naga_inverse_2x2_f32(m: mat2x2<f32>) -> mat2x2<f32> {
    var adj: mat2x2<f32>;
    adj[0][0] = m[1][1];
    adj[0][1] = -m[0][1];
    adj[1][0] = -m[1][0];
    adj[1][1] = m[0][0];

    let det: f32 = m[0][0] * m[1][1] - m[1][0] * m[0][1];
    return adj * (1 / det);
}
