struct pi {
    x2_: array<vec4<u32>>,
}

struct jd {
    x2_: array<vec4<u32>>,
}

struct WB {
    Gd: f32,
    Pe: f32,
    Rg: f32,
    Sg: f32,
    R6_: u32,
    Fa: u32,
    Dg: u32,
    Eg: u32,
    H8_: vec4<i32>,
    Ji: vec2<f32>,
    Qe: vec2<f32>,
    w2_: u32,
    Ni: f32,
    w6_: u32,
    j3_: f32,
    Re: f32,
    wg: u32,
    I3_: f32,
    J3_: f32,
    Se: f32,
    Gi: u32,
    Ea: u32,
    O8_: f32,
    P8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct ug {
    x2_: array<vec4<f32>>,
}

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @location(1) @interpolate(flat, either) member_1: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(6)
var VB: texture_2d<u32>;
@group(0) @binding(4)
var<storage> CD: pi;
@group(0) @binding(2)
var<storage> JB: jd;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> YB_1: vec4<f32>;
var<private> ZB_1: vec4<f32>;
var<private> S: vec4<f32>;
var<private> I0_: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(3)
var<storage> KB: ug;
@group(3) @binding(8)
var fb: sampler;

fn main_1() {
    var phi_2286_: f32;
    var phi_2223_: f32;
    var phi_2313_: f32;
    var phi_2249_: f32;
    var phi_2217_: i32;
    var phi_1363_: bool;
    var phi_2234_: i32;
    var phi_2225_: vec4<u32>;
    var phi_2233_: i32;
    var phi_2224_: vec4<u32>;
    var phi_2232_: i32;
    var phi_2230_: vec4<u32>;
    var phi_2229_: u32;
    var phi_2236_: vec2<i32>;
    var phi_2237_: vec4<u32>;
    var phi_2241_: f32;
    var phi_2324_: f32;
    var phi_2256_: f32;
    var phi_2323_: f32;
    var phi_2264_: f32;
    var phi_2257_: f32;
    var phi_2254_: f32;
    var phi_2268_: f32;
    var phi_2346_: f32;
    var phi_2336_: f32;
    var phi_2321_: f32;
    var phi_2267_: f32;
    var phi_2319_: f32;
    var phi_2405_: f32;
    var phi_2416_: f32;
    var phi_2408_: f32;
    var phi_2470_: f32;
    var phi_2444_: f32;
    var phi_1678_: bool;
    var phi_2449_: f32;
    var phi_2461_: vec2<f32>;
    var phi_2460_: vec2<f32>;
    var phi_2478_: vec4<f32>;
    var phi_2491_: vec2<f32>;
    var phi_2477_: vec4<f32>;
    var phi_2525_: vec4<f32>;
    var phi_2357_: f32;
    var phi_2356_: f32;
    var phi_2358_: f32;
    var phi_2362_: f32;
    var phi_2385_: f32;
    var phi_2383_: f32;
    var phi_2401_: vec4<f32>;
    var phi_2522_: vec2<f32>;
    var phi_2400_: vec4<f32>;
    var phi_2528_: vec4<f32>;
    var phi_2529_: bool;
    var phi_2523_: vec4<f32>;
    var phi_2517_: vec2<f32>;
    var phi_2493_: vec2<f32>;
    var phi_2561_: vec4<f32>;

    let _e71 = gl_InstanceIndex_1;
    let _e72 = YB_1;
    let _e73 = ZB_1;
    let _e75 = i32(_e72.x);
    let _e79 = bitcast<i32>(_e72.w);
    let _e81 = (_e79 >> bitcast<u32>(2i));
    let _e82 = (_e79 & 3i);
    let _e84 = min(_e75, (_e81 - 1i));
    let _e86 = ((_e71 * _e81) + _e84);
    let _e91 = textureLoad(VB, vec2<i32>((_e86 & 2047i), (_e86 >> bitcast<u32>(11i))), 0i);
    let _e98 = CD.x2_[(max((_e91.w & 65535u), 1u) - 1u)];
    let _e100 = bitcast<vec2<f32>>(_e98.xy);
    let _e102 = (_e98.z & 65535u);
    let _e104 = (_e102 * 4u);
    let _e107 = JB.x2_[_e104];
    let _e108 = bitcast<vec4<f32>>(_e107);
    let _e115 = mat2x2<f32>(vec2<f32>(_e108.x, _e108.y), vec2<f32>(_e108.z, _e108.w));
    let _e119 = JB.x2_[(_e104 + 1u)];
    phi_2286_ = 0f;
    phi_2223_ = 0f;
    if ((_e119.z & 32768u) != 0u) {
        let _e128 = JB.x2_[(_e104 + 2u)];
        phi_2286_ = bitcast<f32>(_e128.y);
        phi_2223_ = bitcast<f32>(_e128.x);
    }
    let _e134 = phi_2286_;
    let _e136 = phi_2223_;
    let _e137 = (_e91.w & 8388608u);
    phi_2313_ = _e72.z;
    phi_2249_ = _e72.y;
    phi_2217_ = _e75;
    if (_e137 != 0u) {
        phi_2313_ = _e73.z;
        phi_2249_ = _e73.y;
        phi_2217_ = i32(_e73.x);
    }
    let _e144 = phi_2313_;
    let _e146 = phi_2249_;
    let _e148 = phi_2217_;
    phi_2232_ = _e86;
    phi_2230_ = _e91;
    phi_2229_ = _e91.w;
    if (_e148 != _e84) {
        let _e151 = ((_e86 + _e148) - _e84);
        let _e156 = textureLoad(VB, vec2<i32>((_e151 & 2047i), (_e151 >> bitcast<u32>(11i))), 0i);
        if ((_e156.w & 8454143u) != (_e91.w & 8454143u)) {
            let _e161 = (_e136 == 0f);
            phi_1363_ = _e161;
            if !(_e161) {
                phi_1363_ = (_e100.x != 0f);
            }
            let _e166 = phi_1363_;
            phi_2234_ = _e86;
            phi_2225_ = _e91;
            if _e166 {
                let _e167 = bitcast<i32>(_e98.w);
                let _e172 = textureLoad(VB, vec2<i32>((_e167 & 2047i), (_e167 >> bitcast<u32>(11i))), 0i);
                phi_2234_ = _e167;
                phi_2225_ = _e172;
            }
            let _e174 = phi_2234_;
            let _e176 = phi_2225_;
            phi_2233_ = _e174;
            phi_2224_ = _e176;
        } else {
            phi_2233_ = _e151;
            phi_2224_ = _e156;
        }
        let _e178 = phi_2233_;
        let _e180 = phi_2224_;
        phi_2232_ = _e178;
        phi_2230_ = _e180;
        phi_2229_ = ((_e180.w & 4286578687u) | _e137);
    }
    let _e185 = phi_2232_;
    let _e187 = phi_2230_;
    let _e189 = phi_2229_;
    let _e190 = (_e189 & 469762048u);
    let _e193 = ((_e190 == 67108864u) && (_e82 == 0i));
    if _e193 {
        let _e196 = f32((_e187.z & 65535u));
        let _e199 = f32((_e187.z >> bitcast<u32>(16i)));
        let _e205 = vec2<i32>(i32((-1f - _e196)), i32(((_e199 - _e196) + 1f)));
        phi_2236_ = _e205;
        if ((_e189 & 8388608u) != 0u) {
            phi_2236_ = -(_e205);
        }
        let _e210 = phi_2236_;
        let _e212 = (_e185 + _e210.x);
        let _e217 = textureLoad(VB, vec2<i32>((_e212 & 2047i), (_e212 >> bitcast<u32>(11i))), 0i);
        let _e219 = (_e185 + _e210.y);
        let _e224 = textureLoad(VB, vec2<i32>((_e219 & 2047i), (_e219 >> bitcast<u32>(11i))), 0i);
        phi_2237_ = _e224;
        if ((_e224.w & 8454143u) != (_e217.w & 8454143u)) {
            let _e230 = bitcast<i32>(_e98.w);
            let _e235 = textureLoad(VB, vec2<i32>((_e230 & 2047i), (_e230 >> bitcast<u32>(11i))), 0i);
            phi_2237_ = _e235;
        }
        let _e237 = phi_2237_;
        let _e240 = (f32(_e217.z) * 0.0000000014629181f);
        let _e243 = (f32(_e237.z) * 0.0000000014629181f);
        let _e244 = (_e243 - _e240);
        phi_2241_ = _e244;
        if (abs(_e244) > 3.1415927f) {
            phi_2241_ = (_e244 - (6.2831855f * sign(_e244)));
        }
        let _e251 = phi_2241_;
        let _e252 = (_e199 + -2f);
        let _e258 = clamp(round(((abs(_e251) * 0.31830987f) * _e252)), 1f, (_e199 + -3f));
        let _e259 = (_e252 - _e258);
        if (_e196 <= _e259) {
            phi_2324_ = _e146;
            if (_e196 == _e259) {
                phi_2324_ = -(_e146);
            }
            let _e268 = phi_2324_;
            phi_2323_ = _e268;
            phi_2264_ = -(((3.1415927f * sign(_e251)) - _e251));
            phi_2257_ = _e259;
            phi_2254_ = _e196;
        } else {
            let _e270 = (_e196 == (_e259 + 1f));
            if _e270 {
                phi_2256_ = 0f;
            } else {
                phi_2256_ = (_e196 - (_e259 + 2f));
            }
            let _e274 = phi_2256_;
            phi_2323_ = select(_e146, 0f, _e270);
            phi_2264_ = _e251;
            phi_2257_ = select(_e258, 0f, _e270);
            phi_2254_ = _e274;
        }
        let _e278 = phi_2323_;
        let _e280 = phi_2264_;
        let _e282 = phi_2257_;
        let _e284 = phi_2254_;
        if (_e284 == _e282) {
            phi_2268_ = _e243;
        } else {
            phi_2268_ = (_e240 + (_e280 * (_e284 / _e282)));
        }
        let _e290 = phi_2268_;
        phi_2346_ = _e240;
        phi_2336_ = _e280;
        phi_2321_ = _e278;
        phi_2267_ = _e290;
    } else {
        phi_2346_ = f32();
        phi_2336_ = f32();
        phi_2321_ = _e146;
        phi_2267_ = (f32(_e187.z) * 0.0000000014629181f);
    }
    let _e295 = phi_2346_;
    let _e297 = phi_2336_;
    let _e299 = phi_2321_;
    let _e301 = phi_2267_;
    let _e305 = vec2<f32>(sin(_e301), -(cos(_e301)));
    let _e307 = bitcast<vec2<f32>>(_e187.xy);
    phi_2319_ = _e134;
    if (_e134 != 0f) {
        phi_2319_ = max(_e134, (1f / length((_e115 * _e305))));
    }
    let _e314 = phi_2319_;
    if (_e136 != 0f) {
        let _e318 = (_e299 * sign(determinant(_e115)));
        let _e320 = ((_e189 & 1048576u) != 0u);
        phi_2405_ = _e318;
        if _e320 {
            phi_2405_ = min(_e318, 0f);
        }
        let _e323 = phi_2405_;
        phi_2416_ = _e323;
        if ((_e189 & 524288u) != 0u) {
            phi_2416_ = max(_e323, 0f);
        }
        let _e328 = phi_2416_;
        let _e329 = (_e314 != 0f);
        if _e329 {
            phi_2408_ = _e314;
        } else {
            let _e330 = (_e115 * _e305);
            phi_2408_ = (((abs(_e330.x) + abs(_e330.y)) * (1f / dot(_e330, _e330))) * 0.5f);
        }
        let _e341 = phi_2408_;
        let _e344 = ((_e341 > _e136) && (_e314 == 0f));
        phi_2470_ = 1f;
        if _e344 {
            phi_2470_ = (_e136 / _e341);
        }
        let _e347 = phi_2470_;
        let _e348 = select(_e136, _e341, _e344);
        let _e349 = (_e348 + _e341);
        let _e350 = (_e305 * _e349);
        let _e351 = (_e328 * _e349);
        let _e358 = (((vec2<f32>(_e351, -(_e351)) + vec2(_e348)) * (0.5f / _e341)) + vec2<f32>(0.5f, 0.5f));
        let _e361 = vec4<f32>(_e358.x, _e358.y, 0f, 0f);
        phi_2491_ = _e350;
        phi_2477_ = _e361;
        if (_e190 > 201326592u) {
            let _e367 = f32((_e187.z & 65535u));
            let _e368 = (_e367 * 0.000015259022f);
            let _e372 = sqrt(max((1f - (_e368 * _e368)), 0f));
            phi_2444_ = _e372;
            if (((_e189 & 4194304u) != 0u) == _e320) {
                phi_2444_ = -(_e372);
            }
            let _e376 = phi_2444_;
            let _e381 = (mat2x2<f32>(vec2<f32>(_e368, _e376), vec2<f32>(-(_e376), _e368)) * _e305);
            let _e382 = (_e115 * _e381);
            let _e390 = ((abs(_e382.x) + abs(_e382.y)) * (1f / dot(_e382, _e382)));
            let _e391 = (_e190 == 402653184u);
            phi_1678_ = _e391;
            if !(_e391) {
                phi_1678_ = ((_e190 == 335544320u) && (_e368 >= 0.25f));
            }
            let _e397 = phi_1678_;
            if _e397 {
                phi_2449_ = (_e348 * (1f / max(_e368, select(0.25f, 1f, ((_e189 & 33554432u) != 0u)))));
            } else {
                phi_2449_ = ((_e348 * _e368) + (_e390 * 0.5f));
            }
            let _e408 = phi_2449_;
            let _e410 = (_e408 + (_e390 * 0.5f));
            phi_2460_ = _e350;
            if ((_e189 & 2097152u) != 0u) {
                if (_e349 <= ((_e410 * _e368) + (_e341 * 0.125f))) {
                    phi_2461_ = (_e381 * (_e349 * (65535f / _e367)));
                } else {
                    let _e420 = (_e381 * _e410);
                    phi_2461_ = (vec2<f32>(dot(_e350, _e350), dot(_e420, _e420)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e350, _e420)));
                }
                let _e428 = phi_2461_;
                phi_2460_ = _e428;
            }
            let _e430 = phi_2460_;
            let _e435 = ((_e410 - dot((_e430 * abs(_e328)), _e381)) / _e390);
            if _e320 {
                phi_2478_ = vec4<f32>(_e361.x, _e435, _e361.z, _e361.w);
            } else {
                phi_2478_ = vec4<f32>(_e435, _e361.y, _e361.z, _e361.w);
            }
            let _e447 = phi_2478_;
            phi_2491_ = _e430;
            phi_2477_ = _e447;
        }
        let _e449 = phi_2491_;
        let _e451 = phi_2477_;
        let _e453 = (_e451.xy * _e347);
        let _e459 = vec4<f32>(_e453.x, _e451.y, _e451.z, _e451.w);
        let _e466 = vec4<f32>(_e459.x, max(_e453.y, 0.0001f), _e459.z, _e459.w);
        phi_2525_ = _e466;
        if _e329 {
            phi_2525_ = vec4<f32>((-2f - _e453.x), _e466.y, _e466.z, _e466.w);
        }
        let _e474 = phi_2525_;
        phi_2529_ = (_e82 != 0i);
        phi_2523_ = _e474;
        phi_2517_ = (_e115 * (_e449 * _e328));
        phi_2493_ = _e307;
    } else {
        let _e478 = vec4<f32>(_e144, -1f, 0f, 0f);
        if (_e314 != 0f) {
            let _e484 = vec4<f32>(_e478.x, -2f, _e478.z, _e478.w);
            let _e489 = vec4<f32>(_e484.x, _e484.y, 1000000f, _e484.w);
            phi_2401_ = vec4<f32>(_e489.x, _e489.y, _e489.z, _e144);
            if _e193 {
                phi_2357_ = _e297;
                phi_2356_ = _e295;
                if (_e297 < 0f) {
                    phi_2357_ = -(_e297);
                    phi_2356_ = (_e295 + _e297);
                }
                let _e499 = phi_2357_;
                let _e501 = phi_2356_;
                let _e503 = ((_e301 - _e501) + 1.5707964f);
                let _e509 = clamp(((_e503 - (floor((_e503 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e499);
                phi_2358_ = _e509;
                if (_e509 > (_e499 * 0.5f)) {
                    phi_2358_ = (_e499 - _e509);
                }
                let _e514 = phi_2358_;
                let _e521 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e514), cos(_e514)) * abs(_e299))) * 0.5f);
                if (abs((_e499 - 1.5707964f)) < 0.001f) {
                    phi_2385_ = 0f;
                    phi_2383_ = 0f;
                } else {
                    let _e525 = tan(_e499);
                    let _e530 = (sign((1.5707964f - _e499)) / max(abs(_e525), 0.000001f));
                    if (_e530 >= 0f) {
                        phi_2362_ = (_e521.y - ((1f - _e521.x) * _e525));
                    } else {
                        phi_2362_ = (_e521.y + (_e521.x * _e525));
                    }
                    let _e542 = phi_2362_;
                    phi_2385_ = _e542;
                    phi_2383_ = _e530;
                }
                let _e544 = phi_2385_;
                let _e546 = phi_2383_;
                phi_2401_ = vec4<f32>((max(_e521.x, 0f) + 0.25f), (-2f - _e521.y), _e546, _e544);
            }
            let _e554 = phi_2401_;
            phi_2522_ = (_e115 * (_e305 * (_e299 * _e314)));
            phi_2400_ = _e554;
        } else {
            phi_2522_ = (sign(((_e305 * _e299) * _naga_inverse_2x2_f32(_e115))) * 0.5f);
            phi_2400_ = _e478;
        }
        let _e564 = phi_2522_;
        let _e566 = phi_2400_;
        phi_2528_ = _e566;
        if (((_e189 & 8388608u) != 0u) != ((_e189 & 16777216u) != 0u)) {
            phi_2528_ = (_e566 * vec4<f32>(-1f, 1f, 1f, 1f));
        }
        let _e574 = phi_2528_;
        phi_2529_ = (((_e189 & 2147483648u) != 0u) && (_e82 != 1i));
        phi_2523_ = _e574;
        phi_2517_ = _e564;
        phi_2493_ = select(_e307, _e100, vec2((_e82 == 2i)));
    }
    let _e583 = phi_2529_;
    let _e585 = phi_2523_;
    let _e587 = phi_2517_;
    let _e589 = phi_2493_;
    let _e592 = (((_e115 * _e589) + _e587) + bitcast<vec2<f32>>(_e119.xy));
    let _e595 = j.Gi;
    let _e598 = select(_e585.xy, vec2<f32>(1f, -1f), vec2((_e595 != 0u)));
    let _e604 = vec4<f32>(_e598.x, _e585.y, _e585.z, _e585.w);
    if !(_e583) {
        S = vec4<f32>(_e604.x, _e598.y, _e604.z, _e604.w);
        I0_ = _e102;
        let _e613 = j.Rg;
        let _e615 = j.Sg;
        phi_2561_ = vec4<f32>(((_e592.x * _e613) - 1f), ((_e592.y * _e615) - sign(_e615)), 0f, 1f);
    } else {
        let _e625 = j.j3_;
        phi_2561_ = vec4(_e625);
    }
    let _e628 = phi_2561_;
    unnamed.gl_Position = _e628;
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
    let _e15 = I0_;
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
