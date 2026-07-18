struct CC {
    bc: f32,
    kd: f32,
    bf: f32,
    cf: f32,
    m6_: u32,
    Bg: u32,
    Ne: u32,
    Oe: u32,
    Q7_: vec4<i32>,
    xg: vec2<f32>,
    ld: vec2<f32>,
    a2_: u32,
    Cg: f32,
    Z5_: u32,
    N2_: f32,
    md: f32,
    Ie: u32,
    y3_: f32,
    z3_: f32,
    nd: f32,
    ug: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct VertexOutput {
    @location(1) member: vec2<f32>,
    @location(4) @interpolate(flat, either) member_1: f32,
    @location(6) @interpolate(flat, either) member_2: f32,
    @location(0) member_3: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override Ug: bool = true;
@id(2) override Wg: bool = true;

@group(0) @binding(2)
var PB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> m: CC;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> B2_: vec2<f32>;
@group(0) @binding(3)
var AD: texture_2d<u32>;
var<private> H3_: f32;
var<private> e2_: f32;
@group(0) @binding(4)
var RB: texture_2d<f32>;
var<private> f1_: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var LC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(5)
var ED: texture_2d<u32>;
@group(3) @binding(9)
var Z9_: sampler;

fn main_1() {
    var phi_631_: u32;
    var phi_632_: f32;
    var phi_633_: f32;
    var phi_634_: vec4<f32>;

    let _e40 = LB_1;
    let _e42 = bitcast<u32>(_e40.z);
    let _e43 = (_e42 & 65535u);
    let _e44 = (_e43 * 4u);
    let _e45 = (_e44 + 2u);
    let _e52 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e45 & 127u)), bitcast<i32>((_e45 >> bitcast<u32>(7i)))), 0i);
    let _e54 = _e40.xy;
    let _e56 = bitcast<vec3<f32>>(_e52.yzw);
    let _e62 = m.xg;
    B2_ = (((_e54 * _e56.x) + _e56.yz) * _e62);
    let _e70 = textureLoad(AD, vec2<i32>(bitcast<i32>((_e42 & 127u)), bitcast<i32>((_e43 >> bitcast<u32>(7i)))), 0i);
    let _e72 = (_e70.x & 15u);
    if Ug {
        let _e73 = (_e72 == 0u);
        if _e73 {
            phi_631_ = _e70.y;
        } else {
            phi_631_ = _e70.x;
        }
        let _e76 = phi_631_;
        let _e78 = (_e76 >> bitcast<u32>(16i));
        let _e80 = m.Z5_;
        if (_e78 == 0u) {
            phi_632_ = 0f;
        } else {
            phi_632_ = unpack2x16float(((_e78 + 1023u) * _e80)).x;
        }
        let _e87 = phi_632_;
        phi_633_ = _e87;
        if _e73 {
            phi_633_ = -(_e87);
        }
        let _e90 = phi_633_;
        H3_ = _e90;
    }
    if Wg {
        e2_ = f32(((_e70.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e72 == 1u) {
        let _e151 = unpack4x8unorm(_e70.y);
        if Wg {
            phi_634_ = _e151;
        } else {
            let _e154 = (_e151.xyz * _e151.w);
            let _e160 = vec4<f32>(_e154.x, _e151.y, _e151.z, _e151.w);
            let _e166 = vec4<f32>(_e160.x, _e154.y, _e160.z, _e160.w);
            phi_634_ = vec4<f32>(_e166.x, _e166.y, _e154.z, _e166.w);
        }
        let _e174 = phi_634_;
        f1_ = _e174;
    } else {
        let _e102 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e44 & 127u)), bitcast<i32>((_e44 >> bitcast<u32>(7i)))), 0i);
        let _e110 = (_e44 + 1u);
        let _e117 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e110 & 127u)), bitcast<i32>((_e110 >> bitcast<u32>(7i)))), 0i);
        let _e120 = ((mat2x2<f32>(vec2<f32>(_e102.x, _e102.y), vec2<f32>(_e102.z, _e102.w)) * _e54) + _e117.xy);
        let _e121 = (_e72 == 2u);
        if (_e121 || (_e72 == 3u)) {
            f1_[3u] = -(bitcast<f32>(_e70.y));
            if (_e117.z > 0.9f) {
                f1_[2u] = 2f;
            } else {
                f1_[2u] = _e117.w;
            }
            if _e121 {
                f1_[1u] = 0f;
                f1_[0u] = _e120.x;
            } else {
                let _e141 = f1_[2u];
                f1_[2u] = -(_e141);
                f1_[0u] = _e120.x;
                f1_[1u] = _e120.y;
            }
        } else {
            f1_ = vec4<f32>(_e120.x, _e120.y, bitcast<f32>(_e70.y), (-2f - _e117.z));
        }
    }
    let _e176 = m.bf;
    let _e178 = m.cf;
    let _e186 = vec4<f32>(((_e40.x * _e176) - 1f), ((_e40.y * _e178) - sign(_e178)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e186.x, _e186.y, (1f - (f32(_e52.x) * 0.000061035156f)), _e186.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e11 = B2_;
    let _e12 = H3_;
    let _e13 = e2_;
    let _e14 = f1_;
    let _e15 = unnamed.gl_Position;
    return VertexOutput(_e11, _e12, _e13, _e14, _e15);
}
