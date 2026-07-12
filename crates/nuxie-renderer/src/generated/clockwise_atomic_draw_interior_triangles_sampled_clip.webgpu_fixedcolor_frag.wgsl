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
var<private> i1_1: vec4<f32>;
var<private> j1_1: f32;
var<private> j4_1: vec2<f32>;
var<private> e3_1: vec2<u32>;
var<private> N0_1: vec4<f32>;
var<private> S1_1: vec2<f32>;
@group(2) @binding(1)
var d0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> l1_: vec4<f32>;
@group(3) @binding(10)
var T9_: sampler;
@group(0) @binding(10)
var QC: texture_2d<f32>;
var<private> z0_1: f32;
var<private> Z1_1: f32;

fn main_1() {
    var phi_1160_: f32;
    var phi_1164_: f32;
    var phi_1165_: f32;
    var phi_1167_: vec4<f32>;
    var phi_1166_: vec4<f32>;
    var phi_1169_: f32;
    var phi_598_: bool;
    var phi_1170_: f32;
    var phi_942_: bool;
    var phi_944_: bool;
    var phi_1193_: f32;
    var phi_1188_: u32;
    var phi_1185_: f32;
    var phi_1192_: f32;
    var phi_1187_: u32;
    var phi_1184_: f32;
    var phi_1189_: f32;
    var phi_1186_: u32;
    var phi_1183_: f32;
    var phi_1197_: f32;
    var phi_1199_: f32;
    var phi_1207_: f32;
    var phi_1210_: f32;
    var phi_1227_: vec4<f32>;

    let _e49 = i1_1;
    if (_e49.w >= 0f) {
        phi_1166_ = vec4<f32>(_e49.x, _e49.y, _e49.z, _e49.w);
    } else {
        if (_e49.w > -1f) {
            if (_e49.z > 0f) {
                phi_1164_ = _e49.x;
            } else {
                phi_1164_ = length(_e49.xy);
            }
            let _e75 = phi_1164_;
            let _e76 = clamp(_e75, 0f, 1f);
            let _e77 = abs(_e49.z);
            if (_e77 > 1f) {
                phi_1165_ = ((0.9980469f * _e76) + 0.0009765625f);
            } else {
                phi_1165_ = ((0.001953125f * _e76) + _e77);
            }
            let _e84 = phi_1165_;
            let _e87 = textureSampleLevel(DD, Bb, vec2<f32>(_e84, -(_e49.w)), 0f);
            phi_1167_ = vec4<f32>(_e87.x, _e87.y, _e87.z, _e87.w);
        } else {
            let _e55 = textureSampleLevel(AC, R5_, _e49.xy, (-2f - _e49.w));
            if (_e55.w != 0f) {
                phi_1160_ = (1f / _e55.w);
            } else {
                phi_1160_ = 0f;
            }
            let _e62 = phi_1160_;
            let _e63 = (_e55.xyz * _e62);
            phi_1167_ = vec4<f32>(_e63.x, _e63.y, _e63.z, (_e55.w * _e49.z));
        }
        let _e95 = phi_1167_;
        phi_1166_ = _e95;
    }
    let _e103 = phi_1166_;
    let _e104 = j1_1;
    let _e105 = j4_1;
    let _e108 = e3_1[1u];
    let _e110 = e3_1[0u];
    let _e111 = vec2<u32>(floor(_e105));
    phi_1169_ = 1f;
    if Ng {
        let _e139 = N0_1;
        let _e142 = min(_e139.xy, _e139.zw);
        phi_1169_ = min(min(_e142.x, _e142.y), 1f);
    }
    let _e148 = phi_1169_;
    phi_598_ = Mg;
    if Mg {
        let _e150 = S1_1[0u];
        phi_598_ = (_e150 != 0f);
    }
    let _e153 = phi_598_;
    phi_1170_ = _e148;
    if _e153 {
        let _e154 = gl_FragCoord_1;
        let _e158 = textureLoad(d0_, vec2<i32>(floor(_e154.xy)), 0i);
        phi_1170_ = min(_e158.x, _e148);
    }
    let _e162 = phi_1170_;
    let _e164 = clamp(_e104, 0f, max(_e162, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e170 = u32(((abs(_e164) * 1024f) + 0.5f));
            let _e173 = atomicLoad((&S0_.X1_[(_e110 + (((((_e111.y >> bitcast<u32>(5u)) * (_e108 << bitcast<u32>(5u))) + ((_e111.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e111.x & 28u) << bitcast<u32>(5u)) + ((_e111.y & 28u) << bitcast<u32>(2i)))) + (((_e111.y & 3u) << bitcast<u32>(2i)) + (_e111.x & 3u))))]));
            let _e175 = (min(_e103.w, _e164) >= 1f);
            phi_944_ = _e175;
            if _e175 {
                let _e177 = k.W1_;
                let _e178 = (_e173 < _e177);
                phi_942_ = _e178;
                if !(_e178) {
                    phi_942_ = (_e173 >= (_e177 | 262144u));
                }
                let _e183 = phi_942_;
                phi_944_ = _e183;
            }
            let _e185 = phi_944_;
            if _e185 {
                phi_1199_ = _e103.w;
                break;
            }
            let _e187 = k.W1_;
            phi_1189_ = 0f;
            phi_1186_ = _e170;
            phi_1183_ = _e164;
            if (_e173 < _e187) {
                let _e190 = (_e187 | (262144u + _e170));
                let _e191 = atomicMax((&S0_.X1_[(_e110 + (((((_e111.y >> bitcast<u32>(5u)) * (_e108 << bitcast<u32>(5u))) + ((_e111.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e111.x & 28u) << bitcast<u32>(5u)) + ((_e111.y & 28u) << bitcast<u32>(2i)))) + (((_e111.y & 3u) << bitcast<u32>(2i)) + (_e111.x & 3u))))]), _e190);
                if (_e191 <= _e187) {
                    phi_1192_ = min(_e164, 1f);
                    phi_1187_ = _e170;
                    phi_1184_ = 0f;
                } else {
                    phi_1193_ = 0f;
                    phi_1188_ = _e170;
                    phi_1185_ = _e164;
                    if (_e191 < _e190) {
                        let _e195 = ((_e191 & 524287u) - 262144u);
                        let _e197 = (f32(_e195) * 0.0009765625f);
                        phi_1193_ = ((min(_e164, 1f) - _e197) / max((1f - (_e197 * _e103.w)), 0.000062f));
                        phi_1188_ = _e195;
                        phi_1185_ = _e197;
                    }
                    let _e205 = phi_1193_;
                    let _e207 = phi_1188_;
                    let _e209 = phi_1185_;
                    phi_1192_ = _e205;
                    phi_1187_ = _e207;
                    phi_1184_ = _e209;
                }
                let _e212 = phi_1192_;
                let _e214 = phi_1187_;
                let _e216 = phi_1184_;
                phi_1189_ = _e212;
                phi_1186_ = _e214;
                phi_1183_ = _e216;
            }
            let _e218 = phi_1189_;
            let _e220 = phi_1186_;
            let _e222 = phi_1183_;
            phi_1197_ = _e218;
            if (_e222 > 0f) {
                let _e224 = atomicAdd((&S0_.X1_[(_e110 + (((((_e111.y >> bitcast<u32>(5u)) * (_e108 << bitcast<u32>(5u))) + ((_e111.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e111.x & 28u) << bitcast<u32>(5u)) + ((_e111.y & 28u) << bitcast<u32>(2i)))) + (((_e111.y & 3u) << bitcast<u32>(2i)) + (_e111.x & 3u))))]), _e220);
                let _e229 = (f32(bitcast<i32>(((_e224 & 524287u) - 262144u))) * 0.0009765625f);
                let _e231 = clamp(_e229, 0f, 1f);
                phi_1197_ = (_e218 + ((1f - (_e218 * _e103.w)) * ((clamp((_e229 + _e222), 0f, 1f) - _e231) / max((1f - (_e231 * _e103.w)), 0.000062f))));
            }
            let _e243 = phi_1197_;
            phi_1199_ = (_e103.w * _e243);
            break;
        }
    }
    let _e246 = phi_1199_;
    phi_1210_ = f32();
    if Tg {
        let _e247 = gl_FragCoord_1;
        let _e249 = k.y3_;
        let _e251 = k.z3_;
        if Tg {
            phi_1207_ = ((fract((52.982918f * fract(((0.06711056f * _e247.x) + (0.00583715f * _e247.y))))) * _e249) + _e251);
        } else {
            phi_1207_ = 0f;
        }
        let _e263 = phi_1207_;
        phi_1210_ = _e263;
    }
    let _e265 = phi_1210_;
    let _e267 = (_e103.xyz * _e246);
    let _e271 = vec4<f32>(_e267.x, _e267.y, _e267.z, _e246);
    phi_1227_ = _e271;
    if Tg {
        let _e274 = (_e271.xyz + vec3(_e265));
        let _e280 = vec4<f32>(_e274.x, _e271.y, _e271.z, _e271.w);
        let _e286 = vec4<f32>(_e280.x, _e274.y, _e280.z, _e280.w);
        phi_1227_ = vec4<f32>(_e286.x, _e286.y, _e274.z, _e286.w);
    }
    let _e294 = phi_1227_;
    l1_ = _e294;
    return;
}

@fragment
fn main(@location(0) i1_: vec4<f32>, @location(1) @interpolate(flat) j1_: f32, @location(8) j4_: vec2<f32>, @location(7) @interpolate(flat) e3_: vec2<u32>, @location(5) N0_: vec4<f32>, @location(4) @interpolate(flat) S1_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat) z0_: f32, @location(6) @interpolate(flat) Z1_: f32) -> @location(0) vec4<f32> {
    i1_1 = i1_;
    j1_1 = j1_;
    j4_1 = j4_;
    e3_1 = e3_;
    N0_1 = N0_;
    S1_1 = S1_;
    gl_FragCoord_1 = gl_FragCoord;
    z0_1 = z0_;
    Z1_1 = Z1_;
    main_1();
    let _e19 = l1_;
    return _e19;
}
