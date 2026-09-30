enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct SB {
    yc: f32,
    Id: f32,
    Nf: f32,
    Of: f32,
    r6_: u32,
    Sb: u32,
    zf: u32,
    Af: u32,
    X7_: vec4<i32>,
    kh: vec2<f32>,
    Jd: vec2<f32>,
    f2_: u32,
    oh: f32,
    g6_: u32,
    W2_: f32,
    Kd: f32,
    tf: u32,
    F3_: f32,
    G3_: f32,
    Ld: f32,
    hh: u32,
    Rb: u32,
    ec: f32,
    fc: f32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(4) @interpolate(flat, either) member: vec2<f32>,
    @location(6) @interpolate(flat, either) member_1: f32,
    @location(0) member_2: vec4<f32>,
    @location(9) member_3: vec3<f32>,
}

@id(0) override Kh: bool = true;
@id(2) override Mh: bool = true;
@id(1) override Lh: bool = true;
@id(8) override Sh: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(5)
var HD: texture_2d<u32>;
@group(0) @binding(2)
var OB: texture_2d<u32>;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> UB_1: vec4<f32>;
var<private> VB_1: vec4<f32>;
@group(0) @binding(3)
var CD: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: SB;
var<private> Y1_: vec2<f32>;
var<private> g1_: f32;
@group(0) @binding(4)
var PB: texture_2d<f32>;
var<private> X1_: vec4<f32>;
var<private> C2_: vec3<f32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var ha: sampler;

fn main_1() {
    var phi_2342_: f32;
    var phi_2314_: i32;
    var phi_1509_: bool;
    var phi_2327_: i32;
    var phi_2319_: vec4<u32>;
    var phi_2326_: i32;
    var phi_2318_: vec4<u32>;
    var phi_2325_: i32;
    var phi_2323_: vec4<u32>;
    var phi_2322_: u32;
    var phi_2329_: vec2<i32>;
    var phi_2330_: vec4<u32>;
    var phi_2334_: f32;
    var phi_2405_: f32;
    var phi_2348_: f32;
    var phi_2404_: f32;
    var phi_2352_: f32;
    var phi_2349_: f32;
    var phi_2346_: f32;
    var phi_2356_: f32;
    var phi_2402_: f32;
    var phi_2355_: f32;
    var phi_2411_: f32;
    var phi_2408_: f32;
    var phi_2465_: f32;
    var phi_2437_: i32;
    var phi_2447_: f32;
    var phi_1821_: bool;
    var phi_2454_: f32;
    var phi_2475_: vec2<f32>;
    var phi_2474_: vec2<f32>;
    var phi_2473_: vec2<f32>;
    var phi_2498_: bool;
    var phi_2493_: vec2<f32>;
    var phi_2476_: vec2<f32>;
    var phi_2523_: u32;
    var phi_2524_: f32;
    var phi_2525_: f32;
    var phi_2566_: f32;
    var phi_2564_: vec4<f32>;
    var phi_2565_: vec4<f32>;
    var phi_1196_: bool;
    var phi_2581_: vec4<f32>;

    let _e80 = gl_InstanceIndex_1;
    let _e81 = UB_1;
    let _e82 = VB_1;
    let _e84 = i32(_e81.x);
    let _e87 = bitcast<i32>(_e81.w);
    let _e89 = (_e87 >> bitcast<u32>(2i));
    let _e90 = (_e87 & 3i);
    let _e92 = min(_e84, (_e89 - 1i));
    let _e94 = ((_e80 * _e89) + _e92);
    let _e99 = textureLoad(JC, vec2<i32>((_e94 & 2047i), (_e94 >> bitcast<u32>(11i))), 0i);
    let _e103 = (max((_e99.w & 65535u), 1u) - 1u);
    let _e110 = textureLoad(HD, vec2<i32>(bitcast<i32>((_e103 & 255u)), bitcast<i32>((_e103 >> bitcast<u32>(8i)))), 0i);
    let _e112 = bitcast<vec2<f32>>(_e110.xy);
    let _e114 = (_e110.z & 65535u);
    let _e116 = (_e114 * 4u);
    let _e123 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e116 & 255u)), bitcast<i32>((_e116 >> bitcast<u32>(8i)))), 0i);
    let _e124 = bitcast<vec4<f32>>(_e123);
    let _e131 = mat2x2<f32>(vec2<f32>(_e124.x, _e124.y), vec2<f32>(_e124.z, _e124.w));
    let _e132 = (_e116 + 1u);
    let _e139 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e132 & 255u)), bitcast<i32>((_e132 >> bitcast<u32>(8i)))), 0i);
    let _e143 = bitcast<f32>(_e139.z);
    let _e145 = bitcast<f32>(_e139.w);
    let _e146 = (_e99.w & 8388608u);
    phi_2342_ = _e81.y;
    phi_2314_ = _e84;
    if (_e146 != 0u) {
        phi_2342_ = _e82.y;
        phi_2314_ = i32(_e82.x);
    }
    let _e152 = phi_2342_;
    let _e154 = phi_2314_;
    phi_2325_ = _e94;
    phi_2323_ = _e99;
    phi_2322_ = _e99.w;
    if (_e154 != _e92) {
        let _e157 = ((_e94 + _e154) - _e92);
        let _e162 = textureLoad(JC, vec2<i32>((_e157 & 2047i), (_e157 >> bitcast<u32>(11i))), 0i);
        if ((_e162.w & 8454143u) != (_e99.w & 8454143u)) {
            let _e167 = (_e143 == 0f);
            phi_1509_ = _e167;
            if !(_e167) {
                phi_1509_ = (_e112.x != 0f);
            }
            let _e172 = phi_1509_;
            phi_2327_ = _e94;
            phi_2319_ = _e99;
            if _e172 {
                let _e173 = bitcast<i32>(_e110.w);
                let _e178 = textureLoad(JC, vec2<i32>((_e173 & 2047i), (_e173 >> bitcast<u32>(11i))), 0i);
                phi_2327_ = _e173;
                phi_2319_ = _e178;
            }
            let _e180 = phi_2327_;
            let _e182 = phi_2319_;
            phi_2326_ = _e180;
            phi_2318_ = _e182;
        } else {
            phi_2326_ = _e157;
            phi_2318_ = _e162;
        }
        let _e184 = phi_2326_;
        let _e186 = phi_2318_;
        phi_2325_ = _e184;
        phi_2323_ = _e186;
        phi_2322_ = ((_e186.w & 4286578687u) | _e146);
    }
    let _e191 = phi_2325_;
    let _e193 = phi_2323_;
    let _e195 = phi_2322_;
    let _e196 = (_e195 & 469762048u);
    if ((_e196 == 67108864u) && (_e90 == 0i)) {
        let _e202 = f32((_e193.z & 65535u));
        let _e205 = f32((_e193.z >> bitcast<u32>(16i)));
        let _e211 = vec2<i32>(i32((-1f - _e202)), i32(((_e205 - _e202) + 1f)));
        phi_2329_ = _e211;
        if ((_e195 & 8388608u) != 0u) {
            phi_2329_ = -(_e211);
        }
        let _e216 = phi_2329_;
        let _e218 = (_e191 + _e216.x);
        let _e223 = textureLoad(JC, vec2<i32>((_e218 & 2047i), (_e218 >> bitcast<u32>(11i))), 0i);
        let _e225 = (_e191 + _e216.y);
        let _e230 = textureLoad(JC, vec2<i32>((_e225 & 2047i), (_e225 >> bitcast<u32>(11i))), 0i);
        phi_2330_ = _e230;
        if ((_e230.w & 8454143u) != (_e223.w & 8454143u)) {
            let _e236 = bitcast<i32>(_e110.w);
            let _e241 = textureLoad(JC, vec2<i32>((_e236 & 2047i), (_e236 >> bitcast<u32>(11i))), 0i);
            phi_2330_ = _e241;
        }
        let _e243 = phi_2330_;
        let _e245 = bitcast<f32>(_e223.z);
        let _e247 = bitcast<f32>(_e243.z);
        let _e248 = (_e247 - _e245);
        phi_2334_ = _e248;
        if (abs(_e248) > 3.1415927f) {
            phi_2334_ = (_e248 - (6.2831855f * sign(_e248)));
        }
        let _e255 = phi_2334_;
        let _e256 = (_e205 + -2f);
        let _e262 = clamp(round(((abs(_e255) * 0.31830987f) * _e256)), 1f, (_e205 + -3f));
        let _e263 = (_e256 - _e262);
        if (_e202 <= _e263) {
            phi_2405_ = _e152;
            if (_e202 == _e263) {
                phi_2405_ = -(_e152);
            }
            let _e272 = phi_2405_;
            phi_2404_ = _e272;
            phi_2352_ = -(((3.1415927f * sign(_e255)) - _e255));
            phi_2349_ = _e263;
            phi_2346_ = _e202;
        } else {
            let _e274 = (_e202 == (_e263 + 1f));
            if _e274 {
                phi_2348_ = 0f;
            } else {
                phi_2348_ = (_e202 - (_e263 + 2f));
            }
            let _e278 = phi_2348_;
            phi_2404_ = select(_e152, 0f, _e274);
            phi_2352_ = _e255;
            phi_2349_ = select(_e262, 0f, _e274);
            phi_2346_ = _e278;
        }
        let _e282 = phi_2404_;
        let _e284 = phi_2352_;
        let _e286 = phi_2349_;
        let _e288 = phi_2346_;
        if (_e288 == _e286) {
            phi_2356_ = _e247;
        } else {
            phi_2356_ = (_e245 + (_e284 * (_e288 / _e286)));
        }
        let _e294 = phi_2356_;
        phi_2402_ = _e282;
        phi_2355_ = _e294;
    } else {
        phi_2402_ = _e152;
        phi_2355_ = bitcast<f32>(_e193.z);
    }
    let _e298 = phi_2402_;
    let _e300 = phi_2355_;
    let _e304 = vec2<f32>(sin(_e300), -(cos(_e300)));
    let _e306 = bitcast<vec2<f32>>(_e193.xy);
    phi_2411_ = _e145;
    if (_e145 != 0f) {
        phi_2411_ = max(_e145, (1f / length((_e131 * _e304))));
    }
    let _e313 = phi_2411_;
    if (_e143 != 0f) {
        let _e317 = (_e298 * sign(determinant(_e131)));
        let _e319 = ((_e195 & 1048576u) != 0u);
        phi_2408_ = _e317;
        if _e319 {
            phi_2408_ = min(_e317, 0f);
        }
        let _e322 = phi_2408_;
        phi_2465_ = _e322;
        if ((_e195 & 524288u) != 0u) {
            phi_2465_ = max(_e322, 0f);
        }
        let _e327 = phi_2465_;
        let _e329 = select(0f, _e313, (_e313 != 0f));
        let _e333 = select(_e143, _e329, ((_e329 > _e143) && (_e313 == 0f)));
        let _e334 = (_e333 + _e329);
        let _e335 = (_e304 * _e334);
        phi_2473_ = _e335;
        if (_e196 > 134217728u) {
            let _e337 = (_e195 & 4194304u);
            let _e339 = select(2i, -2i, (_e337 == 0u));
            phi_2437_ = _e339;
            if ((_e195 & 8388608u) != 0u) {
                phi_2437_ = -(_e339);
            }
            let _e344 = phi_2437_;
            let _e345 = (_e191 + _e344);
            let _e350 = textureLoad(JC, vec2<i32>((_e345 & 2047i), (_e345 >> bitcast<u32>(11i))), 0i);
            let _e354 = abs((bitcast<f32>(_e350.z) - _e300));
            phi_2447_ = _e354;
            if (_e354 > 3.1415927f) {
                phi_2447_ = (6.2831855f - _e354);
            }
            let _e358 = phi_2447_;
            let _e363 = ((_e358 * select(0.5f, -0.5f, ((_e337 != 0u) == _e319))) + _e300);
            let _e367 = vec2<f32>(sin(_e363), -(cos(_e363)));
            let _e368 = (_e131 * _e367);
            let _e378 = cos((_e358 * 0.5f));
            let _e379 = (_e196 == 335544320u);
            phi_1821_ = _e379;
            if !(_e379) {
                phi_1821_ = ((_e196 == 268435456u) && (_e378 >= 0.25f));
            }
            let _e385 = phi_1821_;
            if _e385 {
                phi_2454_ = (_e333 * (1f / max(_e378, select(0.25f, 1f, ((_e195 & 33554432u) != 0u)))));
            } else {
                phi_2454_ = ((_e333 * _e378) + (((abs(_e368.x) + abs(_e368.y)) * (1f / dot(_e368, _e368))) * 0.5f));
            }
            let _e396 = phi_2454_;
            phi_2474_ = _e335;
            if ((_e195 & 2097152u) != 0u) {
                if (_e334 <= ((_e396 * _e378) + (_e329 * 0.125f))) {
                    phi_2475_ = (_e367 * (_e334 * (1f / _e378)));
                } else {
                    let _e406 = (_e367 * _e396);
                    phi_2475_ = (vec2<f32>(dot(_e335, _e335), dot(_e406, _e406)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e335, _e406)));
                }
                let _e414 = phi_2475_;
                phi_2474_ = _e414;
            }
            let _e416 = phi_2474_;
            phi_2473_ = _e416;
        }
        let _e418 = phi_2473_;
        phi_2498_ = (_e90 != 0i);
        phi_2493_ = (_e131 * (_e418 * _e327));
        phi_2476_ = _e306;
    } else {
        phi_2498_ = (((_e195 & 2147483648u) != 0u) && (_e90 != 1i));
        phi_2493_ = vec2<f32>(0f, 0f);
        phi_2476_ = select(_e306, _e112, vec2((_e90 == 2i)));
    }
    let _e430 = phi_2498_;
    let _e432 = phi_2493_;
    let _e434 = phi_2476_;
    let _e437 = (((_e131 * _e434) + _e432) + bitcast<vec2<f32>>(_e139.xy));
    let _e438 = (_e116 + 2u);
    let _e445 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e438 & 255u)), bitcast<i32>((_e438 >> bitcast<u32>(8i)))), 0i);
    let _e453 = textureLoad(CD, vec2<i32>(bitcast<i32>((_e110.z & 255u)), bitcast<i32>((_e114 >> bitcast<u32>(8i)))), 0i);
    let _e455 = (_e453.x & 15u);
    if Kh {
        let _e456 = (_e455 == 0u);
        if _e456 {
            phi_2523_ = _e453.y;
        } else {
            phi_2523_ = _e453.x;
        }
        let _e459 = phi_2523_;
        let _e461 = (_e459 >> bitcast<u32>(16i));
        let _e463 = j.g6_;
        if (_e461 == 0u) {
            phi_2524_ = 0f;
        } else {
            phi_2524_ = unpack2x16float(((_e461 + 1023u) * _e463)).x;
        }
        let _e470 = phi_2524_;
        phi_2525_ = _e470;
        if _e456 {
            phi_2525_ = -(_e470);
        }
        let _e473 = phi_2525_;
        Y1_[0u] = _e473;
    }
    if Mh {
        g1_ = f32(((_e453.x >> bitcast<u32>(4i)) & 15u));
    }
    if Lh {
        let _e479 = (_e114 * 8u);
        let _e480 = (_e479 + 2u);
        let _e487 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e480 & 255u)), bitcast<i32>((_e480 >> bitcast<u32>(8i)))), 0i);
        let _e495 = (_e479 + 3u);
        let _e502 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e495 & 255u)), bitcast<i32>((_e495 >> bitcast<u32>(8i)))), 0i);
        if any((_e487 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e507 = ((mat2x2<f32>(vec2<f32>(_e487.x, _e487.y), vec2<f32>(_e487.z, _e487.w)) * _e437) + _e502.xy);
            unnamed.gl_ClipDistance[0i] = (_e507.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e507.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e507.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e507.y);
        } else {
            let _e523 = (_e502.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e523;
            unnamed.gl_ClipDistance[2i] = _e523;
            unnamed.gl_ClipDistance[1i] = _e523;
            unnamed.gl_ClipDistance[0i] = _e523;
        }
    }
    if (_e455 == 1u) {
        X1_ = unpack4x8unorm(_e453.y);
    } else {
        if (Kh && (_e455 == 0u)) {
            let _e538 = (_e453.x >> bitcast<u32>(16i));
            let _e540 = j.g6_;
            if (_e538 == 0u) {
                phi_2566_ = 0f;
            } else {
                phi_2566_ = unpack2x16float(((_e538 + 1023u) * _e540)).x;
            }
            let _e547 = phi_2566_;
            Y1_[1u] = _e547;
        } else {
            let _e549 = (_e114 * 8u);
            let _e556 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e549 & 255u)), bitcast<i32>((_e549 >> bitcast<u32>(8i)))), 0i);
            let _e564 = (_e549 + 1u);
            let _e571 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e564 & 255u)), bitcast<i32>((_e564 >> bitcast<u32>(8i)))), 0i);
            let _e580 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e453.y));
            let _e582 = ((mat2x2<f32>(vec2<f32>(_e556.x, _e556.y), vec2<f32>(_e556.z, _e556.w)) * _e437) + _e571.xy);
            if (_e571.z > 0.9f) {
                phi_2564_ = vec4<f32>(_e580.x, _e580.y, 2f, _e580.w);
            } else {
                phi_2564_ = vec4<f32>(_e580.x, _e580.y, _e571.w, _e580.w);
            }
            let _e597 = phi_2564_;
            if (f32(_e455) == 2f) {
                let _e604 = vec4<f32>(_e582.x, _e597.y, _e597.z, _e597.w);
                phi_2565_ = vec4<f32>(_e604.x, 0f, _e604.z, _e604.w);
            } else {
                let _e616 = vec4<f32>(_e597.x, _e597.y, -(_e597.z), _e597.w);
                let _e622 = vec4<f32>(_e582.x, _e616.y, _e616.z, _e616.w);
                phi_2565_ = vec4<f32>(_e622.x, _e582.y, _e622.z, _e622.w);
            }
            let _e630 = phi_2565_;
            X1_ = _e630;
            let _e632 = X1_[3u];
            X1_[3u] = -(_e632);
        }
    }
    phi_1196_ = Sh;
    if Sh {
        phi_1196_ = ((_e453.x & 2048u) != 0u);
    }
    let _e637 = phi_1196_;
    if _e637 {
        let _e638 = (_e114 * 8u);
        let _e639 = (_e638 + 4u);
        let _e646 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e639 & 255u)), bitcast<i32>((_e639 >> bitcast<u32>(8i)))), 0i);
        let _e654 = (_e638 + 5u);
        let _e661 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e654 & 255u)), bitcast<i32>((_e654 >> bitcast<u32>(8i)))), 0i);
        let _e664 = ((mat2x2<f32>(vec2<f32>(_e646.x, _e646.y), vec2<f32>(_e646.z, _e646.w)) * _e437) + _e661.xy);
        C2_ = vec3<f32>(_e664.x, _e664.y, (1f + _e661.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e430) {
        let _e672 = j.Nf;
        let _e674 = j.Of;
        let _e682 = vec4<f32>(((_e437.x * _e672) - 1f), ((_e437.y * _e674) - sign(_e674)), 0f, 1f);
        phi_2581_ = vec4<f32>(_e682.x, _e682.y, (1f - (f32(_e445.x) * 0.000061035156f)), _e682.w);
    } else {
        let _e692 = j.W2_;
        phi_2581_ = vec4(_e692);
    }
    let _e695 = phi_2581_;
    unnamed.gl_Position = _e695;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) UB: vec4<f32>, @location(1) VB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    UB_1 = UB;
    VB_1 = VB;
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
