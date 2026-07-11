struct LC {
    r9_: vec4<f32>,
    c2_: vec2<f32>,
    x4_: f32,
    ki: f32,
    k2_: vec4<f32>,
    D2_: vec2<f32>,
    V0_: u32,
    n2_: u32,
    Z6_: u32,
}

struct NB {
    Ub: f32,
    dd: f32,
    Xe: f32,
    Ye: f32,
    q5_: u32,
    ug: u32,
    Je: u32,
    Ke: u32,
    U7_: vec4<i32>,
    rg: vec2<f32>,
    ed: vec2<f32>,
    W1_: u32,
    vg: f32,
    Y5_: u32,
    P2_: f32,
    fd: f32,
    Ee: u32,
    y3_: f32,
    z3_: f32,
    gd: f32,
    og: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Rf {
    X1_: array<vec4<u32>>,
}

struct Be {
    X1_: array<vec2<u32>>,
}

struct Ce {
    X1_: array<vec4<f32>>,
}

struct Sf {
    X1_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @location(1) member_1: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(1) override Ng: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
@group(0) @binding(2) 
var<uniform> A0_: LC;
var<private> GC_1: vec2<f32>;
var<private> U0_: vec2<f32>;
var<private> HC_1: vec2<f32>;
var<private> N0_: vec4<f32>;
@group(0) @binding(0) 
var<uniform> k: NB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(8) 
var DC: texture_2d<u32>;
@group(0) @binding(10) 
var QC: texture_2d<f32>;
@group(0) @binding(3) 
var<storage> MB: Rf;
@group(0) @binding(4) 
var<storage> TC: Be;
@group(0) @binding(5) 
var<storage> PB: Ce;
@group(0) @binding(6) 
var<storage> XC: Sf;
@group(3) @binding(10) 
var T9_: sampler;

fn main_1() {
    var phi_288_: bool;
    var phi_341_: vec4<f32>;

    let _e31 = A0_.r9_;
    let _e39 = GC_1;
    let _e42 = A0_.c2_;
    let _e43 = ((mat2x2<f32>(vec2<f32>(_e31.x, _e31.y), vec2<f32>(_e31.z, _e31.w)) * _e39) + _e42);
    let _e44 = HC_1;
    U0_ = _e44;
    if Ng {
        let _e46 = A0_.k2_;
        let _e51 = vec2<f32>(_e46.x, _e46.y);
        let _e52 = vec2<f32>(_e46.z, _e46.w);
        let _e55 = A0_.D2_;
        switch bitcast<i32>(0u) {
            default: {
                let _e59 = (abs(_e51) + abs(_e52));
                let _e61 = (_e59.x != 0f);
                phi_288_ = _e61;
                if _e61 {
                    phi_288_ = (_e59.y != 0f);
                }
                let _e65 = phi_288_;
                if _e65 {
                    let _e69 = ((mat2x2<f32>(_e51, _e52) * _e43) + _e55);
                    let _e70 = -(_e69);
                    let _e76 = (vec2<f32>(1f, 1f) / _e59).xyxy;
                    phi_341_ = (((vec4<f32>(_e69.x, _e69.y, _e70.x, _e70.y) * _e76) + _e76) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_341_ = _e55.xyxy;
                    break;
                }
            }
        }
        let _e81 = phi_341_;
        N0_ = _e81;
    }
    let _e83 = k.Xe;
    let _e85 = k.Ye;
    unnamed.gl_Position = vec4<f32>(((_e43.x * _e83) - 1f), ((_e43.y * _e85) - sign(_e85)), 0f, 1f);
    return;
}

@vertex 
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) GC: vec2<f32>, @location(1) HC: vec2<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    GC_1 = GC;
    HC_1 = HC;
    main_1();
    let _e14 = U0_;
    let _e15 = N0_;
    let _e16 = unnamed.gl_Position;
    return VertexOutput(_e14, _e15, _e16);
}
