struct VB {
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
}

struct zf {
    r2_: array<u32>,
}

struct zf_1 {
    r2_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(0) member: vec4<f32>,
    @location(1) member_1: vec4<f32>,
}

@id(7) override dj: bool = true;
@id(2) override Yi: bool = true;
@id(8) override ej: bool = true;
@id(1) override Xi: bool = true;
@id(0) override Wi: bool = true;

@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var N8_: sampler;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;
@group(0) @binding(6)
var<storage, read_write> Y0_: zf_1;
var<private> U0_1: vec3<f32>;
var<private> P0_1: f32;
var<private> O0_1: vec4<f32>;
var<private> n1_1: f32;
var<private> L4_1: vec2<f32>;
var<private> z3_1: vec2<u32>;
var<private> V0_1: vec4<f32>;
var<private> i2_1: vec2<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> L1_: vec4<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> G0_1: f32;
var<private> m0_: vec4<f32>;

fn main_1() {
    var phi_1399_: f32;
    var phi_1414_: vec4<f32>;
    var phi_1413_: vec4<f32>;
    var phi_890_: bool;
    var phi_905_: bool;
    var phi_1400_: f32;
    var phi_1409_: vec4<f32>;
    var phi_1416_: vec4<f32>;
    var phi_1417_: vec4<f32>;
    var phi_1419_: f32;
    var phi_725_: bool;
    var phi_1420_: f32;
    var phi_1155_: bool;
    var phi_1157_: bool;
    var phi_1441_: f32;
    var phi_1436_: u32;
    var phi_1433_: f32;
    var phi_1440_: f32;
    var phi_1435_: u32;
    var phi_1432_: f32;
    var phi_1437_: f32;
    var phi_1434_: u32;
    var phi_1431_: f32;
    var phi_1442_: f32;
    var phi_1443_: f32;
    var phi_1444_: f32;
    var phi_1454_: f32;
    var phi_1475_: vec3<f32>;

    let _e55 = P0_1;
    let _e57 = U0_1;
    let _e58 = O0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e61 = (Yi && (u32(_e55) != 0u));
            if (_e58.w >= 0f) {
                phi_1413_ = _e58;
            } else {
                let _e65 = j.L8_;
                let _e67 = j.M8_;
                let _e70 = abs(_e58.z);
                let _e71 = floor(_e70);
                let _e74 = (((_e70 - _e71) * 8f) + 0.0009765625f);
                if (floor(_e74) == 2f) {
                    phi_1399_ = _e58.x;
                } else {
                    phi_1399_ = length(_e58.xy);
                }
                let _e83 = phi_1399_;
                let _e89 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e83, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e58.z < 0f))) + _e74), ((_e71 * _e65) + _e67)), 0f);
                phi_1414_ = _e89;
                if !(_e61) {
                    let _e95 = (_e89.xyz * _e89.w);
                    phi_1414_ = vec4<f32>(_e95.x, _e95.y, _e95.z, (_e89.w * (ceil(_e58.w) * -0.003921569f)));
                }
                let _e102 = phi_1414_;
                phi_1413_ = _e102;
            }
            let _e104 = phi_1413_;
            phi_890_ = ej;
            if ej {
                phi_890_ = (_e57.z < 0f);
            }
            let _e108 = phi_890_;
            if _e108 {
                let _e110 = textureSampleLevel(TB, U4_, _e57.xy, 0f);
                phi_1417_ = _e110;
                break;
            }
            phi_905_ = ej;
            if ej {
                phi_905_ = (_e57.z > 0f);
            }
            let _e114 = phi_905_;
            phi_1416_ = _e104;
            if _e114 {
                let _e118 = textureSampleLevel(TB, U4_, _e57.xy, (_e57.z - 1f));
                phi_1409_ = _e118;
                if _e61 {
                    if (_e118.w != 0f) {
                        phi_1400_ = (1f / _e118.w);
                    } else {
                        phi_1400_ = 0f;
                    }
                    let _e124 = phi_1400_;
                    let _e125 = (_e118.xyz * _e124);
                    phi_1409_ = vec4<f32>(_e125.x, _e125.y, _e125.z, _e118.w);
                }
                let _e131 = phi_1409_;
                phi_1416_ = (_e104 * _e131);
            }
            let _e134 = phi_1416_;
            phi_1417_ = _e134;
            break;
        }
    }
    let _e136 = phi_1417_;
    let _e137 = n1_1;
    let _e138 = L4_1;
    let _e141 = z3_1[1u];
    let _e143 = z3_1[0u];
    let _e144 = vec2<u32>(floor(_e138));
    phi_1419_ = 1f;
    if Xi {
        let _e172 = V0_1;
        let _e175 = min(_e172.xy, _e172.zw);
        phi_1419_ = min(min(_e175.x, _e175.y), 1f);
    }
    let _e181 = phi_1419_;
    phi_725_ = Wi;
    if Wi {
        let _e183 = i2_1[0u];
        phi_725_ = (_e183 != 0f);
    }
    let _e186 = phi_725_;
    phi_1420_ = _e181;
    if _e186 {
        phi_1420_ = min(0f, _e181);
    }
    let _e189 = phi_1420_;
    let _e191 = clamp(_e137, 0f, max(_e189, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e197 = u32(((abs(_e191) * 1024f) + 0.5f));
            let _e200 = atomicLoad((&Y0_.r2_[(_e143 + (((((_e144.y >> bitcast<u32>(5u)) * (_e141 << bitcast<u32>(5u))) + ((_e144.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e144.x & 28u) << bitcast<u32>(5u)) + ((_e144.y & 28u) << bitcast<u32>(2i)))) + (((_e144.y & 3u) << bitcast<u32>(2i)) + (_e144.x & 3u))))]));
            let _e202 = (min(_e136.w, _e191) >= 1f);
            phi_1157_ = _e202;
            if _e202 {
                let _e204 = j.q2_;
                let _e205 = (_e200 < _e204);
                phi_1155_ = _e205;
                if !(_e205) {
                    phi_1155_ = (_e200 >= (_e204 | 262144u));
                }
                let _e210 = phi_1155_;
                phi_1157_ = _e210;
            }
            let _e212 = phi_1157_;
            if _e212 {
                phi_1443_ = 1f;
                break;
            }
            let _e214 = j.q2_;
            phi_1437_ = 0f;
            phi_1434_ = _e197;
            phi_1431_ = _e191;
            if (_e200 < _e214) {
                let _e217 = (_e214 | (262144u + _e197));
                let _e218 = atomicMax((&Y0_.r2_[(_e143 + (((((_e144.y >> bitcast<u32>(5u)) * (_e141 << bitcast<u32>(5u))) + ((_e144.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e144.x & 28u) << bitcast<u32>(5u)) + ((_e144.y & 28u) << bitcast<u32>(2i)))) + (((_e144.y & 3u) << bitcast<u32>(2i)) + (_e144.x & 3u))))]), _e217);
                if (_e218 <= _e214) {
                    phi_1440_ = min(_e191, 1f);
                    phi_1435_ = _e197;
                    phi_1432_ = 0f;
                } else {
                    phi_1441_ = 0f;
                    phi_1436_ = _e197;
                    phi_1433_ = _e191;
                    if (_e218 < _e217) {
                        let _e223 = ((_e218 & 524287u) - 262144u);
                        let _e225 = (f32(_e223) * 0.0009765625f);
                        phi_1441_ = ((min(_e191, 1f) - _e225) / max((1f - (_e225 * _e136.w)), 0.000062f));
                        phi_1436_ = _e223;
                        phi_1433_ = _e225;
                    }
                    let _e233 = phi_1441_;
                    let _e235 = phi_1436_;
                    let _e237 = phi_1433_;
                    phi_1440_ = _e233;
                    phi_1435_ = _e235;
                    phi_1432_ = _e237;
                }
                let _e239 = phi_1440_;
                let _e241 = phi_1435_;
                let _e243 = phi_1432_;
                phi_1437_ = _e239;
                phi_1434_ = _e241;
                phi_1431_ = _e243;
            }
            let _e245 = phi_1437_;
            let _e247 = phi_1434_;
            let _e249 = phi_1431_;
            phi_1442_ = _e245;
            if (_e249 > 0f) {
                let _e251 = atomicAdd((&Y0_.r2_[(_e143 + (((((_e144.y >> bitcast<u32>(5u)) * (_e141 << bitcast<u32>(5u))) + ((_e144.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e144.x & 28u) << bitcast<u32>(5u)) + ((_e144.y & 28u) << bitcast<u32>(2i)))) + (((_e144.y & 3u) << bitcast<u32>(2i)) + (_e144.x & 3u))))]), _e247);
                let _e256 = (f32(bitcast<i32>(((_e251 & 524287u) - 262144u))) * 0.0009765625f);
                let _e258 = clamp(_e256, 0f, 1f);
                phi_1442_ = (_e245 + ((1f - (_e245 * _e136.w)) * ((clamp((_e256 + _e249), 0f, 1f) - _e258) / max((1f - (_e258 * _e136.w)), 0.000062f))));
            }
            let _e270 = phi_1442_;
            phi_1443_ = _e270;
            break;
        }
    }
    let _e272 = phi_1443_;
    phi_1454_ = f32();
    if dj {
        let _e273 = gl_FragCoord_1;
        let _e275 = j.F3_;
        let _e277 = j.G3_;
        if dj {
            phi_1444_ = ((fract((52.982918f * fract(((0.06711056f * _e273.x) + (0.00583715f * _e273.y))))) * _e275) + _e277);
        } else {
            phi_1444_ = 0f;
        }
        let _e289 = phi_1444_;
        phi_1454_ = _e289;
    }
    let _e291 = phi_1454_;
    let _e292 = (_e136 * _e272);
    let _e293 = _e292.xyz;
    if (dj && (_e292.w != 0f)) {
        phi_1475_ = (vec3(_e291) + _e293);
    } else {
        phi_1475_ = _e293;
    }
    let _e300 = phi_1475_;
    let _e306 = vec4<f32>(_e300.x, _e292.y, _e292.z, _e292.w);
    let _e312 = vec4<f32>(_e306.x, _e300.y, _e306.z, _e306.w);
    L1_ = vec4<f32>(_e312.x, _e312.y, _e300.z, _e312.w);
    return;
}

@fragment
fn main(@location(9) U0_: vec3<f32>, @location(6) @interpolate(flat, either) P0_: f32, @location(0) O0_: vec4<f32>, @location(1) @interpolate(flat, either) n1_: f32, @location(8) L4_: vec2<f32>, @location(7) @interpolate(flat, either) z3_: vec2<u32>, @location(5) V0_: vec4<f32>, @location(4) @interpolate(flat, either) i2_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32) -> FragmentOutput {
    U0_1 = U0_;
    P0_1 = P0_;
    O0_1 = O0_;
    n1_1 = n1_;
    L4_1 = L4_;
    z3_1 = z3_;
    V0_1 = V0_;
    i2_1 = i2_;
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    main_1();
    let _e22 = L1_;
    let _e23 = m0_;
    return FragmentOutput(_e22, _e23);
}
