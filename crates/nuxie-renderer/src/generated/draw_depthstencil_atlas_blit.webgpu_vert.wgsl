enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct oh {
    k2_: array<vec4<u32>>,
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

struct Gf {
    k2_: array<vec2<u32>>,
}

struct Hf {
    k2_: array<vec4<f32>>,
}

struct ph {
    k2_: array<vec4<u32>>,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(1) member: vec2<f32>,
    @location(4) @interpolate(flat, either) member_1: f32,
    @location(6) @interpolate(flat, either) member_2: f32,
    @location(0) member_3: vec4<f32>,
    @location(9) member_4: vec3<f32>,
}

@id(0) override mi: bool = true;
@id(2) override oi: bool = true;
@id(1) override ni: bool = true;
@id(8) override ui: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(2)
var<storage> LB: oh;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> gl_VertexIndex_1: i32;
var<private> MB_1: vec3<f32>;
var<private> J2_: vec2<f32>;
@group(0) @binding(3)
var<storage> WC: Gf;
var<private> Z3_: f32;
var<private> Q0_: f32;
@group(0) @binding(4)
var<storage> JB: Hf;
var<private> a1_: vec4<f32>;
var<private> v1_: vec3<f32>;
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> ZC: ph;
@group(3) @binding(9)
var xa: sampler;

fn main_1() {
    var phi_820_: u32;
    var phi_821_: f32;
    var phi_822_: f32;
    var phi_831_: vec4<f32>;
    var phi_832_: vec4<f32>;
    var phi_455_: bool;
    var phi_833_: f32;

    let _e54 = MB_1;
    let _e57 = (bitcast<u32>(_e54.z) & 65535u);
    let _e62 = LB.k2_[((_e57 * 4u) + 2u)];
    let _e64 = _e54.xy;
    let _e66 = bitcast<vec3<f32>>(_e62.yzw);
    let _e72 = j.Nh;
    J2_ = (((_e64 * _e66.x) + _e66.yz) * _e72);
    let _e76 = WC.k2_[_e57];
    let _e78 = (_e76.x & 15u);
    if mi {
        let _e79 = (_e78 == 0u);
        if _e79 {
            phi_820_ = _e76.y;
        } else {
            phi_820_ = _e76.x;
        }
        let _e82 = phi_820_;
        let _e84 = (_e82 >> bitcast<u32>(16i));
        let _e86 = j.U4_;
        if (_e84 == 0u) {
            phi_821_ = 0f;
        } else {
            phi_821_ = unpack2x16float(((_e84 + 1023u) * _e86)).x;
        }
        let _e93 = phi_821_;
        phi_822_ = _e93;
        if _e79 {
            phi_822_ = -(_e93);
        }
        let _e96 = phi_822_;
        Z3_ = _e96;
    }
    if oi {
        Q0_ = f32(((_e76.x >> bitcast<u32>(4i)) & 15u));
    }
    if ni {
        let _e101 = (_e57 * 8u);
        let _e105 = JB.k2_[(_e101 + 2u)];
        let _e116 = JB.k2_[(_e101 + 3u)];
        if any((_e105 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e131 = ((mat2x2<f32>(vec2<f32>(_e105.x, _e105.y), vec2<f32>(_e105.z, _e105.w)) * _e64) + _e116.xy);
            unnamed.gl_ClipDistance[0i] = (_e131.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e131.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e131.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e131.y);
        } else {
            let _e121 = (_e116.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e121;
            unnamed.gl_ClipDistance[2i] = _e121;
            unnamed.gl_ClipDistance[1i] = _e121;
            unnamed.gl_ClipDistance[0i] = _e121;
        }
    }
    if (_e78 == 1u) {
        a1_ = unpack4x8unorm(_e76.y);
    } else {
        let _e147 = (_e57 * 8u);
        let _e150 = JB.k2_[_e147];
        let _e161 = JB.k2_[(_e147 + 1u)];
        let _e170 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e76.y));
        let _e172 = ((mat2x2<f32>(vec2<f32>(_e150.x, _e150.y), vec2<f32>(_e150.z, _e150.w)) * _e64) + _e161.xy);
        if (_e161.z > 0.9f) {
            phi_831_ = vec4<f32>(_e170.x, _e170.y, 2f, _e170.w);
        } else {
            phi_831_ = vec4<f32>(_e170.x, _e170.y, _e161.w, _e170.w);
        }
        let _e187 = phi_831_;
        if (f32(_e78) == 2f) {
            let _e213 = vec4<f32>(_e172.x, _e187.y, _e187.z, _e187.w);
            phi_832_ = vec4<f32>(_e213.x, 0f, _e213.z, _e213.w);
        } else {
            let _e195 = vec4<f32>(_e187.x, _e187.y, -(_e187.z), _e187.w);
            let _e201 = vec4<f32>(_e172.x, _e195.y, _e195.z, _e195.w);
            phi_832_ = vec4<f32>(_e201.x, _e172.y, _e201.z, _e201.w);
        }
        let _e220 = phi_832_;
        a1_ = _e220;
        let _e222 = a1_[3u];
        a1_[3u] = -(_e222);
    }
    phi_455_ = ui;
    if ui {
        phi_455_ = ((_e76.x & 2048u) != 0u);
    }
    let _e229 = phi_455_;
    if _e229 {
        let _e230 = (_e57 * 8u);
        let _e234 = JB.k2_[(_e230 + 4u)];
        let _e245 = JB.k2_[(_e230 + 5u)];
        let _e248 = ((mat2x2<f32>(vec2<f32>(_e234.x, _e234.y), vec2<f32>(_e234.z, _e234.w)) * _e64) + _e245.xy);
        phi_833_ = (1f + _e245.z);
        if ((_e76.x & 4096u) != 0u) {
            phi_833_ = (-1f - f32(((_e76.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e259 = phi_833_;
        v1_ = vec3<f32>(_e248.x, _e248.y, _e259);
    } else {
        v1_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e264 = j.dg;
    let _e266 = j.eg;
    let _e274 = vec4<f32>(((_e54.x * _e264) - 1f), ((_e54.y * _e266) - sign(_e266)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e274.x, _e274.y, ((f32(((_e62.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e274.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) MB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    MB_1 = MB;
    main_1();
    let _e13 = unnamed.gl_Position;
    let _e14 = unnamed.gl_ClipDistance;
    let _e15 = J2_;
    let _e16 = Z3_;
    let _e17 = Q0_;
    let _e18 = a1_;
    let _e19 = v1_;
    return VertexOutput(_e13, _e14, _e15, _e16, _e17, _e18, _e19);
}
