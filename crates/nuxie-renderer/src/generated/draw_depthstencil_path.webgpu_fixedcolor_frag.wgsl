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

@id(7) override si: bool = true;
@id(2) override ni: bool = true;
@id(8) override ti: bool = true;

@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var FD: texture_2d<f32>;
@group(3) @binding(8)
var ia: sampler;
@group(1) @binding(11)
var DC: texture_2d<f32>;
@group(1) @binding(13)
var v5_: sampler;
var<private> r1_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> a1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Th: vec4<f32>;
@group(3) @binding(9)
var xa: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
var<private> l1_1: vec2<f32>;

fn main_1() {
    var phi_716_: f32;
    var phi_717_: f32;
    var phi_733_: vec4<f32>;
    var phi_732_: vec4<f32>;
    var phi_502_: bool;
    var phi_517_: bool;
    var phi_718_: f32;
    var phi_728_: vec4<f32>;
    var phi_735_: vec4<f32>;
    var phi_736_: vec4<f32>;
    var phi_737_: vec3<f32>;

    let _e30 = Q0_1;
    let _e32 = r1_1;
    let _e33 = a1_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e36 = (ni && (u32(_e30) != 0u));
            if (_e33.w >= 0f) {
                phi_732_ = _e33;
            } else {
                let _e39 = -(_e33.w);
                let _e44 = j.xc;
                let _e47 = j.yc;
                if (_e33.z > 0f) {
                    phi_716_ = _e33.x;
                } else {
                    phi_716_ = length(_e33.xy);
                }
                let _e55 = phi_716_;
                let _e56 = clamp(_e55, 0f, 1f);
                let _e57 = abs(_e33.z);
                if (_e57 > 1f) {
                    phi_717_ = ((0.9980469f * _e56) + 0.0009765625f);
                } else {
                    phi_717_ = ((0.001953125f * _e56) + _e57);
                }
                let _e64 = phi_717_;
                let _e66 = textureSampleLevel(FD, ia, vec2<f32>(_e64, ((floor(_e39) * _e44) + _e47)), 0f);
                phi_733_ = _e66;
                if !(_e36) {
                    let _e70 = (_e66.xyz * _e66.w);
                    phi_733_ = vec4<f32>(_e70.x, _e70.y, _e70.z, (_e66.w * (fract(_e39) * 1.0039216f)));
                }
                let _e77 = phi_733_;
                phi_732_ = _e77;
            }
            let _e79 = phi_732_;
            phi_502_ = ti;
            if ti {
                phi_502_ = (_e32.z < 0f);
            }
            let _e83 = phi_502_;
            if _e83 {
                let _e85 = textureSampleLevel(DC, v5_, _e32.xy, 0f);
                phi_736_ = _e85;
                break;
            }
            phi_517_ = ti;
            if ti {
                phi_517_ = (_e32.z > 0f);
            }
            let _e89 = phi_517_;
            phi_735_ = _e79;
            if _e89 {
                let _e93 = textureSampleLevel(DC, v5_, _e32.xy, (_e32.z - 1f));
                phi_728_ = _e93;
                if _e36 {
                    if (_e93.w != 0f) {
                        phi_718_ = (1f / _e93.w);
                    } else {
                        phi_718_ = 0f;
                    }
                    let _e99 = phi_718_;
                    let _e100 = (_e93.xyz * _e99);
                    phi_728_ = vec4<f32>(_e100.x, _e100.y, _e100.z, _e93.w);
                }
                let _e106 = phi_728_;
                phi_735_ = (_e79 * _e106);
            }
            let _e109 = phi_735_;
            phi_736_ = _e109;
            break;
        }
    }
    let _e111 = phi_736_;
    let _e112 = (_e111 * 1f);
    let _e113 = _e112.xyz;
    let _e115 = gl_FragCoord_1;
    let _e117 = j.M3_;
    let _e119 = j.N3_;
    if (si && (_e112.w != 0f)) {
        phi_737_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e115.x) + (0.00583715f * _e115.y))))) * _e117) + _e119)) + _e113);
    } else {
        phi_737_ = _e113;
    }
    let _e135 = phi_737_;
    let _e141 = vec4<f32>(_e135.x, _e112.y, _e112.z, _e112.w);
    let _e147 = vec4<f32>(_e141.x, _e135.y, _e141.z, _e141.w);
    Th = vec4<f32>(_e147.x, _e147.y, _e135.z, _e147.w);
    return;
}

@fragment
fn main(@location(9) r1_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) a1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) l1_: vec2<f32>) -> @location(0) vec4<f32> {
    r1_1 = r1_;
    Q0_1 = Q0_;
    a1_1 = a1_;
    gl_FragCoord_1 = gl_FragCoord;
    l1_1 = l1_;
    main_1();
    let _e11 = Th;
    return _e11;
}
