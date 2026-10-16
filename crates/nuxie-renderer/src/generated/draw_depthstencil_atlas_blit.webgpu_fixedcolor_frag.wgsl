struct VB {
    yd: f32,
    Ee: f32,
    Jg: f32,
    Kg: f32,
    S6_: u32,
    Ea: u32,
    vg: u32,
    wg: u32,
    H8_: vec4<i32>,
    Ci: vec2<f32>,
    Fe: vec2<f32>,
    q2_: u32,
    Gi: f32,
    w6_: u32,
    i3_: f32,
    Ge: f32,
    og: u32,
    G3_: f32,
    H3_: f32,
    He: f32,
    zi: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

@id(7) override hj: bool = true;
@id(2) override cj: bool = true;
@id(8) override ij: bool = true;

@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var Q8_: sampler;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;
@group(0) @binding(10)
var JD: texture_2d<f32>;
@group(3) @binding(10)
var Sa: sampler;
var<private> T2_1: vec2<f32>;
var<private> U0_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> P0_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Ji: vec4<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
var<private> f4_1: f32;

fn main_1() {
    var phi_906_: f32;
    var phi_907_: f32;
    var phi_908_: vec2<f32>;
    var phi_925_: vec4<f32>;
    var phi_924_: vec4<f32>;
    var phi_600_: bool;
    var phi_615_: bool;
    var phi_909_: f32;
    var phi_920_: vec4<f32>;
    var phi_927_: vec4<f32>;
    var phi_928_: vec4<f32>;
    var phi_929_: vec3<f32>;

    let _e38 = T2_1;
    let _e39 = textureSampleLevel(JD, Sa, _e38, 0f);
    let _e42 = Q0_1;
    let _e44 = U0_1;
    let _e45 = P0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e48 = (cj && (u32(_e42) != 0u));
            if (_e45.w >= 0f) {
                phi_924_ = _e45;
            } else {
                let _e52 = j.O8_;
                let _e54 = j.P8_;
                let _e57 = abs(_e45.z);
                let _e58 = floor(_e57);
                let _e61 = (((_e57 - _e58) * 8f) + 0.0009765625f);
                let _e63 = ((_e58 * _e52) + _e54);
                let _e64 = floor(_e61);
                switch bitcast<i32>(0u) {
                    default: {
                        if (_e64 == 4f) {
                            let _e82 = normalize(vec2<f32>(_e45.x, -(_e45.y)));
                            let _e85 = acos(clamp(_e82.x, -1f, 1f));
                            if (_e82.y >= 0f) {
                                phi_907_ = _e85;
                            } else {
                                phi_907_ = -(_e85);
                            }
                            let _e90 = phi_907_;
                            phi_908_ = vec2<f32>((_e90 * 0.15915507f), _e63);
                            break;
                        } else {
                            if (_e64 == 2f) {
                                phi_906_ = _e45.x;
                            } else {
                                phi_906_ = length(_e45.xy);
                            }
                            let _e72 = phi_906_;
                            phi_908_ = vec2<f32>(((clamp(_e72, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e45.z < 0f))) + _e61), _e63);
                            break;
                        }
                    }
                }
                let _e94 = phi_908_;
                let _e95 = textureSampleLevel(ZC, Q8_, _e94, 0f);
                phi_925_ = _e95;
                if !(_e48) {
                    let _e101 = (_e95.xyz * _e95.w);
                    phi_925_ = vec4<f32>(_e101.x, _e101.y, _e101.z, (_e95.w * (ceil(_e45.w) * -0.003921569f)));
                }
                let _e108 = phi_925_;
                phi_924_ = _e108;
            }
            let _e110 = phi_924_;
            phi_600_ = ij;
            if ij {
                phi_600_ = (_e44.z < 0f);
            }
            let _e114 = phi_600_;
            if _e114 {
                let _e116 = textureSampleLevel(TB, U4_, _e44.xy, 0f);
                phi_928_ = _e116;
                break;
            }
            phi_615_ = ij;
            if ij {
                phi_615_ = (_e44.z > 0f);
            }
            let _e120 = phi_615_;
            phi_927_ = _e110;
            if _e120 {
                let _e124 = textureSampleLevel(TB, U4_, _e44.xy, (_e44.z - 1f));
                phi_920_ = _e124;
                if _e48 {
                    if (_e124.w != 0f) {
                        phi_909_ = (1f / _e124.w);
                    } else {
                        phi_909_ = 0f;
                    }
                    let _e130 = phi_909_;
                    let _e131 = (_e124.xyz * _e130);
                    phi_920_ = vec4<f32>(_e131.x, _e131.y, _e131.z, _e124.w);
                }
                let _e137 = phi_920_;
                phi_927_ = (_e110 * _e137);
            }
            let _e140 = phi_927_;
            phi_928_ = _e140;
            break;
        }
    }
    let _e142 = phi_928_;
    let _e143 = (_e142 * clamp(_e39.x, 0f, 1f));
    let _e144 = _e143.xyz;
    let _e146 = gl_FragCoord_1;
    let _e148 = j.G3_;
    let _e150 = j.H3_;
    if (hj && (_e143.w != 0f)) {
        phi_929_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e146.x) + (0.00583715f * _e146.y))))) * _e148) + _e150)) + _e144);
    } else {
        phi_929_ = _e144;
    }
    let _e166 = phi_929_;
    let _e172 = vec4<f32>(_e166.x, _e143.y, _e143.z, _e143.w);
    let _e178 = vec4<f32>(_e172.x, _e166.y, _e172.z, _e172.w);
    Ji = vec4<f32>(_e178.x, _e178.y, _e166.z, _e178.w);
    return;
}

@fragment
fn main(@location(1) T2_: vec2<f32>, @location(9) U0_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) P0_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) f4_: f32) -> @location(0) vec4<f32> {
    T2_1 = T2_;
    U0_1 = U0_;
    Q0_1 = Q0_;
    P0_1 = P0_;
    gl_FragCoord_1 = gl_FragCoord;
    f4_1 = f4_;
    main_1();
    let _e13 = Ji;
    return _e13;
}
