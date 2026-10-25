struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    H8_: vec4<i32>,
    Fi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ji: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Ci: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct ki {
    v2_: array<vec4<u32>>,
}

struct og {
    v2_: array<vec2<u32>>,
}

struct pg {
    v2_: array<vec4<f32>>,
}

struct li {
    v2_: array<vec4<u32>>,
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

@id(1) override fj: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> HC_1: vec4<f32>;
var<private> v5_: f32;
var<private> AC_1: vec4<f32>;
var<private> TD_1: vec4<f32>;
var<private> n2_: vec2<f32>;
var<private> GD_1: vec4<f32>;
var<private> PB_1: vec4<f32>;
var<private> W0_: vec4<f32>;
var<private> TB_1: vec4<f32>;
var<private> Y1_: vec4<f32>;
var<private> BC_1: u32;
var<private> U3_: u32;
var<private> CC_1: u32;
var<private> N1_: u32;
var<private> DC_1: u32;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> HD_1: vec4<f32>;
var<private> UD_1: vec4<f32>;
var<private> w5_: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var VB: texture_2d<u32>;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(0) @binding(2)
var<storage> KB: ki;
@group(0) @binding(3)
var<storage> XC: og;
@group(0) @binding(4)
var<storage> JB: pg;
@group(0) @binding(5)
var<storage> DD: li;
@group(3) @binding(9)
var db: sampler;
var<private> MC_1: u32;

fn main_1() {
    var phi_242_: bool;
    var phi_778_: vec2<f32>;
    var phi_780_: vec2<f32>;
    var phi_779_: vec2<f32>;
    var phi_781_: vec2<f32>;
    var phi_610_: bool;
    var phi_782_: vec4<f32>;
    var phi_795_: f32;

    let _e50 = HC_1[2u];
    let _e51 = (_e50 == 0f);
    phi_242_ = _e51;
    if !(_e51) {
        let _e54 = HC_1[3u];
        phi_242_ = (_e54 == 0f);
    }
    let _e57 = phi_242_;
    v5_ = select(1f, 0f, _e57);
    let _e59 = HC_1;
    let _e60 = _e59.xy;
    let _e61 = AC_1;
    let _e66 = vec2<f32>(_e61.x, _e61.y);
    let _e67 = vec2<f32>(_e61.z, _e61.w);
    let _e68 = mat2x2<f32>(_e66, _e67);
    let _e70 = transpose(_naga_inverse_2x2_f32(_e68));
    phi_779_ = _e60;
    if !(_e57) {
        let _e80 = ((0.5f * (abs(_e70[1].x) + abs(_e70[1].y))) / dot(_e67, _e70[1]));
        if (_e80 >= 0.5f) {
            let _e92 = v5_;
            v5_ = (_e92 * (0.5f / _e80));
            phi_778_ = vec2<f32>(0.5f, _e60.y);
        } else {
            phi_778_ = vec2<f32>((_e59.x + (_e80 * _e50)), _e60.y);
        }
        let _e95 = phi_778_;
        let _e104 = ((0.5f * (abs(_e70[0].x) + abs(_e70[0].y))) / dot(_e66, _e70[0]));
        if (_e104 >= 0.5f) {
            let _e118 = v5_;
            v5_ = (_e118 * (0.5f / _e104));
            phi_780_ = vec2<f32>(_e95.x, 0.5f);
        } else {
            let _e107 = HC_1[3u];
            phi_780_ = vec2<f32>(_e95.x, (_e95.y + (_e104 * _e107)));
        }
        let _e121 = phi_780_;
        phi_779_ = _e121;
    }
    let _e123 = phi_779_;
    let _e124 = TD_1;
    let _e133 = GD_1;
    n2_ = ((mat2x2<f32>(vec2<f32>(_e124.x, _e124.y), vec2<f32>(_e124.z, _e124.w)) * _e123) + _e133.xy);
    let _e137 = PB_1;
    let _e139 = ((_e68 * _e123) + _e137.xy);
    phi_781_ = _e139;
    if _e57 {
        let _e141 = (_e70 * _e59.zw);
        phi_781_ = (_e139 + ((_e141 * ((abs(_e141.x) + abs(_e141.y)) / dot(_e141, _e141))) * 0.5f));
    }
    let _e153 = phi_781_;
    if fj {
        let _e154 = TB_1;
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
        W0_ = _e188;
    }
    let _e189 = BC_1;
    Y1_ = unpack4x8unorm(_e189);
    let _e191 = CC_1;
    U3_ = _e191;
    let _e192 = DC_1;
    N1_ = _e192;
    let _e194 = j.Mg;
    let _e196 = j.Ng;
    let _e206 = HD_1[2u];
    let _e207 = bitcast<u32>(_e206);
    if (_e207 != 0u) {
        let _e209 = UD_1;
        let _e218 = HD_1;
        let _e220 = ((mat2x2<f32>(vec2<f32>(_e209.x, _e209.y), vec2<f32>(_e209.z, _e209.w)) * _e153) + _e133.zw);
        let _e231 = ((_e218.y + (f32(_e207) * 0.125f)) + (max(_e218.x, 0f) * 0.00024414063f));
        if (_e218.x < 0f) {
            phi_795_ = -(_e231);
        } else {
            phi_795_ = _e231;
        }
        let _e234 = phi_795_;
        w5_ = vec4<f32>(_e220.x, _e220.y, _e234, (-0.75f - round(255f)));
    } else {
        w5_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    unnamed.gl_Position = vec4<f32>(((_e153.x * _e194) - 1f), ((_e153.y * _e196) - sign(_e196)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) HC: vec4<f32>, @location(2) AC: vec4<f32>, @location(9) TD: vec4<f32>, @location(11) GD: vec4<f32>, @location(4) PB: vec4<f32>, @location(3) TB: vec4<f32>, @location(5) BC: u32, @location(6) CC: u32, @location(7) DC: u32, @location(12) HD: vec4<f32>, @location(10) UD: vec4<f32>, @location(8) MC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    HC_1 = HC;
    AC_1 = AC;
    TD_1 = TD;
    GD_1 = GD;
    PB_1 = PB;
    TB_1 = TB;
    BC_1 = BC;
    CC_1 = CC;
    DC_1 = DC;
    HD_1 = HD;
    UD_1 = UD;
    MC_1 = MC;
    main_1();
    let _e39 = v5_;
    let _e40 = n2_;
    let _e41 = W0_;
    let _e42 = Y1_;
    let _e43 = U3_;
    let _e44 = N1_;
    let _e45 = w5_;
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
