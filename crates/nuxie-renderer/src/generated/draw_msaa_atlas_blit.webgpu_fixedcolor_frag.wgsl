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

@id(7) override Tg: bool = true;
@id(2) override Og: bool = true;

@group(0) @binding(9) 
var DD: texture_2d<f32>;
@group(3) @binding(9) 
var Bb: sampler;
@group(1) @binding(12) 
var AC: texture_2d<f32>;
@group(1) @binding(14) 
var R5_: sampler;
@group(0) @binding(11) 
var UC: texture_2d<f32>;
@group(3) @binding(11) 
var I9_: sampler;
var<private> C2_1: vec2<f32>;
var<private> i1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0) 
var<uniform> k: NB;
var<private> yg: vec4<f32>;
@group(3) @binding(10) 
var T9_: sampler;
@group(0) @binding(10) 
var QC: texture_2d<f32>;
var<private> I3_1: f32;
var<private> Z1_1: f32;

fn main_1() {
    var phi_604_: vec4<f32>;
    var phi_601_: f32;
    var phi_602_: f32;
    var phi_606_: vec4<f32>;
    var phi_597_: f32;
    var phi_607_: vec4<f32>;
    var phi_605_: vec4<f32>;
    var phi_603_: vec4<f32>;
    var phi_608_: vec3<f32>;

    let _e29 = C2_1;
    let _e30 = textureSampleLevel(UC, I9_, _e29, 0f);
    let _e32 = clamp(_e30.x, 0f, 1f);
    let _e33 = i1_1;
    if (_e33.w >= 0f) {
        if Og {
            phi_604_ = vec4<f32>(_e33.x, _e33.y, _e33.z, (_e33.w * _e32));
        } else {
            phi_604_ = (_e33 * _e32);
        }
        let _e45 = phi_604_;
        phi_603_ = _e45;
    } else {
        if (_e33.w > -1f) {
            if (_e33.z > 0f) {
                phi_601_ = _e33.x;
            } else {
                phi_601_ = length(_e33.xy);
            }
            let _e53 = phi_601_;
            let _e54 = clamp(_e53, 0f, 1f);
            let _e55 = abs(_e33.z);
            if (_e55 > 1f) {
                phi_602_ = ((0.9980469f * _e54) + 0.0009765625f);
            } else {
                phi_602_ = ((0.001953125f * _e54) + _e55);
            }
            let _e62 = phi_602_;
            let _e65 = textureSampleLevel(DD, Bb, vec2<f32>(_e62, -(_e33.w)), 0f);
            let _e67 = (_e65.w * _e32);
            let _e72 = vec4<f32>(_e65.x, _e65.y, _e65.z, _e67);
            if Og {
                phi_606_ = _e72;
            } else {
                let _e74 = (_e72.xyz * _e67);
                phi_606_ = vec4<f32>(_e74.x, _e74.y, _e74.z, _e67);
            }
            let _e80 = phi_606_;
            phi_605_ = _e80;
        } else {
            let _e83 = textureSampleLevel(AC, R5_, _e33.xy, (-2f - _e33.w));
            let _e85 = (_e33.z * _e32);
            if Og {
                if (_e83.w != 0f) {
                    phi_597_ = (1f / _e83.w);
                } else {
                    phi_597_ = 0f;
                }
                let _e91 = phi_597_;
                let _e92 = (_e83.xyz * _e91);
                phi_607_ = vec4<f32>(_e92.x, _e92.y, _e92.z, (_e83.w * _e85));
            } else {
                phi_607_ = (_e83 * _e85);
            }
            let _e100 = phi_607_;
            phi_605_ = _e100;
        }
        let _e102 = phi_605_;
        phi_603_ = _e102;
    }
    let _e104 = phi_603_;
    let _e105 = _e104.xyz;
    let _e106 = gl_FragCoord_1;
    let _e108 = k.y3_;
    let _e110 = k.z3_;
    if Tg {
        phi_608_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e106.x) + (0.00583715f * _e106.y))))) * _e108) + _e110)) + _e105);
    } else {
        phi_608_ = _e105;
    }
    let _e124 = phi_608_;
    let _e130 = vec4<f32>(_e124.x, _e104.y, _e104.z, _e104.w);
    let _e136 = vec4<f32>(_e130.x, _e124.y, _e130.z, _e130.w);
    yg = vec4<f32>(_e136.x, _e136.y, _e124.z, _e136.w);
    return;
}

@fragment 
fn main(@location(1) C2_: vec2<f32>, @location(0) i1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat) I3_: f32, @location(6) @interpolate(flat) Z1_: f32) -> @location(0) vec4<f32> {
    C2_1 = C2_;
    i1_1 = i1_;
    gl_FragCoord_1 = gl_FragCoord;
    I3_1 = I3_;
    Z1_1 = Z1_;
    main_1();
    let _e11 = yg;
    return _e11;
}
