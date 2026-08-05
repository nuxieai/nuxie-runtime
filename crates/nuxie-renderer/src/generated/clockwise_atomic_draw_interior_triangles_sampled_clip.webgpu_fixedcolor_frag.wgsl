struct ge {
    c2_: array<u32>,
}

struct CC {
    ec: f32,
    od: f32,
    ff: f32,
    gf: f32,
    m6_: u32,
    Fg: u32,
    Re: u32,
    Se: u32,
    R7_: vec4<i32>,
    Bg: vec2<f32>,
    pd: vec2<f32>,
    a2_: u32,
    Gg: f32,
    Z5_: u32,
    P2_: f32,
    qd: f32,
    Me: u32,
    z3_: f32,
    A3_: f32,
    rd: f32,
    yg: u32,
}

struct ge_1 {
    c2_: array<atomic<u32>>,
}

@id(7) override fh: bool = true;
@id(1) override Zg: bool = true;
@id(0) override Yg: bool = true;

@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(3) @binding(8)
var Mb: sampler;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;
@group(0) @binding(6)
var<storage, read_write> P0_: ge_1;
@group(0) @binding(0)
var<uniform> n: CC;
var<private> f1_1: vec4<f32>;
var<private> i1_1: f32;
var<private> l4_1: vec2<f32>;
var<private> d3_1: vec2<u32>;
var<private> L0_1: vec4<f32>;
var<private> U1_1: vec2<f32>;
@group(2) @binding(1)
var h0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> B0_1: f32;
var<private> e2_1: f32;

fn main_1() {
    var phi_1199_: f32;
    var phi_1203_: f32;
    var phi_1204_: f32;
    var phi_1206_: vec4<f32>;
    var phi_1205_: vec4<f32>;
    var phi_1208_: f32;
    var phi_619_: bool;
    var phi_1209_: f32;
    var phi_965_: bool;
    var phi_967_: bool;
    var phi_1232_: f32;
    var phi_1227_: u32;
    var phi_1224_: f32;
    var phi_1231_: f32;
    var phi_1226_: u32;
    var phi_1223_: f32;
    var phi_1228_: f32;
    var phi_1225_: u32;
    var phi_1222_: f32;
    var phi_1236_: f32;
    var phi_1238_: f32;
    var phi_1246_: f32;
    var phi_1249_: f32;
    var phi_1266_: vec3<f32>;

    let _e49 = f1_1;
    if (_e49.w >= 0f) {
        phi_1205_ = vec4<f32>(_e49.x, _e49.y, _e49.z, _e49.w);
    } else {
        if (_e49.w > -1f) {
            if (_e49.z > 0f) {
                phi_1203_ = _e49.x;
            } else {
                phi_1203_ = length(_e49.xy);
            }
            let _e75 = phi_1203_;
            let _e76 = clamp(_e75, 0f, 1f);
            let _e77 = abs(_e49.z);
            if (_e77 > 1f) {
                phi_1204_ = ((0.9980469f * _e76) + 0.0009765625f);
            } else {
                phi_1204_ = ((0.001953125f * _e76) + _e77);
            }
            let _e84 = phi_1204_;
            let _e87 = textureSampleLevel(KD, Mb, vec2<f32>(_e84, -(_e49.w)), 0f);
            phi_1206_ = vec4<f32>(_e87.x, _e87.y, _e87.z, _e87.w);
        } else {
            let _e55 = textureSampleLevel(IC, S5_, _e49.xy, (-2f - _e49.w));
            if (_e55.w != 0f) {
                phi_1199_ = (1f / _e55.w);
            } else {
                phi_1199_ = 0f;
            }
            let _e62 = phi_1199_;
            let _e63 = (_e55.xyz * _e62);
            phi_1206_ = vec4<f32>(_e63.x, _e63.y, _e63.z, (_e55.w * _e49.z));
        }
        let _e95 = phi_1206_;
        phi_1205_ = _e95;
    }
    let _e103 = phi_1205_;
    let _e104 = i1_1;
    let _e105 = l4_1;
    let _e108 = d3_1[1u];
    let _e110 = d3_1[0u];
    let _e111 = vec2<u32>(floor(_e105));
    phi_1208_ = 1f;
    if Zg {
        let _e139 = L0_1;
        let _e142 = min(_e139.xy, _e139.zw);
        phi_1208_ = min(min(_e142.x, _e142.y), 1f);
    }
    let _e148 = phi_1208_;
    phi_619_ = Yg;
    if Yg {
        let _e150 = U1_1[0u];
        phi_619_ = (_e150 != 0f);
    }
    let _e153 = phi_619_;
    phi_1209_ = _e148;
    if _e153 {
        let _e154 = gl_FragCoord_1;
        let _e158 = textureLoad(h0_, vec2<i32>(floor(_e154.xy)), 0i);
        phi_1209_ = min(_e158.x, _e148);
    }
    let _e162 = phi_1209_;
    let _e164 = clamp(_e104, 0f, max(_e162, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e170 = u32(((abs(_e164) * 1024f) + 0.5f));
            let _e173 = atomicLoad((&P0_.c2_[(_e110 + (((((_e111.y >> bitcast<u32>(5u)) * (_e108 << bitcast<u32>(5u))) + ((_e111.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e111.x & 28u) << bitcast<u32>(5u)) + ((_e111.y & 28u) << bitcast<u32>(2i)))) + (((_e111.y & 3u) << bitcast<u32>(2i)) + (_e111.x & 3u))))]));
            let _e175 = (min(_e103.w, _e164) >= 1f);
            phi_967_ = _e175;
            if _e175 {
                let _e177 = n.a2_;
                let _e178 = (_e173 < _e177);
                phi_965_ = _e178;
                if !(_e178) {
                    phi_965_ = (_e173 >= (_e177 | 262144u));
                }
                let _e183 = phi_965_;
                phi_967_ = _e183;
            }
            let _e185 = phi_967_;
            if _e185 {
                phi_1238_ = _e103.w;
                break;
            }
            let _e187 = n.a2_;
            phi_1228_ = 0f;
            phi_1225_ = _e170;
            phi_1222_ = _e164;
            if (_e173 < _e187) {
                let _e190 = (_e187 | (262144u + _e170));
                let _e191 = atomicMax((&P0_.c2_[(_e110 + (((((_e111.y >> bitcast<u32>(5u)) * (_e108 << bitcast<u32>(5u))) + ((_e111.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e111.x & 28u) << bitcast<u32>(5u)) + ((_e111.y & 28u) << bitcast<u32>(2i)))) + (((_e111.y & 3u) << bitcast<u32>(2i)) + (_e111.x & 3u))))]), _e190);
                if (_e191 <= _e187) {
                    phi_1231_ = min(_e164, 1f);
                    phi_1226_ = _e170;
                    phi_1223_ = 0f;
                } else {
                    phi_1232_ = 0f;
                    phi_1227_ = _e170;
                    phi_1224_ = _e164;
                    if (_e191 < _e190) {
                        let _e195 = ((_e191 & 524287u) - 262144u);
                        let _e197 = (f32(_e195) * 0.0009765625f);
                        phi_1232_ = ((min(_e164, 1f) - _e197) / max((1f - (_e197 * _e103.w)), 0.000062f));
                        phi_1227_ = _e195;
                        phi_1224_ = _e197;
                    }
                    let _e205 = phi_1232_;
                    let _e207 = phi_1227_;
                    let _e209 = phi_1224_;
                    phi_1231_ = _e205;
                    phi_1226_ = _e207;
                    phi_1223_ = _e209;
                }
                let _e212 = phi_1231_;
                let _e214 = phi_1226_;
                let _e216 = phi_1223_;
                phi_1228_ = _e212;
                phi_1225_ = _e214;
                phi_1222_ = _e216;
            }
            let _e218 = phi_1228_;
            let _e220 = phi_1225_;
            let _e222 = phi_1222_;
            phi_1236_ = _e218;
            if (_e222 > 0f) {
                let _e224 = atomicAdd((&P0_.c2_[(_e110 + (((((_e111.y >> bitcast<u32>(5u)) * (_e108 << bitcast<u32>(5u))) + ((_e111.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e111.x & 28u) << bitcast<u32>(5u)) + ((_e111.y & 28u) << bitcast<u32>(2i)))) + (((_e111.y & 3u) << bitcast<u32>(2i)) + (_e111.x & 3u))))]), _e220);
                let _e229 = (f32(bitcast<i32>(((_e224 & 524287u) - 262144u))) * 0.0009765625f);
                let _e231 = clamp(_e229, 0f, 1f);
                phi_1236_ = (_e218 + ((1f - (_e218 * _e103.w)) * ((clamp((_e229 + _e222), 0f, 1f) - _e231) / max((1f - (_e231 * _e103.w)), 0.000062f))));
            }
            let _e243 = phi_1236_;
            phi_1238_ = (_e103.w * _e243);
            break;
        }
    }
    let _e246 = phi_1238_;
    phi_1249_ = f32();
    if fh {
        let _e247 = gl_FragCoord_1;
        let _e249 = n.z3_;
        let _e251 = n.A3_;
        if fh {
            phi_1246_ = ((fract((52.982918f * fract(((0.06711056f * _e247.x) + (0.00583715f * _e247.y))))) * _e249) + _e251);
        } else {
            phi_1246_ = 0f;
        }
        let _e263 = phi_1246_;
        phi_1249_ = _e263;
    }
    let _e265 = phi_1249_;
    let _e267 = (_e103.xyz * _e246);
    let _e271 = vec4<f32>(_e267.x, _e267.y, _e267.z, _e246);
    let _e272 = _e271.xyz;
    if (fh && (_e246 != 0f)) {
        phi_1266_ = (vec3(_e265) + _e272);
    } else {
        phi_1266_ = _e272;
    }
    let _e278 = phi_1266_;
    let _e284 = vec4<f32>(_e278.x, _e271.y, _e271.z, _e271.w);
    let _e290 = vec4<f32>(_e284.x, _e278.y, _e284.z, _e284.w);
    C1_ = vec4<f32>(_e290.x, _e290.y, _e278.z, _e290.w);
    return;
}

@fragment
fn main(@location(0) f1_: vec4<f32>, @location(1) @interpolate(flat, either) i1_: f32, @location(8) l4_: vec2<f32>, @location(7) @interpolate(flat, either) d3_: vec2<u32>, @location(5) L0_: vec4<f32>, @location(4) @interpolate(flat, either) U1_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) B0_: f32, @location(6) @interpolate(flat, either) e2_: f32) -> @location(0) vec4<f32> {
    f1_1 = f1_;
    i1_1 = i1_;
    l4_1 = l4_;
    d3_1 = d3_;
    L0_1 = L0_;
    U1_1 = U1_;
    gl_FragCoord_1 = gl_FragCoord;
    B0_1 = B0_;
    e2_1 = e2_;
    main_1();
    let _e19 = C1_;
    return _e19;
}
