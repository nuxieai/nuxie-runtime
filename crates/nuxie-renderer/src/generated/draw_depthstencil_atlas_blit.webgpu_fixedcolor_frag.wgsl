struct UB {
    Rc: f32,
    Ud: f32,
    dg: f32,
    eg: f32,
    A6_: u32,
    Y9_: u32,
    Pf: u32,
    Qf: u32,
    i8_: vec4<i32>,
    Nh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Rh: f32,
    U4_: u32,
    a3_: f32,
    Wd: f32,
    Jf: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Kh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

@id(7) override ti: bool = true;
@id(2) override oi: bool = true;
@id(8) override ui: bool = true;

@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var ia: sampler;
@group(1) @binding(11)
var CC: texture_2d<f32>;
@group(1) @binding(13)
var r5_: sampler;
@group(0) @binding(10)
var FD: texture_2d<f32>;
@group(3) @binding(10)
var na: sampler;
var<private> J2_1: vec2<f32>;
var<private> v1_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> a1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Uh: vec4<f32>;
@group(3) @binding(9)
var xa: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> Z3_1: f32;

fn main_1() {
    var phi_736_: f32;
    var phi_737_: f32;
    var phi_753_: vec4<f32>;
    var phi_752_: vec4<f32>;
    var phi_522_: bool;
    var phi_537_: bool;
    var phi_738_: f32;
    var phi_748_: vec4<f32>;
    var phi_755_: vec4<f32>;
    var phi_756_: vec4<f32>;
    var phi_757_: vec3<f32>;

    let _e33 = J2_1;
    let _e34 = textureSampleLevel(FD, na, _e33, 0f);
    let _e37 = Q0_1;
    let _e39 = v1_1;
    let _e40 = a1_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e43 = (oi && (u32(_e37) != 0u));
            if (_e40.w >= 0f) {
                phi_752_ = _e40;
            } else {
                let _e46 = -(_e40.w);
                let _e51 = j.xc;
                let _e54 = j.yc;
                if (_e40.z > 0f) {
                    phi_736_ = _e40.x;
                } else {
                    phi_736_ = length(_e40.xy);
                }
                let _e62 = phi_736_;
                let _e63 = clamp(_e62, 0f, 1f);
                let _e64 = abs(_e40.z);
                if (_e64 > 1f) {
                    phi_737_ = ((0.9980469f * _e63) + 0.0009765625f);
                } else {
                    phi_737_ = ((0.001953125f * _e63) + _e64);
                }
                let _e71 = phi_737_;
                let _e73 = textureSampleLevel(ED, ia, vec2<f32>(_e71, ((floor(_e46) * _e51) + _e54)), 0f);
                phi_753_ = _e73;
                if !(_e43) {
                    let _e77 = (_e73.xyz * _e73.w);
                    phi_753_ = vec4<f32>(_e77.x, _e77.y, _e77.z, (_e73.w * (fract(_e46) * 1.0039216f)));
                }
                let _e84 = phi_753_;
                phi_752_ = _e84;
            }
            let _e86 = phi_752_;
            phi_522_ = ui;
            if ui {
                phi_522_ = (_e39.z < 0f);
            }
            let _e90 = phi_522_;
            if _e90 {
                let _e92 = textureSampleLevel(CC, r5_, _e39.xy, 0f);
                phi_756_ = _e92;
                break;
            }
            phi_537_ = ui;
            if ui {
                phi_537_ = (_e39.z > 0f);
            }
            let _e96 = phi_537_;
            phi_755_ = _e86;
            if _e96 {
                let _e100 = textureSampleLevel(CC, r5_, _e39.xy, (_e39.z - 1f));
                phi_748_ = _e100;
                if _e43 {
                    if (_e100.w != 0f) {
                        phi_738_ = (1f / _e100.w);
                    } else {
                        phi_738_ = 0f;
                    }
                    let _e106 = phi_738_;
                    let _e107 = (_e100.xyz * _e106);
                    phi_748_ = vec4<f32>(_e107.x, _e107.y, _e107.z, _e100.w);
                }
                let _e113 = phi_748_;
                phi_755_ = (_e86 * _e113);
            }
            let _e116 = phi_755_;
            phi_756_ = _e116;
            break;
        }
    }
    let _e118 = phi_756_;
    let _e119 = (_e118 * clamp(_e34.x, 0f, 1f));
    let _e120 = _e119.xyz;
    let _e122 = gl_FragCoord_1;
    let _e124 = j.M3_;
    let _e126 = j.N3_;
    if (ti && (_e119.w != 0f)) {
        phi_757_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e122.x) + (0.00583715f * _e122.y))))) * _e124) + _e126)) + _e120);
    } else {
        phi_757_ = _e120;
    }
    let _e142 = phi_757_;
    let _e148 = vec4<f32>(_e142.x, _e119.y, _e119.z, _e119.w);
    let _e154 = vec4<f32>(_e148.x, _e142.y, _e148.z, _e148.w);
    Uh = vec4<f32>(_e154.x, _e154.y, _e142.z, _e154.w);
    return;
}

@fragment
fn main(@location(1) J2_: vec2<f32>, @location(9) v1_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) a1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) Z3_: f32) -> @location(0) vec4<f32> {
    J2_1 = J2_;
    v1_1 = v1_;
    Q0_1 = Q0_;
    a1_1 = a1_;
    gl_FragCoord_1 = gl_FragCoord;
    Z3_1 = Z3_;
    main_1();
    let _e13 = Uh;
    return _e13;
}
