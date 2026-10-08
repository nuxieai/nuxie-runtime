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

struct VertexOutput {
    @location(1) member: vec2<f32>,
    @location(4) @interpolate(flat, either) member_1: f32,
    @location(6) @interpolate(flat, either) member_2: f32,
    @location(0) member_3: vec4<f32>,
    @location(9) member_4: vec3<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override Yi: bool = true;
@id(2) override aj: bool = true;
@id(8) override gj: bool = true;

@group(0) @binding(2)
var KB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> T2_: vec2<f32>;
@group(0) @binding(3)
var WC: texture_2d<u32>;
var<private> e4_: f32;
var<private> Q0_: f32;
@group(0) @binding(4)
var JB: texture_2d<f32>;
var<private> P0_: vec4<f32>;
var<private> V0_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(0) @binding(5)
var CD: texture_2d<u32>;
@group(3) @binding(9)
var Va: sampler;

fn main_1() {
    var phi_789_: u32;
    var phi_790_: f32;
    var phi_791_: f32;
    var phi_798_: vec4<f32>;
    var phi_799_: vec4<f32>;
    var phi_445_: bool;
    var phi_800_: f32;

    let _e51 = LB_1;
    let _e53 = bitcast<u32>(_e51.z);
    let _e54 = (_e53 & 65535u);
    let _e56 = ((_e54 * 4u) + 2u);
    let _e63 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e56 & 255u)), bitcast<i32>((_e56 >> bitcast<u32>(8i)))), 0i);
    let _e65 = _e51.xy;
    let _e67 = bitcast<vec3<f32>>(_e63.yzw);
    let _e73 = j.Bi;
    T2_ = (((_e65 * _e67.x) + _e67.yz) * _e73);
    let _e81 = textureLoad(WC, vec2<i32>(bitcast<i32>((_e53 & 255u)), bitcast<i32>((_e54 >> bitcast<u32>(8i)))), 0i);
    let _e83 = (_e81.x & 15u);
    if Yi {
        let _e84 = (_e83 == 0u);
        if _e84 {
            phi_789_ = _e81.y;
        } else {
            phi_789_ = _e81.x;
        }
        let _e87 = phi_789_;
        let _e89 = (_e87 >> bitcast<u32>(16i));
        let _e91 = j.p6_;
        if (_e89 == 0u) {
            phi_790_ = 0f;
        } else {
            phi_790_ = unpack2x16float(((_e89 + 1023u) * _e91)).x;
        }
        let _e98 = phi_790_;
        phi_791_ = _e98;
        if _e84 {
            phi_791_ = -(_e98);
        }
        let _e101 = phi_791_;
        e4_ = _e101;
    }
    if aj {
        Q0_ = f32(((_e81.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e83 == 1u) {
        P0_ = unpack4x8unorm(_e81.y);
    } else {
        let _e107 = (_e54 * 8u);
        let _e114 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e107 & 255u)), bitcast<i32>((_e107 >> bitcast<u32>(8i)))), 0i);
        let _e122 = (_e107 + 1u);
        let _e129 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e122 & 255u)), bitcast<i32>((_e122 >> bitcast<u32>(8i)))), 0i);
        let _e138 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e81.y));
        let _e140 = ((mat2x2<f32>(vec2<f32>(_e114.x, _e114.y), vec2<f32>(_e114.z, _e114.w)) * _e65) + _e129.xy);
        if (_e129.z > 0.9f) {
            phi_798_ = vec4<f32>(_e138.x, _e138.y, 2f, _e138.w);
        } else {
            phi_798_ = vec4<f32>(_e138.x, _e138.y, _e129.w, _e138.w);
        }
        let _e155 = phi_798_;
        if (f32(_e83) == 2f) {
            let _e181 = vec4<f32>(_e140.x, _e155.y, _e155.z, _e155.w);
            phi_799_ = vec4<f32>(_e181.x, 0f, _e181.z, _e181.w);
        } else {
            let _e163 = vec4<f32>(_e155.x, _e155.y, -(_e155.z), _e155.w);
            let _e169 = vec4<f32>(_e140.x, _e163.y, _e163.z, _e163.w);
            phi_799_ = vec4<f32>(_e169.x, _e140.y, _e169.z, _e169.w);
        }
        let _e188 = phi_799_;
        P0_ = _e188;
        let _e190 = P0_[3u];
        P0_[3u] = -(_e190);
    }
    phi_445_ = gj;
    if gj {
        phi_445_ = ((_e81.x & 2048u) != 0u);
    }
    let _e197 = phi_445_;
    if _e197 {
        let _e198 = (_e54 * 8u);
        let _e199 = (_e198 + 4u);
        let _e206 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e199 & 255u)), bitcast<i32>((_e199 >> bitcast<u32>(8i)))), 0i);
        let _e214 = (_e198 + 5u);
        let _e221 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e214 & 255u)), bitcast<i32>((_e214 >> bitcast<u32>(8i)))), 0i);
        let _e224 = ((mat2x2<f32>(vec2<f32>(_e206.x, _e206.y), vec2<f32>(_e206.z, _e206.w)) * _e65) + _e221.xy);
        phi_800_ = (1f + _e221.z);
        if ((_e81.x & 4096u) != 0u) {
            phi_800_ = (-1f - f32(((_e81.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e235 = phi_800_;
        V0_ = vec3<f32>(_e224.x, _e224.y, _e235);
    } else {
        V0_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e240 = j.Gg;
    let _e242 = j.Hg;
    let _e250 = vec4<f32>(((_e51.x * _e240) - 1f), ((_e51.y * _e242) - sign(_e242)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e250.x, _e250.y, ((f32(((_e63.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e250.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e12 = T2_;
    let _e13 = e4_;
    let _e14 = Q0_;
    let _e15 = P0_;
    let _e16 = V0_;
    let _e17 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
