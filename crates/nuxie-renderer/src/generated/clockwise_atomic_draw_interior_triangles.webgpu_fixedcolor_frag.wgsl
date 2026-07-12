struct Yd {
    X1_: array<u32>,
}

struct NB {
    Ub: f32,
    dd: f32,
    Xe: f32,
    Ye: f32,
    q5_: u32,
    ug: u32,
    Je: u32,
    Ke: u32,
    U7_: vec4<i32>,
    rg: vec2<f32>,
    ed: vec2<f32>,
    W1_: u32,
    vg: f32,
    Y5_: u32,
    P2_: f32,
    fd: f32,
    Ee: u32,
    y3_: f32,
    z3_: f32,
    gd: f32,
    og: u32,
}

struct d0qd {
    X1_: array<u32>,
}

struct Yd_1 {
    X1_: array<atomic<u32>>,
}

@id(7) override Tg: bool = true;
@id(1) override Ng: bool = true;
@id(0) override Mg: bool = true;

@group(0) @binding(9) 
var DD: texture_2d<f32>;
@group(3) @binding(9) 
var Bb: sampler;
@group(1) @binding(12) 
var AC: texture_2d<f32>;
@group(1) @binding(14) 
var R5_: sampler;
@group(0) @binding(7) 
var<storage, read_write> S0_: Yd_1;
@group(0) @binding(0) 
var<uniform> k: NB;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> i1_1: vec4<f32>;
var<private> j1_1: f32;
var<private> j4_1: vec2<f32>;
var<private> e3_1: vec2<u32>;
var<private> N0_1: vec4<f32>;
var<private> S1_1: vec2<f32>;
@group(2) @binding(1) 
var<storage, read_write> d0_: d0qd;
var<private> l1_: vec4<f32>;
@group(3) @binding(10) 
var T9_: sampler;
@group(0) @binding(10) 
var QC: texture_2d<f32>;
var<private> z0_1: f32;
var<private> Z1_1: f32;

fn main_1() {
    var phi_1264_: f32;
    var phi_1268_: f32;
    var phi_1269_: f32;
    var phi_1271_: vec4<f32>;
    var phi_1270_: vec4<f32>;
    var phi_1273_: f32;
    var phi_636_: bool;
    var phi_1274_: f32;
    var phi_1020_: bool;
    var phi_1022_: bool;
    var phi_1297_: f32;
    var phi_1292_: u32;
    var phi_1289_: f32;
    var phi_1296_: f32;
    var phi_1291_: u32;
    var phi_1288_: f32;
    var phi_1293_: f32;
    var phi_1290_: u32;
    var phi_1287_: f32;
    var phi_1301_: f32;
    var phi_1303_: f32;
    var phi_1311_: f32;
    var phi_1314_: f32;
    var phi_1349_: vec4<f32>;

    let _e53 = gl_FragCoord_1;
    let _e57 = bitcast<vec2<u32>>(vec2<i32>(floor(_e53.xy)));
    let _e59 = k.q5_;
    let _e88 = bitcast<i32>((((((_e57.y >> bitcast<u32>(5u)) * (((_e59 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e57.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e57.x & 28u) << bitcast<u32>(5u)) + ((_e57.y & 28u) << bitcast<u32>(2i)))) + (((_e57.y & 3u) << bitcast<u32>(2i)) + (_e57.x & 3u))));
    let _e89 = i1_1;
    if (_e89.w >= 0f) {
        phi_1270_ = vec4<f32>(_e89.x, _e89.y, _e89.z, _e89.w);
    } else {
        if (_e89.w > -1f) {
            if (_e89.z > 0f) {
                phi_1268_ = _e89.x;
            } else {
                phi_1268_ = length(_e89.xy);
            }
            let _e115 = phi_1268_;
            let _e116 = clamp(_e115, 0f, 1f);
            let _e117 = abs(_e89.z);
            if (_e117 > 1f) {
                phi_1269_ = ((0.9980469f * _e116) + 0.0009765625f);
            } else {
                phi_1269_ = ((0.001953125f * _e116) + _e117);
            }
            let _e124 = phi_1269_;
            let _e127 = textureSampleLevel(DD, Bb, vec2<f32>(_e124, -(_e89.w)), 0f);
            phi_1271_ = vec4<f32>(_e127.x, _e127.y, _e127.z, _e127.w);
        } else {
            let _e95 = textureSampleLevel(AC, R5_, _e89.xy, (-2f - _e89.w));
            if (_e95.w != 0f) {
                phi_1264_ = (1f / _e95.w);
            } else {
                phi_1264_ = 0f;
            }
            let _e102 = phi_1264_;
            let _e103 = (_e95.xyz * _e102);
            phi_1271_ = vec4<f32>(_e103.x, _e103.y, _e103.z, (_e95.w * _e89.z));
        }
        let _e135 = phi_1271_;
        phi_1270_ = _e135;
    }
    let _e143 = phi_1270_;
    let _e144 = j1_1;
    let _e145 = j4_1;
    let _e148 = e3_1[1u];
    let _e150 = e3_1[0u];
    let _e151 = vec2<u32>(floor(_e145));
    phi_1273_ = 1f;
    if Ng {
        let _e179 = N0_1;
        let _e182 = min(_e179.xy, _e179.zw);
        phi_1273_ = min(min(_e182.x, _e182.y), 1f);
    }
    let _e188 = phi_1273_;
    phi_636_ = Mg;
    if Mg {
        let _e190 = S1_1[0u];
        phi_636_ = (_e190 != 0f);
    }
    let _e193 = phi_636_;
    phi_1274_ = _e188;
    if _e193 {
        let _e196 = d0_.X1_[_e88];
        phi_1274_ = min(unpack4x8unorm(_e196).x, _e188);
    }
    let _e201 = phi_1274_;
    let _e203 = clamp(_e144, 0f, max(_e201, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e209 = u32(((abs(_e203) * 1024f) + 0.5f));
            let _e212 = atomicLoad((&S0_.X1_[(_e150 + (((((_e151.y >> bitcast<u32>(5u)) * (_e148 << bitcast<u32>(5u))) + ((_e151.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e151.x & 28u) << bitcast<u32>(5u)) + ((_e151.y & 28u) << bitcast<u32>(2i)))) + (((_e151.y & 3u) << bitcast<u32>(2i)) + (_e151.x & 3u))))]));
            let _e214 = (min(_e143.w, _e203) >= 1f);
            phi_1022_ = _e214;
            if _e214 {
                let _e216 = k.W1_;
                let _e217 = (_e212 < _e216);
                phi_1020_ = _e217;
                if !(_e217) {
                    phi_1020_ = (_e212 >= (_e216 | 262144u));
                }
                let _e222 = phi_1020_;
                phi_1022_ = _e222;
            }
            let _e224 = phi_1022_;
            if _e224 {
                phi_1303_ = _e143.w;
                break;
            }
            let _e226 = k.W1_;
            phi_1293_ = 0f;
            phi_1290_ = _e209;
            phi_1287_ = _e203;
            if (_e212 < _e226) {
                let _e229 = (_e226 | (262144u + _e209));
                let _e230 = atomicMax((&S0_.X1_[(_e150 + (((((_e151.y >> bitcast<u32>(5u)) * (_e148 << bitcast<u32>(5u))) + ((_e151.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e151.x & 28u) << bitcast<u32>(5u)) + ((_e151.y & 28u) << bitcast<u32>(2i)))) + (((_e151.y & 3u) << bitcast<u32>(2i)) + (_e151.x & 3u))))]), _e229);
                if (_e230 <= _e226) {
                    phi_1296_ = min(_e203, 1f);
                    phi_1291_ = _e209;
                    phi_1288_ = 0f;
                } else {
                    phi_1297_ = 0f;
                    phi_1292_ = _e209;
                    phi_1289_ = _e203;
                    if (_e230 < _e229) {
                        let _e234 = ((_e230 & 524287u) - 262144u);
                        let _e236 = (f32(_e234) * 0.0009765625f);
                        phi_1297_ = ((min(_e203, 1f) - _e236) / max((1f - (_e236 * _e143.w)), 0.000062f));
                        phi_1292_ = _e234;
                        phi_1289_ = _e236;
                    }
                    let _e244 = phi_1297_;
                    let _e246 = phi_1292_;
                    let _e248 = phi_1289_;
                    phi_1296_ = _e244;
                    phi_1291_ = _e246;
                    phi_1288_ = _e248;
                }
                let _e251 = phi_1296_;
                let _e253 = phi_1291_;
                let _e255 = phi_1288_;
                phi_1293_ = _e251;
                phi_1290_ = _e253;
                phi_1287_ = _e255;
            }
            let _e257 = phi_1293_;
            let _e259 = phi_1290_;
            let _e261 = phi_1287_;
            phi_1301_ = _e257;
            if (_e261 > 0f) {
                let _e263 = atomicAdd((&S0_.X1_[(_e150 + (((((_e151.y >> bitcast<u32>(5u)) * (_e148 << bitcast<u32>(5u))) + ((_e151.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e151.x & 28u) << bitcast<u32>(5u)) + ((_e151.y & 28u) << bitcast<u32>(2i)))) + (((_e151.y & 3u) << bitcast<u32>(2i)) + (_e151.x & 3u))))]), _e259);
                let _e268 = (f32(bitcast<i32>(((_e263 & 524287u) - 262144u))) * 0.0009765625f);
                let _e270 = clamp(_e268, 0f, 1f);
                phi_1301_ = (_e257 + ((1f - (_e257 * _e143.w)) * ((clamp((_e268 + _e261), 0f, 1f) - _e270) / max((1f - (_e270 * _e143.w)), 0.000062f))));
            }
            let _e282 = phi_1301_;
            phi_1303_ = (_e143.w * _e282);
            break;
        }
    }
    let _e285 = phi_1303_;
    phi_1314_ = f32();
    if Tg {
        let _e287 = k.y3_;
        let _e289 = k.z3_;
        if Tg {
            phi_1311_ = ((fract((52.982918f * fract(((0.06711056f * _e53.x) + (0.00583715f * _e53.y))))) * _e287) + _e289);
        } else {
            phi_1311_ = 0f;
        }
        let _e301 = phi_1311_;
        phi_1314_ = _e301;
    }
    let _e303 = phi_1314_;
    let _e305 = (_e143.xyz * _e285);
    let _e309 = vec4<f32>(_e305.x, _e305.y, _e305.z, _e285);
    phi_1349_ = _e309;
    if Tg {
        let _e312 = (_e309.xyz + vec3(_e303));
        let _e318 = vec4<f32>(_e312.x, _e309.y, _e309.z, _e309.w);
        let _e324 = vec4<f32>(_e318.x, _e312.y, _e318.z, _e318.w);
        phi_1349_ = vec4<f32>(_e324.x, _e324.y, _e312.z, _e324.w);
    }
    let _e332 = phi_1349_;
    d0_.X1_[_e88] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    l1_ = _e332;
    return;
}

@fragment 
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) i1_: vec4<f32>, @location(1) @interpolate(flat) j1_: f32, @location(8) j4_: vec2<f32>, @location(7) @interpolate(flat) e3_: vec2<u32>, @location(5) N0_: vec4<f32>, @location(4) @interpolate(flat) S1_: vec2<f32>, @location(3) @interpolate(flat) z0_: f32, @location(6) @interpolate(flat) Z1_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    i1_1 = i1_;
    j1_1 = j1_;
    j4_1 = j4_;
    e3_1 = e3_;
    N0_1 = N0_;
    S1_1 = S1_;
    z0_1 = z0_;
    Z1_1 = Z1_;
    main_1();
    let _e19 = l1_;
    return _e19;
}
