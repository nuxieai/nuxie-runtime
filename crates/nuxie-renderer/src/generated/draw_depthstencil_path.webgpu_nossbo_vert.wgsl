enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct UB {
    Qc: f32,
    Td: f32,
    Yf: f32,
    Zf: f32,
    z6_: u32,
    X9_: u32,
    Kf: u32,
    Lf: u32,
    i8_: vec4<i32>,
    Ch: vec2<f32>,
    Ud: vec2<f32>,
    i2_: u32,
    Gh: f32,
    T4_: u32,
    c3_: f32,
    Vd: f32,
    Ef: u32,
    M3_: f32,
    N3_: f32,
    Wd: f32,
    zh: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(4) @interpolate(flat, either) member: vec2<f32>,
    @location(6) @interpolate(flat, either) member_1: f32,
    @location(0) member_2: vec4<f32>,
    @location(9) member_3: vec3<f32>,
}

@id(0) override bi: bool = true;
@id(2) override di: bool = true;
@id(1) override ci: bool = true;
@id(8) override ji: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(5)
var AD: texture_2d<u32>;
@group(0) @binding(2)
var LB: texture_2d<u32>;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> WB_1: vec4<f32>;
var<private> XB_1: vec4<f32>;
@group(0) @binding(3)
var XC: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> l1_: vec2<f32>;
var<private> Q0_: f32;
@group(0) @binding(4)
var JB: texture_2d<f32>;
var<private> a1_: vec4<f32>;
var<private> F1_: vec3<f32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var wa: sampler;

fn main_1() {
    var phi_2340_: f32;
    var phi_2312_: i32;
    var phi_1510_: bool;
    var phi_2325_: i32;
    var phi_2317_: vec4<u32>;
    var phi_2324_: i32;
    var phi_2316_: vec4<u32>;
    var phi_2323_: i32;
    var phi_2321_: vec4<u32>;
    var phi_2320_: u32;
    var phi_2327_: vec2<i32>;
    var phi_2328_: vec4<u32>;
    var phi_2332_: f32;
    var phi_2403_: f32;
    var phi_2346_: f32;
    var phi_2402_: f32;
    var phi_2350_: f32;
    var phi_2347_: f32;
    var phi_2344_: f32;
    var phi_2354_: f32;
    var phi_2400_: f32;
    var phi_2353_: f32;
    var phi_2409_: f32;
    var phi_2406_: f32;
    var phi_2438_: f32;
    var phi_2424_: f32;
    var phi_1798_: bool;
    var phi_2429_: f32;
    var phi_2446_: vec2<f32>;
    var phi_2445_: vec2<f32>;
    var phi_2444_: vec2<f32>;
    var phi_2467_: bool;
    var phi_2462_: vec2<f32>;
    var phi_2447_: vec2<f32>;
    var phi_2490_: u32;
    var phi_2491_: f32;
    var phi_2492_: f32;
    var phi_2531_: f32;
    var phi_2529_: vec4<f32>;
    var phi_2530_: vec4<f32>;
    var phi_1198_: bool;
    var phi_2546_: vec4<f32>;

    let _e82 = gl_InstanceIndex_1;
    let _e83 = WB_1;
    let _e84 = XB_1;
    let _e86 = i32(_e83.x);
    let _e89 = bitcast<i32>(_e83.w);
    let _e91 = (_e89 >> bitcast<u32>(2i));
    let _e92 = (_e89 & 3i);
    let _e94 = min(_e86, (_e91 - 1i));
    let _e96 = ((_e82 * _e91) + _e94);
    let _e101 = textureLoad(TB, vec2<i32>((_e96 & 2047i), (_e96 >> bitcast<u32>(11i))), 0i);
    let _e105 = (max((_e101.w & 65535u), 1u) - 1u);
    let _e112 = textureLoad(AD, vec2<i32>(bitcast<i32>((_e105 & 255u)), bitcast<i32>((_e105 >> bitcast<u32>(8i)))), 0i);
    let _e114 = bitcast<vec2<f32>>(_e112.xy);
    let _e116 = (_e112.z & 65535u);
    let _e118 = (_e116 * 4u);
    let _e125 = textureLoad(LB, vec2<i32>(bitcast<i32>((_e118 & 255u)), bitcast<i32>((_e118 >> bitcast<u32>(8i)))), 0i);
    let _e126 = bitcast<vec4<f32>>(_e125);
    let _e133 = mat2x2<f32>(vec2<f32>(_e126.x, _e126.y), vec2<f32>(_e126.z, _e126.w));
    let _e134 = (_e118 + 1u);
    let _e141 = textureLoad(LB, vec2<i32>(bitcast<i32>((_e134 & 255u)), bitcast<i32>((_e134 >> bitcast<u32>(8i)))), 0i);
    let _e145 = bitcast<f32>(_e141.z);
    let _e147 = bitcast<f32>(_e141.w);
    let _e148 = (_e101.w & 8388608u);
    phi_2340_ = _e83.y;
    phi_2312_ = _e86;
    if (_e148 != 0u) {
        phi_2340_ = _e84.y;
        phi_2312_ = i32(_e84.x);
    }
    let _e154 = phi_2340_;
    let _e156 = phi_2312_;
    phi_2323_ = _e96;
    phi_2321_ = _e101;
    phi_2320_ = _e101.w;
    if (_e156 != _e94) {
        let _e159 = ((_e96 + _e156) - _e94);
        let _e164 = textureLoad(TB, vec2<i32>((_e159 & 2047i), (_e159 >> bitcast<u32>(11i))), 0i);
        if ((_e164.w & 8454143u) != (_e101.w & 8454143u)) {
            let _e169 = (_e145 == 0f);
            phi_1510_ = _e169;
            if !(_e169) {
                phi_1510_ = (_e114.x != 0f);
            }
            let _e174 = phi_1510_;
            phi_2325_ = _e96;
            phi_2317_ = _e101;
            if _e174 {
                let _e175 = bitcast<i32>(_e112.w);
                let _e180 = textureLoad(TB, vec2<i32>((_e175 & 2047i), (_e175 >> bitcast<u32>(11i))), 0i);
                phi_2325_ = _e175;
                phi_2317_ = _e180;
            }
            let _e182 = phi_2325_;
            let _e184 = phi_2317_;
            phi_2324_ = _e182;
            phi_2316_ = _e184;
        } else {
            phi_2324_ = _e159;
            phi_2316_ = _e164;
        }
        let _e186 = phi_2324_;
        let _e188 = phi_2316_;
        phi_2323_ = _e186;
        phi_2321_ = _e188;
        phi_2320_ = ((_e188.w & 4286578687u) | _e148);
    }
    let _e193 = phi_2323_;
    let _e195 = phi_2321_;
    let _e197 = phi_2320_;
    let _e198 = (_e197 & 469762048u);
    if ((_e198 == 67108864u) && (_e92 == 0i)) {
        let _e204 = f32((_e195.z & 65535u));
        let _e207 = f32((_e195.z >> bitcast<u32>(16i)));
        let _e213 = vec2<i32>(i32((-1f - _e204)), i32(((_e207 - _e204) + 1f)));
        phi_2327_ = _e213;
        if ((_e197 & 8388608u) != 0u) {
            phi_2327_ = -(_e213);
        }
        let _e218 = phi_2327_;
        let _e220 = (_e193 + _e218.x);
        let _e225 = textureLoad(TB, vec2<i32>((_e220 & 2047i), (_e220 >> bitcast<u32>(11i))), 0i);
        let _e227 = (_e193 + _e218.y);
        let _e232 = textureLoad(TB, vec2<i32>((_e227 & 2047i), (_e227 >> bitcast<u32>(11i))), 0i);
        phi_2328_ = _e232;
        if ((_e232.w & 8454143u) != (_e225.w & 8454143u)) {
            let _e238 = bitcast<i32>(_e112.w);
            let _e243 = textureLoad(TB, vec2<i32>((_e238 & 2047i), (_e238 >> bitcast<u32>(11i))), 0i);
            phi_2328_ = _e243;
        }
        let _e245 = phi_2328_;
        let _e248 = (f32(_e225.z) * 0.0000000014629181f);
        let _e251 = (f32(_e245.z) * 0.0000000014629181f);
        let _e252 = (_e251 - _e248);
        phi_2332_ = _e252;
        if (abs(_e252) > 3.1415927f) {
            phi_2332_ = (_e252 - (6.2831855f * sign(_e252)));
        }
        let _e259 = phi_2332_;
        let _e260 = (_e207 + -2f);
        let _e266 = clamp(round(((abs(_e259) * 0.31830987f) * _e260)), 1f, (_e207 + -3f));
        let _e267 = (_e260 - _e266);
        if (_e204 <= _e267) {
            phi_2403_ = _e154;
            if (_e204 == _e267) {
                phi_2403_ = -(_e154);
            }
            let _e276 = phi_2403_;
            phi_2402_ = _e276;
            phi_2350_ = -(((3.1415927f * sign(_e259)) - _e259));
            phi_2347_ = _e267;
            phi_2344_ = _e204;
        } else {
            let _e278 = (_e204 == (_e267 + 1f));
            if _e278 {
                phi_2346_ = 0f;
            } else {
                phi_2346_ = (_e204 - (_e267 + 2f));
            }
            let _e282 = phi_2346_;
            phi_2402_ = select(_e154, 0f, _e278);
            phi_2350_ = _e259;
            phi_2347_ = select(_e266, 0f, _e278);
            phi_2344_ = _e282;
        }
        let _e286 = phi_2402_;
        let _e288 = phi_2350_;
        let _e290 = phi_2347_;
        let _e292 = phi_2344_;
        if (_e292 == _e290) {
            phi_2354_ = _e251;
        } else {
            phi_2354_ = (_e248 + (_e288 * (_e292 / _e290)));
        }
        let _e298 = phi_2354_;
        phi_2400_ = _e286;
        phi_2353_ = _e298;
    } else {
        phi_2400_ = _e154;
        phi_2353_ = (f32(_e195.z) * 0.0000000014629181f);
    }
    let _e303 = phi_2400_;
    let _e305 = phi_2353_;
    let _e309 = vec2<f32>(sin(_e305), -(cos(_e305)));
    let _e311 = bitcast<vec2<f32>>(_e195.xy);
    phi_2409_ = _e147;
    if (_e147 != 0f) {
        phi_2409_ = max(_e147, (1f / length((_e133 * _e309))));
    }
    let _e318 = phi_2409_;
    if (_e145 != 0f) {
        let _e322 = (_e303 * sign(determinant(_e133)));
        let _e324 = ((_e197 & 1048576u) != 0u);
        phi_2406_ = _e322;
        if _e324 {
            phi_2406_ = min(_e322, 0f);
        }
        let _e327 = phi_2406_;
        phi_2438_ = _e327;
        if ((_e197 & 524288u) != 0u) {
            phi_2438_ = max(_e327, 0f);
        }
        let _e332 = phi_2438_;
        let _e334 = select(0f, _e318, (_e318 != 0f));
        let _e338 = select(_e145, _e334, ((_e334 > _e145) && (_e318 == 0f)));
        let _e339 = (_e338 + _e334);
        let _e340 = (_e309 * _e339);
        phi_2444_ = _e340;
        if (_e198 > 134217728u) {
            let _e346 = f32((_e195.z & 65535u));
            let _e347 = (_e346 * 0.000015259022f);
            let _e351 = sqrt(max((1f - (_e347 * _e347)), 0f));
            phi_2424_ = _e351;
            if (((_e197 & 4194304u) != 0u) == _e324) {
                phi_2424_ = -(_e351);
            }
            let _e355 = phi_2424_;
            let _e360 = (mat2x2<f32>(vec2<f32>(_e347, _e355), vec2<f32>(-(_e355), _e347)) * _e309);
            let _e361 = (_e133 * _e360);
            let _e370 = (_e198 == 335544320u);
            phi_1798_ = _e370;
            if !(_e370) {
                phi_1798_ = ((_e198 == 268435456u) && (_e347 >= 0.25f));
            }
            let _e376 = phi_1798_;
            if _e376 {
                phi_2429_ = (_e338 * (1f / max(_e347, select(0.25f, 1f, ((_e197 & 33554432u) != 0u)))));
            } else {
                phi_2429_ = ((_e338 * _e347) + (((abs(_e361.x) + abs(_e361.y)) * (1f / dot(_e361, _e361))) * 0.5f));
            }
            let _e387 = phi_2429_;
            phi_2445_ = _e340;
            if ((_e197 & 2097152u) != 0u) {
                if (_e339 <= ((_e387 * _e347) + (_e334 * 0.125f))) {
                    phi_2446_ = (_e360 * (_e339 * (65535f / _e346)));
                } else {
                    let _e397 = (_e360 * _e387);
                    phi_2446_ = (vec2<f32>(dot(_e340, _e340), dot(_e397, _e397)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e340, _e397)));
                }
                let _e405 = phi_2446_;
                phi_2445_ = _e405;
            }
            let _e407 = phi_2445_;
            phi_2444_ = _e407;
        }
        let _e409 = phi_2444_;
        phi_2467_ = (_e92 != 0i);
        phi_2462_ = (_e133 * (_e409 * _e332));
        phi_2447_ = _e311;
    } else {
        phi_2467_ = (((_e197 & 2147483648u) != 0u) && (_e92 != 1i));
        phi_2462_ = vec2<f32>(0f, 0f);
        phi_2447_ = select(_e311, _e114, vec2((_e92 == 2i)));
    }
    let _e421 = phi_2467_;
    let _e423 = phi_2462_;
    let _e425 = phi_2447_;
    let _e428 = (((_e133 * _e425) + _e423) + bitcast<vec2<f32>>(_e141.xy));
    let _e429 = (_e118 + 2u);
    let _e436 = textureLoad(LB, vec2<i32>(bitcast<i32>((_e429 & 255u)), bitcast<i32>((_e429 >> bitcast<u32>(8i)))), 0i);
    let _e444 = textureLoad(XC, vec2<i32>(bitcast<i32>((_e112.z & 255u)), bitcast<i32>((_e116 >> bitcast<u32>(8i)))), 0i);
    let _e446 = (_e444.x & 15u);
    if bi {
        let _e447 = (_e446 == 0u);
        if _e447 {
            phi_2490_ = _e444.y;
        } else {
            phi_2490_ = _e444.x;
        }
        let _e450 = phi_2490_;
        let _e452 = (_e450 >> bitcast<u32>(16i));
        let _e454 = j.T4_;
        if (_e452 == 0u) {
            phi_2491_ = 0f;
        } else {
            phi_2491_ = unpack2x16float(((_e452 + 1023u) * _e454)).x;
        }
        let _e461 = phi_2491_;
        phi_2492_ = _e461;
        if _e447 {
            phi_2492_ = -(_e461);
        }
        let _e464 = phi_2492_;
        l1_[0u] = _e464;
    }
    if di {
        Q0_ = f32(((_e444.x >> bitcast<u32>(4i)) & 15u));
    }
    if ci {
        let _e470 = (_e116 * 8u);
        let _e471 = (_e470 + 2u);
        let _e478 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e471 & 255u)), bitcast<i32>((_e471 >> bitcast<u32>(8i)))), 0i);
        let _e486 = (_e470 + 3u);
        let _e493 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e486 & 255u)), bitcast<i32>((_e486 >> bitcast<u32>(8i)))), 0i);
        if any((_e478 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e498 = ((mat2x2<f32>(vec2<f32>(_e478.x, _e478.y), vec2<f32>(_e478.z, _e478.w)) * _e428) + _e493.xy);
            unnamed.gl_ClipDistance[0i] = (_e498.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e498.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e498.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e498.y);
        } else {
            let _e514 = (_e493.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e514;
            unnamed.gl_ClipDistance[2i] = _e514;
            unnamed.gl_ClipDistance[1i] = _e514;
            unnamed.gl_ClipDistance[0i] = _e514;
        }
    }
    if (_e446 == 1u) {
        a1_ = unpack4x8unorm(_e444.y);
    } else {
        if (bi && (_e446 == 0u)) {
            let _e529 = (_e444.x >> bitcast<u32>(16i));
            let _e531 = j.T4_;
            if (_e529 == 0u) {
                phi_2531_ = 0f;
            } else {
                phi_2531_ = unpack2x16float(((_e529 + 1023u) * _e531)).x;
            }
            let _e538 = phi_2531_;
            l1_[1u] = _e538;
        } else {
            let _e540 = (_e116 * 8u);
            let _e547 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e540 & 255u)), bitcast<i32>((_e540 >> bitcast<u32>(8i)))), 0i);
            let _e555 = (_e540 + 1u);
            let _e562 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e555 & 255u)), bitcast<i32>((_e555 >> bitcast<u32>(8i)))), 0i);
            let _e571 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e444.y));
            let _e573 = ((mat2x2<f32>(vec2<f32>(_e547.x, _e547.y), vec2<f32>(_e547.z, _e547.w)) * _e428) + _e562.xy);
            if (_e562.z > 0.9f) {
                phi_2529_ = vec4<f32>(_e571.x, _e571.y, 2f, _e571.w);
            } else {
                phi_2529_ = vec4<f32>(_e571.x, _e571.y, _e562.w, _e571.w);
            }
            let _e588 = phi_2529_;
            if (f32(_e446) == 2f) {
                let _e595 = vec4<f32>(_e573.x, _e588.y, _e588.z, _e588.w);
                phi_2530_ = vec4<f32>(_e595.x, 0f, _e595.z, _e595.w);
            } else {
                let _e607 = vec4<f32>(_e588.x, _e588.y, -(_e588.z), _e588.w);
                let _e613 = vec4<f32>(_e573.x, _e607.y, _e607.z, _e607.w);
                phi_2530_ = vec4<f32>(_e613.x, _e573.y, _e613.z, _e613.w);
            }
            let _e621 = phi_2530_;
            a1_ = _e621;
            let _e623 = a1_[3u];
            a1_[3u] = -(_e623);
        }
    }
    phi_1198_ = ji;
    if ji {
        phi_1198_ = ((_e444.x & 2048u) != 0u);
    }
    let _e628 = phi_1198_;
    if _e628 {
        let _e629 = (_e116 * 8u);
        let _e630 = (_e629 + 4u);
        let _e637 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e630 & 255u)), bitcast<i32>((_e630 >> bitcast<u32>(8i)))), 0i);
        let _e645 = (_e629 + 5u);
        let _e652 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e645 & 255u)), bitcast<i32>((_e645 >> bitcast<u32>(8i)))), 0i);
        let _e655 = ((mat2x2<f32>(vec2<f32>(_e637.x, _e637.y), vec2<f32>(_e637.z, _e637.w)) * _e428) + _e652.xy);
        F1_ = vec3<f32>(_e655.x, _e655.y, (1f + _e652.z));
    } else {
        F1_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e421) {
        let _e663 = j.Yf;
        let _e665 = j.Zf;
        let _e673 = vec4<f32>(((_e428.x * _e663) - 1f), ((_e428.y * _e665) - sign(_e665)), 0f, 1f);
        phi_2546_ = vec4<f32>(_e673.x, _e673.y, ((f32(((_e436.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e673.w);
    } else {
        let _e686 = j.c3_;
        phi_2546_ = vec4(_e686);
    }
    let _e689 = phi_2546_;
    unnamed.gl_Position = _e689;
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
    let _e22 = F1_;
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
