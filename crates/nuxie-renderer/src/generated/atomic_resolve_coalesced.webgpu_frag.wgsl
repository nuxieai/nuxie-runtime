struct lf {
    g2_: array<vec2<u32>>,
}

struct i0Td {
    g2_: array<u32>,
}

struct mf {
    g2_: array<vec4<f32>>,
}

struct l0Td {
    g2_: array<u32>,
}

struct AC {
    tc: f32,
    Dd: f32,
    Hf: f32,
    If: f32,
    q6_: u32,
    Qb: u32,
    tf: u32,
    uf: u32,
    X7_: vec4<i32>,
    eh: vec2<f32>,
    Ed: vec2<f32>,
    f2_: u32,
    ih: f32,
    f6_: u32,
    U2_: f32,
    Fd: f32,
    of_: u32,
    F3_: f32,
    G3_: f32,
    Gd: f32,
    bh: u32,
    Pb: u32,
}

struct z4Td {
    g2_: array<u32>,
}

@id(6) override Kh: bool = true;
@id(4) override Ih: bool = true;
@id(0) override Eh: bool = true;
@id(1) override Fh: bool = true;
@id(2) override Gh: bool = true;

@group(0) @binding(3)
var<storage> CD: lf;
@group(2) @binding(1)
var<storage, read_write> i0_: i0Td;
@group(0) @binding(4)
var<storage> PB: mf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var P9_: sampler;
@group(2) @binding(0)
var<storage, read_write> l0_: l0Td;
@group(0) @binding(0)
var<uniform> j: AC;
@group(2) @binding(3)
var<storage, read_write> z4_: z4Td;
var<private> E1_: vec4<f32>;
@group(3) @binding(9)
var ga: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var Y5_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1253_: bool;
    var phi_3177_: f32;
    var phi_3176_: f32;
    var phi_3178_: f32;
    var phi_3181_: f32;
    var phi_3180_: f32;
    var phi_1290_: bool;
    var phi_3197_: f32;
    var phi_3182_: f32;
    var phi_3194_: vec4<f32>;
    var phi_3192_: vec4<f32>;
    var phi_3200_: f32;
    var phi_3623_: vec4<f32>;
    var phi_3567_: i32;
    var phi_3762_: vec4<f32>;
    var phi_3775_: vec4<f32>;
    var phi_3776_: vec4<f32>;

    let _e67 = gl_FragCoord_1;
    let _e68 = _e67.xy;
    let _e71 = bitcast<vec2<u32>>(vec2<i32>(floor(_e68)));
    let _e73 = j.q6_;
    let _e102 = bitcast<i32>((((((_e71.y >> bitcast<u32>(5u)) * (((_e73 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e71.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e71.x & 28u) << bitcast<u32>(5u)) + ((_e71.y & 28u) << bitcast<u32>(2i)))) + (((_e71.y & 3u) << bitcast<u32>(2i)) + (_e71.x & 3u))));
    let _e105 = z4_.g2_[_e102];
    let _e109 = ((f32((_e105 & 131071u)) * 0.00048828125f) + -32f);
    let _e111 = (_e105 >> bitcast<u32>(17u));
    let _e114 = CD.g2_[_e111];
    phi_3176_ = _e109;
    if ((_e114.x & 768u) != 0u) {
        let _e118 = abs(_e109);
        phi_1253_ = Ih;
        if Ih {
            phi_1253_ = ((_e114.x & 512u) != 0u);
        }
        let _e122 = phi_1253_;
        phi_3177_ = _e118;
        if _e122 {
            phi_3177_ = (1f - abs(((fract((_e118 * 0.5f)) * 2f) + -1f)));
        }
        let _e130 = phi_3177_;
        phi_3176_ = _e130;
    }
    let _e132 = phi_3176_;
    let _e133 = clamp(_e132, 0f, 1f);
    phi_3180_ = _e133;
    if Eh {
        let _e135 = (_e114.x >> bitcast<u32>(16u));
        phi_3181_ = _e133;
        if (_e135 != 0u) {
            let _e139 = i0_.g2_[_e102];
            if (_e135 == (_e139 >> bitcast<u32>(16i))) {
                phi_3178_ = min(_e133, unpack2x16float(_e139).x);
            } else {
                phi_3178_ = 0f;
            }
            let _e147 = phi_3178_;
            phi_3181_ = _e147;
        }
        let _e149 = phi_3181_;
        phi_3180_ = _e149;
    }
    let _e151 = phi_3180_;
    phi_1290_ = Fh;
    if Fh {
        phi_1290_ = ((_e114.x & 1024u) != 0u);
    }
    let _e155 = phi_1290_;
    phi_3197_ = _e151;
    if _e155 {
        let _e156 = (_e111 * 8u);
        let _e160 = PB.g2_[(_e156 + 2u)];
        let _e171 = PB.g2_[(_e156 + 3u)];
        let _e176 = _e171.zw;
        let _e178 = ((abs(((mat2x2<f32>(vec2<f32>(_e160.x, _e160.y), vec2<f32>(_e160.z, _e160.w)) * _e68) + _e171.xy)) * _e176) - _e176);
        phi_3197_ = min(_e151, clamp((min(_e178.x, _e178.y) + 0.5f), 0f, 1f));
    }
    let _e186 = phi_3197_;
    let _e187 = (_e114.x & 15u);
    let _e190 = ((_e114.x >> bitcast<u32>(4i)) & 15u);
    let _e192 = (Gh && (_e190 != 0u));
    if (_e187 <= 1u) {
        phi_3192_ = select(unpack4x8unorm(_e114.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((Eh && (_e187 == 0u))));
    } else {
        let _e200 = (_e111 * 8u);
        let _e203 = PB.g2_[_e200];
        let _e214 = PB.g2_[(_e200 + 1u)];
        let _e217 = ((mat2x2<f32>(vec2<f32>(_e203.x, _e203.y), vec2<f32>(_e203.z, _e203.w)) * _e68) + _e214.xy);
        if (_e187 == 2u) {
            phi_3182_ = _e217.x;
        } else {
            phi_3182_ = length(_e217);
        }
        let _e222 = phi_3182_;
        let _e231 = textureSampleLevel(DD, P9_, vec2<f32>(((clamp(_e222, 0f, 1f) * _e214.z) + _e214.w), bitcast<f32>(_e114.y)), 0f);
        phi_3194_ = _e231;
        if !(_e192) {
            let _e235 = (_e231.xyz * _e231.w);
            let _e241 = vec4<f32>(_e235.x, _e231.y, _e231.z, _e231.w);
            let _e247 = vec4<f32>(_e241.x, _e235.y, _e241.z, _e241.w);
            phi_3194_ = vec4<f32>(_e247.x, _e247.y, _e235.z, _e247.w);
        }
        let _e255 = phi_3194_;
        phi_3192_ = _e255;
    }
    let _e257 = phi_3192_;
    phi_3775_ = _e257;
    if _e192 {
        phi_3762_ = _e257;
        if ((_e257.w * _e186) != 0f) {
            let _e263 = l0_.g2_[_e102];
            let _e264 = unpack4x8unorm(_e263);
            let _e265 = _e257.xyz;
            local_2 = _e265;
            let _e266 = _e264.xyz;
            if (_e264.w != 0f) {
                phi_3200_ = (1f / _e264.w);
            } else {
                phi_3200_ = 0f;
            }
            let _e271 = phi_3200_;
            let _e272 = (_e266 * _e271);
            local = _e272;
            switch bitcast<i32>(_e190) {
                case 11: {
                    let _e274 = local_2;
                    local_1 = (_e274 * _e272);
                    break;
                }
                case 1: {
                    let _e276 = local_2;
                    local_1 = ((_e276 + _e272) - (_e276 * _e272));
                    break;
                }
                case 2: {
                    let _e280 = local_2;
                    let _e281 = (_e280 * _e272);
                    local_1 = (select(_e281, (((_e280 + _e272) - _e281) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e272 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e288 = local_2;
                    local_1 = min(_e288, _e272);
                    break;
                }
                case 4: {
                    let _e290 = local_2;
                    local_1 = max(_e290, _e272);
                    break;
                }
                case 5: {
                    let _e293 = clamp(_e266, vec3<f32>(0f, 0f, 0f), _e264.www);
                    let _e299 = vec4<f32>(_e293.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e305 = vec4<f32>(_e299.x, _e293.y, _e299.z, _e299.w);
                    let _e312 = local_2;
                    let _e315 = (clamp((vec3<f32>(1f, 1f, 1f) - _e312), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e264.w);
                    let _e316 = vec4<f32>(_e305.x, _e305.y, _e293.z, _e305.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e316 / _e315)), sign(_e316), (_e315 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e322 = local_2;
                    local_2 = clamp(_e322, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e325 = clamp(_e266, vec3<f32>(0f, 0f, 0f), _e264.www);
                    let _e331 = vec4<f32>(_e325.x, _e264.y, _e264.z, _e264.w);
                    let _e337 = vec4<f32>(_e331.x, _e325.y, _e331.z, _e331.w);
                    phi_3623_ = vec4<f32>(_e337.x, _e337.y, _e325.z, _e337.w);
                    if (_e264.w == 0f) {
                        phi_3623_ = vec4<f32>(_e325.x, _e325.y, _e325.z, 1f);
                    }
                    let _e347 = phi_3623_;
                    let _e351 = (vec3(_e347.w) - _e347.xyz);
                    let _e352 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e351 / (_e352 * _e347.w))), sign(_e351), (_e352 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e360 = local_2;
                    let _e361 = (_e360 * _e272);
                    local_1 = (select(_e361, (((_e360 + _e272) - _e361) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e360 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_3567_ = 0i;
                    loop {
                        let _e369 = phi_3567_;
                        if (_e369 < 3i) {
                            let _e372 = local_2[_e369];
                            if (_e372 <= 0.5f) {
                                let _e375 = local[_e369];
                                local_1[_e369] = (1f - _e375);
                            } else {
                                let _e379 = local[_e369];
                                if (_e379 <= 0.25f) {
                                    let _e381 = local[_e369];
                                    let _e384 = local[_e369];
                                    local_1[_e369] = ((((16f * _e381) - 12f) * _e384) + 3f);
                                } else {
                                    let _e388 = local[_e369];
                                    local_1[_e369] = (inverseSqrt(_e388) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_3567_ = (_e369 + 1i);
                        }
                    }
                    let _e393 = local_2;
                    let _e397 = local_1;
                    local_1 = (_e272 + ((_e272 * ((_e393 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e397));
                    break;
                }
                case 9: {
                    let _e400 = local_2;
                    local_1 = abs((_e272 - _e400));
                    break;
                }
                case 10: {
                    let _e403 = local_2;
                    local_1 = ((_e403 + _e272) - ((_e403 * 2f) * _e272));
                    break;
                }
                case 12: {
                    if Kh {
                        let _e408 = local_2;
                        let _e409 = clamp(_e408, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e409;
                        let _e424 = (_e409 - vec3(min(min(_e409.x, _e409.y), _e409.z)));
                        let _e432 = (_e424 * ((max(max(_e272.x, _e272.y), _e272.z) - min(min(_e272.x, _e272.y), _e272.z)) / max(0.000062f, max(max(_e424.x, _e424.y), _e424.z))));
                        let _e433 = dot(_e272, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e436 = (_e432 - vec3(dot(_e432, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e449 = (vec2<f32>(_e433, (1f - _e433)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e436.x, _e436.y), _e436.z)), max(max(_e436.x, _e436.y), _e436.z))));
                        local_1 = ((_e436 * min(1f, min(_e449.x, _e449.y))) + vec3(_e433));
                    }
                    break;
                }
                case 13: {
                    if Kh {
                        let _e457 = local_2;
                        let _e458 = clamp(_e457, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e458;
                        let _e473 = (_e272 - vec3(min(min(_e272.x, _e272.y), _e272.z)));
                        let _e481 = (_e473 * ((max(max(_e458.x, _e458.y), _e458.z) - min(min(_e458.x, _e458.y), _e458.z)) / max(0.000062f, max(max(_e473.x, _e473.y), _e473.z))));
                        let _e482 = dot(_e272, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e485 = (_e481 - vec3(dot(_e481, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e498 = (vec2<f32>(_e482, (1f - _e482)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e485.x, _e485.y), _e485.z)), max(max(_e485.x, _e485.y), _e485.z))));
                        local_1 = ((_e485 * min(1f, min(_e498.x, _e498.y))) + vec3(_e482));
                    }
                    break;
                }
                case 14: {
                    if Kh {
                        let _e506 = local_2;
                        let _e507 = clamp(_e506, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e507;
                        let _e508 = dot(_e272, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e511 = (_e507 - vec3(dot(_e507, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e524 = (vec2<f32>(_e508, (1f - _e508)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e511.x, _e511.y), _e511.z)), max(max(_e511.x, _e511.y), _e511.z))));
                        local_1 = ((_e511 * min(1f, min(_e524.x, _e524.y))) + vec3(_e508));
                    }
                    break;
                }
                case 15: {
                    if Kh {
                        let _e532 = local_2;
                        let _e533 = clamp(_e532, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e533;
                        let _e534 = dot(_e533, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e537 = (_e272 - vec3(dot(_e272, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e550 = (vec2<f32>(_e534, (1f - _e534)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e537.x, _e537.y), _e537.z)), max(max(_e537.x, _e537.y), _e537.z))));
                        local_1 = ((_e537 * min(1f, min(_e550.x, _e550.y))) + vec3(_e534));
                    }
                    break;
                }
                default: {
                }
            }
            let _e558 = local_1;
            let _e560 = mix(_e265, _e558, vec3(_e264.w));
            let _e566 = vec4<f32>(_e560.x, _e257.y, _e257.z, _e257.w);
            let _e572 = vec4<f32>(_e566.x, _e560.y, _e566.z, _e566.w);
            phi_3762_ = vec4<f32>(_e572.x, _e572.y, _e560.z, _e572.w);
        }
        let _e580 = phi_3762_;
        let _e583 = (_e580.xyz * _e580.w);
        let _e589 = vec4<f32>(_e583.x, _e580.y, _e580.z, _e580.w);
        let _e595 = vec4<f32>(_e589.x, _e583.y, _e589.z, _e589.w);
        phi_3775_ = vec4<f32>(_e595.x, _e595.y, _e583.z, _e595.w);
    }
    let _e603 = phi_3775_;
    let _e604 = (_e603 * _e186);
    let _e606 = (1f - _e604.w);
    phi_3776_ = _e604;
    if (_e606 != 0f) {
        let _e610 = l0_.g2_[_e102];
        phi_3776_ = (_e604 + (unpack4x8unorm(_e610) * _e606));
    }
    let _e615 = phi_3776_;
    E1_ = _e615;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = E1_;
    return _e3;
}
