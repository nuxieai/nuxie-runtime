struct UB {
    Qc: f32,
    Td: f32,
    bg: f32,
    cg: f32,
    A6_: u32,
    X9_: u32,
    Nf: u32,
    Of: u32,
    j8_: vec4<i32>,
    Lh: vec2<f32>,
    Ud: vec2<f32>,
    j2_: u32,
    Ph: f32,
    T4_: u32,
    a3_: f32,
    Vd: f32,
    Hf: u32,
    L3_: f32,
    M3_: f32,
    Wd: f32,
    Ih: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct mh {
    k2_: array<vec4<u32>>,
}

struct Ef {
    k2_: array<vec2<u32>>,
}

struct Ff {
    k2_: array<vec4<f32>>,
}

struct nh {
    k2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(1) member: f32,
    @location(0) member_1: vec2<f32>,
    @location(3) member_2: vec4<f32>,
    @location(4) @interpolate(flat, either) member_3: vec4<f32>,
    @location(5) @interpolate(flat, either) member_4: u32,
    @location(6) @interpolate(flat, either) member_5: u32,
    @location(2) member_6: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(1) override li: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> GC_1: vec4<f32>;
var<private> i5_: f32;
var<private> YB_1: vec4<f32>;
var<private> ND_1: vec4<f32>;
var<private> f2_: vec2<f32>;
var<private> CD_1: vec4<f32>;
var<private> PB_1: vec4<f32>;
var<private> R0_: vec4<f32>;
var<private> SB_1: vec4<f32>;
var<private> Q1_: vec4<f32>;
var<private> ZB_1: u32;
var<private> I3_: u32;
var<private> AC_1: u32;
var<private> H1_: u32;
var<private> BC_1: u32;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> OC_1: vec4<f32>;
var<private> OD_1: vec4<f32>;
var<private> j5_: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> LB: mh;
@group(0) @binding(3)
var<storage> WC: Ef;
@group(0) @binding(4)
var<storage> JB: Ff;
@group(0) @binding(5)
var<storage> ZC: nh;
@group(3) @binding(9)
var wa: sampler;
var<private> LC_1: u32;

fn main_1() {
    var phi_228_: bool;
    var phi_753_: vec2<f32>;
    var phi_755_: vec2<f32>;
    var phi_754_: vec2<f32>;
    var phi_756_: vec2<f32>;
    var phi_593_: bool;
    var phi_757_: vec4<f32>;
    var phi_770_: vec4<f32>;
    var phi_771_: vec4<f32>;

    let _e49 = GC_1[2u];
    let _e50 = (_e49 == 0f);
    phi_228_ = _e50;
    if !(_e50) {
        let _e53 = GC_1[3u];
        phi_228_ = (_e53 == 0f);
    }
    let _e56 = phi_228_;
    i5_ = select(1f, 0f, _e56);
    let _e58 = GC_1;
    let _e59 = _e58.xy;
    let _e60 = YB_1;
    let _e65 = vec2<f32>(_e60.x, _e60.y);
    let _e66 = vec2<f32>(_e60.z, _e60.w);
    let _e67 = mat2x2<f32>(_e65, _e66);
    let _e69 = transpose(_naga_inverse_2x2_f32(_e67));
    phi_754_ = _e59;
    if !(_e56) {
        let _e79 = ((0.5f * (abs(_e69[1].x) + abs(_e69[1].y))) / dot(_e66, _e69[1]));
        if (_e79 >= 0.5f) {
            let _e91 = i5_;
            i5_ = (_e91 * (0.5f / _e79));
            phi_753_ = vec2<f32>(0.5f, _e59.y);
        } else {
            phi_753_ = vec2<f32>((_e58.x + (_e79 * _e49)), _e59.y);
        }
        let _e94 = phi_753_;
        let _e103 = ((0.5f * (abs(_e69[0].x) + abs(_e69[0].y))) / dot(_e65, _e69[0]));
        if (_e103 >= 0.5f) {
            let _e117 = i5_;
            i5_ = (_e117 * (0.5f / _e103));
            phi_755_ = vec2<f32>(_e94.x, 0.5f);
        } else {
            let _e106 = GC_1[3u];
            phi_755_ = vec2<f32>(_e94.x, (_e94.y + (_e103 * _e106)));
        }
        let _e120 = phi_755_;
        phi_754_ = _e120;
    }
    let _e122 = phi_754_;
    let _e123 = ND_1;
    let _e132 = CD_1;
    f2_ = ((mat2x2<f32>(vec2<f32>(_e123.x, _e123.y), vec2<f32>(_e123.z, _e123.w)) * _e122) + _e132.xy);
    let _e136 = PB_1;
    let _e138 = ((_e67 * _e122) + _e136.xy);
    phi_756_ = _e138;
    if _e56 {
        let _e140 = (_e69 * _e58.zw);
        phi_756_ = (_e138 + ((_e140 * ((abs(_e140.x) + abs(_e140.y)) / dot(_e140, _e140))) * 0.5f));
    }
    let _e152 = phi_756_;
    if li {
        let _e153 = SB_1;
        let _e158 = vec2<f32>(_e153.x, _e153.y);
        let _e159 = vec2<f32>(_e153.z, _e153.w);
        switch bitcast<i32>(0u) {
            default: {
                let _e165 = (abs(_e158) + abs(_e159));
                let _e167 = (_e165.x != 0f);
                phi_593_ = _e167;
                if _e167 {
                    phi_593_ = (_e165.y != 0f);
                }
                let _e171 = phi_593_;
                if _e171 {
                    let _e175 = ((mat2x2<f32>(_e158, _e159) * _e152) + _e136.zw);
                    let _e176 = -(_e175);
                    let _e182 = (vec2<f32>(1f, 1f) / _e165).xyxy;
                    phi_757_ = (((vec4<f32>(_e175.x, _e175.y, _e176.x, _e176.y) * _e182) + _e182) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_757_ = _e136.zwzw;
                    break;
                }
            }
        }
        let _e187 = phi_757_;
        R0_ = _e187;
    }
    let _e188 = ZB_1;
    Q1_ = unpack4x8unorm(_e188);
    let _e190 = AC_1;
    I3_ = _e190;
    let _e191 = BC_1;
    H1_ = _e191;
    let _e193 = j.bg;
    let _e195 = j.cg;
    let _e205 = OC_1[3u];
    if (_e205 != 0f) {
        let _e207 = OD_1;
        let _e216 = OC_1;
        let _e218 = OC_1[2u];
        let _e223 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, _e218);
        let _e225 = ((mat2x2<f32>(vec2<f32>(_e207.x, _e207.y), vec2<f32>(_e207.z, _e207.w)) * _e152) + _e132.zw);
        if (_e216.x > 0.9f) {
            phi_770_ = vec4<f32>(_e223.x, _e223.y, 2f, _e223.w);
        } else {
            phi_770_ = vec4<f32>(_e223.x, _e223.y, _e216.y, _e223.w);
        }
        let _e240 = phi_770_;
        if (_e205 == 2f) {
            let _e266 = vec4<f32>(_e225.x, _e240.y, _e240.z, _e240.w);
            phi_771_ = vec4<f32>(_e266.x, 0f, _e266.z, _e266.w);
        } else {
            let _e248 = vec4<f32>(_e240.x, _e240.y, -(_e240.z), _e240.w);
            let _e254 = vec4<f32>(_e225.x, _e248.y, _e248.z, _e248.w);
            phi_771_ = vec4<f32>(_e254.x, _e225.y, _e254.z, _e254.w);
        }
        let _e273 = phi_771_;
        j5_ = _e273;
    } else {
        j5_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    unnamed.gl_Position = vec4<f32>(((_e152.x * _e193) - 1f), ((_e152.y * _e195) - sign(_e195)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) GC: vec4<f32>, @location(2) YB: vec4<f32>, @location(9) ND: vec4<f32>, @location(11) CD: vec4<f32>, @location(4) PB: vec4<f32>, @location(3) SB: vec4<f32>, @location(5) ZB: u32, @location(6) AC: u32, @location(7) BC: u32, @location(12) OC: vec4<f32>, @location(10) OD: vec4<f32>, @location(8) LC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    GC_1 = GC;
    YB_1 = YB;
    ND_1 = ND;
    CD_1 = CD;
    PB_1 = PB;
    SB_1 = SB;
    ZB_1 = ZB;
    AC_1 = AC;
    BC_1 = BC;
    OC_1 = OC;
    OD_1 = OD;
    LC_1 = LC;
    main_1();
    let _e39 = i5_;
    let _e40 = f2_;
    let _e41 = R0_;
    let _e42 = Q1_;
    let _e43 = I3_;
    let _e44 = H1_;
    let _e45 = j5_;
    let _e46 = unnamed.gl_Position;
    return VertexOutput(_e39, _e40, _e41, _e42, _e43, _e44, _e45, _e46);
}

fn _naga_inverse_2x2_f32(m: mat2x2<f32>) -> mat2x2<f32> {
    var adj: mat2x2<f32>;
    adj[0][0] = m[1][1];
    adj[0][1] = -m[0][1];
    adj[1][0] = -m[1][0];
    adj[1][1] = m[0][0];

    let det: f32 = m[0][0] * m[1][1] - m[1][0] * m[0][1];
    return adj * (1 / det);
}
