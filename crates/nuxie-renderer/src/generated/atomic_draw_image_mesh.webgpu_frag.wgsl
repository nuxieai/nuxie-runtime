struct Be {
    X1_: array<vec2<u32>>,
}

struct d0qd {
    X1_: array<u32>,
}

struct Ce {
    X1_: array<vec4<f32>>,
}

struct g0qd {
    X1_: array<u32>,
}

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

struct p4qd {
    X1_: array<u32>,
}

struct LC {
    r9_: vec4<f32>,
    c2_: vec2<f32>,
    x4_: f32,
    ki: f32,
    k2_: vec4<f32>,
    D2_: vec2<f32>,
    V0_: u32,
    n2_: u32,
    Z6_: u32,
}

@id(7) override Tg: bool = true;
@id(6) override Sg: bool = true;
@id(4) override Qg: bool = true;
@id(0) override Mg: bool = true;
@id(1) override Ng: bool = true;
@id(2) override Og: bool = true;

@group(0) @binding(4) 
var<storage> TC: Be;
@group(2) @binding(1) 
var<storage, read_write> d0_: d0qd;
@group(0) @binding(5) 
var<storage> PB: Ce;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(9) 
var DD: texture_2d<f32>;
@group(3) @binding(9) 
var Bb: sampler;
@group(2) @binding(0) 
var<storage, read_write> g0_: g0qd;
@group(0) @binding(0) 
var<uniform> k: NB;
@group(1) @binding(12) 
var AC: texture_2d<f32>;
@group(1) @binding(14) 
var R5_: sampler;
var<private> U0_1: vec2<f32>;
var<private> N0_1: vec4<f32>;
@group(2) @binding(3) 
var<storage, read_write> p4_: p4qd;
@group(0) @binding(2) 
var<uniform> A0_: LC;
@group(3) @binding(10) 
var T9_: sampler;
@group(0) @binding(10) 
var QC: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var local_3: vec3<f32>;
    var local_4: vec3<f32>;
    var local_5: vec3<f32>;
    var phi_5926_: f32;
    var phi_1531_: bool;
    var phi_5196_: f32;
    var phi_5195_: f32;
    var phi_5197_: f32;
    var phi_5200_: f32;
    var phi_5199_: f32;
    var phi_1568_: bool;
    var phi_5202_: f32;
    var phi_5893_: u32;
    var phi_5201_: f32;
    var phi_5892_: u32;
    var phi_5226_: vec4<f32>;
    var phi_1687_: bool;
    var phi_5230_: u32;
    var phi_1696_: bool;
    var phi_5245_: f32;
    var phi_5731_: vec4<f32>;
    var phi_5667_: i32;
    var phi_5888_: vec4<f32>;
    var phi_1269_: bool;
    var phi_5914_: u32;
    var phi_5942_: f32;
    var phi_7321_: f32;
    var phi_1299_: bool;
    var phi_5978_: f32;
    var phi_5979_: f32;
    var phi_7008_: vec4<f32>;
    var phi_6876_: i32;
    var phi_7333_: vec4<f32>;
    var phi_7346_: vec3<f32>;
    var phi_7348_: vec4<f32>;

    let _e82 = gl_FragCoord_1;
    let _e83 = _e82.xy;
    let _e86 = bitcast<vec2<u32>>(vec2<i32>(floor(_e83)));
    let _e88 = k.q5_;
    let _e117 = bitcast<i32>((((((_e86.y >> bitcast<u32>(5u)) * (((_e88 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e86.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e86.x & 28u) << bitcast<u32>(5u)) + ((_e86.y & 28u) << bitcast<u32>(2i)))) + (((_e86.y & 3u) << bitcast<u32>(2i)) + (_e86.x & 3u))));
    let _e118 = U0_1;
    let _e119 = textureSample(AC, R5_, _e118);
    phi_5926_ = 1f;
    if Ng {
        let _e120 = N0_1;
        let _e123 = min(_e120.xy, _e120.zw);
        phi_5926_ = clamp(min(_e123.x, _e123.y), 0f, 1f);
    }
    let _e129 = phi_5926_;
    let _e132 = p4_.X1_[_e117];
    let _e134 = (_e132 >> bitcast<u32>(17u));
    let _e138 = ((f32((_e132 & 131071u)) * 0.00048828125f) + -32f);
    let _e141 = TC.X1_[_e134];
    phi_5195_ = _e138;
    if ((_e141.x & 768u) != 0u) {
        let _e145 = abs(_e138);
        phi_1531_ = Qg;
        if Qg {
            phi_1531_ = ((_e141.x & 512u) != 0u);
        }
        let _e149 = phi_1531_;
        phi_5196_ = _e145;
        if _e149 {
            phi_5196_ = (1f - abs(((fract((_e145 * 0.5f)) * 2f) + -1f)));
        }
        let _e157 = phi_5196_;
        phi_5195_ = _e157;
    }
    let _e159 = phi_5195_;
    let _e160 = clamp(_e159, 0f, 1f);
    phi_5199_ = _e160;
    if Mg {
        let _e162 = (_e141.x >> bitcast<u32>(16u));
        phi_5200_ = _e160;
        if (_e162 != 0u) {
            let _e166 = d0_.X1_[_e117];
            if (_e162 == (_e166 >> bitcast<u32>(16i))) {
                phi_5197_ = min(_e160, unpack2x16float(_e166).x);
            } else {
                phi_5197_ = 0f;
            }
            let _e174 = phi_5197_;
            phi_5200_ = _e174;
        }
        let _e176 = phi_5200_;
        phi_5199_ = _e176;
    }
    let _e178 = phi_5199_;
    phi_1568_ = Ng;
    if Ng {
        phi_1568_ = ((_e141.x & 1024u) != 0u);
    }
    let _e182 = phi_1568_;
    phi_5202_ = _e178;
    if _e182 {
        let _e183 = (_e134 * 4u);
        let _e187 = PB.X1_[(_e183 + 2u)];
        let _e198 = PB.X1_[(_e183 + 3u)];
        let _e203 = _e198.zw;
        let _e205 = ((abs(((mat2x2<f32>(vec2<f32>(_e187.x, _e187.y), vec2<f32>(_e187.z, _e187.w)) * _e83) + _e198.xy)) * _e203) - _e203);
        phi_5202_ = min(_e178, clamp((min(_e205.x, _e205.y) + 0.5f), 0f, 1f));
    }
    let _e213 = phi_5202_;
    let _e214 = (_e141.x & 15u);
    if (_e214 <= 1u) {
        let _e219 = (Mg && (_e214 == 0u));
        phi_5893_ = 0u;
        if _e219 {
            phi_5893_ = (_e141.y | pack2x16float(vec2<f32>(_e213, 0f)));
        }
        let _e224 = phi_5893_;
        phi_5892_ = _e224;
        phi_5226_ = select(unpack4x8unorm(_e141.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e219));
    } else {
        let _e227 = (_e134 * 4u);
        let _e230 = PB.X1_[_e227];
        let _e241 = PB.X1_[(_e227 + 1u)];
        let _e244 = ((mat2x2<f32>(vec2<f32>(_e230.x, _e230.y), vec2<f32>(_e230.z, _e230.w)) * _e83) + _e241.xy);
        if (_e214 == 2u) {
            phi_5201_ = _e244.x;
        } else {
            phi_5201_ = length(_e244);
        }
        let _e249 = phi_5201_;
        let _e258 = textureSampleLevel(DD, Bb, vec2<f32>(((clamp(_e249, 0f, 1f) * _e241.z) + _e241.w), bitcast<f32>(_e141.y)), 0f);
        phi_5892_ = 0u;
        phi_5226_ = _e258;
    }
    let _e260 = phi_5892_;
    let _e262 = phi_5226_;
    let _e264 = (_e262.w * _e213);
    let _e269 = vec4<f32>(_e262.x, _e262.y, _e262.z, _e264);
    phi_1687_ = Og;
    if Og {
        phi_1687_ = (_e264 != 0f);
    }
    let _e272 = phi_1687_;
    phi_5230_ = u32();
    phi_1696_ = _e272;
    if _e272 {
        let _e275 = ((_e141.x >> bitcast<u32>(4i)) & 15u);
        phi_5230_ = _e275;
        phi_1696_ = (_e275 != 0u);
    }
    let _e278 = phi_5230_;
    let _e280 = phi_1696_;
    phi_5888_ = _e269;
    if _e280 {
        let _e283 = g0_.X1_[_e117];
        let _e284 = unpack4x8unorm(_e283);
        let _e285 = _e269.xyz;
        local_5 = _e285;
        let _e286 = _e284.xyz;
        if (_e284.w != 0f) {
            phi_5245_ = (1f / _e284.w);
        } else {
            phi_5245_ = 0f;
        }
        let _e291 = phi_5245_;
        let _e292 = (_e286 * _e291);
        local_3 = _e292;
        switch bitcast<i32>(_e278) {
            case 11: {
                let _e294 = local_5;
                local_4 = (_e294 * _e292);
                break;
            }
            case 1: {
                let _e296 = local_5;
                local_4 = ((_e296 + _e292) - (_e296 * _e292));
                break;
            }
            case 2: {
                let _e300 = local_5;
                let _e301 = (_e300 * _e292);
                local_4 = (select(_e301, (((_e300 + _e292) - _e301) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e292 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e308 = local_5;
                local_4 = min(_e308, _e292);
                break;
            }
            case 4: {
                let _e310 = local_5;
                local_4 = max(_e310, _e292);
                break;
            }
            case 5: {
                let _e313 = clamp(_e286, vec3<f32>(0f, 0f, 0f), _e284.www);
                let _e319 = vec4<f32>(_e313.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e325 = vec4<f32>(_e319.x, _e313.y, _e319.z, _e319.w);
                let _e332 = local_5;
                let _e335 = (clamp((vec3<f32>(1f, 1f, 1f) - _e332), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e284.w);
                let _e336 = vec4<f32>(_e325.x, _e325.y, _e313.z, _e325.w).xyz;
                local_4 = select(min(vec3<f32>(1f, 1f, 1f), (_e336 / _e335)), sign(_e336), (_e335 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e342 = local_5;
                local_5 = clamp(_e342, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e345 = clamp(_e286, vec3<f32>(0f, 0f, 0f), _e284.www);
                let _e351 = vec4<f32>(_e345.x, _e284.y, _e284.z, _e284.w);
                let _e357 = vec4<f32>(_e351.x, _e345.y, _e351.z, _e351.w);
                phi_5731_ = vec4<f32>(_e357.x, _e357.y, _e345.z, _e357.w);
                if (_e284.w == 0f) {
                    phi_5731_ = vec4<f32>(_e345.x, _e345.y, _e345.z, 1f);
                }
                let _e367 = phi_5731_;
                let _e371 = (vec3(_e367.w) - _e367.xyz);
                let _e372 = local_5;
                local_4 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e371 / (_e372 * _e367.w))), sign(_e371), (_e372 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e380 = local_5;
                let _e381 = (_e380 * _e292);
                local_4 = (select(_e381, (((_e380 + _e292) - _e381) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e380 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_5667_ = 0i;
                loop {
                    let _e389 = phi_5667_;
                    if (_e389 < 3i) {
                        let _e392 = local_5[_e389];
                        if (_e392 <= 0.5f) {
                            let _e395 = local_3[_e389];
                            local_4[_e389] = (1f - _e395);
                        } else {
                            let _e399 = local_3[_e389];
                            if (_e399 <= 0.25f) {
                                let _e401 = local_3[_e389];
                                let _e404 = local_3[_e389];
                                local_4[_e389] = ((((16f * _e401) - 12f) * _e404) + 3f);
                            } else {
                                let _e408 = local_3[_e389];
                                local_4[_e389] = (inverseSqrt(_e408) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_5667_ = (_e389 + 1i);
                    }
                }
                let _e413 = local_5;
                let _e417 = local_4;
                local_4 = (_e292 + ((_e292 * ((_e413 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e417));
                break;
            }
            case 9: {
                let _e420 = local_5;
                local_4 = abs((_e292 - _e420));
                break;
            }
            case 10: {
                let _e423 = local_5;
                local_4 = ((_e423 + _e292) - ((_e423 * 2f) * _e292));
                break;
            }
            case 12: {
                if Sg {
                    let _e428 = local_5;
                    let _e429 = clamp(_e428, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_5 = _e429;
                    let _e444 = (_e429 - vec3(min(min(_e429.x, _e429.y), _e429.z)));
                    let _e452 = (_e444 * ((max(max(_e292.x, _e292.y), _e292.z) - min(min(_e292.x, _e292.y), _e292.z)) / max(0.000062f, max(max(_e444.x, _e444.y), _e444.z))));
                    let _e453 = dot(_e292, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e456 = (_e452 - vec3(dot(_e452, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e469 = (vec2<f32>(_e453, (1f - _e453)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e456.x, _e456.y), _e456.z)), max(max(_e456.x, _e456.y), _e456.z))));
                    local_4 = ((_e456 * min(1f, min(_e469.x, _e469.y))) + vec3(_e453));
                }
                break;
            }
            case 13: {
                if Sg {
                    let _e477 = local_5;
                    let _e478 = clamp(_e477, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_5 = _e478;
                    let _e493 = (_e292 - vec3(min(min(_e292.x, _e292.y), _e292.z)));
                    let _e501 = (_e493 * ((max(max(_e478.x, _e478.y), _e478.z) - min(min(_e478.x, _e478.y), _e478.z)) / max(0.000062f, max(max(_e493.x, _e493.y), _e493.z))));
                    let _e502 = dot(_e292, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e505 = (_e501 - vec3(dot(_e501, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e518 = (vec2<f32>(_e502, (1f - _e502)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e505.x, _e505.y), _e505.z)), max(max(_e505.x, _e505.y), _e505.z))));
                    local_4 = ((_e505 * min(1f, min(_e518.x, _e518.y))) + vec3(_e502));
                }
                break;
            }
            case 14: {
                if Sg {
                    let _e526 = local_5;
                    let _e527 = clamp(_e526, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_5 = _e527;
                    let _e528 = dot(_e292, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e531 = (_e527 - vec3(dot(_e527, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e544 = (vec2<f32>(_e528, (1f - _e528)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e531.x, _e531.y), _e531.z)), max(max(_e531.x, _e531.y), _e531.z))));
                    local_4 = ((_e531 * min(1f, min(_e544.x, _e544.y))) + vec3(_e528));
                }
                break;
            }
            case 15: {
                if Sg {
                    let _e552 = local_5;
                    let _e553 = clamp(_e552, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_5 = _e553;
                    let _e554 = dot(_e553, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e557 = (_e292 - vec3(dot(_e292, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e570 = (vec2<f32>(_e554, (1f - _e554)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e557.x, _e557.y), _e557.z)), max(max(_e557.x, _e557.y), _e557.z))));
                    local_4 = ((_e557 * min(1f, min(_e570.x, _e570.y))) + vec3(_e554));
                }
                break;
            }
            default: {
            }
        }
        let _e578 = local_4;
        let _e580 = mix(_e285, _e578, vec3(_e284.w));
        phi_5888_ = vec4<f32>(_e580.x, _e580.y, _e580.z, _e264);
    }
    let _e586 = phi_5888_;
    let _e589 = (_e586.xyz * _e586.w);
    let _e595 = vec4<f32>(_e589.x, _e586.y, _e586.z, _e586.w);
    let _e601 = vec4<f32>(_e595.x, _e589.y, _e595.z, _e595.w);
    let _e607 = vec4<f32>(_e601.x, _e601.y, _e589.z, _e601.w);
    phi_1269_ = Mg;
    if Mg {
        let _e609 = A0_.V0_;
        phi_1269_ = (_e609 != 0u);
    }
    let _e612 = phi_1269_;
    phi_7321_ = _e129;
    if _e612 {
        if (_e260 != 0u) {
            phi_5914_ = _e260;
        } else {
            let _e616 = d0_.X1_[_e117];
            phi_5914_ = _e616;
        }
        let _e618 = phi_5914_;
        let _e620 = A0_.V0_;
        if (_e620 == (_e618 >> bitcast<u32>(16i))) {
            phi_5942_ = min(_e129, unpack2x16float(_e618).x);
        } else {
            phi_5942_ = 0f;
        }
        let _e628 = phi_5942_;
        phi_7321_ = _e628;
    }
    let _e630 = phi_7321_;
    phi_1299_ = Og;
    if Og {
        let _e632 = A0_.n2_;
        phi_1299_ = (_e632 != 0u);
    }
    let _e635 = phi_1299_;
    phi_7333_ = _e119;
    if _e635 {
        let _e638 = g0_.X1_[_e117];
        let _e642 = ((unpack4x8unorm(_e638) * (1f - _e586.w)) + _e607);
        if (_e119.w != 0f) {
            phi_5978_ = (1f / _e119.w);
        } else {
            phi_5978_ = 0f;
        }
        let _e648 = phi_5978_;
        let _e649 = (_e119.xyz * _e648);
        let _e651 = A0_.n2_;
        local_2 = _e649;
        let _e652 = _e642.xyz;
        if (_e642.w != 0f) {
            phi_5979_ = (1f / _e642.w);
        } else {
            phi_5979_ = 0f;
        }
        let _e657 = phi_5979_;
        let _e658 = (_e652 * _e657);
        local = _e658;
        switch bitcast<i32>(_e651) {
            case 11: {
                let _e660 = local_2;
                local_1 = (_e660 * _e658);
                break;
            }
            case 1: {
                let _e662 = local_2;
                local_1 = ((_e662 + _e658) - (_e662 * _e658));
                break;
            }
            case 2: {
                let _e666 = local_2;
                let _e667 = (_e666 * _e658);
                local_1 = (select(_e667, (((_e666 + _e658) - _e667) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e658 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e674 = local_2;
                local_1 = min(_e674, _e658);
                break;
            }
            case 4: {
                let _e676 = local_2;
                local_1 = max(_e676, _e658);
                break;
            }
            case 5: {
                let _e679 = clamp(_e652, vec3<f32>(0f, 0f, 0f), _e642.www);
                let _e685 = vec4<f32>(_e679.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e691 = vec4<f32>(_e685.x, _e679.y, _e685.z, _e685.w);
                let _e698 = local_2;
                let _e701 = (clamp((vec3<f32>(1f, 1f, 1f) - _e698), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e642.w);
                let _e702 = vec4<f32>(_e691.x, _e691.y, _e679.z, _e691.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e702 / _e701)), sign(_e702), (_e701 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e708 = local_2;
                local_2 = clamp(_e708, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e711 = clamp(_e652, vec3<f32>(0f, 0f, 0f), _e642.www);
                let _e717 = vec4<f32>(_e711.x, _e642.y, _e642.z, _e642.w);
                let _e723 = vec4<f32>(_e717.x, _e711.y, _e717.z, _e717.w);
                phi_7008_ = vec4<f32>(_e723.x, _e723.y, _e711.z, _e723.w);
                if (_e642.w == 0f) {
                    phi_7008_ = vec4<f32>(_e711.x, _e711.y, _e711.z, 1f);
                }
                let _e733 = phi_7008_;
                let _e737 = (vec3(_e733.w) - _e733.xyz);
                let _e738 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e737 / (_e738 * _e733.w))), sign(_e737), (_e738 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e746 = local_2;
                let _e747 = (_e746 * _e658);
                local_1 = (select(_e747, (((_e746 + _e658) - _e747) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e746 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_6876_ = 0i;
                loop {
                    let _e755 = phi_6876_;
                    if (_e755 < 3i) {
                        let _e758 = local_2[_e755];
                        if (_e758 <= 0.5f) {
                            let _e761 = local[_e755];
                            local_1[_e755] = (1f - _e761);
                        } else {
                            let _e765 = local[_e755];
                            if (_e765 <= 0.25f) {
                                let _e767 = local[_e755];
                                let _e770 = local[_e755];
                                local_1[_e755] = ((((16f * _e767) - 12f) * _e770) + 3f);
                            } else {
                                let _e774 = local[_e755];
                                local_1[_e755] = (inverseSqrt(_e774) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_6876_ = (_e755 + 1i);
                    }
                }
                let _e779 = local_2;
                let _e783 = local_1;
                local_1 = (_e658 + ((_e658 * ((_e779 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e783));
                break;
            }
            case 9: {
                let _e786 = local_2;
                local_1 = abs((_e658 - _e786));
                break;
            }
            case 10: {
                let _e789 = local_2;
                local_1 = ((_e789 + _e658) - ((_e789 * 2f) * _e658));
                break;
            }
            case 12: {
                if Sg {
                    let _e794 = local_2;
                    let _e795 = clamp(_e794, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e795;
                    let _e810 = (_e795 - vec3(min(min(_e795.x, _e795.y), _e795.z)));
                    let _e818 = (_e810 * ((max(max(_e658.x, _e658.y), _e658.z) - min(min(_e658.x, _e658.y), _e658.z)) / max(0.000062f, max(max(_e810.x, _e810.y), _e810.z))));
                    let _e819 = dot(_e658, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e822 = (_e818 - vec3(dot(_e818, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e835 = (vec2<f32>(_e819, (1f - _e819)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e822.x, _e822.y), _e822.z)), max(max(_e822.x, _e822.y), _e822.z))));
                    local_1 = ((_e822 * min(1f, min(_e835.x, _e835.y))) + vec3(_e819));
                }
                break;
            }
            case 13: {
                if Sg {
                    let _e843 = local_2;
                    let _e844 = clamp(_e843, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e844;
                    let _e859 = (_e658 - vec3(min(min(_e658.x, _e658.y), _e658.z)));
                    let _e867 = (_e859 * ((max(max(_e844.x, _e844.y), _e844.z) - min(min(_e844.x, _e844.y), _e844.z)) / max(0.000062f, max(max(_e859.x, _e859.y), _e859.z))));
                    let _e868 = dot(_e658, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e871 = (_e867 - vec3(dot(_e867, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e884 = (vec2<f32>(_e868, (1f - _e868)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e871.x, _e871.y), _e871.z)), max(max(_e871.x, _e871.y), _e871.z))));
                    local_1 = ((_e871 * min(1f, min(_e884.x, _e884.y))) + vec3(_e868));
                }
                break;
            }
            case 14: {
                if Sg {
                    let _e892 = local_2;
                    let _e893 = clamp(_e892, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e893;
                    let _e894 = dot(_e658, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e897 = (_e893 - vec3(dot(_e893, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e910 = (vec2<f32>(_e894, (1f - _e894)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e897.x, _e897.y), _e897.z)), max(max(_e897.x, _e897.y), _e897.z))));
                    local_1 = ((_e897 * min(1f, min(_e910.x, _e910.y))) + vec3(_e894));
                }
                break;
            }
            case 15: {
                if Sg {
                    let _e918 = local_2;
                    let _e919 = clamp(_e918, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e919;
                    let _e920 = dot(_e919, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e923 = (_e658 - vec3(dot(_e658, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e936 = (vec2<f32>(_e920, (1f - _e920)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e923.x, _e923.y), _e923.z)), max(max(_e923.x, _e923.y), _e923.z))));
                    local_1 = ((_e923 * min(1f, min(_e936.x, _e936.y))) + vec3(_e920));
                }
                break;
            }
            default: {
            }
        }
        let _e944 = local_1;
        let _e947 = (mix(_e649, _e944, vec3(_e642.w)) * _e119.w);
        let _e953 = vec4<f32>(_e947.x, _e119.y, _e119.z, _e119.w);
        let _e959 = vec4<f32>(_e953.x, _e947.y, _e953.z, _e953.w);
        phi_7333_ = vec4<f32>(_e959.x, _e959.y, _e947.z, _e959.w);
    }
    let _e967 = phi_7333_;
    let _e969 = A0_.x4_;
    let _e971 = (_e967 * (_e630 * _e969));
    let _e975 = ((_e607 * (1f - _e971.w)) + _e971);
    let _e976 = _e975.xyz;
    let _e978 = k.y3_;
    let _e980 = k.z3_;
    if Tg {
        phi_7346_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e82.x) + (0.00583715f * _e82.y))))) * _e978) + _e980)) + _e976);
    } else {
        phi_7346_ = _e976;
    }
    let _e994 = phi_7346_;
    let _e1000 = vec4<f32>(_e994.x, _e975.y, _e975.z, _e975.w);
    let _e1006 = vec4<f32>(_e1000.x, _e994.y, _e1000.z, _e1000.w);
    let _e1012 = vec4<f32>(_e1006.x, _e1006.y, _e994.z, _e1006.w);
    switch bitcast<i32>(0u) {
        default: {
            if (_e975.w == 0f) {
                break;
            }
            let _e1016 = (1f - _e975.w);
            phi_7348_ = _e1012;
            if (_e1016 != 0f) {
                let _e1020 = g0_.X1_[_e117];
                phi_7348_ = (_e1012 + (unpack4x8unorm(_e1020) * _e1016));
            }
            let _e1025 = phi_7348_;
            g0_.X1_[_e117] = pack4x8unorm(_e1025);
            break;
        }
    }
    if (_e260 != 0u) {
        d0_.X1_[_e117] = _e260;
    }
    p4_.X1_[_e117] = 65536u;
    return;
}

@fragment 
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) U0_: vec2<f32>, @location(1) N0_: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    U0_1 = U0_;
    N0_1 = N0_;
    main_1();
}
