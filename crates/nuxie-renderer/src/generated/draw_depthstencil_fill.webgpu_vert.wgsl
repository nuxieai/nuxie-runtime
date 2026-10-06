enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct ph {
    k2_: array<vec4<u32>>,
}

struct oh {
    k2_: array<vec4<u32>>,
}

struct Gf {
    k2_: array<vec2<u32>>,
}

struct UB {
    Rc: f32,
    Ud: f32,
    dg: f32,
    eg: f32,
    A6_: u32,
    Y9_: u32,
    Pf: u32,
    Qf: u32,
    i8_: vec4<i32>,
    Nh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Rh: f32,
    U4_: u32,
    a3_: f32,
    Wd: f32,
    Jf: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Kh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

struct Hf {
    k2_: array<vec4<f32>>,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(4) @interpolate(flat, either) member: vec2<f32>,
    @location(6) @interpolate(flat, either) member_1: f32,
    @location(0) member_2: vec4<f32>,
    @location(9) member_3: vec3<f32>,
}

@id(0) override mi: bool = true;
@id(2) override oi: bool = true;
@id(1) override ni: bool = true;
@id(8) override ui: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
var<private> gl_VertexIndex_1: i32;
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(5)
var<storage> ZC: ph;
@group(0) @binding(2)
var<storage> LB: oh;
@group(0) @binding(3)
var<storage> WC: Gf;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> l1_: vec2<f32>;
var<private> Q0_: f32;
@group(0) @binding(4)
var<storage> JB: Hf;
var<private> a1_: vec4<f32>;
var<private> v1_: vec3<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var xa: sampler;

fn main_1() {
    var phi_962_: i32;
    var phi_964_: vec4<u32>;
    var phi_965_: vec4<u32>;
    var phi_967_: vec2<f32>;
    var phi_968_: u32;
    var phi_969_: f32;
    var phi_970_: f32;
    var phi_985_: f32;
    var phi_983_: vec4<f32>;
    var phi_984_: vec4<f32>;
    var phi_609_: bool;

    let _e58 = gl_VertexIndex_1;
    let _e60 = ((_e58 & 1073741824i) != 0i);
    let _e63 = (_e58 & 536870911i);
    let _e64 = select(4i, 5i, _e60);
    let _e70 = (_e63 & ((1i << bitcast<u32>(_e64)) - 1i));
    let _e71 = select(8i, 17i, _e60);
    let _e72 = !(_e60);
    let _e74 = (_e72 && (_e70 == 9i));
    let _e75 = select(_e70, 0i, _e74);
    let _e77 = min(_e75, (_e71 - 1i));
    let _e79 = (((_e63 >> bitcast<u32>(_e64)) * _e71) + _e77);
    let _e84 = textureLoad(TB, vec2<i32>((_e79 & 2047i), (_e79 >> bitcast<u32>(11i))), 0i);
    let _e91 = ZC.k2_[(max((_e84.w & 65535u), 1u) - 1u)];
    let _e95 = (_e91.z & 65535u);
    let _e97 = (_e95 * 4u);
    let _e100 = LB.k2_[_e97];
    let _e101 = bitcast<vec4<f32>>(_e100);
    let _e112 = LB.k2_[(_e97 + 1u)];
    phi_962_ = _e75;
    if ((((_e84.w & 8388608u) != 0u) && _e72) && !(_e74)) {
        phi_962_ = (_e75 - 1i);
    }
    let _e122 = phi_962_;
    phi_965_ = _e84;
    if (_e122 != _e77) {
        let _e125 = ((_e79 + _e122) - _e77);
        let _e130 = textureLoad(TB, vec2<i32>((_e125 & 2047i), (_e125 >> bitcast<u32>(11i))), 0i);
        if ((_e130.w & 8454143u) != (_e84.w & 8454143u)) {
            let _e135 = bitcast<i32>(_e91.w);
            let _e140 = textureLoad(TB, vec2<i32>((_e135 & 2047i), (_e135 >> bitcast<u32>(11i))), 0i);
            phi_964_ = _e140;
        } else {
            phi_964_ = _e130;
        }
        let _e142 = phi_964_;
        phi_965_ = _e142;
    }
    let _e144 = phi_965_;
    if _e74 {
        phi_967_ = bitcast<vec2<f32>>(_e91.xy);
    } else {
        phi_967_ = bitcast<vec2<f32>>(_e144.xy);
    }
    let _e148 = phi_967_;
    let _e150 = ((mat2x2<f32>(vec2<f32>(_e101.x, _e101.y), vec2<f32>(_e101.z, _e101.w)) * _e148) + bitcast<vec2<f32>>(_e112.xy));
    let _e153 = WC.k2_[_e95];
    let _e155 = (_e153.x & 15u);
    if mi {
        let _e156 = (_e155 == 0u);
        if _e156 {
            phi_968_ = _e153.y;
        } else {
            phi_968_ = _e153.x;
        }
        let _e159 = phi_968_;
        let _e161 = (_e159 >> bitcast<u32>(16i));
        let _e163 = j.U4_;
        if (_e161 == 0u) {
            phi_969_ = 0f;
        } else {
            phi_969_ = unpack2x16float(((_e161 + 1023u) * _e163)).x;
        }
        let _e170 = phi_969_;
        phi_970_ = _e170;
        if _e156 {
            phi_970_ = -(_e170);
        }
        let _e173 = phi_970_;
        l1_[0u] = _e173;
    }
    if oi {
        Q0_ = f32(((_e153.x >> bitcast<u32>(4i)) & 15u));
    }
    if ni {
        let _e179 = (_e95 * 8u);
        let _e183 = JB.k2_[(_e179 + 2u)];
        let _e194 = JB.k2_[(_e179 + 3u)];
        if any((_e183 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e199 = ((mat2x2<f32>(vec2<f32>(_e183.x, _e183.y), vec2<f32>(_e183.z, _e183.w)) * _e150) + _e194.xy);
            unnamed.gl_ClipDistance[0i] = (_e199.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e199.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e199.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e199.y);
        } else {
            let _e215 = (_e194.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e215;
            unnamed.gl_ClipDistance[2i] = _e215;
            unnamed.gl_ClipDistance[1i] = _e215;
            unnamed.gl_ClipDistance[0i] = _e215;
        }
    }
    if (_e155 == 1u) {
        a1_ = unpack4x8unorm(_e153.y);
    } else {
        if (mi && (_e155 == 0u)) {
            let _e230 = (_e153.x >> bitcast<u32>(16i));
            let _e232 = j.U4_;
            if (_e230 == 0u) {
                phi_985_ = 0f;
            } else {
                phi_985_ = unpack2x16float(((_e230 + 1023u) * _e232)).x;
            }
            let _e239 = phi_985_;
            l1_[1u] = _e239;
        } else {
            let _e241 = (_e95 * 8u);
            let _e244 = JB.k2_[_e241];
            let _e255 = JB.k2_[(_e241 + 1u)];
            let _e264 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e153.y));
            let _e266 = ((mat2x2<f32>(vec2<f32>(_e244.x, _e244.y), vec2<f32>(_e244.z, _e244.w)) * _e150) + _e255.xy);
            if (_e255.z > 0.9f) {
                phi_983_ = vec4<f32>(_e264.x, _e264.y, 2f, _e264.w);
            } else {
                phi_983_ = vec4<f32>(_e264.x, _e264.y, _e255.w, _e264.w);
            }
            let _e281 = phi_983_;
            if (f32(_e155) == 2f) {
                let _e288 = vec4<f32>(_e266.x, _e281.y, _e281.z, _e281.w);
                phi_984_ = vec4<f32>(_e288.x, 0f, _e288.z, _e288.w);
            } else {
                let _e300 = vec4<f32>(_e281.x, _e281.y, -(_e281.z), _e281.w);
                let _e306 = vec4<f32>(_e266.x, _e300.y, _e300.z, _e300.w);
                phi_984_ = vec4<f32>(_e306.x, _e266.y, _e306.z, _e306.w);
            }
            let _e314 = phi_984_;
            a1_ = _e314;
            let _e316 = a1_[3u];
            a1_[3u] = -(_e316);
        }
    }
    if ((_e58 & 536870912i) != 0i) {
        a1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    phi_609_ = ui;
    if ui {
        phi_609_ = ((_e153.x & 2048u) != 0u);
    }
    let _e321 = phi_609_;
    if _e321 {
        let _e322 = (_e95 * 8u);
        let _e326 = JB.k2_[(_e322 + 4u)];
        let _e337 = JB.k2_[(_e322 + 5u)];
        let _e340 = ((mat2x2<f32>(vec2<f32>(_e326.x, _e326.y), vec2<f32>(_e326.z, _e326.w)) * _e150) + _e337.xy);
        v1_ = vec3<f32>(_e340.x, _e340.y, (1f + _e337.z));
    } else {
        v1_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e347 = j.dg;
    let _e349 = j.eg;
    let _e357 = vec4<f32>(((_e150.x * _e347) - 1f), ((_e150.y * _e349) - sign(_e349)), 0f, 1f);
    let _e361 = LB.k2_[(_e97 + 2u)];
    unnamed.gl_Position = vec4<f32>(_e357.x, _e357.y, ((f32(((_e361.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e357.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e10 = unnamed.gl_Position;
    let _e11 = unnamed.gl_ClipDistance;
    let _e12 = l1_;
    let _e13 = Q0_;
    let _e14 = a1_;
    let _e15 = v1_;
    return VertexOutput(_e10, _e11, _e12, _e13, _e14, _e15);
}
