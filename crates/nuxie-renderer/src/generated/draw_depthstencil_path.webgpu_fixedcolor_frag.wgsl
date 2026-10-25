struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
    P8_: f32,
    Q8_: f32,
}

@id(8) override nj: bool = true;
@id(9) override oj: bool = true;
@id(16) override vj: bool = false;
@id(17) override wj: bool = false;
@id(2) override hj: bool = true;
@id(4) override jj: bool = true;

var<private> Q0_1: f32;
var<private> P0_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var R8_: sampler;
var<private> V0_1: vec3<f32>;
@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(1) @binding(13)
var W4_: sampler;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Ni: vec4<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;

fn main_1() {
    var phi_782_: f32;
    var phi_783_: f32;
    var phi_784_: vec2<f32>;
    var phi_785_: bool;
    var phi_787_: vec4<f32>;
    var phi_804_: vec4<f32>;
    var phi_803_: vec4<f32>;
    var phi_414_: bool;
    var phi_788_: f32;
    var phi_800_: vec4<f32>;
    var phi_806_: vec4<f32>;
    var phi_807_: vec3<f32>;

    let _e43 = Q0_1;
    let _e46 = (hj && (u32(_e43) != 0u));
    let _e48 = P0_1[3u];
    if (_e48 >= 0f) {
        let _e127 = P0_1;
        phi_803_ = _e127;
    } else {
        let _e50 = P0_1;
        let _e52 = j.P8_;
        let _e54 = j.Q8_;
        let _e57 = abs(_e50.z);
        let _e58 = floor(_e57);
        let _e61 = (((_e57 - _e58) * 8f) + 0.0009765625f);
        let _e63 = ((_e58 * _e52) + _e54);
        let _e64 = floor(_e61);
        switch bitcast<i32>(0u) {
            default: {
                if (oj && (_e64 == 4f)) {
                    let _e83 = normalize(vec2<f32>(_e50.x, -(_e50.y)));
                    let _e86 = acos(clamp(_e83.x, -1f, 1f));
                    if (_e83.y >= 0f) {
                        phi_783_ = _e86;
                    } else {
                        phi_783_ = -(_e86);
                    }
                    let _e91 = phi_783_;
                    phi_784_ = vec2<f32>((_e91 * 0.15915494f), _e63);
                    break;
                } else {
                    if (_e64 == 2f) {
                        phi_782_ = _e50.x;
                    } else {
                        phi_782_ = length(_e50.xy);
                    }
                    let _e73 = phi_782_;
                    phi_784_ = vec2<f32>(((clamp(_e73, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e50.z < 0f))) + _e61), _e63);
                    break;
                }
            }
        }
        let _e95 = phi_784_;
        let _e96 = textureSampleLevel(ZC, R8_, _e95, 0f);
        switch bitcast<i32>(0u) {
            default: {
                if vj {
                    phi_785_ = true;
                    break;
                }
                if wj {
                    phi_785_ = true;
                    break;
                }
                phi_785_ = false;
                break;
            }
        }
        let _e99 = phi_785_;
        phi_787_ = _e96;
        if _e99 {
            phi_787_ = vec4<f32>(_e96.x, _e96.y, _e96.z, (_e96.w * ((fract(_e50.w) * -2f) + 1.5f)));
        }
        let _e112 = phi_787_;
        phi_804_ = _e112;
        if !(_e46) {
            let _e116 = (_e112.xyz * _e112.w);
            phi_804_ = vec4<f32>(_e116.x, _e116.y, _e116.z, (_e112.w * (ceil(_e50.w) * -0.003921569f)));
        }
        let _e126 = phi_804_;
        phi_803_ = _e126;
    }
    let _e129 = phi_803_;
    phi_414_ = jj;
    if jj {
        let _e131 = V0_1[2u];
        phi_414_ = (_e131 > 0f);
    }
    let _e134 = phi_414_;
    phi_806_ = _e129;
    if _e134 {
        let _e136 = V0_1[2u];
        let _e138 = V0_1;
        let _e140 = textureSampleLevel(UB, W4_, _e138.xy, (_e136 - 1f));
        phi_800_ = _e140;
        if _e46 {
            if (_e140.w != 0f) {
                phi_788_ = (1f / _e140.w);
            } else {
                phi_788_ = 0f;
            }
            let _e146 = phi_788_;
            let _e147 = (_e140.xyz * _e146);
            phi_800_ = vec4<f32>(_e147.x, _e147.y, _e147.z, _e140.w);
        }
        let _e153 = phi_800_;
        phi_806_ = (_e129 * _e153);
    }
    let _e156 = phi_806_;
    let _e157 = _e156.xyz;
    let _e159 = gl_FragCoord_1;
    let _e161 = j.G3_;
    let _e163 = j.H3_;
    if (nj && (_e156.w != 0f)) {
        phi_807_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e159.x) + (0.00583715f * _e159.y))))) * _e161) + _e163)) + _e157);
    } else {
        phi_807_ = _e157;
    }
    let _e179 = phi_807_;
    let _e185 = vec4<f32>(_e179.x, _e156.y, _e156.z, _e156.w);
    let _e191 = vec4<f32>(_e185.x, _e179.y, _e185.z, _e185.w);
    Ni = vec4<f32>(_e191.x, _e191.y, _e179.z, _e191.w);
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) Q0_: f32, @location(0) P0_: vec4<f32>, @location(2) V0_: vec3<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    Q0_1 = Q0_;
    P0_1 = P0_;
    V0_1 = V0_;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e9 = Ni;
    return _e9;
}
