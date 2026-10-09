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

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct di {
    r2_: array<vec4<u32>>,
}

struct jg {
    r2_: array<vec2<u32>>,
}

struct kg {
    r2_: array<vec4<f32>>,
}

struct ei {
    r2_: array<vec4<u32>>,
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

@id(1) override Xi: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> GC_1: vec4<f32>;
var<private> q5_: f32;
var<private> ZB_1: vec4<f32>;
var<private> QD_1: vec4<f32>;
var<private> l2_: vec2<f32>;
var<private> ED_1: vec4<f32>;
var<private> PB_1: vec4<f32>;
var<private> V0_: vec4<f32>;
var<private> SB_1: vec4<f32>;
var<private> T1_: vec4<f32>;
var<private> AC_1: u32;
var<private> S3_: u32;
var<private> BC_1: u32;
var<private> J1_: u32;
var<private> CC_1: u32;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> FD_1: vec4<f32>;
var<private> RD_1: vec4<f32>;
var<private> r5_: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> KB: di;
@group(0) @binding(3)
var<storage> VC: jg;
@group(0) @binding(4)
var<storage> JB: kg;
@group(0) @binding(5)
var<storage> BD: ei;
@group(3) @binding(9)
var ab: sampler;
var<private> LC_1: u32;

fn main_1() {
    var phi_242_: bool;
    var phi_778_: vec2<f32>;
    var phi_780_: vec2<f32>;
    var phi_779_: vec2<f32>;
    var phi_781_: vec2<f32>;
    var phi_610_: bool;
    var phi_782_: vec4<f32>;
    var phi_795_: f32;

    let _e50 = GC_1[2u];
    let _e51 = (_e50 == 0f);
    phi_242_ = _e51;
    if !(_e51) {
        let _e54 = GC_1[3u];
        phi_242_ = (_e54 == 0f);
    }
    let _e57 = phi_242_;
    q5_ = select(1f, 0f, _e57);
    let _e59 = GC_1;
    let _e60 = _e59.xy;
    let _e61 = ZB_1;
    let _e66 = vec2<f32>(_e61.x, _e61.y);
    let _e67 = vec2<f32>(_e61.z, _e61.w);
    let _e68 = mat2x2<f32>(_e66, _e67);
    let _e70 = transpose(_naga_inverse_2x2_f32(_e68));
    phi_779_ = _e60;
    if !(_e57) {
        let _e80 = ((0.5f * (abs(_e70[1].x) + abs(_e70[1].y))) / dot(_e67, _e70[1]));
        if (_e80 >= 0.5f) {
            let _e92 = q5_;
            q5_ = (_e92 * (0.5f / _e80));
            phi_778_ = vec2<f32>(0.5f, _e60.y);
        } else {
            phi_778_ = vec2<f32>((_e59.x + (_e80 * _e50)), _e60.y);
        }
        let _e95 = phi_778_;
        let _e104 = ((0.5f * (abs(_e70[0].x) + abs(_e70[0].y))) / dot(_e66, _e70[0]));
        if (_e104 >= 0.5f) {
            let _e118 = q5_;
            q5_ = (_e118 * (0.5f / _e104));
            phi_780_ = vec2<f32>(_e95.x, 0.5f);
        } else {
            let _e107 = GC_1[3u];
            phi_780_ = vec2<f32>(_e95.x, (_e95.y + (_e104 * _e107)));
        }
        let _e121 = phi_780_;
        phi_779_ = _e121;
    }
    let _e123 = phi_779_;
    let _e124 = QD_1;
    let _e133 = ED_1;
    l2_ = ((mat2x2<f32>(vec2<f32>(_e124.x, _e124.y), vec2<f32>(_e124.z, _e124.w)) * _e123) + _e133.xy);
    let _e137 = PB_1;
    let _e139 = ((_e68 * _e123) + _e137.xy);
    phi_781_ = _e139;
    if _e57 {
        let _e141 = (_e70 * _e59.zw);
        phi_781_ = (_e139 + ((_e141 * ((abs(_e141.x) + abs(_e141.y)) / dot(_e141, _e141))) * 0.5f));
    }
    let _e153 = phi_781_;
    if Xi {
        let _e154 = SB_1;
        let _e159 = vec2<f32>(_e154.x, _e154.y);
        let _e160 = vec2<f32>(_e154.z, _e154.w);
        switch bitcast<i32>(0u) {
            default: {
                let _e166 = (abs(_e159) + abs(_e160));
                let _e168 = (_e166.x != 0f);
                phi_610_ = _e168;
                if _e168 {
                    phi_610_ = (_e166.y != 0f);
                }
                let _e172 = phi_610_;
                if _e172 {
                    let _e176 = ((mat2x2<f32>(_e159, _e160) * _e153) + _e137.zw);
                    let _e177 = -(_e176);
                    let _e183 = (vec2<f32>(1f, 1f) / _e166).xyxy;
                    phi_782_ = (((vec4<f32>(_e176.x, _e176.y, _e177.x, _e177.y) * _e183) + _e183) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_782_ = _e137.zwzw;
                    break;
                }
            }
        }
        let _e188 = phi_782_;
        V0_ = _e188;
    }
    let _e189 = AC_1;
    T1_ = unpack4x8unorm(_e189);
    let _e191 = BC_1;
    S3_ = _e191;
    let _e192 = CC_1;
    J1_ = _e192;
    let _e194 = j.Hg;
    let _e196 = j.Ig;
    let _e206 = FD_1[2u];
    let _e207 = bitcast<u32>(_e206);
    if (_e207 != 0u) {
        let _e209 = RD_1;
        let _e218 = FD_1;
        let _e220 = ((mat2x2<f32>(vec2<f32>(_e209.x, _e209.y), vec2<f32>(_e209.z, _e209.w)) * _e153) + _e133.zw);
        let _e231 = ((_e218.y + (f32(_e207) * 0.125f)) + (max(_e218.x, 0f) * 0.00024414063f));
        if (_e218.x < 0f) {
            phi_795_ = -(_e231);
        } else {
            phi_795_ = _e231;
        }
        let _e234 = phi_795_;
        r5_ = vec4<f32>(_e220.x, _e220.y, _e234, (-0.75f - round(255f)));
    } else {
        r5_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    unnamed.gl_Position = vec4<f32>(((_e153.x * _e194) - 1f), ((_e153.y * _e196) - sign(_e196)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) GC: vec4<f32>, @location(2) ZB: vec4<f32>, @location(9) QD: vec4<f32>, @location(11) ED: vec4<f32>, @location(4) PB: vec4<f32>, @location(3) SB: vec4<f32>, @location(5) AC: u32, @location(6) BC: u32, @location(7) CC: u32, @location(12) FD: vec4<f32>, @location(10) RD: vec4<f32>, @location(8) LC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    GC_1 = GC;
    ZB_1 = ZB;
    QD_1 = QD;
    ED_1 = ED;
    PB_1 = PB;
    SB_1 = SB;
    AC_1 = AC;
    BC_1 = BC;
    CC_1 = CC;
    FD_1 = FD;
    RD_1 = RD;
    LC_1 = LC;
    main_1();
    let _e39 = q5_;
    let _e40 = l2_;
    let _e41 = V0_;
    let _e42 = T1_;
    let _e43 = S3_;
    let _e44 = J1_;
    let _e45 = r5_;
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
