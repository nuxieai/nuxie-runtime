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
var<private> i1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0) 
var<uniform> k: NB;
var<private> yg: vec4<f32>;
@group(3) @binding(10) 
var T9_: sampler;
@group(0) @binding(10) 
var QC: texture_2d<f32>;
var<private> S1_1: vec2<f32>;
var<private> Z1_1: f32;

fn main_1() {
    var phi_577_: vec4<f32>;
    var phi_574_: f32;
    var phi_575_: f32;
    var phi_579_: vec4<f32>;
    var phi_570_: f32;
    var phi_580_: vec4<f32>;
    var phi_578_: vec4<f32>;
    var phi_576_: vec4<f32>;
    var phi_581_: vec3<f32>;

    let _e26 = i1_1;
    if (_e26.w >= 0f) {
        if Og {
            phi_577_ = vec4<f32>(_e26.x, _e26.y, _e26.z, _e26.w);
        } else {
            phi_577_ = (_e26 * 1f);
        }
        let _e37 = phi_577_;
        phi_576_ = _e37;
    } else {
        if (_e26.w > -1f) {
            if (_e26.z > 0f) {
                phi_574_ = _e26.x;
            } else {
                phi_574_ = length(_e26.xy);
            }
            let _e45 = phi_574_;
            let _e46 = clamp(_e45, 0f, 1f);
            let _e47 = abs(_e26.z);
            if (_e47 > 1f) {
                phi_575_ = ((0.9980469f * _e46) + 0.0009765625f);
            } else {
                phi_575_ = ((0.001953125f * _e46) + _e47);
            }
            let _e54 = phi_575_;
            let _e57 = textureSampleLevel(DD, Bb, vec2<f32>(_e54, -(_e26.w)), 0f);
            let _e63 = vec4<f32>(_e57.x, _e57.y, _e57.z, _e57.w);
            if Og {
                phi_579_ = _e63;
            } else {
                let _e65 = (_e63.xyz * _e57.w);
                phi_579_ = vec4<f32>(_e65.x, _e65.y, _e65.z, _e57.w);
            }
            let _e71 = phi_579_;
            phi_578_ = _e71;
        } else {
            let _e74 = textureSampleLevel(AC, R5_, _e26.xy, (-2f - _e26.w));
            if Og {
                if (_e74.w != 0f) {
                    phi_570_ = (1f / _e74.w);
                } else {
                    phi_570_ = 0f;
                }
                let _e81 = phi_570_;
                let _e82 = (_e74.xyz * _e81);
                phi_580_ = vec4<f32>(_e82.x, _e82.y, _e82.z, (_e74.w * _e26.z));
            } else {
                phi_580_ = (_e74 * _e26.z);
            }
            let _e90 = phi_580_;
            phi_578_ = _e90;
        }
        let _e92 = phi_578_;
        phi_576_ = _e92;
    }
    let _e94 = phi_576_;
    let _e95 = _e94.xyz;
    let _e96 = gl_FragCoord_1;
    let _e98 = k.y3_;
    let _e100 = k.z3_;
    if Tg {
        phi_581_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e96.x) + (0.00583715f * _e96.y))))) * _e98) + _e100)) + _e95);
    } else {
        phi_581_ = _e95;
    }
    let _e114 = phi_581_;
    let _e120 = vec4<f32>(_e114.x, _e94.y, _e94.z, _e94.w);
    let _e126 = vec4<f32>(_e120.x, _e114.y, _e120.z, _e120.w);
    yg = vec4<f32>(_e126.x, _e126.y, _e114.z, _e126.w);
    return;
}

@fragment 
fn main(@location(0) i1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat) S1_: vec2<f32>, @location(6) @interpolate(flat) Z1_: f32) -> @location(0) vec4<f32> {
    i1_1 = i1_;
    gl_FragCoord_1 = gl_FragCoord;
    S1_1 = S1_;
    Z1_1 = Z1_;
    main_1();
    let _e9 = yg;
    return _e9;
}
