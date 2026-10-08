struct VB {
    vd: f32,
    Ce: f32,
    Gg: f32,
    Hg: f32,
    L6_: u32,
    xa: u32,
    sg: u32,
    tg: u32,
    C8_: vec4<i32>,
    Bi: vec2<f32>,
    De: vec2<f32>,
    r2_: u32,
    Fi: f32,
    p6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    E3_: f32,
    F3_: f32,
    Fe: f32,
    yi: u32,
    wa: u32,
    cd: f32,
    g7_: f32,
    Db: f32,
}

@id(7) override fj: bool = true;
@id(2) override aj: bool = true;
@id(8) override gj: bool = true;

@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var I8_: sampler;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var S4_: sampler;
@group(0) @binding(10)
var HD: texture_2d<f32>;
@group(3) @binding(10)
var La: sampler;
var<private> T2_1: vec2<f32>;
var<private> V0_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> P0_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Ii: vec4<f32>;
@group(3) @binding(9)
var Va: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
var<private> e4_1: f32;

fn main_1() {
    var phi_738_: f32;
    var phi_739_: f32;
    var phi_755_: vec4<f32>;
    var phi_754_: vec4<f32>;
    var phi_524_: bool;
    var phi_539_: bool;
    var phi_740_: f32;
    var phi_750_: vec4<f32>;
    var phi_757_: vec4<f32>;
    var phi_758_: vec4<f32>;
    var phi_759_: vec3<f32>;

    let _e33 = T2_1;
    let _e34 = textureSampleLevel(HD, La, _e33, 0f);
    let _e37 = Q0_1;
    let _e39 = V0_1;
    let _e40 = P0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e43 = (aj && (u32(_e37) != 0u));
            if (_e40.w >= 0f) {
                phi_754_ = _e40;
            } else {
                let _e46 = -(_e40.w);
                let _e51 = j.cd;
                let _e54 = j.g7_;
                if (_e40.z > 0f) {
                    phi_738_ = _e40.x;
                } else {
                    phi_738_ = length(_e40.xy);
                }
                let _e62 = phi_738_;
                let _e63 = clamp(_e62, 0f, 1f);
                let _e64 = abs(_e40.z);
                if (_e64 > 1f) {
                    phi_739_ = ((0.9980469f * _e63) + 0.0009765625f);
                } else {
                    phi_739_ = ((0.001953125f * _e63) + _e64);
                }
                let _e71 = phi_739_;
                let _e73 = textureSampleLevel(YC, I8_, vec2<f32>(_e71, ((floor(_e46) * _e51) + _e54)), 0f);
                phi_755_ = _e73;
                if !(_e43) {
                    let _e77 = (_e73.xyz * _e73.w);
                    phi_755_ = vec4<f32>(_e77.x, _e77.y, _e77.z, (_e73.w * (fract(_e46) * 1.0039216f)));
                }
                let _e84 = phi_755_;
                phi_754_ = _e84;
            }
            let _e86 = phi_754_;
            phi_524_ = gj;
            if gj {
                phi_524_ = (_e39.z < 0f);
            }
            let _e90 = phi_524_;
            if _e90 {
                let _e92 = textureSampleLevel(TB, S4_, _e39.xy, 0f);
                phi_758_ = _e92;
                break;
            }
            phi_539_ = gj;
            if gj {
                phi_539_ = (_e39.z > 0f);
            }
            let _e96 = phi_539_;
            phi_757_ = _e86;
            if _e96 {
                let _e100 = textureSampleLevel(TB, S4_, _e39.xy, (_e39.z - 1f));
                phi_750_ = _e100;
                if _e43 {
                    if (_e100.w != 0f) {
                        phi_740_ = (1f / _e100.w);
                    } else {
                        phi_740_ = 0f;
                    }
                    let _e106 = phi_740_;
                    let _e107 = (_e100.xyz * _e106);
                    phi_750_ = vec4<f32>(_e107.x, _e107.y, _e107.z, _e100.w);
                }
                let _e113 = phi_750_;
                phi_757_ = (_e86 * _e113);
            }
            let _e116 = phi_757_;
            phi_758_ = _e116;
            break;
        }
    }
    let _e118 = phi_758_;
    let _e119 = (_e118 * clamp(_e34.x, 0f, 1f));
    let _e120 = _e119.xyz;
    let _e122 = gl_FragCoord_1;
    let _e124 = j.E3_;
    let _e126 = j.F3_;
    if (fj && (_e119.w != 0f)) {
        phi_759_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e122.x) + (0.00583715f * _e122.y))))) * _e124) + _e126)) + _e120);
    } else {
        phi_759_ = _e120;
    }
    let _e142 = phi_759_;
    let _e148 = vec4<f32>(_e142.x, _e119.y, _e119.z, _e119.w);
    let _e154 = vec4<f32>(_e148.x, _e142.y, _e148.z, _e148.w);
    Ii = vec4<f32>(_e154.x, _e154.y, _e142.z, _e154.w);
    return;
}

@fragment
fn main(@location(1) T2_: vec2<f32>, @location(9) V0_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) P0_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) e4_: f32) -> @location(0) vec4<f32> {
    T2_1 = T2_;
    V0_1 = V0_;
    Q0_1 = Q0_;
    P0_1 = P0_;
    gl_FragCoord_1 = gl_FragCoord;
    e4_1 = e4_;
    main_1();
    let _e13 = Ii;
    return _e13;
}
