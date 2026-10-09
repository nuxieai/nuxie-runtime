struct jg {
    r2_: array<vec2<u32>>,
}

struct m0Pe {
    r2_: array<u32>,
}

struct kg {
    r2_: array<vec4<f32>>,
}

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

struct R4Pe {
    r2_: array<u32>,
}

struct R4Pe_1 {
    r2_: array<atomic<u32>>,
}

@id(7) override dj: bool = true;
@id(4) override aj: bool = true;
@id(0) override Wi: bool = true;
@id(1) override Xi: bool = true;
@id(2) override Yi: bool = true;
@id(3) override Zi: bool = true;

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(3)
var<storage> VC: jg;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Pe;
@group(0) @binding(4)
var<storage> JB: kg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var N8_: sampler;
var<private> S_1: vec4<f32>;
var<private> G0_1: u32;
@group(2) @binding(3)
var<storage, read_write> R4_: R4Pe_1;
var<private> L1_: vec4<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;

fn main_1() {
    var phi_894_: bool;
    var phi_907_: bool;
    var phi_1845_: f32;
    var phi_1853_: f32;
    var phi_1861_: f32;
    var phi_1860_: f32;
    var phi_1397_: bool;
    var phi_1864_: f32;
    var phi_1863_: f32;
    var phi_1865_: f32;
    var phi_1868_: f32;
    var phi_1867_: f32;
    var phi_1434_: bool;
    var phi_1870_: f32;
    var phi_1908_: u32;
    var phi_1869_: f32;
    var phi_1906_: vec4<f32>;
    var phi_1907_: u32;
    var phi_1904_: vec4<f32>;
    var phi_1924_: u32;
    var phi_1919_: vec4<f32>;
    var phi_1921_: vec3<f32>;

    let _e79 = gl_FragCoord_1;
    let _e80 = _e79.xy;
    let _e83 = bitcast<vec2<u32>>(vec2<i32>(floor(_e80)));
    let _e85 = j.P6_;
    let _e114 = bitcast<i32>((((((_e83.y >> bitcast<u32>(5u)) * (((_e85 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e83.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e83.x & 28u) << bitcast<u32>(5u)) + ((_e83.y & 28u) << bitcast<u32>(2i)))) + (((_e83.y & 3u) << bitcast<u32>(2i)) + (_e83.x & 3u))));
    phi_894_ = Zi;
    if Zi {
        let _e115 = S_1;
        phi_894_ = (_e115.x < -1.5f);
    }
    let _e119 = phi_894_;
    if _e119 {
        let _e120 = S_1;
        let _e124 = textureSampleLevel(YC, ab, vec2<f32>((3f + _e120.x), 0f), 0f);
        let _e130 = textureSampleLevel(YC, ab, vec2<f32>((1f - _e120.y), 0f), 0f);
        phi_1860_ = ((1f - _e124.x) - _e130.x);
    } else {
        phi_907_ = Zi;
        if Zi {
            let _e133 = S_1;
            phi_907_ = (_e133.y < -1.5f);
        }
        let _e137 = phi_907_;
        if _e137 {
            let _e138 = S_1;
            let _e141 = max(_e138.w, 0f);
            if (_e138.z >= 0f) {
                let _e144 = textureSampleLevel(YC, ab, vec2<f32>(_e141, 0f), 0f);
                phi_1845_ = _e144.x;
            } else {
                phi_1845_ = 0f;
            }
            let _e147 = phi_1845_;
            phi_1853_ = _e147;
            if (abs(_e138.z) < 1000f) {
                let _e154 = (-2f - _e138.y);
                let _e156 = ((_e154 - _e141) * 0.5984134f);
                let _e159 = (vec4(_e141) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e156));
                let _e165 = ((_e159 * -(_e138.z)) + vec4(((_e154 * _e138.z) + (abs(_e138.x) - 0.25f))));
                let _e168 = textureSampleLevel(YC, ab, vec2<f32>(_e165.x, 0f), 0f);
                let _e171 = textureSampleLevel(YC, ab, vec2<f32>(_e165.y, 0f), 0f);
                let _e174 = textureSampleLevel(YC, ab, vec2<f32>(_e165.z, 0f), 0f);
                let _e177 = textureSampleLevel(YC, ab, vec2<f32>(_e165.w, 0f), 0f);
                let _e183 = (_e159 * 5.0959306f);
                phi_1853_ = (_e147 + (dot(vec4<f32>(_e168.x, _e171.x, _e174.x, _e177.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e183) * (_e183 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e156));
            }
            let _e192 = phi_1853_;
            phi_1861_ = (_e192 * sign(_e138.x));
        } else {
            let _e197 = S_1[0u];
            let _e199 = S_1[1u];
            phi_1861_ = min(min(_e197, abs(_e199)), 1f);
        }
        let _e204 = phi_1861_;
        phi_1860_ = _e204;
    }
    let _e206 = phi_1860_;
    let _e210 = u32(round(((_e206 * 2048f) + 65536f)));
    let _e211 = G0_1;
    let _e214 = ((_e211 << bitcast<u32>(17u)) | _e210);
    let _e217 = atomicMax((&R4_.r2_[_e114]), _e214);
    let _e219 = (_e217 >> bitcast<u32>(17u));
    if (_e219 == _e211) {
        let _e221 = S_1;
        if (_e221.y < 0f) {
            let _e228 = atomicAdd((&R4_.r2_[_e114]), ((_e210 + (_e217 - max(_e214, _e217))) - 65536u));
        }
        phi_1924_ = 0u;
        phi_1919_ = vec4<f32>(0f, 0f, 0f, 0f);
    } else {
        let _e232 = ((f32((_e217 & 131071u)) * 0.00048828125f) + -32f);
        let _e235 = VC.r2_[_e219];
        phi_1863_ = _e232;
        if ((_e235.x & 768u) != 0u) {
            let _e239 = abs(_e232);
            phi_1397_ = aj;
            if aj {
                phi_1397_ = ((_e235.x & 512u) != 0u);
            }
            let _e243 = phi_1397_;
            phi_1864_ = _e239;
            if _e243 {
                phi_1864_ = (1f - abs(((fract((_e239 * 0.5f)) * 2f) + -1f)));
            }
            let _e251 = phi_1864_;
            phi_1863_ = _e251;
        }
        let _e253 = phi_1863_;
        let _e254 = clamp(_e253, 0f, 1f);
        phi_1867_ = _e254;
        if Wi {
            let _e256 = (_e235.x >> bitcast<u32>(16u));
            phi_1868_ = _e254;
            if (_e256 != 0u) {
                let _e260 = m0_.r2_[_e114];
                if (_e256 == (_e260 >> bitcast<u32>(16i))) {
                    phi_1865_ = min(_e254, unpack2x16float(_e260).x);
                } else {
                    phi_1865_ = 0f;
                }
                let _e268 = phi_1865_;
                phi_1868_ = _e268;
            }
            let _e270 = phi_1868_;
            phi_1867_ = _e270;
        }
        let _e272 = phi_1867_;
        phi_1434_ = Xi;
        if Xi {
            phi_1434_ = ((_e235.x & 1024u) != 0u);
        }
        let _e276 = phi_1434_;
        phi_1870_ = _e272;
        if _e276 {
            let _e277 = (_e219 * 8u);
            let _e281 = JB.r2_[(_e277 + 2u)];
            let _e292 = JB.r2_[(_e277 + 3u)];
            let _e297 = _e292.zw;
            let _e299 = ((abs(((mat2x2<f32>(vec2<f32>(_e281.x, _e281.y), vec2<f32>(_e281.z, _e281.w)) * _e80) + _e292.xy)) * _e297) - _e297);
            phi_1870_ = min(_e272, clamp((min(_e299.x, _e299.y) + 0.5f), 0f, 1f));
        }
        let _e307 = phi_1870_;
        let _e308 = (_e235.x & 15u);
        if (_e308 <= 1u) {
            let _e318 = (Wi && (_e308 == 0u));
            phi_1908_ = 0u;
            if _e318 {
                phi_1908_ = (_e235.y | pack2x16float(vec2<f32>(_e307, 0f)));
            }
            let _e323 = phi_1908_;
            phi_1907_ = _e323;
            phi_1904_ = select(unpack4x8unorm(_e235.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e318));
        } else {
            let _e326 = (_e219 * 8u);
            let _e329 = JB.r2_[_e326];
            let _e340 = JB.r2_[(_e326 + 1u)];
            let _e343 = ((mat2x2<f32>(vec2<f32>(_e329.x, _e329.y), vec2<f32>(_e329.z, _e329.w)) * _e80) + _e340.xy);
            let _e349 = j.L8_;
            let _e351 = j.M8_;
            if (f32(_e308) == 2f) {
                phi_1869_ = _e343.x;
            } else {
                phi_1869_ = length(_e343);
            }
            let _e361 = phi_1869_;
            let _e367 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e361, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e340.z < 0f))) + ((max(0f, _e340.z) * 0.001953125f) + 0.0009765625f)), ((_e340.w * _e349) + _e351)), 0f);
            phi_1906_ = _e367;
            if !((Yi && (((_e235.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
                let _e371 = (_e367.xyz * _e367.w);
                phi_1906_ = vec4<f32>(_e371.x, _e371.y, _e371.z, (_e367.w * abs(bitcast<f32>(_e235.y))));
            }
            let _e381 = phi_1906_;
            phi_1907_ = 0u;
            phi_1904_ = _e381;
        }
        let _e383 = phi_1907_;
        let _e385 = phi_1904_;
        phi_1924_ = _e383;
        phi_1919_ = (_e385 * _e307);
    }
    let _e388 = phi_1924_;
    let _e390 = phi_1919_;
    let _e391 = _e390.xyz;
    let _e394 = j.F3_;
    let _e396 = j.G3_;
    if (dj && (_e390.w != 0f)) {
        phi_1921_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e79.x) + (0.00583715f * _e79.y))))) * _e394) + _e396)) + _e391);
    } else {
        phi_1921_ = _e391;
    }
    let _e412 = phi_1921_;
    let _e418 = vec4<f32>(_e412.x, _e390.y, _e390.z, _e390.w);
    let _e424 = vec4<f32>(_e418.x, _e412.y, _e418.z, _e418.w);
    L1_ = vec4<f32>(_e424.x, _e424.y, _e412.z, _e424.w);
    if (_e388 != 0u) {
        m0_.r2_[_e114] = _e388;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) S: vec4<f32>, @location(1) @interpolate(flat, either) G0_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    S_1 = S;
    G0_1 = G0_;
    main_1();
    let _e7 = L1_;
    return _e7;
}
