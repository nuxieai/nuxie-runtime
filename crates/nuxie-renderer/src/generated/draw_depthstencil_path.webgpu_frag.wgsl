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

@id(8) override nj: bool = true;
@id(7) override mj: bool = true;
@id(9) override oj: bool = true;
@id(16) override vj: bool = false;
@id(17) override wj: bool = false;
@id(2) override hj: bool = true;
@id(4) override jj: bool = true;

var<private> Q0_1: f32;
var<private> P0_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var R8_: sampler;
var<private> V0_1: vec3<f32>;
@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(1) @binding(13)
var W4_: sampler;
@group(0) @binding(12)
var MD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Ni: vec4<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2995_: f32;
    var phi_2996_: f32;
    var phi_2997_: vec2<f32>;
    var phi_2998_: bool;
    var phi_3000_: vec4<f32>;
    var phi_3017_: vec4<f32>;
    var phi_3016_: vec4<f32>;
    var phi_977_: bool;
    var phi_3001_: f32;
    var phi_3013_: vec4<f32>;
    var phi_3019_: vec4<f32>;
    var phi_3020_: f32;
    var phi_3358_: i32;
    var phi_3410_: vec4<f32>;
    var phi_3540_: vec3<f32>;

    let _e66 = Q0_1;
    let _e67 = u32(_e66);
    let _e69 = (hj && (_e67 != 0u));
    let _e71 = P0_1[3u];
    if (_e71 >= 0f) {
        let _e150 = P0_1;
        phi_3016_ = _e150;
    } else {
        let _e73 = P0_1;
        let _e75 = j.P8_;
        let _e77 = j.Q8_;
        let _e80 = abs(_e73.z);
        let _e81 = floor(_e80);
        let _e84 = (((_e80 - _e81) * 8f) + 0.0009765625f);
        let _e86 = ((_e81 * _e75) + _e77);
        let _e87 = floor(_e84);
        switch bitcast<i32>(0u) {
            default: {
                if (oj && (_e87 == 4f)) {
                    let _e106 = normalize(vec2<f32>(_e73.x, -(_e73.y)));
                    let _e109 = acos(clamp(_e106.x, -1f, 1f));
                    if (_e106.y >= 0f) {
                        phi_2996_ = _e109;
                    } else {
                        phi_2996_ = -(_e109);
                    }
                    let _e114 = phi_2996_;
                    phi_2997_ = vec2<f32>((_e114 * 0.15915494f), _e86);
                    break;
                } else {
                    if (_e87 == 2f) {
                        phi_2995_ = _e73.x;
                    } else {
                        phi_2995_ = length(_e73.xy);
                    }
                    let _e96 = phi_2995_;
                    phi_2997_ = vec2<f32>(((clamp(_e96, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e73.z < 0f))) + _e84), _e86);
                    break;
                }
            }
        }
        let _e118 = phi_2997_;
        let _e119 = textureSampleLevel(ZC, R8_, _e118, 0f);
        switch bitcast<i32>(0u) {
            default: {
                if vj {
                    phi_2998_ = true;
                    break;
                }
                if wj {
                    phi_2998_ = true;
                    break;
                }
                phi_2998_ = false;
                break;
            }
        }
        let _e122 = phi_2998_;
        phi_3000_ = _e119;
        if _e122 {
            phi_3000_ = vec4<f32>(_e119.x, _e119.y, _e119.z, (_e119.w * ((fract(_e73.w) * -2f) + 1.5f)));
        }
        let _e135 = phi_3000_;
        phi_3017_ = _e135;
        if !(_e69) {
            let _e139 = (_e135.xyz * _e135.w);
            phi_3017_ = vec4<f32>(_e139.x, _e139.y, _e139.z, (_e135.w * (ceil(_e73.w) * -0.003921569f)));
        }
        let _e149 = phi_3017_;
        phi_3016_ = _e149;
    }
    let _e152 = phi_3016_;
    phi_977_ = jj;
    if jj {
        let _e154 = V0_1[2u];
        phi_977_ = (_e154 > 0f);
    }
    let _e157 = phi_977_;
    phi_3019_ = _e152;
    if _e157 {
        let _e159 = V0_1[2u];
        let _e161 = V0_1;
        let _e163 = textureSampleLevel(UB, W4_, _e161.xy, (_e159 - 1f));
        phi_3013_ = _e163;
        if _e69 {
            if (_e163.w != 0f) {
                phi_3001_ = (1f / _e163.w);
            } else {
                phi_3001_ = 0f;
            }
            let _e169 = phi_3001_;
            let _e170 = (_e163.xyz * _e169);
            phi_3013_ = vec4<f32>(_e170.x, _e170.y, _e170.z, _e163.w);
        }
        let _e176 = phi_3013_;
        phi_3019_ = (_e152 * _e176);
    }
    let _e179 = phi_3019_;
    let _e180 = gl_FragCoord_1;
    let _e184 = textureLoad(MD, vec2<i32>(floor(_e180.xy)), 0i);
    let _e185 = _e179.xyz;
    local_2 = _e185;
    let _e186 = _e184.xyz;
    if (_e184.w != 0f) {
        phi_3020_ = (1f / _e184.w);
    } else {
        phi_3020_ = 0f;
    }
    let _e191 = phi_3020_;
    let _e192 = (_e186 * _e191);
    local = _e192;
    switch bitcast<i32>(_e67) {
        case 11: {
            let _e476 = local_2;
            local_1 = (_e476 * _e192);
            break;
        }
        case 1: {
            let _e472 = local_2;
            local_1 = ((_e472 + _e192) - (_e472 * _e192));
            break;
        }
        case 2: {
            let _e464 = local_2;
            let _e465 = (_e464 * _e192);
            local_1 = (select(_e465, (((_e464 + _e192) - _e465) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e192 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e462 = local_2;
            local_1 = min(_e462, _e192);
            break;
        }
        case 4: {
            let _e460 = local_2;
            local_1 = max(_e460, _e192);
            break;
        }
        case 5: {
            let _e431 = clamp(_e186, vec3<f32>(0f, 0f, 0f), _e184.www);
            let _e437 = vec4<f32>(_e431.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e443 = vec4<f32>(_e437.x, _e431.y, _e437.z, _e437.w);
            let _e450 = local_2;
            let _e453 = (clamp((vec3<f32>(1f, 1f, 1f) - _e450), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e184.w);
            let _e454 = vec4<f32>(_e443.x, _e443.y, _e431.z, _e443.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e454 / _e453)), sign(_e454), (_e453 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e392 = local_2;
            local_2 = clamp(_e392, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e395 = clamp(_e186, vec3<f32>(0f, 0f, 0f), _e184.www);
            let _e401 = vec4<f32>(_e395.x, _e184.y, _e184.z, _e184.w);
            let _e407 = vec4<f32>(_e401.x, _e395.y, _e401.z, _e401.w);
            phi_3410_ = vec4<f32>(_e407.x, _e407.y, _e395.z, _e407.w);
            if (_e184.w == 0f) {
                phi_3410_ = vec4<f32>(_e395.x, _e395.y, _e395.z, 1f);
            }
            let _e417 = phi_3410_;
            let _e421 = (vec3(_e417.w) - _e417.xyz);
            let _e422 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e421 / (_e422 * _e417.w))), sign(_e421), (_e422 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e384 = local_2;
            let _e385 = (_e384 * _e192);
            local_1 = (select(_e385, (((_e384 + _e192) - _e385) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e384 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3358_ = 0i;
            loop {
                let _e353 = phi_3358_;
                if (_e353 < 3i) {
                    let _e356 = local_2[_e353];
                    if (_e356 <= 0.5f) {
                        let _e373 = local[_e353];
                        local_1[_e353] = (1f - _e373);
                    } else {
                        let _e359 = local[_e353];
                        if (_e359 <= 0.25f) {
                            let _e365 = local[_e353];
                            let _e368 = local[_e353];
                            local_1[_e353] = ((((16f * _e365) - 12f) * _e368) + 3f);
                        } else {
                            let _e361 = local[_e353];
                            local_1[_e353] = (inverseSqrt(_e361) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3358_ = (_e353 + 1i);
                }
            }
            let _e377 = local_2;
            let _e381 = local_1;
            local_1 = (_e192 + ((_e192 * ((_e377 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e381));
            break;
        }
        case 9: {
            let _e349 = local_2;
            local_1 = abs((_e192 - _e349));
            break;
        }
        case 10: {
            let _e344 = local_2;
            local_1 = ((_e344 + _e192) - ((_e344 * 2f) * _e192));
            break;
        }
        case 12: {
            if mj {
                let _e295 = local_2;
                let _e296 = clamp(_e295, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e296;
                let _e311 = (_e296 - vec3(min(min(_e296.x, _e296.y), _e296.z)));
                let _e319 = (_e311 * ((max(max(_e192.x, _e192.y), _e192.z) - min(min(_e192.x, _e192.y), _e192.z)) / max(0.000062f, max(max(_e311.x, _e311.y), _e311.z))));
                let _e320 = dot(_e192, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e323 = (_e319 - vec3(dot(_e319, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e336 = (vec2<f32>(_e320, (1f - _e320)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e323.x, _e323.y), _e323.z)), max(max(_e323.x, _e323.y), _e323.z))));
                local_1 = ((_e323 * min(1f, min(_e336.x, _e336.y))) + vec3(_e320));
            }
            break;
        }
        case 13: {
            if mj {
                let _e246 = local_2;
                let _e247 = clamp(_e246, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e247;
                let _e262 = (_e192 - vec3(min(min(_e192.x, _e192.y), _e192.z)));
                let _e270 = (_e262 * ((max(max(_e247.x, _e247.y), _e247.z) - min(min(_e247.x, _e247.y), _e247.z)) / max(0.000062f, max(max(_e262.x, _e262.y), _e262.z))));
                let _e271 = dot(_e192, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e274 = (_e270 - vec3(dot(_e270, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e287 = (vec2<f32>(_e271, (1f - _e271)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e274.x, _e274.y), _e274.z)), max(max(_e274.x, _e274.y), _e274.z))));
                local_1 = ((_e274 * min(1f, min(_e287.x, _e287.y))) + vec3(_e271));
            }
            break;
        }
        case 14: {
            if mj {
                let _e220 = local_2;
                let _e221 = clamp(_e220, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e221;
                let _e222 = dot(_e192, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e225 = (_e221 - vec3(dot(_e221, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e238 = (vec2<f32>(_e222, (1f - _e222)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e225.x, _e225.y), _e225.z)), max(max(_e225.x, _e225.y), _e225.z))));
                local_1 = ((_e225 * min(1f, min(_e238.x, _e238.y))) + vec3(_e222));
            }
            break;
        }
        case 15: {
            if mj {
                let _e194 = local_2;
                let _e195 = clamp(_e194, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e195;
                let _e196 = dot(_e195, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e199 = (_e192 - vec3(dot(_e192, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e212 = (vec2<f32>(_e196, (1f - _e196)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e199.x, _e199.y), _e199.z)), max(max(_e199.x, _e199.y), _e199.z))));
                local_1 = ((_e199 * min(1f, min(_e212.x, _e212.y))) + vec3(_e196));
            }
            break;
        }
        default: {
        }
    }
    let _e478 = local_1;
    let _e480 = mix(_e185, _e478, vec3(_e184.w));
    let _e486 = vec4<f32>(_e480.x, _e179.y, _e179.z, _e179.w);
    let _e492 = vec4<f32>(_e486.x, _e480.y, _e486.z, _e486.w);
    let _e498 = vec4<f32>(_e492.x, _e492.y, _e480.z, _e492.w);
    let _e501 = (_e498.xyz * _e179.w);
    let _e507 = vec4<f32>(_e501.x, _e498.y, _e498.z, _e498.w);
    let _e513 = vec4<f32>(_e507.x, _e501.y, _e507.z, _e507.w);
    let _e519 = vec4<f32>(_e513.x, _e513.y, _e501.z, _e513.w);
    let _e520 = _e519.xyz;
    let _e521 = gl_FragCoord_1;
    let _e523 = j.G3_;
    let _e525 = j.H3_;
    if (nj && (_e179.w != 0f)) {
        phi_3540_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e521.x) + (0.00583715f * _e521.y))))) * _e523) + _e525)) + _e520);
    } else {
        phi_3540_ = _e520;
    }
    let _e541 = phi_3540_;
    let _e547 = vec4<f32>(_e541.x, _e519.y, _e519.z, _e519.w);
    let _e553 = vec4<f32>(_e547.x, _e541.y, _e547.z, _e547.w);
    Ni = vec4<f32>(_e553.x, _e553.y, _e541.z, _e553.w);
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) Q0_: f32, @location(0) P0_: vec4<f32>, @location(2) V0_: vec3<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    Q0_1 = Q0_;
    P0_1 = P0_;
    V0_1 = V0_;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e9 = Ni;
    return _e9;
}
