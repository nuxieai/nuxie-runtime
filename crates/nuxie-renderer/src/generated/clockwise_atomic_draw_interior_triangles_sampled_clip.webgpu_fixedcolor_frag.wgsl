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

struct zf {
    v2_: array<u32>,
}

struct zf_1 {
    v2_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(1) member: vec4<f32>,
    @location(0) member_1: vec4<f32>,
}

@id(7) override fj: bool = true;
@id(2) override aj: bool = true;
@id(8) override gj: bool = true;
@id(1) override Zi: bool = true;
@id(0) override Yi: bool = true;

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
@group(0) @binding(6)
var<storage, read_write> Z0_: zf_1;
var<private> V0_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> P0_1: vec4<f32>;
var<private> o1_1: f32;
var<private> J4_1: vec2<f32>;
var<private> y3_1: vec2<u32>;
var<private> W0_1: vec4<f32>;
var<private> j2_1: vec2<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> m0_: vec4<f32>;
var<private> N1_: vec4<f32>;
@group(3) @binding(9)
var Va: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
var<private> G0_1: f32;

fn main_1() {
    var phi_1371_: f32;
    var phi_1372_: f32;
    var phi_1388_: vec4<f32>;
    var phi_1387_: vec4<f32>;
    var phi_885_: bool;
    var phi_900_: bool;
    var phi_1373_: f32;
    var phi_1383_: vec4<f32>;
    var phi_1390_: vec4<f32>;
    var phi_1391_: vec4<f32>;
    var phi_1393_: f32;
    var phi_707_: bool;
    var phi_1394_: f32;
    var phi_1116_: bool;
    var phi_1118_: bool;
    var phi_1415_: f32;
    var phi_1410_: u32;
    var phi_1407_: f32;
    var phi_1414_: f32;
    var phi_1409_: u32;
    var phi_1406_: f32;
    var phi_1411_: f32;
    var phi_1408_: u32;
    var phi_1405_: f32;
    var phi_1416_: f32;
    var phi_1417_: f32;
    var phi_1418_: f32;
    var phi_1428_: f32;
    var phi_1450_: vec3<f32>;

    let _e54 = Q0_1;
    let _e56 = V0_1;
    let _e57 = P0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e60 = (aj && (u32(_e54) != 0u));
            if (_e57.w >= 0f) {
                phi_1387_ = _e57;
            } else {
                let _e63 = -(_e57.w);
                let _e68 = j.cd;
                let _e71 = j.g7_;
                if (_e57.z > 0f) {
                    phi_1371_ = _e57.x;
                } else {
                    phi_1371_ = length(_e57.xy);
                }
                let _e79 = phi_1371_;
                let _e80 = clamp(_e79, 0f, 1f);
                let _e81 = abs(_e57.z);
                if (_e81 > 1f) {
                    phi_1372_ = ((0.9980469f * _e80) + 0.0009765625f);
                } else {
                    phi_1372_ = ((0.001953125f * _e80) + _e81);
                }
                let _e88 = phi_1372_;
                let _e90 = textureSampleLevel(YC, I8_, vec2<f32>(_e88, ((floor(_e63) * _e68) + _e71)), 0f);
                phi_1388_ = _e90;
                if !(_e60) {
                    let _e94 = (_e90.xyz * _e90.w);
                    phi_1388_ = vec4<f32>(_e94.x, _e94.y, _e94.z, (_e90.w * (fract(_e63) * 1.0039216f)));
                }
                let _e101 = phi_1388_;
                phi_1387_ = _e101;
            }
            let _e103 = phi_1387_;
            phi_885_ = gj;
            if gj {
                phi_885_ = (_e56.z < 0f);
            }
            let _e107 = phi_885_;
            if _e107 {
                let _e109 = textureSampleLevel(TB, S4_, _e56.xy, 0f);
                phi_1391_ = _e109;
                break;
            }
            phi_900_ = gj;
            if gj {
                phi_900_ = (_e56.z > 0f);
            }
            let _e113 = phi_900_;
            phi_1390_ = _e103;
            if _e113 {
                let _e117 = textureSampleLevel(TB, S4_, _e56.xy, (_e56.z - 1f));
                phi_1383_ = _e117;
                if _e60 {
                    if (_e117.w != 0f) {
                        phi_1373_ = (1f / _e117.w);
                    } else {
                        phi_1373_ = 0f;
                    }
                    let _e123 = phi_1373_;
                    let _e124 = (_e117.xyz * _e123);
                    phi_1383_ = vec4<f32>(_e124.x, _e124.y, _e124.z, _e117.w);
                }
                let _e130 = phi_1383_;
                phi_1390_ = (_e103 * _e130);
            }
            let _e133 = phi_1390_;
            phi_1391_ = _e133;
            break;
        }
    }
    let _e135 = phi_1391_;
    let _e136 = o1_1;
    let _e137 = J4_1;
    let _e140 = y3_1[1u];
    let _e142 = y3_1[0u];
    let _e143 = vec2<u32>(floor(_e137));
    phi_1393_ = 1f;
    if Zi {
        let _e171 = W0_1;
        let _e174 = min(_e171.xy, _e171.zw);
        phi_1393_ = min(min(_e174.x, _e174.y), 1f);
    }
    let _e180 = phi_1393_;
    phi_707_ = Yi;
    if Yi {
        let _e182 = j2_1[0u];
        phi_707_ = (_e182 != 0f);
    }
    let _e185 = phi_707_;
    phi_1394_ = _e180;
    if _e185 {
        phi_1394_ = min(0f, _e180);
    }
    let _e188 = phi_1394_;
    let _e190 = clamp(_e136, 0f, max(_e188, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e196 = u32(((abs(_e190) * 1024f) + 0.5f));
            let _e199 = atomicLoad((&Z0_.v2_[(_e142 + (((((_e143.y >> bitcast<u32>(5u)) * (_e140 << bitcast<u32>(5u))) + ((_e143.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e143.x & 28u) << bitcast<u32>(5u)) + ((_e143.y & 28u) << bitcast<u32>(2i)))) + (((_e143.y & 3u) << bitcast<u32>(2i)) + (_e143.x & 3u))))]));
            let _e201 = (min(_e135.w, _e190) >= 1f);
            phi_1118_ = _e201;
            if _e201 {
                let _e203 = j.r2_;
                let _e204 = (_e199 < _e203);
                phi_1116_ = _e204;
                if !(_e204) {
                    phi_1116_ = (_e199 >= (_e203 | 262144u));
                }
                let _e209 = phi_1116_;
                phi_1118_ = _e209;
            }
            let _e211 = phi_1118_;
            if _e211 {
                phi_1417_ = 1f;
                break;
            }
            let _e213 = j.r2_;
            phi_1411_ = 0f;
            phi_1408_ = _e196;
            phi_1405_ = _e190;
            if (_e199 < _e213) {
                let _e216 = (_e213 | (262144u + _e196));
                let _e217 = atomicMax((&Z0_.v2_[(_e142 + (((((_e143.y >> bitcast<u32>(5u)) * (_e140 << bitcast<u32>(5u))) + ((_e143.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e143.x & 28u) << bitcast<u32>(5u)) + ((_e143.y & 28u) << bitcast<u32>(2i)))) + (((_e143.y & 3u) << bitcast<u32>(2i)) + (_e143.x & 3u))))]), _e216);
                if (_e217 <= _e213) {
                    phi_1414_ = min(_e190, 1f);
                    phi_1409_ = _e196;
                    phi_1406_ = 0f;
                } else {
                    phi_1415_ = 0f;
                    phi_1410_ = _e196;
                    phi_1407_ = _e190;
                    if (_e217 < _e216) {
                        let _e222 = ((_e217 & 524287u) - 262144u);
                        let _e224 = (f32(_e222) * 0.0009765625f);
                        phi_1415_ = ((min(_e190, 1f) - _e224) / max((1f - (_e224 * _e135.w)), 0.000062f));
                        phi_1410_ = _e222;
                        phi_1407_ = _e224;
                    }
                    let _e232 = phi_1415_;
                    let _e234 = phi_1410_;
                    let _e236 = phi_1407_;
                    phi_1414_ = _e232;
                    phi_1409_ = _e234;
                    phi_1406_ = _e236;
                }
                let _e238 = phi_1414_;
                let _e240 = phi_1409_;
                let _e242 = phi_1406_;
                phi_1411_ = _e238;
                phi_1408_ = _e240;
                phi_1405_ = _e242;
            }
            let _e244 = phi_1411_;
            let _e246 = phi_1408_;
            let _e248 = phi_1405_;
            phi_1416_ = _e244;
            if (_e248 > 0f) {
                let _e250 = atomicAdd((&Z0_.v2_[(_e142 + (((((_e143.y >> bitcast<u32>(5u)) * (_e140 << bitcast<u32>(5u))) + ((_e143.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e143.x & 28u) << bitcast<u32>(5u)) + ((_e143.y & 28u) << bitcast<u32>(2i)))) + (((_e143.y & 3u) << bitcast<u32>(2i)) + (_e143.x & 3u))))]), _e246);
                let _e255 = (f32(bitcast<i32>(((_e250 & 524287u) - 262144u))) * 0.0009765625f);
                let _e257 = clamp(_e255, 0f, 1f);
                phi_1416_ = (_e244 + ((1f - (_e244 * _e135.w)) * ((clamp((_e255 + _e248), 0f, 1f) - _e257) / max((1f - (_e257 * _e135.w)), 0.000062f))));
            }
            let _e269 = phi_1416_;
            phi_1417_ = _e269;
            break;
        }
    }
    let _e271 = phi_1417_;
    phi_1428_ = f32();
    if fj {
        let _e272 = gl_FragCoord_1;
        let _e274 = j.E3_;
        let _e276 = j.F3_;
        if fj {
            phi_1418_ = ((fract((52.982918f * fract(((0.06711056f * _e272.x) + (0.00583715f * _e272.y))))) * _e274) + _e276);
        } else {
            phi_1418_ = 0f;
        }
        let _e288 = phi_1418_;
        phi_1428_ = _e288;
    }
    let _e290 = phi_1428_;
    let _e291 = (_e135 * _e271);
    let _e292 = _e291.xyz;
    if (fj && (_e291.w != 0f)) {
        phi_1450_ = (vec3(_e290) + _e292);
    } else {
        phi_1450_ = _e292;
    }
    let _e299 = phi_1450_;
    let _e305 = vec4<f32>(_e299.x, _e291.y, _e291.z, _e291.w);
    let _e311 = vec4<f32>(_e305.x, _e299.y, _e305.z, _e305.w);
    m0_ = vec4<f32>(0f, 0f, 0f, 0f);
    N1_ = vec4<f32>(_e311.x, _e311.y, _e299.z, _e311.w);
    return;
}

@fragment
fn main(@location(9) V0_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) P0_: vec4<f32>, @location(1) @interpolate(flat, either) o1_: f32, @location(8) J4_: vec2<f32>, @location(7) @interpolate(flat, either) y3_: vec2<u32>, @location(5) W0_: vec4<f32>, @location(4) @interpolate(flat, either) j2_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32) -> FragmentOutput {
    V0_1 = V0_;
    Q0_1 = Q0_;
    P0_1 = P0_;
    o1_1 = o1_;
    J4_1 = J4_;
    y3_1 = y3_;
    W0_1 = W0_;
    j2_1 = j2_;
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    main_1();
    let _e22 = m0_;
    let _e23 = N1_;
    return FragmentOutput(_e22, _e23);
}
