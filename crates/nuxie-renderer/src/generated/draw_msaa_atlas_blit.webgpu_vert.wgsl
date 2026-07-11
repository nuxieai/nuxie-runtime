enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct Rf {
    X1_: array<vec4<u32>>,
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
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(1) member: vec2<f32>,
    @location(4) @interpolate(flat) member_1: f32,
    @location(6) @interpolate(flat) member_2: f32,
    @location(0) member_3: vec4<f32>,
}

@id(0) override Mg: bool = true;
@id(2) override Og: bool = true;
@id(1) override Ng: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(3) 
var<storage> MB: Rf;
@group(0) @binding(0) 
var<uniform> k: NB;
var<private> gl_VertexIndex_1: i32;
var<private> KB_1: vec3<f32>;
var<private> C2_: vec2<f32>;
@group(0) @binding(4) 
var<storage> TC: Be;
var<private> I3_: f32;
var<private> Z1_: f32;
@group(0) @binding(5) 
var<storage> PB: Ce;
var<private> i1_: vec4<f32>;
@group(0) @binding(8) 
var DC: texture_2d<u32>;
@group(0) @binding(10) 
var QC: texture_2d<f32>;
@group(0) @binding(6) 
var<storage> XC: Sf;
@group(3) @binding(10) 
var T9_: sampler;

fn main_1() {
    var phi_679_: u32;
    var phi_680_: f32;
    var phi_681_: f32;
    var phi_682_: vec4<f32>;

    let _e42 = KB_1;
    let _e45 = (bitcast<u32>(_e42.z) & 65535u);
    let _e46 = (_e45 * 4u);
    let _e47 = (_e46 + 2u);
    let _e50 = MB.X1_[_e47];
    let _e52 = _e42.xy;
    let _e54 = bitcast<vec3<f32>>(_e50.yzw);
    let _e60 = k.rg;
    C2_ = (((_e52 * _e54.x) + _e54.yz) * _e60);
    let _e64 = TC.X1_[_e45];
    let _e66 = (_e64.x & 15u);
    if Mg {
        let _e67 = (_e66 == 0u);
        if _e67 {
            phi_679_ = _e64.y;
        } else {
            phi_679_ = _e64.x;
        }
        let _e70 = phi_679_;
        let _e72 = (_e70 >> bitcast<u32>(16i));
        let _e74 = k.Y5_;
        if (_e72 == 0u) {
            phi_680_ = 0f;
        } else {
            phi_680_ = unpack2x16float(((_e72 + 1023u) * _e74)).x;
        }
        let _e81 = phi_680_;
        phi_681_ = _e81;
        if _e67 {
            phi_681_ = -(_e81);
        }
        let _e84 = phi_681_;
        I3_ = _e84;
    }
    if Og {
        Z1_ = f32(((_e64.x >> bitcast<u32>(4i)) & 15u));
    }
    if Ng {
        let _e91 = PB.X1_[_e47];
        let _e102 = PB.X1_[(_e46 + 3u)];
        if any((_e91 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e117 = ((mat2x2<f32>(vec2<f32>(_e91.x, _e91.y), vec2<f32>(_e91.z, _e91.w)) * _e52) + _e102.xy);
            unnamed.gl_ClipDistance[0i] = (_e117.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e117.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e117.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e117.y);
        } else {
            let _e107 = (_e102.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e107;
            unnamed.gl_ClipDistance[2i] = _e107;
            unnamed.gl_ClipDistance[1i] = _e107;
            unnamed.gl_ClipDistance[0i] = _e107;
        }
    }
    if (_e66 == 1u) {
        let _e180 = unpack4x8unorm(_e64.y);
        if Og {
            phi_682_ = _e180;
        } else {
            let _e183 = (_e180.xyz * _e180.w);
            let _e189 = vec4<f32>(_e183.x, _e180.y, _e180.z, _e180.w);
            let _e195 = vec4<f32>(_e189.x, _e183.y, _e189.z, _e189.w);
            phi_682_ = vec4<f32>(_e195.x, _e195.y, _e183.z, _e195.w);
        }
        let _e203 = phi_682_;
        i1_ = _e203;
    } else {
        let _e135 = PB.X1_[_e46];
        let _e146 = PB.X1_[(_e46 + 1u)];
        let _e149 = ((mat2x2<f32>(vec2<f32>(_e135.x, _e135.y), vec2<f32>(_e135.z, _e135.w)) * _e52) + _e146.xy);
        let _e150 = (_e66 == 2u);
        if (_e150 || (_e66 == 3u)) {
            i1_[3u] = -(bitcast<f32>(_e64.y));
            if (_e146.z > 0.9f) {
                i1_[2u] = 2f;
            } else {
                i1_[2u] = _e146.w;
            }
            if _e150 {
                i1_[1u] = 0f;
                i1_[0u] = _e149.x;
            } else {
                let _e170 = i1_[2u];
                i1_[2u] = -(_e170);
                i1_[0u] = _e149.x;
                i1_[1u] = _e149.y;
            }
        } else {
            i1_ = vec4<f32>(_e149.x, _e149.y, bitcast<f32>(_e64.y), (-2f - _e146.z));
        }
    }
    let _e205 = k.Xe;
    let _e207 = k.Ye;
    let _e215 = vec4<f32>(((_e42.x * _e205) - 1f), ((_e42.y * _e207) - sign(_e207)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e215.x, _e215.y, (1f - (f32(_e50.x) * 0.000061035156f)), _e215.w);
    return;
}

@vertex 
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) KB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    KB_1 = KB;
    main_1();
    let _e12 = unnamed.gl_Position;
    let _e13 = unnamed.gl_ClipDistance;
    let _e14 = C2_;
    let _e15 = I3_;
    let _e16 = Z1_;
    let _e17 = i1_;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
