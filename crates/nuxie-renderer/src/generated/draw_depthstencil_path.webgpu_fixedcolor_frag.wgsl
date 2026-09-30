struct SB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    eh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    ih: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    bh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

@id(7) override Lh: bool = true;
@id(2) override Gh: bool = true;
@id(8) override Mh: bool = true;

@group(0) @binding(0)
var<uniform> j: SB;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var M9_: sampler;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var W5_: sampler;
var<private> C2_1: vec3<f32>;
var<private> g1_1: f32;
var<private> X1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> lh: vec4<f32>;
@group(3) @binding(9)
var ca: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> Y1_1: vec2<f32>;

fn main_1() {
    var phi_661_: f32;
    var phi_662_: f32;
    var phi_676_: vec4<f32>;
    var phi_675_: vec4<f32>;
    var phi_469_: bool;
    var phi_663_: f32;
    var phi_672_: vec4<f32>;
    var phi_678_: vec4<f32>;
    var phi_679_: vec3<f32>;

    let _e30 = g1_1;
    let _e32 = C2_1;
    let _e33 = X1_1;
    let _e35 = (Gh && (u32(_e30) != 0u));
    if (_e33.w >= 0f) {
        phi_675_ = _e33;
    } else {
        let _e38 = -(_e33.w);
        let _e43 = j.Zb;
        let _e46 = j.ac;
        if (_e33.z > 0f) {
            phi_661_ = _e33.x;
        } else {
            phi_661_ = length(_e33.xy);
        }
        let _e54 = phi_661_;
        let _e55 = clamp(_e54, 0f, 1f);
        let _e56 = abs(_e33.z);
        if (_e56 > 1f) {
            phi_662_ = ((0.9980469f * _e55) + 0.0009765625f);
        } else {
            phi_662_ = ((0.001953125f * _e55) + _e56);
        }
        let _e63 = phi_662_;
        let _e65 = textureSampleLevel(DD, M9_, vec2<f32>(_e63, ((floor(_e38) * _e43) + _e46)), 0f);
        phi_676_ = _e65;
        if !(_e35) {
            let _e69 = (_e65.xyz * _e65.w);
            phi_676_ = vec4<f32>(_e69.x, _e69.y, _e69.z, (_e65.w * (fract(_e38) * 1.0039216f)));
        }
        let _e76 = phi_676_;
        phi_675_ = _e76;
    }
    let _e78 = phi_675_;
    phi_469_ = Mh;
    if Mh {
        phi_469_ = (_e32.z > 0f);
    }
    let _e82 = phi_469_;
    phi_678_ = _e78;
    if _e82 {
        let _e86 = textureSampleLevel(GC, W5_, _e32.xy, (_e32.z - 1f));
        phi_672_ = _e86;
        if _e35 {
            if (_e86.w != 0f) {
                phi_663_ = (1f / _e86.w);
            } else {
                phi_663_ = 0f;
            }
            let _e92 = phi_663_;
            let _e93 = (_e86.xyz * _e92);
            phi_672_ = vec4<f32>(_e93.x, _e93.y, _e93.z, _e86.w);
        }
        let _e99 = phi_672_;
        phi_678_ = (_e78 * _e99);
    }
    let _e102 = phi_678_;
    let _e103 = (_e102 * 1f);
    let _e104 = _e103.xyz;
    let _e106 = gl_FragCoord_1;
    let _e108 = j.F3_;
    let _e110 = j.G3_;
    if (Lh && (_e103.w != 0f)) {
        phi_679_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e106.x) + (0.00583715f * _e106.y))))) * _e108) + _e110)) + _e104);
    } else {
        phi_679_ = _e104;
    }
    let _e126 = phi_679_;
    let _e132 = vec4<f32>(_e126.x, _e103.y, _e103.z, _e103.w);
    let _e138 = vec4<f32>(_e132.x, _e126.y, _e132.z, _e132.w);
    lh = vec4<f32>(_e138.x, _e138.y, _e126.z, _e138.w);
    return;
}

@fragment
fn main(@location(9) C2_: vec3<f32>, @location(6) @interpolate(flat, either) g1_: f32, @location(0) X1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) Y1_: vec2<f32>) -> @location(0) vec4<f32> {
    C2_1 = C2_;
    g1_1 = g1_;
    X1_1 = X1_;
    gl_FragCoord_1 = gl_FragCoord;
    Y1_1 = Y1_;
    main_1();
    let _e11 = lh;
    return _e11;
}
