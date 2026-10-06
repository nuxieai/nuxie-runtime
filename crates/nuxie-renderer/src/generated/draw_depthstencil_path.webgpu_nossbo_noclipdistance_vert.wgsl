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

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct VertexOutput {
    @location(4) @interpolate(flat, either) member: vec2<f32>,
    @location(6) @interpolate(flat, either) member_1: f32,
    @location(0) member_2: vec4<f32>,
    @location(9) member_3: vec3<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override mi: bool = true;
@id(2) override oi: bool = true;
@id(8) override ui: bool = true;

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
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var xa: sampler;

fn main_1() {
    var phi_2274_: f32;
    var phi_2246_: i32;
    var phi_1487_: bool;
    var phi_2259_: i32;
    var phi_2251_: vec4<u32>;
    var phi_2258_: i32;
    var phi_2250_: vec4<u32>;
    var phi_2257_: i32;
    var phi_2255_: vec4<u32>;
    var phi_2254_: u32;
    var phi_2261_: vec2<i32>;
    var phi_2262_: vec4<u32>;
    var phi_2266_: f32;
    var phi_2337_: f32;
    var phi_2280_: f32;
    var phi_2336_: f32;
    var phi_2284_: f32;
    var phi_2281_: f32;
    var phi_2278_: f32;
    var phi_2288_: f32;
    var phi_2334_: f32;
    var phi_2287_: f32;
    var phi_2343_: f32;
    var phi_2340_: f32;
    var phi_2372_: f32;
    var phi_2358_: f32;
    var phi_1775_: bool;
    var phi_2363_: f32;
    var phi_2380_: vec2<f32>;
    var phi_2379_: vec2<f32>;
    var phi_2378_: vec2<f32>;
    var phi_2401_: bool;
    var phi_2396_: vec2<f32>;
    var phi_2381_: vec2<f32>;
    var phi_2424_: u32;
    var phi_2425_: f32;
    var phi_2426_: f32;
    var phi_2463_: f32;
    var phi_2461_: vec4<f32>;
    var phi_2462_: vec4<f32>;
    var phi_1151_: bool;
    var phi_2464_: f32;
    var phi_2478_: vec4<f32>;

    let _e83 = gl_InstanceIndex_1;
    let _e84 = WB_1;
    let _e85 = XB_1;
    let _e87 = i32(_e84.x);
    let _e90 = bitcast<i32>(_e84.w);
    let _e92 = (_e90 >> bitcast<u32>(2i));
    let _e93 = (_e90 & 3i);
    let _e95 = min(_e87, (_e92 - 1i));
    let _e97 = ((_e83 * _e92) + _e95);
    let _e102 = textureLoad(TB, vec2<i32>((_e97 & 2047i), (_e97 >> bitcast<u32>(11i))), 0i);
    let _e106 = (max((_e102.w & 65535u), 1u) - 1u);
    let _e113 = textureLoad(ZC, vec2<i32>(bitcast<i32>((_e106 & 255u)), bitcast<i32>((_e106 >> bitcast<u32>(8i)))), 0i);
    let _e115 = bitcast<vec2<f32>>(_e113.xy);
    let _e117 = (_e113.z & 65535u);
    let _e119 = (_e117 * 4u);
    let _e126 = textureLoad(LB, vec2<i32>(bitcast<i32>((_e119 & 255u)), bitcast<i32>((_e119 >> bitcast<u32>(8i)))), 0i);
    let _e127 = bitcast<vec4<f32>>(_e126);
    let _e134 = mat2x2<f32>(vec2<f32>(_e127.x, _e127.y), vec2<f32>(_e127.z, _e127.w));
    let _e135 = (_e119 + 1u);
    let _e142 = textureLoad(LB, vec2<i32>(bitcast<i32>((_e135 & 255u)), bitcast<i32>((_e135 >> bitcast<u32>(8i)))), 0i);
    let _e146 = bitcast<f32>(_e142.z);
    let _e148 = bitcast<f32>(_e142.w);
    let _e149 = (_e102.w & 8388608u);
    phi_2274_ = _e84.y;
    phi_2246_ = _e87;
    if (_e149 != 0u) {
        phi_2274_ = _e85.y;
        phi_2246_ = i32(_e85.x);
    }
    let _e155 = phi_2274_;
    let _e157 = phi_2246_;
    phi_2257_ = _e97;
    phi_2255_ = _e102;
    phi_2254_ = _e102.w;
    if (_e157 != _e95) {
        let _e160 = ((_e97 + _e157) - _e95);
        let _e165 = textureLoad(TB, vec2<i32>((_e160 & 2047i), (_e160 >> bitcast<u32>(11i))), 0i);
        if ((_e165.w & 8454143u) != (_e102.w & 8454143u)) {
            let _e170 = (_e146 == 0f);
            phi_1487_ = _e170;
            if !(_e170) {
                phi_1487_ = (_e115.x != 0f);
            }
            let _e175 = phi_1487_;
            phi_2259_ = _e97;
            phi_2251_ = _e102;
            if _e175 {
                let _e176 = bitcast<i32>(_e113.w);
                let _e181 = textureLoad(TB, vec2<i32>((_e176 & 2047i), (_e176 >> bitcast<u32>(11i))), 0i);
                phi_2259_ = _e176;
                phi_2251_ = _e181;
            }
            let _e183 = phi_2259_;
            let _e185 = phi_2251_;
            phi_2258_ = _e183;
            phi_2250_ = _e185;
        } else {
            phi_2258_ = _e160;
            phi_2250_ = _e165;
        }
        let _e187 = phi_2258_;
        let _e189 = phi_2250_;
        phi_2257_ = _e187;
        phi_2255_ = _e189;
        phi_2254_ = ((_e189.w & 4286578687u) | _e149);
    }
    let _e194 = phi_2257_;
    let _e196 = phi_2255_;
    let _e198 = phi_2254_;
    let _e199 = (_e198 & 469762048u);
    if ((_e199 == 67108864u) && (_e93 == 0i)) {
        let _e205 = f32((_e196.z & 65535u));
        let _e208 = f32((_e196.z >> bitcast<u32>(16i)));
        let _e214 = vec2<i32>(i32((-1f - _e205)), i32(((_e208 - _e205) + 1f)));
        phi_2261_ = _e214;
        if ((_e198 & 8388608u) != 0u) {
            phi_2261_ = -(_e214);
        }
        let _e219 = phi_2261_;
        let _e221 = (_e194 + _e219.x);
        let _e226 = textureLoad(TB, vec2<i32>((_e221 & 2047i), (_e221 >> bitcast<u32>(11i))), 0i);
        let _e228 = (_e194 + _e219.y);
        let _e233 = textureLoad(TB, vec2<i32>((_e228 & 2047i), (_e228 >> bitcast<u32>(11i))), 0i);
        phi_2262_ = _e233;
        if ((_e233.w & 8454143u) != (_e226.w & 8454143u)) {
            let _e239 = bitcast<i32>(_e113.w);
            let _e244 = textureLoad(TB, vec2<i32>((_e239 & 2047i), (_e239 >> bitcast<u32>(11i))), 0i);
            phi_2262_ = _e244;
        }
        let _e246 = phi_2262_;
        let _e249 = (f32(_e226.z) * 0.0000000014629181f);
        let _e252 = (f32(_e246.z) * 0.0000000014629181f);
        let _e253 = (_e252 - _e249);
        phi_2266_ = _e253;
        if (abs(_e253) > 3.1415927f) {
            phi_2266_ = (_e253 - (6.2831855f * sign(_e253)));
        }
        let _e260 = phi_2266_;
        let _e261 = (_e208 + -2f);
        let _e267 = clamp(round(((abs(_e260) * 0.31830987f) * _e261)), 1f, (_e208 + -3f));
        let _e268 = (_e261 - _e267);
        if (_e205 <= _e268) {
            phi_2337_ = _e155;
            if (_e205 == _e268) {
                phi_2337_ = -(_e155);
            }
            let _e277 = phi_2337_;
            phi_2336_ = _e277;
            phi_2284_ = -(((3.1415927f * sign(_e260)) - _e260));
            phi_2281_ = _e268;
            phi_2278_ = _e205;
        } else {
            let _e279 = (_e205 == (_e268 + 1f));
            if _e279 {
                phi_2280_ = 0f;
            } else {
                phi_2280_ = (_e205 - (_e268 + 2f));
            }
            let _e283 = phi_2280_;
            phi_2336_ = select(_e155, 0f, _e279);
            phi_2284_ = _e260;
            phi_2281_ = select(_e267, 0f, _e279);
            phi_2278_ = _e283;
        }
        let _e287 = phi_2336_;
        let _e289 = phi_2284_;
        let _e291 = phi_2281_;
        let _e293 = phi_2278_;
        if (_e293 == _e291) {
            phi_2288_ = _e252;
        } else {
            phi_2288_ = (_e249 + (_e289 * (_e293 / _e291)));
        }
        let _e299 = phi_2288_;
        phi_2334_ = _e287;
        phi_2287_ = _e299;
    } else {
        phi_2334_ = _e155;
        phi_2287_ = (f32(_e196.z) * 0.0000000014629181f);
    }
    let _e304 = phi_2334_;
    let _e306 = phi_2287_;
    let _e310 = vec2<f32>(sin(_e306), -(cos(_e306)));
    let _e312 = bitcast<vec2<f32>>(_e196.xy);
    phi_2343_ = _e148;
    if (_e148 != 0f) {
        phi_2343_ = max(_e148, (1f / length((_e134 * _e310))));
    }
    let _e319 = phi_2343_;
    if (_e146 != 0f) {
        let _e323 = (_e304 * sign(determinant(_e134)));
        let _e325 = ((_e198 & 1048576u) != 0u);
        phi_2340_ = _e323;
        if _e325 {
            phi_2340_ = min(_e323, 0f);
        }
        let _e328 = phi_2340_;
        phi_2372_ = _e328;
        if ((_e198 & 524288u) != 0u) {
            phi_2372_ = max(_e328, 0f);
        }
        let _e333 = phi_2372_;
        let _e335 = select(0f, _e319, (_e319 != 0f));
        let _e339 = select(_e146, _e335, ((_e335 > _e146) && (_e319 == 0f)));
        let _e340 = (_e339 + _e335);
        let _e341 = (_e310 * _e340);
        phi_2378_ = _e341;
        if (_e199 > 134217728u) {
            let _e347 = f32((_e196.z & 65535u));
            let _e348 = (_e347 * 0.000015259022f);
            let _e352 = sqrt(max((1f - (_e348 * _e348)), 0f));
            phi_2358_ = _e352;
            if (((_e198 & 4194304u) != 0u) == _e325) {
                phi_2358_ = -(_e352);
            }
            let _e356 = phi_2358_;
            let _e361 = (mat2x2<f32>(vec2<f32>(_e348, _e356), vec2<f32>(-(_e356), _e348)) * _e310);
            let _e362 = (_e134 * _e361);
            let _e371 = (_e199 == 335544320u);
            phi_1775_ = _e371;
            if !(_e371) {
                phi_1775_ = ((_e199 == 268435456u) && (_e348 >= 0.25f));
            }
            let _e377 = phi_1775_;
            if _e377 {
                phi_2363_ = (_e339 * (1f / max(_e348, select(0.25f, 1f, ((_e198 & 33554432u) != 0u)))));
            } else {
                phi_2363_ = ((_e339 * _e348) + (((abs(_e362.x) + abs(_e362.y)) * (1f / dot(_e362, _e362))) * 0.5f));
            }
            let _e388 = phi_2363_;
            phi_2379_ = _e341;
            if ((_e198 & 2097152u) != 0u) {
                if (_e340 <= ((_e388 * _e348) + (_e335 * 0.125f))) {
                    phi_2380_ = (_e361 * (_e340 * (65535f / _e347)));
                } else {
                    let _e398 = (_e361 * _e388);
                    phi_2380_ = (vec2<f32>(dot(_e341, _e341), dot(_e398, _e398)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e341, _e398)));
                }
                let _e406 = phi_2380_;
                phi_2379_ = _e406;
            }
            let _e408 = phi_2379_;
            phi_2378_ = _e408;
        }
        let _e410 = phi_2378_;
        phi_2401_ = (_e93 != 0i);
        phi_2396_ = (_e134 * (_e410 * _e333));
        phi_2381_ = _e312;
    } else {
        phi_2401_ = (((_e198 & 2147483648u) != 0u) && (_e93 != 1i));
        phi_2396_ = vec2<f32>(0f, 0f);
        phi_2381_ = select(_e312, _e115, vec2((_e93 == 2i)));
    }
    let _e422 = phi_2401_;
    let _e424 = phi_2396_;
    let _e426 = phi_2381_;
    let _e429 = (((_e134 * _e426) + _e424) + bitcast<vec2<f32>>(_e142.xy));
    let _e430 = (_e119 + 2u);
    let _e437 = textureLoad(LB, vec2<i32>(bitcast<i32>((_e430 & 255u)), bitcast<i32>((_e430 >> bitcast<u32>(8i)))), 0i);
    let _e445 = textureLoad(WC, vec2<i32>(bitcast<i32>((_e113.z & 255u)), bitcast<i32>((_e117 >> bitcast<u32>(8i)))), 0i);
    let _e447 = (_e445.x & 15u);
    if mi {
        let _e448 = (_e447 == 0u);
        if _e448 {
            phi_2424_ = _e445.y;
        } else {
            phi_2424_ = _e445.x;
        }
        let _e451 = phi_2424_;
        let _e453 = (_e451 >> bitcast<u32>(16i));
        let _e455 = j.U4_;
        if (_e453 == 0u) {
            phi_2425_ = 0f;
        } else {
            phi_2425_ = unpack2x16float(((_e453 + 1023u) * _e455)).x;
        }
        let _e462 = phi_2425_;
        phi_2426_ = _e462;
        if _e448 {
            phi_2426_ = -(_e462);
        }
        let _e465 = phi_2426_;
        l1_[0u] = _e465;
    }
    if oi {
        Q0_ = f32(((_e445.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e447 == 1u) {
        a1_ = unpack4x8unorm(_e445.y);
    } else {
        if (mi && (_e447 == 0u)) {
            let _e477 = (_e445.x >> bitcast<u32>(16i));
            let _e479 = j.U4_;
            if (_e477 == 0u) {
                phi_2463_ = 0f;
            } else {
                phi_2463_ = unpack2x16float(((_e477 + 1023u) * _e479)).x;
            }
            let _e486 = phi_2463_;
            l1_[1u] = _e486;
        } else {
            let _e488 = (_e117 * 8u);
            let _e495 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e488 & 255u)), bitcast<i32>((_e488 >> bitcast<u32>(8i)))), 0i);
            let _e503 = (_e488 + 1u);
            let _e510 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e503 & 255u)), bitcast<i32>((_e503 >> bitcast<u32>(8i)))), 0i);
            let _e519 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e445.y));
            let _e521 = ((mat2x2<f32>(vec2<f32>(_e495.x, _e495.y), vec2<f32>(_e495.z, _e495.w)) * _e429) + _e510.xy);
            if (_e510.z > 0.9f) {
                phi_2461_ = vec4<f32>(_e519.x, _e519.y, 2f, _e519.w);
            } else {
                phi_2461_ = vec4<f32>(_e519.x, _e519.y, _e510.w, _e519.w);
            }
            let _e536 = phi_2461_;
            if (f32(_e447) == 2f) {
                let _e543 = vec4<f32>(_e521.x, _e536.y, _e536.z, _e536.w);
                phi_2462_ = vec4<f32>(_e543.x, 0f, _e543.z, _e543.w);
            } else {
                let _e555 = vec4<f32>(_e536.x, _e536.y, -(_e536.z), _e536.w);
                let _e561 = vec4<f32>(_e521.x, _e555.y, _e555.z, _e555.w);
                phi_2462_ = vec4<f32>(_e561.x, _e521.y, _e561.z, _e561.w);
            }
            let _e569 = phi_2462_;
            a1_ = _e569;
            let _e571 = a1_[3u];
            a1_[3u] = -(_e571);
        }
    }
    phi_1151_ = ui;
    if ui {
        phi_1151_ = ((_e445.x & 2048u) != 0u);
    }
    let _e576 = phi_1151_;
    if _e576 {
        let _e577 = (_e117 * 8u);
        let _e578 = (_e577 + 4u);
        let _e585 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e578 & 255u)), bitcast<i32>((_e578 >> bitcast<u32>(8i)))), 0i);
        let _e593 = (_e577 + 5u);
        let _e600 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e593 & 255u)), bitcast<i32>((_e593 >> bitcast<u32>(8i)))), 0i);
        let _e603 = ((mat2x2<f32>(vec2<f32>(_e585.x, _e585.y), vec2<f32>(_e585.z, _e585.w)) * _e429) + _e600.xy);
        phi_2464_ = (1f + _e600.z);
        if ((_e445.x & 4096u) != 0u) {
            phi_2464_ = (-1f - f32(((_e445.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e614 = phi_2464_;
        v1_ = vec3<f32>(_e603.x, _e603.y, _e614);
    } else {
        v1_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e422) {
        let _e620 = j.dg;
        let _e622 = j.eg;
        let _e630 = vec4<f32>(((_e429.x * _e620) - 1f), ((_e429.y * _e622) - sign(_e622)), 0f, 1f);
        phi_2478_ = vec4<f32>(_e630.x, _e630.y, ((f32(((_e437.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e630.w);
    } else {
        let _e643 = j.a3_;
        phi_2478_ = vec4(_e643);
    }
    let _e646 = phi_2478_;
    unnamed.gl_Position = _e646;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) WB: vec4<f32>, @location(1) XB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    WB_1 = WB;
    XB_1 = XB;
    main_1();
    let _e16 = l1_;
    let _e17 = Q0_;
    let _e18 = a1_;
    let _e19 = v1_;
    let _e20 = unnamed.gl_Position;
    return VertexOutput(_e16, _e17, _e18, _e19, _e20);
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
