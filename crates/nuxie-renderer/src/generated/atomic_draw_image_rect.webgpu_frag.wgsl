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
var<private> S4_1: f32;
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
    var phi_5931_: f32;
    var phi_1536_: bool;
    var phi_5201_: f32;
    var phi_5200_: f32;
    var phi_5202_: f32;
    var phi_5205_: f32;
    var phi_5204_: f32;
    var phi_1573_: bool;
    var phi_5207_: f32;
    var phi_5898_: u32;
    var phi_5206_: f32;
    var phi_5897_: u32;
    var phi_5231_: vec4<f32>;
    var phi_1692_: bool;
    var phi_5235_: u32;
    var phi_1701_: bool;
    var phi_5250_: f32;
    var phi_5736_: vec4<f32>;
    var phi_5672_: i32;
    var phi_5893_: vec4<f32>;
    var phi_1274_: bool;
    var phi_5919_: u32;
    var phi_5947_: f32;
    var phi_7326_: f32;
    var phi_1304_: bool;
    var phi_5983_: f32;
    var phi_5984_: f32;
    var phi_7013_: vec4<f32>;
    var phi_6881_: i32;
    var phi_7338_: vec4<f32>;
    var phi_7351_: vec3<f32>;
    var phi_7353_: vec4<f32>;

    let _e83 = gl_FragCoord_1;
    let _e84 = _e83.xy;
    let _e87 = bitcast<vec2<u32>>(vec2<i32>(floor(_e84)));
    let _e89 = k.q5_;
    let _e118 = bitcast<i32>((((((_e87.y >> bitcast<u32>(5u)) * (((_e89 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e87.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e87.x & 28u) << bitcast<u32>(5u)) + ((_e87.y & 28u) << bitcast<u32>(2i)))) + (((_e87.y & 3u) << bitcast<u32>(2i)) + (_e87.x & 3u))));
    let _e119 = U0_1;
    let _e120 = textureSample(AC, R5_, _e119);
    let _e121 = S4_1;
    let _e122 = min(_e121, 1f);
    phi_5931_ = _e122;
    if Ng {
        let _e123 = N0_1;
        let _e126 = min(_e123.xy, _e123.zw);
        phi_5931_ = clamp(min(_e126.x, _e126.y), 0f, _e122);
    }
    let _e132 = phi_5931_;
    let _e135 = p4_.X1_[_e118];
    let _e137 = (_e135 >> bitcast<u32>(17u));
    let _e141 = ((f32((_e135 & 131071u)) * 0.00048828125f) + -32f);
    let _e144 = TC.X1_[_e137];
    phi_5200_ = _e141;
    if ((_e144.x & 768u) != 0u) {
        let _e148 = abs(_e141);
        phi_1536_ = Qg;
        if Qg {
            phi_1536_ = ((_e144.x & 512u) != 0u);
        }
        let _e152 = phi_1536_;
        phi_5201_ = _e148;
        if _e152 {
            phi_5201_ = (1f - abs(((fract((_e148 * 0.5f)) * 2f) + -1f)));
        }
        let _e160 = phi_5201_;
        phi_5200_ = _e160;
    }
    let _e162 = phi_5200_;
    let _e163 = clamp(_e162, 0f, 1f);
    phi_5204_ = _e163;
    if Mg {
        let _e165 = (_e144.x >> bitcast<u32>(16u));
        phi_5205_ = _e163;
        if (_e165 != 0u) {
            let _e169 = d0_.X1_[_e118];
            if (_e165 == (_e169 >> bitcast<u32>(16i))) {
                phi_5202_ = min(_e163, unpack2x16float(_e169).x);
            } else {
                phi_5202_ = 0f;
            }
            let _e177 = phi_5202_;
            phi_5205_ = _e177;
        }
        let _e179 = phi_5205_;
        phi_5204_ = _e179;
    }
    let _e181 = phi_5204_;
    phi_1573_ = Ng;
    if Ng {
        phi_1573_ = ((_e144.x & 1024u) != 0u);
    }
    let _e185 = phi_1573_;
    phi_5207_ = _e181;
    if _e185 {
        let _e186 = (_e137 * 4u);
        let _e190 = PB.X1_[(_e186 + 2u)];
        let _e201 = PB.X1_[(_e186 + 3u)];
        let _e206 = _e201.zw;
        let _e208 = ((abs(((mat2x2<f32>(vec2<f32>(_e190.x, _e190.y), vec2<f32>(_e190.z, _e190.w)) * _e84) + _e201.xy)) * _e206) - _e206);
        phi_5207_ = min(_e181, clamp((min(_e208.x, _e208.y) + 0.5f), 0f, 1f));
    }
    let _e216 = phi_5207_;
    let _e217 = (_e144.x & 15u);
    if (_e217 <= 1u) {
        let _e222 = (Mg && (_e217 == 0u));
        phi_5898_ = 0u;
        if _e222 {
            phi_5898_ = (_e144.y | pack2x16float(vec2<f32>(_e216, 0f)));
        }
        let _e227 = phi_5898_;
        phi_5897_ = _e227;
        phi_5231_ = select(unpack4x8unorm(_e144.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e222));
    } else {
        let _e230 = (_e137 * 4u);
        let _e233 = PB.X1_[_e230];
        let _e244 = PB.X1_[(_e230 + 1u)];
        let _e247 = ((mat2x2<f32>(vec2<f32>(_e233.x, _e233.y), vec2<f32>(_e233.z, _e233.w)) * _e84) + _e244.xy);
        if (_e217 == 2u) {
            phi_5206_ = _e247.x;
        } else {
            phi_5206_ = length(_e247);
        }
        let _e252 = phi_5206_;
        let _e261 = textureSampleLevel(DD, Bb, vec2<f32>(((clamp(_e252, 0f, 1f) * _e244.z) + _e244.w), bitcast<f32>(_e144.y)), 0f);
        phi_5897_ = 0u;
        phi_5231_ = _e261;
    }
    let _e263 = phi_5897_;
    let _e265 = phi_5231_;
    let _e267 = (_e265.w * _e216);
    let _e272 = vec4<f32>(_e265.x, _e265.y, _e265.z, _e267);
    phi_1692_ = Og;
    if Og {
        phi_1692_ = (_e267 != 0f);
    }
    let _e275 = phi_1692_;
    phi_5235_ = u32();
    phi_1701_ = _e275;
    if _e275 {
        let _e278 = ((_e144.x >> bitcast<u32>(4i)) & 15u);
        phi_5235_ = _e278;
        phi_1701_ = (_e278 != 0u);
    }
    let _e281 = phi_5235_;
    let _e283 = phi_1701_;
    phi_5893_ = _e272;
    if _e283 {
        let _e286 = g0_.X1_[_e118];
        let _e287 = unpack4x8unorm(_e286);
        let _e288 = _e272.xyz;
        local_5 = _e288;
        let _e289 = _e287.xyz;
        if (_e287.w != 0f) {
            phi_5250_ = (1f / _e287.w);
        } else {
            phi_5250_ = 0f;
        }
        let _e294 = phi_5250_;
        let _e295 = (_e289 * _e294);
        local_3 = _e295;
        switch bitcast<i32>(_e281) {
            case 11: {
                let _e297 = local_5;
                local_4 = (_e297 * _e295);
                break;
            }
            case 1: {
                let _e299 = local_5;
                local_4 = ((_e299 + _e295) - (_e299 * _e295));
                break;
            }
            case 2: {
                let _e303 = local_5;
                let _e304 = (_e303 * _e295);
                local_4 = (select(_e304, (((_e303 + _e295) - _e304) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e295 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e311 = local_5;
                local_4 = min(_e311, _e295);
                break;
            }
            case 4: {
                let _e313 = local_5;
                local_4 = max(_e313, _e295);
                break;
            }
            case 5: {
                let _e316 = clamp(_e289, vec3<f32>(0f, 0f, 0f), _e287.www);
                let _e322 = vec4<f32>(_e316.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e328 = vec4<f32>(_e322.x, _e316.y, _e322.z, _e322.w);
                let _e335 = local_5;
                let _e338 = (clamp((vec3<f32>(1f, 1f, 1f) - _e335), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e287.w);
                let _e339 = vec4<f32>(_e328.x, _e328.y, _e316.z, _e328.w).xyz;
                local_4 = select(min(vec3<f32>(1f, 1f, 1f), (_e339 / _e338)), sign(_e339), (_e338 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e345 = local_5;
                local_5 = clamp(_e345, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e348 = clamp(_e289, vec3<f32>(0f, 0f, 0f), _e287.www);
                let _e354 = vec4<f32>(_e348.x, _e287.y, _e287.z, _e287.w);
                let _e360 = vec4<f32>(_e354.x, _e348.y, _e354.z, _e354.w);
                phi_5736_ = vec4<f32>(_e360.x, _e360.y, _e348.z, _e360.w);
                if (_e287.w == 0f) {
                    phi_5736_ = vec4<f32>(_e348.x, _e348.y, _e348.z, 1f);
                }
                let _e370 = phi_5736_;
                let _e374 = (vec3(_e370.w) - _e370.xyz);
                let _e375 = local_5;
                local_4 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e374 / (_e375 * _e370.w))), sign(_e374), (_e375 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e383 = local_5;
                let _e384 = (_e383 * _e295);
                local_4 = (select(_e384, (((_e383 + _e295) - _e384) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e383 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_5672_ = 0i;
                loop {
                    let _e392 = phi_5672_;
                    if (_e392 < 3i) {
                        let _e395 = local_5[_e392];
                        if (_e395 <= 0.5f) {
                            let _e398 = local_3[_e392];
                            local_4[_e392] = (1f - _e398);
                        } else {
                            let _e402 = local_3[_e392];
                            if (_e402 <= 0.25f) {
                                let _e404 = local_3[_e392];
                                let _e407 = local_3[_e392];
                                local_4[_e392] = ((((16f * _e404) - 12f) * _e407) + 3f);
                            } else {
                                let _e411 = local_3[_e392];
                                local_4[_e392] = (inverseSqrt(_e411) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_5672_ = (_e392 + 1i);
                    }
                }
                let _e416 = local_5;
                let _e420 = local_4;
                local_4 = (_e295 + ((_e295 * ((_e416 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e420));
                break;
            }
            case 9: {
                let _e423 = local_5;
                local_4 = abs((_e295 - _e423));
                break;
            }
            case 10: {
                let _e426 = local_5;
                local_4 = ((_e426 + _e295) - ((_e426 * 2f) * _e295));
                break;
            }
            case 12: {
                if Sg {
                    let _e431 = local_5;
                    let _e432 = clamp(_e431, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_5 = _e432;
                    let _e447 = (_e432 - vec3(min(min(_e432.x, _e432.y), _e432.z)));
                    let _e455 = (_e447 * ((max(max(_e295.x, _e295.y), _e295.z) - min(min(_e295.x, _e295.y), _e295.z)) / max(0.000062f, max(max(_e447.x, _e447.y), _e447.z))));
                    let _e456 = dot(_e295, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e459 = (_e455 - vec3(dot(_e455, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e472 = (vec2<f32>(_e456, (1f - _e456)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e459.x, _e459.y), _e459.z)), max(max(_e459.x, _e459.y), _e459.z))));
                    local_4 = ((_e459 * min(1f, min(_e472.x, _e472.y))) + vec3(_e456));
                }
                break;
            }
            case 13: {
                if Sg {
                    let _e480 = local_5;
                    let _e481 = clamp(_e480, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_5 = _e481;
                    let _e496 = (_e295 - vec3(min(min(_e295.x, _e295.y), _e295.z)));
                    let _e504 = (_e496 * ((max(max(_e481.x, _e481.y), _e481.z) - min(min(_e481.x, _e481.y), _e481.z)) / max(0.000062f, max(max(_e496.x, _e496.y), _e496.z))));
                    let _e505 = dot(_e295, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e508 = (_e504 - vec3(dot(_e504, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e521 = (vec2<f32>(_e505, (1f - _e505)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e508.x, _e508.y), _e508.z)), max(max(_e508.x, _e508.y), _e508.z))));
                    local_4 = ((_e508 * min(1f, min(_e521.x, _e521.y))) + vec3(_e505));
                }
                break;
            }
            case 14: {
                if Sg {
                    let _e529 = local_5;
                    let _e530 = clamp(_e529, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_5 = _e530;
                    let _e531 = dot(_e295, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e534 = (_e530 - vec3(dot(_e530, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e547 = (vec2<f32>(_e531, (1f - _e531)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e534.x, _e534.y), _e534.z)), max(max(_e534.x, _e534.y), _e534.z))));
                    local_4 = ((_e534 * min(1f, min(_e547.x, _e547.y))) + vec3(_e531));
                }
                break;
            }
            case 15: {
                if Sg {
                    let _e555 = local_5;
                    let _e556 = clamp(_e555, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_5 = _e556;
                    let _e557 = dot(_e556, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e560 = (_e295 - vec3(dot(_e295, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e573 = (vec2<f32>(_e557, (1f - _e557)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e560.x, _e560.y), _e560.z)), max(max(_e560.x, _e560.y), _e560.z))));
                    local_4 = ((_e560 * min(1f, min(_e573.x, _e573.y))) + vec3(_e557));
                }
                break;
            }
            default: {
            }
        }
        let _e581 = local_4;
        let _e583 = mix(_e288, _e581, vec3(_e287.w));
        phi_5893_ = vec4<f32>(_e583.x, _e583.y, _e583.z, _e267);
    }
    let _e589 = phi_5893_;
    let _e592 = (_e589.xyz * _e589.w);
    let _e598 = vec4<f32>(_e592.x, _e589.y, _e589.z, _e589.w);
    let _e604 = vec4<f32>(_e598.x, _e592.y, _e598.z, _e598.w);
    let _e610 = vec4<f32>(_e604.x, _e604.y, _e592.z, _e604.w);
    phi_1274_ = Mg;
    if Mg {
        let _e612 = A0_.V0_;
        phi_1274_ = (_e612 != 0u);
    }
    let _e615 = phi_1274_;
    phi_7326_ = _e132;
    if _e615 {
        if (_e263 != 0u) {
            phi_5919_ = _e263;
        } else {
            let _e619 = d0_.X1_[_e118];
            phi_5919_ = _e619;
        }
        let _e621 = phi_5919_;
        let _e623 = A0_.V0_;
        if (_e623 == (_e621 >> bitcast<u32>(16i))) {
            phi_5947_ = min(_e132, unpack2x16float(_e621).x);
        } else {
            phi_5947_ = 0f;
        }
        let _e631 = phi_5947_;
        phi_7326_ = _e631;
    }
    let _e633 = phi_7326_;
    phi_1304_ = Og;
    if Og {
        let _e635 = A0_.n2_;
        phi_1304_ = (_e635 != 0u);
    }
    let _e638 = phi_1304_;
    phi_7338_ = _e120;
    if _e638 {
        let _e641 = g0_.X1_[_e118];
        let _e645 = ((unpack4x8unorm(_e641) * (1f - _e589.w)) + _e610);
        if (_e120.w != 0f) {
            phi_5983_ = (1f / _e120.w);
        } else {
            phi_5983_ = 0f;
        }
        let _e651 = phi_5983_;
        let _e652 = (_e120.xyz * _e651);
        let _e654 = A0_.n2_;
        local_2 = _e652;
        let _e655 = _e645.xyz;
        if (_e645.w != 0f) {
            phi_5984_ = (1f / _e645.w);
        } else {
            phi_5984_ = 0f;
        }
        let _e660 = phi_5984_;
        let _e661 = (_e655 * _e660);
        local = _e661;
        switch bitcast<i32>(_e654) {
            case 11: {
                let _e663 = local_2;
                local_1 = (_e663 * _e661);
                break;
            }
            case 1: {
                let _e665 = local_2;
                local_1 = ((_e665 + _e661) - (_e665 * _e661));
                break;
            }
            case 2: {
                let _e669 = local_2;
                let _e670 = (_e669 * _e661);
                local_1 = (select(_e670, (((_e669 + _e661) - _e670) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e661 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e677 = local_2;
                local_1 = min(_e677, _e661);
                break;
            }
            case 4: {
                let _e679 = local_2;
                local_1 = max(_e679, _e661);
                break;
            }
            case 5: {
                let _e682 = clamp(_e655, vec3<f32>(0f, 0f, 0f), _e645.www);
                let _e688 = vec4<f32>(_e682.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e694 = vec4<f32>(_e688.x, _e682.y, _e688.z, _e688.w);
                let _e701 = local_2;
                let _e704 = (clamp((vec3<f32>(1f, 1f, 1f) - _e701), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e645.w);
                let _e705 = vec4<f32>(_e694.x, _e694.y, _e682.z, _e694.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e705 / _e704)), sign(_e705), (_e704 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e711 = local_2;
                local_2 = clamp(_e711, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e714 = clamp(_e655, vec3<f32>(0f, 0f, 0f), _e645.www);
                let _e720 = vec4<f32>(_e714.x, _e645.y, _e645.z, _e645.w);
                let _e726 = vec4<f32>(_e720.x, _e714.y, _e720.z, _e720.w);
                phi_7013_ = vec4<f32>(_e726.x, _e726.y, _e714.z, _e726.w);
                if (_e645.w == 0f) {
                    phi_7013_ = vec4<f32>(_e714.x, _e714.y, _e714.z, 1f);
                }
                let _e736 = phi_7013_;
                let _e740 = (vec3(_e736.w) - _e736.xyz);
                let _e741 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e740 / (_e741 * _e736.w))), sign(_e740), (_e741 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e749 = local_2;
                let _e750 = (_e749 * _e661);
                local_1 = (select(_e750, (((_e749 + _e661) - _e750) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e749 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_6881_ = 0i;
                loop {
                    let _e758 = phi_6881_;
                    if (_e758 < 3i) {
                        let _e761 = local_2[_e758];
                        if (_e761 <= 0.5f) {
                            let _e764 = local[_e758];
                            local_1[_e758] = (1f - _e764);
                        } else {
                            let _e768 = local[_e758];
                            if (_e768 <= 0.25f) {
                                let _e770 = local[_e758];
                                let _e773 = local[_e758];
                                local_1[_e758] = ((((16f * _e770) - 12f) * _e773) + 3f);
                            } else {
                                let _e777 = local[_e758];
                                local_1[_e758] = (inverseSqrt(_e777) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_6881_ = (_e758 + 1i);
                    }
                }
                let _e782 = local_2;
                let _e786 = local_1;
                local_1 = (_e661 + ((_e661 * ((_e782 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e786));
                break;
            }
            case 9: {
                let _e789 = local_2;
                local_1 = abs((_e661 - _e789));
                break;
            }
            case 10: {
                let _e792 = local_2;
                local_1 = ((_e792 + _e661) - ((_e792 * 2f) * _e661));
                break;
            }
            case 12: {
                if Sg {
                    let _e797 = local_2;
                    let _e798 = clamp(_e797, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e798;
                    let _e813 = (_e798 - vec3(min(min(_e798.x, _e798.y), _e798.z)));
                    let _e821 = (_e813 * ((max(max(_e661.x, _e661.y), _e661.z) - min(min(_e661.x, _e661.y), _e661.z)) / max(0.000062f, max(max(_e813.x, _e813.y), _e813.z))));
                    let _e822 = dot(_e661, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e825 = (_e821 - vec3(dot(_e821, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e838 = (vec2<f32>(_e822, (1f - _e822)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e825.x, _e825.y), _e825.z)), max(max(_e825.x, _e825.y), _e825.z))));
                    local_1 = ((_e825 * min(1f, min(_e838.x, _e838.y))) + vec3(_e822));
                }
                break;
            }
            case 13: {
                if Sg {
                    let _e846 = local_2;
                    let _e847 = clamp(_e846, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e847;
                    let _e862 = (_e661 - vec3(min(min(_e661.x, _e661.y), _e661.z)));
                    let _e870 = (_e862 * ((max(max(_e847.x, _e847.y), _e847.z) - min(min(_e847.x, _e847.y), _e847.z)) / max(0.000062f, max(max(_e862.x, _e862.y), _e862.z))));
                    let _e871 = dot(_e661, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e874 = (_e870 - vec3(dot(_e870, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e887 = (vec2<f32>(_e871, (1f - _e871)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e874.x, _e874.y), _e874.z)), max(max(_e874.x, _e874.y), _e874.z))));
                    local_1 = ((_e874 * min(1f, min(_e887.x, _e887.y))) + vec3(_e871));
                }
                break;
            }
            case 14: {
                if Sg {
                    let _e895 = local_2;
                    let _e896 = clamp(_e895, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e896;
                    let _e897 = dot(_e661, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e900 = (_e896 - vec3(dot(_e896, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e913 = (vec2<f32>(_e897, (1f - _e897)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e900.x, _e900.y), _e900.z)), max(max(_e900.x, _e900.y), _e900.z))));
                    local_1 = ((_e900 * min(1f, min(_e913.x, _e913.y))) + vec3(_e897));
                }
                break;
            }
            case 15: {
                if Sg {
                    let _e921 = local_2;
                    let _e922 = clamp(_e921, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e922;
                    let _e923 = dot(_e922, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e926 = (_e661 - vec3(dot(_e661, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e939 = (vec2<f32>(_e923, (1f - _e923)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e926.x, _e926.y), _e926.z)), max(max(_e926.x, _e926.y), _e926.z))));
                    local_1 = ((_e926 * min(1f, min(_e939.x, _e939.y))) + vec3(_e923));
                }
                break;
            }
            default: {
            }
        }
        let _e947 = local_1;
        let _e950 = (mix(_e652, _e947, vec3(_e645.w)) * _e120.w);
        let _e956 = vec4<f32>(_e950.x, _e120.y, _e120.z, _e120.w);
        let _e962 = vec4<f32>(_e956.x, _e950.y, _e956.z, _e956.w);
        phi_7338_ = vec4<f32>(_e962.x, _e962.y, _e950.z, _e962.w);
    }
    let _e970 = phi_7338_;
    let _e972 = A0_.x4_;
    let _e974 = (_e970 * (_e633 * _e972));
    let _e978 = ((_e610 * (1f - _e974.w)) + _e974);
    let _e979 = _e978.xyz;
    let _e981 = k.y3_;
    let _e983 = k.z3_;
    if Tg {
        phi_7351_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e83.x) + (0.00583715f * _e83.y))))) * _e981) + _e983)) + _e979);
    } else {
        phi_7351_ = _e979;
    }
    let _e997 = phi_7351_;
    let _e1003 = vec4<f32>(_e997.x, _e978.y, _e978.z, _e978.w);
    let _e1009 = vec4<f32>(_e1003.x, _e997.y, _e1003.z, _e1003.w);
    let _e1015 = vec4<f32>(_e1009.x, _e1009.y, _e997.z, _e1009.w);
    switch bitcast<i32>(0u) {
        default: {
            if (_e978.w == 0f) {
                break;
            }
            let _e1019 = (1f - _e978.w);
            phi_7353_ = _e1015;
            if (_e1019 != 0f) {
                let _e1023 = g0_.X1_[_e118];
                phi_7353_ = (_e1015 + (unpack4x8unorm(_e1023) * _e1019));
            }
            let _e1028 = phi_7353_;
            g0_.X1_[_e118] = pack4x8unorm(_e1028);
            break;
        }
    }
    if (_e263 != 0u) {
        d0_.X1_[_e118] = _e263;
    }
    p4_.X1_[_e118] = 65536u;
    return;
}

@fragment 
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) U0_: vec2<f32>, @location(1) S4_: f32, @location(2) N0_: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    U0_1 = U0_;
    S4_1 = S4_;
    N0_1 = N0_;
    main_1();
}
