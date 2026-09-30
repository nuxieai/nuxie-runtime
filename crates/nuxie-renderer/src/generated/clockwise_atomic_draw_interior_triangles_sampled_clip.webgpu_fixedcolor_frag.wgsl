struct Be {
    g2_: array<u32>,
}

struct AC {
    tc: f32,
    Dd: f32,
    Hf: f32,
    If: f32,
    q6_: u32,
    Qb: u32,
    tf: u32,
    uf: u32,
    X7_: vec4<i32>,
    eh: vec2<f32>,
    Ed: vec2<f32>,
    f2_: u32,
    ih: f32,
    f6_: u32,
    U2_: f32,
    Fd: f32,
    of_: u32,
    F3_: f32,
    G3_: f32,
    Gd: f32,
    bh: u32,
    Pb: u32,
}

struct Be_1 {
    g2_: array<atomic<u32>>,
}

@id(7) override Lh: bool = true;
@id(2) override Gh: bool = true;
@id(8) override Mh: bool = true;
@id(1) override Fh: bool = true;
@id(0) override Eh: bool = true;

@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var P9_: sampler;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var Y5_: sampler;
@group(0) @binding(6)
var<storage, read_write> R0_: Be_1;
@group(0) @binding(0)
var<uniform> j: AC;
var<private> C2_1: vec3<f32>;
var<private> g1_1: f32;
var<private> X1_1: vec4<f32>;
var<private> j1_1: f32;
var<private> r4_1: vec2<f32>;
var<private> i3_1: vec2<u32>;
var<private> N0_1: vec4<f32>;
var<private> Y1_1: vec2<f32>;
@group(2) @binding(1)
var i0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> E1_: vec4<f32>;
@group(3) @binding(9)
var ga: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> D0_1: f32;

fn main_1() {
    var phi_1218_: f32;
    var phi_1219_: f32;
    var phi_1233_: vec4<f32>;
    var phi_1232_: vec4<f32>;
    var phi_786_: bool;
    var phi_1220_: f32;
    var phi_1229_: vec4<f32>;
    var phi_1235_: vec4<f32>;
    var phi_1237_: f32;
    var phi_633_: bool;
    var phi_1238_: f32;
    var phi_991_: bool;
    var phi_993_: bool;
    var phi_1259_: f32;
    var phi_1254_: u32;
    var phi_1251_: f32;
    var phi_1258_: f32;
    var phi_1253_: u32;
    var phi_1250_: f32;
    var phi_1255_: f32;
    var phi_1252_: u32;
    var phi_1249_: f32;
    var phi_1260_: f32;
    var phi_1261_: f32;
    var phi_1262_: f32;
    var phi_1272_: f32;
    var phi_1292_: vec3<f32>;

    let _e50 = g1_1;
    let _e52 = C2_1;
    let _e53 = X1_1;
    let _e55 = (Gh && (u32(_e50) != 0u));
    if (_e53.w >= 0f) {
        phi_1232_ = _e53;
    } else {
        if (_e53.z > 0f) {
            phi_1218_ = _e53.x;
        } else {
            phi_1218_ = length(_e53.xy);
        }
        let _e65 = phi_1218_;
        let _e66 = clamp(_e65, 0f, 1f);
        let _e67 = abs(_e53.z);
        if (_e67 > 1f) {
            phi_1219_ = ((0.9980469f * _e66) + 0.0009765625f);
        } else {
            phi_1219_ = ((0.001953125f * _e66) + _e67);
        }
        let _e74 = phi_1219_;
        let _e76 = textureSampleLevel(DD, P9_, vec2<f32>(_e74, -(_e53.w)), 0f);
        phi_1233_ = _e76;
        if !(_e55) {
            let _e80 = (_e76.xyz * _e76.w);
            let _e86 = vec4<f32>(_e80.x, _e76.y, _e76.z, _e76.w);
            let _e92 = vec4<f32>(_e86.x, _e80.y, _e86.z, _e86.w);
            phi_1233_ = vec4<f32>(_e92.x, _e92.y, _e80.z, _e92.w);
        }
        let _e100 = phi_1233_;
        phi_1232_ = _e100;
    }
    let _e102 = phi_1232_;
    phi_786_ = Mh;
    if Mh {
        phi_786_ = (_e52.z > 0f);
    }
    let _e106 = phi_786_;
    phi_1235_ = _e102;
    if _e106 {
        let _e110 = textureSampleLevel(GC, Y5_, _e52.xy, (_e52.z - 1f));
        phi_1229_ = _e110;
        if _e55 {
            if (_e110.w != 0f) {
                phi_1220_ = (1f / _e110.w);
            } else {
                phi_1220_ = 0f;
            }
            let _e116 = phi_1220_;
            let _e117 = (_e110.xyz * _e116);
            phi_1229_ = vec4<f32>(_e117.x, _e117.y, _e117.z, _e110.w);
        }
        let _e123 = phi_1229_;
        phi_1235_ = (_e102 * _e123);
    }
    let _e126 = phi_1235_;
    let _e127 = j1_1;
    let _e128 = r4_1;
    let _e131 = i3_1[1u];
    let _e133 = i3_1[0u];
    let _e134 = vec2<u32>(floor(_e128));
    phi_1237_ = 1f;
    if Fh {
        let _e162 = N0_1;
        let _e165 = min(_e162.xy, _e162.zw);
        phi_1237_ = min(min(_e165.x, _e165.y), 1f);
    }
    let _e171 = phi_1237_;
    phi_633_ = Eh;
    if Eh {
        let _e173 = Y1_1[0u];
        phi_633_ = (_e173 != 0f);
    }
    let _e176 = phi_633_;
    phi_1238_ = _e171;
    if _e176 {
        let _e177 = gl_FragCoord_1;
        let _e181 = textureLoad(i0_, vec2<i32>(floor(_e177.xy)), 0i);
        phi_1238_ = min(_e181.x, _e171);
    }
    let _e185 = phi_1238_;
    let _e187 = clamp(_e127, 0f, max(_e185, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e193 = u32(((abs(_e187) * 1024f) + 0.5f));
            let _e196 = atomicLoad((&R0_.g2_[(_e133 + (((((_e134.y >> bitcast<u32>(5u)) * (_e131 << bitcast<u32>(5u))) + ((_e134.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e134.x & 28u) << bitcast<u32>(5u)) + ((_e134.y & 28u) << bitcast<u32>(2i)))) + (((_e134.y & 3u) << bitcast<u32>(2i)) + (_e134.x & 3u))))]));
            let _e198 = (min(_e126.w, _e187) >= 1f);
            phi_993_ = _e198;
            if _e198 {
                let _e200 = j.f2_;
                let _e201 = (_e196 < _e200);
                phi_991_ = _e201;
                if !(_e201) {
                    phi_991_ = (_e196 >= (_e200 | 262144u));
                }
                let _e206 = phi_991_;
                phi_993_ = _e206;
            }
            let _e208 = phi_993_;
            if _e208 {
                phi_1261_ = 1f;
                break;
            }
            let _e210 = j.f2_;
            phi_1255_ = 0f;
            phi_1252_ = _e193;
            phi_1249_ = _e187;
            if (_e196 < _e210) {
                let _e213 = (_e210 | (262144u + _e193));
                let _e214 = atomicMax((&R0_.g2_[(_e133 + (((((_e134.y >> bitcast<u32>(5u)) * (_e131 << bitcast<u32>(5u))) + ((_e134.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e134.x & 28u) << bitcast<u32>(5u)) + ((_e134.y & 28u) << bitcast<u32>(2i)))) + (((_e134.y & 3u) << bitcast<u32>(2i)) + (_e134.x & 3u))))]), _e213);
                if (_e214 <= _e210) {
                    phi_1258_ = min(_e187, 1f);
                    phi_1253_ = _e193;
                    phi_1250_ = 0f;
                } else {
                    phi_1259_ = 0f;
                    phi_1254_ = _e193;
                    phi_1251_ = _e187;
                    if (_e214 < _e213) {
                        let _e219 = ((_e214 & 524287u) - 262144u);
                        let _e221 = (f32(_e219) * 0.0009765625f);
                        phi_1259_ = ((min(_e187, 1f) - _e221) / max((1f - (_e221 * _e126.w)), 0.000062f));
                        phi_1254_ = _e219;
                        phi_1251_ = _e221;
                    }
                    let _e229 = phi_1259_;
                    let _e231 = phi_1254_;
                    let _e233 = phi_1251_;
                    phi_1258_ = _e229;
                    phi_1253_ = _e231;
                    phi_1250_ = _e233;
                }
                let _e235 = phi_1258_;
                let _e237 = phi_1253_;
                let _e239 = phi_1250_;
                phi_1255_ = _e235;
                phi_1252_ = _e237;
                phi_1249_ = _e239;
            }
            let _e241 = phi_1255_;
            let _e243 = phi_1252_;
            let _e245 = phi_1249_;
            phi_1260_ = _e241;
            if (_e245 > 0f) {
                let _e247 = atomicAdd((&R0_.g2_[(_e133 + (((((_e134.y >> bitcast<u32>(5u)) * (_e131 << bitcast<u32>(5u))) + ((_e134.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e134.x & 28u) << bitcast<u32>(5u)) + ((_e134.y & 28u) << bitcast<u32>(2i)))) + (((_e134.y & 3u) << bitcast<u32>(2i)) + (_e134.x & 3u))))]), _e243);
                let _e252 = (f32(bitcast<i32>(((_e247 & 524287u) - 262144u))) * 0.0009765625f);
                let _e254 = clamp(_e252, 0f, 1f);
                phi_1260_ = (_e241 + ((1f - (_e241 * _e126.w)) * ((clamp((_e252 + _e245), 0f, 1f) - _e254) / max((1f - (_e254 * _e126.w)), 0.000062f))));
            }
            let _e266 = phi_1260_;
            phi_1261_ = _e266;
            break;
        }
    }
    let _e268 = phi_1261_;
    phi_1272_ = f32();
    if Lh {
        let _e269 = gl_FragCoord_1;
        let _e271 = j.F3_;
        let _e273 = j.G3_;
        if Lh {
            phi_1262_ = ((fract((52.982918f * fract(((0.06711056f * _e269.x) + (0.00583715f * _e269.y))))) * _e271) + _e273);
        } else {
            phi_1262_ = 0f;
        }
        let _e285 = phi_1262_;
        phi_1272_ = _e285;
    }
    let _e287 = phi_1272_;
    let _e288 = (_e126 * _e268);
    let _e289 = _e288.xyz;
    if (Lh && (_e288.w != 0f)) {
        phi_1292_ = (vec3(_e287) + _e289);
    } else {
        phi_1292_ = _e289;
    }
    let _e296 = phi_1292_;
    let _e302 = vec4<f32>(_e296.x, _e288.y, _e288.z, _e288.w);
    let _e308 = vec4<f32>(_e302.x, _e296.y, _e302.z, _e302.w);
    E1_ = vec4<f32>(_e308.x, _e308.y, _e296.z, _e308.w);
    return;
}

@fragment
fn main(@location(9) C2_: vec3<f32>, @location(6) @interpolate(flat, either) g1_: f32, @location(0) X1_: vec4<f32>, @location(1) @interpolate(flat, either) j1_: f32, @location(8) r4_: vec2<f32>, @location(7) @interpolate(flat, either) i3_: vec2<u32>, @location(5) N0_: vec4<f32>, @location(4) @interpolate(flat, either) Y1_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) D0_: f32) -> @location(0) vec4<f32> {
    C2_1 = C2_;
    g1_1 = g1_;
    X1_1 = X1_;
    j1_1 = j1_;
    r4_1 = r4_;
    i3_1 = i3_;
    N0_1 = N0_;
    Y1_1 = Y1_;
    gl_FragCoord_1 = gl_FragCoord;
    D0_1 = D0_;
    main_1();
    let _e21 = E1_;
    return _e21;
}
