struct CC {
    ec: f32,
    od: f32,
    ff: f32,
    gf: f32,
    m6_: u32,
    Gg: u32,
    Re: u32,
    Se: u32,
    R7_: vec4<i32>,
    Cg: vec2<f32>,
    pd: vec2<f32>,
    a2_: u32,
    Hg: f32,
    Z5_: u32,
    P2_: f32,
    qd: f32,
    Me: u32,
    z3_: f32,
    A3_: f32,
    rd: f32,
    zg: u32,
}

@id(7) override gh: bool = true;
@id(2) override bh: bool = true;

@group(0) @binding(8)
var LD: texture_2d<f32>;
@group(3) @binding(8)
var Mb: sampler;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;
@group(0) @binding(10)
var BD: texture_2d<f32>;
@group(3) @binding(10)
var Q9_: sampler;
var<private> C2_1: vec2<f32>;
var<private> X0_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> n: CC;
var<private> Kg: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> I3_1: f32;
var<private> e2_1: f32;

fn main_1() {
    var phi_616_: vec4<f32>;
    var phi_613_: f32;
    var phi_614_: f32;
    var phi_618_: vec4<f32>;
    var phi_609_: f32;
    var phi_619_: vec4<f32>;
    var phi_617_: vec4<f32>;
    var phi_615_: vec4<f32>;
    var phi_620_: vec3<f32>;

    let _e29 = C2_1;
    let _e30 = textureSampleLevel(BD, Q9_, _e29, 0f);
    let _e32 = clamp(_e30.x, 0f, 1f);
    let _e33 = X0_1;
    if (_e33.w >= 0f) {
        if bh {
            phi_616_ = vec4<f32>(_e33.x, _e33.y, _e33.z, (_e33.w * _e32));
        } else {
            phi_616_ = (_e33 * _e32);
        }
        let _e45 = phi_616_;
        phi_615_ = _e45;
    } else {
        if (_e33.w > -1f) {
            if (_e33.z > 0f) {
                phi_613_ = _e33.x;
            } else {
                phi_613_ = length(_e33.xy);
            }
            let _e53 = phi_613_;
            let _e54 = clamp(_e53, 0f, 1f);
            let _e55 = abs(_e33.z);
            if (_e55 > 1f) {
                phi_614_ = ((0.9980469f * _e54) + 0.0009765625f);
            } else {
                phi_614_ = ((0.001953125f * _e54) + _e55);
            }
            let _e62 = phi_614_;
            let _e65 = textureSampleLevel(LD, Mb, vec2<f32>(_e62, -(_e33.w)), 0f);
            let _e67 = (_e65.w * _e32);
            let _e72 = vec4<f32>(_e65.x, _e65.y, _e65.z, _e67);
            if bh {
                phi_618_ = _e72;
            } else {
                let _e74 = (_e72.xyz * _e67);
                phi_618_ = vec4<f32>(_e74.x, _e74.y, _e74.z, _e67);
            }
            let _e80 = phi_618_;
            phi_617_ = _e80;
        } else {
            let _e83 = textureSampleLevel(IC, S5_, _e33.xy, (-2f - _e33.w));
            let _e85 = (_e33.z * _e32);
            if bh {
                if (_e83.w != 0f) {
                    phi_609_ = (1f / _e83.w);
                } else {
                    phi_609_ = 0f;
                }
                let _e91 = phi_609_;
                let _e92 = (_e83.xyz * _e91);
                phi_619_ = vec4<f32>(_e92.x, _e92.y, _e92.z, (_e83.w * _e85));
            } else {
                phi_619_ = (_e83 * _e85);
            }
            let _e100 = phi_619_;
            phi_617_ = _e100;
        }
        let _e102 = phi_617_;
        phi_615_ = _e102;
    }
    let _e104 = phi_615_;
    let _e105 = _e104.xyz;
    let _e107 = gl_FragCoord_1;
    let _e109 = n.z3_;
    let _e111 = n.A3_;
    if (gh && (_e104.w != 0f)) {
        phi_620_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e107.x) + (0.00583715f * _e107.y))))) * _e109) + _e111)) + _e105);
    } else {
        phi_620_ = _e105;
    }
    let _e127 = phi_620_;
    let _e133 = vec4<f32>(_e127.x, _e104.y, _e104.z, _e104.w);
    let _e139 = vec4<f32>(_e133.x, _e127.y, _e133.z, _e133.w);
    Kg = vec4<f32>(_e139.x, _e139.y, _e127.z, _e139.w);
    return;
}

@fragment
fn main(@location(1) C2_: vec2<f32>, @location(0) X0_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) I3_: f32, @location(6) @interpolate(flat, either) e2_: f32) -> @location(0) vec4<f32> {
    C2_1 = C2_;
    X0_1 = X0_;
    gl_FragCoord_1 = gl_FragCoord;
    I3_1 = I3_;
    e2_1 = e2_;
    main_1();
    let _e11 = Kg;
    return _e11;
}
