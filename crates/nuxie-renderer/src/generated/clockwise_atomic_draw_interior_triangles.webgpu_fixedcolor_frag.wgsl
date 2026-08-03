struct ee {
    c2_: array<u32>,
}

struct CC {
    cc: f32,
    md: f32,
    df: f32,
    ef: f32,
    m6_: u32,
    Dg: u32,
    Pe: u32,
    Qe: u32,
    R7_: vec4<i32>,
    zg: vec2<f32>,
    nd: vec2<f32>,
    a2_: u32,
    Eg: f32,
    Z5_: u32,
    P2_: f32,
    od: f32,
    Ke: u32,
    z3_: f32,
    A3_: f32,
    pd: f32,
    wg: u32,
}

struct h0zd {
    c2_: array<u32>,
}

struct ee_1 {
    c2_: array<atomic<u32>>,
}

@id(7) override dh: bool = true;
@id(1) override Xg: bool = true;
@id(0) override Wg: bool = true;

@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(3) @binding(8)
var Kb: sampler;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;
@group(0) @binding(6)
var<storage, read_write> P0_: ee_1;
@group(0) @binding(0)
var<uniform> n: CC;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> f1_1: vec4<f32>;
var<private> i1_1: f32;
var<private> l4_1: vec2<f32>;
var<private> d3_1: vec2<u32>;
var<private> L0_1: vec4<f32>;
var<private> U1_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> h0_: h0zd;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> B0_1: f32;
var<private> e2_1: f32;

fn main_1() {
    var phi_1303_: f32;
    var phi_1307_: f32;
    var phi_1308_: f32;
    var phi_1310_: vec4<f32>;
    var phi_1309_: vec4<f32>;
    var phi_1312_: f32;
    var phi_657_: bool;
    var phi_1313_: f32;
    var phi_1043_: bool;
    var phi_1045_: bool;
    var phi_1336_: f32;
    var phi_1331_: u32;
    var phi_1328_: f32;
    var phi_1335_: f32;
    var phi_1330_: u32;
    var phi_1327_: f32;
    var phi_1332_: f32;
    var phi_1329_: u32;
    var phi_1326_: f32;
    var phi_1340_: f32;
    var phi_1342_: f32;
    var phi_1350_: f32;
    var phi_1353_: f32;
    var phi_1370_: vec3<f32>;

    let _e53 = gl_FragCoord_1;
    let _e57 = bitcast<vec2<u32>>(vec2<i32>(floor(_e53.xy)));
    let _e59 = n.m6_;
    let _e88 = bitcast<i32>((((((_e57.y >> bitcast<u32>(5u)) * (((_e59 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e57.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e57.x & 28u) << bitcast<u32>(5u)) + ((_e57.y & 28u) << bitcast<u32>(2i)))) + (((_e57.y & 3u) << bitcast<u32>(2i)) + (_e57.x & 3u))));
    let _e89 = f1_1;
    if (_e89.w >= 0f) {
        phi_1309_ = vec4<f32>(_e89.x, _e89.y, _e89.z, _e89.w);
    } else {
        if (_e89.w > -1f) {
            if (_e89.z > 0f) {
                phi_1307_ = _e89.x;
            } else {
                phi_1307_ = length(_e89.xy);
            }
            let _e115 = phi_1307_;
            let _e116 = clamp(_e115, 0f, 1f);
            let _e117 = abs(_e89.z);
            if (_e117 > 1f) {
                phi_1308_ = ((0.9980469f * _e116) + 0.0009765625f);
            } else {
                phi_1308_ = ((0.001953125f * _e116) + _e117);
            }
            let _e124 = phi_1308_;
            let _e127 = textureSampleLevel(KD, Kb, vec2<f32>(_e124, -(_e89.w)), 0f);
            phi_1310_ = vec4<f32>(_e127.x, _e127.y, _e127.z, _e127.w);
        } else {
            let _e95 = textureSampleLevel(IC, S5_, _e89.xy, (-2f - _e89.w));
            if (_e95.w != 0f) {
                phi_1303_ = (1f / _e95.w);
            } else {
                phi_1303_ = 0f;
            }
            let _e102 = phi_1303_;
            let _e103 = (_e95.xyz * _e102);
            phi_1310_ = vec4<f32>(_e103.x, _e103.y, _e103.z, (_e95.w * _e89.z));
        }
        let _e135 = phi_1310_;
        phi_1309_ = _e135;
    }
    let _e143 = phi_1309_;
    let _e144 = i1_1;
    let _e145 = l4_1;
    let _e148 = d3_1[1u];
    let _e150 = d3_1[0u];
    let _e151 = vec2<u32>(floor(_e145));
    phi_1312_ = 1f;
    if Xg {
        let _e179 = L0_1;
        let _e182 = min(_e179.xy, _e179.zw);
        phi_1312_ = min(min(_e182.x, _e182.y), 1f);
    }
    let _e188 = phi_1312_;
    phi_657_ = Wg;
    if Wg {
        let _e190 = U1_1[0u];
        phi_657_ = (_e190 != 0f);
    }
    let _e193 = phi_657_;
    phi_1313_ = _e188;
    if _e193 {
        let _e196 = h0_.c2_[_e88];
        phi_1313_ = min(unpack4x8unorm(_e196).x, _e188);
    }
    let _e201 = phi_1313_;
    let _e203 = clamp(_e144, 0f, max(_e201, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e209 = u32(((abs(_e203) * 1024f) + 0.5f));
            let _e212 = atomicLoad((&P0_.c2_[(_e150 + (((((_e151.y >> bitcast<u32>(5u)) * (_e148 << bitcast<u32>(5u))) + ((_e151.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e151.x & 28u) << bitcast<u32>(5u)) + ((_e151.y & 28u) << bitcast<u32>(2i)))) + (((_e151.y & 3u) << bitcast<u32>(2i)) + (_e151.x & 3u))))]));
            let _e214 = (min(_e143.w, _e203) >= 1f);
            phi_1045_ = _e214;
            if _e214 {
                let _e216 = n.a2_;
                let _e217 = (_e212 < _e216);
                phi_1043_ = _e217;
                if !(_e217) {
                    phi_1043_ = (_e212 >= (_e216 | 262144u));
                }
                let _e222 = phi_1043_;
                phi_1045_ = _e222;
            }
            let _e224 = phi_1045_;
            if _e224 {
                phi_1342_ = _e143.w;
                break;
            }
            let _e226 = n.a2_;
            phi_1332_ = 0f;
            phi_1329_ = _e209;
            phi_1326_ = _e203;
            if (_e212 < _e226) {
                let _e229 = (_e226 | (262144u + _e209));
                let _e230 = atomicMax((&P0_.c2_[(_e150 + (((((_e151.y >> bitcast<u32>(5u)) * (_e148 << bitcast<u32>(5u))) + ((_e151.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e151.x & 28u) << bitcast<u32>(5u)) + ((_e151.y & 28u) << bitcast<u32>(2i)))) + (((_e151.y & 3u) << bitcast<u32>(2i)) + (_e151.x & 3u))))]), _e229);
                if (_e230 <= _e226) {
                    phi_1335_ = min(_e203, 1f);
                    phi_1330_ = _e209;
                    phi_1327_ = 0f;
                } else {
                    phi_1336_ = 0f;
                    phi_1331_ = _e209;
                    phi_1328_ = _e203;
                    if (_e230 < _e229) {
                        let _e234 = ((_e230 & 524287u) - 262144u);
                        let _e236 = (f32(_e234) * 0.0009765625f);
                        phi_1336_ = ((min(_e203, 1f) - _e236) / max((1f - (_e236 * _e143.w)), 0.000062f));
                        phi_1331_ = _e234;
                        phi_1328_ = _e236;
                    }
                    let _e244 = phi_1336_;
                    let _e246 = phi_1331_;
                    let _e248 = phi_1328_;
                    phi_1335_ = _e244;
                    phi_1330_ = _e246;
                    phi_1327_ = _e248;
                }
                let _e251 = phi_1335_;
                let _e253 = phi_1330_;
                let _e255 = phi_1327_;
                phi_1332_ = _e251;
                phi_1329_ = _e253;
                phi_1326_ = _e255;
            }
            let _e257 = phi_1332_;
            let _e259 = phi_1329_;
            let _e261 = phi_1326_;
            phi_1340_ = _e257;
            if (_e261 > 0f) {
                let _e263 = atomicAdd((&P0_.c2_[(_e150 + (((((_e151.y >> bitcast<u32>(5u)) * (_e148 << bitcast<u32>(5u))) + ((_e151.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e151.x & 28u) << bitcast<u32>(5u)) + ((_e151.y & 28u) << bitcast<u32>(2i)))) + (((_e151.y & 3u) << bitcast<u32>(2i)) + (_e151.x & 3u))))]), _e259);
                let _e268 = (f32(bitcast<i32>(((_e263 & 524287u) - 262144u))) * 0.0009765625f);
                let _e270 = clamp(_e268, 0f, 1f);
                phi_1340_ = (_e257 + ((1f - (_e257 * _e143.w)) * ((clamp((_e268 + _e261), 0f, 1f) - _e270) / max((1f - (_e270 * _e143.w)), 0.000062f))));
            }
            let _e282 = phi_1340_;
            phi_1342_ = (_e143.w * _e282);
            break;
        }
    }
    let _e285 = phi_1342_;
    phi_1353_ = f32();
    if dh {
        let _e287 = n.z3_;
        let _e289 = n.A3_;
        if dh {
            phi_1350_ = ((fract((52.982918f * fract(((0.06711056f * _e53.x) + (0.00583715f * _e53.y))))) * _e287) + _e289);
        } else {
            phi_1350_ = 0f;
        }
        let _e301 = phi_1350_;
        phi_1353_ = _e301;
    }
    let _e303 = phi_1353_;
    let _e305 = (_e143.xyz * _e285);
    let _e309 = vec4<f32>(_e305.x, _e305.y, _e305.z, _e285);
    let _e310 = _e309.xyz;
    if (dh && (_e285 != 0f)) {
        phi_1370_ = (vec3(_e303) + _e310);
    } else {
        phi_1370_ = _e310;
    }
    let _e316 = phi_1370_;
    let _e322 = vec4<f32>(_e316.x, _e309.y, _e309.z, _e309.w);
    let _e328 = vec4<f32>(_e322.x, _e316.y, _e322.z, _e322.w);
    h0_.c2_[_e88] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    C1_ = vec4<f32>(_e328.x, _e328.y, _e316.z, _e328.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) f1_: vec4<f32>, @location(1) @interpolate(flat, either) i1_: f32, @location(8) l4_: vec2<f32>, @location(7) @interpolate(flat, either) d3_: vec2<u32>, @location(5) L0_: vec4<f32>, @location(4) @interpolate(flat, either) U1_: vec2<f32>, @location(3) @interpolate(flat, either) B0_: f32, @location(6) @interpolate(flat, either) e2_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    f1_1 = f1_;
    i1_1 = i1_;
    l4_1 = l4_;
    d3_1 = d3_;
    L0_1 = L0_;
    U1_1 = U1_;
    B0_1 = B0_;
    e2_1 = e2_;
    main_1();
    let _e19 = C1_;
    return _e19;
}
