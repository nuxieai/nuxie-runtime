struct Dg {
    e2_: array<vec4<u32>>,
}

struct Cg {
    e2_: array<vec4<u32>>,
}

struct jf {
    e2_: array<vec2<u32>>,
}

struct BC {
    rc: f32,
    Bd: f32,
    Ff: f32,
    Gf: f32,
    q6_: u32,
    Ob: u32,
    rf: u32,
    sf: u32,
    V7_: vec4<i32>,
    ch: vec2<f32>,
    Cd: vec2<f32>,
    d2_: u32,
    gh: f32,
    f6_: u32,
    U2_: f32,
    Dd: f32,
    mf: u32,
    C3_: f32,
    D3_: f32,
    Ed: f32,
    Zg: u32,
    Nb: u32,
}

struct kf {
    e2_: array<vec4<f32>>,
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

@id(0) override Ch: bool = true;
@id(2) override Eh: bool = true;
@id(8) override Kh: bool = true;

@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> ID: Dg;
@group(0) @binding(2)
var<storage> PB: Cg;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> UB_1: vec4<f32>;
var<private> VB_1: vec4<f32>;
@group(0) @binding(3)
var<storage> DD: jf;
@group(0) @binding(0)
var<uniform> l: BC;
var<private> W1_: vec2<f32>;
var<private> g2_: f32;
@group(0) @binding(4)
var<storage> QB: kf;
var<private> V1_: vec4<f32>;
var<private> C2_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ea: sampler;

fn main_1() {
    var phi_2182_: f32;
    var phi_2154_: i32;
    var phi_1395_: bool;
    var phi_2167_: i32;
    var phi_2159_: vec4<u32>;
    var phi_2166_: i32;
    var phi_2158_: vec4<u32>;
    var phi_2165_: i32;
    var phi_2163_: vec4<u32>;
    var phi_2162_: u32;
    var phi_2169_: vec2<i32>;
    var phi_2170_: vec4<u32>;
    var phi_2174_: f32;
    var phi_2245_: f32;
    var phi_2188_: f32;
    var phi_2244_: f32;
    var phi_2192_: f32;
    var phi_2189_: f32;
    var phi_2186_: f32;
    var phi_2196_: f32;
    var phi_2242_: f32;
    var phi_2195_: f32;
    var phi_2251_: f32;
    var phi_2248_: f32;
    var phi_2305_: f32;
    var phi_2277_: i32;
    var phi_2287_: f32;
    var phi_1707_: bool;
    var phi_2294_: f32;
    var phi_2315_: vec2<f32>;
    var phi_2314_: vec2<f32>;
    var phi_2313_: vec2<f32>;
    var phi_2331_: vec2<f32>;
    var phi_2316_: vec2<f32>;
    var phi_2364_: u32;
    var phi_2335_: vec2<f32>;
    var phi_2334_: bool;
    var local: u32;
    var phi_2393_: u32;
    var phi_2394_: f32;
    var phi_2395_: f32;
    var phi_2434_: vec4<f32>;
    var phi_2433_: f32;
    var local_1: u32;
    var phi_2431_: vec4<f32>;
    var phi_2432_: vec4<f32>;
    var phi_1108_: bool;
    var local_2: u32;
    var phi_2448_: vec4<f32>;

    let _e78 = gl_InstanceIndex_1;
    let _e79 = UB_1;
    let _e80 = VB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e83 = i32(_e79.x);
            let _e86 = bitcast<i32>(_e79.w);
            let _e88 = (_e86 >> bitcast<u32>(2i));
            let _e89 = (_e86 & 3i);
            let _e91 = min(_e83, (_e88 - 1i));
            let _e93 = ((_e78 * _e88) + _e91);
            let _e98 = textureLoad(KC, vec2<i32>((_e93 & 2047i), (_e93 >> bitcast<u32>(11i))), 0i);
            let _e105 = ID.e2_[(max((_e98.w & 65535u), 1u) - 1u)];
            let _e107 = bitcast<vec2<f32>>(_e105.xy);
            let _e109 = (_e105.z & 65535u);
            let _e111 = (_e109 * 4u);
            let _e114 = PB.e2_[_e111];
            let _e115 = bitcast<vec4<f32>>(_e114);
            let _e122 = mat2x2<f32>(vec2<f32>(_e115.x, _e115.y), vec2<f32>(_e115.z, _e115.w));
            let _e126 = PB.e2_[(_e111 + 1u)];
            let _e130 = bitcast<f32>(_e126.z);
            let _e132 = bitcast<f32>(_e126.w);
            let _e133 = (_e98.w & 8388608u);
            phi_2182_ = _e79.y;
            phi_2154_ = _e83;
            local = _e109;
            local_1 = _e109;
            local_2 = _e109;
            if (_e133 != 0u) {
                phi_2182_ = _e80.y;
                phi_2154_ = i32(_e80.x);
            }
            let _e139 = phi_2182_;
            let _e141 = phi_2154_;
            phi_2165_ = _e93;
            phi_2163_ = _e98;
            phi_2162_ = _e98.w;
            if (_e141 != _e91) {
                let _e144 = ((_e93 + _e141) - _e91);
                let _e149 = textureLoad(KC, vec2<i32>((_e144 & 2047i), (_e144 >> bitcast<u32>(11i))), 0i);
                if ((_e149.w & 8454143u) != (_e98.w & 8454143u)) {
                    let _e154 = (_e130 == 0f);
                    phi_1395_ = _e154;
                    if !(_e154) {
                        phi_1395_ = (_e107.x != 0f);
                    }
                    let _e159 = phi_1395_;
                    phi_2167_ = _e93;
                    phi_2159_ = _e98;
                    if _e159 {
                        let _e160 = bitcast<i32>(_e105.w);
                        let _e165 = textureLoad(KC, vec2<i32>((_e160 & 2047i), (_e160 >> bitcast<u32>(11i))), 0i);
                        phi_2167_ = _e160;
                        phi_2159_ = _e165;
                    }
                    let _e167 = phi_2167_;
                    let _e169 = phi_2159_;
                    phi_2166_ = _e167;
                    phi_2158_ = _e169;
                } else {
                    phi_2166_ = _e144;
                    phi_2158_ = _e149;
                }
                let _e171 = phi_2166_;
                let _e173 = phi_2158_;
                phi_2165_ = _e171;
                phi_2163_ = _e173;
                phi_2162_ = ((_e173.w & 4286578687u) | _e133);
            }
            let _e178 = phi_2165_;
            let _e180 = phi_2163_;
            let _e182 = phi_2162_;
            let _e183 = (_e182 & 469762048u);
            if ((_e183 == 67108864u) && (_e89 == 0i)) {
                let _e189 = f32((_e180.z & 65535u));
                let _e192 = f32((_e180.z >> bitcast<u32>(16i)));
                let _e198 = vec2<i32>(i32((-1f - _e189)), i32(((_e192 - _e189) + 1f)));
                phi_2169_ = _e198;
                if ((_e182 & 8388608u) != 0u) {
                    phi_2169_ = -(_e198);
                }
                let _e203 = phi_2169_;
                let _e205 = (_e178 + _e203.x);
                let _e210 = textureLoad(KC, vec2<i32>((_e205 & 2047i), (_e205 >> bitcast<u32>(11i))), 0i);
                let _e212 = (_e178 + _e203.y);
                let _e217 = textureLoad(KC, vec2<i32>((_e212 & 2047i), (_e212 >> bitcast<u32>(11i))), 0i);
                phi_2170_ = _e217;
                if ((_e217.w & 8454143u) != (_e210.w & 8454143u)) {
                    let _e223 = bitcast<i32>(_e105.w);
                    let _e228 = textureLoad(KC, vec2<i32>((_e223 & 2047i), (_e223 >> bitcast<u32>(11i))), 0i);
                    phi_2170_ = _e228;
                }
                let _e230 = phi_2170_;
                let _e232 = bitcast<f32>(_e210.z);
                let _e234 = bitcast<f32>(_e230.z);
                let _e235 = (_e234 - _e232);
                phi_2174_ = _e235;
                if (abs(_e235) > 3.1415927f) {
                    phi_2174_ = (_e235 - (6.2831855f * sign(_e235)));
                }
                let _e242 = phi_2174_;
                let _e243 = (_e192 + -2f);
                let _e249 = clamp(round(((abs(_e242) * 0.31830987f) * _e243)), 1f, (_e192 + -3f));
                let _e250 = (_e243 - _e249);
                if (_e189 <= _e250) {
                    phi_2245_ = _e139;
                    if (_e189 == _e250) {
                        phi_2245_ = -(_e139);
                    }
                    let _e259 = phi_2245_;
                    phi_2244_ = _e259;
                    phi_2192_ = -(((3.1415927f * sign(_e242)) - _e242));
                    phi_2189_ = _e250;
                    phi_2186_ = _e189;
                } else {
                    let _e261 = (_e189 == (_e250 + 1f));
                    if _e261 {
                        phi_2188_ = 0f;
                    } else {
                        phi_2188_ = (_e189 - (_e250 + 2f));
                    }
                    let _e265 = phi_2188_;
                    phi_2244_ = select(_e139, 0f, _e261);
                    phi_2192_ = _e242;
                    phi_2189_ = select(_e249, 0f, _e261);
                    phi_2186_ = _e265;
                }
                let _e269 = phi_2244_;
                let _e271 = phi_2192_;
                let _e273 = phi_2189_;
                let _e275 = phi_2186_;
                if (_e275 == _e273) {
                    phi_2196_ = _e234;
                } else {
                    phi_2196_ = (_e232 + (_e271 * (_e275 / _e273)));
                }
                let _e281 = phi_2196_;
                phi_2242_ = _e269;
                phi_2195_ = _e281;
            } else {
                phi_2242_ = _e139;
                phi_2195_ = bitcast<f32>(_e180.z);
            }
            let _e285 = phi_2242_;
            let _e287 = phi_2195_;
            let _e291 = vec2<f32>(sin(_e287), -(cos(_e287)));
            let _e293 = bitcast<vec2<f32>>(_e180.xy);
            phi_2251_ = _e132;
            if (_e132 != 0f) {
                phi_2251_ = max(_e132, (1f / length((_e122 * _e291))));
            }
            let _e300 = phi_2251_;
            if (_e130 != 0f) {
                let _e304 = (_e285 * sign(determinant(_e122)));
                let _e306 = ((_e182 & 1048576u) != 0u);
                phi_2248_ = _e304;
                if _e306 {
                    phi_2248_ = min(_e304, 0f);
                }
                let _e309 = phi_2248_;
                phi_2305_ = _e309;
                if ((_e182 & 524288u) != 0u) {
                    phi_2305_ = max(_e309, 0f);
                }
                let _e314 = phi_2305_;
                let _e316 = select(0f, _e300, (_e300 != 0f));
                let _e320 = select(_e130, _e316, ((_e316 > _e130) && (_e300 == 0f)));
                let _e321 = (_e320 + _e316);
                let _e322 = (_e291 * _e321);
                phi_2313_ = _e322;
                if (_e183 > 134217728u) {
                    let _e324 = (_e182 & 4194304u);
                    let _e326 = select(2i, -2i, (_e324 == 0u));
                    phi_2277_ = _e326;
                    if ((_e182 & 8388608u) != 0u) {
                        phi_2277_ = -(_e326);
                    }
                    let _e331 = phi_2277_;
                    let _e332 = (_e178 + _e331);
                    let _e337 = textureLoad(KC, vec2<i32>((_e332 & 2047i), (_e332 >> bitcast<u32>(11i))), 0i);
                    let _e341 = abs((bitcast<f32>(_e337.z) - _e287));
                    phi_2287_ = _e341;
                    if (_e341 > 3.1415927f) {
                        phi_2287_ = (6.2831855f - _e341);
                    }
                    let _e345 = phi_2287_;
                    let _e350 = ((_e345 * select(0.5f, -0.5f, ((_e324 != 0u) == _e306))) + _e287);
                    let _e354 = vec2<f32>(sin(_e350), -(cos(_e350)));
                    let _e355 = (_e122 * _e354);
                    let _e365 = cos((_e345 * 0.5f));
                    let _e366 = (_e183 == 335544320u);
                    phi_1707_ = _e366;
                    if !(_e366) {
                        phi_1707_ = ((_e183 == 268435456u) && (_e365 >= 0.25f));
                    }
                    let _e372 = phi_1707_;
                    if _e372 {
                        phi_2294_ = (_e320 * (1f / max(_e365, select(0.25f, 1f, ((_e182 & 33554432u) != 0u)))));
                    } else {
                        phi_2294_ = ((_e320 * _e365) + (((abs(_e355.x) + abs(_e355.y)) * (1f / dot(_e355, _e355))) * 0.5f));
                    }
                    let _e383 = phi_2294_;
                    phi_2314_ = _e322;
                    if ((_e182 & 2097152u) != 0u) {
                        if (_e321 <= ((_e383 * _e365) + (_e316 * 0.125f))) {
                            phi_2315_ = (_e354 * (_e321 * (1f / _e365)));
                        } else {
                            let _e393 = (_e354 * _e383);
                            phi_2315_ = (vec2<f32>(dot(_e322, _e322), dot(_e393, _e393)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e322, _e393)));
                        }
                        let _e401 = phi_2315_;
                        phi_2314_ = _e401;
                    }
                    let _e403 = phi_2314_;
                    phi_2313_ = _e403;
                }
                let _e405 = phi_2313_;
                if (_e89 != 0i) {
                    phi_2364_ = u32();
                    phi_2335_ = vec2<f32>();
                    phi_2334_ = false;
                    break;
                }
                phi_2331_ = (_e122 * (_e405 * _e314));
                phi_2316_ = _e293;
            } else {
                if (((_e182 & 2147483648u) != 0u) && (_e89 != 1i)) {
                    phi_2364_ = u32();
                    phi_2335_ = vec2<f32>();
                    phi_2334_ = false;
                    break;
                }
                phi_2331_ = vec2<f32>(0f, 0f);
                phi_2316_ = select(_e293, _e107, vec2((_e89 == 2i)));
            }
            let _e417 = phi_2331_;
            let _e419 = phi_2316_;
            let _e426 = PB.e2_[(_e111 + 2u)];
            phi_2364_ = _e426.x;
            phi_2335_ = (((_e122 * _e419) + _e417) + bitcast<vec2<f32>>(_e126.xy));
            phi_2334_ = true;
            break;
        }
    }
    let _e429 = phi_2364_;
    let _e431 = phi_2335_;
    let _e433 = phi_2334_;
    let _e436 = local;
    let _e438 = DD.e2_[_e436];
    let _e440 = (_e438.x & 15u);
    if Ch {
        let _e441 = (_e440 == 0u);
        if _e441 {
            phi_2393_ = _e438.y;
        } else {
            phi_2393_ = _e438.x;
        }
        let _e444 = phi_2393_;
        let _e446 = (_e444 >> bitcast<u32>(16i));
        let _e448 = l.f6_;
        if (_e446 == 0u) {
            phi_2394_ = 0f;
        } else {
            phi_2394_ = unpack2x16float(((_e446 + 1023u) * _e448)).x;
        }
        let _e455 = phi_2394_;
        phi_2395_ = _e455;
        if _e441 {
            phi_2395_ = -(_e455);
        }
        let _e458 = phi_2395_;
        W1_[0u] = _e458;
    }
    if Eh {
        g2_ = f32(((_e438.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e440 == 1u) {
        let _e466 = unpack4x8unorm(_e438.y);
        if Eh {
            phi_2434_ = _e466;
        } else {
            let _e469 = (_e466.xyz * _e466.w);
            let _e475 = vec4<f32>(_e469.x, _e466.y, _e466.z, _e466.w);
            let _e481 = vec4<f32>(_e475.x, _e469.y, _e475.z, _e475.w);
            phi_2434_ = vec4<f32>(_e481.x, _e481.y, _e469.z, _e481.w);
        }
        let _e489 = phi_2434_;
        V1_ = _e489;
    } else {
        if (Ch && (_e440 == 0u)) {
            let _e493 = (_e438.x >> bitcast<u32>(16i));
            let _e495 = l.f6_;
            if (_e493 == 0u) {
                phi_2433_ = 0f;
            } else {
                phi_2433_ = unpack2x16float(((_e493 + 1023u) * _e495)).x;
            }
            let _e502 = phi_2433_;
            W1_[1u] = _e502;
        } else {
            let _e505 = local_1;
            let _e506 = (_e505 * 8u);
            let _e509 = QB.e2_[_e506];
            let _e520 = QB.e2_[(_e506 + 1u)];
            let _e529 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e438.y));
            let _e531 = ((mat2x2<f32>(vec2<f32>(_e509.x, _e509.y), vec2<f32>(_e509.z, _e509.w)) * _e431) + _e520.xy);
            if (_e520.z > 0.9f) {
                phi_2431_ = vec4<f32>(_e529.x, _e529.y, 2f, _e529.w);
            } else {
                phi_2431_ = vec4<f32>(_e529.x, _e529.y, _e520.w, _e529.w);
            }
            let _e546 = phi_2431_;
            if (f32(_e440) == 2f) {
                let _e553 = vec4<f32>(_e531.x, _e546.y, _e546.z, _e546.w);
                phi_2432_ = vec4<f32>(_e553.x, 0f, _e553.z, _e553.w);
            } else {
                let _e565 = vec4<f32>(_e546.x, _e546.y, -(_e546.z), _e546.w);
                let _e571 = vec4<f32>(_e531.x, _e565.y, _e565.z, _e565.w);
                phi_2432_ = vec4<f32>(_e571.x, _e531.y, _e571.z, _e571.w);
            }
            let _e579 = phi_2432_;
            V1_ = _e579;
            let _e581 = V1_[3u];
            V1_[3u] = -(_e581);
        }
    }
    phi_1108_ = Kh;
    if Kh {
        phi_1108_ = ((_e438.x & 2048u) != 0u);
    }
    let _e586 = phi_1108_;
    if _e586 {
        let _e588 = local_2;
        let _e589 = (_e588 * 8u);
        let _e593 = QB.e2_[(_e589 + 4u)];
        let _e604 = QB.e2_[(_e589 + 5u)];
        let _e607 = ((mat2x2<f32>(vec2<f32>(_e593.x, _e593.y), vec2<f32>(_e593.z, _e593.w)) * _e431) + _e604.xy);
        C2_ = vec3<f32>(_e607.x, _e607.y, (1f + _e604.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    if _e433 {
        let _e614 = l.Ff;
        let _e616 = l.Gf;
        let _e624 = vec4<f32>(((_e431.x * _e614) - 1f), ((_e431.y * _e616) - sign(_e616)), 0f, 1f);
        phi_2448_ = vec4<f32>(_e624.x, _e624.y, (1f - (f32(_e429) * 0.000061035156f)), _e624.w);
    } else {
        let _e634 = l.U2_;
        phi_2448_ = vec4(_e634);
    }
    let _e637 = phi_2448_;
    unnamed.gl_Position = _e637;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) UB: vec4<f32>, @location(1) VB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    UB_1 = UB;
    VB_1 = VB;
    main_1();
    let _e16 = W1_;
    let _e17 = g2_;
    let _e18 = V1_;
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
