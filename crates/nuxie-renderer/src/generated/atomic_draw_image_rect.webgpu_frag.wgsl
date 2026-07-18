struct Fe {
    c2_: array<vec2<u32>>,
}

struct h0xd {
    c2_: array<u32>,
}

struct Ge {
    c2_: array<vec4<f32>>,
}

struct j0xd {
    c2_: array<u32>,
}

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

struct q4xd {
    c2_: array<u32>,
}

@id(7) override bh: bool = true;
@id(6) override ah: bool = true;
@id(4) override Yg: bool = true;
@id(0) override Ug: bool = true;
@id(1) override Vg: bool = true;
@id(2) override Wg: bool = true;

@group(0) @binding(3)
var<storage> AD: Fe;
@group(2) @binding(1)
var<storage, read_write> h0_: h0xd;
@group(0) @binding(4)
var<storage> RB: Ge;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(3) @binding(8)
var Jb: sampler;
@group(2) @binding(0)
var<storage, read_write> j0_: j0xd;
@group(0) @binding(0)
var<uniform> m: CC;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;
var<private> X1_1: vec2<f32>;
var<private> R4_1: f32;
var<private> L0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> q4_: q4xd;
var<private> v3_1: u32;
var<private> A1_1: u32;
var<private> H1_1: f32;
@group(3) @binding(9)
var Z9_: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var local_3: vec3<f32>;
    var local_4: vec3<f32>;
    var local_5: vec3<f32>;
    var phi_5911_: f32;
    var phi_1526_: bool;
    var phi_5181_: f32;
    var phi_5180_: f32;
    var phi_5182_: f32;
    var phi_5185_: f32;
    var phi_5184_: f32;
    var phi_1563_: bool;
    var phi_5187_: f32;
    var phi_5878_: u32;
    var phi_5186_: f32;
    var phi_5877_: u32;
    var phi_5211_: vec4<f32>;
    var phi_1682_: bool;
    var phi_5215_: u32;
    var phi_1691_: bool;
    var phi_5230_: f32;
    var phi_5716_: vec4<f32>;
    var phi_5652_: i32;
    var phi_5873_: vec4<f32>;
    var phi_1271_: bool;
    var phi_5899_: u32;
    var phi_5927_: f32;
    var phi_7306_: f32;
    var phi_1299_: bool;
    var phi_5963_: f32;
    var phi_5964_: f32;
    var phi_6993_: vec4<f32>;
    var phi_6861_: i32;
    var phi_7318_: vec4<f32>;
    var phi_7331_: vec3<f32>;
    var phi_7333_: vec4<f32>;

    let _e83 = gl_FragCoord_1;
    let _e84 = _e83.xy;
    let _e87 = bitcast<vec2<u32>>(vec2<i32>(floor(_e84)));
    let _e89 = m.m6_;
    let _e118 = bitcast<i32>((((((_e87.y >> bitcast<u32>(5u)) * (((_e89 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e87.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e87.x & 28u) << bitcast<u32>(5u)) + ((_e87.y & 28u) << bitcast<u32>(2i)))) + (((_e87.y & 3u) << bitcast<u32>(2i)) + (_e87.x & 3u))));
    let _e119 = X1_1;
    let _e120 = textureSample(IC, S5_, _e119);
    let _e121 = R4_1;
    let _e122 = min(_e121, 1f);
    phi_5911_ = _e122;
    if Vg {
        let _e123 = L0_1;
        let _e126 = min(_e123.xy, _e123.zw);
        phi_5911_ = clamp(min(_e126.x, _e126.y), 0f, _e122);
    }
    let _e132 = phi_5911_;
    let _e135 = q4_.c2_[_e118];
    let _e137 = (_e135 >> bitcast<u32>(17u));
    let _e141 = ((f32((_e135 & 131071u)) * 0.00048828125f) + -32f);
    let _e144 = AD.c2_[_e137];
    phi_5180_ = _e141;
    if ((_e144.x & 768u) != 0u) {
        let _e148 = abs(_e141);
        phi_1526_ = Yg;
        if Yg {
            phi_1526_ = ((_e144.x & 512u) != 0u);
        }
        let _e152 = phi_1526_;
        phi_5181_ = _e148;
        if _e152 {
            phi_5181_ = (1f - abs(((fract((_e148 * 0.5f)) * 2f) + -1f)));
        }
        let _e160 = phi_5181_;
        phi_5180_ = _e160;
    }
    let _e162 = phi_5180_;
    let _e163 = clamp(_e162, 0f, 1f);
    phi_5184_ = _e163;
    if Ug {
        let _e165 = (_e144.x >> bitcast<u32>(16u));
        phi_5185_ = _e163;
        if (_e165 != 0u) {
            let _e169 = h0_.c2_[_e118];
            if (_e165 == (_e169 >> bitcast<u32>(16i))) {
                phi_5182_ = min(_e163, unpack2x16float(_e169).x);
            } else {
                phi_5182_ = 0f;
            }
            let _e177 = phi_5182_;
            phi_5185_ = _e177;
        }
        let _e179 = phi_5185_;
        phi_5184_ = _e179;
    }
    let _e181 = phi_5184_;
    phi_1563_ = Vg;
    if Vg {
        phi_1563_ = ((_e144.x & 1024u) != 0u);
    }
    let _e185 = phi_1563_;
    phi_5187_ = _e181;
    if _e185 {
        let _e186 = (_e137 * 4u);
        let _e190 = RB.c2_[(_e186 + 2u)];
        let _e201 = RB.c2_[(_e186 + 3u)];
        let _e206 = _e201.zw;
        let _e208 = ((abs(((mat2x2<f32>(vec2<f32>(_e190.x, _e190.y), vec2<f32>(_e190.z, _e190.w)) * _e84) + _e201.xy)) * _e206) - _e206);
        phi_5187_ = min(_e181, clamp((min(_e208.x, _e208.y) + 0.5f), 0f, 1f));
    }
    let _e216 = phi_5187_;
    let _e217 = (_e144.x & 15u);
    if (_e217 <= 1u) {
        let _e222 = (Ug && (_e217 == 0u));
        phi_5878_ = 0u;
        if _e222 {
            phi_5878_ = (_e144.y | pack2x16float(vec2<f32>(_e216, 0f)));
        }
        let _e227 = phi_5878_;
        phi_5877_ = _e227;
        phi_5211_ = select(unpack4x8unorm(_e144.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e222));
    } else {
        let _e230 = (_e137 * 4u);
        let _e233 = RB.c2_[_e230];
        let _e244 = RB.c2_[(_e230 + 1u)];
        let _e247 = ((mat2x2<f32>(vec2<f32>(_e233.x, _e233.y), vec2<f32>(_e233.z, _e233.w)) * _e84) + _e244.xy);
        if (_e217 == 2u) {
            phi_5186_ = _e247.x;
        } else {
            phi_5186_ = length(_e247);
        }
        let _e252 = phi_5186_;
        let _e261 = textureSampleLevel(KD, Jb, vec2<f32>(((clamp(_e252, 0f, 1f) * _e244.z) + _e244.w), bitcast<f32>(_e144.y)), 0f);
        phi_5877_ = 0u;
        phi_5211_ = _e261;
    }
    let _e263 = phi_5877_;
    let _e265 = phi_5211_;
    let _e267 = (_e265.w * _e216);
    let _e272 = vec4<f32>(_e265.x, _e265.y, _e265.z, _e267);
    phi_1682_ = Wg;
    if Wg {
        phi_1682_ = (_e267 != 0f);
    }
    let _e275 = phi_1682_;
    phi_5215_ = u32();
    phi_1691_ = _e275;
    if _e275 {
        let _e278 = ((_e144.x >> bitcast<u32>(4i)) & 15u);
        phi_5215_ = _e278;
        phi_1691_ = (_e278 != 0u);
    }
    let _e281 = phi_5215_;
    let _e283 = phi_1691_;
    phi_5873_ = _e272;
    if _e283 {
        let _e286 = j0_.c2_[_e118];
        let _e287 = unpack4x8unorm(_e286);
        let _e288 = _e272.xyz;
        local_5 = _e288;
        let _e289 = _e287.xyz;
        if (_e287.w != 0f) {
            phi_5230_ = (1f / _e287.w);
        } else {
            phi_5230_ = 0f;
        }
        let _e294 = phi_5230_;
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
                phi_5716_ = vec4<f32>(_e360.x, _e360.y, _e348.z, _e360.w);
                if (_e287.w == 0f) {
                    phi_5716_ = vec4<f32>(_e348.x, _e348.y, _e348.z, 1f);
                }
                let _e370 = phi_5716_;
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
                phi_5652_ = 0i;
                loop {
                    let _e392 = phi_5652_;
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
                        phi_5652_ = (_e392 + 1i);
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
                if ah {
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
                if ah {
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
                if ah {
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
                if ah {
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
        phi_5873_ = vec4<f32>(_e583.x, _e583.y, _e583.z, _e267);
    }
    let _e589 = phi_5873_;
    let _e592 = (_e589.xyz * _e589.w);
    let _e598 = vec4<f32>(_e592.x, _e589.y, _e589.z, _e589.w);
    let _e604 = vec4<f32>(_e598.x, _e592.y, _e598.z, _e598.w);
    let _e610 = vec4<f32>(_e604.x, _e604.y, _e592.z, _e604.w);
    phi_1271_ = Ug;
    if Ug {
        let _e611 = v3_1;
        phi_1271_ = (_e611 != 0u);
    }
    let _e614 = phi_1271_;
    phi_7306_ = _e132;
    if _e614 {
        if (_e263 != 0u) {
            phi_5899_ = _e263;
        } else {
            let _e618 = h0_.c2_[_e118];
            phi_5899_ = _e618;
        }
        let _e620 = phi_5899_;
        let _e621 = v3_1;
        if (_e621 == (_e620 >> bitcast<u32>(16i))) {
            phi_5927_ = min(_e132, unpack2x16float(_e620).x);
        } else {
            phi_5927_ = 0f;
        }
        let _e629 = phi_5927_;
        phi_7306_ = _e629;
    }
    let _e631 = phi_7306_;
    phi_1299_ = Wg;
    if Wg {
        let _e632 = A1_1;
        phi_1299_ = (_e632 != 0u);
    }
    let _e635 = phi_1299_;
    phi_7318_ = _e120;
    if _e635 {
        let _e638 = j0_.c2_[_e118];
        let _e642 = ((unpack4x8unorm(_e638) * (1f - _e589.w)) + _e610);
        if (_e120.w != 0f) {
            phi_5963_ = (1f / _e120.w);
        } else {
            phi_5963_ = 0f;
        }
        let _e648 = phi_5963_;
        let _e649 = (_e120.xyz * _e648);
        let _e650 = A1_1;
        local_2 = _e649;
        let _e651 = _e642.xyz;
        if (_e642.w != 0f) {
            phi_5964_ = (1f / _e642.w);
        } else {
            phi_5964_ = 0f;
        }
        let _e656 = phi_5964_;
        let _e657 = (_e651 * _e656);
        local = _e657;
        switch bitcast<i32>(_e650) {
            case 11: {
                let _e659 = local_2;
                local_1 = (_e659 * _e657);
                break;
            }
            case 1: {
                let _e661 = local_2;
                local_1 = ((_e661 + _e657) - (_e661 * _e657));
                break;
            }
            case 2: {
                let _e665 = local_2;
                let _e666 = (_e665 * _e657);
                local_1 = (select(_e666, (((_e665 + _e657) - _e666) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e657 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e673 = local_2;
                local_1 = min(_e673, _e657);
                break;
            }
            case 4: {
                let _e675 = local_2;
                local_1 = max(_e675, _e657);
                break;
            }
            case 5: {
                let _e678 = clamp(_e651, vec3<f32>(0f, 0f, 0f), _e642.www);
                let _e684 = vec4<f32>(_e678.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e690 = vec4<f32>(_e684.x, _e678.y, _e684.z, _e684.w);
                let _e697 = local_2;
                let _e700 = (clamp((vec3<f32>(1f, 1f, 1f) - _e697), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e642.w);
                let _e701 = vec4<f32>(_e690.x, _e690.y, _e678.z, _e690.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e701 / _e700)), sign(_e701), (_e700 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e707 = local_2;
                local_2 = clamp(_e707, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e710 = clamp(_e651, vec3<f32>(0f, 0f, 0f), _e642.www);
                let _e716 = vec4<f32>(_e710.x, _e642.y, _e642.z, _e642.w);
                let _e722 = vec4<f32>(_e716.x, _e710.y, _e716.z, _e716.w);
                phi_6993_ = vec4<f32>(_e722.x, _e722.y, _e710.z, _e722.w);
                if (_e642.w == 0f) {
                    phi_6993_ = vec4<f32>(_e710.x, _e710.y, _e710.z, 1f);
                }
                let _e732 = phi_6993_;
                let _e736 = (vec3(_e732.w) - _e732.xyz);
                let _e737 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e736 / (_e737 * _e732.w))), sign(_e736), (_e737 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e745 = local_2;
                let _e746 = (_e745 * _e657);
                local_1 = (select(_e746, (((_e745 + _e657) - _e746) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e745 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_6861_ = 0i;
                loop {
                    let _e754 = phi_6861_;
                    if (_e754 < 3i) {
                        let _e757 = local_2[_e754];
                        if (_e757 <= 0.5f) {
                            let _e760 = local[_e754];
                            local_1[_e754] = (1f - _e760);
                        } else {
                            let _e764 = local[_e754];
                            if (_e764 <= 0.25f) {
                                let _e766 = local[_e754];
                                let _e769 = local[_e754];
                                local_1[_e754] = ((((16f * _e766) - 12f) * _e769) + 3f);
                            } else {
                                let _e773 = local[_e754];
                                local_1[_e754] = (inverseSqrt(_e773) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_6861_ = (_e754 + 1i);
                    }
                }
                let _e778 = local_2;
                let _e782 = local_1;
                local_1 = (_e657 + ((_e657 * ((_e778 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e782));
                break;
            }
            case 9: {
                let _e785 = local_2;
                local_1 = abs((_e657 - _e785));
                break;
            }
            case 10: {
                let _e788 = local_2;
                local_1 = ((_e788 + _e657) - ((_e788 * 2f) * _e657));
                break;
            }
            case 12: {
                if ah {
                    let _e793 = local_2;
                    let _e794 = clamp(_e793, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e794;
                    let _e809 = (_e794 - vec3(min(min(_e794.x, _e794.y), _e794.z)));
                    let _e817 = (_e809 * ((max(max(_e657.x, _e657.y), _e657.z) - min(min(_e657.x, _e657.y), _e657.z)) / max(0.000062f, max(max(_e809.x, _e809.y), _e809.z))));
                    let _e818 = dot(_e657, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e821 = (_e817 - vec3(dot(_e817, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e834 = (vec2<f32>(_e818, (1f - _e818)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e821.x, _e821.y), _e821.z)), max(max(_e821.x, _e821.y), _e821.z))));
                    local_1 = ((_e821 * min(1f, min(_e834.x, _e834.y))) + vec3(_e818));
                }
                break;
            }
            case 13: {
                if ah {
                    let _e842 = local_2;
                    let _e843 = clamp(_e842, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e843;
                    let _e858 = (_e657 - vec3(min(min(_e657.x, _e657.y), _e657.z)));
                    let _e866 = (_e858 * ((max(max(_e843.x, _e843.y), _e843.z) - min(min(_e843.x, _e843.y), _e843.z)) / max(0.000062f, max(max(_e858.x, _e858.y), _e858.z))));
                    let _e867 = dot(_e657, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e870 = (_e866 - vec3(dot(_e866, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e883 = (vec2<f32>(_e867, (1f - _e867)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e870.x, _e870.y), _e870.z)), max(max(_e870.x, _e870.y), _e870.z))));
                    local_1 = ((_e870 * min(1f, min(_e883.x, _e883.y))) + vec3(_e867));
                }
                break;
            }
            case 14: {
                if ah {
                    let _e891 = local_2;
                    let _e892 = clamp(_e891, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e892;
                    let _e893 = dot(_e657, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e896 = (_e892 - vec3(dot(_e892, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e909 = (vec2<f32>(_e893, (1f - _e893)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e896.x, _e896.y), _e896.z)), max(max(_e896.x, _e896.y), _e896.z))));
                    local_1 = ((_e896 * min(1f, min(_e909.x, _e909.y))) + vec3(_e893));
                }
                break;
            }
            case 15: {
                if ah {
                    let _e917 = local_2;
                    let _e918 = clamp(_e917, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e918;
                    let _e919 = dot(_e918, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e922 = (_e657 - vec3(dot(_e657, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e935 = (vec2<f32>(_e919, (1f - _e919)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e922.x, _e922.y), _e922.z)), max(max(_e922.x, _e922.y), _e922.z))));
                    local_1 = ((_e922 * min(1f, min(_e935.x, _e935.y))) + vec3(_e919));
                }
                break;
            }
            default: {
            }
        }
        let _e943 = local_1;
        let _e946 = (mix(_e649, _e943, vec3(_e642.w)) * _e120.w);
        let _e952 = vec4<f32>(_e946.x, _e120.y, _e120.z, _e120.w);
        let _e958 = vec4<f32>(_e952.x, _e946.y, _e952.z, _e952.w);
        phi_7318_ = vec4<f32>(_e958.x, _e958.y, _e946.z, _e958.w);
    }
    let _e966 = phi_7318_;
    let _e967 = H1_1;
    let _e969 = (_e966 * (_e631 * _e967));
    let _e973 = ((_e610 * (1f - _e969.w)) + _e969);
    let _e974 = _e973.xyz;
    let _e976 = m.y3_;
    let _e978 = m.z3_;
    if bh {
        phi_7331_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e83.x) + (0.00583715f * _e83.y))))) * _e976) + _e978)) + _e974);
    } else {
        phi_7331_ = _e974;
    }
    let _e992 = phi_7331_;
    let _e998 = vec4<f32>(_e992.x, _e973.y, _e973.z, _e973.w);
    let _e1004 = vec4<f32>(_e998.x, _e992.y, _e998.z, _e998.w);
    let _e1010 = vec4<f32>(_e1004.x, _e1004.y, _e992.z, _e1004.w);
    switch bitcast<i32>(0u) {
        default: {
            if (_e973.w == 0f) {
                break;
            }
            let _e1014 = (1f - _e973.w);
            phi_7333_ = _e1010;
            if (_e1014 != 0f) {
                let _e1018 = j0_.c2_[_e118];
                phi_7333_ = (_e1010 + (unpack4x8unorm(_e1018) * _e1014));
            }
            let _e1023 = phi_7333_;
            j0_.c2_[_e118] = pack4x8unorm(_e1023);
            break;
        }
    }
    if (_e263 != 0u) {
        h0_.c2_[_e118] = _e263;
    }
    q4_.c2_[_e118] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) X1_: vec2<f32>, @location(1) R4_: f32, @location(2) L0_: vec4<f32>, @location(4) @interpolate(flat) v3_: u32, @location(5) @interpolate(flat) A1_: u32, @location(3) @interpolate(flat) H1_: f32) {
    gl_FragCoord_1 = gl_FragCoord;
    X1_1 = X1_;
    R4_1 = R4_;
    L0_1 = L0_;
    v3_1 = v3_;
    A1_1 = A1_;
    H1_1 = H1_;
    main_1();
}
