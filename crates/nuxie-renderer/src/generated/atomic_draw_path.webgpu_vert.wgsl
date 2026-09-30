struct Ig {
    g2_: array<vec4<u32>>,
}

struct Hg {
    g2_: array<vec4<u32>>,
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

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct kf {
    g2_: array<vec2<u32>>,
}

struct lf {
    g2_: array<vec4<f32>>,
}

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @location(1) @interpolate(flat, either) member_1: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> HD: Ig;
@group(0) @binding(2)
var<storage> OB: Hg;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> VB_1: vec4<f32>;
var<private> WB_1: vec4<f32>;
var<private> O: vec4<f32>;
var<private> D0_: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(3)
var<storage> CD: kf;
@group(0) @binding(4)
var<storage> PB: lf;
@group(3) @binding(9)
var ca: sampler;

fn main_1() {
    var phi_2299_: f32;
    var phi_2237_: f32;
    var phi_2209_: i32;
    var phi_1347_: bool;
    var phi_2222_: i32;
    var phi_2214_: vec4<u32>;
    var phi_2221_: i32;
    var phi_2213_: vec4<u32>;
    var phi_2220_: i32;
    var phi_2218_: vec4<u32>;
    var phi_2217_: u32;
    var phi_2224_: vec2<i32>;
    var phi_2225_: vec4<u32>;
    var phi_2229_: f32;
    var phi_2309_: f32;
    var phi_2243_: f32;
    var phi_2308_: f32;
    var phi_2251_: f32;
    var phi_2244_: f32;
    var phi_2241_: f32;
    var phi_2255_: f32;
    var phi_2330_: f32;
    var phi_2321_: f32;
    var phi_2306_: f32;
    var phi_2254_: f32;
    var phi_2304_: f32;
    var phi_2387_: f32;
    var phi_2398_: f32;
    var phi_2390_: f32;
    var phi_2479_: f32;
    var phi_2436_: i32;
    var phi_2445_: f32;
    var phi_1686_: bool;
    var phi_2452_: f32;
    var phi_2468_: vec2<f32>;
    var phi_2467_: vec2<f32>;
    var phi_2489_: vec4<f32>;
    var phi_2504_: vec2<f32>;
    var phi_2488_: vec4<f32>;
    var phi_2540_: vec4<f32>;
    var phi_2340_: f32;
    var phi_2339_: f32;
    var phi_2341_: f32;
    var phi_2345_: f32;
    var phi_2367_: f32;
    var phi_2365_: f32;
    var phi_2383_: vec4<f32>;
    var phi_2537_: vec2<f32>;
    var phi_2382_: vec4<f32>;
    var phi_2543_: vec4<f32>;
    var phi_2544_: bool;
    var phi_2538_: vec4<f32>;
    var phi_2532_: vec2<f32>;
    var phi_2506_: vec2<f32>;
    var phi_2578_: vec4<f32>;

    let _e69 = gl_InstanceIndex_1;
    let _e70 = VB_1;
    let _e71 = WB_1;
    let _e73 = i32(_e70.x);
    let _e77 = bitcast<i32>(_e70.w);
    let _e79 = (_e77 >> bitcast<u32>(2i));
    let _e80 = (_e77 & 3i);
    let _e82 = min(_e73, (_e79 - 1i));
    let _e84 = ((_e69 * _e79) + _e82);
    let _e89 = textureLoad(JC, vec2<i32>((_e84 & 2047i), (_e84 >> bitcast<u32>(11i))), 0i);
    let _e96 = HD.g2_[(max((_e89.w & 65535u), 1u) - 1u)];
    let _e98 = bitcast<vec2<f32>>(_e96.xy);
    let _e100 = (_e96.z & 65535u);
    let _e102 = (_e100 * 4u);
    let _e105 = OB.g2_[_e102];
    let _e106 = bitcast<vec4<f32>>(_e105);
    let _e113 = mat2x2<f32>(vec2<f32>(_e106.x, _e106.y), vec2<f32>(_e106.z, _e106.w));
    let _e117 = OB.g2_[(_e102 + 1u)];
    let _e121 = bitcast<f32>(_e117.z);
    let _e123 = bitcast<f32>(_e117.w);
    let _e124 = (_e89.w & 8388608u);
    phi_2299_ = _e70.z;
    phi_2237_ = _e70.y;
    phi_2209_ = _e73;
    if (_e124 != 0u) {
        phi_2299_ = _e71.z;
        phi_2237_ = _e71.y;
        phi_2209_ = i32(_e71.x);
    }
    let _e131 = phi_2299_;
    let _e133 = phi_2237_;
    let _e135 = phi_2209_;
    phi_2220_ = _e84;
    phi_2218_ = _e89;
    phi_2217_ = _e89.w;
    if (_e135 != _e82) {
        let _e138 = ((_e84 + _e135) - _e82);
        let _e143 = textureLoad(JC, vec2<i32>((_e138 & 2047i), (_e138 >> bitcast<u32>(11i))), 0i);
        if ((_e143.w & 8454143u) != (_e89.w & 8454143u)) {
            let _e148 = (_e121 == 0f);
            phi_1347_ = _e148;
            if !(_e148) {
                phi_1347_ = (_e98.x != 0f);
            }
            let _e153 = phi_1347_;
            phi_2222_ = _e84;
            phi_2214_ = _e89;
            if _e153 {
                let _e154 = bitcast<i32>(_e96.w);
                let _e159 = textureLoad(JC, vec2<i32>((_e154 & 2047i), (_e154 >> bitcast<u32>(11i))), 0i);
                phi_2222_ = _e154;
                phi_2214_ = _e159;
            }
            let _e161 = phi_2222_;
            let _e163 = phi_2214_;
            phi_2221_ = _e161;
            phi_2213_ = _e163;
        } else {
            phi_2221_ = _e138;
            phi_2213_ = _e143;
        }
        let _e165 = phi_2221_;
        let _e167 = phi_2213_;
        phi_2220_ = _e165;
        phi_2218_ = _e167;
        phi_2217_ = ((_e167.w & 4286578687u) | _e124);
    }
    let _e172 = phi_2220_;
    let _e174 = phi_2218_;
    let _e176 = phi_2217_;
    let _e177 = (_e176 & 469762048u);
    let _e180 = ((_e177 == 67108864u) && (_e80 == 0i));
    if _e180 {
        let _e183 = f32((_e174.z & 65535u));
        let _e186 = f32((_e174.z >> bitcast<u32>(16i)));
        let _e192 = vec2<i32>(i32((-1f - _e183)), i32(((_e186 - _e183) + 1f)));
        phi_2224_ = _e192;
        if ((_e176 & 8388608u) != 0u) {
            phi_2224_ = -(_e192);
        }
        let _e197 = phi_2224_;
        let _e199 = (_e172 + _e197.x);
        let _e204 = textureLoad(JC, vec2<i32>((_e199 & 2047i), (_e199 >> bitcast<u32>(11i))), 0i);
        let _e206 = (_e172 + _e197.y);
        let _e211 = textureLoad(JC, vec2<i32>((_e206 & 2047i), (_e206 >> bitcast<u32>(11i))), 0i);
        phi_2225_ = _e211;
        if ((_e211.w & 8454143u) != (_e204.w & 8454143u)) {
            let _e217 = bitcast<i32>(_e96.w);
            let _e222 = textureLoad(JC, vec2<i32>((_e217 & 2047i), (_e217 >> bitcast<u32>(11i))), 0i);
            phi_2225_ = _e222;
        }
        let _e224 = phi_2225_;
        let _e226 = bitcast<f32>(_e204.z);
        let _e228 = bitcast<f32>(_e224.z);
        let _e229 = (_e228 - _e226);
        phi_2229_ = _e229;
        if (abs(_e229) > 3.1415927f) {
            phi_2229_ = (_e229 - (6.2831855f * sign(_e229)));
        }
        let _e236 = phi_2229_;
        let _e237 = (_e186 + -2f);
        let _e243 = clamp(round(((abs(_e236) * 0.31830987f) * _e237)), 1f, (_e186 + -3f));
        let _e244 = (_e237 - _e243);
        if (_e183 <= _e244) {
            phi_2309_ = _e133;
            if (_e183 == _e244) {
                phi_2309_ = -(_e133);
            }
            let _e253 = phi_2309_;
            phi_2308_ = _e253;
            phi_2251_ = -(((3.1415927f * sign(_e236)) - _e236));
            phi_2244_ = _e244;
            phi_2241_ = _e183;
        } else {
            let _e255 = (_e183 == (_e244 + 1f));
            if _e255 {
                phi_2243_ = 0f;
            } else {
                phi_2243_ = (_e183 - (_e244 + 2f));
            }
            let _e259 = phi_2243_;
            phi_2308_ = select(_e133, 0f, _e255);
            phi_2251_ = _e236;
            phi_2244_ = select(_e243, 0f, _e255);
            phi_2241_ = _e259;
        }
        let _e263 = phi_2308_;
        let _e265 = phi_2251_;
        let _e267 = phi_2244_;
        let _e269 = phi_2241_;
        if (_e269 == _e267) {
            phi_2255_ = _e228;
        } else {
            phi_2255_ = (_e226 + (_e265 * (_e269 / _e267)));
        }
        let _e275 = phi_2255_;
        phi_2330_ = _e226;
        phi_2321_ = _e265;
        phi_2306_ = _e263;
        phi_2254_ = _e275;
    } else {
        phi_2330_ = f32();
        phi_2321_ = f32();
        phi_2306_ = _e133;
        phi_2254_ = bitcast<f32>(_e174.z);
    }
    let _e279 = phi_2330_;
    let _e281 = phi_2321_;
    let _e283 = phi_2306_;
    let _e285 = phi_2254_;
    let _e289 = vec2<f32>(sin(_e285), -(cos(_e285)));
    let _e291 = bitcast<vec2<f32>>(_e174.xy);
    phi_2304_ = _e123;
    if (_e123 != 0f) {
        phi_2304_ = max(_e123, (1f / length((_e113 * _e289))));
    }
    let _e298 = phi_2304_;
    if (_e121 != 0f) {
        let _e302 = (_e283 * sign(determinant(_e113)));
        let _e304 = ((_e176 & 1048576u) != 0u);
        phi_2387_ = _e302;
        if _e304 {
            phi_2387_ = min(_e302, 0f);
        }
        let _e307 = phi_2387_;
        phi_2398_ = _e307;
        if ((_e176 & 524288u) != 0u) {
            phi_2398_ = max(_e307, 0f);
        }
        let _e312 = phi_2398_;
        let _e313 = (_e298 != 0f);
        if _e313 {
            phi_2390_ = _e298;
        } else {
            let _e314 = (_e113 * _e289);
            phi_2390_ = (((abs(_e314.x) + abs(_e314.y)) * (1f / dot(_e314, _e314))) * 0.5f);
        }
        let _e325 = phi_2390_;
        let _e328 = ((_e325 > _e121) && (_e298 == 0f));
        phi_2479_ = 1f;
        if _e328 {
            phi_2479_ = (_e121 / _e325);
        }
        let _e331 = phi_2479_;
        let _e332 = select(_e121, _e325, _e328);
        let _e333 = (_e332 + _e325);
        let _e334 = (_e289 * _e333);
        let _e335 = (_e312 * _e333);
        let _e342 = (((vec2<f32>(_e335, -(_e335)) + vec2(_e332)) * (0.5f / _e325)) + vec2<f32>(0.5f, 0.5f));
        let _e345 = vec4<f32>(_e342.x, _e342.y, 0f, 0f);
        phi_2504_ = _e334;
        phi_2488_ = _e345;
        if (_e177 > 134217728u) {
            let _e347 = (_e176 & 4194304u);
            let _e349 = select(2i, -2i, (_e347 == 0u));
            phi_2436_ = _e349;
            if ((_e176 & 8388608u) != 0u) {
                phi_2436_ = -(_e349);
            }
            let _e354 = phi_2436_;
            let _e355 = (_e172 + _e354);
            let _e360 = textureLoad(JC, vec2<i32>((_e355 & 2047i), (_e355 >> bitcast<u32>(11i))), 0i);
            let _e364 = abs((bitcast<f32>(_e360.z) - _e285));
            phi_2445_ = _e364;
            if (_e364 > 3.1415927f) {
                phi_2445_ = (6.2831855f - _e364);
            }
            let _e368 = phi_2445_;
            let _e373 = ((_e368 * select(0.5f, -0.5f, ((_e347 != 0u) == _e304))) + _e285);
            let _e377 = vec2<f32>(sin(_e373), -(cos(_e373)));
            let _e378 = (_e113 * _e377);
            let _e386 = ((abs(_e378.x) + abs(_e378.y)) * (1f / dot(_e378, _e378)));
            let _e388 = cos((_e368 * 0.5f));
            let _e389 = (_e177 == 335544320u);
            phi_1686_ = _e389;
            if !(_e389) {
                phi_1686_ = ((_e177 == 268435456u) && (_e388 >= 0.25f));
            }
            let _e395 = phi_1686_;
            if _e395 {
                phi_2452_ = (_e332 * (1f / max(_e388, select(0.25f, 1f, ((_e176 & 33554432u) != 0u)))));
            } else {
                phi_2452_ = ((_e332 * _e388) + (_e386 * 0.5f));
            }
            let _e406 = phi_2452_;
            let _e408 = (_e406 + (_e386 * 0.5f));
            phi_2467_ = _e334;
            if ((_e176 & 2097152u) != 0u) {
                if (_e333 <= ((_e408 * _e388) + (_e325 * 0.125f))) {
                    phi_2468_ = (_e377 * (_e333 * (1f / _e388)));
                } else {
                    let _e418 = (_e377 * _e408);
                    phi_2468_ = (vec2<f32>(dot(_e334, _e334), dot(_e418, _e418)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e334, _e418)));
                }
                let _e426 = phi_2468_;
                phi_2467_ = _e426;
            }
            let _e428 = phi_2467_;
            let _e433 = ((_e408 - dot((_e428 * abs(_e312)), _e377)) / _e386);
            if _e304 {
                phi_2489_ = vec4<f32>(_e345.x, _e433, _e345.z, _e345.w);
            } else {
                phi_2489_ = vec4<f32>(_e433, _e345.y, _e345.z, _e345.w);
            }
            let _e445 = phi_2489_;
            phi_2504_ = _e428;
            phi_2488_ = _e445;
        }
        let _e447 = phi_2504_;
        let _e449 = phi_2488_;
        let _e451 = (_e449.xy * _e331);
        let _e457 = vec4<f32>(_e451.x, _e449.y, _e449.z, _e449.w);
        let _e464 = vec4<f32>(_e457.x, max(_e451.y, 0.0001f), _e457.z, _e457.w);
        phi_2540_ = _e464;
        if _e313 {
            phi_2540_ = vec4<f32>((-2f - _e451.x), _e464.y, _e464.z, _e464.w);
        }
        let _e472 = phi_2540_;
        phi_2544_ = (_e80 != 0i);
        phi_2538_ = _e472;
        phi_2532_ = (_e113 * (_e447 * _e312));
        phi_2506_ = _e291;
    } else {
        let _e476 = vec4<f32>(_e131, -1f, 0f, 0f);
        if (_e298 != 0f) {
            let _e482 = vec4<f32>(_e476.x, -2f, _e476.z, _e476.w);
            let _e487 = vec4<f32>(_e482.x, _e482.y, 1000000f, _e482.w);
            phi_2383_ = vec4<f32>(_e487.x, _e487.y, _e487.z, _e131);
            if _e180 {
                phi_2340_ = _e281;
                phi_2339_ = _e279;
                if (_e281 < 0f) {
                    phi_2340_ = -(_e281);
                    phi_2339_ = (_e279 + _e281);
                }
                let _e497 = phi_2340_;
                let _e499 = phi_2339_;
                let _e501 = ((_e285 - _e499) + 1.5707964f);
                let _e507 = clamp(((_e501 - (floor((_e501 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e497);
                phi_2341_ = _e507;
                if (_e507 > (_e497 * 0.5f)) {
                    phi_2341_ = (_e497 - _e507);
                }
                let _e512 = phi_2341_;
                let _e519 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e512), cos(_e512)) * abs(_e283))) * 0.5f);
                if (abs((_e497 - 1.5707964f)) < 0.001f) {
                    phi_2367_ = 0f;
                    phi_2365_ = 0f;
                } else {
                    let _e523 = tan(_e497);
                    let _e528 = (sign((1.5707964f - _e497)) / max(abs(_e523), 0.000001f));
                    if (_e528 >= 0f) {
                        phi_2345_ = (_e519.y - ((1f - _e519.x) * _e523));
                    } else {
                        phi_2345_ = (_e519.y + (_e519.x * _e523));
                    }
                    let _e540 = phi_2345_;
                    phi_2367_ = _e540;
                    phi_2365_ = _e528;
                }
                let _e542 = phi_2367_;
                let _e544 = phi_2365_;
                phi_2383_ = vec4<f32>((max(_e519.x, 0f) + 0.25f), (-2f - _e519.y), _e544, _e542);
            }
            let _e552 = phi_2383_;
            phi_2537_ = (_e113 * (_e289 * (_e283 * _e298)));
            phi_2382_ = _e552;
        } else {
            phi_2537_ = (sign(((_e289 * _e283) * _naga_inverse_2x2_f32(_e113))) * 0.5f);
            phi_2382_ = _e476;
        }
        let _e562 = phi_2537_;
        let _e564 = phi_2382_;
        phi_2543_ = _e564;
        if (((_e176 & 8388608u) != 0u) != ((_e176 & 16777216u) != 0u)) {
            phi_2543_ = (_e564 * vec4<f32>(-1f, 1f, 1f, 1f));
        }
        let _e572 = phi_2543_;
        phi_2544_ = (((_e176 & 2147483648u) != 0u) && (_e80 != 1i));
        phi_2538_ = _e572;
        phi_2532_ = _e562;
        phi_2506_ = select(_e291, _e98, vec2((_e80 == 2i)));
    }
    let _e581 = phi_2544_;
    let _e583 = phi_2538_;
    let _e585 = phi_2532_;
    let _e587 = phi_2506_;
    let _e590 = (((_e113 * _e587) + _e585) + bitcast<vec2<f32>>(_e117.xy));
    let _e593 = j.eh;
    let _e596 = select(_e583.xy, vec2<f32>(1f, -1f), vec2((_e593 != 0u)));
    let _e602 = vec4<f32>(_e596.x, _e583.y, _e583.z, _e583.w);
    if !(_e581) {
        O = vec4<f32>(_e602.x, _e596.y, _e602.z, _e602.w);
        D0_ = _e100;
        let _e611 = j.Hf;
        let _e613 = j.If;
        phi_2578_ = vec4<f32>(((_e590.x * _e611) - 1f), ((_e590.y * _e613) - sign(_e613)), 0f, 1f);
    } else {
        let _e623 = j.W2_;
        phi_2578_ = vec4(_e623);
    }
    let _e626 = phi_2578_;
    unnamed.gl_Position = _e626;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) VB: vec4<f32>, @location(1) WB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    VB_1 = VB;
    WB_1 = WB;
    main_1();
    let _e14 = O;
    let _e15 = D0_;
    let _e16 = unnamed.gl_Position;
    return VertexOutput(_e14, _e15, _e16);
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
