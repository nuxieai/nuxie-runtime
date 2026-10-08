struct VB {
    vd: f32,
    Ce: f32,
    Gg: f32,
    Hg: f32,
    L6_: u32,
    xa: u32,
    sg: u32,
    tg: u32,
    C8_: vec4<i32>,
    Bi: vec2<f32>,
    De: vec2<f32>,
    r2_: u32,
    Fi: f32,
    p6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    E3_: f32,
    F3_: f32,
    Fe: f32,
    yi: u32,
    wa: u32,
    cd: f32,
    g7_: f32,
    Db: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct gi {
    v2_: array<vec4<u32>>,
}

struct jg {
    v2_: array<vec2<u32>>,
}

struct kg {
    v2_: array<vec4<f32>>,
}

struct hi {
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

@id(1) override Zi: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> GC_1: vec4<f32>;
var<private> o5_: f32;
var<private> ZB_1: vec4<f32>;
var<private> QD_1: vec4<f32>;
var<private> m2_: vec2<f32>;
var<private> FD_1: vec4<f32>;
var<private> PB_1: vec4<f32>;
var<private> W0_: vec4<f32>;
var<private> SB_1: vec4<f32>;
var<private> U1_: vec4<f32>;
var<private> AC_1: u32;
var<private> R3_: u32;
var<private> BC_1: u32;
var<private> K1_: u32;
var<private> CC_1: u32;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> OC_1: vec4<f32>;
var<private> RD_1: vec4<f32>;
var<private> p5_: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> KB: gi;
@group(0) @binding(3)
var<storage> WC: jg;
@group(0) @binding(4)
var<storage> JB: kg;
@group(0) @binding(5)
var<storage> CD: hi;
@group(3) @binding(9)
var Va: sampler;
var<private> LC_1: u32;

fn main_1() {
    var phi_228_: bool;
    var phi_755_: vec2<f32>;
    var phi_757_: vec2<f32>;
    var phi_756_: vec2<f32>;
    var phi_758_: vec2<f32>;
    var phi_595_: bool;
    var phi_759_: vec4<f32>;
    var phi_772_: vec4<f32>;
    var phi_773_: vec4<f32>;

    let _e49 = GC_1[2u];
    let _e50 = (_e49 == 0f);
    phi_228_ = _e50;
    if !(_e50) {
        let _e53 = GC_1[3u];
        phi_228_ = (_e53 == 0f);
    }
    let _e56 = phi_228_;
    o5_ = select(1f, 0f, _e56);
    let _e58 = GC_1;
    let _e59 = _e58.xy;
    let _e60 = ZB_1;
    let _e65 = vec2<f32>(_e60.x, _e60.y);
    let _e66 = vec2<f32>(_e60.z, _e60.w);
    let _e67 = mat2x2<f32>(_e65, _e66);
    let _e69 = transpose(_naga_inverse_2x2_f32(_e67));
    phi_756_ = _e59;
    if !(_e56) {
        let _e79 = ((0.5f * (abs(_e69[1].x) + abs(_e69[1].y))) / dot(_e66, _e69[1]));
        if (_e79 >= 0.5f) {
            let _e91 = o5_;
            o5_ = (_e91 * (0.5f / _e79));
            phi_755_ = vec2<f32>(0.5f, _e59.y);
        } else {
            phi_755_ = vec2<f32>((_e58.x + (_e79 * _e49)), _e59.y);
        }
        let _e94 = phi_755_;
        let _e103 = ((0.5f * (abs(_e69[0].x) + abs(_e69[0].y))) / dot(_e65, _e69[0]));
        if (_e103 >= 0.5f) {
            let _e117 = o5_;
            o5_ = (_e117 * (0.5f / _e103));
            phi_757_ = vec2<f32>(_e94.x, 0.5f);
        } else {
            let _e106 = GC_1[3u];
            phi_757_ = vec2<f32>(_e94.x, (_e94.y + (_e103 * _e106)));
        }
        let _e120 = phi_757_;
        phi_756_ = _e120;
    }
    let _e122 = phi_756_;
    let _e123 = QD_1;
    let _e132 = FD_1;
    m2_ = ((mat2x2<f32>(vec2<f32>(_e123.x, _e123.y), vec2<f32>(_e123.z, _e123.w)) * _e122) + _e132.xy);
    let _e136 = PB_1;
    let _e138 = ((_e67 * _e122) + _e136.xy);
    phi_758_ = _e138;
    if _e56 {
        let _e140 = (_e69 * _e58.zw);
        phi_758_ = (_e138 + ((_e140 * ((abs(_e140.x) + abs(_e140.y)) / dot(_e140, _e140))) * 0.5f));
    }
    let _e152 = phi_758_;
    if Zi {
        let _e153 = SB_1;
        let _e158 = vec2<f32>(_e153.x, _e153.y);
        let _e159 = vec2<f32>(_e153.z, _e153.w);
        switch bitcast<i32>(0u) {
            default: {
                let _e165 = (abs(_e158) + abs(_e159));
                let _e167 = (_e165.x != 0f);
                phi_595_ = _e167;
                if _e167 {
                    phi_595_ = (_e165.y != 0f);
                }
                let _e171 = phi_595_;
                if _e171 {
                    let _e175 = ((mat2x2<f32>(_e158, _e159) * _e152) + _e136.zw);
                    let _e176 = -(_e175);
                    let _e182 = (vec2<f32>(1f, 1f) / _e165).xyxy;
                    phi_759_ = (((vec4<f32>(_e175.x, _e175.y, _e176.x, _e176.y) * _e182) + _e182) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_759_ = _e136.zwzw;
                    break;
                }
            }
        }
        let _e187 = phi_759_;
        W0_ = _e187;
    }
    let _e188 = AC_1;
    U1_ = unpack4x8unorm(_e188);
    let _e190 = BC_1;
    R3_ = _e190;
    let _e191 = CC_1;
    K1_ = _e191;
    let _e193 = j.Gg;
    let _e195 = j.Hg;
    let _e205 = OC_1[3u];
    if (_e205 != 0f) {
        let _e207 = RD_1;
        let _e216 = OC_1;
        let _e218 = OC_1[2u];
        let _e223 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, _e218);
        let _e225 = ((mat2x2<f32>(vec2<f32>(_e207.x, _e207.y), vec2<f32>(_e207.z, _e207.w)) * _e152) + _e132.zw);
        if (_e216.x > 0.9f) {
            phi_772_ = vec4<f32>(_e223.x, _e223.y, 2f, _e223.w);
        } else {
            phi_772_ = vec4<f32>(_e223.x, _e223.y, _e216.y, _e223.w);
        }
        let _e240 = phi_772_;
        if (_e205 == 2f) {
            let _e266 = vec4<f32>(_e225.x, _e240.y, _e240.z, _e240.w);
            phi_773_ = vec4<f32>(_e266.x, 0f, _e266.z, _e266.w);
        } else {
            let _e248 = vec4<f32>(_e240.x, _e240.y, -(_e240.z), _e240.w);
            let _e254 = vec4<f32>(_e225.x, _e248.y, _e248.z, _e248.w);
            phi_773_ = vec4<f32>(_e254.x, _e225.y, _e254.z, _e254.w);
        }
        let _e273 = phi_773_;
        p5_ = _e273;
    } else {
        p5_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    unnamed.gl_Position = vec4<f32>(((_e152.x * _e193) - 1f), ((_e152.y * _e195) - sign(_e195)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) GC: vec4<f32>, @location(2) ZB: vec4<f32>, @location(9) QD: vec4<f32>, @location(11) FD: vec4<f32>, @location(4) PB: vec4<f32>, @location(3) SB: vec4<f32>, @location(5) AC: u32, @location(6) BC: u32, @location(7) CC: u32, @location(12) OC: vec4<f32>, @location(10) RD: vec4<f32>, @location(8) LC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    GC_1 = GC;
    ZB_1 = ZB;
    QD_1 = QD;
    FD_1 = FD;
    PB_1 = PB;
    SB_1 = SB;
    AC_1 = AC;
    BC_1 = BC;
    CC_1 = CC;
    OC_1 = OC;
    RD_1 = RD;
    LC_1 = LC;
    main_1();
    let _e39 = o5_;
    let _e40 = m2_;
    let _e41 = W0_;
    let _e42 = U1_;
    let _e43 = R3_;
    let _e44 = K1_;
    let _e45 = p5_;
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
