struct Ae {
    e2_: array<u32>,
}

struct BC {
    sc: f32,
    Cd: f32,
    Gf: f32,
    Hf: f32,
    o6_: u32,
    Pb: u32,
    sf: u32,
    tf: u32,
    V7_: vec4<i32>,
    dh: vec2<f32>,
    Dd: vec2<f32>,
    d2_: u32,
    hh: f32,
    d6_: u32,
    T2_: f32,
    Ed: f32,
    nf: u32,
    C3_: f32,
    D3_: f32,
    Fd: f32,
    ah: u32,
    Ob: u32,
}

struct Ae_1 {
    e2_: array<atomic<u32>>,
}

@id(7) override Kh: bool = true;
@id(2) override Fh: bool = true;
@id(8) override Lh: bool = true;
@id(1) override Eh: bool = true;
@id(0) override Dh: bool = true;

@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var O9_: sampler;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var V5_: sampler;
@group(0) @binding(6)
var<storage, read_write> Q0_: Ae_1;
@group(0) @binding(0)
var<uniform> l: BC;
var<private> V1_1: vec4<f32>;
var<private> B2_1: vec3<f32>;
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
var fa: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> C0_1: f32;
var<private> g2_1: f32;

fn main_1() {
    var phi_1246_: vec4<f32>;
    var phi_1230_: f32;
    var phi_1231_: f32;
    var phi_1247_: vec4<f32>;
    var phi_1245_: vec4<f32>;
    var phi_793_: bool;
    var phi_1232_: f32;
    var phi_1242_: vec4<f32>;
    var phi_1249_: vec4<f32>;
    var phi_1251_: f32;
    var phi_631_: bool;
    var phi_1252_: f32;
    var phi_997_: bool;
    var phi_999_: bool;
    var phi_1273_: f32;
    var phi_1268_: u32;
    var phi_1265_: f32;
    var phi_1272_: f32;
    var phi_1267_: u32;
    var phi_1264_: f32;
    var phi_1269_: f32;
    var phi_1266_: u32;
    var phi_1263_: f32;
    var phi_1274_: f32;
    var phi_1275_: f32;
    var phi_1276_: f32;
    var phi_1286_: f32;
    var phi_1307_: vec3<f32>;

    let _e50 = V1_1;
    let _e51 = B2_1;
    if (_e50.w >= 0f) {
        if Fh {
            phi_1246_ = vec4<f32>(_e50.x, _e50.y, _e50.z, _e50.w);
        } else {
            phi_1246_ = (_e50 * 1f);
        }
        let _e62 = phi_1246_;
        phi_1245_ = _e62;
    } else {
        if (_e50.z > 0f) {
            phi_1230_ = _e50.x;
        } else {
            phi_1230_ = length(_e50.xy);
        }
        let _e70 = phi_1230_;
        let _e71 = clamp(_e70, 0f, 1f);
        let _e72 = abs(_e50.z);
        if (_e72 > 1f) {
            phi_1231_ = ((0.9980469f * _e71) + 0.0009765625f);
        } else {
            phi_1231_ = ((0.001953125f * _e71) + _e72);
        }
        let _e79 = phi_1231_;
        let _e81 = textureSampleLevel(ED, O9_, vec2<f32>(_e79, -(_e50.w)), 0f);
        let _e87 = vec4<f32>(_e81.x, _e81.y, _e81.z, _e81.w);
        if Fh {
            phi_1247_ = _e87;
        } else {
            let _e89 = (_e87.xyz * _e81.w);
            phi_1247_ = vec4<f32>(_e89.x, _e89.y, _e89.z, _e81.w);
        }
        let _e95 = phi_1247_;
        phi_1245_ = _e95;
    }
    let _e97 = phi_1245_;
    phi_793_ = Lh;
    if Lh {
        phi_793_ = (_e51.z > 0f);
    }
    let _e101 = phi_793_;
    phi_1249_ = _e97;
    if _e101 {
        let _e105 = textureSampleLevel(HC, V5_, _e51.xy, (_e51.z - 1f));
        phi_1242_ = _e105;
        if Fh {
            if (_e105.w != 0f) {
                phi_1232_ = (1f / _e105.w);
            } else {
                phi_1232_ = 0f;
            }
            let _e111 = phi_1232_;
            let _e112 = (_e105.xyz * _e111);
            phi_1242_ = vec4<f32>(_e112.x, _e112.y, _e112.z, _e105.w);
        }
        let _e118 = phi_1242_;
        phi_1249_ = (_e97 * _e118);
    }
    let _e121 = phi_1249_;
    let _e122 = h1_1;
    let _e123 = p4_1;
    let _e126 = g3_1[1u];
    let _e128 = g3_1[0u];
    let _e129 = vec2<u32>(floor(_e123));
    phi_1251_ = 1f;
    if Eh {
        let _e157 = M0_1;
        let _e160 = min(_e157.xy, _e157.zw);
        phi_1251_ = min(min(_e160.x, _e160.y), 1f);
    }
    let _e166 = phi_1251_;
    phi_631_ = Dh;
    if Dh {
        let _e168 = W1_1[0u];
        phi_631_ = (_e168 != 0f);
    }
    let _e171 = phi_631_;
    phi_1252_ = _e166;
    if _e171 {
        let _e172 = gl_FragCoord_1;
        let _e176 = textureLoad(h0_, vec2<i32>(floor(_e172.xy)), 0i);
        phi_1252_ = min(_e176.x, _e166);
    }
    let _e180 = phi_1252_;
    let _e182 = clamp(_e122, 0f, max(_e180, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e188 = u32(((abs(_e182) * 1024f) + 0.5f));
            let _e191 = atomicLoad((&Q0_.e2_[(_e128 + (((((_e129.y >> bitcast<u32>(5u)) * (_e126 << bitcast<u32>(5u))) + ((_e129.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e129.x & 28u) << bitcast<u32>(5u)) + ((_e129.y & 28u) << bitcast<u32>(2i)))) + (((_e129.y & 3u) << bitcast<u32>(2i)) + (_e129.x & 3u))))]));
            let _e193 = (min(_e121.w, _e182) >= 1f);
            phi_999_ = _e193;
            if _e193 {
                let _e195 = l.d2_;
                let _e196 = (_e191 < _e195);
                phi_997_ = _e196;
                if !(_e196) {
                    phi_997_ = (_e191 >= (_e195 | 262144u));
                }
                let _e201 = phi_997_;
                phi_999_ = _e201;
            }
            let _e203 = phi_999_;
            if _e203 {
                phi_1275_ = 1f;
                break;
            }
            let _e205 = l.d2_;
            phi_1269_ = 0f;
            phi_1266_ = _e188;
            phi_1263_ = _e182;
            if (_e191 < _e205) {
                let _e208 = (_e205 | (262144u + _e188));
                let _e209 = atomicMax((&Q0_.e2_[(_e128 + (((((_e129.y >> bitcast<u32>(5u)) * (_e126 << bitcast<u32>(5u))) + ((_e129.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e129.x & 28u) << bitcast<u32>(5u)) + ((_e129.y & 28u) << bitcast<u32>(2i)))) + (((_e129.y & 3u) << bitcast<u32>(2i)) + (_e129.x & 3u))))]), _e208);
                if (_e209 <= _e205) {
                    phi_1272_ = min(_e182, 1f);
                    phi_1267_ = _e188;
                    phi_1264_ = 0f;
                } else {
                    phi_1273_ = 0f;
                    phi_1268_ = _e188;
                    phi_1265_ = _e182;
                    if (_e209 < _e208) {
                        let _e214 = ((_e209 & 524287u) - 262144u);
                        let _e216 = (f32(_e214) * 0.0009765625f);
                        phi_1273_ = ((min(_e182, 1f) - _e216) / max((1f - (_e216 * _e121.w)), 0.000062f));
                        phi_1268_ = _e214;
                        phi_1265_ = _e216;
                    }
                    let _e224 = phi_1273_;
                    let _e226 = phi_1268_;
                    let _e228 = phi_1265_;
                    phi_1272_ = _e224;
                    phi_1267_ = _e226;
                    phi_1264_ = _e228;
                }
                let _e230 = phi_1272_;
                let _e232 = phi_1267_;
                let _e234 = phi_1264_;
                phi_1269_ = _e230;
                phi_1266_ = _e232;
                phi_1263_ = _e234;
            }
            let _e236 = phi_1269_;
            let _e238 = phi_1266_;
            let _e240 = phi_1263_;
            phi_1274_ = _e236;
            if (_e240 > 0f) {
                let _e242 = atomicAdd((&Q0_.e2_[(_e128 + (((((_e129.y >> bitcast<u32>(5u)) * (_e126 << bitcast<u32>(5u))) + ((_e129.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e129.x & 28u) << bitcast<u32>(5u)) + ((_e129.y & 28u) << bitcast<u32>(2i)))) + (((_e129.y & 3u) << bitcast<u32>(2i)) + (_e129.x & 3u))))]), _e238);
                let _e247 = (f32(bitcast<i32>(((_e242 & 524287u) - 262144u))) * 0.0009765625f);
                let _e249 = clamp(_e247, 0f, 1f);
                phi_1274_ = (_e236 + ((1f - (_e236 * _e121.w)) * ((clamp((_e247 + _e240), 0f, 1f) - _e249) / max((1f - (_e249 * _e121.w)), 0.000062f))));
            }
            let _e261 = phi_1274_;
            phi_1275_ = _e261;
            break;
        }
    }
    let _e263 = phi_1275_;
    phi_1286_ = f32();
    if Kh {
        let _e264 = gl_FragCoord_1;
        let _e266 = l.C3_;
        let _e268 = l.D3_;
        if Kh {
            phi_1276_ = ((fract((52.982918f * fract(((0.06711056f * _e264.x) + (0.00583715f * _e264.y))))) * _e266) + _e268);
        } else {
            phi_1276_ = 0f;
        }
        let _e280 = phi_1276_;
        phi_1286_ = _e280;
    }
    let _e282 = phi_1286_;
    let _e283 = (_e121 * _e263);
    let _e284 = _e283.xyz;
    if (Kh && (_e283.w != 0f)) {
        phi_1307_ = (vec3(_e282) + _e284);
    } else {
        phi_1307_ = _e284;
    }
    let _e291 = phi_1307_;
    let _e297 = vec4<f32>(_e291.x, _e283.y, _e283.z, _e283.w);
    let _e303 = vec4<f32>(_e297.x, _e291.y, _e297.z, _e297.w);
    C1_ = vec4<f32>(_e303.x, _e303.y, _e291.z, _e303.w);
    return;
}

@fragment
fn main(@location(0) V1_: vec4<f32>, @location(9) B2_: vec3<f32>, @location(1) @interpolate(flat, either) h1_: f32, @location(8) p4_: vec2<f32>, @location(7) @interpolate(flat, either) g3_: vec2<u32>, @location(5) M0_: vec4<f32>, @location(4) @interpolate(flat, either) W1_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) C0_: f32, @location(6) @interpolate(flat, either) g2_: f32) -> @location(0) vec4<f32> {
    V1_1 = V1_;
    B2_1 = B2_;
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
