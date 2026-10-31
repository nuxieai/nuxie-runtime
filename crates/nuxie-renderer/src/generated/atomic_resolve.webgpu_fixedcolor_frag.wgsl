struct nd {
    z2_: array<vec4<u32>>,
}

struct n0hf {
    z2_: array<u32>,
}

struct Cg {
    z2_: array<vec4<f32>>,
}

struct RB {
    Kd: f32,
    Ue: f32,
    ah: f32,
    bh: f32,
    V5_: u32,
    Ia: u32,
    Lg: u32,
    Ng: u32,
    M8_: vec4<i32>,
    Ri: vec2<f32>,
    Ve: vec2<f32>,
    y2_: u32,
    Vi: f32,
    z6_: u32,
    n3_: f32,
    We: f32,
    Eg: u32,
    K3_: f32,
    L3_: f32,
    Xe: f32,
    Oi: u32,
    Ha: u32,
    T8_: f32,
    U8_: f32,
}

struct d5hf {
    z2_: array<u32>,
}

@id(8) override Cj: bool = true;
@id(9) override Dj: bool = true;
@id(5) override zj: bool = true;
@id(0) override uj: bool = true;
@id(1) override vj: bool = true;
@id(2) override wj: bool = true;

@group(0) @binding(2)
var<storage> JB: nd;
@group(2) @binding(1)
var<storage, read_write> n0_: n0hf;
@group(0) @binding(3)
var<storage> KB: Cg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: RB;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(3) @binding(7)
var V8_: sampler;
@group(2) @binding(3)
var<storage, read_write> d5_: d5hf;
var<private> S1_: vec4<f32>;
@group(3) @binding(8)
var ib: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(1) @binding(10)
var SB: texture_2d<f32>;
@group(1) @binding(12)
var x4_: sampler;

fn main_1() {
    var phi_848_: bool;
    var phi_1276_: f32;
    var phi_1275_: f32;
    var phi_1277_: f32;
    var phi_1280_: f32;
    var phi_1279_: f32;
    var phi_885_: bool;
    var phi_1295_: f32;
    var phi_1281_: f32;
    var phi_1282_: f32;
    var phi_1283_: vec2<f32>;
    var phi_1302_: vec4<f32>;
    var phi_1300_: vec4<f32>;
    var phi_1303_: vec3<f32>;

    let _e61 = gl_FragCoord_1;
    let _e62 = _e61.xy;
    let _e65 = bitcast<vec2<u32>>(vec2<i32>(floor(_e62)));
    let _e67 = j.V5_;
    let _e96 = bitcast<i32>((((((_e65.y >> bitcast<u32>(5u)) * (((_e67 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e65.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e65.x & 28u) << bitcast<u32>(5u)) + ((_e65.y & 28u) << bitcast<u32>(2i)))) + (((_e65.y & 3u) << bitcast<u32>(2i)) + (_e65.x & 3u))));
    let _e99 = d5_.z2_[_e96];
    let _e103 = ((f32((_e99 & 131071u)) * 0.00048828125f) + -32f);
    let _e105 = (_e99 >> bitcast<u32>(17u));
    let _e110 = JB.z2_[((_e105 * 4u) + 1u)];
    phi_1275_ = _e103;
    if ((_e110.z & 768u) != 0u) {
        let _e114 = abs(_e103);
        phi_848_ = zj;
        if zj {
            phi_848_ = ((_e110.z & 512u) != 0u);
        }
        let _e118 = phi_848_;
        phi_1276_ = _e114;
        if _e118 {
            phi_1276_ = (1f - abs(((fract((_e114 * 0.5f)) * 2f) + -1f)));
        }
        let _e126 = phi_1276_;
        phi_1275_ = _e126;
    }
    let _e128 = phi_1275_;
    let _e129 = clamp(_e128, 0f, 1f);
    phi_1279_ = _e129;
    if uj {
        let _e131 = (_e110.z >> bitcast<u32>(16u));
        phi_1280_ = _e129;
        if (_e131 != 0u) {
            let _e135 = n0_.z2_[_e96];
            if (_e131 == (_e135 >> bitcast<u32>(16i))) {
                phi_1277_ = min(_e129, unpack2x16float(_e135).x);
            } else {
                phi_1277_ = 0f;
            }
            let _e143 = phi_1277_;
            phi_1280_ = _e143;
        }
        let _e145 = phi_1280_;
        phi_1279_ = _e145;
    }
    let _e147 = phi_1279_;
    phi_885_ = vj;
    if vj {
        phi_885_ = ((_e110.z & 1024u) != 0u);
    }
    let _e151 = phi_885_;
    phi_1295_ = _e147;
    if _e151 {
        let _e152 = (_e105 * 8u);
        let _e156 = KB.z2_[(_e152 + 2u)];
        let _e167 = KB.z2_[(_e152 + 3u)];
        let _e172 = _e167.zw;
        let _e174 = ((abs(((mat2x2<f32>(vec2<f32>(_e156.x, _e156.y), vec2<f32>(_e156.z, _e156.w)) * _e62) + _e167.xy)) * _e172) - _e172);
        phi_1295_ = min(_e147, clamp((min(_e174.x, _e174.y) + 0.5f), 0f, 1f));
    }
    let _e182 = phi_1295_;
    let _e183 = (_e110.z & 15u);
    if (_e183 <= 1u) {
        phi_1300_ = select(unpack4x8unorm(_e110.w), vec4<f32>(0f, 0f, 0f, 0f), vec4((uj && (_e183 == 0u))));
    } else {
        let _e190 = (_e105 * 8u);
        let _e193 = KB.z2_[_e190];
        let _e204 = KB.z2_[(_e190 + 1u)];
        let _e207 = ((mat2x2<f32>(vec2<f32>(_e193.x, _e193.y), vec2<f32>(_e193.z, _e193.w)) * _e62) + _e204.xy);
        let _e211 = f32(_e183);
        let _e213 = j.T8_;
        let _e215 = j.U8_;
        let _e220 = ((_e204.w * _e213) + _e215);
        switch bitcast<i32>(0u) {
            default: {
                if (Dj && (_e211 == 4f)) {
                    let _e234 = normalize(_e207);
                    let _e237 = acos(clamp(_e234.x, -1f, 1f));
                    if (_e234.y >= 0f) {
                        phi_1282_ = _e237;
                    } else {
                        phi_1282_ = -(_e237);
                    }
                    let _e242 = phi_1282_;
                    phi_1283_ = vec2<f32>((_e242 * 0.15915494f), _e220);
                    break;
                } else {
                    if (_e211 == 2f) {
                        phi_1281_ = _e207.x;
                    } else {
                        phi_1281_ = length(_e207);
                    }
                    let _e228 = phi_1281_;
                    phi_1283_ = vec2<f32>(((clamp(_e228, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e204.z < 0f))) + ((max(0f, _e204.z) * 0.001953125f) + 0.0009765625f)), _e220);
                    break;
                }
            }
        }
        let _e246 = phi_1283_;
        let _e247 = textureSampleLevel(YC, V8_, _e246, 0f);
        phi_1302_ = _e247;
        if !((wj && (((_e110.z >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e251 = (_e247.xyz * _e247.w);
            phi_1302_ = vec4<f32>(_e251.x, _e251.y, _e251.z, (_e247.w * abs(bitcast<f32>(_e110.w))));
        }
        let _e261 = phi_1302_;
        phi_1300_ = _e261;
    }
    let _e269 = phi_1300_;
    let _e270 = (_e269 * _e182);
    let _e271 = _e270.xyz;
    let _e274 = j.K3_;
    let _e276 = j.L3_;
    if (Cj && (_e270.w != 0f)) {
        phi_1303_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e61.x) + (0.00583715f * _e61.y))))) * _e274) + _e276)) + _e271);
    } else {
        phi_1303_ = _e271;
    }
    let _e292 = phi_1303_;
    let _e298 = vec4<f32>(_e292.x, _e270.y, _e270.z, _e270.w);
    let _e304 = vec4<f32>(_e298.x, _e292.y, _e298.z, _e298.w);
    S1_ = vec4<f32>(_e304.x, _e304.y, _e292.z, _e304.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = S1_;
    return _e3;
}
