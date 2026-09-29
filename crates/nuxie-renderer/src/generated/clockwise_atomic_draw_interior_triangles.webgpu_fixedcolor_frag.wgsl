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

struct h0Qd {
    e2_: array<u32>,
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
var<private> gl_FragCoord_1: vec4<f32>;
var<private> V1_1: vec4<f32>;
var<private> C2_1: vec3<f32>;
var<private> h1_1: f32;
var<private> p4_1: vec2<f32>;
var<private> g3_1: vec2<u32>;
var<private> M0_1: vec4<f32>;
var<private> W1_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Qd;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var ea: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> C0_1: f32;
var<private> g2_1: f32;

fn main_1() {
    var phi_1326_: f32;
    var phi_1327_: f32;
    var phi_1338_: vec4<f32>;
    var phi_862_: bool;
    var phi_1328_: f32;
    var phi_1339_: vec4<f32>;
    var phi_1341_: f32;
    var phi_669_: bool;
    var phi_1342_: f32;
    var phi_1064_: bool;
    var phi_1066_: bool;
    var phi_1365_: f32;
    var phi_1360_: u32;
    var phi_1357_: f32;
    var phi_1364_: f32;
    var phi_1359_: u32;
    var phi_1356_: f32;
    var phi_1361_: f32;
    var phi_1358_: u32;
    var phi_1355_: f32;
    var phi_1369_: f32;
    var phi_1371_: f32;
    var phi_1379_: f32;
    var phi_1382_: f32;
    var phi_1400_: vec3<f32>;

    let _e53 = gl_FragCoord_1;
    let _e57 = bitcast<vec2<u32>>(vec2<i32>(floor(_e53.xy)));
    let _e59 = n.q6_;
    let _e88 = bitcast<i32>((((((_e57.y >> bitcast<u32>(5u)) * (((_e59 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e57.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e57.x & 28u) << bitcast<u32>(5u)) + ((_e57.y & 28u) << bitcast<u32>(2i)))) + (((_e57.y & 3u) << bitcast<u32>(2i)) + (_e57.x & 3u))));
    let _e89 = V1_1;
    let _e90 = C2_1;
    if (_e89.w >= 0f) {
        phi_1338_ = vec4<f32>(_e89.x, _e89.y, _e89.z, _e89.w);
    } else {
        if (_e89.z > 0f) {
            phi_1326_ = _e89.x;
        } else {
            phi_1326_ = length(_e89.xy);
        }
        let _e100 = phi_1326_;
        let _e101 = clamp(_e100, 0f, 1f);
        let _e102 = abs(_e89.z);
        if (_e102 > 1f) {
            phi_1327_ = ((0.9980469f * _e101) + 0.0009765625f);
        } else {
            phi_1327_ = ((0.001953125f * _e101) + _e102);
        }
        let _e109 = phi_1327_;
        let _e111 = textureSampleLevel(ED, N9_, vec2<f32>(_e109, -(_e89.w)), 0f);
        phi_1338_ = vec4<f32>(_e111.x, _e111.y, _e111.z, _e111.w);
    }
    let _e125 = phi_1338_;
    phi_862_ = Jh;
    if Jh {
        phi_862_ = (_e90.z > 0f);
    }
    let _e129 = phi_862_;
    phi_1339_ = _e125;
    if _e129 {
        let _e133 = textureSampleLevel(HC, W5_, _e90.xy, (_e90.z - 1f));
        if (_e133.w != 0f) {
            phi_1328_ = (1f / _e133.w);
        } else {
            phi_1328_ = 0f;
        }
        let _e139 = phi_1328_;
        let _e140 = (_e133.xyz * _e139);
        phi_1339_ = (_e125 * vec4<f32>(_e140.x, _e140.y, _e140.z, _e133.w));
    }
    let _e147 = phi_1339_;
    let _e148 = h1_1;
    let _e149 = p4_1;
    let _e152 = g3_1[1u];
    let _e154 = g3_1[0u];
    let _e155 = vec2<u32>(floor(_e149));
    phi_1341_ = 1f;
    if Ch {
        let _e183 = M0_1;
        let _e186 = min(_e183.xy, _e183.zw);
        phi_1341_ = min(min(_e186.x, _e186.y), 1f);
    }
    let _e192 = phi_1341_;
    phi_669_ = Bh;
    if Bh {
        let _e194 = W1_1[0u];
        phi_669_ = (_e194 != 0f);
    }
    let _e197 = phi_669_;
    phi_1342_ = _e192;
    if _e197 {
        let _e200 = h0_.e2_[_e88];
        phi_1342_ = min(unpack4x8unorm(_e200).x, _e192);
    }
    let _e205 = phi_1342_;
    let _e207 = clamp(_e148, 0f, max(_e205, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e213 = u32(((abs(_e207) * 1024f) + 0.5f));
            let _e216 = atomicLoad((&Q0_.e2_[(_e154 + (((((_e155.y >> bitcast<u32>(5u)) * (_e152 << bitcast<u32>(5u))) + ((_e155.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e155.x & 28u) << bitcast<u32>(5u)) + ((_e155.y & 28u) << bitcast<u32>(2i)))) + (((_e155.y & 3u) << bitcast<u32>(2i)) + (_e155.x & 3u))))]));
            let _e218 = (min(_e147.w, _e207) >= 1f);
            phi_1066_ = _e218;
            if _e218 {
                let _e220 = n.d2_;
                let _e221 = (_e216 < _e220);
                phi_1064_ = _e221;
                if !(_e221) {
                    phi_1064_ = (_e216 >= (_e220 | 262144u));
                }
                let _e226 = phi_1064_;
                phi_1066_ = _e226;
            }
            let _e228 = phi_1066_;
            if _e228 {
                phi_1371_ = _e147.w;
                break;
            }
            let _e230 = n.d2_;
            phi_1361_ = 0f;
            phi_1358_ = _e213;
            phi_1355_ = _e207;
            if (_e216 < _e230) {
                let _e233 = (_e230 | (262144u + _e213));
                let _e234 = atomicMax((&Q0_.e2_[(_e154 + (((((_e155.y >> bitcast<u32>(5u)) * (_e152 << bitcast<u32>(5u))) + ((_e155.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e155.x & 28u) << bitcast<u32>(5u)) + ((_e155.y & 28u) << bitcast<u32>(2i)))) + (((_e155.y & 3u) << bitcast<u32>(2i)) + (_e155.x & 3u))))]), _e233);
                if (_e234 <= _e230) {
                    phi_1364_ = min(_e207, 1f);
                    phi_1359_ = _e213;
                    phi_1356_ = 0f;
                } else {
                    phi_1365_ = 0f;
                    phi_1360_ = _e213;
                    phi_1357_ = _e207;
                    if (_e234 < _e233) {
                        let _e238 = ((_e234 & 524287u) - 262144u);
                        let _e240 = (f32(_e238) * 0.0009765625f);
                        phi_1365_ = ((min(_e207, 1f) - _e240) / max((1f - (_e240 * _e147.w)), 0.000062f));
                        phi_1360_ = _e238;
                        phi_1357_ = _e240;
                    }
                    let _e248 = phi_1365_;
                    let _e250 = phi_1360_;
                    let _e252 = phi_1357_;
                    phi_1364_ = _e248;
                    phi_1359_ = _e250;
                    phi_1356_ = _e252;
                }
                let _e255 = phi_1364_;
                let _e257 = phi_1359_;
                let _e259 = phi_1356_;
                phi_1361_ = _e255;
                phi_1358_ = _e257;
                phi_1355_ = _e259;
            }
            let _e261 = phi_1361_;
            let _e263 = phi_1358_;
            let _e265 = phi_1355_;
            phi_1369_ = _e261;
            if (_e265 > 0f) {
                let _e267 = atomicAdd((&Q0_.e2_[(_e154 + (((((_e155.y >> bitcast<u32>(5u)) * (_e152 << bitcast<u32>(5u))) + ((_e155.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e155.x & 28u) << bitcast<u32>(5u)) + ((_e155.y & 28u) << bitcast<u32>(2i)))) + (((_e155.y & 3u) << bitcast<u32>(2i)) + (_e155.x & 3u))))]), _e263);
                let _e272 = (f32(bitcast<i32>(((_e267 & 524287u) - 262144u))) * 0.0009765625f);
                let _e274 = clamp(_e272, 0f, 1f);
                phi_1369_ = (_e261 + ((1f - (_e261 * _e147.w)) * ((clamp((_e272 + _e265), 0f, 1f) - _e274) / max((1f - (_e274 * _e147.w)), 0.000062f))));
            }
            let _e286 = phi_1369_;
            phi_1371_ = (_e147.w * _e286);
            break;
        }
    }
    let _e289 = phi_1371_;
    phi_1382_ = f32();
    if Ih {
        let _e291 = n.B3_;
        let _e293 = n.C3_;
        if Ih {
            phi_1379_ = ((fract((52.982918f * fract(((0.06711056f * _e53.x) + (0.00583715f * _e53.y))))) * _e291) + _e293);
        } else {
            phi_1379_ = 0f;
        }
        let _e305 = phi_1379_;
        phi_1382_ = _e305;
    }
    let _e307 = phi_1382_;
    let _e309 = (_e147.xyz * _e289);
    let _e313 = vec4<f32>(_e309.x, _e309.y, _e309.z, _e289);
    let _e314 = _e313.xyz;
    if (Ih && (_e289 != 0f)) {
        phi_1400_ = (vec3(_e307) + _e314);
    } else {
        phi_1400_ = _e314;
    }
    let _e320 = phi_1400_;
    let _e326 = vec4<f32>(_e320.x, _e313.y, _e313.z, _e313.w);
    let _e332 = vec4<f32>(_e326.x, _e320.y, _e326.z, _e326.w);
    h0_.e2_[_e88] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    C1_ = vec4<f32>(_e332.x, _e332.y, _e320.z, _e332.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) V1_: vec4<f32>, @location(9) C2_: vec3<f32>, @location(1) @interpolate(flat, either) h1_: f32, @location(8) p4_: vec2<f32>, @location(7) @interpolate(flat, either) g3_: vec2<u32>, @location(5) M0_: vec4<f32>, @location(4) @interpolate(flat, either) W1_: vec2<f32>, @location(3) @interpolate(flat, either) C0_: f32, @location(6) @interpolate(flat, either) g2_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    V1_1 = V1_;
    C2_1 = C2_;
    h1_1 = h1_;
    p4_1 = p4_;
    g3_1 = g3_;
    M0_1 = M0_;
    W1_1 = W1_;
    C0_1 = C0_;
    g2_1 = g2_;
    main_1();
    let _e21 = C1_;
    return _e21;
}
