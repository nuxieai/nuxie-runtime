struct Ff {
    k2_: array<vec2<u32>>,
}

struct m0he {
    k2_: array<u32>,
}

struct Gf {
    k2_: array<vec4<f32>>,
}

struct UB {
    Rc: f32,
    Ud: f32,
    cg: f32,
    dg: f32,
    B6_: u32,
    Y9_: u32,
    Of: u32,
    Pf: u32,
    k8_: vec4<i32>,
    Mh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Qh: f32,
    U4_: u32,
    c3_: f32,
    Wd: f32,
    If: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Jh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

struct n0he {
    k2_: array<u32>,
}

struct L4he {
    k2_: array<u32>,
}

struct L4he_1 {
    k2_: array<atomic<u32>>,
}

@id(7) override si: bool = true;
@id(6) override ri: bool = true;
@id(4) override pi: bool = true;
@id(0) override li: bool = true;
@id(1) override mi: bool = true;
@id(2) override ni: bool = true;
@id(3) override oi: bool = true;

@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var xa: sampler;
@group(0) @binding(3)
var<storage> XC: Ff;
@group(2) @binding(1)
var<storage, read_write> m0_: m0he;
@group(0) @binding(4)
var<storage> JB: Gf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var FD: texture_2d<f32>;
@group(3) @binding(8)
var ia: sampler;
@group(2) @binding(0)
var<storage, read_write> n0_: n0he;
var<private> S_1: vec4<f32>;
var<private> F0_1: u32;
@group(2) @binding(3)
var<storage, read_write> L4_: L4he_1;
@group(1) @binding(11)
var DC: texture_2d<f32>;
@group(1) @binding(13)
var v5_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1476_: bool;
    var phi_1489_: bool;
    var phi_4088_: f32;
    var phi_4096_: f32;
    var phi_4104_: f32;
    var phi_4103_: f32;
    var phi_1984_: bool;
    var phi_4107_: f32;
    var phi_4106_: f32;
    var phi_4108_: f32;
    var phi_4111_: f32;
    var phi_4110_: f32;
    var phi_2021_: bool;
    var phi_4113_: f32;
    var phi_4973_: u32;
    var phi_4112_: f32;
    var phi_4145_: vec4<f32>;
    var phi_4972_: u32;
    var phi_4143_: vec4<f32>;
    var phi_4150_: f32;
    var phi_4764_: vec4<f32>;
    var phi_4684_: i32;
    var phi_4957_: vec4<f32>;
    var phi_4970_: vec4<f32>;
    var phi_5002_: u32;
    var phi_4995_: vec4<f32>;
    var phi_4997_: vec3<f32>;
    var phi_4999_: vec4<f32>;

    let _e95 = gl_FragCoord_1;
    let _e96 = _e95.xy;
    let _e99 = bitcast<vec2<u32>>(vec2<i32>(floor(_e96)));
    let _e101 = j.B6_;
    let _e130 = bitcast<i32>((((((_e99.y >> bitcast<u32>(5u)) * (((_e101 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e99.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e99.x & 28u) << bitcast<u32>(5u)) + ((_e99.y & 28u) << bitcast<u32>(2i)))) + (((_e99.y & 3u) << bitcast<u32>(2i)) + (_e99.x & 3u))));
    phi_1476_ = oi;
    if oi {
        let _e131 = S_1;
        phi_1476_ = (_e131.x < -1.5f);
    }
    let _e135 = phi_1476_;
    if _e135 {
        let _e136 = S_1;
        let _e140 = textureSampleLevel(ZC, xa, vec2<f32>((3f + _e136.x), 0f), 0f);
        let _e146 = textureSampleLevel(ZC, xa, vec2<f32>((1f - _e136.y), 0f), 0f);
        phi_4103_ = ((1f - _e140.x) - _e146.x);
    } else {
        phi_1489_ = oi;
        if oi {
            let _e149 = S_1;
            phi_1489_ = (_e149.y < -1.5f);
        }
        let _e153 = phi_1489_;
        if _e153 {
            let _e154 = S_1;
            let _e157 = max(_e154.w, 0f);
            if (_e154.z >= 0f) {
                let _e160 = textureSampleLevel(ZC, xa, vec2<f32>(_e157, 0f), 0f);
                phi_4088_ = _e160.x;
            } else {
                phi_4088_ = 0f;
            }
            let _e163 = phi_4088_;
            phi_4096_ = _e163;
            if (abs(_e154.z) < 1000f) {
                let _e170 = (-2f - _e154.y);
                let _e172 = ((_e170 - _e157) * 0.5984134f);
                let _e175 = (vec4(_e157) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e172));
                let _e181 = ((_e175 * -(_e154.z)) + vec4(((_e170 * _e154.z) + (abs(_e154.x) - 0.25f))));
                let _e184 = textureSampleLevel(ZC, xa, vec2<f32>(_e181.x, 0f), 0f);
                let _e187 = textureSampleLevel(ZC, xa, vec2<f32>(_e181.y, 0f), 0f);
                let _e190 = textureSampleLevel(ZC, xa, vec2<f32>(_e181.z, 0f), 0f);
                let _e193 = textureSampleLevel(ZC, xa, vec2<f32>(_e181.w, 0f), 0f);
                let _e199 = (_e175 * 5.0959306f);
                phi_4096_ = (_e163 + (dot(vec4<f32>(_e184.x, _e187.x, _e190.x, _e193.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e199) * (_e199 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e172));
            }
            let _e208 = phi_4096_;
            phi_4104_ = (_e208 * sign(_e154.x));
        } else {
            let _e213 = S_1[0u];
            let _e215 = S_1[1u];
            phi_4104_ = min(min(_e213, abs(_e215)), 1f);
        }
        let _e220 = phi_4104_;
        phi_4103_ = _e220;
    }
    let _e222 = phi_4103_;
    let _e226 = u32(round(((_e222 * 2048f) + 65536f)));
    let _e227 = F0_1;
    let _e230 = ((_e227 << bitcast<u32>(17u)) | _e226);
    let _e233 = atomicMax((&L4_.k2_[_e130]), _e230);
    let _e235 = (_e233 >> bitcast<u32>(17u));
    if (_e235 == _e227) {
        let _e237 = S_1;
        if (_e237.y < 0f) {
            let _e244 = atomicAdd((&L4_.k2_[_e130]), ((_e226 + (_e233 - max(_e230, _e233))) - 65536u));
        }
        phi_5002_ = 0u;
        phi_4995_ = vec4<f32>(0f, 0f, 0f, 0f);
    } else {
        let _e248 = ((f32((_e233 & 131071u)) * 0.00048828125f) + -32f);
        let _e251 = XC.k2_[_e235];
        phi_4106_ = _e248;
        if ((_e251.x & 768u) != 0u) {
            let _e255 = abs(_e248);
            phi_1984_ = pi;
            if pi {
                phi_1984_ = ((_e251.x & 512u) != 0u);
            }
            let _e259 = phi_1984_;
            phi_4107_ = _e255;
            if _e259 {
                phi_4107_ = (1f - abs(((fract((_e255 * 0.5f)) * 2f) + -1f)));
            }
            let _e267 = phi_4107_;
            phi_4106_ = _e267;
        }
        let _e269 = phi_4106_;
        let _e270 = clamp(_e269, 0f, 1f);
        phi_4110_ = _e270;
        if li {
            let _e272 = (_e251.x >> bitcast<u32>(16u));
            phi_4111_ = _e270;
            if (_e272 != 0u) {
                let _e276 = m0_.k2_[_e130];
                if (_e272 == (_e276 >> bitcast<u32>(16i))) {
                    phi_4108_ = min(_e270, unpack2x16float(_e276).x);
                } else {
                    phi_4108_ = 0f;
                }
                let _e284 = phi_4108_;
                phi_4111_ = _e284;
            }
            let _e286 = phi_4111_;
            phi_4110_ = _e286;
        }
        let _e288 = phi_4110_;
        phi_2021_ = mi;
        if mi {
            phi_2021_ = ((_e251.x & 1024u) != 0u);
        }
        let _e292 = phi_2021_;
        phi_4113_ = _e288;
        if _e292 {
            let _e293 = (_e235 * 8u);
            let _e297 = JB.k2_[(_e293 + 2u)];
            let _e308 = JB.k2_[(_e293 + 3u)];
            let _e313 = _e308.zw;
            let _e315 = ((abs(((mat2x2<f32>(vec2<f32>(_e297.x, _e297.y), vec2<f32>(_e297.z, _e297.w)) * _e96) + _e308.xy)) * _e313) - _e313);
            phi_4113_ = min(_e288, clamp((min(_e315.x, _e315.y) + 0.5f), 0f, 1f));
        }
        let _e323 = phi_4113_;
        let _e324 = (_e251.x & 15u);
        let _e327 = ((_e251.x >> bitcast<u32>(4i)) & 15u);
        let _e329 = (ni && (_e327 != 0u));
        if (_e324 <= 1u) {
            let _e334 = (li && (_e324 == 0u));
            phi_4973_ = 0u;
            if _e334 {
                phi_4973_ = (_e251.y | pack2x16float(vec2<f32>(_e323, 0f)));
            }
            let _e339 = phi_4973_;
            phi_4972_ = _e339;
            phi_4143_ = select(unpack4x8unorm(_e251.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e334));
        } else {
            let _e342 = (_e235 * 8u);
            let _e345 = JB.k2_[_e342];
            let _e356 = JB.k2_[(_e342 + 1u)];
            let _e359 = ((mat2x2<f32>(vec2<f32>(_e345.x, _e345.y), vec2<f32>(_e345.z, _e345.w)) * _e96) + _e356.xy);
            if (_e324 == 2u) {
                phi_4112_ = _e359.x;
            } else {
                phi_4112_ = length(_e359);
            }
            let _e364 = phi_4112_;
            let _e371 = bitcast<f32>(_e251.y);
            let _e374 = j.xc;
            let _e377 = j.yc;
            let _e380 = textureSampleLevel(FD, ia, vec2<f32>(((clamp(_e364, 0f, 1f) * _e356.z) + _e356.w), ((floor(_e371) * _e374) + _e377)), 0f);
            phi_4145_ = _e380;
            if !(_e329) {
                let _e384 = (_e380.xyz * _e380.w);
                phi_4145_ = vec4<f32>(_e384.x, _e384.y, _e384.z, (_e380.w * (fract(_e371) * 1.0039216f)));
            }
            let _e393 = phi_4145_;
            phi_4972_ = 0u;
            phi_4143_ = _e393;
        }
        let _e395 = phi_4972_;
        let _e397 = phi_4143_;
        phi_4970_ = _e397;
        if _e329 {
            phi_4957_ = _e397;
            if ((_e397.w * _e323) != 0f) {
                let _e403 = n0_.k2_[_e130];
                let _e404 = unpack4x8unorm(_e403);
                let _e405 = _e397.xyz;
                local_2 = _e405;
                let _e406 = _e404.xyz;
                if (_e404.w != 0f) {
                    phi_4150_ = (1f / _e404.w);
                } else {
                    phi_4150_ = 0f;
                }
                let _e411 = phi_4150_;
                let _e412 = (_e406 * _e411);
                local = _e412;
                switch bitcast<i32>(_e327) {
                    case 11: {
                        let _e414 = local_2;
                        local_1 = (_e414 * _e412);
                        break;
                    }
                    case 1: {
                        let _e416 = local_2;
                        local_1 = ((_e416 + _e412) - (_e416 * _e412));
                        break;
                    }
                    case 2: {
                        let _e420 = local_2;
                        let _e421 = (_e420 * _e412);
                        local_1 = (select(_e421, (((_e420 + _e412) - _e421) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e412 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 3: {
                        let _e428 = local_2;
                        local_1 = min(_e428, _e412);
                        break;
                    }
                    case 4: {
                        let _e430 = local_2;
                        local_1 = max(_e430, _e412);
                        break;
                    }
                    case 5: {
                        let _e433 = clamp(_e406, vec3<f32>(0f, 0f, 0f), _e404.www);
                        let _e439 = vec4<f32>(_e433.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                        let _e445 = vec4<f32>(_e439.x, _e433.y, _e439.z, _e439.w);
                        let _e452 = local_2;
                        let _e455 = (clamp((vec3<f32>(1f, 1f, 1f) - _e452), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e404.w);
                        let _e456 = vec4<f32>(_e445.x, _e445.y, _e433.z, _e445.w).xyz;
                        local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e456 / _e455)), sign(_e456), (_e455 == vec3<f32>(0f, 0f, 0f)));
                        break;
                    }
                    case 6: {
                        let _e462 = local_2;
                        local_2 = clamp(_e462, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        let _e465 = clamp(_e406, vec3<f32>(0f, 0f, 0f), _e404.www);
                        let _e471 = vec4<f32>(_e465.x, _e404.y, _e404.z, _e404.w);
                        let _e477 = vec4<f32>(_e471.x, _e465.y, _e471.z, _e471.w);
                        phi_4764_ = vec4<f32>(_e477.x, _e477.y, _e465.z, _e477.w);
                        if (_e404.w == 0f) {
                            phi_4764_ = vec4<f32>(_e465.x, _e465.y, _e465.z, 1f);
                        }
                        let _e487 = phi_4764_;
                        let _e491 = (vec3(_e487.w) - _e487.xyz);
                        let _e492 = local_2;
                        local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e491 / (_e492 * _e487.w))), sign(_e491), (_e492 == vec3<f32>(0f, 0f, 0f))));
                        break;
                    }
                    case 7: {
                        let _e500 = local_2;
                        let _e501 = (_e500 * _e412);
                        local_1 = (select(_e501, (((_e500 + _e412) - _e501) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e500 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 8: {
                        phi_4684_ = 0i;
                        loop {
                            let _e509 = phi_4684_;
                            if (_e509 < 3i) {
                                let _e512 = local_2[_e509];
                                if (_e512 <= 0.5f) {
                                    let _e515 = local[_e509];
                                    local_1[_e509] = (1f - _e515);
                                } else {
                                    let _e519 = local[_e509];
                                    if (_e519 <= 0.25f) {
                                        let _e521 = local[_e509];
                                        let _e524 = local[_e509];
                                        local_1[_e509] = ((((16f * _e521) - 12f) * _e524) + 3f);
                                    } else {
                                        let _e528 = local[_e509];
                                        local_1[_e509] = (inverseSqrt(_e528) - 1f);
                                    }
                                }
                                continue;
                            } else {
                                break;
                            }
                            continuing {
                                phi_4684_ = (_e509 + 1i);
                            }
                        }
                        let _e533 = local_2;
                        let _e537 = local_1;
                        local_1 = (_e412 + ((_e412 * ((_e533 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e537));
                        break;
                    }
                    case 9: {
                        let _e540 = local_2;
                        local_1 = abs((_e412 - _e540));
                        break;
                    }
                    case 10: {
                        let _e543 = local_2;
                        local_1 = ((_e543 + _e412) - ((_e543 * 2f) * _e412));
                        break;
                    }
                    case 12: {
                        if ri {
                            let _e548 = local_2;
                            let _e549 = clamp(_e548, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e549;
                            let _e564 = (_e549 - vec3(min(min(_e549.x, _e549.y), _e549.z)));
                            let _e572 = (_e564 * ((max(max(_e412.x, _e412.y), _e412.z) - min(min(_e412.x, _e412.y), _e412.z)) / max(0.000062f, max(max(_e564.x, _e564.y), _e564.z))));
                            let _e573 = dot(_e412, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e576 = (_e572 - vec3(dot(_e572, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e589 = (vec2<f32>(_e573, (1f - _e573)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e576.x, _e576.y), _e576.z)), max(max(_e576.x, _e576.y), _e576.z))));
                            local_1 = ((_e576 * min(1f, min(_e589.x, _e589.y))) + vec3(_e573));
                        }
                        break;
                    }
                    case 13: {
                        if ri {
                            let _e597 = local_2;
                            let _e598 = clamp(_e597, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e598;
                            let _e613 = (_e412 - vec3(min(min(_e412.x, _e412.y), _e412.z)));
                            let _e621 = (_e613 * ((max(max(_e598.x, _e598.y), _e598.z) - min(min(_e598.x, _e598.y), _e598.z)) / max(0.000062f, max(max(_e613.x, _e613.y), _e613.z))));
                            let _e622 = dot(_e412, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e625 = (_e621 - vec3(dot(_e621, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e638 = (vec2<f32>(_e622, (1f - _e622)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e625.x, _e625.y), _e625.z)), max(max(_e625.x, _e625.y), _e625.z))));
                            local_1 = ((_e625 * min(1f, min(_e638.x, _e638.y))) + vec3(_e622));
                        }
                        break;
                    }
                    case 14: {
                        if ri {
                            let _e646 = local_2;
                            let _e647 = clamp(_e646, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e647;
                            let _e648 = dot(_e412, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e651 = (_e647 - vec3(dot(_e647, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e664 = (vec2<f32>(_e648, (1f - _e648)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e651.x, _e651.y), _e651.z)), max(max(_e651.x, _e651.y), _e651.z))));
                            local_1 = ((_e651 * min(1f, min(_e664.x, _e664.y))) + vec3(_e648));
                        }
                        break;
                    }
                    case 15: {
                        if ri {
                            let _e672 = local_2;
                            let _e673 = clamp(_e672, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e673;
                            let _e674 = dot(_e673, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e677 = (_e412 - vec3(dot(_e412, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e690 = (vec2<f32>(_e674, (1f - _e674)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e677.x, _e677.y), _e677.z)), max(max(_e677.x, _e677.y), _e677.z))));
                            local_1 = ((_e677 * min(1f, min(_e690.x, _e690.y))) + vec3(_e674));
                        }
                        break;
                    }
                    default: {
                    }
                }
                let _e698 = local_1;
                let _e700 = mix(_e405, _e698, vec3(_e404.w));
                let _e706 = vec4<f32>(_e700.x, _e397.y, _e397.z, _e397.w);
                let _e712 = vec4<f32>(_e706.x, _e700.y, _e706.z, _e706.w);
                phi_4957_ = vec4<f32>(_e712.x, _e712.y, _e700.z, _e712.w);
            }
            let _e720 = phi_4957_;
            let _e723 = (_e720.xyz * _e720.w);
            let _e729 = vec4<f32>(_e723.x, _e720.y, _e720.z, _e720.w);
            let _e735 = vec4<f32>(_e729.x, _e723.y, _e729.z, _e729.w);
            phi_4970_ = vec4<f32>(_e735.x, _e735.y, _e723.z, _e735.w);
        }
        let _e743 = phi_4970_;
        phi_5002_ = _e395;
        phi_4995_ = (_e743 * _e323);
    }
    let _e746 = phi_5002_;
    let _e748 = phi_4995_;
    let _e749 = _e748.xyz;
    let _e752 = j.M3_;
    let _e754 = j.N3_;
    if (si && (_e748.w != 0f)) {
        phi_4997_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e95.x) + (0.00583715f * _e95.y))))) * _e752) + _e754)) + _e749);
    } else {
        phi_4997_ = _e749;
    }
    let _e770 = phi_4997_;
    let _e776 = vec4<f32>(_e770.x, _e748.y, _e748.z, _e748.w);
    let _e782 = vec4<f32>(_e776.x, _e770.y, _e776.z, _e776.w);
    let _e788 = vec4<f32>(_e782.x, _e782.y, _e770.z, _e782.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e770.x + _e770.y) + _e770.z) + _e748.w) == 0f) {
                break;
            }
            let _e794 = (1f - _e748.w);
            phi_4999_ = _e788;
            if (_e794 != 0f) {
                let _e798 = n0_.k2_[_e130];
                phi_4999_ = (_e788 + (unpack4x8unorm(_e798) * _e794));
            }
            let _e803 = phi_4999_;
            n0_.k2_[_e130] = pack4x8unorm(_e803);
            break;
        }
    }
    if (_e746 != 0u) {
        m0_.k2_[_e130] = _e746;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) S: vec4<f32>, @location(1) @interpolate(flat, either) F0_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    S_1 = S;
    F0_1 = F0_;
    main_1();
}
