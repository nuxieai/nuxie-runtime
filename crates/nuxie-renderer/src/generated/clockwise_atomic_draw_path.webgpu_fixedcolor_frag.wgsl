struct ye {
    e2_: array<u32>,
}

struct BC {
    qc: f32,
    Ad: f32,
    Ef: f32,
    Ff: f32,
    q6_: u32,
    Nb: u32,
    qf: u32,
    rf: u32,
    V7_: vec4<i32>,
    bh: vec2<f32>,
    Bd: vec2<f32>,
    d2_: u32,
    fh: f32,
    e6_: u32,
    T2_: f32,
    Cd: f32,
    lf: u32,
    B3_: f32,
    C3_: f32,
    Dd: f32,
    Yg: u32,
}

struct h0Qd {
    e2_: array<u32>,
}

struct ye_1 {
    e2_: array<atomic<u32>>,
}

@id(7) override Ih: bool = true;
@id(8) override Jh: bool = true;
@id(3) override Eh: bool = true;
@id(1) override Ch: bool = true;
@id(0) override Bh: bool = true;

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ea: sampler;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var W5_: sampler;
@group(0) @binding(6)
var<storage, read_write> Q0_: ye_1;
@group(0) @binding(0)
var<uniform> n: BC;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> V1_1: vec4<f32>;
var<private> C2_1: vec3<f32>;
var<private> M_1: vec4<f32>;
var<private> p4_1: vec2<f32>;
var<private> g3_1: vec2<u32>;
var<private> M0_1: vec4<f32>;
var<private> W1_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Qd;
var<private> C1_: vec4<f32>;
var<private> C0_1: f32;
var<private> g2_1: f32;

fn main_1() {
    var phi_2089_: f32;
    var phi_2090_: f32;
    var phi_2101_: vec4<f32>;
    var phi_1242_: bool;
    var phi_2091_: f32;
    var phi_2102_: vec4<f32>;
    var phi_1379_: bool;
    var phi_2103_: f32;
    var phi_2120_: f32;
    var phi_2121_: f32;
    var phi_1561_: bool;
    var phi_2122_: f32;
    var phi_2123_: f32;
    var phi_2125_: f32;
    var phi_1014_: bool;
    var phi_2126_: f32;
    var local: bool;
    var phi_1718_: bool;
    var phi_1720_: bool;
    var phi_2156_: f32;
    var phi_2151_: u32;
    var phi_2148_: f32;
    var phi_2155_: f32;
    var phi_2150_: u32;
    var phi_2147_: f32;
    var phi_2152_: f32;
    var phi_2149_: u32;
    var phi_2146_: f32;
    var phi_2160_: f32;
    var phi_2162_: f32;
    var phi_2170_: f32;
    var phi_2172_: f32;
    var phi_2177_: vec4<f32>;
    var phi_2175_: f32;
    var phi_2179_: f32;
    var phi_2207_: vec3<f32>;

    let _e70 = gl_FragCoord_1;
    let _e74 = bitcast<vec2<u32>>(vec2<i32>(floor(_e70.xy)));
    let _e76 = n.q6_;
    let _e105 = bitcast<i32>((((((_e74.y >> bitcast<u32>(5u)) * (((_e76 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e74.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e74.x & 28u) << bitcast<u32>(5u)) + ((_e74.y & 28u) << bitcast<u32>(2i)))) + (((_e74.y & 3u) << bitcast<u32>(2i)) + (_e74.x & 3u))));
    let _e106 = V1_1;
    let _e107 = C2_1;
    if (_e106.w >= 0f) {
        phi_2101_ = vec4<f32>(_e106.x, _e106.y, _e106.z, _e106.w);
    } else {
        if (_e106.z > 0f) {
            phi_2089_ = _e106.x;
        } else {
            phi_2089_ = length(_e106.xy);
        }
        let _e117 = phi_2089_;
        let _e118 = clamp(_e117, 0f, 1f);
        let _e119 = abs(_e106.z);
        if (_e119 > 1f) {
            phi_2090_ = ((0.9980469f * _e118) + 0.0009765625f);
        } else {
            phi_2090_ = ((0.001953125f * _e118) + _e119);
        }
        let _e126 = phi_2090_;
        let _e128 = textureSampleLevel(ED, N9_, vec2<f32>(_e126, -(_e106.w)), 0f);
        phi_2101_ = vec4<f32>(_e128.x, _e128.y, _e128.z, _e128.w);
    }
    let _e142 = phi_2101_;
    phi_1242_ = Jh;
    if Jh {
        phi_1242_ = (_e107.z > 0f);
    }
    let _e146 = phi_1242_;
    phi_2102_ = _e142;
    if _e146 {
        let _e150 = textureSampleLevel(HC, W5_, _e107.xy, (_e107.z - 1f));
        if (_e150.w != 0f) {
            phi_2091_ = (1f / _e150.w);
        } else {
            phi_2091_ = 0f;
        }
        let _e156 = phi_2091_;
        let _e157 = (_e150.xyz * _e156);
        phi_2102_ = (_e142 * vec4<f32>(_e157.x, _e157.y, _e157.z, _e150.w));
    }
    let _e164 = phi_2102_;
    let _e165 = M_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e168 = (_e165.y >= 0f);
            local = _e168;
            if _e168 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1561_ = Eh;
                        if Eh {
                            phi_1561_ = (_e165.x < -1.5f);
                        }
                        let _e236 = phi_1561_;
                        if _e236 {
                            let _e242 = textureSampleLevel(YC, ea, vec2<f32>((3f + _e165.x), 0f), 0f);
                            let _e247 = textureSampleLevel(YC, ea, vec2<f32>((1f - _e165.y), 0f), 0f);
                            phi_2122_ = ((1f - _e242.x) - _e247.x);
                            break;
                        } else {
                            phi_2122_ = min(_e165.x, _e165.y);
                            break;
                        }
                    }
                }
                let _e251 = phi_2122_;
                phi_2123_ = _e251;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1379_ = Eh;
                        if Eh {
                            phi_1379_ = (_e165.y < -1.5f);
                        }
                        let _e172 = phi_1379_;
                        if _e172 {
                            let _e176 = max(_e165.w, 0f);
                            if (_e165.z >= 0f) {
                                let _e179 = textureSampleLevel(YC, ea, vec2<f32>(_e176, 0f), 0f);
                                phi_2103_ = _e179.x;
                            } else {
                                phi_2103_ = 0f;
                            }
                            let _e182 = phi_2103_;
                            phi_2120_ = _e182;
                            if (abs(_e165.z) < 1000f) {
                                let _e188 = (-2f - _e165.y);
                                let _e190 = ((_e188 - _e176) * 0.5984134f);
                                let _e193 = (vec4(_e176) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e190));
                                let _e199 = ((_e193 * -(_e165.z)) + vec4(((_e188 * _e165.z) + (abs(_e165.x) - 0.25f))));
                                let _e202 = textureSampleLevel(YC, ea, vec2<f32>(_e199.x, 0f), 0f);
                                let _e205 = textureSampleLevel(YC, ea, vec2<f32>(_e199.y, 0f), 0f);
                                let _e208 = textureSampleLevel(YC, ea, vec2<f32>(_e199.z, 0f), 0f);
                                let _e211 = textureSampleLevel(YC, ea, vec2<f32>(_e199.w, 0f), 0f);
                                let _e217 = (_e193 * 5.0959306f);
                                phi_2120_ = (_e182 + (dot(vec4<f32>(_e202.x, _e205.x, _e208.x, _e211.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e217) * (_e217 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e190));
                            }
                            let _e226 = phi_2120_;
                            phi_2121_ = (_e226 * sign(_e165.x));
                            break;
                        } else {
                            phi_2121_ = _e165.x;
                            break;
                        }
                    }
                }
                let _e231 = phi_2121_;
                phi_2123_ = _e231;
                break;
            }
        }
    }
    let _e253 = phi_2123_;
    let _e254 = p4_1;
    let _e257 = g3_1[1u];
    let _e259 = g3_1[0u];
    let _e260 = vec2<u32>(floor(_e254));
    let _e287 = (_e259 + (((((_e260.y >> bitcast<u32>(5u)) * (_e257 << bitcast<u32>(5u))) + ((_e260.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e260.x & 28u) << bitcast<u32>(5u)) + ((_e260.y & 28u) << bitcast<u32>(2i)))) + (((_e260.y & 3u) << bitcast<u32>(2i)) + (_e260.x & 3u))));
    phi_2125_ = 1f;
    if Ch {
        let _e288 = M0_1;
        let _e291 = min(_e288.xy, _e288.zw);
        phi_2125_ = min(min(_e291.x, _e291.y), 1f);
    }
    let _e297 = phi_2125_;
    phi_1014_ = Bh;
    if Bh {
        let _e299 = W1_1[0u];
        phi_1014_ = (_e299 != 0f);
    }
    let _e302 = phi_1014_;
    phi_2126_ = _e297;
    if _e302 {
        let _e305 = h0_.e2_[_e105];
        phi_2126_ = min(unpack4x8unorm(_e305).x, _e297);
    }
    let _e310 = phi_2126_;
    let _e312 = clamp(_e253, 0f, max(_e310, 0f));
    let _e314 = local;
    if _e314 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e164.w, _e312) >= 1f) {
                    phi_2172_ = _e164.w;
                    break;
                }
                let _e411 = n.d2_;
                let _e413 = atomicMax((&Q0_.e2_[_e287]), (_e411 | u32(((abs(_e312) * 1024f) + 0.5f))));
                if (_e413 < _e411) {
                    phi_2170_ = _e312;
                } else {
                    let _e417 = (f32((_e413 & 524287u)) * 0.0009765625f);
                    phi_2170_ = ((max(_e417, _e312) - _e417) / max((1f - (_e417 * _e164.w)), 0.000062f));
                }
                let _e425 = phi_2170_;
                phi_2172_ = (_e164.w * _e425);
                break;
            }
        }
        let _e428 = phi_2172_;
        phi_2177_ = vec4<f32>(_e164.x, _e164.y, _e164.z, _e428);
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e320 = u32(((abs(_e312) * 1024f) + 0.5f));
                let _e323 = atomicLoad((&Q0_.e2_[_e287]));
                let _e325 = (min(_e164.w, _e312) >= 1f);
                phi_1720_ = _e325;
                if _e325 {
                    let _e327 = n.d2_;
                    let _e328 = (_e323 < _e327);
                    phi_1718_ = _e328;
                    if !(_e328) {
                        phi_1718_ = (_e323 >= (_e327 | 262144u));
                    }
                    let _e333 = phi_1718_;
                    phi_1720_ = _e333;
                }
                let _e335 = phi_1720_;
                if _e335 {
                    phi_2162_ = _e164.w;
                    break;
                }
                let _e337 = n.d2_;
                phi_2152_ = 0f;
                phi_2149_ = _e320;
                phi_2146_ = _e312;
                if (_e323 < _e337) {
                    let _e340 = (_e337 | (262144u + _e320));
                    let _e341 = atomicMax((&Q0_.e2_[_e287]), _e340);
                    if (_e341 <= _e337) {
                        phi_2155_ = _e312;
                        phi_2150_ = _e320;
                        phi_2147_ = 0f;
                    } else {
                        phi_2156_ = 0f;
                        phi_2151_ = _e320;
                        phi_2148_ = _e312;
                        if (_e341 < _e340) {
                            let _e345 = ((_e341 & 524287u) - 262144u);
                            let _e347 = (f32(_e345) * 0.0009765625f);
                            phi_2156_ = ((_e312 - _e347) / max((1f - (_e347 * _e164.w)), 0.000062f));
                            phi_2151_ = _e345;
                            phi_2148_ = _e347;
                        }
                        let _e354 = phi_2156_;
                        let _e356 = phi_2151_;
                        let _e358 = phi_2148_;
                        phi_2155_ = _e354;
                        phi_2150_ = _e356;
                        phi_2147_ = _e358;
                    }
                    let _e360 = phi_2155_;
                    let _e362 = phi_2150_;
                    let _e364 = phi_2147_;
                    phi_2152_ = _e360;
                    phi_2149_ = _e362;
                    phi_2146_ = _e364;
                }
                let _e366 = phi_2152_;
                let _e368 = phi_2149_;
                let _e370 = phi_2146_;
                phi_2160_ = _e366;
                if (_e370 > 0f) {
                    let _e372 = atomicAdd((&Q0_.e2_[_e287]), _e368);
                    let _e377 = (f32(bitcast<i32>(((_e372 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e379 = clamp(_e377, 0f, 1f);
                    phi_2160_ = (_e366 + ((1f - (_e366 * _e164.w)) * ((clamp((_e377 + _e370), 0f, 1f) - _e379) / max((1f - (_e379 * _e164.w)), 0.000062f))));
                }
                let _e391 = phi_2160_;
                phi_2162_ = (_e164.w * _e391);
                break;
            }
        }
        let _e394 = phi_2162_;
        phi_2177_ = vec4<f32>(_e164.x, _e164.y, _e164.z, _e394);
    }
    let _e435 = phi_2177_;
    phi_2179_ = f32();
    if Ih {
        let _e437 = n.B3_;
        let _e439 = n.C3_;
        if Ih {
            phi_2175_ = ((fract((52.982918f * fract(((0.06711056f * _e70.x) + (0.00583715f * _e70.y))))) * _e437) + _e439);
        } else {
            phi_2175_ = 0f;
        }
        let _e451 = phi_2175_;
        phi_2179_ = _e451;
    }
    let _e453 = phi_2179_;
    let _e456 = (_e435.xyz * _e435.w);
    let _e462 = vec4<f32>(_e456.x, _e435.y, _e435.z, _e435.w);
    let _e468 = vec4<f32>(_e462.x, _e456.y, _e462.z, _e462.w);
    let _e474 = vec4<f32>(_e468.x, _e468.y, _e456.z, _e468.w);
    let _e475 = _e474.xyz;
    if (Ih && (_e435.w != 0f)) {
        phi_2207_ = (vec3(_e453) + _e475);
    } else {
        phi_2207_ = _e475;
    }
    let _e481 = phi_2207_;
    let _e487 = vec4<f32>(_e481.x, _e474.y, _e474.z, _e474.w);
    let _e493 = vec4<f32>(_e487.x, _e481.y, _e487.z, _e487.w);
    h0_.e2_[_e105] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    C1_ = vec4<f32>(_e493.x, _e493.y, _e481.z, _e493.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) V1_: vec4<f32>, @location(9) C2_: vec3<f32>, @location(2) M: vec4<f32>, @location(8) p4_: vec2<f32>, @location(7) @interpolate(flat, either) g3_: vec2<u32>, @location(5) M0_: vec4<f32>, @location(4) @interpolate(flat, either) W1_: vec2<f32>, @location(3) @interpolate(flat, either) C0_: f32, @location(6) @interpolate(flat, either) g2_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    V1_1 = V1_;
    C2_1 = C2_;
    M_1 = M;
    p4_1 = p4_;
    g3_1 = g3_;
    M0_1 = M0_;
    W1_1 = W1_;
    C0_1 = C0_;
    g2_1 = g2_;
    main_1();
    let _e21 = C1_;
    return _e21;
}
