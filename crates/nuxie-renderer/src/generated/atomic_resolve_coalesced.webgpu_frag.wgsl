struct Ne {
    d2_: array<vec2<u32>>,
}

struct h0Ed {
    d2_: array<u32>,
}

struct Oe {
    d2_: array<vec4<f32>>,
}

struct j0Ed {
    d2_: array<u32>,
}

struct DC {
    hc: f32,
    rd: f32,
    kf: f32,
    lf: f32,
    p6_: u32,
    Mg: u32,
    Ve: u32,
    We: u32,
    U7_: vec4<i32>,
    Ig: vec2<f32>,
    sd: vec2<f32>,
    c2_: u32,
    Ng: f32,
    d6_: u32,
    R2_: f32,
    td: f32,
    Qe: u32,
    B3_: f32,
    C3_: f32,
    ud: f32,
    Fg: u32,
}

struct v4Ed {
    d2_: array<u32>,
}

@id(6) override lh: bool = true;
@id(4) override jh: bool = true;
@id(0) override fh: bool = true;
@id(1) override gh: bool = true;
@id(2) override hh: bool = true;

@group(0) @binding(3)
var<storage> BD: Ne;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Ed;
@group(0) @binding(4)
var<storage> RB: Oe;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(3) @binding(8)
var Pb: sampler;
@group(2) @binding(0)
var<storage, read_write> j0_: j0Ed;
@group(0) @binding(0)
var<uniform> m: DC;
@group(2) @binding(3)
var<storage, read_write> v4_: v4Ed;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(1) @binding(13)
var V5_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1236_: bool;
    var phi_3142_: f32;
    var phi_3141_: f32;
    var phi_3143_: f32;
    var phi_3146_: f32;
    var phi_3145_: f32;
    var phi_1273_: bool;
    var phi_3159_: f32;
    var phi_3147_: f32;
    var phi_3161_: vec4<f32>;
    var phi_1386_: bool;
    var phi_3165_: u32;
    var phi_1395_: bool;
    var phi_3179_: f32;
    var phi_3634_: vec4<f32>;
    var phi_3574_: i32;
    var phi_3782_: vec4<f32>;
    var phi_3783_: vec4<f32>;

    let _e68 = gl_FragCoord_1;
    let _e69 = _e68.xy;
    let _e72 = bitcast<vec2<u32>>(vec2<i32>(floor(_e69)));
    let _e74 = m.p6_;
    let _e103 = bitcast<i32>((((((_e72.y >> bitcast<u32>(5u)) * (((_e74 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e72.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e72.x & 28u) << bitcast<u32>(5u)) + ((_e72.y & 28u) << bitcast<u32>(2i)))) + (((_e72.y & 3u) << bitcast<u32>(2i)) + (_e72.x & 3u))));
    let _e106 = v4_.d2_[_e103];
    let _e110 = ((f32((_e106 & 131071u)) * 0.00048828125f) + -32f);
    let _e112 = (_e106 >> bitcast<u32>(17u));
    let _e115 = BD.d2_[_e112];
    phi_3141_ = _e110;
    if ((_e115.x & 768u) != 0u) {
        let _e119 = abs(_e110);
        phi_1236_ = jh;
        if jh {
            phi_1236_ = ((_e115.x & 512u) != 0u);
        }
        let _e123 = phi_1236_;
        phi_3142_ = _e119;
        if _e123 {
            phi_3142_ = (1f - abs(((fract((_e119 * 0.5f)) * 2f) + -1f)));
        }
        let _e131 = phi_3142_;
        phi_3141_ = _e131;
    }
    let _e133 = phi_3141_;
    let _e134 = clamp(_e133, 0f, 1f);
    phi_3145_ = _e134;
    if fh {
        let _e136 = (_e115.x >> bitcast<u32>(16u));
        phi_3146_ = _e134;
        if (_e136 != 0u) {
            let _e140 = h0_.d2_[_e103];
            if (_e136 == (_e140 >> bitcast<u32>(16i))) {
                phi_3143_ = min(_e134, unpack2x16float(_e140).x);
            } else {
                phi_3143_ = 0f;
            }
            let _e148 = phi_3143_;
            phi_3146_ = _e148;
        }
        let _e150 = phi_3146_;
        phi_3145_ = _e150;
    }
    let _e152 = phi_3145_;
    phi_1273_ = gh;
    if gh {
        phi_1273_ = ((_e115.x & 1024u) != 0u);
    }
    let _e156 = phi_1273_;
    phi_3159_ = _e152;
    if _e156 {
        let _e157 = (_e112 * 8u);
        let _e161 = RB.d2_[(_e157 + 2u)];
        let _e172 = RB.d2_[(_e157 + 3u)];
        let _e177 = _e172.zw;
        let _e179 = ((abs(((mat2x2<f32>(vec2<f32>(_e161.x, _e161.y), vec2<f32>(_e161.z, _e161.w)) * _e69) + _e172.xy)) * _e177) - _e177);
        phi_3159_ = min(_e152, clamp((min(_e179.x, _e179.y) + 0.5f), 0f, 1f));
    }
    let _e187 = phi_3159_;
    let _e188 = (_e115.x & 15u);
    if (_e188 <= 1u) {
        phi_3161_ = select(unpack4x8unorm(_e115.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((fh && (_e188 == 0u))));
    } else {
        let _e196 = (_e112 * 8u);
        let _e199 = RB.d2_[_e196];
        let _e210 = RB.d2_[(_e196 + 1u)];
        let _e213 = ((mat2x2<f32>(vec2<f32>(_e199.x, _e199.y), vec2<f32>(_e199.z, _e199.w)) * _e69) + _e210.xy);
        if (_e188 == 2u) {
            phi_3147_ = _e213.x;
        } else {
            phi_3147_ = length(_e213);
        }
        let _e218 = phi_3147_;
        let _e227 = textureSampleLevel(MD, Pb, vec2<f32>(((clamp(_e218, 0f, 1f) * _e210.z) + _e210.w), bitcast<f32>(_e115.y)), 0f);
        phi_3161_ = _e227;
    }
    let _e229 = phi_3161_;
    let _e231 = (_e229.w * _e187);
    let _e236 = vec4<f32>(_e229.x, _e229.y, _e229.z, _e231);
    phi_1386_ = hh;
    if hh {
        phi_1386_ = (_e231 != 0f);
    }
    let _e239 = phi_1386_;
    phi_3165_ = u32();
    phi_1395_ = _e239;
    if _e239 {
        let _e242 = ((_e115.x >> bitcast<u32>(4i)) & 15u);
        phi_3165_ = _e242;
        phi_1395_ = (_e242 != 0u);
    }
    let _e245 = phi_3165_;
    let _e247 = phi_1395_;
    phi_3782_ = _e236;
    if _e247 {
        let _e250 = j0_.d2_[_e103];
        let _e251 = unpack4x8unorm(_e250);
        let _e252 = _e236.xyz;
        local_2 = _e252;
        let _e253 = _e251.xyz;
        if (_e251.w != 0f) {
            phi_3179_ = (1f / _e251.w);
        } else {
            phi_3179_ = 0f;
        }
        let _e258 = phi_3179_;
        let _e259 = (_e253 * _e258);
        local = _e259;
        switch bitcast<i32>(_e245) {
            case 11: {
                let _e261 = local_2;
                local_1 = (_e261 * _e259);
                break;
            }
            case 1: {
                let _e263 = local_2;
                local_1 = ((_e263 + _e259) - (_e263 * _e259));
                break;
            }
            case 2: {
                let _e267 = local_2;
                let _e268 = (_e267 * _e259);
                local_1 = (select(_e268, (((_e267 + _e259) - _e268) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e259 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e275 = local_2;
                local_1 = min(_e275, _e259);
                break;
            }
            case 4: {
                let _e277 = local_2;
                local_1 = max(_e277, _e259);
                break;
            }
            case 5: {
                let _e280 = clamp(_e253, vec3<f32>(0f, 0f, 0f), _e251.www);
                let _e286 = vec4<f32>(_e280.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e292 = vec4<f32>(_e286.x, _e280.y, _e286.z, _e286.w);
                let _e299 = local_2;
                let _e302 = (clamp((vec3<f32>(1f, 1f, 1f) - _e299), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e251.w);
                let _e303 = vec4<f32>(_e292.x, _e292.y, _e280.z, _e292.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e303 / _e302)), sign(_e303), (_e302 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e309 = local_2;
                local_2 = clamp(_e309, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e312 = clamp(_e253, vec3<f32>(0f, 0f, 0f), _e251.www);
                let _e318 = vec4<f32>(_e312.x, _e251.y, _e251.z, _e251.w);
                let _e324 = vec4<f32>(_e318.x, _e312.y, _e318.z, _e318.w);
                phi_3634_ = vec4<f32>(_e324.x, _e324.y, _e312.z, _e324.w);
                if (_e251.w == 0f) {
                    phi_3634_ = vec4<f32>(_e312.x, _e312.y, _e312.z, 1f);
                }
                let _e334 = phi_3634_;
                let _e338 = (vec3(_e334.w) - _e334.xyz);
                let _e339 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e338 / (_e339 * _e334.w))), sign(_e338), (_e339 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e347 = local_2;
                let _e348 = (_e347 * _e259);
                local_1 = (select(_e348, (((_e347 + _e259) - _e348) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e347 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_3574_ = 0i;
                loop {
                    let _e356 = phi_3574_;
                    if (_e356 < 3i) {
                        let _e359 = local_2[_e356];
                        if (_e359 <= 0.5f) {
                            let _e362 = local[_e356];
                            local_1[_e356] = (1f - _e362);
                        } else {
                            let _e366 = local[_e356];
                            if (_e366 <= 0.25f) {
                                let _e368 = local[_e356];
                                let _e371 = local[_e356];
                                local_1[_e356] = ((((16f * _e368) - 12f) * _e371) + 3f);
                            } else {
                                let _e375 = local[_e356];
                                local_1[_e356] = (inverseSqrt(_e375) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_3574_ = (_e356 + 1i);
                    }
                }
                let _e380 = local_2;
                let _e384 = local_1;
                local_1 = (_e259 + ((_e259 * ((_e380 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e384));
                break;
            }
            case 9: {
                let _e387 = local_2;
                local_1 = abs((_e259 - _e387));
                break;
            }
            case 10: {
                let _e390 = local_2;
                local_1 = ((_e390 + _e259) - ((_e390 * 2f) * _e259));
                break;
            }
            case 12: {
                if lh {
                    let _e395 = local_2;
                    let _e396 = clamp(_e395, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e396;
                    let _e411 = (_e396 - vec3(min(min(_e396.x, _e396.y), _e396.z)));
                    let _e419 = (_e411 * ((max(max(_e259.x, _e259.y), _e259.z) - min(min(_e259.x, _e259.y), _e259.z)) / max(0.000062f, max(max(_e411.x, _e411.y), _e411.z))));
                    let _e420 = dot(_e259, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e423 = (_e419 - vec3(dot(_e419, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e436 = (vec2<f32>(_e420, (1f - _e420)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e423.x, _e423.y), _e423.z)), max(max(_e423.x, _e423.y), _e423.z))));
                    local_1 = ((_e423 * min(1f, min(_e436.x, _e436.y))) + vec3(_e420));
                }
                break;
            }
            case 13: {
                if lh {
                    let _e444 = local_2;
                    let _e445 = clamp(_e444, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e445;
                    let _e460 = (_e259 - vec3(min(min(_e259.x, _e259.y), _e259.z)));
                    let _e468 = (_e460 * ((max(max(_e445.x, _e445.y), _e445.z) - min(min(_e445.x, _e445.y), _e445.z)) / max(0.000062f, max(max(_e460.x, _e460.y), _e460.z))));
                    let _e469 = dot(_e259, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e472 = (_e468 - vec3(dot(_e468, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e485 = (vec2<f32>(_e469, (1f - _e469)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e472.x, _e472.y), _e472.z)), max(max(_e472.x, _e472.y), _e472.z))));
                    local_1 = ((_e472 * min(1f, min(_e485.x, _e485.y))) + vec3(_e469));
                }
                break;
            }
            case 14: {
                if lh {
                    let _e493 = local_2;
                    let _e494 = clamp(_e493, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e494;
                    let _e495 = dot(_e259, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e498 = (_e494 - vec3(dot(_e494, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e511 = (vec2<f32>(_e495, (1f - _e495)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e498.x, _e498.y), _e498.z)), max(max(_e498.x, _e498.y), _e498.z))));
                    local_1 = ((_e498 * min(1f, min(_e511.x, _e511.y))) + vec3(_e495));
                }
                break;
            }
            case 15: {
                if lh {
                    let _e519 = local_2;
                    let _e520 = clamp(_e519, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e520;
                    let _e521 = dot(_e520, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e524 = (_e259 - vec3(dot(_e259, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e537 = (vec2<f32>(_e521, (1f - _e521)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e524.x, _e524.y), _e524.z)), max(max(_e524.x, _e524.y), _e524.z))));
                    local_1 = ((_e524 * min(1f, min(_e537.x, _e537.y))) + vec3(_e521));
                }
                break;
            }
            default: {
            }
        }
        let _e545 = local_1;
        let _e547 = mix(_e252, _e545, vec3(_e251.w));
        phi_3782_ = vec4<f32>(_e547.x, _e547.y, _e547.z, _e231);
    }
    let _e553 = phi_3782_;
    let _e556 = (_e553.xyz * _e553.w);
    let _e562 = vec4<f32>(_e556.x, _e553.y, _e553.z, _e553.w);
    let _e568 = vec4<f32>(_e562.x, _e556.y, _e562.z, _e562.w);
    let _e574 = vec4<f32>(_e568.x, _e568.y, _e556.z, _e568.w);
    let _e575 = (1f - _e553.w);
    phi_3783_ = _e574;
    if (_e575 != 0f) {
        let _e579 = j0_.d2_[_e103];
        phi_3783_ = (_e574 + (unpack4x8unorm(_e579) * _e575));
    }
    let _e584 = phi_3783_;
    C1_ = _e584;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = C1_;
    return _e3;
}
