struct og {
    v2_: array<vec2<u32>>,
}

struct m0Ue {
    v2_: array<u32>,
}

struct pg {
    v2_: array<vec4<f32>>,
}

struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
    P8_: f32,
    Q8_: f32,
}

struct n0Ue {
    v2_: array<u32>,
}

struct T4Ue {
    v2_: array<u32>,
}

@id(8) override nj: bool = true;
@id(7) override mj: bool = true;
@id(9) override oj: bool = true;
@id(5) override kj: bool = true;
@id(0) override fj: bool = true;
@id(1) override gj: bool = true;
@id(2) override hj: bool = true;

@group(0) @binding(3)
var<storage> XC: og;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Ue;
@group(0) @binding(4)
var<storage> JB: pg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var R8_: sampler;
@group(2) @binding(0)
var<storage, read_write> n0_: n0Ue;
@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(1) @binding(13)
var W4_: sampler;
var<private> n2_1: vec2<f32>;
var<private> v5_1: f32;
var<private> W0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> T4_: T4Ue;
var<private> U3_1: u32;
var<private> w5_1: vec4<f32>;
var<private> Y1_1: vec4<f32>;
var<private> N1_1: u32;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var local_3: vec3<f32>;
    var local_4: vec3<f32>;
    var local_5: vec3<f32>;
    var phi_6536_: f32;
    var phi_1802_: bool;
    var phi_5763_: f32;
    var phi_5762_: f32;
    var phi_5764_: f32;
    var phi_5767_: f32;
    var phi_5766_: f32;
    var phi_1839_: bool;
    var phi_5771_: f32;
    var phi_5768_: f32;
    var phi_5769_: f32;
    var phi_5770_: vec2<f32>;
    var phi_5794_: vec4<f32>;
    var phi_6501_: u32;
    var phi_6500_: u32;
    var phi_5792_: vec4<f32>;
    var phi_5801_: f32;
    var phi_6251_: i32;
    var phi_6319_: vec4<f32>;
    var phi_6485_: vec4<f32>;
    var phi_6498_: vec4<f32>;
    var phi_1494_: bool;
    var phi_6526_: u32;
    var phi_6556_: f32;
    var phi_8226_: f32;
    var phi_6588_: f32;
    var phi_6589_: f32;
    var phi_6590_: vec2<f32>;
    var phi_6625_: vec4<f32>;
    var phi_1562_: bool;
    var phi_6636_: f32;
    var phi_6637_: f32;
    var phi_7702_: i32;
    var phi_7858_: vec4<f32>;
    var phi_8241_: vec4<f32>;
    var phi_8254_: vec3<f32>;
    var phi_8256_: vec4<f32>;

    let _e92 = gl_FragCoord_1;
    let _e93 = _e92.xy;
    let _e96 = bitcast<vec2<u32>>(vec2<i32>(floor(_e93)));
    let _e98 = j.T6_;
    let _e127 = bitcast<i32>((((((_e96.y >> bitcast<u32>(5u)) * (((_e98 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e96.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e96.x & 28u) << bitcast<u32>(5u)) + ((_e96.y & 28u) << bitcast<u32>(2i)))) + (((_e96.y & 3u) << bitcast<u32>(2i)) + (_e96.x & 3u))));
    let _e128 = n2_1;
    let _e129 = textureSample(UB, W4_, _e128);
    let _e130 = v5_1;
    let _e131 = min(_e130, 1f);
    phi_6536_ = _e131;
    if gj {
        let _e132 = W0_1;
        let _e135 = min(_e132.xy, _e132.zw);
        phi_6536_ = clamp(min(_e135.x, _e135.y), 0f, _e131);
    }
    let _e141 = phi_6536_;
    let _e144 = T4_.v2_[_e127];
    let _e146 = (_e144 >> bitcast<u32>(17u));
    let _e150 = ((f32((_e144 & 131071u)) * 0.00048828125f) + -32f);
    let _e153 = XC.v2_[_e146];
    phi_5762_ = _e150;
    if ((_e153.x & 768u) != 0u) {
        let _e157 = abs(_e150);
        phi_1802_ = kj;
        if kj {
            phi_1802_ = ((_e153.x & 512u) != 0u);
        }
        let _e161 = phi_1802_;
        phi_5763_ = _e157;
        if _e161 {
            phi_5763_ = (1f - abs(((fract((_e157 * 0.5f)) * 2f) + -1f)));
        }
        let _e169 = phi_5763_;
        phi_5762_ = _e169;
    }
    let _e171 = phi_5762_;
    let _e172 = clamp(_e171, 0f, 1f);
    phi_5766_ = _e172;
    if fj {
        let _e174 = (_e153.x >> bitcast<u32>(16u));
        phi_5767_ = _e172;
        if (_e174 != 0u) {
            let _e178 = m0_.v2_[_e127];
            if (_e174 == (_e178 >> bitcast<u32>(16i))) {
                phi_5764_ = min(_e172, unpack2x16float(_e178).x);
            } else {
                phi_5764_ = 0f;
            }
            let _e186 = phi_5764_;
            phi_5767_ = _e186;
        }
        let _e188 = phi_5767_;
        phi_5766_ = _e188;
    }
    let _e190 = phi_5766_;
    phi_1839_ = gj;
    if gj {
        phi_1839_ = ((_e153.x & 1024u) != 0u);
    }
    let _e194 = phi_1839_;
    phi_5771_ = _e190;
    if _e194 {
        let _e195 = (_e146 * 8u);
        let _e199 = JB.v2_[(_e195 + 2u)];
        let _e210 = JB.v2_[(_e195 + 3u)];
        let _e215 = _e210.zw;
        let _e217 = ((abs(((mat2x2<f32>(vec2<f32>(_e199.x, _e199.y), vec2<f32>(_e199.z, _e199.w)) * _e93) + _e210.xy)) * _e215) - _e215);
        phi_5771_ = min(_e190, clamp((min(_e217.x, _e217.y) + 0.5f), 0f, 1f));
    }
    let _e225 = phi_5771_;
    let _e226 = (_e153.x & 15u);
    let _e229 = ((_e153.x >> bitcast<u32>(4i)) & 15u);
    let _e231 = (hj && (_e229 != 0u));
    if (_e226 <= 1u) {
        let _e312 = (fj && (_e226 == 0u));
        phi_6501_ = 0u;
        if _e312 {
            phi_6501_ = (_e153.y | pack2x16float(vec2<f32>(_e225, 0f)));
        }
        let _e317 = phi_6501_;
        phi_6500_ = _e317;
        phi_5792_ = select(unpack4x8unorm(_e153.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e312));
    } else {
        let _e233 = (_e146 * 8u);
        let _e236 = JB.v2_[_e233];
        let _e247 = JB.v2_[(_e233 + 1u)];
        let _e250 = ((mat2x2<f32>(vec2<f32>(_e236.x, _e236.y), vec2<f32>(_e236.z, _e236.w)) * _e93) + _e247.xy);
        let _e254 = f32(_e226);
        let _e256 = j.P8_;
        let _e258 = j.Q8_;
        let _e263 = ((_e247.w * _e256) + _e258);
        switch bitcast<i32>(0u) {
            default: {
                if (oj && (_e254 == 4f)) {
                    let _e281 = normalize(vec2<f32>(_e250.x, -(_e250.y)));
                    let _e284 = acos(clamp(_e281.x, -1f, 1f));
                    if (_e281.y >= 0f) {
                        phi_5769_ = _e284;
                    } else {
                        phi_5769_ = -(_e284);
                    }
                    let _e289 = phi_5769_;
                    phi_5770_ = vec2<f32>((_e289 * 0.15915494f), _e263);
                    break;
                } else {
                    if (_e254 == 2f) {
                        phi_5768_ = _e250.x;
                    } else {
                        phi_5768_ = length(_e250);
                    }
                    let _e271 = phi_5768_;
                    phi_5770_ = vec2<f32>(((clamp(_e271, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e247.z < 0f))) + ((max(0f, _e247.z) * 0.001953125f) + 0.0009765625f)), _e263);
                    break;
                }
            }
        }
        let _e293 = phi_5770_;
        let _e294 = textureSampleLevel(ZC, R8_, _e293, 0f);
        phi_5794_ = _e294;
        if !(_e231) {
            let _e298 = (_e294.xyz * _e294.w);
            phi_5794_ = vec4<f32>(_e298.x, _e298.y, _e298.z, (_e294.w * abs(bitcast<f32>(_e153.y))));
        }
        let _e308 = phi_5794_;
        phi_6500_ = 0u;
        phi_5792_ = _e308;
    }
    let _e321 = phi_6500_;
    let _e323 = phi_5792_;
    phi_6498_ = _e323;
    if _e231 {
        phi_6485_ = _e323;
        if ((_e323.w * _e225) != 0f) {
            let _e329 = n0_.v2_[_e127];
            let _e330 = unpack4x8unorm(_e329);
            let _e331 = _e323.xyz;
            local_5 = _e331;
            let _e332 = _e330.xyz;
            if (_e330.w != 0f) {
                phi_5801_ = (1f / _e330.w);
            } else {
                phi_5801_ = 0f;
            }
            let _e337 = phi_5801_;
            let _e338 = (_e332 * _e337);
            local_3 = _e338;
            switch bitcast<i32>(_e229) {
                case 11: {
                    let _e622 = local_5;
                    local_4 = (_e622 * _e338);
                    break;
                }
                case 1: {
                    let _e618 = local_5;
                    local_4 = ((_e618 + _e338) - (_e618 * _e338));
                    break;
                }
                case 2: {
                    let _e610 = local_5;
                    let _e611 = (_e610 * _e338);
                    local_4 = (select(_e611, (((_e610 + _e338) - _e611) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e338 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e608 = local_5;
                    local_4 = min(_e608, _e338);
                    break;
                }
                case 4: {
                    let _e606 = local_5;
                    local_4 = max(_e606, _e338);
                    break;
                }
                case 5: {
                    let _e577 = clamp(_e332, vec3<f32>(0f, 0f, 0f), _e330.www);
                    let _e583 = vec4<f32>(_e577.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e589 = vec4<f32>(_e583.x, _e577.y, _e583.z, _e583.w);
                    let _e596 = local_5;
                    let _e599 = (clamp((vec3<f32>(1f, 1f, 1f) - _e596), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e330.w);
                    let _e600 = vec4<f32>(_e589.x, _e589.y, _e577.z, _e589.w).xyz;
                    local_4 = select(min(vec3<f32>(1f, 1f, 1f), (_e600 / _e599)), sign(_e600), (_e599 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e538 = local_5;
                    local_5 = clamp(_e538, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e541 = clamp(_e332, vec3<f32>(0f, 0f, 0f), _e330.www);
                    let _e547 = vec4<f32>(_e541.x, _e330.y, _e330.z, _e330.w);
                    let _e553 = vec4<f32>(_e547.x, _e541.y, _e547.z, _e547.w);
                    phi_6319_ = vec4<f32>(_e553.x, _e553.y, _e541.z, _e553.w);
                    if (_e330.w == 0f) {
                        phi_6319_ = vec4<f32>(_e541.x, _e541.y, _e541.z, 1f);
                    }
                    let _e563 = phi_6319_;
                    let _e567 = (vec3(_e563.w) - _e563.xyz);
                    let _e568 = local_5;
                    local_4 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e567 / (_e568 * _e563.w))), sign(_e567), (_e568 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e530 = local_5;
                    let _e531 = (_e530 * _e338);
                    local_4 = (select(_e531, (((_e530 + _e338) - _e531) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e530 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_6251_ = 0i;
                    loop {
                        let _e499 = phi_6251_;
                        if (_e499 < 3i) {
                            let _e502 = local_5[_e499];
                            if (_e502 <= 0.5f) {
                                let _e519 = local_3[_e499];
                                local_4[_e499] = (1f - _e519);
                            } else {
                                let _e505 = local_3[_e499];
                                if (_e505 <= 0.25f) {
                                    let _e511 = local_3[_e499];
                                    let _e514 = local_3[_e499];
                                    local_4[_e499] = ((((16f * _e511) - 12f) * _e514) + 3f);
                                } else {
                                    let _e507 = local_3[_e499];
                                    local_4[_e499] = (inverseSqrt(_e507) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_6251_ = (_e499 + 1i);
                        }
                    }
                    let _e523 = local_5;
                    let _e527 = local_4;
                    local_4 = (_e338 + ((_e338 * ((_e523 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e527));
                    break;
                }
                case 9: {
                    let _e495 = local_5;
                    local_4 = abs((_e338 - _e495));
                    break;
                }
                case 10: {
                    let _e490 = local_5;
                    local_4 = ((_e490 + _e338) - ((_e490 * 2f) * _e338));
                    break;
                }
                case 12: {
                    if mj {
                        let _e441 = local_5;
                        let _e442 = clamp(_e441, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e442;
                        let _e457 = (_e442 - vec3(min(min(_e442.x, _e442.y), _e442.z)));
                        let _e465 = (_e457 * ((max(max(_e338.x, _e338.y), _e338.z) - min(min(_e338.x, _e338.y), _e338.z)) / max(0.000062f, max(max(_e457.x, _e457.y), _e457.z))));
                        let _e466 = dot(_e338, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e469 = (_e465 - vec3(dot(_e465, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e482 = (vec2<f32>(_e466, (1f - _e466)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e469.x, _e469.y), _e469.z)), max(max(_e469.x, _e469.y), _e469.z))));
                        local_4 = ((_e469 * min(1f, min(_e482.x, _e482.y))) + vec3(_e466));
                    }
                    break;
                }
                case 13: {
                    if mj {
                        let _e392 = local_5;
                        let _e393 = clamp(_e392, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e393;
                        let _e408 = (_e338 - vec3(min(min(_e338.x, _e338.y), _e338.z)));
                        let _e416 = (_e408 * ((max(max(_e393.x, _e393.y), _e393.z) - min(min(_e393.x, _e393.y), _e393.z)) / max(0.000062f, max(max(_e408.x, _e408.y), _e408.z))));
                        let _e417 = dot(_e338, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e420 = (_e416 - vec3(dot(_e416, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e433 = (vec2<f32>(_e417, (1f - _e417)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e420.x, _e420.y), _e420.z)), max(max(_e420.x, _e420.y), _e420.z))));
                        local_4 = ((_e420 * min(1f, min(_e433.x, _e433.y))) + vec3(_e417));
                    }
                    break;
                }
                case 14: {
                    if mj {
                        let _e366 = local_5;
                        let _e367 = clamp(_e366, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e367;
                        let _e368 = dot(_e338, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e371 = (_e367 - vec3(dot(_e367, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e384 = (vec2<f32>(_e368, (1f - _e368)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e371.x, _e371.y), _e371.z)), max(max(_e371.x, _e371.y), _e371.z))));
                        local_4 = ((_e371 * min(1f, min(_e384.x, _e384.y))) + vec3(_e368));
                    }
                    break;
                }
                case 15: {
                    if mj {
                        let _e340 = local_5;
                        let _e341 = clamp(_e340, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e341;
                        let _e342 = dot(_e341, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e345 = (_e338 - vec3(dot(_e338, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e358 = (vec2<f32>(_e342, (1f - _e342)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e345.x, _e345.y), _e345.z)), max(max(_e345.x, _e345.y), _e345.z))));
                        local_4 = ((_e345 * min(1f, min(_e358.x, _e358.y))) + vec3(_e342));
                    }
                    break;
                }
                default: {
                }
            }
            let _e624 = local_4;
            let _e626 = mix(_e331, _e624, vec3(_e330.w));
            let _e632 = vec4<f32>(_e626.x, _e323.y, _e323.z, _e323.w);
            let _e638 = vec4<f32>(_e632.x, _e626.y, _e632.z, _e632.w);
            phi_6485_ = vec4<f32>(_e638.x, _e638.y, _e626.z, _e638.w);
        }
        let _e646 = phi_6485_;
        let _e649 = (_e646.xyz * _e646.w);
        let _e655 = vec4<f32>(_e649.x, _e646.y, _e646.z, _e646.w);
        let _e661 = vec4<f32>(_e655.x, _e649.y, _e655.z, _e655.w);
        phi_6498_ = vec4<f32>(_e661.x, _e661.y, _e649.z, _e661.w);
    }
    let _e669 = phi_6498_;
    let _e670 = (_e669 * _e225);
    phi_1494_ = fj;
    if fj {
        let _e671 = U3_1;
        phi_1494_ = (_e671 != 0u);
    }
    let _e674 = phi_1494_;
    phi_8226_ = _e141;
    if _e674 {
        if (_e321 != 0u) {
            phi_6526_ = _e321;
        } else {
            let _e678 = m0_.v2_[_e127];
            phi_6526_ = _e678;
        }
        let _e680 = phi_6526_;
        let _e681 = U3_1;
        if (_e681 == (_e680 >> bitcast<u32>(16i))) {
            phi_6556_ = min(_e141, unpack2x16float(_e680).x);
        } else {
            phi_6556_ = 0f;
        }
        let _e689 = phi_6556_;
        phi_8226_ = _e689;
    }
    let _e691 = phi_8226_;
    let _e693 = w5_1[3u];
    phi_6625_ = _e129;
    if (_e693 != 0f) {
        let _e695 = w5_1;
        let _e697 = j.P8_;
        let _e699 = j.Q8_;
        let _e702 = abs(_e695.z);
        let _e703 = floor(_e702);
        let _e706 = (((_e702 - _e703) * 8f) + 0.0009765625f);
        let _e708 = ((_e703 * _e697) + _e699);
        let _e709 = floor(_e706);
        switch bitcast<i32>(0u) {
            default: {
                if (oj && (_e709 == 4f)) {
                    let _e728 = normalize(vec2<f32>(_e695.x, -(_e695.y)));
                    let _e731 = acos(clamp(_e728.x, -1f, 1f));
                    if (_e728.y >= 0f) {
                        phi_6589_ = _e731;
                    } else {
                        phi_6589_ = -(_e731);
                    }
                    let _e736 = phi_6589_;
                    phi_6590_ = vec2<f32>((_e736 * 0.15915494f), _e708);
                    break;
                } else {
                    if (_e709 == 2f) {
                        phi_6588_ = _e695.x;
                    } else {
                        phi_6588_ = length(_e695.xy);
                    }
                    let _e718 = phi_6588_;
                    phi_6590_ = vec2<f32>(((clamp(_e718, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e695.z < 0f))) + _e706), _e708);
                    break;
                }
            }
        }
        let _e740 = phi_6590_;
        let _e741 = textureSampleLevel(ZC, R8_, _e740, 0f);
        let _e744 = (_e741.xyz * _e741.w);
        let _e750 = vec4<f32>(_e744.x, _e741.y, _e741.z, _e741.w);
        let _e756 = vec4<f32>(_e750.x, _e744.y, _e750.z, _e750.w);
        phi_6625_ = (_e129 * vec4<f32>(_e756.x, _e756.y, _e744.z, _e756.w));
    }
    let _e765 = phi_6625_;
    let _e766 = Y1_1;
    let _e767 = (_e765 * _e766);
    phi_1562_ = hj;
    if hj {
        let _e768 = N1_1;
        phi_1562_ = (_e768 != 0u);
    }
    let _e771 = phi_1562_;
    phi_8241_ = _e767;
    if _e771 {
        let _e774 = n0_.v2_[_e127];
        let _e779 = ((unpack4x8unorm(_e774) * (1f - _e670.w)) + _e670);
        if (_e767.w != 0f) {
            phi_6636_ = (1f / _e767.w);
        } else {
            phi_6636_ = 0f;
        }
        let _e785 = phi_6636_;
        let _e786 = (_e767.xyz * _e785);
        let _e787 = N1_1;
        local_2 = _e786;
        let _e788 = _e779.xyz;
        if (_e779.w != 0f) {
            phi_6637_ = (1f / _e779.w);
        } else {
            phi_6637_ = 0f;
        }
        let _e793 = phi_6637_;
        let _e794 = (_e788 * _e793);
        local = _e794;
        switch bitcast<i32>(_e787) {
            case 11: {
                let _e1078 = local_2;
                local_1 = (_e1078 * _e794);
                break;
            }
            case 1: {
                let _e1074 = local_2;
                local_1 = ((_e1074 + _e794) - (_e1074 * _e794));
                break;
            }
            case 2: {
                let _e1066 = local_2;
                let _e1067 = (_e1066 * _e794);
                local_1 = (select(_e1067, (((_e1066 + _e794) - _e1067) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e794 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e1064 = local_2;
                local_1 = min(_e1064, _e794);
                break;
            }
            case 4: {
                let _e1062 = local_2;
                local_1 = max(_e1062, _e794);
                break;
            }
            case 5: {
                let _e1033 = clamp(_e788, vec3<f32>(0f, 0f, 0f), _e779.www);
                let _e1039 = vec4<f32>(_e1033.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e1045 = vec4<f32>(_e1039.x, _e1033.y, _e1039.z, _e1039.w);
                let _e1052 = local_2;
                let _e1055 = (clamp((vec3<f32>(1f, 1f, 1f) - _e1052), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e779.w);
                let _e1056 = vec4<f32>(_e1045.x, _e1045.y, _e1033.z, _e1045.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e1056 / _e1055)), sign(_e1056), (_e1055 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e994 = local_2;
                local_2 = clamp(_e994, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e997 = clamp(_e788, vec3<f32>(0f, 0f, 0f), _e779.www);
                let _e1003 = vec4<f32>(_e997.x, _e779.y, _e779.z, _e779.w);
                let _e1009 = vec4<f32>(_e1003.x, _e997.y, _e1003.z, _e1003.w);
                phi_7858_ = vec4<f32>(_e1009.x, _e1009.y, _e997.z, _e1009.w);
                if (_e779.w == 0f) {
                    phi_7858_ = vec4<f32>(_e997.x, _e997.y, _e997.z, 1f);
                }
                let _e1019 = phi_7858_;
                let _e1023 = (vec3(_e1019.w) - _e1019.xyz);
                let _e1024 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e1023 / (_e1024 * _e1019.w))), sign(_e1023), (_e1024 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e986 = local_2;
                let _e987 = (_e986 * _e794);
                local_1 = (select(_e987, (((_e986 + _e794) - _e987) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e986 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_7702_ = 0i;
                loop {
                    let _e955 = phi_7702_;
                    if (_e955 < 3i) {
                        let _e958 = local_2[_e955];
                        if (_e958 <= 0.5f) {
                            let _e975 = local[_e955];
                            local_1[_e955] = (1f - _e975);
                        } else {
                            let _e961 = local[_e955];
                            if (_e961 <= 0.25f) {
                                let _e967 = local[_e955];
                                let _e970 = local[_e955];
                                local_1[_e955] = ((((16f * _e967) - 12f) * _e970) + 3f);
                            } else {
                                let _e963 = local[_e955];
                                local_1[_e955] = (inverseSqrt(_e963) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_7702_ = (_e955 + 1i);
                    }
                }
                let _e979 = local_2;
                let _e983 = local_1;
                local_1 = (_e794 + ((_e794 * ((_e979 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e983));
                break;
            }
            case 9: {
                let _e951 = local_2;
                local_1 = abs((_e794 - _e951));
                break;
            }
            case 10: {
                let _e946 = local_2;
                local_1 = ((_e946 + _e794) - ((_e946 * 2f) * _e794));
                break;
            }
            case 12: {
                if mj {
                    let _e897 = local_2;
                    let _e898 = clamp(_e897, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e898;
                    let _e913 = (_e898 - vec3(min(min(_e898.x, _e898.y), _e898.z)));
                    let _e921 = (_e913 * ((max(max(_e794.x, _e794.y), _e794.z) - min(min(_e794.x, _e794.y), _e794.z)) / max(0.000062f, max(max(_e913.x, _e913.y), _e913.z))));
                    let _e922 = dot(_e794, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e925 = (_e921 - vec3(dot(_e921, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e938 = (vec2<f32>(_e922, (1f - _e922)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e925.x, _e925.y), _e925.z)), max(max(_e925.x, _e925.y), _e925.z))));
                    local_1 = ((_e925 * min(1f, min(_e938.x, _e938.y))) + vec3(_e922));
                }
                break;
            }
            case 13: {
                if mj {
                    let _e848 = local_2;
                    let _e849 = clamp(_e848, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e849;
                    let _e864 = (_e794 - vec3(min(min(_e794.x, _e794.y), _e794.z)));
                    let _e872 = (_e864 * ((max(max(_e849.x, _e849.y), _e849.z) - min(min(_e849.x, _e849.y), _e849.z)) / max(0.000062f, max(max(_e864.x, _e864.y), _e864.z))));
                    let _e873 = dot(_e794, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e876 = (_e872 - vec3(dot(_e872, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e889 = (vec2<f32>(_e873, (1f - _e873)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e876.x, _e876.y), _e876.z)), max(max(_e876.x, _e876.y), _e876.z))));
                    local_1 = ((_e876 * min(1f, min(_e889.x, _e889.y))) + vec3(_e873));
                }
                break;
            }
            case 14: {
                if mj {
                    let _e822 = local_2;
                    let _e823 = clamp(_e822, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e823;
                    let _e824 = dot(_e794, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e827 = (_e823 - vec3(dot(_e823, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e840 = (vec2<f32>(_e824, (1f - _e824)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e827.x, _e827.y), _e827.z)), max(max(_e827.x, _e827.y), _e827.z))));
                    local_1 = ((_e827 * min(1f, min(_e840.x, _e840.y))) + vec3(_e824));
                }
                break;
            }
            case 15: {
                if mj {
                    let _e796 = local_2;
                    let _e797 = clamp(_e796, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e797;
                    let _e798 = dot(_e797, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e801 = (_e794 - vec3(dot(_e794, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e814 = (vec2<f32>(_e798, (1f - _e798)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e801.x, _e801.y), _e801.z)), max(max(_e801.x, _e801.y), _e801.z))));
                    local_1 = ((_e801 * min(1f, min(_e814.x, _e814.y))) + vec3(_e798));
                }
                break;
            }
            default: {
            }
        }
        let _e1080 = local_1;
        let _e1083 = (mix(_e786, _e1080, vec3(_e779.w)) * _e767.w);
        let _e1089 = vec4<f32>(_e1083.x, _e767.y, _e767.z, _e767.w);
        let _e1095 = vec4<f32>(_e1089.x, _e1083.y, _e1089.z, _e1089.w);
        phi_8241_ = vec4<f32>(_e1095.x, _e1095.y, _e1083.z, _e1095.w);
    }
    let _e1103 = phi_8241_;
    let _e1104 = (_e1103 * _e691);
    let _e1108 = ((_e670 * (1f - _e1104.w)) + _e1104);
    let _e1109 = _e1108.xyz;
    let _e1112 = j.G3_;
    let _e1114 = j.H3_;
    if (nj && (_e1108.w != 0f)) {
        phi_8254_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e92.x) + (0.00583715f * _e92.y))))) * _e1112) + _e1114)) + _e1109);
    } else {
        phi_8254_ = _e1109;
    }
    let _e1130 = phi_8254_;
    let _e1136 = vec4<f32>(_e1130.x, _e1108.y, _e1108.z, _e1108.w);
    let _e1142 = vec4<f32>(_e1136.x, _e1130.y, _e1136.z, _e1136.w);
    let _e1148 = vec4<f32>(_e1142.x, _e1142.y, _e1130.z, _e1142.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e1130.x + _e1130.y) + _e1130.z) + _e1108.w) == 0f) {
                break;
            }
            phi_8256_ = _e1148;
            if (_e1108.w != 1f) {
                let _e1157 = n0_.v2_[_e127];
                phi_8256_ = (_e1148 + (unpack4x8unorm(_e1157) * (1f - _e1108.w)));
            }
            let _e1163 = phi_8256_;
            n0_.v2_[_e127] = pack4x8unorm(_e1163);
            break;
        }
    }
    if (_e321 != 0u) {
        m0_.v2_[_e127] = _e321;
    }
    T4_.v2_[_e127] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) n2_: vec2<f32>, @location(1) v5_: f32, @location(3) W0_: vec4<f32>, @location(5) @interpolate(flat, either) U3_: u32, @location(2) w5_: vec4<f32>, @location(4) @interpolate(flat, either) Y1_: vec4<f32>, @location(6) @interpolate(flat, either) N1_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    n2_1 = n2_;
    v5_1 = v5_;
    W0_1 = W0_;
    U3_1 = U3_;
    w5_1 = w5_;
    Y1_1 = Y1_;
    N1_1 = N1_;
    main_1();
}
