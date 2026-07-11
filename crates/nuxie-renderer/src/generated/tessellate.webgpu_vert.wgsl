struct NB {
    Ub: f32,
    dd: f32,
    Xe: f32,
    Ye: f32,
    q5_: u32,
    ug: u32,
    Je: u32,
    Ke: u32,
    U7_: vec4<i32>,
    rg: vec2<f32>,
    ed: vec2<f32>,
    W1_: u32,
    vg: f32,
    Y5_: u32,
    P2_: f32,
    fd: f32,
    Ee: u32,
    y3_: f32,
    z3_: f32,
    gd: f32,
    og: u32,
}

struct Sf {
    X1_: array<vec4<u32>>,
}

struct Rf {
    X1_: array<vec4<u32>>,
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
var<private> ZC_1: vec4<f32>;
var<private> AD_1: vec4<f32>;
var<private> NC_1: vec4<f32>;
var<private> RB_1: vec4<u32>;
@group(0) @binding(0) 
var<uniform> k: NB;
@group(0) @binding(6) 
var<storage> XC: Sf;
@group(0) @binding(3) 
var<storage> MB: Rf;
@group(0) @binding(10) 
var QC: texture_2d<f32>;
@group(3) @binding(10) 
var T9_: sampler;
var<private> w6_: vec4<f32>;
var<private> x6_: vec4<f32>;
var<private> L4_: vec4<f32>;
var<private> C5_: vec3<f32>;
var<private> I7_: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    var phi_1770_: f32;
    var phi_1771_: u32;
    var phi_1772_: i32;
    var phi_1773_: f32;
    var phi_1970_: vec2<f32>;
    var phi_1774_: u32;
    var phi_1775_: vec4<u32>;
    var phi_1791_: f32;
    var phi_1790_: i32;
    var phi_1792_: f32;
    var phi_1794_: f32;
    var phi_1795_: f32;
    var local: f32;
    var local_1: f32;
    var local_2: f32;
    var phi_1800_: f32;
    var phi_1799_: f32;
    var local_3: vec2<f32>;
    var phi_1811_: f32;
    var phi_1812_: f32;
    var phi_1814_: f32;
    var phi_1813_: f32;
    var phi_1815_: vec2<f32>;
    var phi_1834_: vec2<f32>;
    var phi_1836_: vec2<f32>;
    var phi_1837_: vec2<f32>;
    var phi_1838_: f32;
    var phi_1839_: f32;
    var local_4: vec2<f32>;
    var local_5: vec2<f32>;
    var phi_1840_: f32;
    var phi_1842_: f32;
    var phi_1843_: f32;
    var phi_1892_: vec2<f32>;
    var phi_1891_: vec2<f32>;
    var phi_1914_: u32;
    var phi_1917_: vec2<f32>;
    var phi_1946_: vec2<f32>;
    var phi_1948_: f32;
    var phi_1955_: f32;
    var phi_1994_: f32;
    var phi_1995_: f32;
    var phi_2003_: f32;
    var phi_2004_: f32;
    var phi_2008_: u32;
    var local_6: f32;
    var local_7: f32;

    let _e59 = gl_VertexIndex_1;
    let _e60 = ZC_1;
    let _e61 = _e60.xy;
    let _e62 = _e60.zw;
    let _e63 = AD_1;
    let _e64 = _e63.xy;
    let _e65 = _e63.zw;
    let _e66 = (_e59 < 4i);
    if _e66 {
        let _e68 = NC_1[2u];
        phi_1770_ = _e68;
    } else {
        let _e70 = NC_1[3u];
        phi_1770_ = _e70;
    }
    let _e72 = phi_1770_;
    if _e66 {
        let _e74 = RB_1[0u];
        phi_1771_ = _e74;
    } else {
        let _e76 = RB_1[1u];
        phi_1771_ = _e76;
    }
    let _e78 = phi_1771_;
    let _e79 = bitcast<i32>(_e78);
    let _e81 = (_e79 << bitcast<u32>(16i));
    let _e83 = RB_1[2u];
    phi_1772_ = _e81;
    if (_e83 == 4294967295u) {
        phi_1772_ = (_e81 - 1i);
    }
    let _e87 = phi_1772_;
    let _e90 = f32((_e87 >> bitcast<u32>(16i)));
    let _e93 = f32((_e79 >> bitcast<u32>(16i)));
    if ((_e59 & 2i) == 0i) {
        phi_1773_ = (_e72 + 1f);
    } else {
        phi_1773_ = _e72;
    }
    let _e101 = phi_1773_;
    let _e102 = vec2<f32>(select(_e93, _e90, ((_e59 & 1i) == 0i)), _e101);
    let _e105 = k.dd;
    phi_1970_ = _e102;
    if (((_e93 - _e90) * _e105) < 0f) {
        phi_1970_ = vec2<f32>(_e102.x, (((2f * _e72) + 1f) - _e101));
    }
    let _e115 = phi_1970_;
    let _e116 = (_e83 & 1023u);
    let _e119 = ((_e83 >> bitcast<u32>(10i)) & 1023u);
    let _e121 = (_e83 >> bitcast<u32>(20i));
    let _e123 = RB_1[3u];
    let _e124 = (_e123 & 65535u);
    if (_e124 > 0u) {
        let _e131 = XC.X1_[(max(_e124, 1u) - 1u)][2u];
        phi_1774_ = _e131;
    } else {
        phi_1774_ = 0u;
    }
    let _e133 = phi_1774_;
    if (_e133 != 0u) {
        let _e139 = MB.X1_[((_e133 * 4u) + 1u)];
        phi_1775_ = _e139;
    } else {
        phi_1775_ = vec4<u32>(0u, 0u, 0u, 0u);
    }
    let _e141 = phi_1775_;
    let _e145 = bitcast<f32>(_e141.w);
    phi_1892_ = _e64;
    phi_1891_ = _e62;
    if ((_e145 != 0f) && (bitcast<f32>(_e141.z) == 0f)) {
        switch bitcast<i32>(0u) {
            default: {
                let _e150 = (_e65 - _e61);
                let _e151 = length(_e150);
                local_3 = _e150;
                local_4 = _e150;
                local_5 = _e150;
                if (_e151 == 0f) {
                    phi_1800_ = 0.5f;
                    phi_1799_ = 0f;
                    break;
                }
                let _e158 = (vec2<f32>(-(_e150.y), _e150.x) / vec2(_e151));
                let _e162 = dot(_e158, (_e62 - _e61));
                let _e163 = (_e162 - dot(_e158, (_e64 - _e61)));
                let _e164 = (3f * _e163);
                let _e166 = (-(_e162) - _e163);
                phi_1791_ = 0.5f;
                phi_1790_ = 0i;
                loop {
                    let _e168 = phi_1791_;
                    let _e170 = phi_1790_;
                    local = _e168;
                    local_1 = _e168;
                    local_2 = _e168;
                    local_7 = _e168;
                    if (_e170 < 3i) {
                        let _e172 = (_e164 * _e168);
                        let _e174 = ((_e172 * _e168) - _e162);
                        let _e176 = (2f * (_e172 + _e166));
                        if (_e176 < 0f) {
                            phi_1792_ = -(_e174);
                        } else {
                            phi_1792_ = _e174;
                        }
                        let _e180 = phi_1792_;
                        let _e181 = abs(_e176);
                        if (_e180 > 0f) {
                            if (_e180 < _e181) {
                                phi_1794_ = (_e180 / _e181);
                            } else {
                                phi_1794_ = 1f;
                            }
                            let _e186 = phi_1794_;
                            phi_1795_ = _e186;
                        } else {
                            phi_1795_ = 0f;
                        }
                        let _e188 = phi_1795_;
                        local_6 = _e188;
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        let _e537 = local_6;
                        phi_1791_ = _e537;
                        phi_1790_ = (_e170 + 1i);
                    }
                }
                let _e191 = local;
                let _e196 = local_1;
                let _e201 = local_2;
                let _e547 = local_7;
                phi_1800_ = _e547;
                phi_1799_ = abs((_e201 * ((_e196 * ((_e191 * _e164) + (3f * _e166))) + (3f * _e162))));
                break;
            }
        }
        let _e205 = phi_1800_;
        let _e207 = phi_1799_;
        let _e208 = (_e145 * 0.33333334f);
        switch bitcast<i32>(0u) {
            default: {
                let _e210 = (_e62 - _e61);
                let _e211 = (_e64 - _e62);
                let _e212 = (_e211 - _e210);
                let _e215 = local_3;
                let _e216 = ((_e211 * -3f) + _e215);
                let _e222 = (((((_e216 * _e205) + (_e212 * 2f)) * _e205) + _e210) * 3f);
                let _e223 = length(_e222);
                if (_e223 == 0f) {
                    phi_1839_ = 0f;
                    break;
                }
                let _e226 = (_e222 * (1f / _e223));
                let _e227 = dot(_e216, _e226);
                let _e228 = (2f * _e227);
                let _e237 = (((3f * ((_e228 * _e205) + (4f * dot(_e212, _e226)))) * _e205) + (6f * dot(_e210, _e226)));
                let _e239 = min(_e205, (1f - _e205));
                let _e245 = min(_e208, (((((_e228 * _e239) * _e239) + _e237) * _e239) * 0.9999f));
                if (_e228 == 0f) {
                    phi_1813_ = (_e245 / _e237);
                } else {
                    let _e248 = (0.5f / _e227);
                    let _e252 = (-0.33333334f * (_e237 * _e248));
                    let _e253 = (0.5f * (-(_e245) * _e248));
                    let _e257 = ((_e253 * _e253) - ((_e252 * _e252) * _e252));
                    if (_e257 < 0f) {
                        let _e259 = sqrt(_e252);
                        phi_1814_ = ((-2f * _e259) * cos(((acos((_e253 / ((_e259 * _e259) * _e259))) * 0.33333334f) + -2.0943952f)));
                    } else {
                        let _e272 = pow((abs(_e253) + sqrt(_e257)), 0.33333334f);
                        phi_1811_ = _e272;
                        if (_e253 < 0f) {
                            phi_1811_ = -(_e272);
                        }
                        let _e276 = phi_1811_;
                        if (_e276 != 0f) {
                            phi_1812_ = (_e276 + (_e252 / _e276));
                        } else {
                            phi_1812_ = 0f;
                        }
                        let _e281 = phi_1812_;
                        phi_1814_ = _e281;
                    }
                    let _e283 = phi_1814_;
                    phi_1813_ = _e283;
                }
                let _e285 = phi_1813_;
                let _e286 = abs(_e285);
                let _e287 = -(_e286);
                let _e290 = (vec4(_e205) + vec4<f32>(_e287, _e287, _e286, _e286));
                let _e298 = ((((_e216.xyxy * _e290) + (_e212.xyxy * 2f)) * _e290) + _e210.xyxy);
                if any((_e61 != _e62)) {
                    phi_1815_ = _e62;
                } else {
                    phi_1815_ = select(_e65, _e64, vec2(any((_e62 != _e64))));
                }
                let _e306 = phi_1815_;
                if any((_e65 != _e64)) {
                    phi_1834_ = _e64;
                } else {
                    phi_1834_ = select(_e61, _e62, vec2(any((_e64 != _e62))));
                }
                let _e315 = phi_1834_;
                if (_e290.x < 0.001f) {
                    phi_1836_ = (_e306 - _e61);
                } else {
                    phi_1836_ = _e298.xy;
                }
                let _e321 = phi_1836_;
                if (_e290.z > 0.999f) {
                    phi_1837_ = (_e65 - _e315);
                } else {
                    phi_1837_ = _e298.zw;
                }
                let _e326 = phi_1837_;
                let _e330 = (dot(_e321, _e321) * dot(_e326, _e326));
                if (_e330 == 0f) {
                    phi_1838_ = 1f;
                } else {
                    phi_1838_ = clamp((dot(_e321, _e326) * inverseSqrt(_e330)), -1f, 1f);
                }
                let _e336 = phi_1838_;
                phi_1839_ = acos(_e336);
                break;
            }
        }
        let _e339 = phi_1839_;
        let _e343 = local_4;
        let _e345 = local_5;
        let _e355 = textureSampleLevel(QC, T9_, vec2<f32>((0.5f * min(min((1f - (_e339 * 0.31830987f)), (((dot(_e343, _e345) / (_e208 * _e208)) - 1f) * 0.5f)), 0.99f)), 1f), 0f);
        let _e359 = (((_e355.x * -2f) + 1f) * _e145);
        if (_e207 < 0f) {
            phi_1840_ = -(_e359);
        } else {
            phi_1840_ = _e359;
        }
        let _e363 = phi_1840_;
        let _e364 = abs(_e207);
        if (_e363 > 0f) {
            if (_e363 < _e364) {
                phi_1842_ = (_e363 / _e364);
            } else {
                phi_1842_ = 1f;
            }
            let _e369 = phi_1842_;
            phi_1843_ = _e369;
        } else {
            phi_1843_ = 0f;
        }
        let _e371 = phi_1843_;
        let _e374 = mix(_e60.xyxy, _e63.zwzw, vec4<f32>(0.33333334f, 0.33333334f, 0.6666667f, 0.6666667f));
        let _e376 = vec2(_e371);
        phi_1892_ = mix(_e64, _e374.zw, _e376);
        phi_1891_ = mix(_e62, _e374.xy, _e376);
    }
    let _e381 = phi_1892_;
    let _e383 = phi_1891_;
    phi_1914_ = _e116;
    if ((_e123 & 536870912u) != 0u) {
        let _e389 = MB.X1_[(_e133 * 4u)];
        let _e390 = bitcast<vec4<f32>>(_e389);
        let _e397 = mat2x2<f32>(vec2<f32>(_e390.x, _e390.y), vec2<f32>(_e390.z, _e390.w));
        let _e401 = (_e397 * (((_e383 * -2f) + _e381) + _e61));
        let _e405 = (_e397 * (((_e381 * -2f) + _e65) + _e383));
        phi_1914_ = min(u32(max(ceil(sqrt((3f * sqrt(max(dot(_e401, _e401), dot(_e405, _e405)))))), 1f)), _e116);
    }
    let _e417 = phi_1914_;
    if any((_e61 != _e383)) {
        phi_1917_ = _e383;
    } else {
        phi_1917_ = select(_e65, _e381, vec2(any((_e383 != _e381))));
    }
    let _e428 = phi_1917_;
    let _e429 = (_e428 - _e61);
    if any((_e65 != _e381)) {
        phi_1946_ = _e381;
    } else {
        phi_1946_ = select(_e61, _e383, vec2(any((_e381 != _e383))));
    }
    let _e437 = phi_1946_;
    let _e438 = (_e65 - _e437);
    let _e442 = dot(_e438, _e438);
    let _e443 = (dot(_e429, _e429) * _e442);
    if (_e443 == 0f) {
        phi_1948_ = 1f;
    } else {
        phi_1948_ = clamp((dot(_e429, _e438) * inverseSqrt(_e443)), -1f, 1f);
    }
    let _e449 = phi_1948_;
    let _e452 = (acos(_e449) / f32(_e119));
    let _e456 = determinant(mat2x2<f32>((_e381 - _e61), (_e65 - _e383)));
    phi_1955_ = _e456;
    if (_e456 == 0f) {
        phi_1955_ = determinant(mat2x2<f32>(_e429, _e438));
    }
    let _e460 = phi_1955_;
    phi_1994_ = _e452;
    if (_e460 < 0f) {
        phi_1994_ = -(_e452);
    }
    let _e464 = phi_1994_;
    w6_ = vec4<f32>(_e60.x, _e60.y, _e383.x, _e383.y);
    x6_ = vec4<f32>(_e381.x, _e381.y, _e63.z, _e63.w);
    let _e475 = f32((((_e417 + _e119) + _e121) - 1u));
    L4_ = vec4<f32>((_e475 - abs((_e93 - _e115.x))), _e475, f32(((_e121 << bitcast<u32>(10i)) | _e417)), _e464);
    if (_e121 > 1u) {
        let _e486 = NC_1;
        let _e489 = vec2<f32>(_e486.x, _e486.y);
        let _e493 = (_e442 * dot(_e489, _e489));
        if (_e493 == 0f) {
            phi_1995_ = 1f;
        } else {
            phi_1995_ = clamp((dot(_e438, _e489) * inverseSqrt(_e493)), -1f, 1f);
        }
        let _e499 = phi_1995_;
        let _e501 = f32(_e121);
        phi_2003_ = _e501;
        if ((_e123 & 503316480u) == 167772160u) {
            phi_2003_ = (_e501 - 2f);
        }
        let _e506 = phi_2003_;
        let _e507 = (acos(_e499) / _e506);
        phi_2004_ = _e507;
        if (determinant(mat2x2<f32>(_e438, _e489)) < 0f) {
            phi_2004_ = -(_e507);
        }
        let _e512 = phi_2004_;
        C5_[0u] = _e486.x;
        C5_[1u] = _e486.y;
        C5_[2u] = _e512;
    }
    phi_2008_ = _e123;
    if (_e93 < _e90) {
        phi_2008_ = (_e123 | 8388608u);
    }
    let _e519 = phi_2008_;
    I7_ = _e519;
    unnamed.gl_Position = vec4<f32>(((_e115.x * 0.0009765625f) - 1f), ((_e115.y * _e105) - sign(_e105)), 0f, 1f);
    return;
}

@vertex 
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) ZC: vec4<f32>, @location(1) AD: vec4<f32>, @location(2) NC: vec4<f32>, @location(3) RB: vec4<u32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    ZC_1 = ZC;
    AD_1 = AD;
    NC_1 = NC;
    RB_1 = RB;
    main_1();
    let _e18 = w6_;
    let _e19 = x6_;
    let _e20 = L4_;
    let _e21 = C5_;
    let _e22 = I7_;
    let _e23 = unnamed.gl_Position;
    return VertexOutput(_e18, _e19, _e20, _e21, _e22, _e23);
}
