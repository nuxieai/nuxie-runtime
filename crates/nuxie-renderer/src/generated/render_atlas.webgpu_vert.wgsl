struct Sf {
    X1_: array<vec4<u32>>,
}

struct Rf {
    X1_: array<vec4<u32>>,
}

struct NB {
    Ub: f32,
    dd: f32,
    Xe: f32,
    Ye: f32,
    q5_: u32,
    ug: u32,
    Je: u32,
    Ke: u32,
    U7_: vec4<i32>,
    rg: vec2<f32>,
    ed: vec2<f32>,
    W1_: u32,
    vg: f32,
    Y5_: u32,
    P2_: f32,
    fd: f32,
    Ee: u32,
    y3_: f32,
    z3_: f32,
    gd: f32,
    og: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Be {
    X1_: array<vec2<u32>>,
}

struct Ce {
    X1_: array<vec4<f32>>,
}

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(8) 
var DC: texture_2d<u32>;
@group(0) @binding(6) 
var<storage> XC: Sf;
@group(0) @binding(3) 
var<storage> MB: Rf;
@group(0) @binding(0) 
var<uniform> k: NB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> SB_1: vec4<f32>;
var<private> TB_1: vec4<f32>;
var<private> I: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(10) 
var QC: texture_2d<f32>;
@group(0) @binding(4) 
var<storage> TC: Be;
@group(0) @binding(5) 
var<storage> PB: Ce;
@group(3) @binding(10) 
var T9_: sampler;

fn main_1() {
    var phi_2290_: f32;
    var phi_2228_: f32;
    var phi_2200_: i32;
    var phi_1342_: bool;
    var phi_2213_: i32;
    var phi_2205_: vec4<u32>;
    var phi_2212_: i32;
    var phi_2204_: vec4<u32>;
    var phi_2211_: i32;
    var phi_2209_: vec4<u32>;
    var phi_2208_: u32;
    var phi_2215_: vec2<i32>;
    var phi_2216_: vec4<u32>;
    var phi_2220_: f32;
    var phi_2300_: f32;
    var phi_2234_: f32;
    var phi_2299_: f32;
    var phi_2242_: f32;
    var phi_2235_: f32;
    var phi_2232_: f32;
    var phi_2246_: f32;
    var phi_2321_: f32;
    var phi_2312_: f32;
    var phi_2297_: f32;
    var phi_2245_: f32;
    var phi_2295_: f32;
    var phi_2378_: f32;
    var phi_2389_: f32;
    var phi_2381_: f32;
    var phi_2470_: f32;
    var phi_2427_: i32;
    var phi_2436_: f32;
    var phi_1681_: bool;
    var phi_2443_: f32;
    var phi_2459_: vec2<f32>;
    var phi_2458_: vec2<f32>;
    var phi_2480_: vec4<f32>;
    var phi_2495_: vec2<f32>;
    var phi_2479_: vec4<f32>;
    var phi_2526_: vec4<f32>;
    var phi_2331_: f32;
    var phi_2330_: f32;
    var phi_2332_: f32;
    var phi_2336_: f32;
    var phi_2358_: f32;
    var phi_2356_: f32;
    var phi_2374_: vec4<f32>;
    var phi_2524_: vec2<f32>;
    var phi_2373_: vec4<f32>;
    var phi_2528_: vec4<f32>;
    var phi_2525_: vec4<f32>;
    var phi_2521_: vec2<f32>;
    var phi_2497_: vec2<f32>;
    var phi_2568_: vec4<f32>;
    var phi_2530_: vec2<f32>;
    var phi_2529_: bool;
    var local: u32;
    var phi_2569_: vec4<f32>;

    let _e71 = gl_InstanceIndex_1;
    let _e72 = SB_1;
    let _e73 = TB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e76 = i32(_e72.x);
            let _e80 = bitcast<i32>(_e72.w);
            let _e82 = (_e80 >> bitcast<u32>(2i));
            let _e83 = (_e80 & 3i);
            let _e85 = min(_e76, (_e82 - 1i));
            let _e87 = ((_e71 * _e82) + _e85);
            let _e92 = textureLoad(DC, vec2<i32>((_e87 & 2047i), (_e87 >> bitcast<u32>(11i))), 0i);
            let _e99 = XC.X1_[(max((_e92.w & 65535u), 1u) - 1u)];
            let _e101 = bitcast<vec2<f32>>(_e99.xy);
            let _e105 = ((_e99.z & 65535u) * 4u);
            let _e108 = MB.X1_[_e105];
            let _e109 = bitcast<vec4<f32>>(_e108);
            let _e116 = mat2x2<f32>(vec2<f32>(_e109.x, _e109.y), vec2<f32>(_e109.z, _e109.w));
            let _e120 = MB.X1_[(_e105 + 1u)];
            let _e124 = bitcast<f32>(_e120.z);
            let _e126 = bitcast<f32>(_e120.w);
            let _e127 = (_e92.w & 8388608u);
            phi_2290_ = _e72.z;
            phi_2228_ = _e72.y;
            phi_2200_ = _e76;
            local = _e105;
            if (_e127 != 0u) {
                phi_2290_ = _e73.z;
                phi_2228_ = _e73.y;
                phi_2200_ = i32(_e73.x);
            }
            let _e134 = phi_2290_;
            let _e136 = phi_2228_;
            let _e138 = phi_2200_;
            phi_2211_ = _e87;
            phi_2209_ = _e92;
            phi_2208_ = _e92.w;
            if (_e138 != _e85) {
                let _e141 = ((_e87 + _e138) - _e85);
                let _e146 = textureLoad(DC, vec2<i32>((_e141 & 2047i), (_e141 >> bitcast<u32>(11i))), 0i);
                if ((_e146.w & 8454143u) != (_e92.w & 8454143u)) {
                    let _e151 = (_e124 == 0f);
                    phi_1342_ = _e151;
                    if !(_e151) {
                        phi_1342_ = (_e101.x != 0f);
                    }
                    let _e156 = phi_1342_;
                    phi_2213_ = _e87;
                    phi_2205_ = _e92;
                    if _e156 {
                        let _e157 = bitcast<i32>(_e99.w);
                        let _e162 = textureLoad(DC, vec2<i32>((_e157 & 2047i), (_e157 >> bitcast<u32>(11i))), 0i);
                        phi_2213_ = _e157;
                        phi_2205_ = _e162;
                    }
                    let _e164 = phi_2213_;
                    let _e166 = phi_2205_;
                    phi_2212_ = _e164;
                    phi_2204_ = _e166;
                } else {
                    phi_2212_ = _e141;
                    phi_2204_ = _e146;
                }
                let _e168 = phi_2212_;
                let _e170 = phi_2204_;
                phi_2211_ = _e168;
                phi_2209_ = _e170;
                phi_2208_ = ((_e170.w & 4286578687u) | _e127);
            }
            let _e175 = phi_2211_;
            let _e177 = phi_2209_;
            let _e179 = phi_2208_;
            let _e180 = (_e179 & 469762048u);
            let _e183 = ((_e180 == 67108864u) && (_e83 == 0i));
            if _e183 {
                let _e186 = f32((_e177.z & 65535u));
                let _e189 = f32((_e177.z >> bitcast<u32>(16i)));
                let _e195 = vec2<i32>(i32((-1f - _e186)), i32(((_e189 - _e186) + 1f)));
                phi_2215_ = _e195;
                if ((_e179 & 8388608u) != 0u) {
                    phi_2215_ = -(_e195);
                }
                let _e200 = phi_2215_;
                let _e202 = (_e175 + _e200.x);
                let _e207 = textureLoad(DC, vec2<i32>((_e202 & 2047i), (_e202 >> bitcast<u32>(11i))), 0i);
                let _e209 = (_e175 + _e200.y);
                let _e214 = textureLoad(DC, vec2<i32>((_e209 & 2047i), (_e209 >> bitcast<u32>(11i))), 0i);
                phi_2216_ = _e214;
                if ((_e214.w & 8454143u) != (_e207.w & 8454143u)) {
                    let _e220 = bitcast<i32>(_e99.w);
                    let _e225 = textureLoad(DC, vec2<i32>((_e220 & 2047i), (_e220 >> bitcast<u32>(11i))), 0i);
                    phi_2216_ = _e225;
                }
                let _e227 = phi_2216_;
                let _e229 = bitcast<f32>(_e207.z);
                let _e231 = bitcast<f32>(_e227.z);
                let _e232 = (_e231 - _e229);
                phi_2220_ = _e232;
                if (abs(_e232) > 3.1415927f) {
                    phi_2220_ = (_e232 - (6.2831855f * sign(_e232)));
                }
                let _e239 = phi_2220_;
                let _e240 = (_e189 + -2f);
                let _e246 = clamp(round(((abs(_e239) * 0.31830987f) * _e240)), 1f, (_e189 + -3f));
                let _e247 = (_e240 - _e246);
                if (_e186 <= _e247) {
                    phi_2300_ = _e136;
                    if (_e186 == _e247) {
                        phi_2300_ = -(_e136);
                    }
                    let _e256 = phi_2300_;
                    phi_2299_ = _e256;
                    phi_2242_ = -(((3.1415927f * sign(_e239)) - _e239));
                    phi_2235_ = _e247;
                    phi_2232_ = _e186;
                } else {
                    let _e258 = (_e186 == (_e247 + 1f));
                    if _e258 {
                        phi_2234_ = 0f;
                    } else {
                        phi_2234_ = (_e186 - (_e247 + 2f));
                    }
                    let _e262 = phi_2234_;
                    phi_2299_ = select(_e136, 0f, _e258);
                    phi_2242_ = _e239;
                    phi_2235_ = select(_e246, 0f, _e258);
                    phi_2232_ = _e262;
                }
                let _e266 = phi_2299_;
                let _e268 = phi_2242_;
                let _e270 = phi_2235_;
                let _e272 = phi_2232_;
                if (_e272 == _e270) {
                    phi_2246_ = _e231;
                } else {
                    phi_2246_ = (_e229 + (_e268 * (_e272 / _e270)));
                }
                let _e278 = phi_2246_;
                phi_2321_ = _e229;
                phi_2312_ = _e268;
                phi_2297_ = _e266;
                phi_2245_ = _e278;
            } else {
                phi_2321_ = f32();
                phi_2312_ = f32();
                phi_2297_ = _e136;
                phi_2245_ = bitcast<f32>(_e177.z);
            }
            let _e282 = phi_2321_;
            let _e284 = phi_2312_;
            let _e286 = phi_2297_;
            let _e288 = phi_2245_;
            let _e292 = vec2<f32>(sin(_e288), -(cos(_e288)));
            let _e294 = bitcast<vec2<f32>>(_e177.xy);
            phi_2295_ = _e126;
            if (_e126 != 0f) {
                phi_2295_ = max(_e126, (1f / length((_e116 * _e292))));
            }
            let _e301 = phi_2295_;
            if (_e124 != 0f) {
                let _e305 = (_e286 * sign(determinant(_e116)));
                let _e307 = ((_e179 & 1048576u) != 0u);
                phi_2378_ = _e305;
                if _e307 {
                    phi_2378_ = min(_e305, 0f);
                }
                let _e310 = phi_2378_;
                phi_2389_ = _e310;
                if ((_e179 & 524288u) != 0u) {
                    phi_2389_ = max(_e310, 0f);
                }
                let _e315 = phi_2389_;
                let _e316 = (_e301 != 0f);
                if _e316 {
                    phi_2381_ = _e301;
                } else {
                    let _e317 = (_e116 * _e292);
                    phi_2381_ = (((abs(_e317.x) + abs(_e317.y)) * (1f / dot(_e317, _e317))) * 0.5f);
                }
                let _e328 = phi_2381_;
                let _e331 = ((_e328 > _e124) && (_e301 == 0f));
                phi_2470_ = 1f;
                if _e331 {
                    phi_2470_ = (_e124 / _e328);
                }
                let _e334 = phi_2470_;
                let _e335 = select(_e124, _e328, _e331);
                let _e336 = (_e335 + _e328);
                let _e337 = (_e292 * _e336);
                let _e338 = (_e315 * _e336);
                let _e345 = (((vec2<f32>(_e338, -(_e338)) + vec2(_e335)) * (0.5f / _e328)) + vec2<f32>(0.5f, 0.5f));
                let _e348 = vec4<f32>(_e345.x, _e345.y, 0f, 0f);
                phi_2495_ = _e337;
                phi_2479_ = _e348;
                if (_e180 > 134217728u) {
                    let _e350 = (_e179 & 4194304u);
                    let _e352 = select(2i, -2i, (_e350 == 0u));
                    phi_2427_ = _e352;
                    if ((_e179 & 8388608u) != 0u) {
                        phi_2427_ = -(_e352);
                    }
                    let _e357 = phi_2427_;
                    let _e358 = (_e175 + _e357);
                    let _e363 = textureLoad(DC, vec2<i32>((_e358 & 2047i), (_e358 >> bitcast<u32>(11i))), 0i);
                    let _e367 = abs((bitcast<f32>(_e363.z) - _e288));
                    phi_2436_ = _e367;
                    if (_e367 > 3.1415927f) {
                        phi_2436_ = (6.2831855f - _e367);
                    }
                    let _e371 = phi_2436_;
                    let _e376 = ((_e371 * select(0.5f, -0.5f, ((_e350 != 0u) == _e307))) + _e288);
                    let _e380 = vec2<f32>(sin(_e376), -(cos(_e376)));
                    let _e381 = (_e116 * _e380);
                    let _e389 = ((abs(_e381.x) + abs(_e381.y)) * (1f / dot(_e381, _e381)));
                    let _e391 = cos((_e371 * 0.5f));
                    let _e392 = (_e180 == 335544320u);
                    phi_1681_ = _e392;
                    if !(_e392) {
                        phi_1681_ = ((_e180 == 268435456u) && (_e391 >= 0.25f));
                    }
                    let _e398 = phi_1681_;
                    if _e398 {
                        phi_2443_ = (_e335 * (1f / max(_e391, select(0.25f, 1f, ((_e179 & 33554432u) != 0u)))));
                    } else {
                        phi_2443_ = ((_e335 * _e391) + (_e389 * 0.5f));
                    }
                    let _e409 = phi_2443_;
                    let _e411 = (_e409 + (_e389 * 0.5f));
                    phi_2458_ = _e337;
                    if ((_e179 & 2097152u) != 0u) {
                        if (_e336 <= ((_e411 * _e391) + (_e328 * 0.125f))) {
                            phi_2459_ = (_e380 * (_e336 * (1f / _e391)));
                        } else {
                            let _e421 = (_e380 * _e411);
                            phi_2459_ = (vec2<f32>(dot(_e337, _e337), dot(_e421, _e421)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e337, _e421)));
                        }
                        let _e429 = phi_2459_;
                        phi_2458_ = _e429;
                    }
                    let _e431 = phi_2458_;
                    let _e436 = ((_e411 - dot((_e431 * abs(_e315)), _e380)) / _e389);
                    if _e307 {
                        phi_2480_ = vec4<f32>(_e348.x, _e436, _e348.z, _e348.w);
                    } else {
                        phi_2480_ = vec4<f32>(_e436, _e348.y, _e348.z, _e348.w);
                    }
                    let _e448 = phi_2480_;
                    phi_2495_ = _e431;
                    phi_2479_ = _e448;
                }
                let _e450 = phi_2495_;
                let _e452 = phi_2479_;
                let _e454 = (_e452.xy * _e334);
                let _e460 = vec4<f32>(_e454.x, _e452.y, _e452.z, _e452.w);
                let _e467 = vec4<f32>(_e460.x, max(_e454.y, 0.0001f), _e460.z, _e460.w);
                phi_2526_ = _e467;
                if _e316 {
                    phi_2526_ = vec4<f32>((-2f - _e454.x), _e467.y, _e467.z, _e467.w);
                }
                let _e475 = phi_2526_;
                if (_e83 != 0i) {
                    phi_2568_ = _e475;
                    phi_2530_ = vec2<f32>();
                    phi_2529_ = false;
                    break;
                }
                phi_2525_ = _e475;
                phi_2521_ = (_e116 * (_e450 * _e315));
                phi_2497_ = _e294;
            } else {
                let _e479 = vec4<f32>(_e134, -1f, 0f, 0f);
                if (_e301 != 0f) {
                    let _e485 = vec4<f32>(_e479.x, -2f, _e479.z, _e479.w);
                    let _e490 = vec4<f32>(_e485.x, _e485.y, 1000000f, _e485.w);
                    phi_2374_ = vec4<f32>(_e490.x, _e490.y, _e490.z, _e134);
                    if _e183 {
                        phi_2331_ = _e284;
                        phi_2330_ = _e282;
                        if (_e284 < 0f) {
                            phi_2331_ = -(_e284);
                            phi_2330_ = (_e282 + _e284);
                        }
                        let _e500 = phi_2331_;
                        let _e502 = phi_2330_;
                        let _e504 = ((_e288 - _e502) + 1.5707964f);
                        let _e510 = clamp(((_e504 - (floor((_e504 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e500);
                        phi_2332_ = _e510;
                        if (_e510 > (_e500 * 0.5f)) {
                            phi_2332_ = (_e500 - _e510);
                        }
                        let _e515 = phi_2332_;
                        let _e522 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e515), cos(_e515)) * abs(_e286))) * 0.5f);
                        if (abs((_e500 - 1.5707964f)) < 0.001f) {
                            phi_2358_ = 0f;
                            phi_2356_ = 0f;
                        } else {
                            let _e526 = tan(_e500);
                            let _e531 = (sign((1.5707964f - _e500)) / max(abs(_e526), 0.000001f));
                            if (_e531 >= 0f) {
                                phi_2336_ = (_e522.y - ((1f - _e522.x) * _e526));
                            } else {
                                phi_2336_ = (_e522.y + (_e522.x * _e526));
                            }
                            let _e543 = phi_2336_;
                            phi_2358_ = _e543;
                            phi_2356_ = _e531;
                        }
                        let _e545 = phi_2358_;
                        let _e547 = phi_2356_;
                        phi_2374_ = vec4<f32>((max(_e522.x, 0f) + 0.25f), (-2f - _e522.y), _e547, _e545);
                    }
                    let _e555 = phi_2374_;
                    phi_2524_ = (_e116 * (_e292 * (_e286 * _e301)));
                    phi_2373_ = _e555;
                } else {
                    phi_2524_ = (sign(((_e292 * _e286) * _naga_inverse_2x2_f32(_e116))) * 0.5f);
                    phi_2373_ = _e479;
                }
                let _e565 = phi_2524_;
                let _e567 = phi_2373_;
                phi_2528_ = _e567;
                if (((_e179 & 8388608u) != 0u) != ((_e179 & 16777216u) != 0u)) {
                    phi_2528_ = (_e567 * vec4<f32>(-1f, 1f, 1f, 1f));
                }
                let _e575 = phi_2528_;
                if (((_e179 & 2147483648u) != 0u) && (_e83 != 1i)) {
                    phi_2568_ = _e575;
                    phi_2530_ = vec2<f32>();
                    phi_2529_ = false;
                    break;
                }
                phi_2525_ = _e575;
                phi_2521_ = _e565;
                phi_2497_ = select(_e294, _e101, vec2((_e83 == 2i)));
            }
            let _e584 = phi_2525_;
            let _e586 = phi_2521_;
            let _e588 = phi_2497_;
            let _e594 = k.og;
            let _e597 = select(_e584.xy, vec2<f32>(1f, -1f), vec2((_e594 != 0u)));
            let _e603 = vec4<f32>(_e597.x, _e584.y, _e584.z, _e584.w);
            phi_2568_ = vec4<f32>(_e603.x, _e597.y, _e603.z, _e603.w);
            phi_2530_ = (((_e116 * _e588) + _e586) + bitcast<vec2<f32>>(_e120.xy));
            phi_2529_ = true;
            break;
        }
    }
    let _e611 = phi_2568_;
    let _e613 = phi_2530_;
    let _e615 = phi_2529_;
    I = _e611;
    if _e615 {
        let _e617 = local;
        let _e621 = MB.X1_[(_e617 + 2u)];
        let _e623 = bitcast<vec3<f32>>(_e621.yzw);
        let _e627 = ((_e613 * _e623.x) + _e623.yz);
        let _e630 = k.ed[0u];
        let _e633 = k.ed[1u];
        phi_2569_ = vec4<f32>(((_e627.x * _e630) - 1f), ((_e627.y * _e633) - sign(_e633)), 0f, 1f);
    } else {
        let _e643 = k.P2_;
        phi_2569_ = vec4(_e643);
    }
    let _e646 = phi_2569_;
    unnamed.gl_Position = _e646;
    return;
}

@vertex 
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) SB: vec4<f32>, @location(1) TB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    SB_1 = SB;
    TB_1 = TB;
    main_1();
    let _e13 = I;
    let _e14 = unnamed.gl_Position;
    return VertexOutput(_e13, _e14);
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
