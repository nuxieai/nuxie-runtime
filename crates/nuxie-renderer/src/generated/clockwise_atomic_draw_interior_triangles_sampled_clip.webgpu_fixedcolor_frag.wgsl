struct ye {
    e2_: array<u32>,
}

struct BC {
    qc: f32,
    Ad: f32,
    Ef: f32,
    Ff: f32,
    q6_: u32,
    Nb: u32,
    qf: u32,
    rf: u32,
    V7_: vec4<i32>,
    bh: vec2<f32>,
    Bd: vec2<f32>,
    d2_: u32,
    fh: f32,
    e6_: u32,
    T2_: f32,
    Cd: f32,
    lf: u32,
    B3_: f32,
    C3_: f32,
    Dd: f32,
    Yg: u32,
}

struct ye_1 {
    e2_: array<atomic<u32>>,
}

@id(7) override Ih: bool = true;
@id(8) override Jh: bool = true;
@id(1) override Ch: bool = true;
@id(0) override Bh: bool = true;

@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var W5_: sampler;
@group(0) @binding(6)
var<storage, read_write> Q0_: ye_1;
@group(0) @binding(0)
var<uniform> n: BC;
var<private> V1_1: vec4<f32>;
var<private> C2_1: vec3<f32>;
var<private> h1_1: f32;
var<private> p4_1: vec2<f32>;
var<private> g3_1: vec2<u32>;
var<private> M0_1: vec4<f32>;
var<private> W1_1: vec2<f32>;
@group(2) @binding(1)
var h0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var ea: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> C0_1: f32;
var<private> g2_1: f32;

fn main_1() {
    var phi_1222_: f32;
    var phi_1223_: f32;
    var phi_1234_: vec4<f32>;
    var phi_784_: bool;
    var phi_1224_: f32;
    var phi_1235_: vec4<f32>;
    var phi_1237_: f32;
    var phi_631_: bool;
    var phi_1238_: f32;
    var phi_986_: bool;
    var phi_988_: bool;
    var phi_1261_: f32;
    var phi_1256_: u32;
    var phi_1253_: f32;
    var phi_1260_: f32;
    var phi_1255_: u32;
    var phi_1252_: f32;
    var phi_1257_: f32;
    var phi_1254_: u32;
    var phi_1251_: f32;
    var phi_1265_: f32;
    var phi_1267_: f32;
    var phi_1275_: f32;
    var phi_1278_: f32;
    var phi_1296_: vec3<f32>;

    let _e49 = V1_1;
    let _e50 = C2_1;
    if (_e49.w >= 0f) {
        phi_1234_ = vec4<f32>(_e49.x, _e49.y, _e49.z, _e49.w);
    } else {
        if (_e49.z > 0f) {
            phi_1222_ = _e49.x;
        } else {
            phi_1222_ = length(_e49.xy);
        }
        let _e60 = phi_1222_;
        let _e61 = clamp(_e60, 0f, 1f);
        let _e62 = abs(_e49.z);
        if (_e62 > 1f) {
            phi_1223_ = ((0.9980469f * _e61) + 0.0009765625f);
        } else {
            phi_1223_ = ((0.001953125f * _e61) + _e62);
        }
        let _e69 = phi_1223_;
        let _e71 = textureSampleLevel(ED, N9_, vec2<f32>(_e69, -(_e49.w)), 0f);
        phi_1234_ = vec4<f32>(_e71.x, _e71.y, _e71.z, _e71.w);
    }
    let _e85 = phi_1234_;
    phi_784_ = Jh;
    if Jh {
        phi_784_ = (_e50.z > 0f);
    }
    let _e89 = phi_784_;
    phi_1235_ = _e85;
    if _e89 {
        let _e93 = textureSampleLevel(HC, W5_, _e50.xy, (_e50.z - 1f));
        if (_e93.w != 0f) {
            phi_1224_ = (1f / _e93.w);
        } else {
            phi_1224_ = 0f;
        }
        let _e99 = phi_1224_;
        let _e100 = (_e93.xyz * _e99);
        phi_1235_ = (_e85 * vec4<f32>(_e100.x, _e100.y, _e100.z, _e93.w));
    }
    let _e107 = phi_1235_;
    let _e108 = h1_1;
    let _e109 = p4_1;
    let _e112 = g3_1[1u];
    let _e114 = g3_1[0u];
    let _e115 = vec2<u32>(floor(_e109));
    phi_1237_ = 1f;
    if Ch {
        let _e143 = M0_1;
        let _e146 = min(_e143.xy, _e143.zw);
        phi_1237_ = min(min(_e146.x, _e146.y), 1f);
    }
    let _e152 = phi_1237_;
    phi_631_ = Bh;
    if Bh {
        let _e154 = W1_1[0u];
        phi_631_ = (_e154 != 0f);
    }
    let _e157 = phi_631_;
    phi_1238_ = _e152;
    if _e157 {
        let _e158 = gl_FragCoord_1;
        let _e162 = textureLoad(h0_, vec2<i32>(floor(_e158.xy)), 0i);
        phi_1238_ = min(_e162.x, _e152);
    }
    let _e166 = phi_1238_;
    let _e168 = clamp(_e108, 0f, max(_e166, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e174 = u32(((abs(_e168) * 1024f) + 0.5f));
            let _e177 = atomicLoad((&Q0_.e2_[(_e114 + (((((_e115.y >> bitcast<u32>(5u)) * (_e112 << bitcast<u32>(5u))) + ((_e115.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e115.x & 28u) << bitcast<u32>(5u)) + ((_e115.y & 28u) << bitcast<u32>(2i)))) + (((_e115.y & 3u) << bitcast<u32>(2i)) + (_e115.x & 3u))))]));
            let _e179 = (min(_e107.w, _e168) >= 1f);
            phi_988_ = _e179;
            if _e179 {
                let _e181 = n.d2_;
                let _e182 = (_e177 < _e181);
                phi_986_ = _e182;
                if !(_e182) {
                    phi_986_ = (_e177 >= (_e181 | 262144u));
                }
                let _e187 = phi_986_;
                phi_988_ = _e187;
            }
            let _e189 = phi_988_;
            if _e189 {
                phi_1267_ = _e107.w;
                break;
            }
            let _e191 = n.d2_;
            phi_1257_ = 0f;
            phi_1254_ = _e174;
            phi_1251_ = _e168;
            if (_e177 < _e191) {
                let _e194 = (_e191 | (262144u + _e174));
                let _e195 = atomicMax((&Q0_.e2_[(_e114 + (((((_e115.y >> bitcast<u32>(5u)) * (_e112 << bitcast<u32>(5u))) + ((_e115.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e115.x & 28u) << bitcast<u32>(5u)) + ((_e115.y & 28u) << bitcast<u32>(2i)))) + (((_e115.y & 3u) << bitcast<u32>(2i)) + (_e115.x & 3u))))]), _e194);
                if (_e195 <= _e191) {
                    phi_1260_ = min(_e168, 1f);
                    phi_1255_ = _e174;
                    phi_1252_ = 0f;
                } else {
                    phi_1261_ = 0f;
                    phi_1256_ = _e174;
                    phi_1253_ = _e168;
                    if (_e195 < _e194) {
                        let _e199 = ((_e195 & 524287u) - 262144u);
                        let _e201 = (f32(_e199) * 0.0009765625f);
                        phi_1261_ = ((min(_e168, 1f) - _e201) / max((1f - (_e201 * _e107.w)), 0.000062f));
                        phi_1256_ = _e199;
                        phi_1253_ = _e201;
                    }
                    let _e209 = phi_1261_;
                    let _e211 = phi_1256_;
                    let _e213 = phi_1253_;
                    phi_1260_ = _e209;
                    phi_1255_ = _e211;
                    phi_1252_ = _e213;
                }
                let _e216 = phi_1260_;
                let _e218 = phi_1255_;
                let _e220 = phi_1252_;
                phi_1257_ = _e216;
                phi_1254_ = _e218;
                phi_1251_ = _e220;
            }
            let _e222 = phi_1257_;
            let _e224 = phi_1254_;
            let _e226 = phi_1251_;
            phi_1265_ = _e222;
            if (_e226 > 0f) {
                let _e228 = atomicAdd((&Q0_.e2_[(_e114 + (((((_e115.y >> bitcast<u32>(5u)) * (_e112 << bitcast<u32>(5u))) + ((_e115.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e115.x & 28u) << bitcast<u32>(5u)) + ((_e115.y & 28u) << bitcast<u32>(2i)))) + (((_e115.y & 3u) << bitcast<u32>(2i)) + (_e115.x & 3u))))]), _e224);
                let _e233 = (f32(bitcast<i32>(((_e228 & 524287u) - 262144u))) * 0.0009765625f);
                let _e235 = clamp(_e233, 0f, 1f);
                phi_1265_ = (_e222 + ((1f - (_e222 * _e107.w)) * ((clamp((_e233 + _e226), 0f, 1f) - _e235) / max((1f - (_e235 * _e107.w)), 0.000062f))));
            }
            let _e247 = phi_1265_;
            phi_1267_ = (_e107.w * _e247);
            break;
        }
    }
    let _e250 = phi_1267_;
    phi_1278_ = f32();
    if Ih {
        let _e251 = gl_FragCoord_1;
        let _e253 = n.B3_;
        let _e255 = n.C3_;
        if Ih {
            phi_1275_ = ((fract((52.982918f * fract(((0.06711056f * _e251.x) + (0.00583715f * _e251.y))))) * _e253) + _e255);
        } else {
            phi_1275_ = 0f;
        }
        let _e267 = phi_1275_;
        phi_1278_ = _e267;
    }
    let _e269 = phi_1278_;
    let _e271 = (_e107.xyz * _e250);
    let _e275 = vec4<f32>(_e271.x, _e271.y, _e271.z, _e250);
    let _e276 = _e275.xyz;
    if (Ih && (_e250 != 0f)) {
        phi_1296_ = (vec3(_e269) + _e276);
    } else {
        phi_1296_ = _e276;
    }
    let _e282 = phi_1296_;
    let _e288 = vec4<f32>(_e282.x, _e275.y, _e275.z, _e275.w);
    let _e294 = vec4<f32>(_e288.x, _e282.y, _e288.z, _e288.w);
    C1_ = vec4<f32>(_e294.x, _e294.y, _e282.z, _e294.w);
    return;
}

@fragment
fn main(@location(0) V1_: vec4<f32>, @location(9) C2_: vec3<f32>, @location(1) @interpolate(flat, either) h1_: f32, @location(8) p4_: vec2<f32>, @location(7) @interpolate(flat, either) g3_: vec2<u32>, @location(5) M0_: vec4<f32>, @location(4) @interpolate(flat, either) W1_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) C0_: f32, @location(6) @interpolate(flat, either) g2_: f32) -> @location(0) vec4<f32> {
    V1_1 = V1_;
    C2_1 = C2_;
    h1_1 = h1_;
    p4_1 = p4_;
    g3_1 = g3_;
    M0_1 = M0_;
    W1_1 = W1_;
    gl_FragCoord_1 = gl_FragCoord;
    C0_1 = C0_;
    g2_1 = g2_;
    main_1();
    let _e21 = C1_;
    return _e21;
}
