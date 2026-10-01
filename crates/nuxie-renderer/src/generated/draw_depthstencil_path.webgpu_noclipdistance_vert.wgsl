struct nh {
    k2_: array<vec4<u32>>,
}

struct mh {
    k2_: array<vec4<u32>>,
}

struct Ef {
    k2_: array<vec2<u32>>,
}

struct UB {
    Qc: f32,
    Td: f32,
    bg: f32,
    cg: f32,
    A6_: u32,
    X9_: u32,
    Nf: u32,
    Of: u32,
    j8_: vec4<i32>,
    Lh: vec2<f32>,
    Ud: vec2<f32>,
    j2_: u32,
    Ph: f32,
    T4_: u32,
    a3_: f32,
    Vd: f32,
    Hf: u32,
    L3_: f32,
    M3_: f32,
    Wd: f32,
    Ih: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

struct Ff {
    k2_: array<vec4<f32>>,
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

@id(0) override ki: bool = true;
@id(2) override mi: bool = true;
@id(8) override si: bool = true;

@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(5)
var<storage> ZC: nh;
@group(0) @binding(2)
var<storage> LB: mh;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> WB_1: vec4<f32>;
var<private> XB_1: vec4<f32>;
@group(0) @binding(3)
var<storage> WC: Ef;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> l1_: vec2<f32>;
var<private> Q0_: f32;
@group(0) @binding(4)
var<storage> JB: Ff;
var<private> a1_: vec4<f32>;
var<private> r1_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var wa: sampler;

fn main_1() {
    var phi_2174_: f32;
    var phi_2146_: i32;
    var phi_1395_: bool;
    var phi_2159_: i32;
    var phi_2151_: vec4<u32>;
    var phi_2158_: i32;
    var phi_2150_: vec4<u32>;
    var phi_2157_: i32;
    var phi_2155_: vec4<u32>;
    var phi_2154_: u32;
    var phi_2161_: vec2<i32>;
    var phi_2162_: vec4<u32>;
    var phi_2166_: f32;
    var phi_2237_: f32;
    var phi_2180_: f32;
    var phi_2236_: f32;
    var phi_2184_: f32;
    var phi_2181_: f32;
    var phi_2178_: f32;
    var phi_2188_: f32;
    var phi_2234_: f32;
    var phi_2187_: f32;
    var phi_2243_: f32;
    var phi_2240_: f32;
    var phi_2272_: f32;
    var phi_2258_: f32;
    var phi_1683_: bool;
    var phi_2263_: f32;
    var phi_2280_: vec2<f32>;
    var phi_2279_: vec2<f32>;
    var phi_2278_: vec2<f32>;
    var phi_2301_: bool;
    var phi_2296_: vec2<f32>;
    var phi_2281_: vec2<f32>;
    var phi_2324_: u32;
    var phi_2325_: f32;
    var phi_2326_: f32;
    var phi_2363_: f32;
    var phi_2361_: vec4<f32>;
    var phi_2362_: vec4<f32>;
    var phi_1094_: bool;
    var phi_2364_: f32;
    var phi_2378_: vec4<f32>;

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
    let _e108 = ZC.k2_[(max((_e101.w & 65535u), 1u) - 1u)];
    let _e110 = bitcast<vec2<f32>>(_e108.xy);
    let _e112 = (_e108.z & 65535u);
    let _e114 = (_e112 * 4u);
    let _e117 = LB.k2_[_e114];
    let _e118 = bitcast<vec4<f32>>(_e117);
    let _e125 = mat2x2<f32>(vec2<f32>(_e118.x, _e118.y), vec2<f32>(_e118.z, _e118.w));
    let _e129 = LB.k2_[(_e114 + 1u)];
    let _e133 = bitcast<f32>(_e129.z);
    let _e135 = bitcast<f32>(_e129.w);
    let _e136 = (_e101.w & 8388608u);
    phi_2174_ = _e83.y;
    phi_2146_ = _e86;
    if (_e136 != 0u) {
        phi_2174_ = _e84.y;
        phi_2146_ = i32(_e84.x);
    }
    let _e142 = phi_2174_;
    let _e144 = phi_2146_;
    phi_2157_ = _e96;
    phi_2155_ = _e101;
    phi_2154_ = _e101.w;
    if (_e144 != _e94) {
        let _e147 = ((_e96 + _e144) - _e94);
        let _e152 = textureLoad(TB, vec2<i32>((_e147 & 2047i), (_e147 >> bitcast<u32>(11i))), 0i);
        if ((_e152.w & 8454143u) != (_e101.w & 8454143u)) {
            let _e157 = (_e133 == 0f);
            phi_1395_ = _e157;
            if !(_e157) {
                phi_1395_ = (_e110.x != 0f);
            }
            let _e162 = phi_1395_;
            phi_2159_ = _e96;
            phi_2151_ = _e101;
            if _e162 {
                let _e163 = bitcast<i32>(_e108.w);
                let _e168 = textureLoad(TB, vec2<i32>((_e163 & 2047i), (_e163 >> bitcast<u32>(11i))), 0i);
                phi_2159_ = _e163;
                phi_2151_ = _e168;
            }
            let _e170 = phi_2159_;
            let _e172 = phi_2151_;
            phi_2158_ = _e170;
            phi_2150_ = _e172;
        } else {
            phi_2158_ = _e147;
            phi_2150_ = _e152;
        }
        let _e174 = phi_2158_;
        let _e176 = phi_2150_;
        phi_2157_ = _e174;
        phi_2155_ = _e176;
        phi_2154_ = ((_e176.w & 4286578687u) | _e136);
    }
    let _e181 = phi_2157_;
    let _e183 = phi_2155_;
    let _e185 = phi_2154_;
    let _e186 = (_e185 & 469762048u);
    if ((_e186 == 67108864u) && (_e92 == 0i)) {
        let _e192 = f32((_e183.z & 65535u));
        let _e195 = f32((_e183.z >> bitcast<u32>(16i)));
        let _e201 = vec2<i32>(i32((-1f - _e192)), i32(((_e195 - _e192) + 1f)));
        phi_2161_ = _e201;
        if ((_e185 & 8388608u) != 0u) {
            phi_2161_ = -(_e201);
        }
        let _e206 = phi_2161_;
        let _e208 = (_e181 + _e206.x);
        let _e213 = textureLoad(TB, vec2<i32>((_e208 & 2047i), (_e208 >> bitcast<u32>(11i))), 0i);
        let _e215 = (_e181 + _e206.y);
        let _e220 = textureLoad(TB, vec2<i32>((_e215 & 2047i), (_e215 >> bitcast<u32>(11i))), 0i);
        phi_2162_ = _e220;
        if ((_e220.w & 8454143u) != (_e213.w & 8454143u)) {
            let _e226 = bitcast<i32>(_e108.w);
            let _e231 = textureLoad(TB, vec2<i32>((_e226 & 2047i), (_e226 >> bitcast<u32>(11i))), 0i);
            phi_2162_ = _e231;
        }
        let _e233 = phi_2162_;
        let _e236 = (f32(_e213.z) * 0.0000000014629181f);
        let _e239 = (f32(_e233.z) * 0.0000000014629181f);
        let _e240 = (_e239 - _e236);
        phi_2166_ = _e240;
        if (abs(_e240) > 3.1415927f) {
            phi_2166_ = (_e240 - (6.2831855f * sign(_e240)));
        }
        let _e247 = phi_2166_;
        let _e248 = (_e195 + -2f);
        let _e254 = clamp(round(((abs(_e247) * 0.31830987f) * _e248)), 1f, (_e195 + -3f));
        let _e255 = (_e248 - _e254);
        if (_e192 <= _e255) {
            phi_2237_ = _e142;
            if (_e192 == _e255) {
                phi_2237_ = -(_e142);
            }
            let _e264 = phi_2237_;
            phi_2236_ = _e264;
            phi_2184_ = -(((3.1415927f * sign(_e247)) - _e247));
            phi_2181_ = _e255;
            phi_2178_ = _e192;
        } else {
            let _e266 = (_e192 == (_e255 + 1f));
            if _e266 {
                phi_2180_ = 0f;
            } else {
                phi_2180_ = (_e192 - (_e255 + 2f));
            }
            let _e270 = phi_2180_;
            phi_2236_ = select(_e142, 0f, _e266);
            phi_2184_ = _e247;
            phi_2181_ = select(_e254, 0f, _e266);
            phi_2178_ = _e270;
        }
        let _e274 = phi_2236_;
        let _e276 = phi_2184_;
        let _e278 = phi_2181_;
        let _e280 = phi_2178_;
        if (_e280 == _e278) {
            phi_2188_ = _e239;
        } else {
            phi_2188_ = (_e236 + (_e276 * (_e280 / _e278)));
        }
        let _e286 = phi_2188_;
        phi_2234_ = _e274;
        phi_2187_ = _e286;
    } else {
        phi_2234_ = _e142;
        phi_2187_ = (f32(_e183.z) * 0.0000000014629181f);
    }
    let _e291 = phi_2234_;
    let _e293 = phi_2187_;
    let _e297 = vec2<f32>(sin(_e293), -(cos(_e293)));
    let _e299 = bitcast<vec2<f32>>(_e183.xy);
    phi_2243_ = _e135;
    if (_e135 != 0f) {
        phi_2243_ = max(_e135, (1f / length((_e125 * _e297))));
    }
    let _e306 = phi_2243_;
    if (_e133 != 0f) {
        let _e310 = (_e291 * sign(determinant(_e125)));
        let _e312 = ((_e185 & 1048576u) != 0u);
        phi_2240_ = _e310;
        if _e312 {
            phi_2240_ = min(_e310, 0f);
        }
        let _e315 = phi_2240_;
        phi_2272_ = _e315;
        if ((_e185 & 524288u) != 0u) {
            phi_2272_ = max(_e315, 0f);
        }
        let _e320 = phi_2272_;
        let _e322 = select(0f, _e306, (_e306 != 0f));
        let _e326 = select(_e133, _e322, ((_e322 > _e133) && (_e306 == 0f)));
        let _e327 = (_e326 + _e322);
        let _e328 = (_e297 * _e327);
        phi_2278_ = _e328;
        if (_e186 > 134217728u) {
            let _e334 = f32((_e183.z & 65535u));
            let _e335 = (_e334 * 0.000015259022f);
            let _e339 = sqrt(max((1f - (_e335 * _e335)), 0f));
            phi_2258_ = _e339;
            if (((_e185 & 4194304u) != 0u) == _e312) {
                phi_2258_ = -(_e339);
            }
            let _e343 = phi_2258_;
            let _e348 = (mat2x2<f32>(vec2<f32>(_e335, _e343), vec2<f32>(-(_e343), _e335)) * _e297);
            let _e349 = (_e125 * _e348);
            let _e358 = (_e186 == 335544320u);
            phi_1683_ = _e358;
            if !(_e358) {
                phi_1683_ = ((_e186 == 268435456u) && (_e335 >= 0.25f));
            }
            let _e364 = phi_1683_;
            if _e364 {
                phi_2263_ = (_e326 * (1f / max(_e335, select(0.25f, 1f, ((_e185 & 33554432u) != 0u)))));
            } else {
                phi_2263_ = ((_e326 * _e335) + (((abs(_e349.x) + abs(_e349.y)) * (1f / dot(_e349, _e349))) * 0.5f));
            }
            let _e375 = phi_2263_;
            phi_2279_ = _e328;
            if ((_e185 & 2097152u) != 0u) {
                if (_e327 <= ((_e375 * _e335) + (_e322 * 0.125f))) {
                    phi_2280_ = (_e348 * (_e327 * (65535f / _e334)));
                } else {
                    let _e385 = (_e348 * _e375);
                    phi_2280_ = (vec2<f32>(dot(_e328, _e328), dot(_e385, _e385)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e328, _e385)));
                }
                let _e393 = phi_2280_;
                phi_2279_ = _e393;
            }
            let _e395 = phi_2279_;
            phi_2278_ = _e395;
        }
        let _e397 = phi_2278_;
        phi_2301_ = (_e92 != 0i);
        phi_2296_ = (_e125 * (_e397 * _e320));
        phi_2281_ = _e299;
    } else {
        phi_2301_ = (((_e185 & 2147483648u) != 0u) && (_e92 != 1i));
        phi_2296_ = vec2<f32>(0f, 0f);
        phi_2281_ = select(_e299, _e110, vec2((_e92 == 2i)));
    }
    let _e409 = phi_2301_;
    let _e411 = phi_2296_;
    let _e413 = phi_2281_;
    let _e416 = (((_e125 * _e413) + _e411) + bitcast<vec2<f32>>(_e129.xy));
    let _e420 = LB.k2_[(_e114 + 2u)];
    let _e424 = WC.k2_[_e112];
    let _e426 = (_e424.x & 15u);
    if ki {
        let _e427 = (_e426 == 0u);
        if _e427 {
            phi_2324_ = _e424.y;
        } else {
            phi_2324_ = _e424.x;
        }
        let _e430 = phi_2324_;
        let _e432 = (_e430 >> bitcast<u32>(16i));
        let _e434 = j.T4_;
        if (_e432 == 0u) {
            phi_2325_ = 0f;
        } else {
            phi_2325_ = unpack2x16float(((_e432 + 1023u) * _e434)).x;
        }
        let _e441 = phi_2325_;
        phi_2326_ = _e441;
        if _e427 {
            phi_2326_ = -(_e441);
        }
        let _e444 = phi_2326_;
        l1_[0u] = _e444;
    }
    if mi {
        Q0_ = f32(((_e424.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e426 == 1u) {
        a1_ = unpack4x8unorm(_e424.y);
    } else {
        if (ki && (_e426 == 0u)) {
            let _e456 = (_e424.x >> bitcast<u32>(16i));
            let _e458 = j.T4_;
            if (_e456 == 0u) {
                phi_2363_ = 0f;
            } else {
                phi_2363_ = unpack2x16float(((_e456 + 1023u) * _e458)).x;
            }
            let _e465 = phi_2363_;
            l1_[1u] = _e465;
        } else {
            let _e467 = (_e112 * 8u);
            let _e470 = JB.k2_[_e467];
            let _e481 = JB.k2_[(_e467 + 1u)];
            let _e490 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e424.y));
            let _e492 = ((mat2x2<f32>(vec2<f32>(_e470.x, _e470.y), vec2<f32>(_e470.z, _e470.w)) * _e416) + _e481.xy);
            if (_e481.z > 0.9f) {
                phi_2361_ = vec4<f32>(_e490.x, _e490.y, 2f, _e490.w);
            } else {
                phi_2361_ = vec4<f32>(_e490.x, _e490.y, _e481.w, _e490.w);
            }
            let _e507 = phi_2361_;
            if (f32(_e426) == 2f) {
                let _e514 = vec4<f32>(_e492.x, _e507.y, _e507.z, _e507.w);
                phi_2362_ = vec4<f32>(_e514.x, 0f, _e514.z, _e514.w);
            } else {
                let _e526 = vec4<f32>(_e507.x, _e507.y, -(_e507.z), _e507.w);
                let _e532 = vec4<f32>(_e492.x, _e526.y, _e526.z, _e526.w);
                phi_2362_ = vec4<f32>(_e532.x, _e492.y, _e532.z, _e532.w);
            }
            let _e540 = phi_2362_;
            a1_ = _e540;
            let _e542 = a1_[3u];
            a1_[3u] = -(_e542);
        }
    }
    phi_1094_ = si;
    if si {
        phi_1094_ = ((_e424.x & 2048u) != 0u);
    }
    let _e547 = phi_1094_;
    if _e547 {
        let _e548 = (_e112 * 8u);
        let _e552 = JB.k2_[(_e548 + 4u)];
        let _e563 = JB.k2_[(_e548 + 5u)];
        let _e566 = ((mat2x2<f32>(vec2<f32>(_e552.x, _e552.y), vec2<f32>(_e552.z, _e552.w)) * _e416) + _e563.xy);
        phi_2364_ = (1f + _e563.z);
        if ((_e424.x & 4096u) != 0u) {
            phi_2364_ = (-1f - f32(((_e424.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e577 = phi_2364_;
        r1_ = vec3<f32>(_e566.x, _e566.y, _e577);
    } else {
        r1_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e409) {
        let _e583 = j.bg;
        let _e585 = j.cg;
        let _e593 = vec4<f32>(((_e416.x * _e583) - 1f), ((_e416.y * _e585) - sign(_e585)), 0f, 1f);
        phi_2378_ = vec4<f32>(_e593.x, _e593.y, ((f32(((_e420.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e593.w);
    } else {
        let _e606 = j.a3_;
        phi_2378_ = vec4(_e606);
    }
    let _e609 = phi_2378_;
    unnamed.gl_Position = _e609;
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
    let _e19 = r1_;
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
