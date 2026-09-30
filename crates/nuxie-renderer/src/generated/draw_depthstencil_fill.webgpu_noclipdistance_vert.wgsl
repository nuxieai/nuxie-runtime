struct oh {
    k2_: array<vec4<u32>>,
}

struct nh {
    k2_: array<vec4<u32>>,
}

struct Ff {
    k2_: array<vec2<u32>>,
}

struct UB {
    Rc: f32,
    Ud: f32,
    cg: f32,
    dg: f32,
    B6_: u32,
    Y9_: u32,
    Of: u32,
    Pf: u32,
    k8_: vec4<i32>,
    Mh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Qh: f32,
    U4_: u32,
    c3_: f32,
    Wd: f32,
    If: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Jh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

struct Gf {
    k2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct VertexOutput {
    @location(4) @interpolate(flat, either) member: vec2<f32>,
    @location(6) @interpolate(flat, either) member_1: f32,
    @location(0) member_2: vec4<f32>,
    @location(9) member_3: vec3<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override li: bool = true;
@id(2) override ni: bool = true;
@id(8) override ti: bool = true;

var<private> gl_VertexIndex_1: i32;
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(5)
var<storage> AD: oh;
@group(0) @binding(2)
var<storage> LB: nh;
@group(0) @binding(3)
var<storage> XC: Ff;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> l1_: vec2<f32>;
var<private> Q0_: f32;
@group(0) @binding(4)
var<storage> JB: Gf;
var<private> a1_: vec4<f32>;
var<private> r1_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var xa: sampler;

fn main_1() {
    var phi_874_: i32;
    var phi_876_: vec4<u32>;
    var phi_877_: vec4<u32>;
    var phi_879_: vec2<f32>;
    var phi_880_: u32;
    var phi_881_: f32;
    var phi_882_: f32;
    var phi_895_: f32;
    var phi_893_: vec4<f32>;
    var phi_894_: vec4<f32>;
    var phi_560_: bool;

    let _e56 = gl_VertexIndex_1;
    let _e58 = ((_e56 & 1073741824i) != 0i);
    let _e61 = (_e56 & 536870911i);
    let _e62 = select(4i, 5i, _e58);
    let _e68 = (_e61 & ((1i << bitcast<u32>(_e62)) - 1i));
    let _e69 = select(8i, 17i, _e58);
    let _e70 = !(_e58);
    let _e72 = (_e70 && (_e68 == 9i));
    let _e73 = select(_e68, 0i, _e72);
    let _e75 = min(_e73, (_e69 - 1i));
    let _e77 = (((_e61 >> bitcast<u32>(_e62)) * _e69) + _e75);
    let _e82 = textureLoad(TB, vec2<i32>((_e77 & 2047i), (_e77 >> bitcast<u32>(11i))), 0i);
    let _e89 = AD.k2_[(max((_e82.w & 65535u), 1u) - 1u)];
    let _e93 = (_e89.z & 65535u);
    let _e95 = (_e93 * 4u);
    let _e98 = LB.k2_[_e95];
    let _e99 = bitcast<vec4<f32>>(_e98);
    let _e110 = LB.k2_[(_e95 + 1u)];
    phi_874_ = _e73;
    if ((((_e82.w & 8388608u) != 0u) && _e70) && !(_e72)) {
        phi_874_ = (_e73 - 1i);
    }
    let _e120 = phi_874_;
    phi_877_ = _e82;
    if (_e120 != _e75) {
        let _e123 = ((_e77 + _e120) - _e75);
        let _e128 = textureLoad(TB, vec2<i32>((_e123 & 2047i), (_e123 >> bitcast<u32>(11i))), 0i);
        if ((_e128.w & 8454143u) != (_e82.w & 8454143u)) {
            let _e133 = bitcast<i32>(_e89.w);
            let _e138 = textureLoad(TB, vec2<i32>((_e133 & 2047i), (_e133 >> bitcast<u32>(11i))), 0i);
            phi_876_ = _e138;
        } else {
            phi_876_ = _e128;
        }
        let _e140 = phi_876_;
        phi_877_ = _e140;
    }
    let _e142 = phi_877_;
    if _e72 {
        phi_879_ = bitcast<vec2<f32>>(_e89.xy);
    } else {
        phi_879_ = bitcast<vec2<f32>>(_e142.xy);
    }
    let _e146 = phi_879_;
    let _e148 = ((mat2x2<f32>(vec2<f32>(_e99.x, _e99.y), vec2<f32>(_e99.z, _e99.w)) * _e146) + bitcast<vec2<f32>>(_e110.xy));
    let _e151 = XC.k2_[_e93];
    let _e153 = (_e151.x & 15u);
    if li {
        let _e154 = (_e153 == 0u);
        if _e154 {
            phi_880_ = _e151.y;
        } else {
            phi_880_ = _e151.x;
        }
        let _e157 = phi_880_;
        let _e159 = (_e157 >> bitcast<u32>(16i));
        let _e161 = j.U4_;
        if (_e159 == 0u) {
            phi_881_ = 0f;
        } else {
            phi_881_ = unpack2x16float(((_e159 + 1023u) * _e161)).x;
        }
        let _e168 = phi_881_;
        phi_882_ = _e168;
        if _e154 {
            phi_882_ = -(_e168);
        }
        let _e171 = phi_882_;
        l1_[0u] = _e171;
    }
    if ni {
        Q0_ = f32(((_e151.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e153 == 1u) {
        a1_ = unpack4x8unorm(_e151.y);
    } else {
        if (li && (_e153 == 0u)) {
            let _e183 = (_e151.x >> bitcast<u32>(16i));
            let _e185 = j.U4_;
            if (_e183 == 0u) {
                phi_895_ = 0f;
            } else {
                phi_895_ = unpack2x16float(((_e183 + 1023u) * _e185)).x;
            }
            let _e192 = phi_895_;
            l1_[1u] = _e192;
        } else {
            let _e194 = (_e93 * 8u);
            let _e197 = JB.k2_[_e194];
            let _e208 = JB.k2_[(_e194 + 1u)];
            let _e217 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e151.y));
            let _e219 = ((mat2x2<f32>(vec2<f32>(_e197.x, _e197.y), vec2<f32>(_e197.z, _e197.w)) * _e148) + _e208.xy);
            if (_e208.z > 0.9f) {
                phi_893_ = vec4<f32>(_e217.x, _e217.y, 2f, _e217.w);
            } else {
                phi_893_ = vec4<f32>(_e217.x, _e217.y, _e208.w, _e217.w);
            }
            let _e234 = phi_893_;
            if (f32(_e153) == 2f) {
                let _e241 = vec4<f32>(_e219.x, _e234.y, _e234.z, _e234.w);
                phi_894_ = vec4<f32>(_e241.x, 0f, _e241.z, _e241.w);
            } else {
                let _e253 = vec4<f32>(_e234.x, _e234.y, -(_e234.z), _e234.w);
                let _e259 = vec4<f32>(_e219.x, _e253.y, _e253.z, _e253.w);
                phi_894_ = vec4<f32>(_e259.x, _e219.y, _e259.z, _e259.w);
            }
            let _e267 = phi_894_;
            a1_ = _e267;
            let _e269 = a1_[3u];
            a1_[3u] = -(_e269);
        }
    }
    if ((_e56 & 536870912i) != 0i) {
        a1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    phi_560_ = ti;
    if ti {
        phi_560_ = ((_e151.x & 2048u) != 0u);
    }
    let _e274 = phi_560_;
    if _e274 {
        let _e275 = (_e93 * 8u);
        let _e279 = JB.k2_[(_e275 + 4u)];
        let _e290 = JB.k2_[(_e275 + 5u)];
        let _e293 = ((mat2x2<f32>(vec2<f32>(_e279.x, _e279.y), vec2<f32>(_e279.z, _e279.w)) * _e148) + _e290.xy);
        r1_ = vec3<f32>(_e293.x, _e293.y, (1f + _e290.z));
    } else {
        r1_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e300 = j.cg;
    let _e302 = j.dg;
    let _e310 = vec4<f32>(((_e148.x * _e300) - 1f), ((_e148.y * _e302) - sign(_e302)), 0f, 1f);
    let _e314 = LB.k2_[(_e95 + 2u)];
    unnamed.gl_Position = vec4<f32>(_e310.x, _e310.y, ((f32(((_e314.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e310.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e9 = l1_;
    let _e10 = Q0_;
    let _e11 = a1_;
    let _e12 = r1_;
    let _e13 = unnamed.gl_Position;
    return VertexOutput(_e9, _e10, _e11, _e12, _e13);
}
