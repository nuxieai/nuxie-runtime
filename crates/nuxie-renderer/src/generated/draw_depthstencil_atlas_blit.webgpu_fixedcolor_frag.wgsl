struct VB {
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
}

@id(7) override dj: bool = true;
@id(2) override Yi: bool = true;
@id(8) override ej: bool = true;

@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var N8_: sampler;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;
@group(0) @binding(10)
var HD: texture_2d<f32>;
@group(3) @binding(10)
var Pa: sampler;
var<private> S2_1: vec2<f32>;
var<private> U0_1: vec3<f32>;
var<private> P0_1: f32;
var<private> O0_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Fi: vec4<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> f4_1: f32;

fn main_1() {
    var phi_805_: f32;
    var phi_820_: vec4<f32>;
    var phi_819_: vec4<f32>;
    var phi_548_: bool;
    var phi_563_: bool;
    var phi_806_: f32;
    var phi_815_: vec4<f32>;
    var phi_822_: vec4<f32>;
    var phi_823_: vec4<f32>;
    var phi_824_: vec3<f32>;

    let _e35 = S2_1;
    let _e36 = textureSampleLevel(HD, Pa, _e35, 0f);
    let _e39 = P0_1;
    let _e41 = U0_1;
    let _e42 = O0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e45 = (Yi && (u32(_e39) != 0u));
            if (_e42.w >= 0f) {
                phi_819_ = _e42;
            } else {
                let _e49 = j.L8_;
                let _e51 = j.M8_;
                let _e54 = abs(_e42.z);
                let _e55 = floor(_e54);
                let _e58 = (((_e54 - _e55) * 8f) + 0.0009765625f);
                if (floor(_e58) == 2f) {
                    phi_805_ = _e42.x;
                } else {
                    phi_805_ = length(_e42.xy);
                }
                let _e67 = phi_805_;
                let _e73 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e67, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e42.z < 0f))) + _e58), ((_e55 * _e49) + _e51)), 0f);
                phi_820_ = _e73;
                if !(_e45) {
                    let _e79 = (_e73.xyz * _e73.w);
                    phi_820_ = vec4<f32>(_e79.x, _e79.y, _e79.z, (_e73.w * (ceil(_e42.w) * -0.003921569f)));
                }
                let _e86 = phi_820_;
                phi_819_ = _e86;
            }
            let _e88 = phi_819_;
            phi_548_ = ej;
            if ej {
                phi_548_ = (_e41.z < 0f);
            }
            let _e92 = phi_548_;
            if _e92 {
                let _e94 = textureSampleLevel(TB, U4_, _e41.xy, 0f);
                phi_823_ = _e94;
                break;
            }
            phi_563_ = ej;
            if ej {
                phi_563_ = (_e41.z > 0f);
            }
            let _e98 = phi_563_;
            phi_822_ = _e88;
            if _e98 {
                let _e102 = textureSampleLevel(TB, U4_, _e41.xy, (_e41.z - 1f));
                phi_815_ = _e102;
                if _e45 {
                    if (_e102.w != 0f) {
                        phi_806_ = (1f / _e102.w);
                    } else {
                        phi_806_ = 0f;
                    }
                    let _e108 = phi_806_;
                    let _e109 = (_e102.xyz * _e108);
                    phi_815_ = vec4<f32>(_e109.x, _e109.y, _e109.z, _e102.w);
                }
                let _e115 = phi_815_;
                phi_822_ = (_e88 * _e115);
            }
            let _e118 = phi_822_;
            phi_823_ = _e118;
            break;
        }
    }
    let _e120 = phi_823_;
    let _e121 = (_e120 * clamp(_e36.x, 0f, 1f));
    let _e122 = _e121.xyz;
    let _e124 = gl_FragCoord_1;
    let _e126 = j.F3_;
    let _e128 = j.G3_;
    if (dj && (_e121.w != 0f)) {
        phi_824_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e124.x) + (0.00583715f * _e124.y))))) * _e126) + _e128)) + _e122);
    } else {
        phi_824_ = _e122;
    }
    let _e144 = phi_824_;
    let _e150 = vec4<f32>(_e144.x, _e121.y, _e121.z, _e121.w);
    let _e156 = vec4<f32>(_e150.x, _e144.y, _e150.z, _e150.w);
    Fi = vec4<f32>(_e156.x, _e156.y, _e144.z, _e156.w);
    return;
}

@fragment
fn main(@location(1) S2_: vec2<f32>, @location(9) U0_: vec3<f32>, @location(6) @interpolate(flat, either) P0_: f32, @location(0) O0_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) f4_: f32) -> @location(0) vec4<f32> {
    S2_1 = S2_;
    U0_1 = U0_;
    P0_1 = P0_;
    O0_1 = O0_;
    gl_FragCoord_1 = gl_FragCoord;
    f4_1 = f4_;
    main_1();
    let _e13 = Fi;
    return _e13;
}
