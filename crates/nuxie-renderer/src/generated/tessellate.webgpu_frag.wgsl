struct VB {
    yd: f32,
    Ee: f32,
    Jg: f32,
    Kg: f32,
    S6_: u32,
    Ea: u32,
    vg: u32,
    wg: u32,
    H8_: vec4<i32>,
    Ci: vec2<f32>,
    Fe: vec2<f32>,
    q2_: u32,
    Gi: f32,
    w6_: u32,
    i3_: f32,
    Ge: f32,
    og: u32,
    G3_: f32,
    H3_: f32,
    He: f32,
    zi: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

var<private> d7_1: vec4<f32>;
var<private> e7_1: vec4<f32>;
var<private> m5_1: vec4<f32>;
var<private> q8_1: u32;
var<private> n5_1: vec3<f32>;
var<private> Ji: vec4<u32>;
@group(0) @binding(0)
var<uniform> j: VB;

fn main_1() {
    var phi_963_: vec2<f32>;
    var phi_966_: vec2<f32>;
    var phi_257_: bool;
    var phi_971_: f32;
    var phi_970_: f32;
    var phi_275_: bool;
    var phi_979_: u32;
    var phi_978_: u32;
    var phi_991_: u32;
    var phi_322_: bool;
    var phi_329_: bool;
    var phi_1003_: u32;
    var phi_1002_: u32;
    var phi_998_: f32;
    var phi_1065_: f32;
    var phi_1054_: mat2x2<f32>;
    var phi_1001_: u32;
    var phi_999_: f32;
    var phi_997_: f32;
    var phi_1119_: vec2<f32>;
    var phi_1120_: f32;
    var phi_1125_: vec2<f32>;
    var phi_1074_: f32;
    var phi_1073_: i32;
    var phi_1248_: f32;
    var local: f32;
    var local_1: f32;
    var phi_1077_: f32;
    var phi_1080_: f32;
    var phi_1081_: vec2<f32>;
    var phi_1082_: f32;
    var phi_1186_: f32;
    var phi_1114_: f32;
    var phi_1104_: f32;
    var phi_1115_: f32;
    var phi_1185_: f32;
    var phi_1181_: f32;
    var phi_1124_: vec2<f32>;
    var phi_1180_: f32;
    var phi_1121_: vec2<f32>;
    var phi_1209_: f32;
    var phi_1210_: u32;
    var phi_1247_: vec4<u32>;
    var local_2: f32;

    let _e44 = d7_1;
    let _e45 = _e44.xy;
    let _e46 = _e44.zw;
    let _e47 = e7_1;
    let _e48 = _e47.xy;
    let _e49 = _e47.zw;
    if any((_e45 != _e46)) {
        phi_963_ = _e46;
    } else {
        phi_963_ = select(_e49, _e48, vec2(any((_e46 != _e48))));
    }
    let _e57 = phi_963_;
    if any((_e49 != _e48)) {
        phi_966_ = _e48;
    } else {
        phi_966_ = select(_e45, _e46, vec2(any((_e48 != _e46))));
    }
    let _e66 = phi_966_;
    let _e67 = (_e49 - _e66);
    let _e70 = m5_1[0u];
    let _e72 = max(floor(_e70), 0f);
    let _e74 = m5_1[1u];
    let _e76 = m5_1[2u];
    let _e77 = u32(_e76);
    let _e82 = f32((_e77 >> bitcast<u32>(10i)));
    let _e84 = m5_1[3u];
    let _e85 = q8_1;
    let _e86 = (_e74 - _e82);
    let _e87 = (_e72 <= _e86);
    if _e87 {
        phi_1065_ = _e84;
        phi_1054_ = mat2x2<f32>((_e57 - _e45), _e67);
        phi_1001_ = (_e85 & 3825205247u);
        phi_999_ = _e86;
        phi_997_ = _e72;
    } else {
        let _e89 = n5_1;
        let _e94 = (_e72 - _e86);
        let _e96 = n5_1[2u];
        let _e98 = ((_e85 & 33554432u) != 0u);
        phi_257_ = _e98;
        if !(_e98) {
            phi_257_ = ((_e85 & 469762048u) == 67108864u);
        }
        let _e103 = phi_257_;
        phi_971_ = _e82;
        phi_970_ = _e94;
        if _e103 {
            phi_971_ = (_e82 - 2f);
            phi_970_ = (_e94 - 1f);
        }
        let _e107 = phi_971_;
        let _e109 = phi_970_;
        phi_275_ = _e98;
        if _e98 {
            phi_275_ = ((_e109 == 0f) || (_e109 == _e107));
        }
        let _e114 = phi_275_;
        if _e114 {
            phi_978_ = (_e85 & 3825205247u);
        } else {
            phi_979_ = _e85;
            if ((_e85 & 469762048u) != 134217728u) {
                phi_979_ = (_e85 | select(524288u, 1048576u, (_e96 < 0f)));
            }
            let _e122 = phi_979_;
            phi_978_ = _e122;
        }
        let _e124 = phi_978_;
        phi_1002_ = _e124;
        phi_998_ = _e109;
        if ((_e124 & 469762048u) > 201326592u) {
            let _e127 = (_e107 * 0.5f);
            let _e128 = (_e109 < _e127);
            phi_991_ = _e124;
            if _e128 {
                phi_991_ = (_e124 | 4194304u);
            }
            let _e131 = phi_991_;
            let _e132 = (_e107 > 3f);
            phi_322_ = _e132;
            if _e132 {
                phi_322_ = (_e109 > (_e127 - 1f));
            }
            let _e136 = phi_322_;
            phi_329_ = _e136;
            if _e136 {
                phi_329_ = (_e109 < (_e127 + 1f));
            }
            let _e140 = phi_329_;
            phi_1003_ = _e131;
            if _e140 {
                phi_1003_ = (_e131 | 2097152u);
            }
            let _e143 = phi_1003_;
            phi_1002_ = _e143;
            phi_998_ = select(_e107, 0f, _e128);
        }
        let _e146 = phi_1002_;
        let _e148 = phi_998_;
        phi_1065_ = _e96;
        phi_1054_ = mat2x2<f32>(_e67, vec2<f32>(_e89.x, _e89.y));
        phi_1001_ = _e146;
        phi_999_ = _e107;
        phi_997_ = _e148;
    }
    let _e150 = phi_1065_;
    let _e152 = phi_1054_;
    let _e154 = phi_1001_;
    let _e156 = phi_999_;
    let _e158 = phi_997_;
    let _e159 = vec2(_e87);
    let _e160 = select(_e49, _e48, _e159);
    let _e161 = select(_e49, _e45, _e159);
    let _e162 = select(_e49, _e46, _e159);
    let _e163 = select(1f, f32((_e77 & 1023u)), _e87);
    if ((_e158 == 0f) || (_e158 == _e156)) {
        let _e168 = (_e158 < (_e156 * 0.5f));
        if _e168 {
            phi_1119_ = _e152[0];
        } else {
            phi_1119_ = _e152[1];
        }
        let _e174 = phi_1119_;
        let _e175 = normalize(_e174);
        let _e178 = acos(clamp(_e175.x, -1f, 1f));
        if (_e175.y >= 0f) {
            phi_1120_ = _e178;
        } else {
            phi_1120_ = -(_e178);
        }
        let _e183 = phi_1120_;
        phi_1180_ = _e183;
        phi_1121_ = select(_e49, _e161, vec2(_e168));
    } else {
        if ((_e154 & 2147483648u) != 0u) {
            phi_1125_ = select(select(_e161, _e162, vec2((_e158 >= 8f))), _e160, vec2((_e158 >= 12f)));
            if (_e158 >= 14f) {
                let _e193 = n5_1;
                phi_1125_ = _e193.xy;
            }
            let _e196 = phi_1125_;
            phi_1181_ = 0f;
            phi_1124_ = _e196;
        } else {
            if (_e163 == _e156) {
                phi_1186_ = 0f;
                phi_1114_ = 0f;
                phi_1104_ = (_e158 / _e163);
            } else {
                let _e199 = (_e162 - _e161);
                let _e201 = (_e160 - _e162);
                let _e202 = (_e201 - _e199);
                let _e204 = ((_e201 * -3f) + (_e49 - _e161));
                let _e212 = normalize(_e152[0]);
                let _e213 = abs(_e150);
                phi_1074_ = 0f;
                phi_1073_ = 9i;
                loop {
                    let _e218 = phi_1074_;
                    let _e220 = phi_1073_;
                    local = _e218;
                    local_1 = _e218;
                    if (_e220 >= 0i) {
                        let _e224 = (_e218 + exp2(f32(_e220)));
                        phi_1248_ = _e218;
                        if (_e224 <= min((_e163 - 1f), _e158)) {
                            phi_1248_ = select(_e218, _e224, (dot(normalize(((((_e204 * _e224) + (_e202 * (_e163 * 2f))) * _e224) + (_e199 * (_e163 * _e163)))), _e212) >= cos(min(((_e224 * -(_e213)) + ((1f + _e158) * _e213)), 3.1415927f))));
                        }
                        let _e239 = phi_1248_;
                        local_2 = _e239;
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        let _e431 = local_2;
                        phi_1074_ = _e431;
                        phi_1073_ = (_e220 - 1i);
                    }
                }
                let _e242 = local;
                let _e245 = local_1;
                let _e246 = (_e158 - _e245);
                let _e249 = acos(clamp(_e212.x, -1f, 1f));
                if (_e212.y >= 0f) {
                    phi_1077_ = _e249;
                } else {
                    phi_1077_ = -(_e249);
                }
                let _e254 = phi_1077_;
                let _e256 = ((_e246 * _e150) + _e254);
                let _e260 = vec2<f32>(sin(_e256), -(cos(_e256)));
                let _e261 = dot(_e260, _e204);
                let _e262 = dot(_e260, _e202);
                let _e263 = dot(_e260, _e199);
                let _e265 = (_e261 * _e263);
                let _e268 = sqrt(max(((_e262 * _e262) - _e265), 0f));
                phi_1080_ = _e268;
                if (_e262 > 0f) {
                    phi_1080_ = -(_e268);
                }
                let _e272 = phi_1080_;
                let _e273 = (_e272 - _e262);
                let _e275 = ((-0.5f * _e273) * _e261);
                if (abs(((_e273 * _e273) + _e275)) < abs((_e265 + _e275))) {
                    phi_1081_ = vec2<f32>(_e273, _e261);
                } else {
                    phi_1081_ = vec2<f32>(_e263, _e273);
                }
                let _e285 = phi_1081_;
                if (_e285.y != 0f) {
                    phi_1082_ = (_e285.x / _e285.y);
                } else {
                    phi_1082_ = 0f;
                }
                let _e291 = phi_1082_;
                let _e294 = select(clamp(_e291, 0f, 1f), 0f, (_e246 == 0f));
                phi_1186_ = _e256;
                phi_1114_ = _e294;
                phi_1104_ = max((_e242 / _e163), _e294);
            }
            let _e297 = phi_1186_;
            let _e299 = phi_1114_;
            let _e301 = phi_1104_;
            let _e304 = (((_e162 - _e161) * _e301) + _e161);
            let _e307 = (((_e160 - _e162) * _e301) + _e162);
            let _e313 = (((_e307 - _e304) * _e301) + _e304);
            let _e317 = (((((((_e49 - _e160) * _e301) + _e160) - _e307) * _e301) + _e307) - _e313);
            phi_1185_ = _e297;
            if (_e301 != _e299) {
                let _e321 = normalize(_e317);
                let _e324 = acos(clamp(_e321.x, -1f, 1f));
                if (_e321.y >= 0f) {
                    phi_1115_ = _e324;
                } else {
                    phi_1115_ = -(_e324);
                }
                let _e329 = phi_1115_;
                phi_1185_ = _e329;
            }
            let _e331 = phi_1185_;
            phi_1181_ = _e331;
            phi_1124_ = ((_e317 * _e301) + _e313);
        }
        let _e333 = phi_1181_;
        let _e335 = phi_1124_;
        phi_1180_ = _e333;
        phi_1121_ = _e335;
    }
    let _e337 = phi_1180_;
    let _e339 = phi_1121_;
    let _e340 = bitcast<vec2<u32>>(_e339);
    let _e346 = vec4<u32>(_e340.x, vec4<u32>().y, vec4<u32>().z, vec4<u32>().w);
    let _e352 = vec4<u32>(_e346.x, _e340.y, _e346.z, _e346.w);
    let _e353 = (_e154 & 469762048u);
    if (_e353 == 67108864u) {
        phi_1247_ = vec4<u32>(_e352.x, _e352.y, ((u32(_e156) << bitcast<u32>(16i)) | u32(_e158)), _e352.w);
    } else {
        phi_1210_ = 0u;
        if (_e353 > 201326592u) {
            let _e376 = (dot(_e152[0], _e152[0]) * dot(_e152[1], _e152[1]));
            if (_e376 == 0f) {
                phi_1209_ = 1f;
            } else {
                phi_1209_ = clamp((dot(_e152[0], _e152[1]) * inverseSqrt(_e376)), -1f, 1f);
            }
            let _e382 = phi_1209_;
            phi_1210_ = u32(round((sqrt(((1f + clamp(_e382, -1f, 1f)) * 0.5f)) * 65535f)));
        }
        let _e391 = phi_1210_;
        phi_1247_ = vec4<u32>(_e352.x, _e352.y, (((bitcast<u32>(i32(round((_e337 * 10430.378f)))) & 65535u) << bitcast<u32>(16i)) | _e391), _e352.w);
    }
    let _e401 = phi_1247_;
    Ji = vec4<u32>(_e401.x, _e401.y, _e401.z, _e154);
    return;
}

@fragment
fn main(@location(0) d7_: vec4<f32>, @location(1) e7_: vec4<f32>, @location(2) m5_: vec4<f32>, @location(4) @interpolate(flat, either) q8_: u32, @location(3) n5_: vec3<f32>) -> @location(0) vec4<u32> {
    d7_1 = d7_;
    e7_1 = e7_;
    m5_1 = m5_;
    q8_1 = q8_;
    n5_1 = n5_;
    main_1();
    let _e11 = Ji;
    return _e11;
}
