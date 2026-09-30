enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct Fg {
    g2_: array<vec4<u32>>,
}

struct Eg {
    g2_: array<vec4<u32>>,
}

struct kf {
    g2_: array<vec2<u32>>,
}

struct SB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    eh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    ih: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    bh: u32,
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

@id(0) override Eh: bool = true;
@id(2) override Gh: bool = true;
@id(1) override Fh: bool = true;
@id(8) override Mh: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> HD: Fg;
@group(0) @binding(2)
var<storage> OB: Eg;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> UB_1: vec4<f32>;
var<private> VB_1: vec4<f32>;
@group(0) @binding(3)
var<storage> CD: kf;
@group(0) @binding(0)
var<uniform> j: SB;
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
    var phi_2241_: f32;
    var phi_2213_: i32;
    var phi_1416_: bool;
    var phi_2226_: i32;
    var phi_2218_: vec4<u32>;
    var phi_2225_: i32;
    var phi_2217_: vec4<u32>;
    var phi_2224_: i32;
    var phi_2222_: vec4<u32>;
    var phi_2221_: u32;
    var phi_2228_: vec2<i32>;
    var phi_2229_: vec4<u32>;
    var phi_2233_: f32;
    var phi_2304_: f32;
    var phi_2247_: f32;
    var phi_2303_: f32;
    var phi_2251_: f32;
    var phi_2248_: f32;
    var phi_2245_: f32;
    var phi_2255_: f32;
    var phi_2301_: f32;
    var phi_2254_: f32;
    var phi_2310_: f32;
    var phi_2307_: f32;
    var phi_2364_: f32;
    var phi_2336_: i32;
    var phi_2346_: f32;
    var phi_1728_: bool;
    var phi_2353_: f32;
    var phi_2374_: vec2<f32>;
    var phi_2373_: vec2<f32>;
    var phi_2372_: vec2<f32>;
    var phi_2397_: bool;
    var phi_2392_: vec2<f32>;
    var phi_2375_: vec2<f32>;
    var phi_2422_: u32;
    var phi_2423_: f32;
    var phi_2424_: f32;
    var phi_2465_: f32;
    var phi_2463_: vec4<f32>;
    var phi_2464_: vec4<f32>;
    var phi_1139_: bool;
    var phi_2480_: vec4<f32>;

    let _e78 = gl_InstanceIndex_1;
    let _e79 = UB_1;
    let _e80 = VB_1;
    let _e82 = i32(_e79.x);
    let _e85 = bitcast<i32>(_e79.w);
    let _e87 = (_e85 >> bitcast<u32>(2i));
    let _e88 = (_e85 & 3i);
    let _e90 = min(_e82, (_e87 - 1i));
    let _e92 = ((_e78 * _e87) + _e90);
    let _e97 = textureLoad(JC, vec2<i32>((_e92 & 2047i), (_e92 >> bitcast<u32>(11i))), 0i);
    let _e104 = HD.g2_[(max((_e97.w & 65535u), 1u) - 1u)];
    let _e106 = bitcast<vec2<f32>>(_e104.xy);
    let _e108 = (_e104.z & 65535u);
    let _e110 = (_e108 * 4u);
    let _e113 = OB.g2_[_e110];
    let _e114 = bitcast<vec4<f32>>(_e113);
    let _e121 = mat2x2<f32>(vec2<f32>(_e114.x, _e114.y), vec2<f32>(_e114.z, _e114.w));
    let _e125 = OB.g2_[(_e110 + 1u)];
    let _e129 = bitcast<f32>(_e125.z);
    let _e131 = bitcast<f32>(_e125.w);
    let _e132 = (_e97.w & 8388608u);
    phi_2241_ = _e79.y;
    phi_2213_ = _e82;
    if (_e132 != 0u) {
        phi_2241_ = _e80.y;
        phi_2213_ = i32(_e80.x);
    }
    let _e138 = phi_2241_;
    let _e140 = phi_2213_;
    phi_2224_ = _e92;
    phi_2222_ = _e97;
    phi_2221_ = _e97.w;
    if (_e140 != _e90) {
        let _e143 = ((_e92 + _e140) - _e90);
        let _e148 = textureLoad(JC, vec2<i32>((_e143 & 2047i), (_e143 >> bitcast<u32>(11i))), 0i);
        if ((_e148.w & 8454143u) != (_e97.w & 8454143u)) {
            let _e153 = (_e129 == 0f);
            phi_1416_ = _e153;
            if !(_e153) {
                phi_1416_ = (_e106.x != 0f);
            }
            let _e158 = phi_1416_;
            phi_2226_ = _e92;
            phi_2218_ = _e97;
            if _e158 {
                let _e159 = bitcast<i32>(_e104.w);
                let _e164 = textureLoad(JC, vec2<i32>((_e159 & 2047i), (_e159 >> bitcast<u32>(11i))), 0i);
                phi_2226_ = _e159;
                phi_2218_ = _e164;
            }
            let _e166 = phi_2226_;
            let _e168 = phi_2218_;
            phi_2225_ = _e166;
            phi_2217_ = _e168;
        } else {
            phi_2225_ = _e143;
            phi_2217_ = _e148;
        }
        let _e170 = phi_2225_;
        let _e172 = phi_2217_;
        phi_2224_ = _e170;
        phi_2222_ = _e172;
        phi_2221_ = ((_e172.w & 4286578687u) | _e132);
    }
    let _e177 = phi_2224_;
    let _e179 = phi_2222_;
    let _e181 = phi_2221_;
    let _e182 = (_e181 & 469762048u);
    if ((_e182 == 67108864u) && (_e88 == 0i)) {
        let _e188 = f32((_e179.z & 65535u));
        let _e191 = f32((_e179.z >> bitcast<u32>(16i)));
        let _e197 = vec2<i32>(i32((-1f - _e188)), i32(((_e191 - _e188) + 1f)));
        phi_2228_ = _e197;
        if ((_e181 & 8388608u) != 0u) {
            phi_2228_ = -(_e197);
        }
        let _e202 = phi_2228_;
        let _e204 = (_e177 + _e202.x);
        let _e209 = textureLoad(JC, vec2<i32>((_e204 & 2047i), (_e204 >> bitcast<u32>(11i))), 0i);
        let _e211 = (_e177 + _e202.y);
        let _e216 = textureLoad(JC, vec2<i32>((_e211 & 2047i), (_e211 >> bitcast<u32>(11i))), 0i);
        phi_2229_ = _e216;
        if ((_e216.w & 8454143u) != (_e209.w & 8454143u)) {
            let _e222 = bitcast<i32>(_e104.w);
            let _e227 = textureLoad(JC, vec2<i32>((_e222 & 2047i), (_e222 >> bitcast<u32>(11i))), 0i);
            phi_2229_ = _e227;
        }
        let _e229 = phi_2229_;
        let _e231 = bitcast<f32>(_e209.z);
        let _e233 = bitcast<f32>(_e229.z);
        let _e234 = (_e233 - _e231);
        phi_2233_ = _e234;
        if (abs(_e234) > 3.1415927f) {
            phi_2233_ = (_e234 - (6.2831855f * sign(_e234)));
        }
        let _e241 = phi_2233_;
        let _e242 = (_e191 + -2f);
        let _e248 = clamp(round(((abs(_e241) * 0.31830987f) * _e242)), 1f, (_e191 + -3f));
        let _e249 = (_e242 - _e248);
        if (_e188 <= _e249) {
            phi_2304_ = _e138;
            if (_e188 == _e249) {
                phi_2304_ = -(_e138);
            }
            let _e258 = phi_2304_;
            phi_2303_ = _e258;
            phi_2251_ = -(((3.1415927f * sign(_e241)) - _e241));
            phi_2248_ = _e249;
            phi_2245_ = _e188;
        } else {
            let _e260 = (_e188 == (_e249 + 1f));
            if _e260 {
                phi_2247_ = 0f;
            } else {
                phi_2247_ = (_e188 - (_e249 + 2f));
            }
            let _e264 = phi_2247_;
            phi_2303_ = select(_e138, 0f, _e260);
            phi_2251_ = _e241;
            phi_2248_ = select(_e248, 0f, _e260);
            phi_2245_ = _e264;
        }
        let _e268 = phi_2303_;
        let _e270 = phi_2251_;
        let _e272 = phi_2248_;
        let _e274 = phi_2245_;
        if (_e274 == _e272) {
            phi_2255_ = _e233;
        } else {
            phi_2255_ = (_e231 + (_e270 * (_e274 / _e272)));
        }
        let _e280 = phi_2255_;
        phi_2301_ = _e268;
        phi_2254_ = _e280;
    } else {
        phi_2301_ = _e138;
        phi_2254_ = bitcast<f32>(_e179.z);
    }
    let _e284 = phi_2301_;
    let _e286 = phi_2254_;
    let _e290 = vec2<f32>(sin(_e286), -(cos(_e286)));
    let _e292 = bitcast<vec2<f32>>(_e179.xy);
    phi_2310_ = _e131;
    if (_e131 != 0f) {
        phi_2310_ = max(_e131, (1f / length((_e121 * _e290))));
    }
    let _e299 = phi_2310_;
    if (_e129 != 0f) {
        let _e303 = (_e284 * sign(determinant(_e121)));
        let _e305 = ((_e181 & 1048576u) != 0u);
        phi_2307_ = _e303;
        if _e305 {
            phi_2307_ = min(_e303, 0f);
        }
        let _e308 = phi_2307_;
        phi_2364_ = _e308;
        if ((_e181 & 524288u) != 0u) {
            phi_2364_ = max(_e308, 0f);
        }
        let _e313 = phi_2364_;
        let _e315 = select(0f, _e299, (_e299 != 0f));
        let _e319 = select(_e129, _e315, ((_e315 > _e129) && (_e299 == 0f)));
        let _e320 = (_e319 + _e315);
        let _e321 = (_e290 * _e320);
        phi_2372_ = _e321;
        if (_e182 > 134217728u) {
            let _e323 = (_e181 & 4194304u);
            let _e325 = select(2i, -2i, (_e323 == 0u));
            phi_2336_ = _e325;
            if ((_e181 & 8388608u) != 0u) {
                phi_2336_ = -(_e325);
            }
            let _e330 = phi_2336_;
            let _e331 = (_e177 + _e330);
            let _e336 = textureLoad(JC, vec2<i32>((_e331 & 2047i), (_e331 >> bitcast<u32>(11i))), 0i);
            let _e340 = abs((bitcast<f32>(_e336.z) - _e286));
            phi_2346_ = _e340;
            if (_e340 > 3.1415927f) {
                phi_2346_ = (6.2831855f - _e340);
            }
            let _e344 = phi_2346_;
            let _e349 = ((_e344 * select(0.5f, -0.5f, ((_e323 != 0u) == _e305))) + _e286);
            let _e353 = vec2<f32>(sin(_e349), -(cos(_e349)));
            let _e354 = (_e121 * _e353);
            let _e364 = cos((_e344 * 0.5f));
            let _e365 = (_e182 == 335544320u);
            phi_1728_ = _e365;
            if !(_e365) {
                phi_1728_ = ((_e182 == 268435456u) && (_e364 >= 0.25f));
            }
            let _e371 = phi_1728_;
            if _e371 {
                phi_2353_ = (_e319 * (1f / max(_e364, select(0.25f, 1f, ((_e181 & 33554432u) != 0u)))));
            } else {
                phi_2353_ = ((_e319 * _e364) + (((abs(_e354.x) + abs(_e354.y)) * (1f / dot(_e354, _e354))) * 0.5f));
            }
            let _e382 = phi_2353_;
            phi_2373_ = _e321;
            if ((_e181 & 2097152u) != 0u) {
                if (_e320 <= ((_e382 * _e364) + (_e315 * 0.125f))) {
                    phi_2374_ = (_e353 * (_e320 * (1f / _e364)));
                } else {
                    let _e392 = (_e353 * _e382);
                    phi_2374_ = (vec2<f32>(dot(_e321, _e321), dot(_e392, _e392)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e321, _e392)));
                }
                let _e400 = phi_2374_;
                phi_2373_ = _e400;
            }
            let _e402 = phi_2373_;
            phi_2372_ = _e402;
        }
        let _e404 = phi_2372_;
        phi_2397_ = (_e88 != 0i);
        phi_2392_ = (_e121 * (_e404 * _e313));
        phi_2375_ = _e292;
    } else {
        phi_2397_ = (((_e181 & 2147483648u) != 0u) && (_e88 != 1i));
        phi_2392_ = vec2<f32>(0f, 0f);
        phi_2375_ = select(_e292, _e106, vec2((_e88 == 2i)));
    }
    let _e416 = phi_2397_;
    let _e418 = phi_2392_;
    let _e420 = phi_2375_;
    let _e423 = (((_e121 * _e420) + _e418) + bitcast<vec2<f32>>(_e125.xy));
    let _e427 = OB.g2_[(_e110 + 2u)];
    let _e431 = CD.g2_[_e108];
    let _e433 = (_e431.x & 15u);
    if Eh {
        let _e434 = (_e433 == 0u);
        if _e434 {
            phi_2422_ = _e431.y;
        } else {
            phi_2422_ = _e431.x;
        }
        let _e437 = phi_2422_;
        let _e439 = (_e437 >> bitcast<u32>(16i));
        let _e441 = j.c6_;
        if (_e439 == 0u) {
            phi_2423_ = 0f;
        } else {
            phi_2423_ = unpack2x16float(((_e439 + 1023u) * _e441)).x;
        }
        let _e448 = phi_2423_;
        phi_2424_ = _e448;
        if _e434 {
            phi_2424_ = -(_e448);
        }
        let _e451 = phi_2424_;
        Y1_[0u] = _e451;
    }
    if Gh {
        g1_ = f32(((_e431.x >> bitcast<u32>(4i)) & 15u));
    }
    if Fh {
        let _e457 = (_e108 * 8u);
        let _e461 = PB.g2_[(_e457 + 2u)];
        let _e472 = PB.g2_[(_e457 + 3u)];
        if any((_e461 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e477 = ((mat2x2<f32>(vec2<f32>(_e461.x, _e461.y), vec2<f32>(_e461.z, _e461.w)) * _e423) + _e472.xy);
            unnamed.gl_ClipDistance[0i] = (_e477.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e477.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e477.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e477.y);
        } else {
            let _e493 = (_e472.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e493;
            unnamed.gl_ClipDistance[2i] = _e493;
            unnamed.gl_ClipDistance[1i] = _e493;
            unnamed.gl_ClipDistance[0i] = _e493;
        }
    }
    if (_e433 == 1u) {
        X1_ = unpack4x8unorm(_e431.y);
    } else {
        if (Eh && (_e433 == 0u)) {
            let _e508 = (_e431.x >> bitcast<u32>(16i));
            let _e510 = j.c6_;
            if (_e508 == 0u) {
                phi_2465_ = 0f;
            } else {
                phi_2465_ = unpack2x16float(((_e508 + 1023u) * _e510)).x;
            }
            let _e517 = phi_2465_;
            Y1_[1u] = _e517;
        } else {
            let _e519 = (_e108 * 8u);
            let _e522 = PB.g2_[_e519];
            let _e533 = PB.g2_[(_e519 + 1u)];
            let _e542 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e431.y));
            let _e544 = ((mat2x2<f32>(vec2<f32>(_e522.x, _e522.y), vec2<f32>(_e522.z, _e522.w)) * _e423) + _e533.xy);
            if (_e533.z > 0.9f) {
                phi_2463_ = vec4<f32>(_e542.x, _e542.y, 2f, _e542.w);
            } else {
                phi_2463_ = vec4<f32>(_e542.x, _e542.y, _e533.w, _e542.w);
            }
            let _e559 = phi_2463_;
            if (f32(_e433) == 2f) {
                let _e566 = vec4<f32>(_e544.x, _e559.y, _e559.z, _e559.w);
                phi_2464_ = vec4<f32>(_e566.x, 0f, _e566.z, _e566.w);
            } else {
                let _e578 = vec4<f32>(_e559.x, _e559.y, -(_e559.z), _e559.w);
                let _e584 = vec4<f32>(_e544.x, _e578.y, _e578.z, _e578.w);
                phi_2464_ = vec4<f32>(_e584.x, _e544.y, _e584.z, _e584.w);
            }
            let _e592 = phi_2464_;
            X1_ = _e592;
            let _e594 = X1_[3u];
            X1_[3u] = -(_e594);
        }
    }
    phi_1139_ = Mh;
    if Mh {
        phi_1139_ = ((_e431.x & 2048u) != 0u);
    }
    let _e599 = phi_1139_;
    if _e599 {
        let _e600 = (_e108 * 8u);
        let _e604 = PB.g2_[(_e600 + 4u)];
        let _e615 = PB.g2_[(_e600 + 5u)];
        let _e618 = ((mat2x2<f32>(vec2<f32>(_e604.x, _e604.y), vec2<f32>(_e604.z, _e604.w)) * _e423) + _e615.xy);
        C2_ = vec3<f32>(_e618.x, _e618.y, (1f + _e615.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e416) {
        let _e626 = j.Hf;
        let _e628 = j.If;
        let _e636 = vec4<f32>(((_e423.x * _e626) - 1f), ((_e423.y * _e628) - sign(_e628)), 0f, 1f);
        phi_2480_ = vec4<f32>(_e636.x, _e636.y, (1f - (f32(_e427.x) * 0.000061035156f)), _e636.w);
    } else {
        let _e646 = j.W2_;
        phi_2480_ = vec4(_e646);
    }
    let _e649 = phi_2480_;
    unnamed.gl_Position = _e649;
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
