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

@id(7) override bh: bool = true;
@id(2) override Wg: bool = true;

@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(3) @binding(8)
var Jb: sampler;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;
var<private> f1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: CC;
var<private> Fg: vec4<f32>;
@group(3) @binding(9)
var Z9_: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> U1_1: vec2<f32>;
var<private> e2_1: f32;

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

    let _e26 = f1_1;
    if (_e26.w >= 0f) {
        if Wg {
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
            let _e57 = textureSampleLevel(KD, Jb, vec2<f32>(_e54, -(_e26.w)), 0f);
            let _e63 = vec4<f32>(_e57.x, _e57.y, _e57.z, _e57.w);
            if Wg {
                phi_579_ = _e63;
            } else {
                let _e65 = (_e63.xyz * _e57.w);
                phi_579_ = vec4<f32>(_e65.x, _e65.y, _e65.z, _e57.w);
            }
            let _e71 = phi_579_;
            phi_578_ = _e71;
        } else {
            let _e74 = textureSampleLevel(IC, S5_, _e26.xy, (-2f - _e26.w));
            if Wg {
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
    let _e98 = m.y3_;
    let _e100 = m.z3_;
    if bh {
        phi_581_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e96.x) + (0.00583715f * _e96.y))))) * _e98) + _e100)) + _e95);
    } else {
        phi_581_ = _e95;
    }
    let _e114 = phi_581_;
    let _e120 = vec4<f32>(_e114.x, _e94.y, _e94.z, _e94.w);
    let _e126 = vec4<f32>(_e120.x, _e114.y, _e120.z, _e120.w);
    Fg = vec4<f32>(_e126.x, _e126.y, _e114.z, _e126.w);
    return;
}

@fragment
fn main(@location(0) f1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) U1_: vec2<f32>, @location(6) @interpolate(flat, either) e2_: f32) -> @location(0) vec4<f32> {
    f1_1 = f1_;
    gl_FragCoord_1 = gl_FragCoord;
    U1_1 = U1_;
    e2_1 = e2_;
    main_1();
    let _e9 = Fg;
    return _e9;
}
