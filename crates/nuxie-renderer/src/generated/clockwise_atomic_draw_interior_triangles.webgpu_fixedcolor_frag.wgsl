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

struct h0Ed {
    d2_: array<u32>,
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
var<private> gl_FragCoord_1: vec4<f32>;
var<private> f1_1: vec4<f32>;
var<private> A2_1: vec3<f32>;
var<private> i1_1: f32;
var<private> n4_1: vec2<f32>;
var<private> f3_1: vec2<u32>;
var<private> M0_1: vec4<f32>;
var<private> V1_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Ed;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> B0_1: f32;
var<private> f2_1: f32;

fn main_1() {
    var phi_1299_: f32;
    var phi_1300_: f32;
    var phi_1311_: vec4<f32>;
    var phi_881_: bool;
    var phi_1301_: f32;
    var phi_1312_: vec4<f32>;
    var phi_1314_: f32;
    var phi_655_: bool;
    var phi_1315_: f32;
    var phi_1040_: bool;
    var phi_1042_: bool;
    var phi_1338_: f32;
    var phi_1333_: u32;
    var phi_1330_: f32;
    var phi_1337_: f32;
    var phi_1332_: u32;
    var phi_1329_: f32;
    var phi_1334_: f32;
    var phi_1331_: u32;
    var phi_1328_: f32;
    var phi_1342_: f32;
    var phi_1344_: f32;
    var phi_1352_: f32;
    var phi_1355_: f32;
    var phi_1373_: vec3<f32>;

    let _e53 = gl_FragCoord_1;
    let _e57 = bitcast<vec2<u32>>(vec2<i32>(floor(_e53.xy)));
    let _e59 = m.p6_;
    let _e88 = bitcast<i32>((((((_e57.y >> bitcast<u32>(5u)) * (((_e59 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e57.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e57.x & 28u) << bitcast<u32>(5u)) + ((_e57.y & 28u) << bitcast<u32>(2i)))) + (((_e57.y & 3u) << bitcast<u32>(2i)) + (_e57.x & 3u))));
    let _e89 = f1_1;
    let _e90 = A2_1;
    if (_e89.w >= 0f) {
        phi_1311_ = vec4<f32>(_e89.x, _e89.y, _e89.z, _e89.w);
    } else {
        if (_e89.z > 0f) {
            phi_1299_ = _e89.x;
        } else {
            phi_1299_ = length(_e89.xy);
        }
        let _e99 = phi_1299_;
        let _e100 = clamp(_e99, 0f, 1f);
        let _e101 = abs(_e89.z);
        if (_e101 > 1f) {
            phi_1300_ = ((0.9980469f * _e100) + 0.0009765625f);
        } else {
            phi_1300_ = ((0.001953125f * _e100) + _e101);
        }
        let _e108 = phi_1300_;
        let _e111 = textureSampleLevel(MD, Pb, vec2<f32>(_e108, -(_e89.w)), 0f);
        phi_1311_ = vec4<f32>(_e111.x, _e111.y, _e111.z, _e111.w);
    }
    let _e125 = phi_1311_;
    phi_881_ = nh;
    if nh {
        phi_881_ = (_e90.z > 0f);
    }
    let _e129 = phi_881_;
    phi_1312_ = _e125;
    if _e129 {
        let _e133 = textureSampleLevel(JC, V5_, _e90.xy, (_e90.z - 1f));
        if (_e133.w != 0f) {
            phi_1301_ = (1f / _e133.w);
        } else {
            phi_1301_ = 0f;
        }
        let _e139 = phi_1301_;
        let _e140 = (_e133.xyz * _e139);
        phi_1312_ = (_e125 * vec4<f32>(_e140.x, _e140.y, _e140.z, _e133.w));
    }
    let _e147 = phi_1312_;
    let _e148 = i1_1;
    let _e149 = n4_1;
    let _e152 = f3_1[1u];
    let _e154 = f3_1[0u];
    let _e155 = vec2<u32>(floor(_e149));
    phi_1314_ = 1f;
    if gh {
        let _e183 = M0_1;
        let _e186 = min(_e183.xy, _e183.zw);
        phi_1314_ = min(min(_e186.x, _e186.y), 1f);
    }
    let _e192 = phi_1314_;
    phi_655_ = fh;
    if fh {
        let _e194 = V1_1[0u];
        phi_655_ = (_e194 != 0f);
    }
    let _e197 = phi_655_;
    phi_1315_ = _e192;
    if _e197 {
        let _e200 = h0_.d2_[_e88];
        phi_1315_ = min(unpack4x8unorm(_e200).x, _e192);
    }
    let _e205 = phi_1315_;
    let _e207 = clamp(_e148, 0f, max(_e205, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e213 = u32(((abs(_e207) * 1024f) + 0.5f));
            let _e216 = atomicLoad((&P0_.d2_[(_e154 + (((((_e155.y >> bitcast<u32>(5u)) * (_e152 << bitcast<u32>(5u))) + ((_e155.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e155.x & 28u) << bitcast<u32>(5u)) + ((_e155.y & 28u) << bitcast<u32>(2i)))) + (((_e155.y & 3u) << bitcast<u32>(2i)) + (_e155.x & 3u))))]));
            let _e218 = (min(_e147.w, _e207) >= 1f);
            phi_1042_ = _e218;
            if _e218 {
                let _e220 = m.c2_;
                let _e221 = (_e216 < _e220);
                phi_1040_ = _e221;
                if !(_e221) {
                    phi_1040_ = (_e216 >= (_e220 | 262144u));
                }
                let _e226 = phi_1040_;
                phi_1042_ = _e226;
            }
            let _e228 = phi_1042_;
            if _e228 {
                phi_1344_ = _e147.w;
                break;
            }
            let _e230 = m.c2_;
            phi_1334_ = 0f;
            phi_1331_ = _e213;
            phi_1328_ = _e207;
            if (_e216 < _e230) {
                let _e233 = (_e230 | (262144u + _e213));
                let _e234 = atomicMax((&P0_.d2_[(_e154 + (((((_e155.y >> bitcast<u32>(5u)) * (_e152 << bitcast<u32>(5u))) + ((_e155.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e155.x & 28u) << bitcast<u32>(5u)) + ((_e155.y & 28u) << bitcast<u32>(2i)))) + (((_e155.y & 3u) << bitcast<u32>(2i)) + (_e155.x & 3u))))]), _e233);
                if (_e234 <= _e230) {
                    phi_1337_ = min(_e207, 1f);
                    phi_1332_ = _e213;
                    phi_1329_ = 0f;
                } else {
                    phi_1338_ = 0f;
                    phi_1333_ = _e213;
                    phi_1330_ = _e207;
                    if (_e234 < _e233) {
                        let _e238 = ((_e234 & 524287u) - 262144u);
                        let _e240 = (f32(_e238) * 0.0009765625f);
                        phi_1338_ = ((min(_e207, 1f) - _e240) / max((1f - (_e240 * _e147.w)), 0.000062f));
                        phi_1333_ = _e238;
                        phi_1330_ = _e240;
                    }
                    let _e248 = phi_1338_;
                    let _e250 = phi_1333_;
                    let _e252 = phi_1330_;
                    phi_1337_ = _e248;
                    phi_1332_ = _e250;
                    phi_1329_ = _e252;
                }
                let _e255 = phi_1337_;
                let _e257 = phi_1332_;
                let _e259 = phi_1329_;
                phi_1334_ = _e255;
                phi_1331_ = _e257;
                phi_1328_ = _e259;
            }
            let _e261 = phi_1334_;
            let _e263 = phi_1331_;
            let _e265 = phi_1328_;
            phi_1342_ = _e261;
            if (_e265 > 0f) {
                let _e267 = atomicAdd((&P0_.d2_[(_e154 + (((((_e155.y >> bitcast<u32>(5u)) * (_e152 << bitcast<u32>(5u))) + ((_e155.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e155.x & 28u) << bitcast<u32>(5u)) + ((_e155.y & 28u) << bitcast<u32>(2i)))) + (((_e155.y & 3u) << bitcast<u32>(2i)) + (_e155.x & 3u))))]), _e263);
                let _e272 = (f32(bitcast<i32>(((_e267 & 524287u) - 262144u))) * 0.0009765625f);
                let _e274 = clamp(_e272, 0f, 1f);
                phi_1342_ = (_e261 + ((1f - (_e261 * _e147.w)) * ((clamp((_e272 + _e265), 0f, 1f) - _e274) / max((1f - (_e274 * _e147.w)), 0.000062f))));
            }
            let _e286 = phi_1342_;
            phi_1344_ = (_e147.w * _e286);
            break;
        }
    }
    let _e289 = phi_1344_;
    phi_1355_ = f32();
    if mh {
        let _e291 = m.B3_;
        let _e293 = m.C3_;
        if mh {
            phi_1352_ = ((fract((52.982918f * fract(((0.06711056f * _e53.x) + (0.00583715f * _e53.y))))) * _e291) + _e293);
        } else {
            phi_1352_ = 0f;
        }
        let _e305 = phi_1352_;
        phi_1355_ = _e305;
    }
    let _e307 = phi_1355_;
    let _e309 = (_e147.xyz * _e289);
    let _e313 = vec4<f32>(_e309.x, _e309.y, _e309.z, _e289);
    let _e314 = _e313.xyz;
    if (mh && (_e289 != 0f)) {
        phi_1373_ = (vec3(_e307) + _e314);
    } else {
        phi_1373_ = _e314;
    }
    let _e320 = phi_1373_;
    let _e326 = vec4<f32>(_e320.x, _e313.y, _e313.z, _e313.w);
    let _e332 = vec4<f32>(_e326.x, _e320.y, _e326.z, _e326.w);
    h0_.d2_[_e88] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    C1_ = vec4<f32>(_e332.x, _e332.y, _e320.z, _e332.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) f1_: vec4<f32>, @location(9) A2_: vec3<f32>, @location(1) @interpolate(flat, either) i1_: f32, @location(8) n4_: vec2<f32>, @location(7) @interpolate(flat, either) f3_: vec2<u32>, @location(5) M0_: vec4<f32>, @location(4) @interpolate(flat, either) V1_: vec2<f32>, @location(3) @interpolate(flat, either) B0_: f32, @location(6) @interpolate(flat, either) f2_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    f1_1 = f1_;
    A2_1 = A2_;
    i1_1 = i1_;
    n4_1 = n4_;
    f3_1 = f3_;
    M0_1 = M0_;
    V1_1 = V1_;
    B0_1 = B0_;
    f2_1 = f2_;
    main_1();
    let _e21 = C1_;
    return _e21;
}
