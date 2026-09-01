struct DC {
    gc: f32,
    qd: f32,
    jf: f32,
    kf: f32,
    o6_: u32,
    Lg: u32,
    Ue: u32,
    Ve: u32,
    T7_: vec4<i32>,
    Hg: vec2<f32>,
    rd: vec2<f32>,
    a2_: u32,
    Mg: f32,
    c6_: u32,
    R2_: f32,
    sd: f32,
    Pe: u32,
    B3_: f32,
    C3_: f32,
    td: f32,
    Eg: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct hg {
    c2_: array<vec4<u32>>,
}

struct Me {
    c2_: array<vec2<u32>>,
}

struct Ne {
    c2_: array<vec4<f32>>,
}

struct ig {
    c2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(1) member: f32,
    @location(0) member_1: vec2<f32>,
    @location(2) member_2: vec4<f32>,
    @location(3) @interpolate(flat, either) member_3: f32,
    @location(4) @interpolate(flat, either) member_4: u32,
    @location(5) @interpolate(flat, either) member_5: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(1) override fh: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> IC_1: vec4<f32>;
var<private> T4_: f32;
var<private> XB_1: vec4<f32>;
var<private> X1_: vec2<f32>;
var<private> OB_1: vec4<f32>;
var<private> M0_: vec4<f32>;
var<private> SB_1: vec4<f32>;
var<private> H1_: f32;
var<private> IB_1: vec4<u32>;
var<private> x3_: u32;
var<private> A1_: u32;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var MC: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> QB: hg;
@group(0) @binding(3)
var<storage> BD: Me;
@group(0) @binding(4)
var<storage> RB: Ne;
@group(0) @binding(5)
var<storage> FD: ig;
@group(3) @binding(9)
var aa: sampler;

fn main_1() {
    var phi_175_: bool;
    var phi_568_: vec2<f32>;
    var phi_570_: vec2<f32>;
    var phi_569_: vec2<f32>;
    var phi_571_: vec2<f32>;
    var phi_483_: bool;
    var phi_572_: vec4<f32>;

    let _e37 = IC_1[2u];
    let _e38 = (_e37 == 0f);
    phi_175_ = _e38;
    if !(_e38) {
        let _e41 = IC_1[3u];
        phi_175_ = (_e41 == 0f);
    }
    let _e44 = phi_175_;
    T4_ = select(1f, 0f, _e44);
    let _e46 = IC_1;
    let _e47 = _e46.xy;
    let _e48 = XB_1;
    let _e53 = vec2<f32>(_e48.x, _e48.y);
    let _e54 = vec2<f32>(_e48.z, _e48.w);
    let _e55 = mat2x2<f32>(_e53, _e54);
    let _e57 = transpose(_naga_inverse_2x2_f32(_e55));
    phi_569_ = _e47;
    if !(_e44) {
        let _e67 = ((0.5f * (abs(_e57[1].x) + abs(_e57[1].y))) / dot(_e54, _e57[1]));
        if (_e67 >= 0.5f) {
            let _e79 = T4_;
            T4_ = (_e79 * (0.5f / _e67));
            phi_568_ = vec2<f32>(0.5f, _e47.y);
        } else {
            phi_568_ = vec2<f32>((_e46.x + (_e67 * _e37)), _e47.y);
        }
        let _e82 = phi_568_;
        let _e91 = ((0.5f * (abs(_e57[0].x) + abs(_e57[0].y))) / dot(_e53, _e57[0]));
        if (_e91 >= 0.5f) {
            let _e105 = T4_;
            T4_ = (_e105 * (0.5f / _e91));
            phi_570_ = vec2<f32>(_e82.x, 0.5f);
        } else {
            let _e94 = IC_1[3u];
            phi_570_ = vec2<f32>(_e82.x, (_e82.y + (_e91 * _e94)));
        }
        let _e108 = phi_570_;
        phi_569_ = _e108;
    }
    let _e110 = phi_569_;
    X1_ = _e110;
    let _e112 = OB_1;
    let _e114 = ((_e55 * _e110) + _e112.xy);
    phi_571_ = _e114;
    if _e44 {
        let _e116 = (_e57 * _e46.zw);
        phi_571_ = (_e114 + ((_e116 * ((abs(_e116.x) + abs(_e116.y)) / dot(_e116, _e116))) * 0.5f));
    }
    let _e128 = phi_571_;
    if fh {
        let _e129 = SB_1;
        let _e134 = vec2<f32>(_e129.x, _e129.y);
        let _e135 = vec2<f32>(_e129.z, _e129.w);
        switch bitcast<i32>(0u) {
            default: {
                let _e141 = (abs(_e134) + abs(_e135));
                let _e143 = (_e141.x != 0f);
                phi_483_ = _e143;
                if _e143 {
                    phi_483_ = (_e141.y != 0f);
                }
                let _e147 = phi_483_;
                if _e147 {
                    let _e151 = ((mat2x2<f32>(_e134, _e135) * _e128) + _e112.zw);
                    let _e152 = -(_e151);
                    let _e158 = (vec2<f32>(1f, 1f) / _e141).xyxy;
                    phi_572_ = (((vec4<f32>(_e151.x, _e151.y, _e152.x, _e152.y) * _e158) + _e158) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_572_ = _e112.zwzw;
                    break;
                }
            }
        }
        let _e163 = phi_572_;
        M0_ = _e163;
    }
    let _e165 = IB_1[0u];
    H1_ = bitcast<f32>(_e165);
    let _e168 = IB_1[1u];
    x3_ = _e168;
    let _e170 = IB_1[2u];
    A1_ = _e170;
    let _e172 = m.jf;
    let _e174 = m.kf;
    unnamed.gl_Position = vec4<f32>(((_e128.x * _e172) - 1f), ((_e128.y * _e174) - sign(_e174)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) IC: vec4<f32>, @location(2) XB: vec4<f32>, @location(4) OB: vec4<f32>, @location(3) SB: vec4<f32>, @location(5) IB: vec4<u32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    IC_1 = IC;
    XB_1 = XB;
    OB_1 = OB;
    SB_1 = SB;
    IB_1 = IB;
    main_1();
    let _e24 = T4_;
    let _e25 = X1_;
    let _e26 = M0_;
    let _e27 = H1_;
    let _e28 = x3_;
    let _e29 = A1_;
    let _e30 = unnamed.gl_Position;
    return VertexOutput(_e24, _e25, _e26, _e27, _e28, _e29, _e30);
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
