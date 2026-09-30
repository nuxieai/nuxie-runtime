enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct TB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    hh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    eh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(4) @interpolate(flat, either) member: vec2<f32>,
    @location(6) @interpolate(flat, either) member_1: f32,
    @location(0) member_2: vec4<f32>,
    @location(9) member_3: vec3<f32>,
}

@id(0) override Hh: bool = true;
@id(2) override Jh: bool = true;
@id(1) override Ih: bool = true;
@id(8) override Ph: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(5)
var HD: texture_2d<u32>;
@group(0) @binding(2)
var OB: texture_2d<u32>;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> VB_1: vec4<f32>;
var<private> WB_1: vec4<f32>;
@group(0) @binding(3)
var CD: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> Y1_: vec2<f32>;
var<private> g1_: f32;
@group(0) @binding(4)
var PB: texture_2d<f32>;
var<private> X1_: vec4<f32>;
var<private> C2_: vec3<f32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var ca: sampler;

fn main_1() {
    var phi_2358_: f32;
    var phi_2330_: i32;
    var phi_1518_: bool;
    var phi_2343_: i32;
    var phi_2335_: vec4<u32>;
    var phi_2342_: i32;
    var phi_2334_: vec4<u32>;
    var phi_2341_: i32;
    var phi_2339_: vec4<u32>;
    var phi_2338_: u32;
    var phi_2345_: vec2<i32>;
    var phi_2346_: vec4<u32>;
    var phi_2350_: f32;
    var phi_2421_: f32;
    var phi_2364_: f32;
    var phi_2420_: f32;
    var phi_2368_: f32;
    var phi_2365_: f32;
    var phi_2362_: f32;
    var phi_2372_: f32;
    var phi_2418_: f32;
    var phi_2371_: f32;
    var phi_2427_: f32;
    var phi_2424_: f32;
    var phi_2481_: f32;
    var phi_2453_: i32;
    var phi_2463_: f32;
    var phi_1830_: bool;
    var phi_2470_: f32;
    var phi_2491_: vec2<f32>;
    var phi_2490_: vec2<f32>;
    var phi_2489_: vec2<f32>;
    var phi_2514_: bool;
    var phi_2509_: vec2<f32>;
    var phi_2492_: vec2<f32>;
    var phi_2539_: u32;
    var phi_2540_: f32;
    var phi_2541_: f32;
    var phi_2582_: f32;
    var phi_2580_: vec4<f32>;
    var phi_2581_: vec4<f32>;
    var phi_1204_: bool;
    var phi_2597_: vec4<f32>;

    let _e81 = gl_InstanceIndex_1;
    let _e82 = VB_1;
    let _e83 = WB_1;
    let _e85 = i32(_e82.x);
    let _e88 = bitcast<i32>(_e82.w);
    let _e90 = (_e88 >> bitcast<u32>(2i));
    let _e91 = (_e88 & 3i);
    let _e93 = min(_e85, (_e90 - 1i));
    let _e95 = ((_e81 * _e90) + _e93);
    let _e100 = textureLoad(JC, vec2<i32>((_e95 & 2047i), (_e95 >> bitcast<u32>(11i))), 0i);
    let _e104 = (max((_e100.w & 65535u), 1u) - 1u);
    let _e111 = textureLoad(HD, vec2<i32>(bitcast<i32>((_e104 & 255u)), bitcast<i32>((_e104 >> bitcast<u32>(8i)))), 0i);
    let _e113 = bitcast<vec2<f32>>(_e111.xy);
    let _e115 = (_e111.z & 65535u);
    let _e117 = (_e115 * 4u);
    let _e124 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e117 & 255u)), bitcast<i32>((_e117 >> bitcast<u32>(8i)))), 0i);
    let _e125 = bitcast<vec4<f32>>(_e124);
    let _e132 = mat2x2<f32>(vec2<f32>(_e125.x, _e125.y), vec2<f32>(_e125.z, _e125.w));
    let _e133 = (_e117 + 1u);
    let _e140 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e133 & 255u)), bitcast<i32>((_e133 >> bitcast<u32>(8i)))), 0i);
    let _e144 = bitcast<f32>(_e140.z);
    let _e146 = bitcast<f32>(_e140.w);
    let _e147 = (_e100.w & 8388608u);
    phi_2358_ = _e82.y;
    phi_2330_ = _e85;
    if (_e147 != 0u) {
        phi_2358_ = _e83.y;
        phi_2330_ = i32(_e83.x);
    }
    let _e153 = phi_2358_;
    let _e155 = phi_2330_;
    phi_2341_ = _e95;
    phi_2339_ = _e100;
    phi_2338_ = _e100.w;
    if (_e155 != _e93) {
        let _e158 = ((_e95 + _e155) - _e93);
        let _e163 = textureLoad(JC, vec2<i32>((_e158 & 2047i), (_e158 >> bitcast<u32>(11i))), 0i);
        if ((_e163.w & 8454143u) != (_e100.w & 8454143u)) {
            let _e168 = (_e144 == 0f);
            phi_1518_ = _e168;
            if !(_e168) {
                phi_1518_ = (_e113.x != 0f);
            }
            let _e173 = phi_1518_;
            phi_2343_ = _e95;
            phi_2335_ = _e100;
            if _e173 {
                let _e174 = bitcast<i32>(_e111.w);
                let _e179 = textureLoad(JC, vec2<i32>((_e174 & 2047i), (_e174 >> bitcast<u32>(11i))), 0i);
                phi_2343_ = _e174;
                phi_2335_ = _e179;
            }
            let _e181 = phi_2343_;
            let _e183 = phi_2335_;
            phi_2342_ = _e181;
            phi_2334_ = _e183;
        } else {
            phi_2342_ = _e158;
            phi_2334_ = _e163;
        }
        let _e185 = phi_2342_;
        let _e187 = phi_2334_;
        phi_2341_ = _e185;
        phi_2339_ = _e187;
        phi_2338_ = ((_e187.w & 4286578687u) | _e147);
    }
    let _e192 = phi_2341_;
    let _e194 = phi_2339_;
    let _e196 = phi_2338_;
    let _e197 = (_e196 & 469762048u);
    if ((_e197 == 67108864u) && (_e91 == 0i)) {
        let _e203 = f32((_e194.z & 65535u));
        let _e206 = f32((_e194.z >> bitcast<u32>(16i)));
        let _e212 = vec2<i32>(i32((-1f - _e203)), i32(((_e206 - _e203) + 1f)));
        phi_2345_ = _e212;
        if ((_e196 & 8388608u) != 0u) {
            phi_2345_ = -(_e212);
        }
        let _e217 = phi_2345_;
        let _e219 = (_e192 + _e217.x);
        let _e224 = textureLoad(JC, vec2<i32>((_e219 & 2047i), (_e219 >> bitcast<u32>(11i))), 0i);
        let _e226 = (_e192 + _e217.y);
        let _e231 = textureLoad(JC, vec2<i32>((_e226 & 2047i), (_e226 >> bitcast<u32>(11i))), 0i);
        phi_2346_ = _e231;
        if ((_e231.w & 8454143u) != (_e224.w & 8454143u)) {
            let _e237 = bitcast<i32>(_e111.w);
            let _e242 = textureLoad(JC, vec2<i32>((_e237 & 2047i), (_e237 >> bitcast<u32>(11i))), 0i);
            phi_2346_ = _e242;
        }
        let _e244 = phi_2346_;
        let _e246 = bitcast<f32>(_e224.z);
        let _e248 = bitcast<f32>(_e244.z);
        let _e249 = (_e248 - _e246);
        phi_2350_ = _e249;
        if (abs(_e249) > 3.1415927f) {
            phi_2350_ = (_e249 - (6.2831855f * sign(_e249)));
        }
        let _e256 = phi_2350_;
        let _e257 = (_e206 + -2f);
        let _e263 = clamp(round(((abs(_e256) * 0.31830987f) * _e257)), 1f, (_e206 + -3f));
        let _e264 = (_e257 - _e263);
        if (_e203 <= _e264) {
            phi_2421_ = _e153;
            if (_e203 == _e264) {
                phi_2421_ = -(_e153);
            }
            let _e273 = phi_2421_;
            phi_2420_ = _e273;
            phi_2368_ = -(((3.1415927f * sign(_e256)) - _e256));
            phi_2365_ = _e264;
            phi_2362_ = _e203;
        } else {
            let _e275 = (_e203 == (_e264 + 1f));
            if _e275 {
                phi_2364_ = 0f;
            } else {
                phi_2364_ = (_e203 - (_e264 + 2f));
            }
            let _e279 = phi_2364_;
            phi_2420_ = select(_e153, 0f, _e275);
            phi_2368_ = _e256;
            phi_2365_ = select(_e263, 0f, _e275);
            phi_2362_ = _e279;
        }
        let _e283 = phi_2420_;
        let _e285 = phi_2368_;
        let _e287 = phi_2365_;
        let _e289 = phi_2362_;
        if (_e289 == _e287) {
            phi_2372_ = _e248;
        } else {
            phi_2372_ = (_e246 + (_e285 * (_e289 / _e287)));
        }
        let _e295 = phi_2372_;
        phi_2418_ = _e283;
        phi_2371_ = _e295;
    } else {
        phi_2418_ = _e153;
        phi_2371_ = bitcast<f32>(_e194.z);
    }
    let _e299 = phi_2418_;
    let _e301 = phi_2371_;
    let _e305 = vec2<f32>(sin(_e301), -(cos(_e301)));
    let _e307 = bitcast<vec2<f32>>(_e194.xy);
    phi_2427_ = _e146;
    if (_e146 != 0f) {
        phi_2427_ = max(_e146, (1f / length((_e132 * _e305))));
    }
    let _e314 = phi_2427_;
    if (_e144 != 0f) {
        let _e318 = (_e299 * sign(determinant(_e132)));
        let _e320 = ((_e196 & 1048576u) != 0u);
        phi_2424_ = _e318;
        if _e320 {
            phi_2424_ = min(_e318, 0f);
        }
        let _e323 = phi_2424_;
        phi_2481_ = _e323;
        if ((_e196 & 524288u) != 0u) {
            phi_2481_ = max(_e323, 0f);
        }
        let _e328 = phi_2481_;
        let _e330 = select(0f, _e314, (_e314 != 0f));
        let _e334 = select(_e144, _e330, ((_e330 > _e144) && (_e314 == 0f)));
        let _e335 = (_e334 + _e330);
        let _e336 = (_e305 * _e335);
        phi_2489_ = _e336;
        if (_e197 > 134217728u) {
            let _e338 = (_e196 & 4194304u);
            let _e340 = select(2i, -2i, (_e338 == 0u));
            phi_2453_ = _e340;
            if ((_e196 & 8388608u) != 0u) {
                phi_2453_ = -(_e340);
            }
            let _e345 = phi_2453_;
            let _e346 = (_e192 + _e345);
            let _e351 = textureLoad(JC, vec2<i32>((_e346 & 2047i), (_e346 >> bitcast<u32>(11i))), 0i);
            let _e355 = abs((bitcast<f32>(_e351.z) - _e301));
            phi_2463_ = _e355;
            if (_e355 > 3.1415927f) {
                phi_2463_ = (6.2831855f - _e355);
            }
            let _e359 = phi_2463_;
            let _e364 = ((_e359 * select(0.5f, -0.5f, ((_e338 != 0u) == _e320))) + _e301);
            let _e368 = vec2<f32>(sin(_e364), -(cos(_e364)));
            let _e369 = (_e132 * _e368);
            let _e379 = cos((_e359 * 0.5f));
            let _e380 = (_e197 == 335544320u);
            phi_1830_ = _e380;
            if !(_e380) {
                phi_1830_ = ((_e197 == 268435456u) && (_e379 >= 0.25f));
            }
            let _e386 = phi_1830_;
            if _e386 {
                phi_2470_ = (_e334 * (1f / max(_e379, select(0.25f, 1f, ((_e196 & 33554432u) != 0u)))));
            } else {
                phi_2470_ = ((_e334 * _e379) + (((abs(_e369.x) + abs(_e369.y)) * (1f / dot(_e369, _e369))) * 0.5f));
            }
            let _e397 = phi_2470_;
            phi_2490_ = _e336;
            if ((_e196 & 2097152u) != 0u) {
                if (_e335 <= ((_e397 * _e379) + (_e330 * 0.125f))) {
                    phi_2491_ = (_e368 * (_e335 * (1f / _e379)));
                } else {
                    let _e407 = (_e368 * _e397);
                    phi_2491_ = (vec2<f32>(dot(_e336, _e336), dot(_e407, _e407)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e336, _e407)));
                }
                let _e415 = phi_2491_;
                phi_2490_ = _e415;
            }
            let _e417 = phi_2490_;
            phi_2489_ = _e417;
        }
        let _e419 = phi_2489_;
        phi_2514_ = (_e91 != 0i);
        phi_2509_ = (_e132 * (_e419 * _e328));
        phi_2492_ = _e307;
    } else {
        phi_2514_ = (((_e196 & 2147483648u) != 0u) && (_e91 != 1i));
        phi_2509_ = vec2<f32>(0f, 0f);
        phi_2492_ = select(_e307, _e113, vec2((_e91 == 2i)));
    }
    let _e431 = phi_2514_;
    let _e433 = phi_2509_;
    let _e435 = phi_2492_;
    let _e438 = (((_e132 * _e435) + _e433) + bitcast<vec2<f32>>(_e140.xy));
    let _e439 = (_e117 + 2u);
    let _e446 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e439 & 255u)), bitcast<i32>((_e439 >> bitcast<u32>(8i)))), 0i);
    let _e454 = textureLoad(CD, vec2<i32>(bitcast<i32>((_e111.z & 255u)), bitcast<i32>((_e115 >> bitcast<u32>(8i)))), 0i);
    let _e456 = (_e454.x & 15u);
    if Hh {
        let _e457 = (_e456 == 0u);
        if _e457 {
            phi_2539_ = _e454.y;
        } else {
            phi_2539_ = _e454.x;
        }
        let _e460 = phi_2539_;
        let _e462 = (_e460 >> bitcast<u32>(16i));
        let _e464 = j.c6_;
        if (_e462 == 0u) {
            phi_2540_ = 0f;
        } else {
            phi_2540_ = unpack2x16float(((_e462 + 1023u) * _e464)).x;
        }
        let _e471 = phi_2540_;
        phi_2541_ = _e471;
        if _e457 {
            phi_2541_ = -(_e471);
        }
        let _e474 = phi_2541_;
        Y1_[0u] = _e474;
    }
    if Jh {
        g1_ = f32(((_e454.x >> bitcast<u32>(4i)) & 15u));
    }
    if Ih {
        let _e480 = (_e115 * 8u);
        let _e481 = (_e480 + 2u);
        let _e488 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e481 & 255u)), bitcast<i32>((_e481 >> bitcast<u32>(8i)))), 0i);
        let _e496 = (_e480 + 3u);
        let _e503 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e496 & 255u)), bitcast<i32>((_e496 >> bitcast<u32>(8i)))), 0i);
        if any((_e488 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e508 = ((mat2x2<f32>(vec2<f32>(_e488.x, _e488.y), vec2<f32>(_e488.z, _e488.w)) * _e438) + _e503.xy);
            unnamed.gl_ClipDistance[0i] = (_e508.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e508.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e508.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e508.y);
        } else {
            let _e524 = (_e503.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e524;
            unnamed.gl_ClipDistance[2i] = _e524;
            unnamed.gl_ClipDistance[1i] = _e524;
            unnamed.gl_ClipDistance[0i] = _e524;
        }
    }
    if (_e456 == 1u) {
        X1_ = unpack4x8unorm(_e454.y);
    } else {
        if (Hh && (_e456 == 0u)) {
            let _e539 = (_e454.x >> bitcast<u32>(16i));
            let _e541 = j.c6_;
            if (_e539 == 0u) {
                phi_2582_ = 0f;
            } else {
                phi_2582_ = unpack2x16float(((_e539 + 1023u) * _e541)).x;
            }
            let _e548 = phi_2582_;
            Y1_[1u] = _e548;
        } else {
            let _e550 = (_e115 * 8u);
            let _e557 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e550 & 255u)), bitcast<i32>((_e550 >> bitcast<u32>(8i)))), 0i);
            let _e565 = (_e550 + 1u);
            let _e572 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e565 & 255u)), bitcast<i32>((_e565 >> bitcast<u32>(8i)))), 0i);
            let _e581 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e454.y));
            let _e583 = ((mat2x2<f32>(vec2<f32>(_e557.x, _e557.y), vec2<f32>(_e557.z, _e557.w)) * _e438) + _e572.xy);
            if (_e572.z > 0.9f) {
                phi_2580_ = vec4<f32>(_e581.x, _e581.y, 2f, _e581.w);
            } else {
                phi_2580_ = vec4<f32>(_e581.x, _e581.y, _e572.w, _e581.w);
            }
            let _e598 = phi_2580_;
            if (f32(_e456) == 2f) {
                let _e605 = vec4<f32>(_e583.x, _e598.y, _e598.z, _e598.w);
                phi_2581_ = vec4<f32>(_e605.x, 0f, _e605.z, _e605.w);
            } else {
                let _e617 = vec4<f32>(_e598.x, _e598.y, -(_e598.z), _e598.w);
                let _e623 = vec4<f32>(_e583.x, _e617.y, _e617.z, _e617.w);
                phi_2581_ = vec4<f32>(_e623.x, _e583.y, _e623.z, _e623.w);
            }
            let _e631 = phi_2581_;
            X1_ = _e631;
            let _e633 = X1_[3u];
            X1_[3u] = -(_e633);
        }
    }
    phi_1204_ = Ph;
    if Ph {
        phi_1204_ = ((_e454.x & 2048u) != 0u);
    }
    let _e638 = phi_1204_;
    if _e638 {
        let _e639 = (_e115 * 8u);
        let _e640 = (_e639 + 4u);
        let _e647 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e640 & 255u)), bitcast<i32>((_e640 >> bitcast<u32>(8i)))), 0i);
        let _e655 = (_e639 + 5u);
        let _e662 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e655 & 255u)), bitcast<i32>((_e655 >> bitcast<u32>(8i)))), 0i);
        let _e665 = ((mat2x2<f32>(vec2<f32>(_e647.x, _e647.y), vec2<f32>(_e647.z, _e647.w)) * _e438) + _e662.xy);
        C2_ = vec3<f32>(_e665.x, _e665.y, (1f + _e662.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e431) {
        let _e673 = j.Hf;
        let _e675 = j.If;
        let _e683 = vec4<f32>(((_e438.x * _e673) - 1f), ((_e438.y * _e675) - sign(_e675)), 0f, 1f);
        phi_2597_ = vec4<f32>(_e683.x, _e683.y, ((f32(((_e446.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e683.w);
    } else {
        let _e696 = j.W2_;
        phi_2597_ = vec4(_e696);
    }
    let _e699 = phi_2597_;
    unnamed.gl_Position = _e699;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) VB: vec4<f32>, @location(1) WB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    VB_1 = VB;
    WB_1 = WB;
    main_1();
    let _e17 = unnamed.gl_Position;
    let _e18 = unnamed.gl_ClipDistance;
    let _e19 = Y1_;
    let _e20 = g1_;
    let _e21 = X1_;
    let _e22 = C2_;
    return VertexOutput(_e17, _e18, _e19, _e20, _e21, _e22);
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
