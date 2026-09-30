struct lf {
    g2_: array<vec2<u32>>,
}

struct i0Td {
    g2_: array<u32>,
}

struct mf {
    g2_: array<vec4<f32>>,
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

struct z4Td {
    g2_: array<u32>,
}

@id(7) override Lh: bool = true;
@id(4) override Ih: bool = true;
@id(0) override Eh: bool = true;
@id(1) override Fh: bool = true;
@id(2) override Gh: bool = true;

@group(0) @binding(3)
var<storage> CD: lf;
@group(2) @binding(1)
var<storage, read_write> i0_: i0Td;
@group(0) @binding(4)
var<storage> PB: mf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var P9_: sampler;
@group(0) @binding(0)
var<uniform> j: AC;
@group(2) @binding(3)
var<storage, read_write> z4_: z4Td;
var<private> D0_1: u32;
var<private> j1_1: f32;
var<private> E1_: vec4<f32>;
@group(3) @binding(9)
var ga: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var Y5_: sampler;

fn main_1() {
    var phi_1160_: u32;
    var phi_808_: bool;
    var phi_1165_: f32;
    var phi_1164_: f32;
    var phi_1166_: f32;
    var phi_1169_: f32;
    var phi_1168_: f32;
    var phi_845_: bool;
    var phi_1171_: f32;
    var phi_1199_: u32;
    var phi_1170_: f32;
    var phi_1197_: vec4<f32>;
    var phi_1198_: u32;
    var phi_1195_: vec4<f32>;
    var phi_1214_: u32;
    var phi_1210_: vec4<f32>;
    var phi_1211_: vec3<f32>;

    let _e56 = gl_FragCoord_1;
    let _e57 = _e56.xy;
    let _e60 = bitcast<vec2<u32>>(vec2<i32>(floor(_e57)));
    let _e62 = j.q6_;
    let _e91 = bitcast<i32>((((((_e60.y >> bitcast<u32>(5u)) * (((_e62 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e60.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e60.x & 28u) << bitcast<u32>(5u)) + ((_e60.y & 28u) << bitcast<u32>(2i)))) + (((_e60.y & 3u) << bitcast<u32>(2i)) + (_e60.x & 3u))));
    let _e94 = z4_.g2_[_e91];
    let _e96 = (_e94 >> bitcast<u32>(17u));
    let _e97 = D0_1;
    if (_e96 == _e97) {
        phi_1160_ = _e94;
    } else {
        phi_1160_ = ((_e97 << bitcast<u32>(17u)) + 65536u);
    }
    let _e103 = phi_1160_;
    let _e104 = j1_1;
    z4_.g2_[_e91] = (_e103 + bitcast<u32>(i32(round((_e104 * 2048f)))));
    phi_1214_ = 0u;
    phi_1210_ = vec4<f32>(0f, 0f, 0f, 0f);
    if (_e96 != _e97) {
        let _e114 = ((f32((_e94 & 131071u)) * 0.00048828125f) + -32f);
        let _e117 = CD.g2_[_e96];
        phi_1164_ = _e114;
        if ((_e117.x & 768u) != 0u) {
            let _e121 = abs(_e114);
            phi_808_ = Ih;
            if Ih {
                phi_808_ = ((_e117.x & 512u) != 0u);
            }
            let _e125 = phi_808_;
            phi_1165_ = _e121;
            if _e125 {
                phi_1165_ = (1f - abs(((fract((_e121 * 0.5f)) * 2f) + -1f)));
            }
            let _e133 = phi_1165_;
            phi_1164_ = _e133;
        }
        let _e135 = phi_1164_;
        let _e136 = clamp(_e135, 0f, 1f);
        phi_1168_ = _e136;
        if Eh {
            let _e138 = (_e117.x >> bitcast<u32>(16u));
            phi_1169_ = _e136;
            if (_e138 != 0u) {
                let _e142 = i0_.g2_[_e91];
                if (_e138 == (_e142 >> bitcast<u32>(16i))) {
                    phi_1166_ = min(_e136, unpack2x16float(_e142).x);
                } else {
                    phi_1166_ = 0f;
                }
                let _e150 = phi_1166_;
                phi_1169_ = _e150;
            }
            let _e152 = phi_1169_;
            phi_1168_ = _e152;
        }
        let _e154 = phi_1168_;
        phi_845_ = Fh;
        if Fh {
            phi_845_ = ((_e117.x & 1024u) != 0u);
        }
        let _e158 = phi_845_;
        phi_1171_ = _e154;
        if _e158 {
            let _e159 = (_e96 * 8u);
            let _e163 = PB.g2_[(_e159 + 2u)];
            let _e174 = PB.g2_[(_e159 + 3u)];
            let _e179 = _e174.zw;
            let _e181 = ((abs(((mat2x2<f32>(vec2<f32>(_e163.x, _e163.y), vec2<f32>(_e163.z, _e163.w)) * _e57) + _e174.xy)) * _e179) - _e179);
            phi_1171_ = min(_e154, clamp((min(_e181.x, _e181.y) + 0.5f), 0f, 1f));
        }
        let _e189 = phi_1171_;
        let _e190 = (_e117.x & 15u);
        if (_e190 <= 1u) {
            let _e200 = (Eh && (_e190 == 0u));
            phi_1199_ = 0u;
            if _e200 {
                phi_1199_ = (_e117.y | pack2x16float(vec2<f32>(_e189, 0f)));
            }
            let _e205 = phi_1199_;
            phi_1198_ = _e205;
            phi_1195_ = select(unpack4x8unorm(_e117.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e200));
        } else {
            let _e208 = (_e96 * 8u);
            let _e211 = PB.g2_[_e208];
            let _e222 = PB.g2_[(_e208 + 1u)];
            let _e225 = ((mat2x2<f32>(vec2<f32>(_e211.x, _e211.y), vec2<f32>(_e211.z, _e211.w)) * _e57) + _e222.xy);
            if (_e190 == 2u) {
                phi_1170_ = _e225.x;
            } else {
                phi_1170_ = length(_e225);
            }
            let _e230 = phi_1170_;
            let _e239 = textureSampleLevel(DD, P9_, vec2<f32>(((clamp(_e230, 0f, 1f) * _e222.z) + _e222.w), bitcast<f32>(_e117.y)), 0f);
            phi_1197_ = _e239;
            if !((Gh && (((_e117.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
                let _e243 = (_e239.xyz * _e239.w);
                let _e249 = vec4<f32>(_e243.x, _e239.y, _e239.z, _e239.w);
                let _e255 = vec4<f32>(_e249.x, _e243.y, _e249.z, _e249.w);
                phi_1197_ = vec4<f32>(_e255.x, _e255.y, _e243.z, _e255.w);
            }
            let _e263 = phi_1197_;
            phi_1198_ = 0u;
            phi_1195_ = _e263;
        }
        let _e265 = phi_1198_;
        let _e267 = phi_1195_;
        phi_1214_ = _e265;
        phi_1210_ = (_e267 * _e189);
    }
    let _e270 = phi_1214_;
    let _e272 = phi_1210_;
    let _e273 = _e272.xyz;
    let _e276 = j.F3_;
    let _e278 = j.G3_;
    if (Lh && (_e272.w != 0f)) {
        phi_1211_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e56.x) + (0.00583715f * _e56.y))))) * _e276) + _e278)) + _e273);
    } else {
        phi_1211_ = _e273;
    }
    let _e294 = phi_1211_;
    let _e300 = vec4<f32>(_e294.x, _e272.y, _e272.z, _e272.w);
    let _e306 = vec4<f32>(_e300.x, _e294.y, _e300.z, _e300.w);
    E1_ = vec4<f32>(_e306.x, _e306.y, _e294.z, _e306.w);
    if (_e270 != 0u) {
        i0_.g2_[_e91] = _e270;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) D0_: u32, @location(0) @interpolate(flat, either) j1_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    D0_1 = D0_;
    j1_1 = j1_;
    main_1();
    let _e7 = E1_;
    return _e7;
}
