struct hf {
    e2_: array<vec2<u32>>,
}

struct h0Qd {
    e2_: array<u32>,
}

struct jf {
    e2_: array<vec4<f32>>,
}

struct k0Qd {
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

struct x4Qd {
    e2_: array<u32>,
}

@id(7) override Ih: bool = true;
@id(6) override Hh: bool = true;
@id(4) override Fh: bool = true;
@id(0) override Bh: bool = true;
@id(1) override Ch: bool = true;
@id(2) override Dh: bool = true;

@group(0) @binding(3)
var<storage> DD: hf;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Qd;
@group(0) @binding(4)
var<storage> QB: jf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(2) @binding(0)
var<storage, read_write> k0_: k0Qd;
@group(0) @binding(0)
var<uniform> n: BC;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var W5_: sampler;
var<private> Z1_1: vec2<f32>;
var<private> V4_1: f32;
var<private> M0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> x4_: x4Qd;
var<private> x3_1: u32;
var<private> M5_1: vec4<f32>;
var<private> H1_1: vec4<f32>;
var<private> A1_1: u32;
@group(3) @binding(9)
var ea: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var local_3: vec3<f32>;
    var local_4: vec3<f32>;
    var local_5: vec3<f32>;
    var phi_6059_: f32;
    var phi_1616_: bool;
    var phi_5329_: f32;
    var phi_5328_: f32;
    var phi_5330_: f32;
    var phi_5333_: f32;
    var phi_5332_: f32;
    var phi_1653_: bool;
    var phi_5335_: f32;
    var phi_6026_: u32;
    var phi_5334_: f32;
    var phi_6025_: u32;
    var phi_5359_: vec4<f32>;
    var phi_1772_: bool;
    var phi_5363_: u32;
    var phi_1781_: bool;
    var phi_5378_: f32;
    var phi_5864_: vec4<f32>;
    var phi_5800_: i32;
    var phi_6021_: vec4<f32>;
    var phi_1325_: bool;
    var phi_6047_: u32;
    var phi_6075_: f32;
    var phi_7586_: f32;
    var phi_6076_: f32;
    var phi_6077_: f32;
    var phi_6109_: vec4<f32>;
    var phi_1387_: bool;
    var phi_6119_: f32;
    var phi_6120_: f32;
    var phi_7245_: vec4<f32>;
    var phi_7101_: i32;
    var phi_7600_: vec4<f32>;
    var phi_7613_: vec3<f32>;
    var phi_7615_: vec4<f32>;

    let _e87 = gl_FragCoord_1;
    let _e88 = _e87.xy;
    let _e91 = bitcast<vec2<u32>>(vec2<i32>(floor(_e88)));
    let _e93 = n.q6_;
    let _e122 = bitcast<i32>((((((_e91.y >> bitcast<u32>(5u)) * (((_e93 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e91.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e91.x & 28u) << bitcast<u32>(5u)) + ((_e91.y & 28u) << bitcast<u32>(2i)))) + (((_e91.y & 3u) << bitcast<u32>(2i)) + (_e91.x & 3u))));
    let _e123 = Z1_1;
    let _e124 = textureSample(HC, W5_, _e123);
    let _e125 = V4_1;
    let _e126 = min(_e125, 1f);
    phi_6059_ = _e126;
    if Ch {
        let _e127 = M0_1;
        let _e130 = min(_e127.xy, _e127.zw);
        phi_6059_ = clamp(min(_e130.x, _e130.y), 0f, _e126);
    }
    let _e136 = phi_6059_;
    let _e139 = x4_.e2_[_e122];
    let _e141 = (_e139 >> bitcast<u32>(17u));
    let _e145 = ((f32((_e139 & 131071u)) * 0.00048828125f) + -32f);
    let _e148 = DD.e2_[_e141];
    phi_5328_ = _e145;
    if ((_e148.x & 768u) != 0u) {
        let _e152 = abs(_e145);
        phi_1616_ = Fh;
        if Fh {
            phi_1616_ = ((_e148.x & 512u) != 0u);
        }
        let _e156 = phi_1616_;
        phi_5329_ = _e152;
        if _e156 {
            phi_5329_ = (1f - abs(((fract((_e152 * 0.5f)) * 2f) + -1f)));
        }
        let _e164 = phi_5329_;
        phi_5328_ = _e164;
    }
    let _e166 = phi_5328_;
    let _e167 = clamp(_e166, 0f, 1f);
    phi_5332_ = _e167;
    if Bh {
        let _e169 = (_e148.x >> bitcast<u32>(16u));
        phi_5333_ = _e167;
        if (_e169 != 0u) {
            let _e173 = h0_.e2_[_e122];
            if (_e169 == (_e173 >> bitcast<u32>(16i))) {
                phi_5330_ = min(_e167, unpack2x16float(_e173).x);
            } else {
                phi_5330_ = 0f;
            }
            let _e181 = phi_5330_;
            phi_5333_ = _e181;
        }
        let _e183 = phi_5333_;
        phi_5332_ = _e183;
    }
    let _e185 = phi_5332_;
    phi_1653_ = Ch;
    if Ch {
        phi_1653_ = ((_e148.x & 1024u) != 0u);
    }
    let _e189 = phi_1653_;
    phi_5335_ = _e185;
    if _e189 {
        let _e190 = (_e141 * 8u);
        let _e194 = QB.e2_[(_e190 + 2u)];
        let _e205 = QB.e2_[(_e190 + 3u)];
        let _e210 = _e205.zw;
        let _e212 = ((abs(((mat2x2<f32>(vec2<f32>(_e194.x, _e194.y), vec2<f32>(_e194.z, _e194.w)) * _e88) + _e205.xy)) * _e210) - _e210);
        phi_5335_ = min(_e185, clamp((min(_e212.x, _e212.y) + 0.5f), 0f, 1f));
    }
    let _e220 = phi_5335_;
    let _e221 = (_e148.x & 15u);
    if (_e221 <= 1u) {
        let _e226 = (Bh && (_e221 == 0u));
        phi_6026_ = 0u;
        if _e226 {
            phi_6026_ = (_e148.y | pack2x16float(vec2<f32>(_e220, 0f)));
        }
        let _e231 = phi_6026_;
        phi_6025_ = _e231;
        phi_5359_ = select(unpack4x8unorm(_e148.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e226));
    } else {
        let _e234 = (_e141 * 8u);
        let _e237 = QB.e2_[_e234];
        let _e248 = QB.e2_[(_e234 + 1u)];
        let _e251 = ((mat2x2<f32>(vec2<f32>(_e237.x, _e237.y), vec2<f32>(_e237.z, _e237.w)) * _e88) + _e248.xy);
        if (_e221 == 2u) {
            phi_5334_ = _e251.x;
        } else {
            phi_5334_ = length(_e251);
        }
        let _e256 = phi_5334_;
        let _e265 = textureSampleLevel(ED, N9_, vec2<f32>(((clamp(_e256, 0f, 1f) * _e248.z) + _e248.w), bitcast<f32>(_e148.y)), 0f);
        phi_6025_ = 0u;
        phi_5359_ = _e265;
    }
    let _e267 = phi_6025_;
    let _e269 = phi_5359_;
    let _e271 = (_e269.w * _e220);
    let _e276 = vec4<f32>(_e269.x, _e269.y, _e269.z, _e271);
    phi_1772_ = Dh;
    if Dh {
        phi_1772_ = (_e271 != 0f);
    }
    let _e279 = phi_1772_;
    phi_5363_ = u32();
    phi_1781_ = _e279;
    if _e279 {
        let _e282 = ((_e148.x >> bitcast<u32>(4i)) & 15u);
        phi_5363_ = _e282;
        phi_1781_ = (_e282 != 0u);
    }
    let _e285 = phi_5363_;
    let _e287 = phi_1781_;
    phi_6021_ = _e276;
    if _e287 {
        let _e290 = k0_.e2_[_e122];
        let _e291 = unpack4x8unorm(_e290);
        let _e292 = _e276.xyz;
        local_5 = _e292;
        let _e293 = _e291.xyz;
        if (_e291.w != 0f) {
            phi_5378_ = (1f / _e291.w);
        } else {
            phi_5378_ = 0f;
        }
        let _e298 = phi_5378_;
        let _e299 = (_e293 * _e298);
        local_3 = _e299;
        switch bitcast<i32>(_e285) {
            case 11: {
                let _e301 = local_5;
                local_4 = (_e301 * _e299);
                break;
            }
            case 1: {
                let _e303 = local_5;
                local_4 = ((_e303 + _e299) - (_e303 * _e299));
                break;
            }
            case 2: {
                let _e307 = local_5;
                let _e308 = (_e307 * _e299);
                local_4 = (select(_e308, (((_e307 + _e299) - _e308) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e299 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e315 = local_5;
                local_4 = min(_e315, _e299);
                break;
            }
            case 4: {
                let _e317 = local_5;
                local_4 = max(_e317, _e299);
                break;
            }
            case 5: {
                let _e320 = clamp(_e293, vec3<f32>(0f, 0f, 0f), _e291.www);
                let _e326 = vec4<f32>(_e320.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e332 = vec4<f32>(_e326.x, _e320.y, _e326.z, _e326.w);
                let _e339 = local_5;
                let _e342 = (clamp((vec3<f32>(1f, 1f, 1f) - _e339), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e291.w);
                let _e343 = vec4<f32>(_e332.x, _e332.y, _e320.z, _e332.w).xyz;
                local_4 = select(min(vec3<f32>(1f, 1f, 1f), (_e343 / _e342)), sign(_e343), (_e342 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e349 = local_5;
                local_5 = clamp(_e349, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e352 = clamp(_e293, vec3<f32>(0f, 0f, 0f), _e291.www);
                let _e358 = vec4<f32>(_e352.x, _e291.y, _e291.z, _e291.w);
                let _e364 = vec4<f32>(_e358.x, _e352.y, _e358.z, _e358.w);
                phi_5864_ = vec4<f32>(_e364.x, _e364.y, _e352.z, _e364.w);
                if (_e291.w == 0f) {
                    phi_5864_ = vec4<f32>(_e352.x, _e352.y, _e352.z, 1f);
                }
                let _e374 = phi_5864_;
                let _e378 = (vec3(_e374.w) - _e374.xyz);
                let _e379 = local_5;
                local_4 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e378 / (_e379 * _e374.w))), sign(_e378), (_e379 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e387 = local_5;
                let _e388 = (_e387 * _e299);
                local_4 = (select(_e388, (((_e387 + _e299) - _e388) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e387 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_5800_ = 0i;
                loop {
                    let _e396 = phi_5800_;
                    if (_e396 < 3i) {
                        let _e399 = local_5[_e396];
                        if (_e399 <= 0.5f) {
                            let _e402 = local_3[_e396];
                            local_4[_e396] = (1f - _e402);
                        } else {
                            let _e406 = local_3[_e396];
                            if (_e406 <= 0.25f) {
                                let _e408 = local_3[_e396];
                                let _e411 = local_3[_e396];
                                local_4[_e396] = ((((16f * _e408) - 12f) * _e411) + 3f);
                            } else {
                                let _e415 = local_3[_e396];
                                local_4[_e396] = (inverseSqrt(_e415) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_5800_ = (_e396 + 1i);
                    }
                }
                let _e420 = local_5;
                let _e424 = local_4;
                local_4 = (_e299 + ((_e299 * ((_e420 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e424));
                break;
            }
            case 9: {
                let _e427 = local_5;
                local_4 = abs((_e299 - _e427));
                break;
            }
            case 10: {
                let _e430 = local_5;
                local_4 = ((_e430 + _e299) - ((_e430 * 2f) * _e299));
                break;
            }
            case 12: {
                if Hh {
                    let _e435 = local_5;
                    let _e436 = clamp(_e435, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_5 = _e436;
                    let _e451 = (_e436 - vec3(min(min(_e436.x, _e436.y), _e436.z)));
                    let _e459 = (_e451 * ((max(max(_e299.x, _e299.y), _e299.z) - min(min(_e299.x, _e299.y), _e299.z)) / max(0.000062f, max(max(_e451.x, _e451.y), _e451.z))));
                    let _e460 = dot(_e299, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e463 = (_e459 - vec3(dot(_e459, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e476 = (vec2<f32>(_e460, (1f - _e460)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e463.x, _e463.y), _e463.z)), max(max(_e463.x, _e463.y), _e463.z))));
                    local_4 = ((_e463 * min(1f, min(_e476.x, _e476.y))) + vec3(_e460));
                }
                break;
            }
            case 13: {
                if Hh {
                    let _e484 = local_5;
                    let _e485 = clamp(_e484, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_5 = _e485;
                    let _e500 = (_e299 - vec3(min(min(_e299.x, _e299.y), _e299.z)));
                    let _e508 = (_e500 * ((max(max(_e485.x, _e485.y), _e485.z) - min(min(_e485.x, _e485.y), _e485.z)) / max(0.000062f, max(max(_e500.x, _e500.y), _e500.z))));
                    let _e509 = dot(_e299, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e512 = (_e508 - vec3(dot(_e508, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e525 = (vec2<f32>(_e509, (1f - _e509)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e512.x, _e512.y), _e512.z)), max(max(_e512.x, _e512.y), _e512.z))));
                    local_4 = ((_e512 * min(1f, min(_e525.x, _e525.y))) + vec3(_e509));
                }
                break;
            }
            case 14: {
                if Hh {
                    let _e533 = local_5;
                    let _e534 = clamp(_e533, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_5 = _e534;
                    let _e535 = dot(_e299, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e538 = (_e534 - vec3(dot(_e534, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e551 = (vec2<f32>(_e535, (1f - _e535)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e538.x, _e538.y), _e538.z)), max(max(_e538.x, _e538.y), _e538.z))));
                    local_4 = ((_e538 * min(1f, min(_e551.x, _e551.y))) + vec3(_e535));
                }
                break;
            }
            case 15: {
                if Hh {
                    let _e559 = local_5;
                    let _e560 = clamp(_e559, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_5 = _e560;
                    let _e561 = dot(_e560, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e564 = (_e299 - vec3(dot(_e299, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e577 = (vec2<f32>(_e561, (1f - _e561)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e564.x, _e564.y), _e564.z)), max(max(_e564.x, _e564.y), _e564.z))));
                    local_4 = ((_e564 * min(1f, min(_e577.x, _e577.y))) + vec3(_e561));
                }
                break;
            }
            default: {
            }
        }
        let _e585 = local_4;
        let _e587 = mix(_e292, _e585, vec3(_e291.w));
        phi_6021_ = vec4<f32>(_e587.x, _e587.y, _e587.z, _e271);
    }
    let _e593 = phi_6021_;
    let _e596 = (_e593.xyz * _e593.w);
    let _e602 = vec4<f32>(_e596.x, _e593.y, _e593.z, _e593.w);
    let _e608 = vec4<f32>(_e602.x, _e596.y, _e602.z, _e602.w);
    let _e614 = vec4<f32>(_e608.x, _e608.y, _e596.z, _e608.w);
    phi_1325_ = Bh;
    if Bh {
        let _e615 = x3_1;
        phi_1325_ = (_e615 != 0u);
    }
    let _e618 = phi_1325_;
    phi_7586_ = _e136;
    if _e618 {
        if (_e267 != 0u) {
            phi_6047_ = _e267;
        } else {
            let _e622 = h0_.e2_[_e122];
            phi_6047_ = _e622;
        }
        let _e624 = phi_6047_;
        let _e625 = x3_1;
        if (_e625 == (_e624 >> bitcast<u32>(16i))) {
            phi_6075_ = min(_e136, unpack2x16float(_e624).x);
        } else {
            phi_6075_ = 0f;
        }
        let _e633 = phi_6075_;
        phi_7586_ = _e633;
    }
    let _e635 = phi_7586_;
    let _e637 = M5_1[3u];
    phi_6109_ = _e124;
    if (_e637 != 0f) {
        let _e639 = M5_1;
        if (_e639.z > 0f) {
            phi_6076_ = _e639.x;
        } else {
            phi_6076_ = length(_e639.xy);
        }
        let _e646 = phi_6076_;
        let _e647 = clamp(_e646, 0f, 1f);
        let _e648 = abs(_e639.z);
        if (_e648 > 1f) {
            phi_6077_ = ((0.9980469f * _e647) + 0.0009765625f);
        } else {
            phi_6077_ = ((0.001953125f * _e647) + _e648);
        }
        let _e655 = phi_6077_;
        let _e658 = textureSampleLevel(ED, N9_, vec2<f32>(_e655, _e639.w), 0f);
        let _e661 = (_e658.xyz * _e658.w);
        let _e667 = vec4<f32>(_e661.x, _e658.y, _e658.z, _e658.w);
        let _e673 = vec4<f32>(_e667.x, _e661.y, _e667.z, _e667.w);
        phi_6109_ = (_e124 * vec4<f32>(_e673.x, _e673.y, _e661.z, _e673.w));
    }
    let _e682 = phi_6109_;
    let _e683 = H1_1;
    let _e684 = (_e682 * _e683);
    phi_1387_ = Dh;
    if Dh {
        let _e685 = A1_1;
        phi_1387_ = (_e685 != 0u);
    }
    let _e688 = phi_1387_;
    phi_7600_ = _e684;
    if _e688 {
        let _e691 = k0_.e2_[_e122];
        let _e695 = ((unpack4x8unorm(_e691) * (1f - _e593.w)) + _e614);
        if (_e684.w != 0f) {
            phi_6119_ = (1f / _e684.w);
        } else {
            phi_6119_ = 0f;
        }
        let _e701 = phi_6119_;
        let _e702 = (_e684.xyz * _e701);
        let _e703 = A1_1;
        local_2 = _e702;
        let _e704 = _e695.xyz;
        if (_e695.w != 0f) {
            phi_6120_ = (1f / _e695.w);
        } else {
            phi_6120_ = 0f;
        }
        let _e709 = phi_6120_;
        let _e710 = (_e704 * _e709);
        local = _e710;
        switch bitcast<i32>(_e703) {
            case 11: {
                let _e712 = local_2;
                local_1 = (_e712 * _e710);
                break;
            }
            case 1: {
                let _e714 = local_2;
                local_1 = ((_e714 + _e710) - (_e714 * _e710));
                break;
            }
            case 2: {
                let _e718 = local_2;
                let _e719 = (_e718 * _e710);
                local_1 = (select(_e719, (((_e718 + _e710) - _e719) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e710 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e726 = local_2;
                local_1 = min(_e726, _e710);
                break;
            }
            case 4: {
                let _e728 = local_2;
                local_1 = max(_e728, _e710);
                break;
            }
            case 5: {
                let _e731 = clamp(_e704, vec3<f32>(0f, 0f, 0f), _e695.www);
                let _e737 = vec4<f32>(_e731.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e743 = vec4<f32>(_e737.x, _e731.y, _e737.z, _e737.w);
                let _e750 = local_2;
                let _e753 = (clamp((vec3<f32>(1f, 1f, 1f) - _e750), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e695.w);
                let _e754 = vec4<f32>(_e743.x, _e743.y, _e731.z, _e743.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e754 / _e753)), sign(_e754), (_e753 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e760 = local_2;
                local_2 = clamp(_e760, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e763 = clamp(_e704, vec3<f32>(0f, 0f, 0f), _e695.www);
                let _e769 = vec4<f32>(_e763.x, _e695.y, _e695.z, _e695.w);
                let _e775 = vec4<f32>(_e769.x, _e763.y, _e769.z, _e769.w);
                phi_7245_ = vec4<f32>(_e775.x, _e775.y, _e763.z, _e775.w);
                if (_e695.w == 0f) {
                    phi_7245_ = vec4<f32>(_e763.x, _e763.y, _e763.z, 1f);
                }
                let _e785 = phi_7245_;
                let _e789 = (vec3(_e785.w) - _e785.xyz);
                let _e790 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e789 / (_e790 * _e785.w))), sign(_e789), (_e790 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e798 = local_2;
                let _e799 = (_e798 * _e710);
                local_1 = (select(_e799, (((_e798 + _e710) - _e799) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e798 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_7101_ = 0i;
                loop {
                    let _e807 = phi_7101_;
                    if (_e807 < 3i) {
                        let _e810 = local_2[_e807];
                        if (_e810 <= 0.5f) {
                            let _e813 = local[_e807];
                            local_1[_e807] = (1f - _e813);
                        } else {
                            let _e817 = local[_e807];
                            if (_e817 <= 0.25f) {
                                let _e819 = local[_e807];
                                let _e822 = local[_e807];
                                local_1[_e807] = ((((16f * _e819) - 12f) * _e822) + 3f);
                            } else {
                                let _e826 = local[_e807];
                                local_1[_e807] = (inverseSqrt(_e826) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_7101_ = (_e807 + 1i);
                    }
                }
                let _e831 = local_2;
                let _e835 = local_1;
                local_1 = (_e710 + ((_e710 * ((_e831 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e835));
                break;
            }
            case 9: {
                let _e838 = local_2;
                local_1 = abs((_e710 - _e838));
                break;
            }
            case 10: {
                let _e841 = local_2;
                local_1 = ((_e841 + _e710) - ((_e841 * 2f) * _e710));
                break;
            }
            case 12: {
                if Hh {
                    let _e846 = local_2;
                    let _e847 = clamp(_e846, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e847;
                    let _e862 = (_e847 - vec3(min(min(_e847.x, _e847.y), _e847.z)));
                    let _e870 = (_e862 * ((max(max(_e710.x, _e710.y), _e710.z) - min(min(_e710.x, _e710.y), _e710.z)) / max(0.000062f, max(max(_e862.x, _e862.y), _e862.z))));
                    let _e871 = dot(_e710, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e874 = (_e870 - vec3(dot(_e870, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e887 = (vec2<f32>(_e871, (1f - _e871)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e874.x, _e874.y), _e874.z)), max(max(_e874.x, _e874.y), _e874.z))));
                    local_1 = ((_e874 * min(1f, min(_e887.x, _e887.y))) + vec3(_e871));
                }
                break;
            }
            case 13: {
                if Hh {
                    let _e895 = local_2;
                    let _e896 = clamp(_e895, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e896;
                    let _e911 = (_e710 - vec3(min(min(_e710.x, _e710.y), _e710.z)));
                    let _e919 = (_e911 * ((max(max(_e896.x, _e896.y), _e896.z) - min(min(_e896.x, _e896.y), _e896.z)) / max(0.000062f, max(max(_e911.x, _e911.y), _e911.z))));
                    let _e920 = dot(_e710, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e923 = (_e919 - vec3(dot(_e919, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e936 = (vec2<f32>(_e920, (1f - _e920)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e923.x, _e923.y), _e923.z)), max(max(_e923.x, _e923.y), _e923.z))));
                    local_1 = ((_e923 * min(1f, min(_e936.x, _e936.y))) + vec3(_e920));
                }
                break;
            }
            case 14: {
                if Hh {
                    let _e944 = local_2;
                    let _e945 = clamp(_e944, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e945;
                    let _e946 = dot(_e710, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e949 = (_e945 - vec3(dot(_e945, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e962 = (vec2<f32>(_e946, (1f - _e946)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e949.x, _e949.y), _e949.z)), max(max(_e949.x, _e949.y), _e949.z))));
                    local_1 = ((_e949 * min(1f, min(_e962.x, _e962.y))) + vec3(_e946));
                }
                break;
            }
            case 15: {
                if Hh {
                    let _e970 = local_2;
                    let _e971 = clamp(_e970, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e971;
                    let _e972 = dot(_e971, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e975 = (_e710 - vec3(dot(_e710, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e988 = (vec2<f32>(_e972, (1f - _e972)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e975.x, _e975.y), _e975.z)), max(max(_e975.x, _e975.y), _e975.z))));
                    local_1 = ((_e975 * min(1f, min(_e988.x, _e988.y))) + vec3(_e972));
                }
                break;
            }
            default: {
            }
        }
        let _e996 = local_1;
        let _e999 = (mix(_e702, _e996, vec3(_e695.w)) * _e684.w);
        let _e1005 = vec4<f32>(_e999.x, _e684.y, _e684.z, _e684.w);
        let _e1011 = vec4<f32>(_e1005.x, _e999.y, _e1005.z, _e1005.w);
        phi_7600_ = vec4<f32>(_e1011.x, _e1011.y, _e999.z, _e1011.w);
    }
    let _e1019 = phi_7600_;
    let _e1020 = (_e1019 * _e635);
    let _e1024 = ((_e614 * (1f - _e1020.w)) + _e1020);
    let _e1025 = _e1024.xyz;
    let _e1028 = n.B3_;
    let _e1030 = n.C3_;
    if (Ih && (_e1024.w != 0f)) {
        phi_7613_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e87.x) + (0.00583715f * _e87.y))))) * _e1028) + _e1030)) + _e1025);
    } else {
        phi_7613_ = _e1025;
    }
    let _e1046 = phi_7613_;
    let _e1052 = vec4<f32>(_e1046.x, _e1024.y, _e1024.z, _e1024.w);
    let _e1058 = vec4<f32>(_e1052.x, _e1046.y, _e1052.z, _e1052.w);
    let _e1064 = vec4<f32>(_e1058.x, _e1058.y, _e1046.z, _e1058.w);
    switch bitcast<i32>(0u) {
        default: {
            if (_e1024.w == 0f) {
                break;
            }
            let _e1067 = (1f - _e1024.w);
            phi_7615_ = _e1064;
            if (_e1067 != 0f) {
                let _e1071 = k0_.e2_[_e122];
                phi_7615_ = (_e1064 + (unpack4x8unorm(_e1071) * _e1067));
            }
            let _e1076 = phi_7615_;
            k0_.e2_[_e122] = pack4x8unorm(_e1076);
            break;
        }
    }
    if (_e267 != 0u) {
        h0_.e2_[_e122] = _e267;
    }
    x4_.e2_[_e122] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) Z1_: vec2<f32>, @location(1) V4_: f32, @location(3) M0_: vec4<f32>, @location(5) @interpolate(flat, either) x3_: u32, @location(2) M5_: vec4<f32>, @location(4) @interpolate(flat, either) H1_: vec4<f32>, @location(6) @interpolate(flat, either) A1_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    Z1_1 = Z1_;
    V4_1 = V4_;
    M0_1 = M0_;
    x3_1 = x3_;
    M5_1 = M5_;
    H1_1 = H1_;
    A1_1 = A1_;
    main_1();
}
