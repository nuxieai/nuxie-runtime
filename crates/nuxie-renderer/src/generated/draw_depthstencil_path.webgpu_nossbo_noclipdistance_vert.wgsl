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

@id(0) override Eh: bool = true;
@id(2) override Gh: bool = true;
@id(8) override Mh: bool = true;

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
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var ca: sampler;

fn main_1() {
    var phi_2254_: f32;
    var phi_2226_: i32;
    var phi_1466_: bool;
    var phi_2239_: i32;
    var phi_2231_: vec4<u32>;
    var phi_2238_: i32;
    var phi_2230_: vec4<u32>;
    var phi_2237_: i32;
    var phi_2235_: vec4<u32>;
    var phi_2234_: u32;
    var phi_2241_: vec2<i32>;
    var phi_2242_: vec4<u32>;
    var phi_2246_: f32;
    var phi_2317_: f32;
    var phi_2260_: f32;
    var phi_2316_: f32;
    var phi_2264_: f32;
    var phi_2261_: f32;
    var phi_2258_: f32;
    var phi_2268_: f32;
    var phi_2314_: f32;
    var phi_2267_: f32;
    var phi_2323_: f32;
    var phi_2320_: f32;
    var phi_2377_: f32;
    var phi_2349_: i32;
    var phi_2359_: f32;
    var phi_1778_: bool;
    var phi_2366_: f32;
    var phi_2387_: vec2<f32>;
    var phi_2386_: vec2<f32>;
    var phi_2385_: vec2<f32>;
    var phi_2410_: bool;
    var phi_2405_: vec2<f32>;
    var phi_2388_: vec2<f32>;
    var phi_2435_: u32;
    var phi_2436_: f32;
    var phi_2437_: f32;
    var phi_2476_: f32;
    var phi_2474_: vec4<f32>;
    var phi_2475_: vec4<f32>;
    var phi_1149_: bool;
    var phi_2489_: vec4<f32>;

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
    let _e101 = (max((_e97.w & 65535u), 1u) - 1u);
    let _e108 = textureLoad(HD, vec2<i32>(bitcast<i32>((_e101 & 255u)), bitcast<i32>((_e101 >> bitcast<u32>(8i)))), 0i);
    let _e110 = bitcast<vec2<f32>>(_e108.xy);
    let _e112 = (_e108.z & 65535u);
    let _e114 = (_e112 * 4u);
    let _e121 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e114 & 255u)), bitcast<i32>((_e114 >> bitcast<u32>(8i)))), 0i);
    let _e122 = bitcast<vec4<f32>>(_e121);
    let _e129 = mat2x2<f32>(vec2<f32>(_e122.x, _e122.y), vec2<f32>(_e122.z, _e122.w));
    let _e130 = (_e114 + 1u);
    let _e137 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e130 & 255u)), bitcast<i32>((_e130 >> bitcast<u32>(8i)))), 0i);
    let _e141 = bitcast<f32>(_e137.z);
    let _e143 = bitcast<f32>(_e137.w);
    let _e144 = (_e97.w & 8388608u);
    phi_2254_ = _e79.y;
    phi_2226_ = _e82;
    if (_e144 != 0u) {
        phi_2254_ = _e80.y;
        phi_2226_ = i32(_e80.x);
    }
    let _e150 = phi_2254_;
    let _e152 = phi_2226_;
    phi_2237_ = _e92;
    phi_2235_ = _e97;
    phi_2234_ = _e97.w;
    if (_e152 != _e90) {
        let _e155 = ((_e92 + _e152) - _e90);
        let _e160 = textureLoad(JC, vec2<i32>((_e155 & 2047i), (_e155 >> bitcast<u32>(11i))), 0i);
        if ((_e160.w & 8454143u) != (_e97.w & 8454143u)) {
            let _e165 = (_e141 == 0f);
            phi_1466_ = _e165;
            if !(_e165) {
                phi_1466_ = (_e110.x != 0f);
            }
            let _e170 = phi_1466_;
            phi_2239_ = _e92;
            phi_2231_ = _e97;
            if _e170 {
                let _e171 = bitcast<i32>(_e108.w);
                let _e176 = textureLoad(JC, vec2<i32>((_e171 & 2047i), (_e171 >> bitcast<u32>(11i))), 0i);
                phi_2239_ = _e171;
                phi_2231_ = _e176;
            }
            let _e178 = phi_2239_;
            let _e180 = phi_2231_;
            phi_2238_ = _e178;
            phi_2230_ = _e180;
        } else {
            phi_2238_ = _e155;
            phi_2230_ = _e160;
        }
        let _e182 = phi_2238_;
        let _e184 = phi_2230_;
        phi_2237_ = _e182;
        phi_2235_ = _e184;
        phi_2234_ = ((_e184.w & 4286578687u) | _e144);
    }
    let _e189 = phi_2237_;
    let _e191 = phi_2235_;
    let _e193 = phi_2234_;
    let _e194 = (_e193 & 469762048u);
    if ((_e194 == 67108864u) && (_e88 == 0i)) {
        let _e200 = f32((_e191.z & 65535u));
        let _e203 = f32((_e191.z >> bitcast<u32>(16i)));
        let _e209 = vec2<i32>(i32((-1f - _e200)), i32(((_e203 - _e200) + 1f)));
        phi_2241_ = _e209;
        if ((_e193 & 8388608u) != 0u) {
            phi_2241_ = -(_e209);
        }
        let _e214 = phi_2241_;
        let _e216 = (_e189 + _e214.x);
        let _e221 = textureLoad(JC, vec2<i32>((_e216 & 2047i), (_e216 >> bitcast<u32>(11i))), 0i);
        let _e223 = (_e189 + _e214.y);
        let _e228 = textureLoad(JC, vec2<i32>((_e223 & 2047i), (_e223 >> bitcast<u32>(11i))), 0i);
        phi_2242_ = _e228;
        if ((_e228.w & 8454143u) != (_e221.w & 8454143u)) {
            let _e234 = bitcast<i32>(_e108.w);
            let _e239 = textureLoad(JC, vec2<i32>((_e234 & 2047i), (_e234 >> bitcast<u32>(11i))), 0i);
            phi_2242_ = _e239;
        }
        let _e241 = phi_2242_;
        let _e243 = bitcast<f32>(_e221.z);
        let _e245 = bitcast<f32>(_e241.z);
        let _e246 = (_e245 - _e243);
        phi_2246_ = _e246;
        if (abs(_e246) > 3.1415927f) {
            phi_2246_ = (_e246 - (6.2831855f * sign(_e246)));
        }
        let _e253 = phi_2246_;
        let _e254 = (_e203 + -2f);
        let _e260 = clamp(round(((abs(_e253) * 0.31830987f) * _e254)), 1f, (_e203 + -3f));
        let _e261 = (_e254 - _e260);
        if (_e200 <= _e261) {
            phi_2317_ = _e150;
            if (_e200 == _e261) {
                phi_2317_ = -(_e150);
            }
            let _e270 = phi_2317_;
            phi_2316_ = _e270;
            phi_2264_ = -(((3.1415927f * sign(_e253)) - _e253));
            phi_2261_ = _e261;
            phi_2258_ = _e200;
        } else {
            let _e272 = (_e200 == (_e261 + 1f));
            if _e272 {
                phi_2260_ = 0f;
            } else {
                phi_2260_ = (_e200 - (_e261 + 2f));
            }
            let _e276 = phi_2260_;
            phi_2316_ = select(_e150, 0f, _e272);
            phi_2264_ = _e253;
            phi_2261_ = select(_e260, 0f, _e272);
            phi_2258_ = _e276;
        }
        let _e280 = phi_2316_;
        let _e282 = phi_2264_;
        let _e284 = phi_2261_;
        let _e286 = phi_2258_;
        if (_e286 == _e284) {
            phi_2268_ = _e245;
        } else {
            phi_2268_ = (_e243 + (_e282 * (_e286 / _e284)));
        }
        let _e292 = phi_2268_;
        phi_2314_ = _e280;
        phi_2267_ = _e292;
    } else {
        phi_2314_ = _e150;
        phi_2267_ = bitcast<f32>(_e191.z);
    }
    let _e296 = phi_2314_;
    let _e298 = phi_2267_;
    let _e302 = vec2<f32>(sin(_e298), -(cos(_e298)));
    let _e304 = bitcast<vec2<f32>>(_e191.xy);
    phi_2323_ = _e143;
    if (_e143 != 0f) {
        phi_2323_ = max(_e143, (1f / length((_e129 * _e302))));
    }
    let _e311 = phi_2323_;
    if (_e141 != 0f) {
        let _e315 = (_e296 * sign(determinant(_e129)));
        let _e317 = ((_e193 & 1048576u) != 0u);
        phi_2320_ = _e315;
        if _e317 {
            phi_2320_ = min(_e315, 0f);
        }
        let _e320 = phi_2320_;
        phi_2377_ = _e320;
        if ((_e193 & 524288u) != 0u) {
            phi_2377_ = max(_e320, 0f);
        }
        let _e325 = phi_2377_;
        let _e327 = select(0f, _e311, (_e311 != 0f));
        let _e331 = select(_e141, _e327, ((_e327 > _e141) && (_e311 == 0f)));
        let _e332 = (_e331 + _e327);
        let _e333 = (_e302 * _e332);
        phi_2385_ = _e333;
        if (_e194 > 134217728u) {
            let _e335 = (_e193 & 4194304u);
            let _e337 = select(2i, -2i, (_e335 == 0u));
            phi_2349_ = _e337;
            if ((_e193 & 8388608u) != 0u) {
                phi_2349_ = -(_e337);
            }
            let _e342 = phi_2349_;
            let _e343 = (_e189 + _e342);
            let _e348 = textureLoad(JC, vec2<i32>((_e343 & 2047i), (_e343 >> bitcast<u32>(11i))), 0i);
            let _e352 = abs((bitcast<f32>(_e348.z) - _e298));
            phi_2359_ = _e352;
            if (_e352 > 3.1415927f) {
                phi_2359_ = (6.2831855f - _e352);
            }
            let _e356 = phi_2359_;
            let _e361 = ((_e356 * select(0.5f, -0.5f, ((_e335 != 0u) == _e317))) + _e298);
            let _e365 = vec2<f32>(sin(_e361), -(cos(_e361)));
            let _e366 = (_e129 * _e365);
            let _e376 = cos((_e356 * 0.5f));
            let _e377 = (_e194 == 335544320u);
            phi_1778_ = _e377;
            if !(_e377) {
                phi_1778_ = ((_e194 == 268435456u) && (_e376 >= 0.25f));
            }
            let _e383 = phi_1778_;
            if _e383 {
                phi_2366_ = (_e331 * (1f / max(_e376, select(0.25f, 1f, ((_e193 & 33554432u) != 0u)))));
            } else {
                phi_2366_ = ((_e331 * _e376) + (((abs(_e366.x) + abs(_e366.y)) * (1f / dot(_e366, _e366))) * 0.5f));
            }
            let _e394 = phi_2366_;
            phi_2386_ = _e333;
            if ((_e193 & 2097152u) != 0u) {
                if (_e332 <= ((_e394 * _e376) + (_e327 * 0.125f))) {
                    phi_2387_ = (_e365 * (_e332 * (1f / _e376)));
                } else {
                    let _e404 = (_e365 * _e394);
                    phi_2387_ = (vec2<f32>(dot(_e333, _e333), dot(_e404, _e404)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e333, _e404)));
                }
                let _e412 = phi_2387_;
                phi_2386_ = _e412;
            }
            let _e414 = phi_2386_;
            phi_2385_ = _e414;
        }
        let _e416 = phi_2385_;
        phi_2410_ = (_e88 != 0i);
        phi_2405_ = (_e129 * (_e416 * _e325));
        phi_2388_ = _e304;
    } else {
        phi_2410_ = (((_e193 & 2147483648u) != 0u) && (_e88 != 1i));
        phi_2405_ = vec2<f32>(0f, 0f);
        phi_2388_ = select(_e304, _e110, vec2((_e88 == 2i)));
    }
    let _e428 = phi_2410_;
    let _e430 = phi_2405_;
    let _e432 = phi_2388_;
    let _e435 = (((_e129 * _e432) + _e430) + bitcast<vec2<f32>>(_e137.xy));
    let _e436 = (_e114 + 2u);
    let _e443 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e436 & 255u)), bitcast<i32>((_e436 >> bitcast<u32>(8i)))), 0i);
    let _e451 = textureLoad(CD, vec2<i32>(bitcast<i32>((_e108.z & 255u)), bitcast<i32>((_e112 >> bitcast<u32>(8i)))), 0i);
    let _e453 = (_e451.x & 15u);
    if Eh {
        let _e454 = (_e453 == 0u);
        if _e454 {
            phi_2435_ = _e451.y;
        } else {
            phi_2435_ = _e451.x;
        }
        let _e457 = phi_2435_;
        let _e459 = (_e457 >> bitcast<u32>(16i));
        let _e461 = j.c6_;
        if (_e459 == 0u) {
            phi_2436_ = 0f;
        } else {
            phi_2436_ = unpack2x16float(((_e459 + 1023u) * _e461)).x;
        }
        let _e468 = phi_2436_;
        phi_2437_ = _e468;
        if _e454 {
            phi_2437_ = -(_e468);
        }
        let _e471 = phi_2437_;
        Y1_[0u] = _e471;
    }
    if Gh {
        g1_ = f32(((_e451.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e453 == 1u) {
        X1_ = unpack4x8unorm(_e451.y);
    } else {
        if (Eh && (_e453 == 0u)) {
            let _e483 = (_e451.x >> bitcast<u32>(16i));
            let _e485 = j.c6_;
            if (_e483 == 0u) {
                phi_2476_ = 0f;
            } else {
                phi_2476_ = unpack2x16float(((_e483 + 1023u) * _e485)).x;
            }
            let _e492 = phi_2476_;
            Y1_[1u] = _e492;
        } else {
            let _e494 = (_e112 * 8u);
            let _e501 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e494 & 255u)), bitcast<i32>((_e494 >> bitcast<u32>(8i)))), 0i);
            let _e509 = (_e494 + 1u);
            let _e516 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e509 & 255u)), bitcast<i32>((_e509 >> bitcast<u32>(8i)))), 0i);
            let _e525 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e451.y));
            let _e527 = ((mat2x2<f32>(vec2<f32>(_e501.x, _e501.y), vec2<f32>(_e501.z, _e501.w)) * _e435) + _e516.xy);
            if (_e516.z > 0.9f) {
                phi_2474_ = vec4<f32>(_e525.x, _e525.y, 2f, _e525.w);
            } else {
                phi_2474_ = vec4<f32>(_e525.x, _e525.y, _e516.w, _e525.w);
            }
            let _e542 = phi_2474_;
            if (f32(_e453) == 2f) {
                let _e549 = vec4<f32>(_e527.x, _e542.y, _e542.z, _e542.w);
                phi_2475_ = vec4<f32>(_e549.x, 0f, _e549.z, _e549.w);
            } else {
                let _e561 = vec4<f32>(_e542.x, _e542.y, -(_e542.z), _e542.w);
                let _e567 = vec4<f32>(_e527.x, _e561.y, _e561.z, _e561.w);
                phi_2475_ = vec4<f32>(_e567.x, _e527.y, _e567.z, _e567.w);
            }
            let _e575 = phi_2475_;
            X1_ = _e575;
            let _e577 = X1_[3u];
            X1_[3u] = -(_e577);
        }
    }
    phi_1149_ = Mh;
    if Mh {
        phi_1149_ = ((_e451.x & 2048u) != 0u);
    }
    let _e582 = phi_1149_;
    if _e582 {
        let _e583 = (_e112 * 8u);
        let _e584 = (_e583 + 4u);
        let _e591 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e584 & 255u)), bitcast<i32>((_e584 >> bitcast<u32>(8i)))), 0i);
        let _e599 = (_e583 + 5u);
        let _e606 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e599 & 255u)), bitcast<i32>((_e599 >> bitcast<u32>(8i)))), 0i);
        let _e609 = ((mat2x2<f32>(vec2<f32>(_e591.x, _e591.y), vec2<f32>(_e591.z, _e591.w)) * _e435) + _e606.xy);
        C2_ = vec3<f32>(_e609.x, _e609.y, (1f + _e606.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e428) {
        let _e617 = j.Hf;
        let _e619 = j.If;
        let _e627 = vec4<f32>(((_e435.x * _e617) - 1f), ((_e435.y * _e619) - sign(_e619)), 0f, 1f);
        phi_2489_ = vec4<f32>(_e627.x, _e627.y, (1f - (f32(_e443.x) * 0.000061035156f)), _e627.w);
    } else {
        let _e637 = j.W2_;
        phi_2489_ = vec4(_e637);
    }
    let _e640 = phi_2489_;
    unnamed.gl_Position = _e640;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) UB: vec4<f32>, @location(1) VB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    UB_1 = UB;
    VB_1 = VB;
    main_1();
    let _e16 = Y1_;
    let _e17 = g1_;
    let _e18 = X1_;
    let _e19 = C2_;
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
