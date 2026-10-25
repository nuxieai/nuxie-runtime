struct li {
    v2_: array<vec4<u32>>,
}

struct og {
    v2_: array<vec2<u32>>,
}

struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
    P8_: f32,
    Q8_: f32,
}

struct pg {
    v2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct mi {
    v2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(1) @interpolate(flat, either) member: f32,
    @location(3) @interpolate(flat, either) member_1: f32,
    @location(4) @interpolate(flat, either) member_2: vec2<f32>,
    @location(6) @interpolate(flat, either) member_3: f32,
    @location(5) member_4: vec4<f32>,
    @location(0) member_5: vec4<f32>,
    @location(9) member_6: vec3<f32>,
    @location(7) @interpolate(flat, either) member_7: vec2<u32>,
    @location(8) member_8: vec2<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override fj: bool = true;
@id(2) override hj: bool = true;
@id(1) override gj: bool = true;
@id(4) override jj: bool = true;

@group(0) @binding(2)
var<storage> KB: li;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> p1_: f32;
@group(0) @binding(3)
var<storage> XC: og;
var<private> G0_: f32;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> k2_: vec2<f32>;
var<private> Q0_: f32;
@group(0) @binding(4)
var<storage> JB: pg;
var<private> W0_: vec4<f32>;
var<private> P0_: vec4<f32>;
var<private> V0_: vec3<f32>;
var<private> A3_: vec2<u32>;
var<private> N4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var VB: texture_2d<u32>;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(0) @binding(5)
var<storage> DD: mi;
@group(3) @binding(9)
var db: sampler;

fn main_1() {
    var phi_934_: f32;
    var phi_935_: u32;
    var phi_936_: f32;
    var phi_937_: f32;
    var phi_752_: bool;
    var phi_938_: vec4<f32>;
    var phi_950_: f32;
    var phi_952_: f32;
    var phi_484_: bool;
    var phi_953_: f32;

    let _e57 = LB_1;
    let _e60 = (bitcast<u32>(_e57.z) & 65535u);
    let _e66 = (_e60 * 4u);
    let _e69 = KB.v2_[_e66];
    let _e70 = bitcast<vec4<f32>>(_e69);
    let _e81 = KB.v2_[(_e66 + 1u)];
    let _e85 = ((mat2x2<f32>(vec2<f32>(_e70.x, _e70.y), vec2<f32>(_e70.z, _e70.w)) * _e57.xy) + bitcast<vec2<f32>>(_e81.xy));
    p1_ = f32((bitcast<i32>(_e57.z) >> bitcast<u32>(16i)));
    let _e88 = XC.v2_[_e60];
    let _e90 = j.z6_;
    if (_e60 == 0u) {
        phi_934_ = 0f;
    } else {
        phi_934_ = unpack2x16float(((_e60 + 1023u) * _e90)).x;
    }
    let _e97 = phi_934_;
    G0_ = _e97;
    if ((_e88.x & 512u) != 0u) {
        let _e101 = G0_;
        G0_ = -(_e101);
    }
    let _e103 = (_e88.x & 15u);
    if fj {
        let _e104 = (_e103 == 0u);
        if _e104 {
            phi_935_ = _e88.y;
        } else {
            phi_935_ = _e88.x;
        }
        let _e107 = phi_935_;
        let _e109 = (_e107 >> bitcast<u32>(16i));
        if (_e109 == 0u) {
            phi_936_ = 0f;
        } else {
            phi_936_ = unpack2x16float(((_e109 + 1023u) * _e90)).x;
        }
        let _e116 = phi_936_;
        phi_937_ = _e116;
        if _e104 {
            phi_937_ = -(_e116);
        }
        let _e119 = phi_937_;
        k2_[0u] = _e119;
    }
    if hj {
        Q0_ = f32(((_e88.x >> bitcast<u32>(4i)) & 15u));
    }
    if gj {
        let _e125 = (_e60 * 8u);
        let _e129 = JB.v2_[(_e125 + 2u)];
        let _e134 = vec2<f32>(_e129.x, _e129.y);
        let _e135 = vec2<f32>(_e129.z, _e129.w);
        let _e140 = JB.v2_[(_e125 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e145 = (abs(_e134) + abs(_e135));
                let _e147 = (_e145.x != 0f);
                phi_752_ = _e147;
                if _e147 {
                    phi_752_ = (_e145.y != 0f);
                }
                let _e151 = phi_752_;
                if _e151 {
                    let _e155 = ((mat2x2<f32>(_e134, _e135) * _e85) + _e140.xy);
                    let _e156 = -(_e155);
                    let _e162 = (vec2<f32>(1f, 1f) / _e145).xyxy;
                    phi_938_ = (((vec4<f32>(_e155.x, _e155.y, _e156.x, _e156.y) * _e162) + _e162) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_938_ = _e140.xyxy;
                    break;
                }
            }
        }
        let _e167 = phi_938_;
        W0_ = _e167;
    }
    if (_e103 == 1u) {
        P0_ = unpack4x8unorm(_e88.y);
    } else {
        if (fj && (_e103 == 0u)) {
            let _e210 = (_e88.x >> bitcast<u32>(16i));
            if (_e210 == 0u) {
                phi_952_ = 0f;
            } else {
                phi_952_ = unpack2x16float(((_e210 + 1023u) * _e90)).x;
            }
            let _e217 = phi_952_;
            k2_[1u] = _e217;
        } else {
            let _e171 = (_e60 * 8u);
            let _e174 = JB.v2_[_e171];
            let _e185 = JB.v2_[(_e171 + 1u)];
            let _e190 = ((mat2x2<f32>(vec2<f32>(_e174.x, _e174.y), vec2<f32>(_e174.z, _e174.w)) * _e85) + _e185.xy);
            let _e201 = ((_e185.w + (f32(_e103) * 0.125f)) + (max(_e185.z, 0f) * 0.00024414063f));
            if (_e185.z < 0f) {
                phi_950_ = -(_e201);
            } else {
                phi_950_ = _e201;
            }
            let _e204 = phi_950_;
            P0_ = vec4<f32>(_e190.x, _e190.y, _e204, (-0.75f - round((bitcast<f32>(_e88.y) * 255f))));
        }
    }
    phi_484_ = jj;
    if jj {
        phi_484_ = ((_e88.x & 2048u) != 0u);
    }
    let _e224 = phi_484_;
    if _e224 {
        let _e225 = (_e60 * 8u);
        let _e229 = JB.v2_[(_e225 + 4u)];
        let _e240 = JB.v2_[(_e225 + 5u)];
        let _e243 = ((mat2x2<f32>(vec2<f32>(_e229.x, _e229.y), vec2<f32>(_e229.z, _e229.w)) * _e85) + _e240.xy);
        phi_953_ = (1f + _e240.z);
        if ((_e88.x & 4096u) != 0u) {
            phi_953_ = (-1f - f32(((_e88.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e254 = phi_953_;
        V0_ = vec3<f32>(_e243.x, _e243.y, _e254);
    } else {
        V0_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e259 = j.Mg;
    let _e261 = j.Ng;
    let _e273 = KB.v2_[(_e66 + 3u)];
    A3_ = _e273.xy;
    N4_ = (_e85 + bitcast<vec2<f32>>(_e273.zw));
    unnamed.gl_Position = vec4<f32>(((_e85.x * _e259) - 1f), ((_e85.y * _e261) - sign(_e261)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e16 = p1_;
    let _e17 = G0_;
    let _e18 = k2_;
    let _e19 = Q0_;
    let _e20 = W0_;
    let _e21 = P0_;
    let _e22 = V0_;
    let _e23 = A3_;
    let _e24 = N4_;
    let _e25 = unnamed.gl_Position;
    return VertexOutput(_e16, _e17, _e18, _e19, _e20, _e21, _e22, _e23, _e24, _e25);
}
