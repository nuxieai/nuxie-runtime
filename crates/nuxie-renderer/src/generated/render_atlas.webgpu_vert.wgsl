struct hi {
    v2_: array<vec4<u32>>,
}

struct gi {
    v2_: array<vec4<u32>>,
}

struct VB {
    vd: f32,
    Ce: f32,
    Gg: f32,
    Hg: f32,
    L6_: u32,
    xa: u32,
    sg: u32,
    tg: u32,
    C8_: vec4<i32>,
    Bi: vec2<f32>,
    De: vec2<f32>,
    r2_: u32,
    Fi: f32,
    p6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    E3_: f32,
    F3_: f32,
    Fe: f32,
    yi: u32,
    wa: u32,
    cd: f32,
    g7_: f32,
    Db: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct jg {
    v2_: array<vec2<u32>>,
}

struct kg {
    v2_: array<vec4<f32>>,
}

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(5)
var<storage> CD: hi;
@group(0) @binding(2)
var<storage> KB: gi;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> XB_1: vec4<f32>;
var<private> YB_1: vec4<f32>;
var<private> S: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(0) @binding(3)
var<storage> WC: jg;
@group(0) @binding(4)
var<storage> JB: kg;
@group(3) @binding(9)
var Va: sampler;

fn main_1() {
    var phi_2267_: f32;
    var phi_2205_: f32;
    var phi_2177_: i32;
    var phi_1329_: bool;
    var phi_2190_: i32;
    var phi_2182_: vec4<u32>;
    var phi_2189_: i32;
    var phi_2181_: vec4<u32>;
    var phi_2188_: i32;
    var phi_2186_: vec4<u32>;
    var phi_2185_: u32;
    var phi_2192_: vec2<i32>;
    var phi_2193_: vec4<u32>;
    var phi_2197_: f32;
    var phi_2277_: f32;
    var phi_2211_: f32;
    var phi_2276_: f32;
    var phi_2219_: f32;
    var phi_2212_: f32;
    var phi_2209_: f32;
    var phi_2223_: f32;
    var phi_2298_: f32;
    var phi_2289_: f32;
    var phi_2274_: f32;
    var phi_2222_: f32;
    var phi_2272_: f32;
    var phi_2355_: f32;
    var phi_2366_: f32;
    var phi_2358_: f32;
    var phi_2419_: f32;
    var phi_2393_: f32;
    var phi_1644_: bool;
    var phi_2398_: f32;
    var phi_2410_: vec2<f32>;
    var phi_2409_: vec2<f32>;
    var phi_2427_: vec4<f32>;
    var phi_2440_: vec2<f32>;
    var phi_2426_: vec4<f32>;
    var phi_2474_: vec4<f32>;
    var phi_2308_: f32;
    var phi_2307_: f32;
    var phi_2309_: f32;
    var phi_2313_: f32;
    var phi_2335_: f32;
    var phi_2333_: f32;
    var phi_2351_: vec4<f32>;
    var phi_2471_: vec2<f32>;
    var phi_2350_: vec4<f32>;
    var phi_2477_: vec4<f32>;
    var phi_2478_: bool;
    var phi_2472_: vec4<f32>;
    var phi_2466_: vec2<f32>;
    var phi_2442_: vec2<f32>;
    var phi_2510_: vec4<f32>;

    let _e71 = gl_InstanceIndex_1;
    let _e72 = XB_1;
    let _e73 = YB_1;
    let _e75 = i32(_e72.x);
    let _e79 = bitcast<i32>(_e72.w);
    let _e81 = (_e79 >> bitcast<u32>(2i));
    let _e82 = (_e79 & 3i);
    let _e84 = min(_e75, (_e81 - 1i));
    let _e86 = ((_e71 * _e81) + _e84);
    let _e91 = textureLoad(UB, vec2<i32>((_e86 & 2047i), (_e86 >> bitcast<u32>(11i))), 0i);
    let _e98 = CD.v2_[(max((_e91.w & 65535u), 1u) - 1u)];
    let _e100 = bitcast<vec2<f32>>(_e98.xy);
    let _e104 = ((_e98.z & 65535u) * 4u);
    let _e107 = KB.v2_[_e104];
    let _e108 = bitcast<vec4<f32>>(_e107);
    let _e115 = mat2x2<f32>(vec2<f32>(_e108.x, _e108.y), vec2<f32>(_e108.z, _e108.w));
    let _e119 = KB.v2_[(_e104 + 1u)];
    let _e123 = bitcast<f32>(_e119.z);
    let _e125 = bitcast<f32>(_e119.w);
    let _e126 = (_e91.w & 8388608u);
    phi_2267_ = _e72.z;
    phi_2205_ = _e72.y;
    phi_2177_ = _e75;
    if (_e126 != 0u) {
        phi_2267_ = _e73.z;
        phi_2205_ = _e73.y;
        phi_2177_ = i32(_e73.x);
    }
    let _e133 = phi_2267_;
    let _e135 = phi_2205_;
    let _e137 = phi_2177_;
    phi_2188_ = _e86;
    phi_2186_ = _e91;
    phi_2185_ = _e91.w;
    if (_e137 != _e84) {
        let _e140 = ((_e86 + _e137) - _e84);
        let _e145 = textureLoad(UB, vec2<i32>((_e140 & 2047i), (_e140 >> bitcast<u32>(11i))), 0i);
        if ((_e145.w & 8454143u) != (_e91.w & 8454143u)) {
            let _e150 = (_e123 == 0f);
            phi_1329_ = _e150;
            if !(_e150) {
                phi_1329_ = (_e100.x != 0f);
            }
            let _e155 = phi_1329_;
            phi_2190_ = _e86;
            phi_2182_ = _e91;
            if _e155 {
                let _e156 = bitcast<i32>(_e98.w);
                let _e161 = textureLoad(UB, vec2<i32>((_e156 & 2047i), (_e156 >> bitcast<u32>(11i))), 0i);
                phi_2190_ = _e156;
                phi_2182_ = _e161;
            }
            let _e163 = phi_2190_;
            let _e165 = phi_2182_;
            phi_2189_ = _e163;
            phi_2181_ = _e165;
        } else {
            phi_2189_ = _e140;
            phi_2181_ = _e145;
        }
        let _e167 = phi_2189_;
        let _e169 = phi_2181_;
        phi_2188_ = _e167;
        phi_2186_ = _e169;
        phi_2185_ = ((_e169.w & 4286578687u) | _e126);
    }
    let _e174 = phi_2188_;
    let _e176 = phi_2186_;
    let _e178 = phi_2185_;
    let _e179 = (_e178 & 469762048u);
    let _e182 = ((_e179 == 67108864u) && (_e82 == 0i));
    if _e182 {
        let _e185 = f32((_e176.z & 65535u));
        let _e188 = f32((_e176.z >> bitcast<u32>(16i)));
        let _e194 = vec2<i32>(i32((-1f - _e185)), i32(((_e188 - _e185) + 1f)));
        phi_2192_ = _e194;
        if ((_e178 & 8388608u) != 0u) {
            phi_2192_ = -(_e194);
        }
        let _e199 = phi_2192_;
        let _e201 = (_e174 + _e199.x);
        let _e206 = textureLoad(UB, vec2<i32>((_e201 & 2047i), (_e201 >> bitcast<u32>(11i))), 0i);
        let _e208 = (_e174 + _e199.y);
        let _e213 = textureLoad(UB, vec2<i32>((_e208 & 2047i), (_e208 >> bitcast<u32>(11i))), 0i);
        phi_2193_ = _e213;
        if ((_e213.w & 8454143u) != (_e206.w & 8454143u)) {
            let _e219 = bitcast<i32>(_e98.w);
            let _e224 = textureLoad(UB, vec2<i32>((_e219 & 2047i), (_e219 >> bitcast<u32>(11i))), 0i);
            phi_2193_ = _e224;
        }
        let _e226 = phi_2193_;
        let _e229 = (f32(_e206.z) * 0.0000000014629181f);
        let _e232 = (f32(_e226.z) * 0.0000000014629181f);
        let _e233 = (_e232 - _e229);
        phi_2197_ = _e233;
        if (abs(_e233) > 3.1415927f) {
            phi_2197_ = (_e233 - (6.2831855f * sign(_e233)));
        }
        let _e240 = phi_2197_;
        let _e241 = (_e188 + -2f);
        let _e247 = clamp(round(((abs(_e240) * 0.31830987f) * _e241)), 1f, (_e188 + -3f));
        let _e248 = (_e241 - _e247);
        if (_e185 <= _e248) {
            phi_2277_ = _e135;
            if (_e185 == _e248) {
                phi_2277_ = -(_e135);
            }
            let _e257 = phi_2277_;
            phi_2276_ = _e257;
            phi_2219_ = -(((3.1415927f * sign(_e240)) - _e240));
            phi_2212_ = _e248;
            phi_2209_ = _e185;
        } else {
            let _e259 = (_e185 == (_e248 + 1f));
            if _e259 {
                phi_2211_ = 0f;
            } else {
                phi_2211_ = (_e185 - (_e248 + 2f));
            }
            let _e263 = phi_2211_;
            phi_2276_ = select(_e135, 0f, _e259);
            phi_2219_ = _e240;
            phi_2212_ = select(_e247, 0f, _e259);
            phi_2209_ = _e263;
        }
        let _e267 = phi_2276_;
        let _e269 = phi_2219_;
        let _e271 = phi_2212_;
        let _e273 = phi_2209_;
        if (_e273 == _e271) {
            phi_2223_ = _e232;
        } else {
            phi_2223_ = (_e229 + (_e269 * (_e273 / _e271)));
        }
        let _e279 = phi_2223_;
        phi_2298_ = _e229;
        phi_2289_ = _e269;
        phi_2274_ = _e267;
        phi_2222_ = _e279;
    } else {
        phi_2298_ = f32();
        phi_2289_ = f32();
        phi_2274_ = _e135;
        phi_2222_ = (f32(_e176.z) * 0.0000000014629181f);
    }
    let _e284 = phi_2298_;
    let _e286 = phi_2289_;
    let _e288 = phi_2274_;
    let _e290 = phi_2222_;
    let _e294 = vec2<f32>(sin(_e290), -(cos(_e290)));
    let _e296 = bitcast<vec2<f32>>(_e176.xy);
    phi_2272_ = _e125;
    if (_e125 != 0f) {
        phi_2272_ = max(_e125, (1f / length((_e115 * _e294))));
    }
    let _e303 = phi_2272_;
    if (_e123 != 0f) {
        let _e307 = (_e288 * sign(determinant(_e115)));
        let _e309 = ((_e178 & 1048576u) != 0u);
        phi_2355_ = _e307;
        if _e309 {
            phi_2355_ = min(_e307, 0f);
        }
        let _e312 = phi_2355_;
        phi_2366_ = _e312;
        if ((_e178 & 524288u) != 0u) {
            phi_2366_ = max(_e312, 0f);
        }
        let _e317 = phi_2366_;
        let _e318 = (_e303 != 0f);
        if _e318 {
            phi_2358_ = _e303;
        } else {
            let _e319 = (_e115 * _e294);
            phi_2358_ = (((abs(_e319.x) + abs(_e319.y)) * (1f / dot(_e319, _e319))) * 0.5f);
        }
        let _e330 = phi_2358_;
        let _e333 = ((_e330 > _e123) && (_e303 == 0f));
        phi_2419_ = 1f;
        if _e333 {
            phi_2419_ = (_e123 / _e330);
        }
        let _e336 = phi_2419_;
        let _e337 = select(_e123, _e330, _e333);
        let _e338 = (_e337 + _e330);
        let _e339 = (_e294 * _e338);
        let _e340 = (_e317 * _e338);
        let _e347 = (((vec2<f32>(_e340, -(_e340)) + vec2(_e337)) * (0.5f / _e330)) + vec2<f32>(0.5f, 0.5f));
        let _e350 = vec4<f32>(_e347.x, _e347.y, 0f, 0f);
        phi_2440_ = _e339;
        phi_2426_ = _e350;
        if (_e179 > 134217728u) {
            let _e356 = f32((_e176.z & 65535u));
            let _e357 = (_e356 * 0.000015259022f);
            let _e361 = sqrt(max((1f - (_e357 * _e357)), 0f));
            phi_2393_ = _e361;
            if (((_e178 & 4194304u) != 0u) == _e309) {
                phi_2393_ = -(_e361);
            }
            let _e365 = phi_2393_;
            let _e370 = (mat2x2<f32>(vec2<f32>(_e357, _e365), vec2<f32>(-(_e365), _e357)) * _e294);
            let _e371 = (_e115 * _e370);
            let _e379 = ((abs(_e371.x) + abs(_e371.y)) * (1f / dot(_e371, _e371)));
            let _e380 = (_e179 == 335544320u);
            phi_1644_ = _e380;
            if !(_e380) {
                phi_1644_ = ((_e179 == 268435456u) && (_e357 >= 0.25f));
            }
            let _e386 = phi_1644_;
            if _e386 {
                phi_2398_ = (_e337 * (1f / max(_e357, select(0.25f, 1f, ((_e178 & 33554432u) != 0u)))));
            } else {
                phi_2398_ = ((_e337 * _e357) + (_e379 * 0.5f));
            }
            let _e397 = phi_2398_;
            let _e399 = (_e397 + (_e379 * 0.5f));
            phi_2409_ = _e339;
            if ((_e178 & 2097152u) != 0u) {
                if (_e338 <= ((_e399 * _e357) + (_e330 * 0.125f))) {
                    phi_2410_ = (_e370 * (_e338 * (65535f / _e356)));
                } else {
                    let _e409 = (_e370 * _e399);
                    phi_2410_ = (vec2<f32>(dot(_e339, _e339), dot(_e409, _e409)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e339, _e409)));
                }
                let _e417 = phi_2410_;
                phi_2409_ = _e417;
            }
            let _e419 = phi_2409_;
            let _e424 = ((_e399 - dot((_e419 * abs(_e317)), _e370)) / _e379);
            if _e309 {
                phi_2427_ = vec4<f32>(_e350.x, _e424, _e350.z, _e350.w);
            } else {
                phi_2427_ = vec4<f32>(_e424, _e350.y, _e350.z, _e350.w);
            }
            let _e436 = phi_2427_;
            phi_2440_ = _e419;
            phi_2426_ = _e436;
        }
        let _e438 = phi_2440_;
        let _e440 = phi_2426_;
        let _e442 = (_e440.xy * _e336);
        let _e448 = vec4<f32>(_e442.x, _e440.y, _e440.z, _e440.w);
        let _e455 = vec4<f32>(_e448.x, max(_e442.y, 0.0001f), _e448.z, _e448.w);
        phi_2474_ = _e455;
        if _e318 {
            phi_2474_ = vec4<f32>((-2f - _e442.x), _e455.y, _e455.z, _e455.w);
        }
        let _e463 = phi_2474_;
        phi_2478_ = (_e82 != 0i);
        phi_2472_ = _e463;
        phi_2466_ = (_e115 * (_e438 * _e317));
        phi_2442_ = _e296;
    } else {
        let _e467 = vec4<f32>(_e133, -1f, 0f, 0f);
        if (_e303 != 0f) {
            let _e473 = vec4<f32>(_e467.x, -2f, _e467.z, _e467.w);
            let _e478 = vec4<f32>(_e473.x, _e473.y, 1000000f, _e473.w);
            phi_2351_ = vec4<f32>(_e478.x, _e478.y, _e478.z, _e133);
            if _e182 {
                phi_2308_ = _e286;
                phi_2307_ = _e284;
                if (_e286 < 0f) {
                    phi_2308_ = -(_e286);
                    phi_2307_ = (_e284 + _e286);
                }
                let _e488 = phi_2308_;
                let _e490 = phi_2307_;
                let _e492 = ((_e290 - _e490) + 1.5707964f);
                let _e498 = clamp(((_e492 - (floor((_e492 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e488);
                phi_2309_ = _e498;
                if (_e498 > (_e488 * 0.5f)) {
                    phi_2309_ = (_e488 - _e498);
                }
                let _e503 = phi_2309_;
                let _e510 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e503), cos(_e503)) * abs(_e288))) * 0.5f);
                if (abs((_e488 - 1.5707964f)) < 0.001f) {
                    phi_2335_ = 0f;
                    phi_2333_ = 0f;
                } else {
                    let _e514 = tan(_e488);
                    let _e519 = (sign((1.5707964f - _e488)) / max(abs(_e514), 0.000001f));
                    if (_e519 >= 0f) {
                        phi_2313_ = (_e510.y - ((1f - _e510.x) * _e514));
                    } else {
                        phi_2313_ = (_e510.y + (_e510.x * _e514));
                    }
                    let _e531 = phi_2313_;
                    phi_2335_ = _e531;
                    phi_2333_ = _e519;
                }
                let _e533 = phi_2335_;
                let _e535 = phi_2333_;
                phi_2351_ = vec4<f32>((max(_e510.x, 0f) + 0.25f), (-2f - _e510.y), _e535, _e533);
            }
            let _e543 = phi_2351_;
            phi_2471_ = (_e115 * (_e294 * (_e288 * _e303)));
            phi_2350_ = _e543;
        } else {
            phi_2471_ = (sign(((_e294 * _e288) * _naga_inverse_2x2_f32(_e115))) * 0.5f);
            phi_2350_ = _e467;
        }
        let _e553 = phi_2471_;
        let _e555 = phi_2350_;
        phi_2477_ = _e555;
        if (((_e178 & 8388608u) != 0u) != ((_e178 & 16777216u) != 0u)) {
            phi_2477_ = (_e555 * vec4<f32>(-1f, 1f, 1f, 1f));
        }
        let _e563 = phi_2477_;
        phi_2478_ = (((_e178 & 2147483648u) != 0u) && (_e82 != 1i));
        phi_2472_ = _e563;
        phi_2466_ = _e553;
        phi_2442_ = select(_e296, _e100, vec2((_e82 == 2i)));
    }
    let _e572 = phi_2478_;
    let _e574 = phi_2472_;
    let _e576 = phi_2466_;
    let _e578 = phi_2442_;
    let _e584 = j.yi;
    let _e587 = select(_e574.xy, vec2<f32>(1f, -1f), vec2((_e584 != 0u)));
    let _e593 = vec4<f32>(_e587.x, _e574.y, _e574.z, _e574.w);
    S = vec4<f32>(_e593.x, _e587.y, _e593.z, _e593.w);
    if !(_e572) {
        let _e604 = KB.v2_[(_e104 + 2u)];
        let _e606 = bitcast<vec3<f32>>(_e604.yzw);
        let _e610 = (((((_e115 * _e578) + _e576) + bitcast<vec2<f32>>(_e119.xy)) * _e606.x) + _e606.yz);
        let _e613 = j.De[0u];
        let _e616 = j.De[1u];
        phi_2510_ = vec4<f32>(((_e610.x * _e613) - 1f), ((_e610.y * _e616) - sign(_e616)), 0f, 1f);
    } else {
        let _e626 = j.h3_;
        phi_2510_ = vec4(_e626);
    }
    let _e629 = phi_2510_;
    unnamed.gl_Position = _e629;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) XB: vec4<f32>, @location(1) YB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    XB_1 = XB;
    YB_1 = YB;
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
