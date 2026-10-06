enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct UB {
    Rc: f32,
    Ud: f32,
    dg: f32,
    eg: f32,
    A6_: u32,
    Y9_: u32,
    Pf: u32,
    Qf: u32,
    i8_: vec4<i32>,
    Nh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Rh: f32,
    U4_: u32,
    a3_: f32,
    Wd: f32,
    Jf: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Kh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(4) @interpolate(flat, either) member: vec2<f32>,
    @location(6) @interpolate(flat, either) member_1: f32,
    @location(0) member_2: vec4<f32>,
    @location(9) member_3: vec3<f32>,
}

@id(0) override mi: bool = true;
@id(2) override oi: bool = true;
@id(1) override ni: bool = true;
@id(8) override ui: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(5)
var ZC: texture_2d<u32>;
@group(0) @binding(2)
var LB: texture_2d<u32>;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> WB_1: vec4<f32>;
var<private> XB_1: vec4<f32>;
@group(0) @binding(3)
var WC: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> l1_: vec2<f32>;
var<private> Q0_: f32;
@group(0) @binding(4)
var JB: texture_2d<f32>;
var<private> a1_: vec4<f32>;
var<private> v1_: vec3<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var xa: sampler;

fn main_1() {
    var phi_2362_: f32;
    var phi_2334_: i32;
    var phi_1530_: bool;
    var phi_2347_: i32;
    var phi_2339_: vec4<u32>;
    var phi_2346_: i32;
    var phi_2338_: vec4<u32>;
    var phi_2345_: i32;
    var phi_2343_: vec4<u32>;
    var phi_2342_: u32;
    var phi_2349_: vec2<i32>;
    var phi_2350_: vec4<u32>;
    var phi_2354_: f32;
    var phi_2425_: f32;
    var phi_2368_: f32;
    var phi_2424_: f32;
    var phi_2372_: f32;
    var phi_2369_: f32;
    var phi_2366_: f32;
    var phi_2376_: f32;
    var phi_2422_: f32;
    var phi_2375_: f32;
    var phi_2431_: f32;
    var phi_2428_: f32;
    var phi_2460_: f32;
    var phi_2446_: f32;
    var phi_1818_: bool;
    var phi_2451_: f32;
    var phi_2468_: vec2<f32>;
    var phi_2467_: vec2<f32>;
    var phi_2466_: vec2<f32>;
    var phi_2489_: bool;
    var phi_2484_: vec2<f32>;
    var phi_2469_: vec2<f32>;
    var phi_2512_: u32;
    var phi_2513_: f32;
    var phi_2514_: f32;
    var phi_2553_: f32;
    var phi_2551_: vec4<f32>;
    var phi_2552_: vec4<f32>;
    var phi_1198_: bool;
    var phi_2554_: f32;
    var phi_2570_: vec4<f32>;

    let _e85 = gl_InstanceIndex_1;
    let _e86 = WB_1;
    let _e87 = XB_1;
    let _e89 = i32(_e86.x);
    let _e92 = bitcast<i32>(_e86.w);
    let _e94 = (_e92 >> bitcast<u32>(2i));
    let _e95 = (_e92 & 3i);
    let _e97 = min(_e89, (_e94 - 1i));
    let _e99 = ((_e85 * _e94) + _e97);
    let _e104 = textureLoad(TB, vec2<i32>((_e99 & 2047i), (_e99 >> bitcast<u32>(11i))), 0i);
    let _e108 = (max((_e104.w & 65535u), 1u) - 1u);
    let _e115 = textureLoad(ZC, vec2<i32>(bitcast<i32>((_e108 & 255u)), bitcast<i32>((_e108 >> bitcast<u32>(8i)))), 0i);
    let _e117 = bitcast<vec2<f32>>(_e115.xy);
    let _e119 = (_e115.z & 65535u);
    let _e121 = (_e119 * 4u);
    let _e128 = textureLoad(LB, vec2<i32>(bitcast<i32>((_e121 & 255u)), bitcast<i32>((_e121 >> bitcast<u32>(8i)))), 0i);
    let _e129 = bitcast<vec4<f32>>(_e128);
    let _e136 = mat2x2<f32>(vec2<f32>(_e129.x, _e129.y), vec2<f32>(_e129.z, _e129.w));
    let _e137 = (_e121 + 1u);
    let _e144 = textureLoad(LB, vec2<i32>(bitcast<i32>((_e137 & 255u)), bitcast<i32>((_e137 >> bitcast<u32>(8i)))), 0i);
    let _e148 = bitcast<f32>(_e144.z);
    let _e150 = bitcast<f32>(_e144.w);
    let _e151 = (_e104.w & 8388608u);
    phi_2362_ = _e86.y;
    phi_2334_ = _e89;
    if (_e151 != 0u) {
        phi_2362_ = _e87.y;
        phi_2334_ = i32(_e87.x);
    }
    let _e157 = phi_2362_;
    let _e159 = phi_2334_;
    phi_2345_ = _e99;
    phi_2343_ = _e104;
    phi_2342_ = _e104.w;
    if (_e159 != _e97) {
        let _e162 = ((_e99 + _e159) - _e97);
        let _e167 = textureLoad(TB, vec2<i32>((_e162 & 2047i), (_e162 >> bitcast<u32>(11i))), 0i);
        if ((_e167.w & 8454143u) != (_e104.w & 8454143u)) {
            let _e172 = (_e148 == 0f);
            phi_1530_ = _e172;
            if !(_e172) {
                phi_1530_ = (_e117.x != 0f);
            }
            let _e177 = phi_1530_;
            phi_2347_ = _e99;
            phi_2339_ = _e104;
            if _e177 {
                let _e178 = bitcast<i32>(_e115.w);
                let _e183 = textureLoad(TB, vec2<i32>((_e178 & 2047i), (_e178 >> bitcast<u32>(11i))), 0i);
                phi_2347_ = _e178;
                phi_2339_ = _e183;
            }
            let _e185 = phi_2347_;
            let _e187 = phi_2339_;
            phi_2346_ = _e185;
            phi_2338_ = _e187;
        } else {
            phi_2346_ = _e162;
            phi_2338_ = _e167;
        }
        let _e189 = phi_2346_;
        let _e191 = phi_2338_;
        phi_2345_ = _e189;
        phi_2343_ = _e191;
        phi_2342_ = ((_e191.w & 4286578687u) | _e151);
    }
    let _e196 = phi_2345_;
    let _e198 = phi_2343_;
    let _e200 = phi_2342_;
    let _e201 = (_e200 & 469762048u);
    if ((_e201 == 67108864u) && (_e95 == 0i)) {
        let _e207 = f32((_e198.z & 65535u));
        let _e210 = f32((_e198.z >> bitcast<u32>(16i)));
        let _e216 = vec2<i32>(i32((-1f - _e207)), i32(((_e210 - _e207) + 1f)));
        phi_2349_ = _e216;
        if ((_e200 & 8388608u) != 0u) {
            phi_2349_ = -(_e216);
        }
        let _e221 = phi_2349_;
        let _e223 = (_e196 + _e221.x);
        let _e228 = textureLoad(TB, vec2<i32>((_e223 & 2047i), (_e223 >> bitcast<u32>(11i))), 0i);
        let _e230 = (_e196 + _e221.y);
        let _e235 = textureLoad(TB, vec2<i32>((_e230 & 2047i), (_e230 >> bitcast<u32>(11i))), 0i);
        phi_2350_ = _e235;
        if ((_e235.w & 8454143u) != (_e228.w & 8454143u)) {
            let _e241 = bitcast<i32>(_e115.w);
            let _e246 = textureLoad(TB, vec2<i32>((_e241 & 2047i), (_e241 >> bitcast<u32>(11i))), 0i);
            phi_2350_ = _e246;
        }
        let _e248 = phi_2350_;
        let _e251 = (f32(_e228.z) * 0.0000000014629181f);
        let _e254 = (f32(_e248.z) * 0.0000000014629181f);
        let _e255 = (_e254 - _e251);
        phi_2354_ = _e255;
        if (abs(_e255) > 3.1415927f) {
            phi_2354_ = (_e255 - (6.2831855f * sign(_e255)));
        }
        let _e262 = phi_2354_;
        let _e263 = (_e210 + -2f);
        let _e269 = clamp(round(((abs(_e262) * 0.31830987f) * _e263)), 1f, (_e210 + -3f));
        let _e270 = (_e263 - _e269);
        if (_e207 <= _e270) {
            phi_2425_ = _e157;
            if (_e207 == _e270) {
                phi_2425_ = -(_e157);
            }
            let _e279 = phi_2425_;
            phi_2424_ = _e279;
            phi_2372_ = -(((3.1415927f * sign(_e262)) - _e262));
            phi_2369_ = _e270;
            phi_2366_ = _e207;
        } else {
            let _e281 = (_e207 == (_e270 + 1f));
            if _e281 {
                phi_2368_ = 0f;
            } else {
                phi_2368_ = (_e207 - (_e270 + 2f));
            }
            let _e285 = phi_2368_;
            phi_2424_ = select(_e157, 0f, _e281);
            phi_2372_ = _e262;
            phi_2369_ = select(_e269, 0f, _e281);
            phi_2366_ = _e285;
        }
        let _e289 = phi_2424_;
        let _e291 = phi_2372_;
        let _e293 = phi_2369_;
        let _e295 = phi_2366_;
        if (_e295 == _e293) {
            phi_2376_ = _e254;
        } else {
            phi_2376_ = (_e251 + (_e291 * (_e295 / _e293)));
        }
        let _e301 = phi_2376_;
        phi_2422_ = _e289;
        phi_2375_ = _e301;
    } else {
        phi_2422_ = _e157;
        phi_2375_ = (f32(_e198.z) * 0.0000000014629181f);
    }
    let _e306 = phi_2422_;
    let _e308 = phi_2375_;
    let _e312 = vec2<f32>(sin(_e308), -(cos(_e308)));
    let _e314 = bitcast<vec2<f32>>(_e198.xy);
    phi_2431_ = _e150;
    if (_e150 != 0f) {
        phi_2431_ = max(_e150, (1f / length((_e136 * _e312))));
    }
    let _e321 = phi_2431_;
    if (_e148 != 0f) {
        let _e325 = (_e306 * sign(determinant(_e136)));
        let _e327 = ((_e200 & 1048576u) != 0u);
        phi_2428_ = _e325;
        if _e327 {
            phi_2428_ = min(_e325, 0f);
        }
        let _e330 = phi_2428_;
        phi_2460_ = _e330;
        if ((_e200 & 524288u) != 0u) {
            phi_2460_ = max(_e330, 0f);
        }
        let _e335 = phi_2460_;
        let _e337 = select(0f, _e321, (_e321 != 0f));
        let _e341 = select(_e148, _e337, ((_e337 > _e148) && (_e321 == 0f)));
        let _e342 = (_e341 + _e337);
        let _e343 = (_e312 * _e342);
        phi_2466_ = _e343;
        if (_e201 > 134217728u) {
            let _e349 = f32((_e198.z & 65535u));
            let _e350 = (_e349 * 0.000015259022f);
            let _e354 = sqrt(max((1f - (_e350 * _e350)), 0f));
            phi_2446_ = _e354;
            if (((_e200 & 4194304u) != 0u) == _e327) {
                phi_2446_ = -(_e354);
            }
            let _e358 = phi_2446_;
            let _e363 = (mat2x2<f32>(vec2<f32>(_e350, _e358), vec2<f32>(-(_e358), _e350)) * _e312);
            let _e364 = (_e136 * _e363);
            let _e373 = (_e201 == 335544320u);
            phi_1818_ = _e373;
            if !(_e373) {
                phi_1818_ = ((_e201 == 268435456u) && (_e350 >= 0.25f));
            }
            let _e379 = phi_1818_;
            if _e379 {
                phi_2451_ = (_e341 * (1f / max(_e350, select(0.25f, 1f, ((_e200 & 33554432u) != 0u)))));
            } else {
                phi_2451_ = ((_e341 * _e350) + (((abs(_e364.x) + abs(_e364.y)) * (1f / dot(_e364, _e364))) * 0.5f));
            }
            let _e390 = phi_2451_;
            phi_2467_ = _e343;
            if ((_e200 & 2097152u) != 0u) {
                if (_e342 <= ((_e390 * _e350) + (_e337 * 0.125f))) {
                    phi_2468_ = (_e363 * (_e342 * (65535f / _e349)));
                } else {
                    let _e400 = (_e363 * _e390);
                    phi_2468_ = (vec2<f32>(dot(_e343, _e343), dot(_e400, _e400)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e343, _e400)));
                }
                let _e408 = phi_2468_;
                phi_2467_ = _e408;
            }
            let _e410 = phi_2467_;
            phi_2466_ = _e410;
        }
        let _e412 = phi_2466_;
        phi_2489_ = (_e95 != 0i);
        phi_2484_ = (_e136 * (_e412 * _e335));
        phi_2469_ = _e314;
    } else {
        phi_2489_ = (((_e200 & 2147483648u) != 0u) && (_e95 != 1i));
        phi_2484_ = vec2<f32>(0f, 0f);
        phi_2469_ = select(_e314, _e117, vec2((_e95 == 2i)));
    }
    let _e424 = phi_2489_;
    let _e426 = phi_2484_;
    let _e428 = phi_2469_;
    let _e431 = (((_e136 * _e428) + _e426) + bitcast<vec2<f32>>(_e144.xy));
    let _e432 = (_e121 + 2u);
    let _e439 = textureLoad(LB, vec2<i32>(bitcast<i32>((_e432 & 255u)), bitcast<i32>((_e432 >> bitcast<u32>(8i)))), 0i);
    let _e447 = textureLoad(WC, vec2<i32>(bitcast<i32>((_e115.z & 255u)), bitcast<i32>((_e119 >> bitcast<u32>(8i)))), 0i);
    let _e449 = (_e447.x & 15u);
    if mi {
        let _e450 = (_e449 == 0u);
        if _e450 {
            phi_2512_ = _e447.y;
        } else {
            phi_2512_ = _e447.x;
        }
        let _e453 = phi_2512_;
        let _e455 = (_e453 >> bitcast<u32>(16i));
        let _e457 = j.U4_;
        if (_e455 == 0u) {
            phi_2513_ = 0f;
        } else {
            phi_2513_ = unpack2x16float(((_e455 + 1023u) * _e457)).x;
        }
        let _e464 = phi_2513_;
        phi_2514_ = _e464;
        if _e450 {
            phi_2514_ = -(_e464);
        }
        let _e467 = phi_2514_;
        l1_[0u] = _e467;
    }
    if oi {
        Q0_ = f32(((_e447.x >> bitcast<u32>(4i)) & 15u));
    }
    if ni {
        let _e473 = (_e119 * 8u);
        let _e474 = (_e473 + 2u);
        let _e481 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e474 & 255u)), bitcast<i32>((_e474 >> bitcast<u32>(8i)))), 0i);
        let _e489 = (_e473 + 3u);
        let _e496 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e489 & 255u)), bitcast<i32>((_e489 >> bitcast<u32>(8i)))), 0i);
        if any((_e481 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e501 = ((mat2x2<f32>(vec2<f32>(_e481.x, _e481.y), vec2<f32>(_e481.z, _e481.w)) * _e431) + _e496.xy);
            unnamed.gl_ClipDistance[0i] = (_e501.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e501.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e501.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e501.y);
        } else {
            let _e517 = (_e496.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e517;
            unnamed.gl_ClipDistance[2i] = _e517;
            unnamed.gl_ClipDistance[1i] = _e517;
            unnamed.gl_ClipDistance[0i] = _e517;
        }
    }
    if (_e449 == 1u) {
        a1_ = unpack4x8unorm(_e447.y);
    } else {
        if (mi && (_e449 == 0u)) {
            let _e532 = (_e447.x >> bitcast<u32>(16i));
            let _e534 = j.U4_;
            if (_e532 == 0u) {
                phi_2553_ = 0f;
            } else {
                phi_2553_ = unpack2x16float(((_e532 + 1023u) * _e534)).x;
            }
            let _e541 = phi_2553_;
            l1_[1u] = _e541;
        } else {
            let _e543 = (_e119 * 8u);
            let _e550 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e543 & 255u)), bitcast<i32>((_e543 >> bitcast<u32>(8i)))), 0i);
            let _e558 = (_e543 + 1u);
            let _e565 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e558 & 255u)), bitcast<i32>((_e558 >> bitcast<u32>(8i)))), 0i);
            let _e574 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e447.y));
            let _e576 = ((mat2x2<f32>(vec2<f32>(_e550.x, _e550.y), vec2<f32>(_e550.z, _e550.w)) * _e431) + _e565.xy);
            if (_e565.z > 0.9f) {
                phi_2551_ = vec4<f32>(_e574.x, _e574.y, 2f, _e574.w);
            } else {
                phi_2551_ = vec4<f32>(_e574.x, _e574.y, _e565.w, _e574.w);
            }
            let _e591 = phi_2551_;
            if (f32(_e449) == 2f) {
                let _e598 = vec4<f32>(_e576.x, _e591.y, _e591.z, _e591.w);
                phi_2552_ = vec4<f32>(_e598.x, 0f, _e598.z, _e598.w);
            } else {
                let _e610 = vec4<f32>(_e591.x, _e591.y, -(_e591.z), _e591.w);
                let _e616 = vec4<f32>(_e576.x, _e610.y, _e610.z, _e610.w);
                phi_2552_ = vec4<f32>(_e616.x, _e576.y, _e616.z, _e616.w);
            }
            let _e624 = phi_2552_;
            a1_ = _e624;
            let _e626 = a1_[3u];
            a1_[3u] = -(_e626);
        }
    }
    phi_1198_ = ui;
    if ui {
        phi_1198_ = ((_e447.x & 2048u) != 0u);
    }
    let _e631 = phi_1198_;
    if _e631 {
        let _e632 = (_e119 * 8u);
        let _e633 = (_e632 + 4u);
        let _e640 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e633 & 255u)), bitcast<i32>((_e633 >> bitcast<u32>(8i)))), 0i);
        let _e648 = (_e632 + 5u);
        let _e655 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e648 & 255u)), bitcast<i32>((_e648 >> bitcast<u32>(8i)))), 0i);
        let _e658 = ((mat2x2<f32>(vec2<f32>(_e640.x, _e640.y), vec2<f32>(_e640.z, _e640.w)) * _e431) + _e655.xy);
        phi_2554_ = (1f + _e655.z);
        if ((_e447.x & 4096u) != 0u) {
            phi_2554_ = (-1f - f32(((_e447.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e669 = phi_2554_;
        v1_ = vec3<f32>(_e658.x, _e658.y, _e669);
    } else {
        v1_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e424) {
        let _e675 = j.dg;
        let _e677 = j.eg;
        let _e685 = vec4<f32>(((_e431.x * _e675) - 1f), ((_e431.y * _e677) - sign(_e677)), 0f, 1f);
        phi_2570_ = vec4<f32>(_e685.x, _e685.y, ((f32(((_e439.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e685.w);
    } else {
        let _e698 = j.a3_;
        phi_2570_ = vec4(_e698);
    }
    let _e701 = phi_2570_;
    unnamed.gl_Position = _e701;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) WB: vec4<f32>, @location(1) XB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    WB_1 = WB;
    XB_1 = XB;
    main_1();
    let _e17 = unnamed.gl_Position;
    let _e18 = unnamed.gl_ClipDistance;
    let _e19 = l1_;
    let _e20 = Q0_;
    let _e21 = a1_;
    let _e22 = v1_;
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
