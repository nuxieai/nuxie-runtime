struct UB {
    Rc: f32,
    Ud: f32,
    cg: f32,
    dg: f32,
    B6_: u32,
    Y9_: u32,
    Of: u32,
    Pf: u32,
    k8_: vec4<i32>,
    Mh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Qh: f32,
    U4_: u32,
    c3_: f32,
    Wd: f32,
    If: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Jh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct VertexOutput {
    @location(4) @interpolate(flat, either) member: vec2<f32>,
    @location(6) @interpolate(flat, either) member_1: f32,
    @location(0) member_2: vec4<f32>,
    @location(9) member_3: vec3<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override li: bool = true;
@id(2) override ni: bool = true;
@id(8) override ti: bool = true;

var<private> gl_VertexIndex_1: i32;
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(5)
var AD: texture_2d<u32>;
@group(0) @binding(2)
var LB: texture_2d<u32>;
@group(0) @binding(3)
var XC: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> l1_: vec2<f32>;
var<private> Q0_: f32;
@group(0) @binding(4)
var JB: texture_2d<f32>;
var<private> a1_: vec4<f32>;
var<private> r1_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var xa: sampler;

fn main_1() {
    var phi_943_: i32;
    var phi_945_: vec4<u32>;
    var phi_946_: vec4<u32>;
    var phi_948_: vec2<f32>;
    var phi_949_: u32;
    var phi_950_: f32;
    var phi_951_: f32;
    var phi_964_: f32;
    var phi_962_: vec4<f32>;
    var phi_963_: vec4<f32>;
    var phi_608_: bool;

    let _e56 = gl_VertexIndex_1;
    let _e58 = ((_e56 & 1073741824i) != 0i);
    let _e61 = (_e56 & 536870911i);
    let _e62 = select(4i, 5i, _e58);
    let _e68 = (_e61 & ((1i << bitcast<u32>(_e62)) - 1i));
    let _e69 = select(8i, 17i, _e58);
    let _e70 = !(_e58);
    let _e72 = (_e70 && (_e68 == 9i));
    let _e73 = select(_e68, 0i, _e72);
    let _e75 = min(_e73, (_e69 - 1i));
    let _e77 = (((_e61 >> bitcast<u32>(_e62)) * _e69) + _e75);
    let _e82 = textureLoad(TB, vec2<i32>((_e77 & 2047i), (_e77 >> bitcast<u32>(11i))), 0i);
    let _e86 = (max((_e82.w & 65535u), 1u) - 1u);
    let _e93 = textureLoad(AD, vec2<i32>(bitcast<i32>((_e86 & 255u)), bitcast<i32>((_e86 >> bitcast<u32>(8i)))), 0i);
    let _e97 = (_e93.z & 65535u);
    let _e99 = (_e97 * 4u);
    let _e106 = textureLoad(LB, vec2<i32>(bitcast<i32>((_e99 & 255u)), bitcast<i32>((_e99 >> bitcast<u32>(8i)))), 0i);
    let _e107 = bitcast<vec4<f32>>(_e106);
    let _e115 = (_e99 + 1u);
    let _e122 = textureLoad(LB, vec2<i32>(bitcast<i32>((_e115 & 255u)), bitcast<i32>((_e115 >> bitcast<u32>(8i)))), 0i);
    phi_943_ = _e73;
    if ((((_e82.w & 8388608u) != 0u) && _e70) && !(_e72)) {
        phi_943_ = (_e73 - 1i);
    }
    let _e132 = phi_943_;
    phi_946_ = _e82;
    if (_e132 != _e75) {
        let _e135 = ((_e77 + _e132) - _e75);
        let _e140 = textureLoad(TB, vec2<i32>((_e135 & 2047i), (_e135 >> bitcast<u32>(11i))), 0i);
        if ((_e140.w & 8454143u) != (_e82.w & 8454143u)) {
            let _e145 = bitcast<i32>(_e93.w);
            let _e150 = textureLoad(TB, vec2<i32>((_e145 & 2047i), (_e145 >> bitcast<u32>(11i))), 0i);
            phi_945_ = _e150;
        } else {
            phi_945_ = _e140;
        }
        let _e152 = phi_945_;
        phi_946_ = _e152;
    }
    let _e154 = phi_946_;
    if _e72 {
        phi_948_ = bitcast<vec2<f32>>(_e93.xy);
    } else {
        phi_948_ = bitcast<vec2<f32>>(_e154.xy);
    }
    let _e158 = phi_948_;
    let _e160 = ((mat2x2<f32>(vec2<f32>(_e107.x, _e107.y), vec2<f32>(_e107.z, _e107.w)) * _e158) + bitcast<vec2<f32>>(_e122.xy));
    let _e167 = textureLoad(XC, vec2<i32>(bitcast<i32>((_e93.z & 255u)), bitcast<i32>((_e97 >> bitcast<u32>(8i)))), 0i);
    let _e169 = (_e167.x & 15u);
    if li {
        let _e170 = (_e169 == 0u);
        if _e170 {
            phi_949_ = _e167.y;
        } else {
            phi_949_ = _e167.x;
        }
        let _e173 = phi_949_;
        let _e175 = (_e173 >> bitcast<u32>(16i));
        let _e177 = j.U4_;
        if (_e175 == 0u) {
            phi_950_ = 0f;
        } else {
            phi_950_ = unpack2x16float(((_e175 + 1023u) * _e177)).x;
        }
        let _e184 = phi_950_;
        phi_951_ = _e184;
        if _e170 {
            phi_951_ = -(_e184);
        }
        let _e187 = phi_951_;
        l1_[0u] = _e187;
    }
    if ni {
        Q0_ = f32(((_e167.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e169 == 1u) {
        a1_ = unpack4x8unorm(_e167.y);
    } else {
        if (li && (_e169 == 0u)) {
            let _e199 = (_e167.x >> bitcast<u32>(16i));
            let _e201 = j.U4_;
            if (_e199 == 0u) {
                phi_964_ = 0f;
            } else {
                phi_964_ = unpack2x16float(((_e199 + 1023u) * _e201)).x;
            }
            let _e208 = phi_964_;
            l1_[1u] = _e208;
        } else {
            let _e210 = (_e97 * 8u);
            let _e217 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e210 & 255u)), bitcast<i32>((_e210 >> bitcast<u32>(8i)))), 0i);
            let _e225 = (_e210 + 1u);
            let _e232 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e225 & 255u)), bitcast<i32>((_e225 >> bitcast<u32>(8i)))), 0i);
            let _e241 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e167.y));
            let _e243 = ((mat2x2<f32>(vec2<f32>(_e217.x, _e217.y), vec2<f32>(_e217.z, _e217.w)) * _e160) + _e232.xy);
            if (_e232.z > 0.9f) {
                phi_962_ = vec4<f32>(_e241.x, _e241.y, 2f, _e241.w);
            } else {
                phi_962_ = vec4<f32>(_e241.x, _e241.y, _e232.w, _e241.w);
            }
            let _e258 = phi_962_;
            if (f32(_e169) == 2f) {
                let _e265 = vec4<f32>(_e243.x, _e258.y, _e258.z, _e258.w);
                phi_963_ = vec4<f32>(_e265.x, 0f, _e265.z, _e265.w);
            } else {
                let _e277 = vec4<f32>(_e258.x, _e258.y, -(_e258.z), _e258.w);
                let _e283 = vec4<f32>(_e243.x, _e277.y, _e277.z, _e277.w);
                phi_963_ = vec4<f32>(_e283.x, _e243.y, _e283.z, _e283.w);
            }
            let _e291 = phi_963_;
            a1_ = _e291;
            let _e293 = a1_[3u];
            a1_[3u] = -(_e293);
        }
    }
    if ((_e56 & 536870912i) != 0i) {
        a1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    phi_608_ = ti;
    if ti {
        phi_608_ = ((_e167.x & 2048u) != 0u);
    }
    let _e298 = phi_608_;
    if _e298 {
        let _e299 = (_e97 * 8u);
        let _e300 = (_e299 + 4u);
        let _e307 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e300 & 255u)), bitcast<i32>((_e300 >> bitcast<u32>(8i)))), 0i);
        let _e315 = (_e299 + 5u);
        let _e322 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e315 & 255u)), bitcast<i32>((_e315 >> bitcast<u32>(8i)))), 0i);
        let _e325 = ((mat2x2<f32>(vec2<f32>(_e307.x, _e307.y), vec2<f32>(_e307.z, _e307.w)) * _e160) + _e322.xy);
        r1_ = vec3<f32>(_e325.x, _e325.y, (1f + _e322.z));
    } else {
        r1_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e332 = j.cg;
    let _e334 = j.dg;
    let _e342 = vec4<f32>(((_e160.x * _e332) - 1f), ((_e160.y * _e334) - sign(_e334)), 0f, 1f);
    let _e343 = (_e99 + 2u);
    let _e350 = textureLoad(LB, vec2<i32>(bitcast<i32>((_e343 & 255u)), bitcast<i32>((_e343 >> bitcast<u32>(8i)))), 0i);
    unnamed.gl_Position = vec4<f32>(_e342.x, _e342.y, ((f32(((_e350.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e342.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e9 = l1_;
    let _e10 = Q0_;
    let _e11 = a1_;
    let _e12 = r1_;
    let _e13 = unnamed.gl_Position;
    return VertexOutput(_e9, _e10, _e11, _e12, _e13);
}
