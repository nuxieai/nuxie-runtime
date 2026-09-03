struct ke {
    d2_: array<u32>,
}

struct DC {
    hc: f32,
    rd: f32,
    kf: f32,
    lf: f32,
    p6_: u32,
    Mg: u32,
    Ve: u32,
    We: u32,
    U7_: vec4<i32>,
    Ig: vec2<f32>,
    sd: vec2<f32>,
    c2_: u32,
    Ng: f32,
    d6_: u32,
    R2_: f32,
    td: f32,
    Qe: u32,
    B3_: f32,
    C3_: f32,
    ud: f32,
    Fg: u32,
}

struct ke_1 {
    d2_: array<atomic<u32>>,
}

@id(7) override mh: bool = true;
@id(8) override nh: bool = true;
@id(1) override gh: bool = true;
@id(0) override fh: bool = true;

@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(3) @binding(8)
var Pb: sampler;
@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(1) @binding(13)
var V5_: sampler;
@group(0) @binding(6)
var<storage, read_write> P0_: ke_1;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> f1_1: vec4<f32>;
var<private> A2_1: vec3<f32>;
var<private> i1_1: f32;
var<private> n4_1: vec2<f32>;
var<private> f3_1: vec2<u32>;
var<private> M0_1: vec4<f32>;
var<private> V1_1: vec2<f32>;
@group(2) @binding(1)
var h0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> B0_1: f32;
var<private> f2_1: f32;

fn main_1() {
    var phi_1195_: f32;
    var phi_1196_: f32;
    var phi_1207_: vec4<f32>;
    var phi_803_: bool;
    var phi_1197_: f32;
    var phi_1208_: vec4<f32>;
    var phi_1210_: f32;
    var phi_617_: bool;
    var phi_1211_: f32;
    var phi_962_: bool;
    var phi_964_: bool;
    var phi_1234_: f32;
    var phi_1229_: u32;
    var phi_1226_: f32;
    var phi_1233_: f32;
    var phi_1228_: u32;
    var phi_1225_: f32;
    var phi_1230_: f32;
    var phi_1227_: u32;
    var phi_1224_: f32;
    var phi_1238_: f32;
    var phi_1240_: f32;
    var phi_1248_: f32;
    var phi_1251_: f32;
    var phi_1269_: vec3<f32>;

    let _e49 = f1_1;
    let _e50 = A2_1;
    if (_e49.w >= 0f) {
        phi_1207_ = vec4<f32>(_e49.x, _e49.y, _e49.z, _e49.w);
    } else {
        if (_e49.z > 0f) {
            phi_1195_ = _e49.x;
        } else {
            phi_1195_ = length(_e49.xy);
        }
        let _e59 = phi_1195_;
        let _e60 = clamp(_e59, 0f, 1f);
        let _e61 = abs(_e49.z);
        if (_e61 > 1f) {
            phi_1196_ = ((0.9980469f * _e60) + 0.0009765625f);
        } else {
            phi_1196_ = ((0.001953125f * _e60) + _e61);
        }
        let _e68 = phi_1196_;
        let _e71 = textureSampleLevel(MD, Pb, vec2<f32>(_e68, -(_e49.w)), 0f);
        phi_1207_ = vec4<f32>(_e71.x, _e71.y, _e71.z, _e71.w);
    }
    let _e85 = phi_1207_;
    phi_803_ = nh;
    if nh {
        phi_803_ = (_e50.z > 0f);
    }
    let _e89 = phi_803_;
    phi_1208_ = _e85;
    if _e89 {
        let _e93 = textureSampleLevel(JC, V5_, _e50.xy, (_e50.z - 1f));
        if (_e93.w != 0f) {
            phi_1197_ = (1f / _e93.w);
        } else {
            phi_1197_ = 0f;
        }
        let _e99 = phi_1197_;
        let _e100 = (_e93.xyz * _e99);
        phi_1208_ = (_e85 * vec4<f32>(_e100.x, _e100.y, _e100.z, _e93.w));
    }
    let _e107 = phi_1208_;
    let _e108 = i1_1;
    let _e109 = n4_1;
    let _e112 = f3_1[1u];
    let _e114 = f3_1[0u];
    let _e115 = vec2<u32>(floor(_e109));
    phi_1210_ = 1f;
    if gh {
        let _e143 = M0_1;
        let _e146 = min(_e143.xy, _e143.zw);
        phi_1210_ = min(min(_e146.x, _e146.y), 1f);
    }
    let _e152 = phi_1210_;
    phi_617_ = fh;
    if fh {
        let _e154 = V1_1[0u];
        phi_617_ = (_e154 != 0f);
    }
    let _e157 = phi_617_;
    phi_1211_ = _e152;
    if _e157 {
        let _e158 = gl_FragCoord_1;
        let _e162 = textureLoad(h0_, vec2<i32>(floor(_e158.xy)), 0i);
        phi_1211_ = min(_e162.x, _e152);
    }
    let _e166 = phi_1211_;
    let _e168 = clamp(_e108, 0f, max(_e166, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e174 = u32(((abs(_e168) * 1024f) + 0.5f));
            let _e177 = atomicLoad((&P0_.d2_[(_e114 + (((((_e115.y >> bitcast<u32>(5u)) * (_e112 << bitcast<u32>(5u))) + ((_e115.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e115.x & 28u) << bitcast<u32>(5u)) + ((_e115.y & 28u) << bitcast<u32>(2i)))) + (((_e115.y & 3u) << bitcast<u32>(2i)) + (_e115.x & 3u))))]));
            let _e179 = (min(_e107.w, _e168) >= 1f);
            phi_964_ = _e179;
            if _e179 {
                let _e181 = m.c2_;
                let _e182 = (_e177 < _e181);
                phi_962_ = _e182;
                if !(_e182) {
                    phi_962_ = (_e177 >= (_e181 | 262144u));
                }
                let _e187 = phi_962_;
                phi_964_ = _e187;
            }
            let _e189 = phi_964_;
            if _e189 {
                phi_1240_ = _e107.w;
                break;
            }
            let _e191 = m.c2_;
            phi_1230_ = 0f;
            phi_1227_ = _e174;
            phi_1224_ = _e168;
            if (_e177 < _e191) {
                let _e194 = (_e191 | (262144u + _e174));
                let _e195 = atomicMax((&P0_.d2_[(_e114 + (((((_e115.y >> bitcast<u32>(5u)) * (_e112 << bitcast<u32>(5u))) + ((_e115.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e115.x & 28u) << bitcast<u32>(5u)) + ((_e115.y & 28u) << bitcast<u32>(2i)))) + (((_e115.y & 3u) << bitcast<u32>(2i)) + (_e115.x & 3u))))]), _e194);
                if (_e195 <= _e191) {
                    phi_1233_ = min(_e168, 1f);
                    phi_1228_ = _e174;
                    phi_1225_ = 0f;
                } else {
                    phi_1234_ = 0f;
                    phi_1229_ = _e174;
                    phi_1226_ = _e168;
                    if (_e195 < _e194) {
                        let _e199 = ((_e195 & 524287u) - 262144u);
                        let _e201 = (f32(_e199) * 0.0009765625f);
                        phi_1234_ = ((min(_e168, 1f) - _e201) / max((1f - (_e201 * _e107.w)), 0.000062f));
                        phi_1229_ = _e199;
                        phi_1226_ = _e201;
                    }
                    let _e209 = phi_1234_;
                    let _e211 = phi_1229_;
                    let _e213 = phi_1226_;
                    phi_1233_ = _e209;
                    phi_1228_ = _e211;
                    phi_1225_ = _e213;
                }
                let _e216 = phi_1233_;
                let _e218 = phi_1228_;
                let _e220 = phi_1225_;
                phi_1230_ = _e216;
                phi_1227_ = _e218;
                phi_1224_ = _e220;
            }
            let _e222 = phi_1230_;
            let _e224 = phi_1227_;
            let _e226 = phi_1224_;
            phi_1238_ = _e222;
            if (_e226 > 0f) {
                let _e228 = atomicAdd((&P0_.d2_[(_e114 + (((((_e115.y >> bitcast<u32>(5u)) * (_e112 << bitcast<u32>(5u))) + ((_e115.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e115.x & 28u) << bitcast<u32>(5u)) + ((_e115.y & 28u) << bitcast<u32>(2i)))) + (((_e115.y & 3u) << bitcast<u32>(2i)) + (_e115.x & 3u))))]), _e224);
                let _e233 = (f32(bitcast<i32>(((_e228 & 524287u) - 262144u))) * 0.0009765625f);
                let _e235 = clamp(_e233, 0f, 1f);
                phi_1238_ = (_e222 + ((1f - (_e222 * _e107.w)) * ((clamp((_e233 + _e226), 0f, 1f) - _e235) / max((1f - (_e235 * _e107.w)), 0.000062f))));
            }
            let _e247 = phi_1238_;
            phi_1240_ = (_e107.w * _e247);
            break;
        }
    }
    let _e250 = phi_1240_;
    phi_1251_ = f32();
    if mh {
        let _e251 = gl_FragCoord_1;
        let _e253 = m.B3_;
        let _e255 = m.C3_;
        if mh {
            phi_1248_ = ((fract((52.982918f * fract(((0.06711056f * _e251.x) + (0.00583715f * _e251.y))))) * _e253) + _e255);
        } else {
            phi_1248_ = 0f;
        }
        let _e267 = phi_1248_;
        phi_1251_ = _e267;
    }
    let _e269 = phi_1251_;
    let _e271 = (_e107.xyz * _e250);
    let _e275 = vec4<f32>(_e271.x, _e271.y, _e271.z, _e250);
    let _e276 = _e275.xyz;
    if (mh && (_e250 != 0f)) {
        phi_1269_ = (vec3(_e269) + _e276);
    } else {
        phi_1269_ = _e276;
    }
    let _e282 = phi_1269_;
    let _e288 = vec4<f32>(_e282.x, _e275.y, _e275.z, _e275.w);
    let _e294 = vec4<f32>(_e288.x, _e282.y, _e288.z, _e288.w);
    C1_ = vec4<f32>(_e294.x, _e294.y, _e282.z, _e294.w);
    return;
}

@fragment
fn main(@location(0) f1_: vec4<f32>, @location(9) A2_: vec3<f32>, @location(1) @interpolate(flat, either) i1_: f32, @location(8) n4_: vec2<f32>, @location(7) @interpolate(flat, either) f3_: vec2<u32>, @location(5) M0_: vec4<f32>, @location(4) @interpolate(flat, either) V1_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) B0_: f32, @location(6) @interpolate(flat, either) f2_: f32) -> @location(0) vec4<f32> {
    f1_1 = f1_;
    A2_1 = A2_;
    i1_1 = i1_;
    n4_1 = n4_;
    f3_1 = f3_;
    M0_1 = M0_;
    V1_1 = V1_;
    gl_FragCoord_1 = gl_FragCoord;
    B0_1 = B0_;
    f2_1 = f2_;
    main_1();
    let _e21 = C1_;
    return _e21;
}
