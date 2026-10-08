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
@id(15) override nj: bool = false;
@id(16) override oj: bool = false;
@id(2) override aj: bool = true;
@id(8) override gj: bool = true;

var<private> Q0_1: f32;
var<private> P0_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var I8_: sampler;
var<private> V0_1: vec3<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var S4_: sampler;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Ii: vec4<f32>;
@group(3) @binding(9)
var Va: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;

fn main_1() {
    var phi_625_: f32;
    var phi_621_: f32;
    var phi_623_: vec2<f32>;
    var phi_626_: bool;
    var phi_628_: vec4<f32>;
    var phi_644_: vec4<f32>;
    var phi_643_: vec4<f32>;
    var phi_342_: bool;
    var phi_629_: f32;
    var phi_640_: vec4<f32>;
    var phi_646_: vec4<f32>;
    var phi_647_: vec3<f32>;

    let _e41 = Q0_1;
    let _e44 = (aj && (u32(_e41) != 0u));
    let _e46 = P0_1[3u];
    if (_e46 >= 0f) {
        let _e48 = P0_1;
        phi_643_ = _e48;
    } else {
        let _e49 = P0_1;
        let _e51 = j.Db;
        let _e53 = j.g7_;
        let _e55 = bitcast<u32>(_e49.w);
        let _e59 = ((f32((_e55 & 268304384u)) * _e51) - _e53);
        let _e61 = abs(_e49.z);
        if (_e61 < 1.5f) {
            phi_625_ = _e61;
            phi_621_ = _e49.x;
        } else {
            phi_625_ = (_e61 - 2f);
            phi_621_ = length(_e49.xy);
        }
        let _e68 = phi_625_;
        let _e70 = phi_621_;
        let _e71 = clamp(_e70, 0f, 1f);
        if (_e49.z < 0f) {
            phi_623_ = vec2<f32>(((_e71 * 0.9980469f) + 0.0009765625f), _e59);
        } else {
            phi_623_ = vec2<f32>(((_e71 * 0.001953125f) + ((f32((_e55 & 130816u)) * 0.0000076293945f) + 0.0009765625f)), _e59);
        }
        let _e84 = phi_623_;
        let _e85 = textureSampleLevel(YC, I8_, _e84, 0f);
        switch bitcast<i32>(0u) {
            default: {
                if nj {
                    phi_626_ = true;
                    break;
                }
                if oj {
                    phi_626_ = true;
                    break;
                }
                phi_626_ = false;
                break;
            }
        }
        let _e88 = phi_626_;
        phi_628_ = _e85;
        if _e88 {
            phi_628_ = vec4<f32>(_e85.x, _e85.y, _e85.z, (_e85.w * _e68));
        }
        let _e97 = phi_628_;
        phi_644_ = _e97;
        if !(_e44) {
            let _e101 = (_e97.xyz * _e97.w);
            phi_644_ = vec4<f32>(_e101.x, _e101.y, _e101.z, (_e97.w * (f32((_e55 & 255u)) * 0.003921569f)));
        }
        let _e111 = phi_644_;
        phi_643_ = _e111;
    }
    let _e113 = phi_643_;
    phi_342_ = gj;
    if gj {
        let _e115 = V0_1[2u];
        phi_342_ = (_e115 > 0f);
    }
    let _e118 = phi_342_;
    phi_646_ = _e113;
    if _e118 {
        let _e120 = V0_1[2u];
        let _e122 = V0_1;
        let _e124 = textureSampleLevel(TB, S4_, _e122.xy, (_e120 - 1f));
        phi_640_ = _e124;
        if _e44 {
            if (_e124.w != 0f) {
                phi_629_ = (1f / _e124.w);
            } else {
                phi_629_ = 0f;
            }
            let _e130 = phi_629_;
            let _e131 = (_e124.xyz * _e130);
            phi_640_ = vec4<f32>(_e131.x, _e131.y, _e131.z, _e124.w);
        }
        let _e137 = phi_640_;
        phi_646_ = (_e113 * _e137);
    }
    let _e140 = phi_646_;
    let _e141 = _e140.xyz;
    let _e143 = gl_FragCoord_1;
    let _e145 = j.E3_;
    let _e147 = j.F3_;
    if (fj && (_e140.w != 0f)) {
        phi_647_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e143.x) + (0.00583715f * _e143.y))))) * _e145) + _e147)) + _e141);
    } else {
        phi_647_ = _e141;
    }
    let _e163 = phi_647_;
    let _e169 = vec4<f32>(_e163.x, _e140.y, _e140.z, _e140.w);
    let _e175 = vec4<f32>(_e169.x, _e163.y, _e169.z, _e169.w);
    Ii = vec4<f32>(_e175.x, _e175.y, _e163.z, _e175.w);
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) Q0_: f32, @location(0) P0_: vec4<f32>, @location(2) V0_: vec3<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    Q0_1 = Q0_;
    P0_1 = P0_;
    V0_1 = V0_;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e9 = Ii;
    return _e9;
}
