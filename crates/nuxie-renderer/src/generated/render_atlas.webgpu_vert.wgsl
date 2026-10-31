struct qi {
    x2_: array<vec4<u32>>,
}

struct kd {
    x2_: array<vec4<u32>>,
}

struct WB {
    Hd: f32,
    Qe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    I8_: vec4<i32>,
    Ki: vec2<f32>,
    Re: vec2<f32>,
    w2_: u32,
    Oi: f32,
    w6_: u32,
    j3_: f32,
    Se: f32,
    xg: u32,
    I3_: f32,
    J3_: f32,
    Te: f32,
    Hi: u32,
    Ea: u32,
    P8_: f32,
    Q8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct vg {
    x2_: array<vec4<f32>>,
}

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(6)
var VB: texture_2d<u32>;
@group(0) @binding(4)
var<storage> CD: qi;
@group(0) @binding(2)
var<storage> JB: kd;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> YB_1: vec4<f32>;
var<private> ZB_1: vec4<f32>;
var<private> S: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(3)
var<storage> KB: vg;
@group(3) @binding(8)
var fb: sampler;

fn main_1() {
    var phi_2346_: f32;
    var phi_2296_: f32;
    var phi_2233_: f32;
    var phi_2323_: f32;
    var phi_2259_: f32;
    var phi_2227_: i32;
    var phi_1367_: bool;
    var phi_2244_: i32;
    var phi_2235_: vec4<u32>;
    var phi_2243_: i32;
    var phi_2234_: vec4<u32>;
    var phi_2242_: i32;
    var phi_2240_: vec4<u32>;
    var phi_2239_: u32;
    var phi_2246_: vec2<i32>;
    var phi_2247_: vec4<u32>;
    var phi_2251_: f32;
    var phi_2334_: f32;
    var phi_2266_: f32;
    var phi_2333_: f32;
    var phi_2274_: f32;
    var phi_2267_: f32;
    var phi_2264_: f32;
    var phi_2278_: f32;
    var phi_2371_: f32;
    var phi_2361_: f32;
    var phi_2331_: f32;
    var phi_2277_: f32;
    var phi_2329_: f32;
    var phi_2432_: f32;
    var phi_2443_: f32;
    var phi_2435_: f32;
    var phi_2497_: f32;
    var phi_2471_: f32;
    var phi_1682_: bool;
    var phi_2476_: f32;
    var phi_2488_: vec2<f32>;
    var phi_2487_: vec2<f32>;
    var phi_2505_: vec4<f32>;
    var phi_2518_: vec2<f32>;
    var phi_2504_: vec4<f32>;
    var phi_2554_: vec4<f32>;
    var phi_2382_: f32;
    var phi_2381_: f32;
    var phi_2383_: f32;
    var phi_2387_: f32;
    var phi_2410_: f32;
    var phi_2408_: f32;
    var phi_2427_: vec4<f32>;
    var phi_2551_: vec2<f32>;
    var phi_2550_: vec2<f32>;
    var phi_2426_: vec4<f32>;
    var phi_2557_: vec4<f32>;
    var phi_2558_: bool;
    var phi_2552_: vec4<f32>;
    var phi_2545_: vec2<f32>;
    var phi_2520_: vec2<f32>;
    var phi_2591_: vec4<f32>;

    let _e72 = gl_InstanceIndex_1;
    let _e73 = YB_1;
    let _e74 = ZB_1;
    let _e76 = i32(_e73.x);
    let _e80 = bitcast<i32>(_e73.w);
    let _e82 = (_e80 >> bitcast<u32>(2i));
    let _e83 = (_e80 & 3i);
    let _e85 = min(_e76, (_e82 - 1i));
    let _e87 = ((_e72 * _e82) + _e85);
    let _e92 = textureLoad(VB, vec2<i32>((_e87 & 2047i), (_e87 >> bitcast<u32>(11i))), 0i);
    let _e99 = CD.x2_[(max((_e92.w & 65535u), 1u) - 1u)];
    let _e101 = bitcast<vec2<f32>>(_e99.xy);
    let _e105 = ((_e99.z & 65535u) * 4u);
    let _e108 = JB.x2_[_e105];
    let _e109 = bitcast<vec4<f32>>(_e108);
    let _e116 = mat2x2<f32>(vec2<f32>(_e109.x, _e109.y), vec2<f32>(_e109.z, _e109.w));
    let _e120 = JB.x2_[(_e105 + 1u)];
    phi_2346_ = 0f;
    phi_2296_ = 0f;
    phi_2233_ = 0f;
    if ((_e120.z & 32768u) != 0u) {
        let _e129 = JB.x2_[(_e105 + 2u)];
        phi_2346_ = bitcast<f32>(_e129.z);
        phi_2296_ = bitcast<f32>(_e129.y);
        phi_2233_ = bitcast<f32>(_e129.x);
    }
    let _e137 = phi_2346_;
    let _e139 = phi_2296_;
    let _e141 = phi_2233_;
    let _e142 = (_e92.w & 8388608u);
    phi_2323_ = _e73.z;
    phi_2259_ = _e73.y;
    phi_2227_ = _e76;
    if (_e142 != 0u) {
        phi_2323_ = _e74.z;
        phi_2259_ = _e74.y;
        phi_2227_ = i32(_e74.x);
    }
    let _e149 = phi_2323_;
    let _e151 = phi_2259_;
    let _e153 = phi_2227_;
    phi_2242_ = _e87;
    phi_2240_ = _e92;
    phi_2239_ = _e92.w;
    if (_e153 != _e85) {
        let _e156 = ((_e87 + _e153) - _e85);
        let _e161 = textureLoad(VB, vec2<i32>((_e156 & 2047i), (_e156 >> bitcast<u32>(11i))), 0i);
        if ((_e161.w & 8454143u) != (_e92.w & 8454143u)) {
            let _e166 = (_e141 == 0f);
            phi_1367_ = _e166;
            if !(_e166) {
                phi_1367_ = (_e101.x != 0f);
            }
            let _e171 = phi_1367_;
            phi_2244_ = _e87;
            phi_2235_ = _e92;
            if _e171 {
                let _e172 = bitcast<i32>(_e99.w);
                let _e177 = textureLoad(VB, vec2<i32>((_e172 & 2047i), (_e172 >> bitcast<u32>(11i))), 0i);
                phi_2244_ = _e172;
                phi_2235_ = _e177;
            }
            let _e179 = phi_2244_;
            let _e181 = phi_2235_;
            phi_2243_ = _e179;
            phi_2234_ = _e181;
        } else {
            phi_2243_ = _e156;
            phi_2234_ = _e161;
        }
        let _e183 = phi_2243_;
        let _e185 = phi_2234_;
        phi_2242_ = _e183;
        phi_2240_ = _e185;
        phi_2239_ = ((_e185.w & 4286578687u) | _e142);
    }
    let _e190 = phi_2242_;
    let _e192 = phi_2240_;
    let _e194 = phi_2239_;
    let _e195 = (_e194 & 469762048u);
    let _e198 = ((_e195 == 67108864u) && (_e83 == 0i));
    if _e198 {
        let _e201 = f32((_e192.z & 65535u));
        let _e204 = f32((_e192.z >> bitcast<u32>(16i)));
        let _e210 = vec2<i32>(i32((-1f - _e201)), i32(((_e204 - _e201) + 1f)));
        phi_2246_ = _e210;
        if ((_e194 & 8388608u) != 0u) {
            phi_2246_ = -(_e210);
        }
        let _e215 = phi_2246_;
        let _e217 = (_e190 + _e215.x);
        let _e222 = textureLoad(VB, vec2<i32>((_e217 & 2047i), (_e217 >> bitcast<u32>(11i))), 0i);
        let _e224 = (_e190 + _e215.y);
        let _e229 = textureLoad(VB, vec2<i32>((_e224 & 2047i), (_e224 >> bitcast<u32>(11i))), 0i);
        phi_2247_ = _e229;
        if ((_e229.w & 8454143u) != (_e222.w & 8454143u)) {
            let _e235 = bitcast<i32>(_e99.w);
            let _e240 = textureLoad(VB, vec2<i32>((_e235 & 2047i), (_e235 >> bitcast<u32>(11i))), 0i);
            phi_2247_ = _e240;
        }
        let _e242 = phi_2247_;
        let _e245 = (f32(_e222.z) * 0.0000000014629181f);
        let _e248 = (f32(_e242.z) * 0.0000000014629181f);
        let _e249 = (_e248 - _e245);
        phi_2251_ = _e249;
        if (abs(_e249) > 3.1415927f) {
            phi_2251_ = (_e249 - (6.2831855f * sign(_e249)));
        }
        let _e256 = phi_2251_;
        let _e257 = (_e204 + -2f);
        let _e263 = clamp(round(((abs(_e256) * 0.31830987f) * _e257)), 1f, (_e204 + -3f));
        let _e264 = (_e257 - _e263);
        if (_e201 <= _e264) {
            phi_2334_ = _e151;
            if (_e201 == _e264) {
                phi_2334_ = -(_e151);
            }
            let _e273 = phi_2334_;
            phi_2333_ = _e273;
            phi_2274_ = -(((3.1415927f * sign(_e256)) - _e256));
            phi_2267_ = _e264;
            phi_2264_ = _e201;
        } else {
            let _e275 = (_e201 == (_e264 + 1f));
            if _e275 {
                phi_2266_ = 0f;
            } else {
                phi_2266_ = (_e201 - (_e264 + 2f));
            }
            let _e279 = phi_2266_;
            phi_2333_ = select(_e151, 0f, _e275);
            phi_2274_ = _e256;
            phi_2267_ = select(_e263, 0f, _e275);
            phi_2264_ = _e279;
        }
        let _e283 = phi_2333_;
        let _e285 = phi_2274_;
        let _e287 = phi_2267_;
        let _e289 = phi_2264_;
        if (_e289 == _e287) {
            phi_2278_ = _e248;
        } else {
            phi_2278_ = (_e245 + (_e285 * (_e289 / _e287)));
        }
        let _e295 = phi_2278_;
        phi_2371_ = _e245;
        phi_2361_ = _e285;
        phi_2331_ = _e283;
        phi_2277_ = _e295;
    } else {
        phi_2371_ = f32();
        phi_2361_ = f32();
        phi_2331_ = _e151;
        phi_2277_ = (f32(_e192.z) * 0.0000000014629181f);
    }
    let _e300 = phi_2371_;
    let _e302 = phi_2361_;
    let _e304 = phi_2331_;
    let _e306 = phi_2277_;
    let _e310 = vec2<f32>(sin(_e306), -(cos(_e306)));
    let _e312 = bitcast<vec2<f32>>(_e192.xy);
    phi_2329_ = _e139;
    if (_e139 != 0f) {
        phi_2329_ = max(_e139, (1f / length((_e116 * _e310))));
    }
    let _e319 = phi_2329_;
    if (_e141 != 0f) {
        let _e323 = (_e304 * sign(determinant(_e116)));
        let _e325 = ((_e194 & 1048576u) != 0u);
        phi_2432_ = _e323;
        if _e325 {
            phi_2432_ = min(_e323, 0f);
        }
        let _e328 = phi_2432_;
        phi_2443_ = _e328;
        if ((_e194 & 524288u) != 0u) {
            phi_2443_ = max(_e328, 0f);
        }
        let _e333 = phi_2443_;
        let _e334 = (_e319 != 0f);
        if _e334 {
            phi_2435_ = _e319;
        } else {
            let _e335 = (_e116 * _e310);
            phi_2435_ = (((abs(_e335.x) + abs(_e335.y)) * (1f / dot(_e335, _e335))) * 0.5f);
        }
        let _e346 = phi_2435_;
        let _e349 = ((_e346 > _e141) && (_e319 == 0f));
        phi_2497_ = 1f;
        if _e349 {
            phi_2497_ = (_e141 / _e346);
        }
        let _e352 = phi_2497_;
        let _e353 = select(_e141, _e346, _e349);
        let _e354 = (_e353 + _e346);
        let _e355 = (_e310 * _e354);
        let _e356 = (_e333 * _e354);
        let _e363 = (((vec2<f32>(_e356, -(_e356)) + vec2(_e353)) * (0.5f / _e346)) + vec2<f32>(0.5f, 0.5f));
        let _e366 = vec4<f32>(_e363.x, _e363.y, 0f, 0f);
        phi_2518_ = _e355;
        phi_2504_ = _e366;
        if (_e195 > 201326592u) {
            let _e372 = f32((_e192.z & 65535u));
            let _e373 = (_e372 * 0.000015259022f);
            let _e377 = sqrt(max((1f - (_e373 * _e373)), 0f));
            phi_2471_ = _e377;
            if (((_e194 & 4194304u) != 0u) == _e325) {
                phi_2471_ = -(_e377);
            }
            let _e381 = phi_2471_;
            let _e386 = (mat2x2<f32>(vec2<f32>(_e373, _e381), vec2<f32>(-(_e381), _e373)) * _e310);
            let _e387 = (_e116 * _e386);
            let _e395 = ((abs(_e387.x) + abs(_e387.y)) * (1f / dot(_e387, _e387)));
            let _e396 = (_e195 == 402653184u);
            phi_1682_ = _e396;
            if !(_e396) {
                phi_1682_ = ((_e195 == 335544320u) && (_e373 >= 0.25f));
            }
            let _e402 = phi_1682_;
            if _e402 {
                phi_2476_ = (_e353 * (1f / max(_e373, select(0.25f, 1f, ((_e194 & 33554432u) != 0u)))));
            } else {
                phi_2476_ = ((_e353 * _e373) + (_e395 * 0.5f));
            }
            let _e413 = phi_2476_;
            let _e415 = (_e413 + (_e395 * 0.5f));
            phi_2487_ = _e355;
            if ((_e194 & 2097152u) != 0u) {
                if (_e354 <= ((_e415 * _e373) + (_e346 * 0.125f))) {
                    phi_2488_ = (_e386 * (_e354 * (65535f / _e372)));
                } else {
                    let _e425 = (_e386 * _e415);
                    phi_2488_ = (vec2<f32>(dot(_e355, _e355), dot(_e425, _e425)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e355, _e425)));
                }
                let _e433 = phi_2488_;
                phi_2487_ = _e433;
            }
            let _e435 = phi_2487_;
            let _e440 = ((_e415 - dot((_e435 * abs(_e333)), _e386)) / _e395);
            if _e325 {
                phi_2505_ = vec4<f32>(_e366.x, _e440, _e366.z, _e366.w);
            } else {
                phi_2505_ = vec4<f32>(_e440, _e366.y, _e366.z, _e366.w);
            }
            let _e452 = phi_2505_;
            phi_2518_ = _e435;
            phi_2504_ = _e452;
        }
        let _e454 = phi_2518_;
        let _e456 = phi_2504_;
        let _e458 = (_e456.xy * _e352);
        let _e464 = vec4<f32>(_e458.x, _e456.y, _e456.z, _e456.w);
        let _e471 = vec4<f32>(_e464.x, max(_e458.y, 0.0001f), _e464.z, _e464.w);
        phi_2554_ = _e471;
        if _e334 {
            phi_2554_ = vec4<f32>((-2f - _e458.x), _e471.y, _e471.z, _e471.w);
        }
        let _e479 = phi_2554_;
        phi_2558_ = (_e83 != 0i);
        phi_2552_ = _e479;
        phi_2545_ = (_e116 * (_e454 * _e333));
        phi_2520_ = _e312;
    } else {
        let _e483 = vec4<f32>(_e149, -1f, 0f, 0f);
        if (_e319 != 0f) {
            let _e489 = vec4<f32>(_e483.x, -2f, _e483.z, _e483.w);
            let _e494 = vec4<f32>(_e489.x, _e489.y, 1000000f, _e489.w);
            phi_2427_ = vec4<f32>(_e494.x, _e494.y, _e494.z, _e149);
            if _e198 {
                phi_2382_ = _e302;
                phi_2381_ = _e300;
                if (_e302 < 0f) {
                    phi_2382_ = -(_e302);
                    phi_2381_ = (_e300 + _e302);
                }
                let _e504 = phi_2382_;
                let _e506 = phi_2381_;
                let _e508 = ((_e306 - _e506) + 1.5707964f);
                let _e514 = clamp(((_e508 - (floor((_e508 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e504);
                phi_2383_ = _e514;
                if (_e514 > (_e504 * 0.5f)) {
                    phi_2383_ = (_e504 - _e514);
                }
                let _e519 = phi_2383_;
                let _e526 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e519), cos(_e519)) * abs(_e304))) * 0.5f);
                if (abs((_e504 - 1.5707964f)) < 0.001f) {
                    phi_2410_ = 0f;
                    phi_2408_ = 0f;
                } else {
                    let _e530 = tan(_e504);
                    let _e535 = (sign((1.5707964f - _e504)) / max(abs(_e530), 0.000001f));
                    if (_e535 >= 0f) {
                        phi_2387_ = (_e526.y - ((1f - _e526.x) * _e530));
                    } else {
                        phi_2387_ = (_e526.y + (_e526.x * _e530));
                    }
                    let _e547 = phi_2387_;
                    phi_2410_ = _e547;
                    phi_2408_ = _e535;
                }
                let _e549 = phi_2410_;
                let _e551 = phi_2408_;
                phi_2427_ = vec4<f32>((max(_e526.x, 0f) + 0.25f), (-2f - _e526.y), _e551, _e549);
            }
            let _e559 = phi_2427_;
            phi_2550_ = (_e116 * (_e310 * (_e304 * _e319)));
            phi_2426_ = _e559;
        } else {
            let _e567 = (sign(((_e310 * _e304) * _naga_inverse_2x2_f32(_e116))) * 0.5f);
            phi_2551_ = _e567;
            if (_e83 != 2i) {
                phi_2551_ = (_e567 + (_e310 * _e137));
            }
            let _e572 = phi_2551_;
            phi_2550_ = _e572;
            phi_2426_ = _e483;
        }
        let _e574 = phi_2550_;
        let _e576 = phi_2426_;
        phi_2557_ = _e576;
        if (((_e194 & 8388608u) != 0u) != ((_e194 & 16777216u) != 0u)) {
            phi_2557_ = (_e576 * vec4<f32>(-1f, 1f, 1f, 1f));
        }
        let _e584 = phi_2557_;
        phi_2558_ = (((_e194 & 2147483648u) != 0u) && (_e83 != 1i));
        phi_2552_ = _e584;
        phi_2545_ = _e574;
        phi_2520_ = select(_e312, _e101, vec2((_e83 == 2i)));
    }
    let _e593 = phi_2558_;
    let _e595 = phi_2552_;
    let _e597 = phi_2545_;
    let _e599 = phi_2520_;
    let _e605 = j.Hi;
    let _e608 = select(_e595.xy, vec2<f32>(1f, -1f), vec2((_e605 != 0u)));
    let _e614 = vec4<f32>(_e608.x, _e595.y, _e595.z, _e595.w);
    S = vec4<f32>(_e614.x, _e608.y, _e614.z, _e614.w);
    if !(_e593) {
        let _e625 = JB.x2_[(_e105 + 3u)];
        let _e627 = bitcast<vec3<f32>>(_e625.xyz);
        let _e631 = (((((_e116 * _e599) + _e597) + bitcast<vec2<f32>>(_e120.xy)) * _e627.x) + _e627.yz);
        let _e634 = j.Re[0u];
        let _e637 = j.Re[1u];
        phi_2591_ = vec4<f32>(((_e631.x * _e634) - 1f), ((_e631.y * _e637) - sign(_e637)), 0f, 1f);
    } else {
        let _e647 = j.j3_;
        phi_2591_ = vec4(_e647);
    }
    let _e650 = phi_2591_;
    unnamed.gl_Position = _e650;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) YB: vec4<f32>, @location(1) ZB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    YB_1 = YB;
    ZB_1 = ZB;
    main_1();
    let _e13 = S;
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
