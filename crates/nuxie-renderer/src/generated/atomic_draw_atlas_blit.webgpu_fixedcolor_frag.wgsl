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
@group(0) @binding(10)
var ED: texture_2d<f32>;
@group(3) @binding(10)
var T9_: sampler;
var<private> F2_1: vec2<f32>;
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
    var phi_814_: bool;
    var phi_1169_: f32;
    var phi_1168_: f32;
    var phi_1170_: f32;
    var phi_1173_: f32;
    var phi_1172_: f32;
    var phi_851_: bool;
    var phi_1175_: f32;
    var phi_1201_: u32;
    var phi_1174_: f32;
    var phi_1199_: vec4<f32>;
    var phi_1200_: u32;
    var phi_1197_: vec4<f32>;
    var phi_1212_: vec3<f32>;

    let _e58 = gl_FragCoord_1;
    let _e59 = _e58.xy;
    let _e62 = bitcast<vec2<u32>>(vec2<i32>(floor(_e59)));
    let _e64 = j.q6_;
    let _e93 = bitcast<i32>((((((_e62.y >> bitcast<u32>(5u)) * (((_e64 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e62.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e62.x & 28u) << bitcast<u32>(5u)) + ((_e62.y & 28u) << bitcast<u32>(2i)))) + (((_e62.y & 3u) << bitcast<u32>(2i)) + (_e62.x & 3u))));
    let _e96 = z4_.g2_[_e93];
    let _e98 = (_e96 >> bitcast<u32>(17u));
    let _e99 = D0_1;
    let _e103 = F2_1;
    let _e104 = textureSampleLevel(ED, T9_, _e103, 0f);
    z4_.g2_[_e93] = (((_e99 << bitcast<u32>(17u)) + 65536u) + bitcast<u32>(i32(round((clamp(_e104.x, 0f, 1f) * 2048f)))));
    let _e115 = ((f32((_e96 & 131071u)) * 0.00048828125f) + -32f);
    let _e118 = CD.g2_[_e98];
    phi_1168_ = _e115;
    if ((_e118.x & 768u) != 0u) {
        let _e122 = abs(_e115);
        phi_814_ = Ih;
        if Ih {
            phi_814_ = ((_e118.x & 512u) != 0u);
        }
        let _e126 = phi_814_;
        phi_1169_ = _e122;
        if _e126 {
            phi_1169_ = (1f - abs(((fract((_e122 * 0.5f)) * 2f) + -1f)));
        }
        let _e134 = phi_1169_;
        phi_1168_ = _e134;
    }
    let _e136 = phi_1168_;
    let _e137 = clamp(_e136, 0f, 1f);
    phi_1172_ = _e137;
    if Eh {
        let _e139 = (_e118.x >> bitcast<u32>(16u));
        phi_1173_ = _e137;
        if (_e139 != 0u) {
            let _e143 = i0_.g2_[_e93];
            if (_e139 == (_e143 >> bitcast<u32>(16i))) {
                phi_1170_ = min(_e137, unpack2x16float(_e143).x);
            } else {
                phi_1170_ = 0f;
            }
            let _e151 = phi_1170_;
            phi_1173_ = _e151;
        }
        let _e153 = phi_1173_;
        phi_1172_ = _e153;
    }
    let _e155 = phi_1172_;
    phi_851_ = Fh;
    if Fh {
        phi_851_ = ((_e118.x & 1024u) != 0u);
    }
    let _e159 = phi_851_;
    phi_1175_ = _e155;
    if _e159 {
        let _e160 = (_e98 * 8u);
        let _e164 = PB.g2_[(_e160 + 2u)];
        let _e175 = PB.g2_[(_e160 + 3u)];
        let _e180 = _e175.zw;
        let _e182 = ((abs(((mat2x2<f32>(vec2<f32>(_e164.x, _e164.y), vec2<f32>(_e164.z, _e164.w)) * _e59) + _e175.xy)) * _e180) - _e180);
        phi_1175_ = min(_e155, clamp((min(_e182.x, _e182.y) + 0.5f), 0f, 1f));
    }
    let _e190 = phi_1175_;
    let _e191 = (_e118.x & 15u);
    if (_e191 <= 1u) {
        let _e201 = (Eh && (_e191 == 0u));
        phi_1201_ = 0u;
        if _e201 {
            phi_1201_ = (_e118.y | pack2x16float(vec2<f32>(_e190, 0f)));
        }
        let _e206 = phi_1201_;
        phi_1200_ = _e206;
        phi_1197_ = select(unpack4x8unorm(_e118.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e201));
    } else {
        let _e209 = (_e98 * 8u);
        let _e212 = PB.g2_[_e209];
        let _e223 = PB.g2_[(_e209 + 1u)];
        let _e226 = ((mat2x2<f32>(vec2<f32>(_e212.x, _e212.y), vec2<f32>(_e212.z, _e212.w)) * _e59) + _e223.xy);
        if (_e191 == 2u) {
            phi_1174_ = _e226.x;
        } else {
            phi_1174_ = length(_e226);
        }
        let _e231 = phi_1174_;
        let _e240 = textureSampleLevel(DD, P9_, vec2<f32>(((clamp(_e231, 0f, 1f) * _e223.z) + _e223.w), bitcast<f32>(_e118.y)), 0f);
        phi_1199_ = _e240;
        if !((Gh && (((_e118.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e244 = (_e240.xyz * _e240.w);
            let _e250 = vec4<f32>(_e244.x, _e240.y, _e240.z, _e240.w);
            let _e256 = vec4<f32>(_e250.x, _e244.y, _e250.z, _e250.w);
            phi_1199_ = vec4<f32>(_e256.x, _e256.y, _e244.z, _e256.w);
        }
        let _e264 = phi_1199_;
        phi_1200_ = 0u;
        phi_1197_ = _e264;
    }
    let _e266 = phi_1200_;
    let _e268 = phi_1197_;
    let _e269 = (_e268 * _e190);
    let _e270 = _e269.xyz;
    let _e273 = j.F3_;
    let _e275 = j.G3_;
    if (Lh && (_e269.w != 0f)) {
        phi_1212_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e58.x) + (0.00583715f * _e58.y))))) * _e273) + _e275)) + _e270);
    } else {
        phi_1212_ = _e270;
    }
    let _e291 = phi_1212_;
    let _e297 = vec4<f32>(_e291.x, _e269.y, _e269.z, _e269.w);
    let _e303 = vec4<f32>(_e297.x, _e291.y, _e297.z, _e297.w);
    E1_ = vec4<f32>(_e303.x, _e303.y, _e291.z, _e303.w);
    if (_e266 != 0u) {
        i0_.g2_[_e93] = _e266;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) D0_: u32, @location(0) F2_: vec2<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    D0_1 = D0_;
    F2_1 = F2_;
    main_1();
    let _e7 = E1_;
    return _e7;
}
