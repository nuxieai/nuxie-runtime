struct xi {
    z2_: array<vec4<u32>>,
}

struct nd {
    z2_: array<vec4<u32>>,
}

struct RB {
    Kd: f32,
    Ue: f32,
    ah: f32,
    bh: f32,
    V5_: u32,
    Ia: u32,
    Lg: u32,
    Ng: u32,
    M8_: vec4<i32>,
    Ri: vec2<f32>,
    Ve: vec2<f32>,
    y2_: u32,
    Vi: f32,
    z6_: u32,
    n3_: f32,
    We: f32,
    Eg: u32,
    K3_: f32,
    L3_: f32,
    Xe: f32,
    Oi: u32,
    Ha: u32,
    T8_: f32,
    U8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Cg {
    z2_: array<vec4<f32>>,
}

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @location(1) @interpolate(flat, either) member_1: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(6)
var WB: texture_2d<u32>;
@group(0) @binding(4)
var<storage> CD: xi;
@group(0) @binding(2)
var<storage> JB: nd;
@group(0) @binding(0)
var<uniform> j: RB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> YB_1: vec4<f32>;
var<private> ZB_1: vec4<f32>;
var<private> S: vec4<f32>;
var<private> J0_: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(3)
var<storage> KB: Cg;
@group(3) @binding(8)
var ib: sampler;

fn main_1() {
    var phi_2363_: f32;
    var phi_2313_: f32;
    var phi_2250_: f32;
    var phi_2340_: f32;
    var phi_2276_: f32;
    var phi_2244_: i32;
    var phi_1380_: bool;
    var phi_2261_: i32;
    var phi_2252_: vec4<u32>;
    var phi_2260_: i32;
    var phi_2251_: vec4<u32>;
    var phi_2259_: i32;
    var phi_2257_: vec4<u32>;
    var phi_2256_: u32;
    var phi_2263_: vec2<i32>;
    var phi_2264_: vec4<u32>;
    var phi_2268_: f32;
    var phi_2351_: f32;
    var phi_2283_: f32;
    var phi_2350_: f32;
    var phi_2291_: f32;
    var phi_2284_: f32;
    var phi_2281_: f32;
    var phi_2295_: f32;
    var phi_2388_: f32;
    var phi_2378_: f32;
    var phi_2348_: f32;
    var phi_2294_: f32;
    var phi_2346_: f32;
    var phi_2449_: f32;
    var phi_2460_: f32;
    var phi_2452_: f32;
    var phi_2514_: f32;
    var phi_2488_: f32;
    var phi_1695_: bool;
    var phi_2493_: f32;
    var phi_2505_: vec2<f32>;
    var phi_2504_: vec2<f32>;
    var phi_2522_: vec4<f32>;
    var phi_2535_: vec2<f32>;
    var phi_2521_: vec4<f32>;
    var phi_2571_: vec4<f32>;
    var phi_2399_: f32;
    var phi_2398_: f32;
    var phi_2400_: f32;
    var phi_2404_: f32;
    var phi_2427_: f32;
    var phi_2425_: f32;
    var phi_2444_: vec4<f32>;
    var phi_2568_: vec2<f32>;
    var phi_2567_: vec2<f32>;
    var phi_2443_: vec4<f32>;
    var phi_2574_: vec4<f32>;
    var phi_2575_: bool;
    var phi_2569_: vec4<f32>;
    var phi_2562_: vec2<f32>;
    var phi_2537_: vec2<f32>;
    var phi_2608_: vec4<f32>;

    let _e71 = gl_InstanceIndex_1;
    let _e72 = YB_1;
    let _e73 = ZB_1;
    let _e75 = i32(_e72.x);
    let _e79 = bitcast<i32>(_e72.w);
    let _e81 = (_e79 >> bitcast<u32>(2i));
    let _e82 = (_e79 & 3i);
    let _e84 = min(_e75, (_e81 - 1i));
    let _e86 = ((_e71 * _e81) + _e84);
    let _e91 = textureLoad(WB, vec2<i32>((_e86 & 2047i), (_e86 >> bitcast<u32>(11i))), 0i);
    let _e98 = CD.z2_[(max((_e91.w & 65535u), 1u) - 1u)];
    let _e100 = bitcast<vec2<f32>>(_e98.xy);
    let _e102 = (_e98.z & 65535u);
    let _e104 = (_e102 * 4u);
    let _e107 = JB.z2_[_e104];
    let _e108 = bitcast<vec4<f32>>(_e107);
    let _e115 = mat2x2<f32>(vec2<f32>(_e108.x, _e108.y), vec2<f32>(_e108.z, _e108.w));
    let _e119 = JB.z2_[(_e104 + 1u)];
    phi_2363_ = 0f;
    phi_2313_ = 0f;
    phi_2250_ = 0f;
    if ((_e119.z & 32768u) != 0u) {
        let _e128 = JB.z2_[(_e104 + 2u)];
        phi_2363_ = bitcast<f32>(_e128.z);
        phi_2313_ = bitcast<f32>(_e128.y);
        phi_2250_ = bitcast<f32>(_e128.x);
    }
    let _e136 = phi_2363_;
    let _e138 = phi_2313_;
    let _e140 = phi_2250_;
    let _e141 = (_e91.w & 8388608u);
    phi_2340_ = _e72.z;
    phi_2276_ = _e72.y;
    phi_2244_ = _e75;
    if (_e141 != 0u) {
        phi_2340_ = _e73.z;
        phi_2276_ = _e73.y;
        phi_2244_ = i32(_e73.x);
    }
    let _e148 = phi_2340_;
    let _e150 = phi_2276_;
    let _e152 = phi_2244_;
    phi_2259_ = _e86;
    phi_2257_ = _e91;
    phi_2256_ = _e91.w;
    if (_e152 != _e84) {
        let _e155 = ((_e86 + _e152) - _e84);
        let _e160 = textureLoad(WB, vec2<i32>((_e155 & 2047i), (_e155 >> bitcast<u32>(11i))), 0i);
        if ((_e160.w & 8454143u) != (_e91.w & 8454143u)) {
            let _e165 = (_e140 == 0f);
            phi_1380_ = _e165;
            if !(_e165) {
                phi_1380_ = (_e100.x != 0f);
            }
            let _e170 = phi_1380_;
            phi_2261_ = _e86;
            phi_2252_ = _e91;
            if _e170 {
                let _e171 = bitcast<i32>(_e98.w);
                let _e176 = textureLoad(WB, vec2<i32>((_e171 & 2047i), (_e171 >> bitcast<u32>(11i))), 0i);
                phi_2261_ = _e171;
                phi_2252_ = _e176;
            }
            let _e178 = phi_2261_;
            let _e180 = phi_2252_;
            phi_2260_ = _e178;
            phi_2251_ = _e180;
        } else {
            phi_2260_ = _e155;
            phi_2251_ = _e160;
        }
        let _e182 = phi_2260_;
        let _e184 = phi_2251_;
        phi_2259_ = _e182;
        phi_2257_ = _e184;
        phi_2256_ = ((_e184.w & 4286578687u) | _e141);
    }
    let _e189 = phi_2259_;
    let _e191 = phi_2257_;
    let _e193 = phi_2256_;
    let _e194 = (_e193 & 469762048u);
    let _e197 = ((_e194 == 67108864u) && (_e82 == 0i));
    if _e197 {
        let _e200 = f32((_e191.z & 65535u));
        let _e203 = f32((_e191.z >> bitcast<u32>(16i)));
        let _e209 = vec2<i32>(i32((-1f - _e200)), i32(((_e203 - _e200) + 1f)));
        phi_2263_ = _e209;
        if ((_e193 & 8388608u) != 0u) {
            phi_2263_ = -(_e209);
        }
        let _e214 = phi_2263_;
        let _e216 = (_e189 + _e214.x);
        let _e221 = textureLoad(WB, vec2<i32>((_e216 & 2047i), (_e216 >> bitcast<u32>(11i))), 0i);
        let _e223 = (_e189 + _e214.y);
        let _e228 = textureLoad(WB, vec2<i32>((_e223 & 2047i), (_e223 >> bitcast<u32>(11i))), 0i);
        phi_2264_ = _e228;
        if ((_e228.w & 8454143u) != (_e221.w & 8454143u)) {
            let _e234 = bitcast<i32>(_e98.w);
            let _e239 = textureLoad(WB, vec2<i32>((_e234 & 2047i), (_e234 >> bitcast<u32>(11i))), 0i);
            phi_2264_ = _e239;
        }
        let _e241 = phi_2264_;
        let _e244 = (f32(_e221.z) * 0.0000000014629181f);
        let _e247 = (f32(_e241.z) * 0.0000000014629181f);
        let _e248 = (_e247 - _e244);
        phi_2268_ = _e248;
        if (abs(_e248) > 3.1415927f) {
            phi_2268_ = (_e248 - (6.2831855f * sign(_e248)));
        }
        let _e255 = phi_2268_;
        let _e256 = (_e203 + -2f);
        let _e262 = clamp(round(((abs(_e255) * 0.31830987f) * _e256)), 1f, (_e203 + -3f));
        let _e263 = (_e256 - _e262);
        if (_e200 <= _e263) {
            phi_2351_ = _e150;
            if (_e200 == _e263) {
                phi_2351_ = -(_e150);
            }
            let _e272 = phi_2351_;
            phi_2350_ = _e272;
            phi_2291_ = -(((3.1415927f * sign(_e255)) - _e255));
            phi_2284_ = _e263;
            phi_2281_ = _e200;
        } else {
            let _e274 = (_e200 == (_e263 + 1f));
            if _e274 {
                phi_2283_ = 0f;
            } else {
                phi_2283_ = (_e200 - (_e263 + 2f));
            }
            let _e278 = phi_2283_;
            phi_2350_ = select(_e150, 0f, _e274);
            phi_2291_ = _e255;
            phi_2284_ = select(_e262, 0f, _e274);
            phi_2281_ = _e278;
        }
        let _e282 = phi_2350_;
        let _e284 = phi_2291_;
        let _e286 = phi_2284_;
        let _e288 = phi_2281_;
        if (_e288 == _e286) {
            phi_2295_ = _e247;
        } else {
            phi_2295_ = (_e244 + (_e284 * (_e288 / _e286)));
        }
        let _e294 = phi_2295_;
        phi_2388_ = _e244;
        phi_2378_ = _e284;
        phi_2348_ = _e282;
        phi_2294_ = _e294;
    } else {
        phi_2388_ = f32();
        phi_2378_ = f32();
        phi_2348_ = _e150;
        phi_2294_ = (f32(_e191.z) * 0.0000000014629181f);
    }
    let _e299 = phi_2388_;
    let _e301 = phi_2378_;
    let _e303 = phi_2348_;
    let _e305 = phi_2294_;
    let _e309 = vec2<f32>(sin(_e305), -(cos(_e305)));
    let _e311 = bitcast<vec2<f32>>(_e191.xy);
    phi_2346_ = _e138;
    if (_e138 != 0f) {
        phi_2346_ = max(_e138, (1f / length((_e115 * _e309))));
    }
    let _e318 = phi_2346_;
    if (_e140 != 0f) {
        let _e322 = (_e303 * sign(determinant(_e115)));
        let _e324 = ((_e193 & 1048576u) != 0u);
        phi_2449_ = _e322;
        if _e324 {
            phi_2449_ = min(_e322, 0f);
        }
        let _e327 = phi_2449_;
        phi_2460_ = _e327;
        if ((_e193 & 524288u) != 0u) {
            phi_2460_ = max(_e327, 0f);
        }
        let _e332 = phi_2460_;
        let _e333 = (_e318 != 0f);
        if _e333 {
            phi_2452_ = _e318;
        } else {
            let _e334 = (_e115 * _e309);
            phi_2452_ = (((abs(_e334.x) + abs(_e334.y)) * (1f / dot(_e334, _e334))) * 0.5f);
        }
        let _e345 = phi_2452_;
        let _e348 = ((_e345 > _e140) && (_e318 == 0f));
        phi_2514_ = 1f;
        if _e348 {
            phi_2514_ = (_e140 / _e345);
        }
        let _e351 = phi_2514_;
        let _e352 = select(_e140, _e345, _e348);
        let _e353 = (_e352 + _e345);
        let _e354 = (_e309 * _e353);
        let _e355 = (_e332 * _e353);
        let _e362 = (((vec2<f32>(_e355, -(_e355)) + vec2(_e352)) * (0.5f / _e345)) + vec2<f32>(0.5f, 0.5f));
        let _e365 = vec4<f32>(_e362.x, _e362.y, 0f, 0f);
        phi_2535_ = _e354;
        phi_2521_ = _e365;
        if (_e194 > 201326592u) {
            let _e371 = f32((_e191.z & 65535u));
            let _e372 = (_e371 * 0.000015259022f);
            let _e376 = sqrt(max((1f - (_e372 * _e372)), 0f));
            phi_2488_ = _e376;
            if (((_e193 & 4194304u) != 0u) == _e324) {
                phi_2488_ = -(_e376);
            }
            let _e380 = phi_2488_;
            let _e385 = (mat2x2<f32>(vec2<f32>(_e372, _e380), vec2<f32>(-(_e380), _e372)) * _e309);
            let _e386 = (_e115 * _e385);
            let _e394 = ((abs(_e386.x) + abs(_e386.y)) * (1f / dot(_e386, _e386)));
            let _e395 = (_e194 == 402653184u);
            phi_1695_ = _e395;
            if !(_e395) {
                phi_1695_ = ((_e194 == 335544320u) && (_e372 >= 0.25f));
            }
            let _e401 = phi_1695_;
            if _e401 {
                phi_2493_ = (_e352 * (1f / max(_e372, select(0.25f, 1f, ((_e193 & 33554432u) != 0u)))));
            } else {
                phi_2493_ = ((_e352 * _e372) + (_e394 * 0.5f));
            }
            let _e412 = phi_2493_;
            let _e414 = (_e412 + (_e394 * 0.5f));
            phi_2504_ = _e354;
            if ((_e193 & 2097152u) != 0u) {
                if (_e353 <= ((_e414 * _e372) + (_e345 * 0.125f))) {
                    phi_2505_ = (_e385 * (_e353 * (65535f / _e371)));
                } else {
                    let _e424 = (_e385 * _e414);
                    phi_2505_ = (vec2<f32>(dot(_e354, _e354), dot(_e424, _e424)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e354, _e424)));
                }
                let _e432 = phi_2505_;
                phi_2504_ = _e432;
            }
            let _e434 = phi_2504_;
            let _e439 = ((_e414 - dot((_e434 * abs(_e332)), _e385)) / _e394);
            if _e324 {
                phi_2522_ = vec4<f32>(_e365.x, _e439, _e365.z, _e365.w);
            } else {
                phi_2522_ = vec4<f32>(_e439, _e365.y, _e365.z, _e365.w);
            }
            let _e451 = phi_2522_;
            phi_2535_ = _e434;
            phi_2521_ = _e451;
        }
        let _e453 = phi_2535_;
        let _e455 = phi_2521_;
        let _e457 = (_e455.xy * _e351);
        let _e463 = vec4<f32>(_e457.x, _e455.y, _e455.z, _e455.w);
        let _e470 = vec4<f32>(_e463.x, max(_e457.y, 0.0001f), _e463.z, _e463.w);
        phi_2571_ = _e470;
        if _e333 {
            phi_2571_ = vec4<f32>((-2f - _e457.x), _e470.y, _e470.z, _e470.w);
        }
        let _e478 = phi_2571_;
        phi_2575_ = (_e82 != 0i);
        phi_2569_ = _e478;
        phi_2562_ = (_e115 * (_e453 * _e332));
        phi_2537_ = _e311;
    } else {
        let _e482 = vec4<f32>(_e148, -1f, 0f, 0f);
        if (_e318 != 0f) {
            let _e488 = vec4<f32>(_e482.x, -2f, _e482.z, _e482.w);
            let _e493 = vec4<f32>(_e488.x, _e488.y, 1000000f, _e488.w);
            phi_2444_ = vec4<f32>(_e493.x, _e493.y, _e493.z, _e148);
            if _e197 {
                phi_2399_ = _e301;
                phi_2398_ = _e299;
                if (_e301 < 0f) {
                    phi_2399_ = -(_e301);
                    phi_2398_ = (_e299 + _e301);
                }
                let _e503 = phi_2399_;
                let _e505 = phi_2398_;
                let _e507 = ((_e305 - _e505) + 1.5707964f);
                let _e513 = clamp(((_e507 - (floor((_e507 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e503);
                phi_2400_ = _e513;
                if (_e513 > (_e503 * 0.5f)) {
                    phi_2400_ = (_e503 - _e513);
                }
                let _e518 = phi_2400_;
                let _e525 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e518), cos(_e518)) * abs(_e303))) * 0.5f);
                if (abs((_e503 - 1.5707964f)) < 0.001f) {
                    phi_2427_ = 0f;
                    phi_2425_ = 0f;
                } else {
                    let _e529 = tan(_e503);
                    let _e534 = (sign((1.5707964f - _e503)) / max(abs(_e529), 0.000001f));
                    if (_e534 >= 0f) {
                        phi_2404_ = (_e525.y - ((1f - _e525.x) * _e529));
                    } else {
                        phi_2404_ = (_e525.y + (_e525.x * _e529));
                    }
                    let _e546 = phi_2404_;
                    phi_2427_ = _e546;
                    phi_2425_ = _e534;
                }
                let _e548 = phi_2427_;
                let _e550 = phi_2425_;
                phi_2444_ = vec4<f32>((max(_e525.x, 0f) + 0.25f), (-2f - _e525.y), _e550, _e548);
            }
            let _e558 = phi_2444_;
            phi_2567_ = (_e115 * (_e309 * (_e303 * _e318)));
            phi_2443_ = _e558;
        } else {
            let _e566 = (sign(((_e309 * _e303) * _naga_inverse_2x2_f32(_e115))) * 0.5f);
            phi_2568_ = _e566;
            if (_e82 != 2i) {
                phi_2568_ = (_e566 + (_e309 * _e136));
            }
            let _e571 = phi_2568_;
            phi_2567_ = _e571;
            phi_2443_ = _e482;
        }
        let _e573 = phi_2567_;
        let _e575 = phi_2443_;
        phi_2574_ = _e575;
        if (((_e193 & 8388608u) != 0u) != ((_e193 & 16777216u) != 0u)) {
            phi_2574_ = (_e575 * vec4<f32>(-1f, 1f, 1f, 1f));
        }
        let _e583 = phi_2574_;
        phi_2575_ = (((_e193 & 2147483648u) != 0u) && (_e82 != 1i));
        phi_2569_ = _e583;
        phi_2562_ = _e573;
        phi_2537_ = select(_e311, _e100, vec2((_e82 == 2i)));
    }
    let _e592 = phi_2575_;
    let _e594 = phi_2569_;
    let _e596 = phi_2562_;
    let _e598 = phi_2537_;
    let _e601 = (((_e115 * _e598) + _e596) + bitcast<vec2<f32>>(_e119.xy));
    let _e604 = j.Oi;
    let _e607 = select(_e594.xy, vec2<f32>(1f, -1f), vec2((_e604 != 0u)));
    let _e613 = vec4<f32>(_e607.x, _e594.y, _e594.z, _e594.w);
    if !(_e592) {
        S = vec4<f32>(_e613.x, _e607.y, _e613.z, _e613.w);
        J0_ = _e102;
        let _e622 = j.ah;
        let _e624 = j.bh;
        phi_2608_ = vec4<f32>(((_e601.x * _e622) - 1f), ((_e601.y * _e624) - sign(_e624)), 0f, 1f);
    } else {
        let _e634 = j.n3_;
        phi_2608_ = vec4(_e634);
    }
    let _e637 = phi_2608_;
    unnamed.gl_Position = _e637;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) YB: vec4<f32>, @location(1) ZB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    YB_1 = YB;
    ZB_1 = ZB;
    main_1();
    let _e14 = S;
    let _e15 = J0_;
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
