struct TB {
    tc: f32,
    Bd: f32,
    Hf: f32,
    If: f32,
    o6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    T7_: vec4<i32>,
    hh: vec2<f32>,
    Cd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    X2_: f32,
    Dd: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Ed: f32,
    eh: u32,
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
    @location(0) member: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(7)
var MC: texture_2d<u32>;
@group(0) @binding(5)
var ID: texture_2d<u32>;
@group(0) @binding(2)
var OB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> VB_1: vec4<f32>;
var<private> WB_1: vec4<f32>;
var<private> O: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(3)
var DD: texture_2d<u32>;
@group(0) @binding(4)
var PB: texture_2d<f32>;
@group(3) @binding(9)
var ca: sampler;

fn main_1() {
    var phi_2308_: f32;
    var phi_2246_: f32;
    var phi_2218_: i32;
    var phi_1370_: bool;
    var phi_2231_: i32;
    var phi_2223_: vec4<u32>;
    var phi_2230_: i32;
    var phi_2222_: vec4<u32>;
    var phi_2229_: i32;
    var phi_2227_: vec4<u32>;
    var phi_2226_: u32;
    var phi_2233_: vec2<i32>;
    var phi_2234_: vec4<u32>;
    var phi_2238_: f32;
    var phi_2318_: f32;
    var phi_2252_: f32;
    var phi_2317_: f32;
    var phi_2260_: f32;
    var phi_2253_: f32;
    var phi_2250_: f32;
    var phi_2264_: f32;
    var phi_2339_: f32;
    var phi_2330_: f32;
    var phi_2315_: f32;
    var phi_2263_: f32;
    var phi_2313_: f32;
    var phi_2396_: f32;
    var phi_2407_: f32;
    var phi_2399_: f32;
    var phi_2460_: f32;
    var phi_2434_: f32;
    var phi_1685_: bool;
    var phi_2439_: f32;
    var phi_2451_: vec2<f32>;
    var phi_2450_: vec2<f32>;
    var phi_2468_: vec4<f32>;
    var phi_2481_: vec2<f32>;
    var phi_2467_: vec4<f32>;
    var phi_2515_: vec4<f32>;
    var phi_2349_: f32;
    var phi_2348_: f32;
    var phi_2350_: f32;
    var phi_2354_: f32;
    var phi_2376_: f32;
    var phi_2374_: f32;
    var phi_2392_: vec4<f32>;
    var phi_2512_: vec2<f32>;
    var phi_2391_: vec4<f32>;
    var phi_2518_: vec4<f32>;
    var phi_2519_: bool;
    var phi_2513_: vec4<f32>;
    var phi_2507_: vec2<f32>;
    var phi_2483_: vec2<f32>;
    var phi_2551_: vec4<f32>;

    let _e73 = gl_InstanceIndex_1;
    let _e74 = VB_1;
    let _e75 = WB_1;
    let _e77 = i32(_e74.x);
    let _e81 = bitcast<i32>(_e74.w);
    let _e83 = (_e81 >> bitcast<u32>(2i));
    let _e84 = (_e81 & 3i);
    let _e86 = min(_e77, (_e83 - 1i));
    let _e88 = ((_e73 * _e83) + _e86);
    let _e93 = textureLoad(MC, vec2<i32>((_e88 & 2047i), (_e88 >> bitcast<u32>(11i))), 0i);
    let _e97 = (max((_e93.w & 65535u), 1u) - 1u);
    let _e104 = textureLoad(ID, vec2<i32>(bitcast<i32>((_e97 & 255u)), bitcast<i32>((_e97 >> bitcast<u32>(8i)))), 0i);
    let _e106 = bitcast<vec2<f32>>(_e104.xy);
    let _e110 = ((_e104.z & 65535u) * 4u);
    let _e117 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e110 & 255u)), bitcast<i32>((_e110 >> bitcast<u32>(8i)))), 0i);
    let _e118 = bitcast<vec4<f32>>(_e117);
    let _e125 = mat2x2<f32>(vec2<f32>(_e118.x, _e118.y), vec2<f32>(_e118.z, _e118.w));
    let _e126 = (_e110 + 1u);
    let _e133 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e126 & 255u)), bitcast<i32>((_e126 >> bitcast<u32>(8i)))), 0i);
    let _e137 = bitcast<f32>(_e133.z);
    let _e139 = bitcast<f32>(_e133.w);
    let _e140 = (_e93.w & 8388608u);
    phi_2308_ = _e74.z;
    phi_2246_ = _e74.y;
    phi_2218_ = _e77;
    if (_e140 != 0u) {
        phi_2308_ = _e75.z;
        phi_2246_ = _e75.y;
        phi_2218_ = i32(_e75.x);
    }
    let _e147 = phi_2308_;
    let _e149 = phi_2246_;
    let _e151 = phi_2218_;
    phi_2229_ = _e88;
    phi_2227_ = _e93;
    phi_2226_ = _e93.w;
    if (_e151 != _e86) {
        let _e154 = ((_e88 + _e151) - _e86);
        let _e159 = textureLoad(MC, vec2<i32>((_e154 & 2047i), (_e154 >> bitcast<u32>(11i))), 0i);
        if ((_e159.w & 8454143u) != (_e93.w & 8454143u)) {
            let _e164 = (_e137 == 0f);
            phi_1370_ = _e164;
            if !(_e164) {
                phi_1370_ = (_e106.x != 0f);
            }
            let _e169 = phi_1370_;
            phi_2231_ = _e88;
            phi_2223_ = _e93;
            if _e169 {
                let _e170 = bitcast<i32>(_e104.w);
                let _e175 = textureLoad(MC, vec2<i32>((_e170 & 2047i), (_e170 >> bitcast<u32>(11i))), 0i);
                phi_2231_ = _e170;
                phi_2223_ = _e175;
            }
            let _e177 = phi_2231_;
            let _e179 = phi_2223_;
            phi_2230_ = _e177;
            phi_2222_ = _e179;
        } else {
            phi_2230_ = _e154;
            phi_2222_ = _e159;
        }
        let _e181 = phi_2230_;
        let _e183 = phi_2222_;
        phi_2229_ = _e181;
        phi_2227_ = _e183;
        phi_2226_ = ((_e183.w & 4286578687u) | _e140);
    }
    let _e188 = phi_2229_;
    let _e190 = phi_2227_;
    let _e192 = phi_2226_;
    let _e193 = (_e192 & 469762048u);
    let _e196 = ((_e193 == 67108864u) && (_e84 == 0i));
    if _e196 {
        let _e199 = f32((_e190.z & 65535u));
        let _e202 = f32((_e190.z >> bitcast<u32>(16i)));
        let _e208 = vec2<i32>(i32((-1f - _e199)), i32(((_e202 - _e199) + 1f)));
        phi_2233_ = _e208;
        if ((_e192 & 8388608u) != 0u) {
            phi_2233_ = -(_e208);
        }
        let _e213 = phi_2233_;
        let _e215 = (_e188 + _e213.x);
        let _e220 = textureLoad(MC, vec2<i32>((_e215 & 2047i), (_e215 >> bitcast<u32>(11i))), 0i);
        let _e222 = (_e188 + _e213.y);
        let _e227 = textureLoad(MC, vec2<i32>((_e222 & 2047i), (_e222 >> bitcast<u32>(11i))), 0i);
        phi_2234_ = _e227;
        if ((_e227.w & 8454143u) != (_e220.w & 8454143u)) {
            let _e233 = bitcast<i32>(_e104.w);
            let _e238 = textureLoad(MC, vec2<i32>((_e233 & 2047i), (_e233 >> bitcast<u32>(11i))), 0i);
            phi_2234_ = _e238;
        }
        let _e240 = phi_2234_;
        let _e243 = (f32(_e220.z) * 0.0000000014629181f);
        let _e246 = (f32(_e240.z) * 0.0000000014629181f);
        let _e247 = (_e246 - _e243);
        phi_2238_ = _e247;
        if (abs(_e247) > 3.1415927f) {
            phi_2238_ = (_e247 - (6.2831855f * sign(_e247)));
        }
        let _e254 = phi_2238_;
        let _e255 = (_e202 + -2f);
        let _e261 = clamp(round(((abs(_e254) * 0.31830987f) * _e255)), 1f, (_e202 + -3f));
        let _e262 = (_e255 - _e261);
        if (_e199 <= _e262) {
            phi_2318_ = _e149;
            if (_e199 == _e262) {
                phi_2318_ = -(_e149);
            }
            let _e271 = phi_2318_;
            phi_2317_ = _e271;
            phi_2260_ = -(((3.1415927f * sign(_e254)) - _e254));
            phi_2253_ = _e262;
            phi_2250_ = _e199;
        } else {
            let _e273 = (_e199 == (_e262 + 1f));
            if _e273 {
                phi_2252_ = 0f;
            } else {
                phi_2252_ = (_e199 - (_e262 + 2f));
            }
            let _e277 = phi_2252_;
            phi_2317_ = select(_e149, 0f, _e273);
            phi_2260_ = _e254;
            phi_2253_ = select(_e261, 0f, _e273);
            phi_2250_ = _e277;
        }
        let _e281 = phi_2317_;
        let _e283 = phi_2260_;
        let _e285 = phi_2253_;
        let _e287 = phi_2250_;
        if (_e287 == _e285) {
            phi_2264_ = _e246;
        } else {
            phi_2264_ = (_e243 + (_e283 * (_e287 / _e285)));
        }
        let _e293 = phi_2264_;
        phi_2339_ = _e243;
        phi_2330_ = _e283;
        phi_2315_ = _e281;
        phi_2263_ = _e293;
    } else {
        phi_2339_ = f32();
        phi_2330_ = f32();
        phi_2315_ = _e149;
        phi_2263_ = (f32(_e190.z) * 0.0000000014629181f);
    }
    let _e298 = phi_2339_;
    let _e300 = phi_2330_;
    let _e302 = phi_2315_;
    let _e304 = phi_2263_;
    let _e308 = vec2<f32>(sin(_e304), -(cos(_e304)));
    let _e310 = bitcast<vec2<f32>>(_e190.xy);
    phi_2313_ = _e139;
    if (_e139 != 0f) {
        phi_2313_ = max(_e139, (1f / length((_e125 * _e308))));
    }
    let _e317 = phi_2313_;
    if (_e137 != 0f) {
        let _e321 = (_e302 * sign(determinant(_e125)));
        let _e323 = ((_e192 & 1048576u) != 0u);
        phi_2396_ = _e321;
        if _e323 {
            phi_2396_ = min(_e321, 0f);
        }
        let _e326 = phi_2396_;
        phi_2407_ = _e326;
        if ((_e192 & 524288u) != 0u) {
            phi_2407_ = max(_e326, 0f);
        }
        let _e331 = phi_2407_;
        let _e332 = (_e317 != 0f);
        if _e332 {
            phi_2399_ = _e317;
        } else {
            let _e333 = (_e125 * _e308);
            phi_2399_ = (((abs(_e333.x) + abs(_e333.y)) * (1f / dot(_e333, _e333))) * 0.5f);
        }
        let _e344 = phi_2399_;
        let _e347 = ((_e344 > _e137) && (_e317 == 0f));
        phi_2460_ = 1f;
        if _e347 {
            phi_2460_ = (_e137 / _e344);
        }
        let _e350 = phi_2460_;
        let _e351 = select(_e137, _e344, _e347);
        let _e352 = (_e351 + _e344);
        let _e353 = (_e308 * _e352);
        let _e354 = (_e331 * _e352);
        let _e361 = (((vec2<f32>(_e354, -(_e354)) + vec2(_e351)) * (0.5f / _e344)) + vec2<f32>(0.5f, 0.5f));
        let _e364 = vec4<f32>(_e361.x, _e361.y, 0f, 0f);
        phi_2481_ = _e353;
        phi_2467_ = _e364;
        if (_e193 > 134217728u) {
            let _e370 = f32((_e190.z & 65535u));
            let _e371 = (_e370 * 0.000015259022f);
            let _e375 = sqrt(max((1f - (_e371 * _e371)), 0f));
            phi_2434_ = _e375;
            if (((_e192 & 4194304u) != 0u) == _e323) {
                phi_2434_ = -(_e375);
            }
            let _e379 = phi_2434_;
            let _e384 = (mat2x2<f32>(vec2<f32>(_e371, _e379), vec2<f32>(-(_e379), _e371)) * _e308);
            let _e385 = (_e125 * _e384);
            let _e393 = ((abs(_e385.x) + abs(_e385.y)) * (1f / dot(_e385, _e385)));
            let _e394 = (_e193 == 335544320u);
            phi_1685_ = _e394;
            if !(_e394) {
                phi_1685_ = ((_e193 == 268435456u) && (_e371 >= 0.25f));
            }
            let _e400 = phi_1685_;
            if _e400 {
                phi_2439_ = (_e351 * (1f / max(_e371, select(0.25f, 1f, ((_e192 & 33554432u) != 0u)))));
            } else {
                phi_2439_ = ((_e351 * _e371) + (_e393 * 0.5f));
            }
            let _e411 = phi_2439_;
            let _e413 = (_e411 + (_e393 * 0.5f));
            phi_2450_ = _e353;
            if ((_e192 & 2097152u) != 0u) {
                if (_e352 <= ((_e413 * _e371) + (_e344 * 0.125f))) {
                    phi_2451_ = (_e384 * (_e352 * (65535f / _e370)));
                } else {
                    let _e423 = (_e384 * _e413);
                    phi_2451_ = (vec2<f32>(dot(_e353, _e353), dot(_e423, _e423)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e353, _e423)));
                }
                let _e431 = phi_2451_;
                phi_2450_ = _e431;
            }
            let _e433 = phi_2450_;
            let _e438 = ((_e413 - dot((_e433 * abs(_e331)), _e384)) / _e393);
            if _e323 {
                phi_2468_ = vec4<f32>(_e364.x, _e438, _e364.z, _e364.w);
            } else {
                phi_2468_ = vec4<f32>(_e438, _e364.y, _e364.z, _e364.w);
            }
            let _e450 = phi_2468_;
            phi_2481_ = _e433;
            phi_2467_ = _e450;
        }
        let _e452 = phi_2481_;
        let _e454 = phi_2467_;
        let _e456 = (_e454.xy * _e350);
        let _e462 = vec4<f32>(_e456.x, _e454.y, _e454.z, _e454.w);
        let _e469 = vec4<f32>(_e462.x, max(_e456.y, 0.0001f), _e462.z, _e462.w);
        phi_2515_ = _e469;
        if _e332 {
            phi_2515_ = vec4<f32>((-2f - _e456.x), _e469.y, _e469.z, _e469.w);
        }
        let _e477 = phi_2515_;
        phi_2519_ = (_e84 != 0i);
        phi_2513_ = _e477;
        phi_2507_ = (_e125 * (_e452 * _e331));
        phi_2483_ = _e310;
    } else {
        let _e481 = vec4<f32>(_e147, -1f, 0f, 0f);
        if (_e317 != 0f) {
            let _e487 = vec4<f32>(_e481.x, -2f, _e481.z, _e481.w);
            let _e492 = vec4<f32>(_e487.x, _e487.y, 1000000f, _e487.w);
            phi_2392_ = vec4<f32>(_e492.x, _e492.y, _e492.z, _e147);
            if _e196 {
                phi_2349_ = _e300;
                phi_2348_ = _e298;
                if (_e300 < 0f) {
                    phi_2349_ = -(_e300);
                    phi_2348_ = (_e298 + _e300);
                }
                let _e502 = phi_2349_;
                let _e504 = phi_2348_;
                let _e506 = ((_e304 - _e504) + 1.5707964f);
                let _e512 = clamp(((_e506 - (floor((_e506 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e502);
                phi_2350_ = _e512;
                if (_e512 > (_e502 * 0.5f)) {
                    phi_2350_ = (_e502 - _e512);
                }
                let _e517 = phi_2350_;
                let _e524 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e517), cos(_e517)) * abs(_e302))) * 0.5f);
                if (abs((_e502 - 1.5707964f)) < 0.001f) {
                    phi_2376_ = 0f;
                    phi_2374_ = 0f;
                } else {
                    let _e528 = tan(_e502);
                    let _e533 = (sign((1.5707964f - _e502)) / max(abs(_e528), 0.000001f));
                    if (_e533 >= 0f) {
                        phi_2354_ = (_e524.y - ((1f - _e524.x) * _e528));
                    } else {
                        phi_2354_ = (_e524.y + (_e524.x * _e528));
                    }
                    let _e545 = phi_2354_;
                    phi_2376_ = _e545;
                    phi_2374_ = _e533;
                }
                let _e547 = phi_2376_;
                let _e549 = phi_2374_;
                phi_2392_ = vec4<f32>((max(_e524.x, 0f) + 0.25f), (-2f - _e524.y), _e549, _e547);
            }
            let _e557 = phi_2392_;
            phi_2512_ = (_e125 * (_e308 * (_e302 * _e317)));
            phi_2391_ = _e557;
        } else {
            phi_2512_ = (sign(((_e308 * _e302) * _naga_inverse_2x2_f32(_e125))) * 0.5f);
            phi_2391_ = _e481;
        }
        let _e567 = phi_2512_;
        let _e569 = phi_2391_;
        phi_2518_ = _e569;
        if (((_e192 & 8388608u) != 0u) != ((_e192 & 16777216u) != 0u)) {
            phi_2518_ = (_e569 * vec4<f32>(-1f, 1f, 1f, 1f));
        }
        let _e577 = phi_2518_;
        phi_2519_ = (((_e192 & 2147483648u) != 0u) && (_e84 != 1i));
        phi_2513_ = _e577;
        phi_2507_ = _e567;
        phi_2483_ = select(_e310, _e106, vec2((_e84 == 2i)));
    }
    let _e586 = phi_2519_;
    let _e588 = phi_2513_;
    let _e590 = phi_2507_;
    let _e592 = phi_2483_;
    let _e598 = j.eh;
    let _e601 = select(_e588.xy, vec2<f32>(1f, -1f), vec2((_e598 != 0u)));
    let _e607 = vec4<f32>(_e601.x, _e588.y, _e588.z, _e588.w);
    O = vec4<f32>(_e607.x, _e601.y, _e607.z, _e607.w);
    if !(_e586) {
        let _e615 = (_e110 + 2u);
        let _e622 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e615 & 255u)), bitcast<i32>((_e615 >> bitcast<u32>(8i)))), 0i);
        let _e624 = bitcast<vec3<f32>>(_e622.yzw);
        let _e628 = (((((_e125 * _e592) + _e590) + bitcast<vec2<f32>>(_e133.xy)) * _e624.x) + _e624.yz);
        let _e631 = j.Cd[0u];
        let _e634 = j.Cd[1u];
        phi_2551_ = vec4<f32>(((_e628.x * _e631) - 1f), ((_e628.y * _e634) - sign(_e634)), 0f, 1f);
    } else {
        let _e644 = j.X2_;
        phi_2551_ = vec4(_e644);
    }
    let _e647 = phi_2551_;
    unnamed.gl_Position = _e647;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) VB: vec4<f32>, @location(1) WB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    VB_1 = VB;
    WB_1 = WB;
    main_1();
    let _e13 = O;
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
