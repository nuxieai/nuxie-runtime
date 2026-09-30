enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct Ig {
    g2_: array<vec4<u32>>,
}

struct Hg {
    g2_: array<vec4<u32>>,
}

struct kf {
    g2_: array<vec2<u32>>,
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

struct lf {
    g2_: array<vec4<f32>>,
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
var<storage> HD: Ig;
@group(0) @binding(2)
var<storage> OB: Hg;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> VB_1: vec4<f32>;
var<private> WB_1: vec4<f32>;
@group(0) @binding(3)
var<storage> CD: kf;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> Y1_: vec2<f32>;
var<private> g1_: f32;
@group(0) @binding(4)
var<storage> PB: lf;
var<private> X1_: vec4<f32>;
var<private> C2_: vec3<f32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var ca: sampler;

fn main_1() {
    var phi_2258_: f32;
    var phi_2230_: i32;
    var phi_1426_: bool;
    var phi_2243_: i32;
    var phi_2235_: vec4<u32>;
    var phi_2242_: i32;
    var phi_2234_: vec4<u32>;
    var phi_2241_: i32;
    var phi_2239_: vec4<u32>;
    var phi_2238_: u32;
    var phi_2245_: vec2<i32>;
    var phi_2246_: vec4<u32>;
    var phi_2250_: f32;
    var phi_2321_: f32;
    var phi_2264_: f32;
    var phi_2320_: f32;
    var phi_2268_: f32;
    var phi_2265_: f32;
    var phi_2262_: f32;
    var phi_2272_: f32;
    var phi_2318_: f32;
    var phi_2271_: f32;
    var phi_2327_: f32;
    var phi_2324_: f32;
    var phi_2381_: f32;
    var phi_2353_: i32;
    var phi_2363_: f32;
    var phi_1738_: bool;
    var phi_2370_: f32;
    var phi_2391_: vec2<f32>;
    var phi_2390_: vec2<f32>;
    var phi_2389_: vec2<f32>;
    var phi_2414_: bool;
    var phi_2409_: vec2<f32>;
    var phi_2392_: vec2<f32>;
    var phi_2439_: u32;
    var phi_2440_: f32;
    var phi_2441_: f32;
    var phi_2482_: f32;
    var phi_2480_: vec4<f32>;
    var phi_2481_: vec4<f32>;
    var phi_1147_: bool;
    var phi_2497_: vec4<f32>;

    let _e80 = gl_InstanceIndex_1;
    let _e81 = VB_1;
    let _e82 = WB_1;
    let _e84 = i32(_e81.x);
    let _e87 = bitcast<i32>(_e81.w);
    let _e89 = (_e87 >> bitcast<u32>(2i));
    let _e90 = (_e87 & 3i);
    let _e92 = min(_e84, (_e89 - 1i));
    let _e94 = ((_e80 * _e89) + _e92);
    let _e99 = textureLoad(JC, vec2<i32>((_e94 & 2047i), (_e94 >> bitcast<u32>(11i))), 0i);
    let _e106 = HD.g2_[(max((_e99.w & 65535u), 1u) - 1u)];
    let _e108 = bitcast<vec2<f32>>(_e106.xy);
    let _e110 = (_e106.z & 65535u);
    let _e112 = (_e110 * 4u);
    let _e115 = OB.g2_[_e112];
    let _e116 = bitcast<vec4<f32>>(_e115);
    let _e123 = mat2x2<f32>(vec2<f32>(_e116.x, _e116.y), vec2<f32>(_e116.z, _e116.w));
    let _e127 = OB.g2_[(_e112 + 1u)];
    let _e131 = bitcast<f32>(_e127.z);
    let _e133 = bitcast<f32>(_e127.w);
    let _e134 = (_e99.w & 8388608u);
    phi_2258_ = _e81.y;
    phi_2230_ = _e84;
    if (_e134 != 0u) {
        phi_2258_ = _e82.y;
        phi_2230_ = i32(_e82.x);
    }
    let _e140 = phi_2258_;
    let _e142 = phi_2230_;
    phi_2241_ = _e94;
    phi_2239_ = _e99;
    phi_2238_ = _e99.w;
    if (_e142 != _e92) {
        let _e145 = ((_e94 + _e142) - _e92);
        let _e150 = textureLoad(JC, vec2<i32>((_e145 & 2047i), (_e145 >> bitcast<u32>(11i))), 0i);
        if ((_e150.w & 8454143u) != (_e99.w & 8454143u)) {
            let _e155 = (_e131 == 0f);
            phi_1426_ = _e155;
            if !(_e155) {
                phi_1426_ = (_e108.x != 0f);
            }
            let _e160 = phi_1426_;
            phi_2243_ = _e94;
            phi_2235_ = _e99;
            if _e160 {
                let _e161 = bitcast<i32>(_e106.w);
                let _e166 = textureLoad(JC, vec2<i32>((_e161 & 2047i), (_e161 >> bitcast<u32>(11i))), 0i);
                phi_2243_ = _e161;
                phi_2235_ = _e166;
            }
            let _e168 = phi_2243_;
            let _e170 = phi_2235_;
            phi_2242_ = _e168;
            phi_2234_ = _e170;
        } else {
            phi_2242_ = _e145;
            phi_2234_ = _e150;
        }
        let _e172 = phi_2242_;
        let _e174 = phi_2234_;
        phi_2241_ = _e172;
        phi_2239_ = _e174;
        phi_2238_ = ((_e174.w & 4286578687u) | _e134);
    }
    let _e179 = phi_2241_;
    let _e181 = phi_2239_;
    let _e183 = phi_2238_;
    let _e184 = (_e183 & 469762048u);
    if ((_e184 == 67108864u) && (_e90 == 0i)) {
        let _e190 = f32((_e181.z & 65535u));
        let _e193 = f32((_e181.z >> bitcast<u32>(16i)));
        let _e199 = vec2<i32>(i32((-1f - _e190)), i32(((_e193 - _e190) + 1f)));
        phi_2245_ = _e199;
        if ((_e183 & 8388608u) != 0u) {
            phi_2245_ = -(_e199);
        }
        let _e204 = phi_2245_;
        let _e206 = (_e179 + _e204.x);
        let _e211 = textureLoad(JC, vec2<i32>((_e206 & 2047i), (_e206 >> bitcast<u32>(11i))), 0i);
        let _e213 = (_e179 + _e204.y);
        let _e218 = textureLoad(JC, vec2<i32>((_e213 & 2047i), (_e213 >> bitcast<u32>(11i))), 0i);
        phi_2246_ = _e218;
        if ((_e218.w & 8454143u) != (_e211.w & 8454143u)) {
            let _e224 = bitcast<i32>(_e106.w);
            let _e229 = textureLoad(JC, vec2<i32>((_e224 & 2047i), (_e224 >> bitcast<u32>(11i))), 0i);
            phi_2246_ = _e229;
        }
        let _e231 = phi_2246_;
        let _e233 = bitcast<f32>(_e211.z);
        let _e235 = bitcast<f32>(_e231.z);
        let _e236 = (_e235 - _e233);
        phi_2250_ = _e236;
        if (abs(_e236) > 3.1415927f) {
            phi_2250_ = (_e236 - (6.2831855f * sign(_e236)));
        }
        let _e243 = phi_2250_;
        let _e244 = (_e193 + -2f);
        let _e250 = clamp(round(((abs(_e243) * 0.31830987f) * _e244)), 1f, (_e193 + -3f));
        let _e251 = (_e244 - _e250);
        if (_e190 <= _e251) {
            phi_2321_ = _e140;
            if (_e190 == _e251) {
                phi_2321_ = -(_e140);
            }
            let _e260 = phi_2321_;
            phi_2320_ = _e260;
            phi_2268_ = -(((3.1415927f * sign(_e243)) - _e243));
            phi_2265_ = _e251;
            phi_2262_ = _e190;
        } else {
            let _e262 = (_e190 == (_e251 + 1f));
            if _e262 {
                phi_2264_ = 0f;
            } else {
                phi_2264_ = (_e190 - (_e251 + 2f));
            }
            let _e266 = phi_2264_;
            phi_2320_ = select(_e140, 0f, _e262);
            phi_2268_ = _e243;
            phi_2265_ = select(_e250, 0f, _e262);
            phi_2262_ = _e266;
        }
        let _e270 = phi_2320_;
        let _e272 = phi_2268_;
        let _e274 = phi_2265_;
        let _e276 = phi_2262_;
        if (_e276 == _e274) {
            phi_2272_ = _e235;
        } else {
            phi_2272_ = (_e233 + (_e272 * (_e276 / _e274)));
        }
        let _e282 = phi_2272_;
        phi_2318_ = _e270;
        phi_2271_ = _e282;
    } else {
        phi_2318_ = _e140;
        phi_2271_ = bitcast<f32>(_e181.z);
    }
    let _e286 = phi_2318_;
    let _e288 = phi_2271_;
    let _e292 = vec2<f32>(sin(_e288), -(cos(_e288)));
    let _e294 = bitcast<vec2<f32>>(_e181.xy);
    phi_2327_ = _e133;
    if (_e133 != 0f) {
        phi_2327_ = max(_e133, (1f / length((_e123 * _e292))));
    }
    let _e301 = phi_2327_;
    if (_e131 != 0f) {
        let _e305 = (_e286 * sign(determinant(_e123)));
        let _e307 = ((_e183 & 1048576u) != 0u);
        phi_2324_ = _e305;
        if _e307 {
            phi_2324_ = min(_e305, 0f);
        }
        let _e310 = phi_2324_;
        phi_2381_ = _e310;
        if ((_e183 & 524288u) != 0u) {
            phi_2381_ = max(_e310, 0f);
        }
        let _e315 = phi_2381_;
        let _e317 = select(0f, _e301, (_e301 != 0f));
        let _e321 = select(_e131, _e317, ((_e317 > _e131) && (_e301 == 0f)));
        let _e322 = (_e321 + _e317);
        let _e323 = (_e292 * _e322);
        phi_2389_ = _e323;
        if (_e184 > 134217728u) {
            let _e325 = (_e183 & 4194304u);
            let _e327 = select(2i, -2i, (_e325 == 0u));
            phi_2353_ = _e327;
            if ((_e183 & 8388608u) != 0u) {
                phi_2353_ = -(_e327);
            }
            let _e332 = phi_2353_;
            let _e333 = (_e179 + _e332);
            let _e338 = textureLoad(JC, vec2<i32>((_e333 & 2047i), (_e333 >> bitcast<u32>(11i))), 0i);
            let _e342 = abs((bitcast<f32>(_e338.z) - _e288));
            phi_2363_ = _e342;
            if (_e342 > 3.1415927f) {
                phi_2363_ = (6.2831855f - _e342);
            }
            let _e346 = phi_2363_;
            let _e351 = ((_e346 * select(0.5f, -0.5f, ((_e325 != 0u) == _e307))) + _e288);
            let _e355 = vec2<f32>(sin(_e351), -(cos(_e351)));
            let _e356 = (_e123 * _e355);
            let _e366 = cos((_e346 * 0.5f));
            let _e367 = (_e184 == 335544320u);
            phi_1738_ = _e367;
            if !(_e367) {
                phi_1738_ = ((_e184 == 268435456u) && (_e366 >= 0.25f));
            }
            let _e373 = phi_1738_;
            if _e373 {
                phi_2370_ = (_e321 * (1f / max(_e366, select(0.25f, 1f, ((_e183 & 33554432u) != 0u)))));
            } else {
                phi_2370_ = ((_e321 * _e366) + (((abs(_e356.x) + abs(_e356.y)) * (1f / dot(_e356, _e356))) * 0.5f));
            }
            let _e384 = phi_2370_;
            phi_2390_ = _e323;
            if ((_e183 & 2097152u) != 0u) {
                if (_e322 <= ((_e384 * _e366) + (_e317 * 0.125f))) {
                    phi_2391_ = (_e355 * (_e322 * (1f / _e366)));
                } else {
                    let _e394 = (_e355 * _e384);
                    phi_2391_ = (vec2<f32>(dot(_e323, _e323), dot(_e394, _e394)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e323, _e394)));
                }
                let _e402 = phi_2391_;
                phi_2390_ = _e402;
            }
            let _e404 = phi_2390_;
            phi_2389_ = _e404;
        }
        let _e406 = phi_2389_;
        phi_2414_ = (_e90 != 0i);
        phi_2409_ = (_e123 * (_e406 * _e315));
        phi_2392_ = _e294;
    } else {
        phi_2414_ = (((_e183 & 2147483648u) != 0u) && (_e90 != 1i));
        phi_2409_ = vec2<f32>(0f, 0f);
        phi_2392_ = select(_e294, _e108, vec2((_e90 == 2i)));
    }
    let _e418 = phi_2414_;
    let _e420 = phi_2409_;
    let _e422 = phi_2392_;
    let _e425 = (((_e123 * _e422) + _e420) + bitcast<vec2<f32>>(_e127.xy));
    let _e429 = OB.g2_[(_e112 + 2u)];
    let _e433 = CD.g2_[_e110];
    let _e435 = (_e433.x & 15u);
    if Hh {
        let _e436 = (_e435 == 0u);
        if _e436 {
            phi_2439_ = _e433.y;
        } else {
            phi_2439_ = _e433.x;
        }
        let _e439 = phi_2439_;
        let _e441 = (_e439 >> bitcast<u32>(16i));
        let _e443 = j.c6_;
        if (_e441 == 0u) {
            phi_2440_ = 0f;
        } else {
            phi_2440_ = unpack2x16float(((_e441 + 1023u) * _e443)).x;
        }
        let _e450 = phi_2440_;
        phi_2441_ = _e450;
        if _e436 {
            phi_2441_ = -(_e450);
        }
        let _e453 = phi_2441_;
        Y1_[0u] = _e453;
    }
    if Jh {
        g1_ = f32(((_e433.x >> bitcast<u32>(4i)) & 15u));
    }
    if Ih {
        let _e459 = (_e110 * 8u);
        let _e463 = PB.g2_[(_e459 + 2u)];
        let _e474 = PB.g2_[(_e459 + 3u)];
        if any((_e463 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e479 = ((mat2x2<f32>(vec2<f32>(_e463.x, _e463.y), vec2<f32>(_e463.z, _e463.w)) * _e425) + _e474.xy);
            unnamed.gl_ClipDistance[0i] = (_e479.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e479.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e479.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e479.y);
        } else {
            let _e495 = (_e474.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e495;
            unnamed.gl_ClipDistance[2i] = _e495;
            unnamed.gl_ClipDistance[1i] = _e495;
            unnamed.gl_ClipDistance[0i] = _e495;
        }
    }
    if (_e435 == 1u) {
        X1_ = unpack4x8unorm(_e433.y);
    } else {
        if (Hh && (_e435 == 0u)) {
            let _e510 = (_e433.x >> bitcast<u32>(16i));
            let _e512 = j.c6_;
            if (_e510 == 0u) {
                phi_2482_ = 0f;
            } else {
                phi_2482_ = unpack2x16float(((_e510 + 1023u) * _e512)).x;
            }
            let _e519 = phi_2482_;
            Y1_[1u] = _e519;
        } else {
            let _e521 = (_e110 * 8u);
            let _e524 = PB.g2_[_e521];
            let _e535 = PB.g2_[(_e521 + 1u)];
            let _e544 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e433.y));
            let _e546 = ((mat2x2<f32>(vec2<f32>(_e524.x, _e524.y), vec2<f32>(_e524.z, _e524.w)) * _e425) + _e535.xy);
            if (_e535.z > 0.9f) {
                phi_2480_ = vec4<f32>(_e544.x, _e544.y, 2f, _e544.w);
            } else {
                phi_2480_ = vec4<f32>(_e544.x, _e544.y, _e535.w, _e544.w);
            }
            let _e561 = phi_2480_;
            if (f32(_e435) == 2f) {
                let _e568 = vec4<f32>(_e546.x, _e561.y, _e561.z, _e561.w);
                phi_2481_ = vec4<f32>(_e568.x, 0f, _e568.z, _e568.w);
            } else {
                let _e580 = vec4<f32>(_e561.x, _e561.y, -(_e561.z), _e561.w);
                let _e586 = vec4<f32>(_e546.x, _e580.y, _e580.z, _e580.w);
                phi_2481_ = vec4<f32>(_e586.x, _e546.y, _e586.z, _e586.w);
            }
            let _e594 = phi_2481_;
            X1_ = _e594;
            let _e596 = X1_[3u];
            X1_[3u] = -(_e596);
        }
    }
    phi_1147_ = Ph;
    if Ph {
        phi_1147_ = ((_e433.x & 2048u) != 0u);
    }
    let _e601 = phi_1147_;
    if _e601 {
        let _e602 = (_e110 * 8u);
        let _e606 = PB.g2_[(_e602 + 4u)];
        let _e617 = PB.g2_[(_e602 + 5u)];
        let _e620 = ((mat2x2<f32>(vec2<f32>(_e606.x, _e606.y), vec2<f32>(_e606.z, _e606.w)) * _e425) + _e617.xy);
        C2_ = vec3<f32>(_e620.x, _e620.y, (1f + _e617.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e418) {
        let _e628 = j.Hf;
        let _e630 = j.If;
        let _e638 = vec4<f32>(((_e425.x * _e628) - 1f), ((_e425.y * _e630) - sign(_e630)), 0f, 1f);
        phi_2497_ = vec4<f32>(_e638.x, _e638.y, ((f32(((_e429.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e638.w);
    } else {
        let _e651 = j.W2_;
        phi_2497_ = vec4(_e651);
    }
    let _e654 = phi_2497_;
    unnamed.gl_Position = _e654;
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
