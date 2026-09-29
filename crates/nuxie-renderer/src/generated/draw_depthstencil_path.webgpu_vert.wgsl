enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct jg {
    d2_: array<vec4<u32>>,
}

struct ig {
    d2_: array<vec4<u32>>,
}

struct Ne {
    d2_: array<vec2<u32>>,
}

struct DC {
    hc: f32,
    rd: f32,
    kf: f32,
    lf: f32,
    p6_: u32,
    Mg: u32,
    Ve: u32,
    We: u32,
    U7_: vec4<i32>,
    Ig: vec2<f32>,
    sd: vec2<f32>,
    c2_: u32,
    Ng: f32,
    d6_: u32,
    R2_: f32,
    td: f32,
    Qe: u32,
    B3_: f32,
    C3_: f32,
    ud: f32,
    Fg: u32,
}

struct Oe {
    d2_: array<vec4<f32>>,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(4) @interpolate(flat, either) member: vec2<f32>,
    @location(6) @interpolate(flat, either) member_1: f32,
    @location(0) member_2: vec4<f32>,
    @location(9) member_3: vec3<f32>,
}

@id(0) override fh: bool = true;
@id(2) override hh: bool = true;
@id(1) override gh: bool = true;
@id(8) override nh: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(7)
var MC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> FD: jg;
@group(0) @binding(2)
var<storage> QB: ig;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> VB_1: vec4<f32>;
var<private> WB_1: vec4<f32>;
@group(0) @binding(3)
var<storage> BD: Ne;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> V1_: vec2<f32>;
var<private> f2_: f32;
@group(0) @binding(4)
var<storage> RB: Oe;
var<private> f1_: vec4<f32>;
var<private> A2_: vec3<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var aa: sampler;

fn main_1() {
    var phi_2187_: f32;
    var phi_2159_: i32;
    var phi_1414_: bool;
    var phi_2172_: i32;
    var phi_2164_: vec4<u32>;
    var phi_2171_: i32;
    var phi_2163_: vec4<u32>;
    var phi_2170_: i32;
    var phi_2168_: vec4<u32>;
    var phi_2167_: u32;
    var phi_2174_: vec2<i32>;
    var phi_2175_: vec4<u32>;
    var phi_2179_: f32;
    var phi_2250_: f32;
    var phi_2193_: f32;
    var phi_2249_: f32;
    var phi_2197_: f32;
    var phi_2194_: f32;
    var phi_2191_: f32;
    var phi_2201_: f32;
    var phi_2247_: f32;
    var phi_2200_: f32;
    var phi_2256_: f32;
    var phi_2253_: f32;
    var phi_2310_: f32;
    var phi_2282_: i32;
    var phi_2292_: f32;
    var phi_1726_: bool;
    var phi_2299_: f32;
    var phi_2320_: vec2<f32>;
    var phi_2319_: vec2<f32>;
    var phi_2318_: vec2<f32>;
    var phi_2336_: vec2<f32>;
    var phi_2321_: vec2<f32>;
    var phi_2369_: u32;
    var phi_2340_: vec2<f32>;
    var phi_2339_: bool;
    var local: u32;
    var phi_2398_: u32;
    var phi_2399_: f32;
    var phi_2400_: f32;
    var local_1: u32;
    var phi_2402_: vec4<f32>;
    var phi_2401_: f32;
    var local_2: u32;
    var phi_1131_: bool;
    var local_3: u32;
    var phi_2419_: vec4<f32>;

    let _e79 = gl_InstanceIndex_1;
    let _e80 = VB_1;
    let _e81 = WB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e84 = i32(_e80.x);
            let _e87 = bitcast<i32>(_e80.w);
            let _e89 = (_e87 >> bitcast<u32>(2i));
            let _e90 = (_e87 & 3i);
            let _e92 = min(_e84, (_e89 - 1i));
            let _e94 = ((_e79 * _e89) + _e92);
            let _e99 = textureLoad(MC, vec2<i32>((_e94 & 2047i), (_e94 >> bitcast<u32>(11i))), 0i);
            let _e106 = FD.d2_[(max((_e99.w & 65535u), 1u) - 1u)];
            let _e108 = bitcast<vec2<f32>>(_e106.xy);
            let _e110 = (_e106.z & 65535u);
            let _e112 = (_e110 * 4u);
            let _e115 = QB.d2_[_e112];
            let _e116 = bitcast<vec4<f32>>(_e115);
            let _e123 = mat2x2<f32>(vec2<f32>(_e116.x, _e116.y), vec2<f32>(_e116.z, _e116.w));
            let _e127 = QB.d2_[(_e112 + 1u)];
            let _e131 = bitcast<f32>(_e127.z);
            let _e133 = bitcast<f32>(_e127.w);
            let _e134 = (_e99.w & 8388608u);
            phi_2187_ = _e80.y;
            phi_2159_ = _e84;
            local = _e110;
            local_1 = _e110;
            local_2 = _e110;
            local_3 = _e110;
            if (_e134 != 0u) {
                phi_2187_ = _e81.y;
                phi_2159_ = i32(_e81.x);
            }
            let _e140 = phi_2187_;
            let _e142 = phi_2159_;
            phi_2170_ = _e94;
            phi_2168_ = _e99;
            phi_2167_ = _e99.w;
            if (_e142 != _e92) {
                let _e145 = ((_e94 + _e142) - _e92);
                let _e150 = textureLoad(MC, vec2<i32>((_e145 & 2047i), (_e145 >> bitcast<u32>(11i))), 0i);
                if ((_e150.w & 8454143u) != (_e99.w & 8454143u)) {
                    let _e155 = (_e131 == 0f);
                    phi_1414_ = _e155;
                    if !(_e155) {
                        phi_1414_ = (_e108.x != 0f);
                    }
                    let _e160 = phi_1414_;
                    phi_2172_ = _e94;
                    phi_2164_ = _e99;
                    if _e160 {
                        let _e161 = bitcast<i32>(_e106.w);
                        let _e166 = textureLoad(MC, vec2<i32>((_e161 & 2047i), (_e161 >> bitcast<u32>(11i))), 0i);
                        phi_2172_ = _e161;
                        phi_2164_ = _e166;
                    }
                    let _e168 = phi_2172_;
                    let _e170 = phi_2164_;
                    phi_2171_ = _e168;
                    phi_2163_ = _e170;
                } else {
                    phi_2171_ = _e145;
                    phi_2163_ = _e150;
                }
                let _e172 = phi_2171_;
                let _e174 = phi_2163_;
                phi_2170_ = _e172;
                phi_2168_ = _e174;
                phi_2167_ = ((_e174.w & 4286578687u) | _e134);
            }
            let _e179 = phi_2170_;
            let _e181 = phi_2168_;
            let _e183 = phi_2167_;
            let _e184 = (_e183 & 469762048u);
            if ((_e184 == 67108864u) && (_e90 == 0i)) {
                let _e190 = f32((_e181.z & 65535u));
                let _e193 = f32((_e181.z >> bitcast<u32>(16i)));
                let _e199 = vec2<i32>(i32((-1f - _e190)), i32(((_e193 - _e190) + 1f)));
                phi_2174_ = _e199;
                if ((_e183 & 8388608u) != 0u) {
                    phi_2174_ = -(_e199);
                }
                let _e204 = phi_2174_;
                let _e206 = (_e179 + _e204.x);
                let _e211 = textureLoad(MC, vec2<i32>((_e206 & 2047i), (_e206 >> bitcast<u32>(11i))), 0i);
                let _e213 = (_e179 + _e204.y);
                let _e218 = textureLoad(MC, vec2<i32>((_e213 & 2047i), (_e213 >> bitcast<u32>(11i))), 0i);
                phi_2175_ = _e218;
                if ((_e218.w & 8454143u) != (_e211.w & 8454143u)) {
                    let _e224 = bitcast<i32>(_e106.w);
                    let _e229 = textureLoad(MC, vec2<i32>((_e224 & 2047i), (_e224 >> bitcast<u32>(11i))), 0i);
                    phi_2175_ = _e229;
                }
                let _e231 = phi_2175_;
                let _e233 = bitcast<f32>(_e211.z);
                let _e235 = bitcast<f32>(_e231.z);
                let _e236 = (_e235 - _e233);
                phi_2179_ = _e236;
                if (abs(_e236) > 3.1415927f) {
                    phi_2179_ = (_e236 - (6.2831855f * sign(_e236)));
                }
                let _e243 = phi_2179_;
                let _e244 = (_e193 + -2f);
                let _e250 = clamp(round(((abs(_e243) * 0.31830987f) * _e244)), 1f, (_e193 + -3f));
                let _e251 = (_e244 - _e250);
                if (_e190 <= _e251) {
                    phi_2250_ = _e140;
                    if (_e190 == _e251) {
                        phi_2250_ = -(_e140);
                    }
                    let _e260 = phi_2250_;
                    phi_2249_ = _e260;
                    phi_2197_ = -(((3.1415927f * sign(_e243)) - _e243));
                    phi_2194_ = _e251;
                    phi_2191_ = _e190;
                } else {
                    let _e262 = (_e190 == (_e251 + 1f));
                    if _e262 {
                        phi_2193_ = 0f;
                    } else {
                        phi_2193_ = (_e190 - (_e251 + 2f));
                    }
                    let _e266 = phi_2193_;
                    phi_2249_ = select(_e140, 0f, _e262);
                    phi_2197_ = _e243;
                    phi_2194_ = select(_e250, 0f, _e262);
                    phi_2191_ = _e266;
                }
                let _e270 = phi_2249_;
                let _e272 = phi_2197_;
                let _e274 = phi_2194_;
                let _e276 = phi_2191_;
                if (_e276 == _e274) {
                    phi_2201_ = _e235;
                } else {
                    phi_2201_ = (_e233 + (_e272 * (_e276 / _e274)));
                }
                let _e282 = phi_2201_;
                phi_2247_ = _e270;
                phi_2200_ = _e282;
            } else {
                phi_2247_ = _e140;
                phi_2200_ = bitcast<f32>(_e181.z);
            }
            let _e286 = phi_2247_;
            let _e288 = phi_2200_;
            let _e292 = vec2<f32>(sin(_e288), -(cos(_e288)));
            let _e294 = bitcast<vec2<f32>>(_e181.xy);
            phi_2256_ = _e133;
            if (_e133 != 0f) {
                phi_2256_ = max(_e133, (1f / length((_e123 * _e292))));
            }
            let _e301 = phi_2256_;
            if (_e131 != 0f) {
                let _e305 = (_e286 * sign(determinant(_e123)));
                let _e307 = ((_e183 & 1048576u) != 0u);
                phi_2253_ = _e305;
                if _e307 {
                    phi_2253_ = min(_e305, 0f);
                }
                let _e310 = phi_2253_;
                phi_2310_ = _e310;
                if ((_e183 & 524288u) != 0u) {
                    phi_2310_ = max(_e310, 0f);
                }
                let _e315 = phi_2310_;
                let _e317 = select(0f, _e301, (_e301 != 0f));
                let _e321 = select(_e131, _e317, ((_e317 > _e131) && (_e301 == 0f)));
                let _e322 = (_e321 + _e317);
                let _e323 = (_e292 * _e322);
                phi_2318_ = _e323;
                if (_e184 > 134217728u) {
                    let _e325 = (_e183 & 4194304u);
                    let _e327 = select(2i, -2i, (_e325 == 0u));
                    phi_2282_ = _e327;
                    if ((_e183 & 8388608u) != 0u) {
                        phi_2282_ = -(_e327);
                    }
                    let _e332 = phi_2282_;
                    let _e333 = (_e179 + _e332);
                    let _e338 = textureLoad(MC, vec2<i32>((_e333 & 2047i), (_e333 >> bitcast<u32>(11i))), 0i);
                    let _e342 = abs((bitcast<f32>(_e338.z) - _e288));
                    phi_2292_ = _e342;
                    if (_e342 > 3.1415927f) {
                        phi_2292_ = (6.2831855f - _e342);
                    }
                    let _e346 = phi_2292_;
                    let _e351 = ((_e346 * select(0.5f, -0.5f, ((_e325 != 0u) == _e307))) + _e288);
                    let _e355 = vec2<f32>(sin(_e351), -(cos(_e351)));
                    let _e356 = (_e123 * _e355);
                    let _e366 = cos((_e346 * 0.5f));
                    let _e367 = (_e184 == 335544320u);
                    phi_1726_ = _e367;
                    if !(_e367) {
                        phi_1726_ = ((_e184 == 268435456u) && (_e366 >= 0.25f));
                    }
                    let _e373 = phi_1726_;
                    if _e373 {
                        phi_2299_ = (_e321 * (1f / max(_e366, select(0.25f, 1f, ((_e183 & 33554432u) != 0u)))));
                    } else {
                        phi_2299_ = ((_e321 * _e366) + (((abs(_e356.x) + abs(_e356.y)) * (1f / dot(_e356, _e356))) * 0.5f));
                    }
                    let _e384 = phi_2299_;
                    phi_2319_ = _e323;
                    if ((_e183 & 2097152u) != 0u) {
                        if (_e322 <= ((_e384 * _e366) + (_e317 * 0.125f))) {
                            phi_2320_ = (_e355 * (_e322 * (1f / _e366)));
                        } else {
                            let _e394 = (_e355 * _e384);
                            phi_2320_ = (vec2<f32>(dot(_e323, _e323), dot(_e394, _e394)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e323, _e394)));
                        }
                        let _e402 = phi_2320_;
                        phi_2319_ = _e402;
                    }
                    let _e404 = phi_2319_;
                    phi_2318_ = _e404;
                }
                let _e406 = phi_2318_;
                if (_e90 != 0i) {
                    phi_2369_ = u32();
                    phi_2340_ = vec2<f32>();
                    phi_2339_ = false;
                    break;
                }
                phi_2336_ = (_e123 * (_e406 * _e315));
                phi_2321_ = _e294;
            } else {
                if (((_e183 & 2147483648u) != 0u) && (_e90 != 1i)) {
                    phi_2369_ = u32();
                    phi_2340_ = vec2<f32>();
                    phi_2339_ = false;
                    break;
                }
                phi_2336_ = vec2<f32>(0f, 0f);
                phi_2321_ = select(_e294, _e108, vec2((_e90 == 2i)));
            }
            let _e418 = phi_2336_;
            let _e420 = phi_2321_;
            let _e427 = QB.d2_[(_e112 + 2u)];
            phi_2369_ = _e427.x;
            phi_2340_ = (((_e123 * _e420) + _e418) + bitcast<vec2<f32>>(_e127.xy));
            phi_2339_ = true;
            break;
        }
    }
    let _e430 = phi_2369_;
    let _e432 = phi_2340_;
    let _e434 = phi_2339_;
    let _e437 = local;
    let _e439 = BD.d2_[_e437];
    let _e441 = (_e439.x & 15u);
    if fh {
        let _e442 = (_e441 == 0u);
        if _e442 {
            phi_2398_ = _e439.y;
        } else {
            phi_2398_ = _e439.x;
        }
        let _e445 = phi_2398_;
        let _e447 = (_e445 >> bitcast<u32>(16i));
        let _e449 = m.d6_;
        if (_e447 == 0u) {
            phi_2399_ = 0f;
        } else {
            phi_2399_ = unpack2x16float(((_e447 + 1023u) * _e449)).x;
        }
        let _e456 = phi_2399_;
        phi_2400_ = _e456;
        if _e442 {
            phi_2400_ = -(_e456);
        }
        let _e459 = phi_2400_;
        V1_[0u] = _e459;
    }
    if hh {
        f2_ = f32(((_e439.x >> bitcast<u32>(4i)) & 15u));
    }
    if gh {
        let _e466 = local_1;
        let _e467 = (_e466 * 8u);
        let _e471 = RB.d2_[(_e467 + 2u)];
        let _e482 = RB.d2_[(_e467 + 3u)];
        if any((_e471 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e487 = ((mat2x2<f32>(vec2<f32>(_e471.x, _e471.y), vec2<f32>(_e471.z, _e471.w)) * _e432) + _e482.xy);
            unnamed.gl_ClipDistance[0i] = (_e487.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e487.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e487.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e487.y);
        } else {
            let _e503 = (_e482.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e503;
            unnamed.gl_ClipDistance[2i] = _e503;
            unnamed.gl_ClipDistance[1i] = _e503;
            unnamed.gl_ClipDistance[0i] = _e503;
        }
    }
    if (_e441 == 1u) {
        let _e514 = unpack4x8unorm(_e439.y);
        if hh {
            phi_2402_ = _e514;
        } else {
            let _e517 = (_e514.xyz * _e514.w);
            let _e523 = vec4<f32>(_e517.x, _e514.y, _e514.z, _e514.w);
            let _e529 = vec4<f32>(_e523.x, _e517.y, _e523.z, _e523.w);
            phi_2402_ = vec4<f32>(_e529.x, _e529.y, _e517.z, _e529.w);
        }
        let _e537 = phi_2402_;
        f1_ = _e537;
    } else {
        if (fh && (_e441 == 0u)) {
            let _e541 = (_e439.x >> bitcast<u32>(16i));
            let _e543 = m.d6_;
            if (_e541 == 0u) {
                phi_2401_ = 0f;
            } else {
                phi_2401_ = unpack2x16float(((_e541 + 1023u) * _e543)).x;
            }
            let _e550 = phi_2401_;
            V1_[1u] = _e550;
        } else {
            let _e553 = local_2;
            let _e554 = (_e553 * 8u);
            let _e557 = RB.d2_[_e554];
            let _e568 = RB.d2_[(_e554 + 1u)];
            let _e571 = ((mat2x2<f32>(vec2<f32>(_e557.x, _e557.y), vec2<f32>(_e557.z, _e557.w)) * _e432) + _e568.xy);
            let _e572 = (_e441 == 2u);
            if (_e572 || (_e441 == 3u)) {
                f1_[3u] = -(bitcast<f32>(_e439.y));
                if (_e568.z > 0.9f) {
                    f1_[2u] = 2f;
                } else {
                    f1_[2u] = _e568.w;
                }
                if _e572 {
                    f1_[1u] = 0f;
                    f1_[0u] = _e571.x;
                } else {
                    let _e588 = f1_[2u];
                    f1_[2u] = -(_e588);
                    f1_[0u] = _e571.x;
                    f1_[1u] = _e571.y;
                }
            }
        }
    }
    phi_1131_ = nh;
    if nh {
        phi_1131_ = ((_e439.x & 2048u) != 0u);
    }
    let _e597 = phi_1131_;
    if _e597 {
        let _e599 = local_3;
        let _e600 = (_e599 * 8u);
        let _e604 = RB.d2_[(_e600 + 4u)];
        let _e615 = RB.d2_[(_e600 + 5u)];
        let _e618 = ((mat2x2<f32>(vec2<f32>(_e604.x, _e604.y), vec2<f32>(_e604.z, _e604.w)) * _e432) + _e615.xy);
        A2_ = vec3<f32>(_e618.x, _e618.y, (1f + _e615.z));
    } else {
        A2_ = vec3<f32>(0f, 0f, 0f);
    }
    if _e434 {
        let _e625 = m.kf;
        let _e627 = m.lf;
        let _e635 = vec4<f32>(((_e432.x * _e625) - 1f), ((_e432.y * _e627) - sign(_e627)), 0f, 1f);
        phi_2419_ = vec4<f32>(_e635.x, _e635.y, (1f - (f32(_e430) * 0.000061035156f)), _e635.w);
    } else {
        let _e645 = m.R2_;
        phi_2419_ = vec4(_e645);
    }
    let _e648 = phi_2419_;
    unnamed.gl_Position = _e648;
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
    let _e19 = V1_;
    let _e20 = f2_;
    let _e21 = f1_;
    let _e22 = A2_;
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
