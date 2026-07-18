struct CC {
    bc: f32,
    kd: f32,
    bf: f32,
    cf: f32,
    m6_: u32,
    Bg: u32,
    Ne: u32,
    Oe: u32,
    Q7_: vec4<i32>,
    xg: vec2<f32>,
    ld: vec2<f32>,
    a2_: u32,
    Cg: f32,
    Z5_: u32,
    N2_: f32,
    md: f32,
    Ie: u32,
    y3_: f32,
    z3_: f32,
    nd: f32,
    ug: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @location(1) member_1: vec4<f32>,
    @location(2) member_2: vec4<f32>,
    @location(3) member_3: vec3<f32>,
    @location(4) @interpolate(flat) member_4: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

var<private> gl_VertexIndex_1: i32;
var<private> GD_1: vec4<f32>;
var<private> HD_1: vec4<f32>;
var<private> UC_1: vec4<f32>;
var<private> TB_1: vec4<u32>;
@group(0) @binding(0)
var<uniform> m: CC;
@group(0) @binding(5)
var ED: texture_2d<u32>;
@group(0) @binding(2)
var PB: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var Z9_: sampler;
var<private> x6_: vec4<f32>;
var<private> y6_: vec4<f32>;
var<private> L4_: vec4<f32>;
var<private> C5_: vec3<f32>;
var<private> E7_: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    var phi_1791_: f32;
    var phi_1792_: u32;
    var phi_1793_: i32;
    var phi_1794_: f32;
    var phi_1991_: vec2<f32>;
    var phi_1795_: u32;
    var phi_1796_: vec4<u32>;
    var phi_1812_: f32;
    var phi_1811_: i32;
    var phi_1813_: f32;
    var phi_1815_: f32;
    var phi_1816_: f32;
    var local: f32;
    var local_1: f32;
    var local_2: f32;
    var phi_1821_: f32;
    var phi_1820_: f32;
    var local_3: vec2<f32>;
    var phi_1832_: f32;
    var phi_1833_: f32;
    var phi_1835_: f32;
    var phi_1834_: f32;
    var phi_1836_: vec2<f32>;
    var phi_1855_: vec2<f32>;
    var phi_1857_: vec2<f32>;
    var phi_1858_: vec2<f32>;
    var phi_1859_: f32;
    var phi_1860_: f32;
    var local_4: vec2<f32>;
    var local_5: vec2<f32>;
    var phi_1861_: f32;
    var phi_1863_: f32;
    var phi_1864_: f32;
    var phi_1913_: vec2<f32>;
    var phi_1912_: vec2<f32>;
    var phi_1935_: u32;
    var phi_1938_: vec2<f32>;
    var phi_1967_: vec2<f32>;
    var phi_1969_: f32;
    var phi_1976_: f32;
    var phi_2015_: f32;
    var phi_2016_: f32;
    var phi_2024_: f32;
    var phi_2025_: f32;
    var phi_2029_: u32;
    var local_6: f32;
    var local_7: f32;

    let _e61 = gl_VertexIndex_1;
    let _e62 = GD_1;
    let _e63 = _e62.xy;
    let _e64 = _e62.zw;
    let _e65 = HD_1;
    let _e66 = _e65.xy;
    let _e67 = _e65.zw;
    let _e68 = (_e61 < 4i);
    if _e68 {
        let _e70 = UC_1[2u];
        phi_1791_ = _e70;
    } else {
        let _e72 = UC_1[3u];
        phi_1791_ = _e72;
    }
    let _e74 = phi_1791_;
    if _e68 {
        let _e76 = TB_1[0u];
        phi_1792_ = _e76;
    } else {
        let _e78 = TB_1[1u];
        phi_1792_ = _e78;
    }
    let _e80 = phi_1792_;
    let _e81 = bitcast<i32>(_e80);
    let _e83 = (_e81 << bitcast<u32>(16i));
    let _e85 = TB_1[2u];
    phi_1793_ = _e83;
    if (_e85 == 4294967295u) {
        phi_1793_ = (_e83 - 1i);
    }
    let _e89 = phi_1793_;
    let _e92 = f32((_e89 >> bitcast<u32>(16i)));
    let _e95 = f32((_e81 >> bitcast<u32>(16i)));
    if ((_e61 & 2i) == 0i) {
        phi_1794_ = (_e74 + 1f);
    } else {
        phi_1794_ = _e74;
    }
    let _e103 = phi_1794_;
    let _e104 = vec2<f32>(select(_e95, _e92, ((_e61 & 1i) == 0i)), _e103);
    let _e107 = m.kd;
    phi_1991_ = _e104;
    if (((_e95 - _e92) * _e107) < 0f) {
        phi_1991_ = vec2<f32>(_e104.x, (((2f * _e74) + 1f) - _e103));
    }
    let _e117 = phi_1991_;
    let _e118 = (_e85 & 1023u);
    let _e121 = ((_e85 >> bitcast<u32>(10i)) & 1023u);
    let _e123 = (_e85 >> bitcast<u32>(20i));
    let _e125 = TB_1[3u];
    let _e126 = (_e125 & 65535u);
    if (_e126 > 0u) {
        let _e129 = (max(_e126, 1u) - 1u);
        let _e136 = textureLoad(ED, vec2<i32>(bitcast<i32>((_e129 & 127u)), bitcast<i32>((_e129 >> bitcast<u32>(7i)))), 0i);
        phi_1795_ = _e136.z;
    } else {
        phi_1795_ = 0u;
    }
    let _e139 = phi_1795_;
    if (_e139 != 0u) {
        let _e142 = ((_e139 * 4u) + 1u);
        let _e149 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e142 & 127u)), bitcast<i32>((_e142 >> bitcast<u32>(7i)))), 0i);
        phi_1796_ = _e149;
    } else {
        phi_1796_ = vec4<u32>(0u, 0u, 0u, 0u);
    }
    let _e151 = phi_1796_;
    let _e155 = bitcast<f32>(_e151.w);
    phi_1913_ = _e66;
    phi_1912_ = _e64;
    if ((_e155 != 0f) && (bitcast<f32>(_e151.z) == 0f)) {
        switch bitcast<i32>(0u) {
            default: {
                let _e160 = (_e67 - _e63);
                let _e161 = length(_e160);
                local_3 = _e160;
                local_4 = _e160;
                local_5 = _e160;
                if (_e161 == 0f) {
                    phi_1821_ = 0.5f;
                    phi_1820_ = 0f;
                    break;
                }
                let _e168 = (vec2<f32>(-(_e160.y), _e160.x) / vec2(_e161));
                let _e172 = dot(_e168, (_e64 - _e63));
                let _e173 = (_e172 - dot(_e168, (_e66 - _e63)));
                let _e174 = (3f * _e173);
                let _e176 = (-(_e172) - _e173);
                phi_1812_ = 0.5f;
                phi_1811_ = 0i;
                loop {
                    let _e178 = phi_1812_;
                    let _e180 = phi_1811_;
                    local = _e178;
                    local_1 = _e178;
                    local_2 = _e178;
                    local_7 = _e178;
                    if (_e180 < 3i) {
                        let _e182 = (_e174 * _e178);
                        let _e184 = ((_e182 * _e178) - _e172);
                        let _e186 = (2f * (_e182 + _e176));
                        if (_e186 < 0f) {
                            phi_1813_ = -(_e184);
                        } else {
                            phi_1813_ = _e184;
                        }
                        let _e190 = phi_1813_;
                        let _e191 = abs(_e186);
                        if (_e190 > 0f) {
                            if (_e190 < _e191) {
                                phi_1815_ = (_e190 / _e191);
                            } else {
                                phi_1815_ = 1f;
                            }
                            let _e196 = phi_1815_;
                            phi_1816_ = _e196;
                        } else {
                            phi_1816_ = 0f;
                        }
                        let _e198 = phi_1816_;
                        local_6 = _e198;
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        let _e551 = local_6;
                        phi_1812_ = _e551;
                        phi_1811_ = (_e180 + 1i);
                    }
                }
                let _e201 = local;
                let _e206 = local_1;
                let _e211 = local_2;
                let _e561 = local_7;
                phi_1821_ = _e561;
                phi_1820_ = abs((_e211 * ((_e206 * ((_e201 * _e174) + (3f * _e176))) + (3f * _e172))));
                break;
            }
        }
        let _e215 = phi_1821_;
        let _e217 = phi_1820_;
        let _e218 = (_e155 * 0.33333334f);
        switch bitcast<i32>(0u) {
            default: {
                let _e220 = (_e64 - _e63);
                let _e221 = (_e66 - _e64);
                let _e222 = (_e221 - _e220);
                let _e225 = local_3;
                let _e226 = ((_e221 * -3f) + _e225);
                let _e232 = (((((_e226 * _e215) + (_e222 * 2f)) * _e215) + _e220) * 3f);
                let _e233 = length(_e232);
                if (_e233 == 0f) {
                    phi_1860_ = 0f;
                    break;
                }
                let _e236 = (_e232 * (1f / _e233));
                let _e237 = dot(_e226, _e236);
                let _e238 = (2f * _e237);
                let _e247 = (((3f * ((_e238 * _e215) + (4f * dot(_e222, _e236)))) * _e215) + (6f * dot(_e220, _e236)));
                let _e249 = min(_e215, (1f - _e215));
                let _e255 = min(_e218, (((((_e238 * _e249) * _e249) + _e247) * _e249) * 0.9999f));
                if (_e238 == 0f) {
                    phi_1834_ = (_e255 / _e247);
                } else {
                    let _e258 = (0.5f / _e237);
                    let _e262 = (-0.33333334f * (_e247 * _e258));
                    let _e263 = (0.5f * (-(_e255) * _e258));
                    let _e267 = ((_e263 * _e263) - ((_e262 * _e262) * _e262));
                    if (_e267 < 0f) {
                        let _e269 = sqrt(_e262);
                        phi_1835_ = ((-2f * _e269) * cos(((acos((_e263 / ((_e269 * _e269) * _e269))) * 0.33333334f) + -2.0943952f)));
                    } else {
                        let _e282 = pow((abs(_e263) + sqrt(_e267)), 0.33333334f);
                        phi_1832_ = _e282;
                        if (_e263 < 0f) {
                            phi_1832_ = -(_e282);
                        }
                        let _e286 = phi_1832_;
                        if (_e286 != 0f) {
                            phi_1833_ = (_e286 + (_e262 / _e286));
                        } else {
                            phi_1833_ = 0f;
                        }
                        let _e291 = phi_1833_;
                        phi_1835_ = _e291;
                    }
                    let _e293 = phi_1835_;
                    phi_1834_ = _e293;
                }
                let _e295 = phi_1834_;
                let _e296 = abs(_e295);
                let _e297 = -(_e296);
                let _e300 = (vec4(_e215) + vec4<f32>(_e297, _e297, _e296, _e296));
                let _e308 = ((((_e226.xyxy * _e300) + (_e222.xyxy * 2f)) * _e300) + _e220.xyxy);
                if any((_e63 != _e64)) {
                    phi_1836_ = _e64;
                } else {
                    phi_1836_ = select(_e67, _e66, vec2(any((_e64 != _e66))));
                }
                let _e316 = phi_1836_;
                if any((_e67 != _e66)) {
                    phi_1855_ = _e66;
                } else {
                    phi_1855_ = select(_e63, _e64, vec2(any((_e66 != _e64))));
                }
                let _e325 = phi_1855_;
                if (_e300.x < 0.001f) {
                    phi_1857_ = (_e316 - _e63);
                } else {
                    phi_1857_ = _e308.xy;
                }
                let _e331 = phi_1857_;
                if (_e300.z > 0.999f) {
                    phi_1858_ = (_e67 - _e325);
                } else {
                    phi_1858_ = _e308.zw;
                }
                let _e336 = phi_1858_;
                let _e340 = (dot(_e331, _e331) * dot(_e336, _e336));
                if (_e340 == 0f) {
                    phi_1859_ = 1f;
                } else {
                    phi_1859_ = clamp((dot(_e331, _e336) * inverseSqrt(_e340)), -1f, 1f);
                }
                let _e346 = phi_1859_;
                phi_1860_ = acos(_e346);
                break;
            }
        }
        let _e349 = phi_1860_;
        let _e353 = local_4;
        let _e355 = local_5;
        let _e365 = textureSampleLevel(XC, Z9_, vec2<f32>((0.5f * min(min((1f - (_e349 * 0.31830987f)), (((dot(_e353, _e355) / (_e218 * _e218)) - 1f) * 0.5f)), 0.99f)), 1f), 0f);
        let _e369 = (((_e365.x * -2f) + 1f) * _e155);
        if (_e217 < 0f) {
            phi_1861_ = -(_e369);
        } else {
            phi_1861_ = _e369;
        }
        let _e373 = phi_1861_;
        let _e374 = abs(_e217);
        if (_e373 > 0f) {
            if (_e373 < _e374) {
                phi_1863_ = (_e373 / _e374);
            } else {
                phi_1863_ = 1f;
            }
            let _e379 = phi_1863_;
            phi_1864_ = _e379;
        } else {
            phi_1864_ = 0f;
        }
        let _e381 = phi_1864_;
        let _e384 = mix(_e62.xyxy, _e65.zwzw, vec4<f32>(0.33333334f, 0.33333334f, 0.6666667f, 0.6666667f));
        let _e386 = vec2(_e381);
        phi_1913_ = mix(_e66, _e384.zw, _e386);
        phi_1912_ = mix(_e64, _e384.xy, _e386);
    }
    let _e391 = phi_1913_;
    let _e393 = phi_1912_;
    phi_1935_ = _e118;
    if ((_e125 & 536870912u) != 0u) {
        let _e396 = (_e139 * 4u);
        let _e403 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e396 & 127u)), bitcast<i32>((_e396 >> bitcast<u32>(7i)))), 0i);
        let _e404 = bitcast<vec4<f32>>(_e403);
        let _e411 = mat2x2<f32>(vec2<f32>(_e404.x, _e404.y), vec2<f32>(_e404.z, _e404.w));
        let _e415 = (_e411 * (((_e393 * -2f) + _e391) + _e63));
        let _e419 = (_e411 * (((_e391 * -2f) + _e67) + _e393));
        phi_1935_ = min(u32(max(ceil(sqrt((3f * sqrt(max(dot(_e415, _e415), dot(_e419, _e419)))))), 1f)), _e118);
    }
    let _e431 = phi_1935_;
    if any((_e63 != _e393)) {
        phi_1938_ = _e393;
    } else {
        phi_1938_ = select(_e67, _e391, vec2(any((_e393 != _e391))));
    }
    let _e442 = phi_1938_;
    let _e443 = (_e442 - _e63);
    if any((_e67 != _e391)) {
        phi_1967_ = _e391;
    } else {
        phi_1967_ = select(_e63, _e393, vec2(any((_e391 != _e393))));
    }
    let _e451 = phi_1967_;
    let _e452 = (_e67 - _e451);
    let _e456 = dot(_e452, _e452);
    let _e457 = (dot(_e443, _e443) * _e456);
    if (_e457 == 0f) {
        phi_1969_ = 1f;
    } else {
        phi_1969_ = clamp((dot(_e443, _e452) * inverseSqrt(_e457)), -1f, 1f);
    }
    let _e463 = phi_1969_;
    let _e466 = (acos(_e463) / f32(_e121));
    let _e470 = determinant(mat2x2<f32>((_e391 - _e63), (_e67 - _e393)));
    phi_1976_ = _e470;
    if (_e470 == 0f) {
        phi_1976_ = determinant(mat2x2<f32>(_e443, _e452));
    }
    let _e474 = phi_1976_;
    phi_2015_ = _e466;
    if (_e474 < 0f) {
        phi_2015_ = -(_e466);
    }
    let _e478 = phi_2015_;
    x6_ = vec4<f32>(_e62.x, _e62.y, _e393.x, _e393.y);
    y6_ = vec4<f32>(_e391.x, _e391.y, _e65.z, _e65.w);
    let _e489 = f32((((_e431 + _e121) + _e123) - 1u));
    L4_ = vec4<f32>((_e489 - abs((_e95 - _e117.x))), _e489, f32(((_e123 << bitcast<u32>(10i)) | _e431)), _e478);
    if (_e123 > 1u) {
        let _e500 = UC_1;
        let _e503 = vec2<f32>(_e500.x, _e500.y);
        let _e507 = (_e456 * dot(_e503, _e503));
        if (_e507 == 0f) {
            phi_2016_ = 1f;
        } else {
            phi_2016_ = clamp((dot(_e452, _e503) * inverseSqrt(_e507)), -1f, 1f);
        }
        let _e513 = phi_2016_;
        let _e515 = f32(_e123);
        phi_2024_ = _e515;
        if ((_e125 & 503316480u) == 167772160u) {
            phi_2024_ = (_e515 - 2f);
        }
        let _e520 = phi_2024_;
        let _e521 = (acos(_e513) / _e520);
        phi_2025_ = _e521;
        if (determinant(mat2x2<f32>(_e452, _e503)) < 0f) {
            phi_2025_ = -(_e521);
        }
        let _e526 = phi_2025_;
        C5_[0u] = _e500.x;
        C5_[1u] = _e500.y;
        C5_[2u] = _e526;
    }
    phi_2029_ = _e125;
    if (_e95 < _e92) {
        phi_2029_ = (_e125 | 8388608u);
    }
    let _e533 = phi_2029_;
    E7_ = _e533;
    unnamed.gl_Position = vec4<f32>(((_e117.x * 0.0009765625f) - 1f), ((_e117.y * _e107) - sign(_e107)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) GD: vec4<f32>, @location(1) HD: vec4<f32>, @location(2) UC: vec4<f32>, @location(3) TB: vec4<u32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    GD_1 = GD;
    HD_1 = HD;
    UC_1 = UC;
    TB_1 = TB;
    main_1();
    let _e18 = x6_;
    let _e19 = y6_;
    let _e20 = L4_;
    let _e21 = C5_;
    let _e22 = E7_;
    let _e23 = unnamed.gl_Position;
    return VertexOutput(_e18, _e19, _e20, _e21, _e22, _e23);
}
