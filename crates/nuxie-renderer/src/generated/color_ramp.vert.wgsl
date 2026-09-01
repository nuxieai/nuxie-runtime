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

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

var<private> gl_VertexIndex_1: i32;
var<private> LC_1: vec4<u32>;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> T6_: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    var phi_239_: u32;
    var phi_240_: f32;
    var phi_243_: f32;
    var phi_242_: f32;
    var phi_247_: f32;
    var phi_246_: f32;
    var phi_244_: u32;

    let _e31 = gl_VertexIndex_1;
    let _e33 = (_e31 >> bitcast<u32>(1i));
    let _e34 = (_e33 <= 1i);
    if _e34 {
        let _e36 = LC_1[0u];
        phi_239_ = (_e36 & 65535u);
    } else {
        let _e39 = LC_1[0u];
        phi_239_ = (_e39 >> bitcast<u32>(16i));
    }
    let _e43 = phi_239_;
    let _e45 = (f32(_e43) * 0.000015258789f);
    let _e48 = select(1f, 0f, ((_e31 & 1i) == 0i));
    let _e50 = m.gc;
    phi_240_ = _e48;
    if (_e50 < 0f) {
        phi_240_ = (1f - _e48);
    }
    let _e54 = phi_240_;
    let _e56 = LC_1[1u];
    phi_242_ = _e45;
    if (((_e56 & 2147483648u) != 0u) && (_e33 == 0i)) {
        if ((_e56 & 536870912u) != 0u) {
            phi_243_ = 0f;
        } else {
            phi_243_ = (_e45 - 0.001953125f);
        }
        let _e68 = phi_243_;
        phi_242_ = _e68;
    }
    let _e70 = phi_242_;
    phi_246_ = _e70;
    if (((_e56 & 1073741824u) != 0u) && (_e33 == 3i)) {
        if ((_e56 & 536870912u) != 0u) {
            phi_247_ = 1f;
        } else {
            phi_247_ = (_e70 + 0.001953125f);
        }
        let _e79 = phi_247_;
        phi_246_ = _e79;
    }
    let _e81 = phi_246_;
    if _e34 {
        let _e83 = LC_1[2u];
        phi_244_ = _e83;
    } else {
        let _e85 = LC_1[3u];
        phi_244_ = _e85;
    }
    let _e87 = phi_244_;
    T6_ = (vec4<f32>(((vec4(_e87) >> bitcast<vec4<u32>>(vec4<u32>(16u, 8u, 0u, 24u))) & vec4<u32>(255u, 255u, 255u, 255u))) * vec4<f32>(0.003921569f, 0.003921569f, 0.003921569f, 0.003921569f));
    unnamed.gl_Position = vec4<f32>(((_e81 * 2f) - 1f), (((f32((_e56 & 536870911u)) + _e54) * _e50) - sign(_e50)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LC: vec4<u32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LC_1 = LC;
    main_1();
    let _e8 = T6_;
    let _e9 = unnamed.gl_Position;
    return VertexOutput(_e8, _e9);
}
