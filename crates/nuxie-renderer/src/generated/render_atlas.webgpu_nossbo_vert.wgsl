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

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(6)
var VB: texture_2d<u32>;
@group(0) @binding(4)
var CD: texture_2d<u32>;
@group(0) @binding(2)
var JB: texture_2d<u32>;
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
var KB: texture_2d<f32>;
@group(3) @binding(8)
var fb: sampler;

fn main_1() {
    var phi_2406_: f32;
    var phi_2356_: f32;
    var phi_2293_: f32;
    var phi_2383_: f32;
    var phi_2319_: f32;
    var phi_2287_: i32;
    var phi_1427_: bool;
    var phi_2304_: i32;
    var phi_2295_: vec4<u32>;
    var phi_2303_: i32;
    var phi_2294_: vec4<u32>;
    var phi_2302_: i32;
    var phi_2300_: vec4<u32>;
    var phi_2299_: u32;
    var phi_2306_: vec2<i32>;
    var phi_2307_: vec4<u32>;
    var phi_2311_: f32;
    var phi_2394_: f32;
    var phi_2326_: f32;
    var phi_2393_: f32;
    var phi_2334_: f32;
    var phi_2327_: f32;
    var phi_2324_: f32;
    var phi_2338_: f32;
    var phi_2431_: f32;
    var phi_2421_: f32;
    var phi_2391_: f32;
    var phi_2337_: f32;
    var phi_2389_: f32;
    var phi_2492_: f32;
    var phi_2503_: f32;
    var phi_2495_: f32;
    var phi_2557_: f32;
    var phi_2531_: f32;
    var phi_1742_: bool;
    var phi_2536_: f32;
    var phi_2548_: vec2<f32>;
    var phi_2547_: vec2<f32>;
    var phi_2565_: vec4<f32>;
    var phi_2578_: vec2<f32>;
    var phi_2564_: vec4<f32>;
    var phi_2614_: vec4<f32>;
    var phi_2442_: f32;
    var phi_2441_: f32;
    var phi_2443_: f32;
    var phi_2447_: f32;
    var phi_2470_: f32;
    var phi_2468_: f32;
    var phi_2487_: vec4<f32>;
    var phi_2611_: vec2<f32>;
    var phi_2610_: vec2<f32>;
    var phi_2486_: vec4<f32>;
    var phi_2617_: vec4<f32>;
    var phi_2618_: bool;
    var phi_2612_: vec4<f32>;
    var phi_2605_: vec2<f32>;
    var phi_2580_: vec2<f32>;
    var phi_2651_: vec4<f32>;

    let _e74 = gl_InstanceIndex_1;
    let _e75 = YB_1;
    let _e76 = ZB_1;
    let _e78 = i32(_e75.x);
    let _e82 = bitcast<i32>(_e75.w);
    let _e84 = (_e82 >> bitcast<u32>(2i));
    let _e85 = (_e82 & 3i);
    let _e87 = min(_e78, (_e84 - 1i));
    let _e89 = ((_e74 * _e84) + _e87);
    let _e94 = textureLoad(VB, vec2<i32>((_e89 & 2047i), (_e89 >> bitcast<u32>(11i))), 0i);
    let _e98 = (max((_e94.w & 65535u), 1u) - 1u);
    let _e105 = textureLoad(CD, vec2<i32>(bitcast<i32>((_e98 & 255u)), bitcast<i32>((_e98 >> bitcast<u32>(8i)))), 0i);
    let _e107 = bitcast<vec2<f32>>(_e105.xy);
    let _e111 = ((_e105.z & 65535u) * 4u);
    let _e118 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e111 & 255u)), bitcast<i32>((_e111 >> bitcast<u32>(8i)))), 0i);
    let _e119 = bitcast<vec4<f32>>(_e118);
    let _e126 = mat2x2<f32>(vec2<f32>(_e119.x, _e119.y), vec2<f32>(_e119.z, _e119.w));
    let _e127 = (_e111 + 1u);
    let _e134 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e127 & 255u)), bitcast<i32>((_e127 >> bitcast<u32>(8i)))), 0i);
    phi_2406_ = 0f;
    phi_2356_ = 0f;
    phi_2293_ = 0f;
    if ((_e134.z & 32768u) != 0u) {
        let _e140 = (_e111 + 2u);
        let _e147 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e140 & 255u)), bitcast<i32>((_e140 >> bitcast<u32>(8i)))), 0i);
        phi_2406_ = bitcast<f32>(_e147.z);
        phi_2356_ = bitcast<f32>(_e147.y);
        phi_2293_ = bitcast<f32>(_e147.x);
    }
    let _e155 = phi_2406_;
    let _e157 = phi_2356_;
    let _e159 = phi_2293_;
    let _e160 = (_e94.w & 8388608u);
    phi_2383_ = _e75.z;
    phi_2319_ = _e75.y;
    phi_2287_ = _e78;
    if (_e160 != 0u) {
        phi_2383_ = _e76.z;
        phi_2319_ = _e76.y;
        phi_2287_ = i32(_e76.x);
    }
    let _e167 = phi_2383_;
    let _e169 = phi_2319_;
    let _e171 = phi_2287_;
    phi_2302_ = _e89;
    phi_2300_ = _e94;
    phi_2299_ = _e94.w;
    if (_e171 != _e87) {
        let _e174 = ((_e89 + _e171) - _e87);
        let _e179 = textureLoad(VB, vec2<i32>((_e174 & 2047i), (_e174 >> bitcast<u32>(11i))), 0i);
        if ((_e179.w & 8454143u) != (_e94.w & 8454143u)) {
            let _e184 = (_e159 == 0f);
            phi_1427_ = _e184;
            if !(_e184) {
                phi_1427_ = (_e107.x != 0f);
            }
            let _e189 = phi_1427_;
            phi_2304_ = _e89;
            phi_2295_ = _e94;
            if _e189 {
                let _e190 = bitcast<i32>(_e105.w);
                let _e195 = textureLoad(VB, vec2<i32>((_e190 & 2047i), (_e190 >> bitcast<u32>(11i))), 0i);
                phi_2304_ = _e190;
                phi_2295_ = _e195;
            }
            let _e197 = phi_2304_;
            let _e199 = phi_2295_;
            phi_2303_ = _e197;
            phi_2294_ = _e199;
        } else {
            phi_2303_ = _e174;
            phi_2294_ = _e179;
        }
        let _e201 = phi_2303_;
        let _e203 = phi_2294_;
        phi_2302_ = _e201;
        phi_2300_ = _e203;
        phi_2299_ = ((_e203.w & 4286578687u) | _e160);
    }
    let _e208 = phi_2302_;
    let _e210 = phi_2300_;
    let _e212 = phi_2299_;
    let _e213 = (_e212 & 469762048u);
    let _e216 = ((_e213 == 67108864u) && (_e85 == 0i));
    if _e216 {
        let _e219 = f32((_e210.z & 65535u));
        let _e222 = f32((_e210.z >> bitcast<u32>(16i)));
        let _e228 = vec2<i32>(i32((-1f - _e219)), i32(((_e222 - _e219) + 1f)));
        phi_2306_ = _e228;
        if ((_e212 & 8388608u) != 0u) {
            phi_2306_ = -(_e228);
        }
        let _e233 = phi_2306_;
        let _e235 = (_e208 + _e233.x);
        let _e240 = textureLoad(VB, vec2<i32>((_e235 & 2047i), (_e235 >> bitcast<u32>(11i))), 0i);
        let _e242 = (_e208 + _e233.y);
        let _e247 = textureLoad(VB, vec2<i32>((_e242 & 2047i), (_e242 >> bitcast<u32>(11i))), 0i);
        phi_2307_ = _e247;
        if ((_e247.w & 8454143u) != (_e240.w & 8454143u)) {
            let _e253 = bitcast<i32>(_e105.w);
            let _e258 = textureLoad(VB, vec2<i32>((_e253 & 2047i), (_e253 >> bitcast<u32>(11i))), 0i);
            phi_2307_ = _e258;
        }
        let _e260 = phi_2307_;
        let _e263 = (f32(_e240.z) * 0.0000000014629181f);
        let _e266 = (f32(_e260.z) * 0.0000000014629181f);
        let _e267 = (_e266 - _e263);
        phi_2311_ = _e267;
        if (abs(_e267) > 3.1415927f) {
            phi_2311_ = (_e267 - (6.2831855f * sign(_e267)));
        }
        let _e274 = phi_2311_;
        let _e275 = (_e222 + -2f);
        let _e281 = clamp(round(((abs(_e274) * 0.31830987f) * _e275)), 1f, (_e222 + -3f));
        let _e282 = (_e275 - _e281);
        if (_e219 <= _e282) {
            phi_2394_ = _e169;
            if (_e219 == _e282) {
                phi_2394_ = -(_e169);
            }
            let _e291 = phi_2394_;
            phi_2393_ = _e291;
            phi_2334_ = -(((3.1415927f * sign(_e274)) - _e274));
            phi_2327_ = _e282;
            phi_2324_ = _e219;
        } else {
            let _e293 = (_e219 == (_e282 + 1f));
            if _e293 {
                phi_2326_ = 0f;
            } else {
                phi_2326_ = (_e219 - (_e282 + 2f));
            }
            let _e297 = phi_2326_;
            phi_2393_ = select(_e169, 0f, _e293);
            phi_2334_ = _e274;
            phi_2327_ = select(_e281, 0f, _e293);
            phi_2324_ = _e297;
        }
        let _e301 = phi_2393_;
        let _e303 = phi_2334_;
        let _e305 = phi_2327_;
        let _e307 = phi_2324_;
        if (_e307 == _e305) {
            phi_2338_ = _e266;
        } else {
            phi_2338_ = (_e263 + (_e303 * (_e307 / _e305)));
        }
        let _e313 = phi_2338_;
        phi_2431_ = _e263;
        phi_2421_ = _e303;
        phi_2391_ = _e301;
        phi_2337_ = _e313;
    } else {
        phi_2431_ = f32();
        phi_2421_ = f32();
        phi_2391_ = _e169;
        phi_2337_ = (f32(_e210.z) * 0.0000000014629181f);
    }
    let _e318 = phi_2431_;
    let _e320 = phi_2421_;
    let _e322 = phi_2391_;
    let _e324 = phi_2337_;
    let _e328 = vec2<f32>(sin(_e324), -(cos(_e324)));
    let _e330 = bitcast<vec2<f32>>(_e210.xy);
    phi_2389_ = _e157;
    if (_e157 != 0f) {
        phi_2389_ = max(_e157, (1f / length((_e126 * _e328))));
    }
    let _e337 = phi_2389_;
    if (_e159 != 0f) {
        let _e341 = (_e322 * sign(determinant(_e126)));
        let _e343 = ((_e212 & 1048576u) != 0u);
        phi_2492_ = _e341;
        if _e343 {
            phi_2492_ = min(_e341, 0f);
        }
        let _e346 = phi_2492_;
        phi_2503_ = _e346;
        if ((_e212 & 524288u) != 0u) {
            phi_2503_ = max(_e346, 0f);
        }
        let _e351 = phi_2503_;
        let _e352 = (_e337 != 0f);
        if _e352 {
            phi_2495_ = _e337;
        } else {
            let _e353 = (_e126 * _e328);
            phi_2495_ = (((abs(_e353.x) + abs(_e353.y)) * (1f / dot(_e353, _e353))) * 0.5f);
        }
        let _e364 = phi_2495_;
        let _e367 = ((_e364 > _e159) && (_e337 == 0f));
        phi_2557_ = 1f;
        if _e367 {
            phi_2557_ = (_e159 / _e364);
        }
        let _e370 = phi_2557_;
        let _e371 = select(_e159, _e364, _e367);
        let _e372 = (_e371 + _e364);
        let _e373 = (_e328 * _e372);
        let _e374 = (_e351 * _e372);
        let _e381 = (((vec2<f32>(_e374, -(_e374)) + vec2(_e371)) * (0.5f / _e364)) + vec2<f32>(0.5f, 0.5f));
        let _e384 = vec4<f32>(_e381.x, _e381.y, 0f, 0f);
        phi_2578_ = _e373;
        phi_2564_ = _e384;
        if (_e213 > 201326592u) {
            let _e390 = f32((_e210.z & 65535u));
            let _e391 = (_e390 * 0.000015259022f);
            let _e395 = sqrt(max((1f - (_e391 * _e391)), 0f));
            phi_2531_ = _e395;
            if (((_e212 & 4194304u) != 0u) == _e343) {
                phi_2531_ = -(_e395);
            }
            let _e399 = phi_2531_;
            let _e404 = (mat2x2<f32>(vec2<f32>(_e391, _e399), vec2<f32>(-(_e399), _e391)) * _e328);
            let _e405 = (_e126 * _e404);
            let _e413 = ((abs(_e405.x) + abs(_e405.y)) * (1f / dot(_e405, _e405)));
            let _e414 = (_e213 == 402653184u);
            phi_1742_ = _e414;
            if !(_e414) {
                phi_1742_ = ((_e213 == 335544320u) && (_e391 >= 0.25f));
            }
            let _e420 = phi_1742_;
            if _e420 {
                phi_2536_ = (_e371 * (1f / max(_e391, select(0.25f, 1f, ((_e212 & 33554432u) != 0u)))));
            } else {
                phi_2536_ = ((_e371 * _e391) + (_e413 * 0.5f));
            }
            let _e431 = phi_2536_;
            let _e433 = (_e431 + (_e413 * 0.5f));
            phi_2547_ = _e373;
            if ((_e212 & 2097152u) != 0u) {
                if (_e372 <= ((_e433 * _e391) + (_e364 * 0.125f))) {
                    phi_2548_ = (_e404 * (_e372 * (65535f / _e390)));
                } else {
                    let _e443 = (_e404 * _e433);
                    phi_2548_ = (vec2<f32>(dot(_e373, _e373), dot(_e443, _e443)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e373, _e443)));
                }
                let _e451 = phi_2548_;
                phi_2547_ = _e451;
            }
            let _e453 = phi_2547_;
            let _e458 = ((_e433 - dot((_e453 * abs(_e351)), _e404)) / _e413);
            if _e343 {
                phi_2565_ = vec4<f32>(_e384.x, _e458, _e384.z, _e384.w);
            } else {
                phi_2565_ = vec4<f32>(_e458, _e384.y, _e384.z, _e384.w);
            }
            let _e470 = phi_2565_;
            phi_2578_ = _e453;
            phi_2564_ = _e470;
        }
        let _e472 = phi_2578_;
        let _e474 = phi_2564_;
        let _e476 = (_e474.xy * _e370);
        let _e482 = vec4<f32>(_e476.x, _e474.y, _e474.z, _e474.w);
        let _e489 = vec4<f32>(_e482.x, max(_e476.y, 0.0001f), _e482.z, _e482.w);
        phi_2614_ = _e489;
        if _e352 {
            phi_2614_ = vec4<f32>((-2f - _e476.x), _e489.y, _e489.z, _e489.w);
        }
        let _e497 = phi_2614_;
        phi_2618_ = (_e85 != 0i);
        phi_2612_ = _e497;
        phi_2605_ = (_e126 * (_e472 * _e351));
        phi_2580_ = _e330;
    } else {
        let _e501 = vec4<f32>(_e167, -1f, 0f, 0f);
        if (_e337 != 0f) {
            let _e507 = vec4<f32>(_e501.x, -2f, _e501.z, _e501.w);
            let _e512 = vec4<f32>(_e507.x, _e507.y, 1000000f, _e507.w);
            phi_2487_ = vec4<f32>(_e512.x, _e512.y, _e512.z, _e167);
            if _e216 {
                phi_2442_ = _e320;
                phi_2441_ = _e318;
                if (_e320 < 0f) {
                    phi_2442_ = -(_e320);
                    phi_2441_ = (_e318 + _e320);
                }
                let _e522 = phi_2442_;
                let _e524 = phi_2441_;
                let _e526 = ((_e324 - _e524) + 1.5707964f);
                let _e532 = clamp(((_e526 - (floor((_e526 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e522);
                phi_2443_ = _e532;
                if (_e532 > (_e522 * 0.5f)) {
                    phi_2443_ = (_e522 - _e532);
                }
                let _e537 = phi_2443_;
                let _e544 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e537), cos(_e537)) * abs(_e322))) * 0.5f);
                if (abs((_e522 - 1.5707964f)) < 0.001f) {
                    phi_2470_ = 0f;
                    phi_2468_ = 0f;
                } else {
                    let _e548 = tan(_e522);
                    let _e553 = (sign((1.5707964f - _e522)) / max(abs(_e548), 0.000001f));
                    if (_e553 >= 0f) {
                        phi_2447_ = (_e544.y - ((1f - _e544.x) * _e548));
                    } else {
                        phi_2447_ = (_e544.y + (_e544.x * _e548));
                    }
                    let _e565 = phi_2447_;
                    phi_2470_ = _e565;
                    phi_2468_ = _e553;
                }
                let _e567 = phi_2470_;
                let _e569 = phi_2468_;
                phi_2487_ = vec4<f32>((max(_e544.x, 0f) + 0.25f), (-2f - _e544.y), _e569, _e567);
            }
            let _e577 = phi_2487_;
            phi_2610_ = (_e126 * (_e328 * (_e322 * _e337)));
            phi_2486_ = _e577;
        } else {
            let _e585 = (sign(((_e328 * _e322) * _naga_inverse_2x2_f32(_e126))) * 0.5f);
            phi_2611_ = _e585;
            if (_e85 != 2i) {
                phi_2611_ = (_e585 + (_e328 * _e155));
            }
            let _e590 = phi_2611_;
            phi_2610_ = _e590;
            phi_2486_ = _e501;
        }
        let _e592 = phi_2610_;
        let _e594 = phi_2486_;
        phi_2617_ = _e594;
        if (((_e212 & 8388608u) != 0u) != ((_e212 & 16777216u) != 0u)) {
            phi_2617_ = (_e594 * vec4<f32>(-1f, 1f, 1f, 1f));
        }
        let _e602 = phi_2617_;
        phi_2618_ = (((_e212 & 2147483648u) != 0u) && (_e85 != 1i));
        phi_2612_ = _e602;
        phi_2605_ = _e592;
        phi_2580_ = select(_e330, _e107, vec2((_e85 == 2i)));
    }
    let _e611 = phi_2618_;
    let _e613 = phi_2612_;
    let _e615 = phi_2605_;
    let _e617 = phi_2580_;
    let _e623 = j.Hi;
    let _e626 = select(_e613.xy, vec2<f32>(1f, -1f), vec2((_e623 != 0u)));
    let _e632 = vec4<f32>(_e626.x, _e613.y, _e613.z, _e613.w);
    S = vec4<f32>(_e632.x, _e626.y, _e632.z, _e632.w);
    if !(_e611) {
        let _e640 = (_e111 + 3u);
        let _e647 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e640 & 255u)), bitcast<i32>((_e640 >> bitcast<u32>(8i)))), 0i);
        let _e649 = bitcast<vec3<f32>>(_e647.xyz);
        let _e653 = (((((_e126 * _e617) + _e615) + bitcast<vec2<f32>>(_e134.xy)) * _e649.x) + _e649.yz);
        let _e656 = j.Re[0u];
        let _e659 = j.Re[1u];
        phi_2651_ = vec4<f32>(((_e653.x * _e656) - 1f), ((_e653.y * _e659) - sign(_e659)), 0f, 1f);
    } else {
        let _e669 = j.j3_;
        phi_2651_ = vec4(_e669);
    }
    let _e672 = phi_2651_;
    unnamed.gl_Position = _e672;
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
