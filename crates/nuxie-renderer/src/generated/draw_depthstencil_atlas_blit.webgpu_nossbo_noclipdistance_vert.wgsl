struct BC {
    jc: f32,
    sd: f32,
    of_: f32,
    pf: f32,
    p6_: u32,
    Pg: u32,
    Ze: u32,
    af: u32,
    U7_: vec4<i32>,
    Lg: vec2<f32>,
    td: vec2<f32>,
    c2_: u32,
    Qg: f32,
    d6_: u32,
    R2_: f32,
    ud: f32,
    Ue: u32,
    A3_: f32,
    B3_: f32,
    vd: f32,
    Ig: u32,
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
    @location(9) member_4: vec3<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override jh: bool = true;
@id(2) override lh: bool = true;
@id(8) override rh: bool = true;

@group(0) @binding(2)
var PB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> m: BC;
var<private> gl_VertexIndex_1: i32;
var<private> KB_1: vec3<f32>;
var<private> D2_: vec2<f32>;
@group(0) @binding(3)
var AD: texture_2d<u32>;
var<private> J3_: f32;
var<private> f2_: f32;
@group(0) @binding(4)
var QB: texture_2d<f32>;
var<private> f1_: vec4<f32>;
var<private> A2_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(5)
var ED: texture_2d<u32>;
@group(3) @binding(9)
var Z9_: sampler;

fn main_1() {
    var phi_683_: u32;
    var phi_684_: f32;
    var phi_685_: f32;
    var phi_686_: vec4<f32>;
    var phi_422_: bool;

    let _e45 = KB_1;
    let _e47 = bitcast<u32>(_e45.z);
    let _e48 = (_e47 & 65535u);
    let _e50 = ((_e48 * 4u) + 2u);
    let _e57 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e50 & 255u)), bitcast<i32>((_e50 >> bitcast<u32>(8i)))), 0i);
    let _e59 = _e45.xy;
    let _e61 = bitcast<vec3<f32>>(_e57.yzw);
    let _e67 = m.Lg;
    D2_ = (((_e59 * _e61.x) + _e61.yz) * _e67);
    let _e75 = textureLoad(AD, vec2<i32>(bitcast<i32>((_e47 & 255u)), bitcast<i32>((_e48 >> bitcast<u32>(8i)))), 0i);
    let _e77 = (_e75.x & 15u);
    if jh {
        let _e78 = (_e77 == 0u);
        if _e78 {
            phi_683_ = _e75.y;
        } else {
            phi_683_ = _e75.x;
        }
        let _e81 = phi_683_;
        let _e83 = (_e81 >> bitcast<u32>(16i));
        let _e85 = m.d6_;
        if (_e83 == 0u) {
            phi_684_ = 0f;
        } else {
            phi_684_ = unpack2x16float(((_e83 + 1023u) * _e85)).x;
        }
        let _e92 = phi_684_;
        phi_685_ = _e92;
        if _e78 {
            phi_685_ = -(_e92);
        }
        let _e95 = phi_685_;
        J3_ = _e95;
    }
    if lh {
        f2_ = f32(((_e75.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e77 == 1u) {
        let _e148 = unpack4x8unorm(_e75.y);
        if lh {
            phi_686_ = _e148;
        } else {
            let _e151 = (_e148.xyz * _e148.w);
            let _e157 = vec4<f32>(_e151.x, _e148.y, _e148.z, _e148.w);
            let _e163 = vec4<f32>(_e157.x, _e151.y, _e157.z, _e157.w);
            phi_686_ = vec4<f32>(_e163.x, _e163.y, _e151.z, _e163.w);
        }
        let _e171 = phi_686_;
        f1_ = _e171;
    } else {
        let _e101 = (_e48 * 8u);
        let _e108 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e101 & 255u)), bitcast<i32>((_e101 >> bitcast<u32>(8i)))), 0i);
        let _e116 = (_e101 + 1u);
        let _e123 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e116 & 255u)), bitcast<i32>((_e116 >> bitcast<u32>(8i)))), 0i);
        let _e126 = ((mat2x2<f32>(vec2<f32>(_e108.x, _e108.y), vec2<f32>(_e108.z, _e108.w)) * _e59) + _e123.xy);
        f1_[3u] = -(bitcast<f32>(_e75.y));
        if (_e123.z > 0.9f) {
            f1_[2u] = 2f;
        } else {
            f1_[2u] = _e123.w;
        }
        if (_e77 == 2u) {
            f1_[1u] = 0f;
            f1_[0u] = _e126.x;
        } else {
            let _e138 = f1_[2u];
            f1_[2u] = -(_e138);
            f1_[0u] = _e126.x;
            f1_[1u] = _e126.y;
        }
    }
    phi_422_ = rh;
    if rh {
        phi_422_ = ((_e75.x & 2048u) != 0u);
    }
    let _e175 = phi_422_;
    if _e175 {
        let _e176 = (_e48 * 8u);
        let _e177 = (_e176 + 4u);
        let _e184 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e177 & 255u)), bitcast<i32>((_e177 >> bitcast<u32>(8i)))), 0i);
        let _e192 = (_e176 + 5u);
        let _e199 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e192 & 255u)), bitcast<i32>((_e192 >> bitcast<u32>(8i)))), 0i);
        let _e202 = ((mat2x2<f32>(vec2<f32>(_e184.x, _e184.y), vec2<f32>(_e184.z, _e184.w)) * _e59) + _e199.xy);
        A2_ = vec3<f32>(_e202.x, _e202.y, (1f + _e199.z));
    } else {
        A2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e209 = m.of_;
    let _e211 = m.pf;
    let _e219 = vec4<f32>(((_e45.x * _e209) - 1f), ((_e45.y * _e211) - sign(_e211)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e219.x, _e219.y, (1f - (f32(_e57.x) * 0.000061035156f)), _e219.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) KB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    KB_1 = KB;
    main_1();
    let _e12 = D2_;
    let _e13 = J3_;
    let _e14 = f2_;
    let _e15 = f1_;
    let _e16 = A2_;
    let _e17 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
