struct VB {
    td: f32,
    ze: f32,
    Dg: f32,
    Eg: f32,
    L6_: u32,
    va: u32,
    pg: u32,
    qg: u32,
    B8_: vec4<i32>,
    xi: vec2<f32>,
    Ae: vec2<f32>,
    r2_: u32,
    Bi: f32,
    p6_: u32,
    h3_: f32,
    Be: f32,
    jg: u32,
    E3_: f32,
    F3_: f32,
    Ce: f32,
    ui: u32,
    ua: u32,
    ad: f32,
    g7_: f32,
    Bb: f32,
}

struct wf {
    v2_: array<u32>,
}

struct m0Me {
    v2_: array<u32>,
}

struct wf_1 {
    v2_: array<atomic<u32>>,
}

@id(7) override bj: bool = true;
@id(2) override Wi: bool = true;
@id(8) override cj: bool = true;
@id(1) override Vi: bool = true;
@id(0) override Ui: bool = true;

@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var H8_: sampler;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var S4_: sampler;
@group(0) @binding(6)
var<storage, read_write> Z0_: wf_1;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> V0_1: vec3<f32>;
var<private> P0_1: f32;
var<private> O0_1: vec4<f32>;
var<private> o1_1: f32;
var<private> J4_1: vec2<f32>;
var<private> y3_1: vec2<u32>;
var<private> W0_1: vec4<f32>;
var<private> j2_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Me;
var<private> N1_: vec4<f32>;
@group(3) @binding(9)
var Ta: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
var<private> G0_1: f32;

fn main_1() {
    var phi_1444_: f32;
    var phi_1445_: f32;
    var phi_1461_: vec4<f32>;
    var phi_1460_: vec4<f32>;
    var phi_952_: bool;
    var phi_967_: bool;
    var phi_1446_: f32;
    var phi_1456_: vec4<f32>;
    var phi_1463_: vec4<f32>;
    var phi_1464_: vec4<f32>;
    var phi_1466_: f32;
    var phi_729_: bool;
    var phi_1467_: f32;
    var phi_1183_: bool;
    var phi_1185_: bool;
    var phi_1488_: f32;
    var phi_1483_: u32;
    var phi_1480_: f32;
    var phi_1487_: f32;
    var phi_1482_: u32;
    var phi_1479_: f32;
    var phi_1484_: f32;
    var phi_1481_: u32;
    var phi_1478_: f32;
    var phi_1489_: f32;
    var phi_1490_: f32;
    var phi_1491_: f32;
    var phi_1501_: f32;
    var phi_1523_: vec3<f32>;

    let _e57 = gl_FragCoord_1;
    let _e61 = bitcast<vec2<u32>>(vec2<i32>(floor(_e57.xy)));
    let _e63 = j.L6_;
    let _e92 = bitcast<i32>((((((_e61.y >> bitcast<u32>(5u)) * (((_e63 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e61.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e61.x & 28u) << bitcast<u32>(5u)) + ((_e61.y & 28u) << bitcast<u32>(2i)))) + (((_e61.y & 3u) << bitcast<u32>(2i)) + (_e61.x & 3u))));
    let _e93 = P0_1;
    let _e95 = V0_1;
    let _e96 = O0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e99 = (Wi && (u32(_e93) != 0u));
            if (_e96.w >= 0f) {
                phi_1460_ = _e96;
            } else {
                let _e102 = -(_e96.w);
                let _e107 = j.ad;
                let _e110 = j.g7_;
                if (_e96.z > 0f) {
                    phi_1444_ = _e96.x;
                } else {
                    phi_1444_ = length(_e96.xy);
                }
                let _e118 = phi_1444_;
                let _e119 = clamp(_e118, 0f, 1f);
                let _e120 = abs(_e96.z);
                if (_e120 > 1f) {
                    phi_1445_ = ((0.9980469f * _e119) + 0.0009765625f);
                } else {
                    phi_1445_ = ((0.001953125f * _e119) + _e120);
                }
                let _e127 = phi_1445_;
                let _e129 = textureSampleLevel(YC, H8_, vec2<f32>(_e127, ((floor(_e102) * _e107) + _e110)), 0f);
                phi_1461_ = _e129;
                if !(_e99) {
                    let _e133 = (_e129.xyz * _e129.w);
                    phi_1461_ = vec4<f32>(_e133.x, _e133.y, _e133.z, (_e129.w * (fract(_e102) * 1.0039216f)));
                }
                let _e140 = phi_1461_;
                phi_1460_ = _e140;
            }
            let _e142 = phi_1460_;
            phi_952_ = cj;
            if cj {
                phi_952_ = (_e95.z < 0f);
            }
            let _e146 = phi_952_;
            if _e146 {
                let _e148 = textureSampleLevel(TB, S4_, _e95.xy, 0f);
                phi_1464_ = _e148;
                break;
            }
            phi_967_ = cj;
            if cj {
                phi_967_ = (_e95.z > 0f);
            }
            let _e152 = phi_967_;
            phi_1463_ = _e142;
            if _e152 {
                let _e156 = textureSampleLevel(TB, S4_, _e95.xy, (_e95.z - 1f));
                phi_1456_ = _e156;
                if _e99 {
                    if (_e156.w != 0f) {
                        phi_1446_ = (1f / _e156.w);
                    } else {
                        phi_1446_ = 0f;
                    }
                    let _e162 = phi_1446_;
                    let _e163 = (_e156.xyz * _e162);
                    phi_1456_ = vec4<f32>(_e163.x, _e163.y, _e163.z, _e156.w);
                }
                let _e169 = phi_1456_;
                phi_1463_ = (_e142 * _e169);
            }
            let _e172 = phi_1463_;
            phi_1464_ = _e172;
            break;
        }
    }
    let _e174 = phi_1464_;
    let _e175 = o1_1;
    let _e176 = J4_1;
    let _e179 = y3_1[1u];
    let _e181 = y3_1[0u];
    let _e182 = vec2<u32>(floor(_e176));
    phi_1466_ = 1f;
    if Vi {
        let _e210 = W0_1;
        let _e213 = min(_e210.xy, _e210.zw);
        phi_1466_ = min(min(_e213.x, _e213.y), 1f);
    }
    let _e219 = phi_1466_;
    phi_729_ = Ui;
    if Ui {
        let _e221 = j2_1[0u];
        phi_729_ = (_e221 != 0f);
    }
    let _e224 = phi_729_;
    phi_1467_ = _e219;
    if _e224 {
        let _e227 = m0_.v2_[_e92];
        phi_1467_ = min(unpack4x8unorm(_e227).x, _e219);
    }
    let _e232 = phi_1467_;
    let _e234 = clamp(_e175, 0f, max(_e232, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e240 = u32(((abs(_e234) * 1024f) + 0.5f));
            let _e243 = atomicLoad((&Z0_.v2_[(_e181 + (((((_e182.y >> bitcast<u32>(5u)) * (_e179 << bitcast<u32>(5u))) + ((_e182.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e182.x & 28u) << bitcast<u32>(5u)) + ((_e182.y & 28u) << bitcast<u32>(2i)))) + (((_e182.y & 3u) << bitcast<u32>(2i)) + (_e182.x & 3u))))]));
            let _e245 = (min(_e174.w, _e234) >= 1f);
            phi_1185_ = _e245;
            if _e245 {
                let _e247 = j.r2_;
                let _e248 = (_e243 < _e247);
                phi_1183_ = _e248;
                if !(_e248) {
                    phi_1183_ = (_e243 >= (_e247 | 262144u));
                }
                let _e253 = phi_1183_;
                phi_1185_ = _e253;
            }
            let _e255 = phi_1185_;
            if _e255 {
                phi_1490_ = 1f;
                break;
            }
            let _e257 = j.r2_;
            phi_1484_ = 0f;
            phi_1481_ = _e240;
            phi_1478_ = _e234;
            if (_e243 < _e257) {
                let _e260 = (_e257 | (262144u + _e240));
                let _e261 = atomicMax((&Z0_.v2_[(_e181 + (((((_e182.y >> bitcast<u32>(5u)) * (_e179 << bitcast<u32>(5u))) + ((_e182.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e182.x & 28u) << bitcast<u32>(5u)) + ((_e182.y & 28u) << bitcast<u32>(2i)))) + (((_e182.y & 3u) << bitcast<u32>(2i)) + (_e182.x & 3u))))]), _e260);
                if (_e261 <= _e257) {
                    phi_1487_ = min(_e234, 1f);
                    phi_1482_ = _e240;
                    phi_1479_ = 0f;
                } else {
                    phi_1488_ = 0f;
                    phi_1483_ = _e240;
                    phi_1480_ = _e234;
                    if (_e261 < _e260) {
                        let _e266 = ((_e261 & 524287u) - 262144u);
                        let _e268 = (f32(_e266) * 0.0009765625f);
                        phi_1488_ = ((min(_e234, 1f) - _e268) / max((1f - (_e268 * _e174.w)), 0.000062f));
                        phi_1483_ = _e266;
                        phi_1480_ = _e268;
                    }
                    let _e276 = phi_1488_;
                    let _e278 = phi_1483_;
                    let _e280 = phi_1480_;
                    phi_1487_ = _e276;
                    phi_1482_ = _e278;
                    phi_1479_ = _e280;
                }
                let _e282 = phi_1487_;
                let _e284 = phi_1482_;
                let _e286 = phi_1479_;
                phi_1484_ = _e282;
                phi_1481_ = _e284;
                phi_1478_ = _e286;
            }
            let _e288 = phi_1484_;
            let _e290 = phi_1481_;
            let _e292 = phi_1478_;
            phi_1489_ = _e288;
            if (_e292 > 0f) {
                let _e294 = atomicAdd((&Z0_.v2_[(_e181 + (((((_e182.y >> bitcast<u32>(5u)) * (_e179 << bitcast<u32>(5u))) + ((_e182.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e182.x & 28u) << bitcast<u32>(5u)) + ((_e182.y & 28u) << bitcast<u32>(2i)))) + (((_e182.y & 3u) << bitcast<u32>(2i)) + (_e182.x & 3u))))]), _e290);
                let _e299 = (f32(bitcast<i32>(((_e294 & 524287u) - 262144u))) * 0.0009765625f);
                let _e301 = clamp(_e299, 0f, 1f);
                phi_1489_ = (_e288 + ((1f - (_e288 * _e174.w)) * ((clamp((_e299 + _e292), 0f, 1f) - _e301) / max((1f - (_e301 * _e174.w)), 0.000062f))));
            }
            let _e313 = phi_1489_;
            phi_1490_ = _e313;
            break;
        }
    }
    let _e315 = phi_1490_;
    phi_1501_ = f32();
    if bj {
        let _e317 = j.E3_;
        let _e319 = j.F3_;
        if bj {
            phi_1491_ = ((fract((52.982918f * fract(((0.06711056f * _e57.x) + (0.00583715f * _e57.y))))) * _e317) + _e319);
        } else {
            phi_1491_ = 0f;
        }
        let _e331 = phi_1491_;
        phi_1501_ = _e331;
    }
    let _e333 = phi_1501_;
    let _e334 = (_e174 * _e315);
    let _e335 = _e334.xyz;
    if (bj && (_e334.w != 0f)) {
        phi_1523_ = (vec3(_e333) + _e335);
    } else {
        phi_1523_ = _e335;
    }
    let _e342 = phi_1523_;
    let _e348 = vec4<f32>(_e342.x, _e334.y, _e334.z, _e334.w);
    let _e354 = vec4<f32>(_e348.x, _e342.y, _e348.z, _e348.w);
    m0_.v2_[_e92] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    N1_ = vec4<f32>(_e354.x, _e354.y, _e342.z, _e354.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(9) V0_: vec3<f32>, @location(6) @interpolate(flat, either) P0_: f32, @location(0) O0_: vec4<f32>, @location(1) @interpolate(flat, either) o1_: f32, @location(8) J4_: vec2<f32>, @location(7) @interpolate(flat, either) y3_: vec2<u32>, @location(5) W0_: vec4<f32>, @location(4) @interpolate(flat, either) j2_: vec2<f32>, @location(3) @interpolate(flat, either) G0_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    V0_1 = V0_;
    P0_1 = P0_;
    O0_1 = O0_;
    o1_1 = o1_;
    J4_1 = J4_;
    y3_1 = y3_;
    W0_1 = W0_;
    j2_1 = j2_;
    G0_1 = G0_;
    main_1();
    let _e21 = N1_;
    return _e21;
}
