struct VB {
    td: f32,
    ze: f32,
    Dg: f32,
    Eg: f32,
    L6_: u32,
    va: u32,
    pg: u32,
    qg: u32,
    B8_: vec4<i32>,
    xi: vec2<f32>,
    Ae: vec2<f32>,
    r2_: u32,
    Bi: f32,
    p6_: u32,
    h3_: f32,
    Be: f32,
    jg: u32,
    E3_: f32,
    F3_: f32,
    Ce: f32,
    ui: u32,
    ua: u32,
    ad: f32,
    g7_: f32,
    Bb: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @location(2) member_1: vec3<f32>,
    @location(1) @interpolate(flat, either) member_2: f32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(15) override A6_: bool = false;
@id(2) override Wi: bool = true;
@id(8) override cj: bool = true;

var<private> gl_VertexIndex_1: i32;
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(5)
var BD: texture_2d<u32>;
@group(0) @binding(2)
var KB: texture_2d<u32>;
@group(0) @binding(4)
var JB: texture_2d<f32>;
var<private> O0_: vec4<f32>;
var<private> V0_: vec3<f32>;
@group(0) @binding(3)
var WC: texture_2d<u32>;
var<private> P0_: f32;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var Ta: sampler;

fn main_1() {
    var phi_1197_: i32;
    var phi_1237_: f32;
    var phi_1225_: f32;
    var phi_1203_: bool;
    var phi_1201_: i32;
    var phi_1200_: i32;
    var phi_1198_: i32;
    var phi_1206_: i32;
    var phi_1205_: i32;
    var phi_1209_: bool;
    var phi_1211_: vec4<u32>;
    var phi_1210_: vec4<u32>;
    var phi_1230_: u32;
    var phi_1221_: vec4<u32>;
    var phi_1232_: f32;
    var phi_1242_: f32;
    var phi_1246_: f32;
    var phi_617_: bool;
    var phi_1249_: f32;
    var phi_1260_: vec2<f32>;
    var phi_1259_: vec2<f32>;
    var phi_1266_: f32;
    var phi_1273_: vec2<f32>;
    var phi_1265_: f32;
    var phi_1256_: vec2<f32>;
    var phi_1282_: vec2<f32>;
    var phi_1222_: vec2<f32>;
    var phi_1281_: vec2<f32>;
    var phi_1320_: bool;
    var phi_1318_: vec4<f32>;
    var phi_1319_: vec4<f32>;
    var phi_872_: bool;
    var phi_1337_: u32;
    var phi_1336_: u32;

    let _e75 = gl_VertexIndex_1;
    let _e80 = ((_e75 & 536870912i) != 0i);
    let _e81 = (_e75 & 268435455i);
    if A6_ {
        let _e82 = select(5i, 6i, _e80);
        let _e88 = (_e81 & ((1i << bitcast<u32>(_e82)) - 1i));
        let _e89 = select(1i, 2i, _e80);
        let _e95 = (_e88 & ((1i << bitcast<u32>(_e89)) - 1i));
        phi_1197_ = _e95;
        if !(_e80) {
            phi_1197_ = (_e95 + 1i);
        }
        let _e99 = phi_1197_;
        phi_1237_ = select(1f, 0f, ((_e99 == 0i) || (_e99 == 3i)));
        phi_1225_ = select(1f, -1f, (_e99 < 2i));
        phi_1203_ = false;
        phi_1201_ = (_e81 >> bitcast<u32>(_e82));
        phi_1200_ = 8i;
        phi_1198_ = (_e88 >> bitcast<u32>(_e89));
    } else {
        let _e106 = select(4i, 5i, _e80);
        let _e112 = (_e81 & ((1i << bitcast<u32>(_e106)) - 1i));
        let _e116 = (!(_e80) && (_e112 == 9i));
        phi_1237_ = 1f;
        phi_1225_ = 0f;
        phi_1203_ = _e116;
        phi_1201_ = (_e81 >> bitcast<u32>(_e106));
        phi_1200_ = select(8i, 17i, _e80);
        phi_1198_ = select(_e112, 0i, _e116);
    }
    let _e119 = phi_1237_;
    let _e121 = phi_1225_;
    let _e123 = phi_1203_;
    let _e125 = phi_1201_;
    let _e127 = phi_1200_;
    let _e129 = phi_1198_;
    let _e131 = min(_e129, (_e127 - 1i));
    let _e133 = ((_e125 * _e127) + _e131);
    let _e138 = textureLoad(UB, vec2<i32>((_e133 & 2047i), (_e133 >> bitcast<u32>(11i))), 0i);
    let _e142 = (max((_e138.w & 65535u), 1u) - 1u);
    let _e149 = textureLoad(BD, vec2<i32>(bitcast<i32>((_e142 & 255u)), bitcast<i32>((_e142 >> bitcast<u32>(8i)))), 0i);
    let _e151 = bitcast<vec2<f32>>(_e149.xy);
    let _e153 = (_e149.z & 65535u);
    let _e155 = (_e153 * 4u);
    let _e162 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e155 & 255u)), bitcast<i32>((_e155 >> bitcast<u32>(8i)))), 0i);
    let _e163 = bitcast<vec4<f32>>(_e162);
    let _e170 = mat2x2<f32>(vec2<f32>(_e163.x, _e163.y), vec2<f32>(_e163.z, _e163.w));
    let _e171 = (_e155 + 1u);
    let _e178 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e171 & 255u)), bitcast<i32>((_e171 >> bitcast<u32>(8i)))), 0i);
    let _e180 = bitcast<vec2<f32>>(_e178.xy);
    let _e183 = (_e155 + 2u);
    let _e190 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e183 & 255u)), bitcast<i32>((_e183 >> bitcast<u32>(8i)))), 0i);
    let _e192 = (_e138.w & 8388608u);
    if A6_ {
        phi_1205_ = _e129;
    } else {
        phi_1206_ = _e129;
        if (((_e192 != 0u) && !(_e80)) && !(_e123)) {
            phi_1206_ = (_e129 - 1i);
        }
        let _e200 = phi_1206_;
        phi_1205_ = _e200;
    }
    let _e202 = phi_1205_;
    phi_1230_ = _e138.w;
    phi_1221_ = _e138;
    if (_e202 != _e131) {
        let _e205 = ((_e133 + _e202) - _e131);
        let _e210 = textureLoad(UB, vec2<i32>((_e205 & 2047i), (_e205 >> bitcast<u32>(11i))), 0i);
        if ((_e210.w & 8454143u) != (_e138.w & 8454143u)) {
            if A6_ {
                phi_1209_ = (_e151.x != 0f);
            } else {
                phi_1209_ = true;
            }
            let _e218 = phi_1209_;
            phi_1211_ = _e138;
            if _e218 {
                let _e219 = bitcast<i32>(_e149.w);
                let _e224 = textureLoad(UB, vec2<i32>((_e219 & 2047i), (_e219 >> bitcast<u32>(11i))), 0i);
                phi_1211_ = _e224;
            }
            let _e226 = phi_1211_;
            phi_1210_ = _e226;
        } else {
            phi_1210_ = _e210;
        }
        let _e228 = phi_1210_;
        phi_1230_ = ((_e228.w & 4286578687u) | _e192);
        phi_1221_ = _e228;
    }
    let _e233 = phi_1230_;
    let _e235 = phi_1221_;
    if A6_ {
        let _e238 = (f32(_e235.z) * 0.0000000014629181f);
        let _e242 = vec2<f32>(sin(_e238), -(cos(_e238)));
        let _e247 = (_e121 * sign(determinant(_e170)));
        let _e249 = ((_e233 & 1048576u) != 0u);
        phi_1232_ = _e247;
        if _e249 {
            phi_1232_ = min(_e247, 0f);
        }
        let _e252 = phi_1232_;
        phi_1242_ = _e252;
        if ((_e233 & 524288u) != 0u) {
            phi_1242_ = max(_e252, 0f);
        }
        let _e257 = phi_1242_;
        let _e259 = select(0f, _e257, (_e119 == 0f));
        let _e260 = (_e233 & 469762048u);
        phi_1273_ = _e242;
        phi_1265_ = _e259;
        phi_1256_ = _e242;
        if (_e260 > 134217728u) {
            let _e265 = f32((_e235.z & 65535u));
            let _e266 = (_e265 * 0.000015259022f);
            let _e270 = sqrt(max((1f - (_e266 * _e266)), 0f));
            phi_1246_ = _e270;
            if (((_e233 & 4194304u) != 0u) == _e249) {
                phi_1246_ = -(_e270);
            }
            let _e274 = phi_1246_;
            let _e279 = (mat2x2<f32>(vec2<f32>(_e266, _e274), vec2<f32>(-(_e274), _e266)) * _e242);
            let _e280 = (_e260 == 201326592u);
            phi_617_ = _e280;
            if !(_e280) {
                phi_617_ = ((_e260 != 335544320u) && (_e266 < 0.25f));
            }
            let _e286 = phi_617_;
            let _e288 = ((_e233 & 2097152u) != 0u);
            if (_e260 == 335544320u) {
                phi_1259_ = (_e242 + _e279);
            } else {
                phi_1260_ = _e242;
                if (_e288 || !(_e286)) {
                    if _e286 {
                        phi_1249_ = _e266;
                    } else {
                        phi_1249_ = (65535f / _e265);
                    }
                    let _e295 = phi_1249_;
                    phi_1260_ = (_e279 * _e295);
                }
                let _e298 = phi_1260_;
                phi_1259_ = _e298;
            }
            let _e300 = phi_1259_;
            phi_1266_ = _e259;
            if (!(_e80) && _e286) {
                phi_1266_ = (0.5f * _e257);
            }
            let _e308 = phi_1266_;
            phi_1273_ = select(_e242, _e279, vec2((_e286 || _e288)));
            phi_1265_ = _e308;
            phi_1256_ = _e300;
        }
        let _e310 = phi_1273_;
        let _e312 = phi_1265_;
        let _e314 = phi_1256_;
        let _e319 = ((_e170 * (bitcast<vec2<f32>>(_e235.xy) + (_e314 * (_e257 * bitcast<f32>(_e178.z))))) + _e180);
        phi_1282_ = _e319;
        if (_e312 != 0f) {
            phi_1282_ = (_e319 + (sign((_e310 * _naga_inverse_2x2_f32(_e170))) * _e312));
        }
        let _e327 = phi_1282_;
        phi_1281_ = _e327;
    } else {
        if _e123 {
            phi_1222_ = _e151;
        } else {
            phi_1222_ = bitcast<vec2<f32>>(_e235.xy);
        }
        let _e331 = phi_1222_;
        phi_1281_ = ((_e170 * _e331) + _e180);
    }
    let _e335 = phi_1281_;
    if ((_e75 & 268435456i) != 0i) {
        O0_ = vec4<f32>(0f, 0f, 0f, 0f);
        V0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e342 = textureLoad(WC, vec2<i32>(bitcast<i32>((_e149.z & 255u)), bitcast<i32>((_e153 >> bitcast<u32>(8i)))), 0i);
        let _e344 = (_e342.x & 15u);
        phi_1320_ = false;
        if Wi {
            let _e347 = ((_e342.x >> bitcast<u32>(4i)) & 15u);
            P0_ = f32(_e347);
            phi_1320_ = (_e347 != 0u);
        }
        let _e351 = phi_1320_;
        if (_e344 == 1u) {
            O0_ = unpack4x8unorm(_e342.y);
            if _e351 {
                let _e356 = O0_[3u];
                O0_[3u] = (_e356 * _e119);
            } else {
                let _e358 = O0_;
                O0_ = (_e358 * _e119);
            }
        } else {
            let _e360 = (_e153 * 8u);
            let _e367 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e360 & 255u)), bitcast<i32>((_e360 >> bitcast<u32>(8i)))), 0i);
            let _e375 = (_e360 + 1u);
            let _e382 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e375 & 255u)), bitcast<i32>((_e375 >> bitcast<u32>(8i)))), 0i);
            let _e384 = bitcast<f32>(_e342.y);
            let _e387 = ((mat2x2<f32>(vec2<f32>(_e367.x, _e367.y), vec2<f32>(_e367.z, _e367.w)) * _e335) + _e382.xy);
            let _e393 = vec4<f32>(_e387.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e399 = vec4<f32>(_e393.x, _e387.y, _e393.z, _e393.w);
            let _e404 = vec4<f32>(_e399.x, _e399.y, _e119, _e399.w);
            phi_1318_ = _e404;
            if (_e344 != 2u) {
                phi_1318_ = vec4<f32>(_e404.x, _e404.y, (_e119 + 2f), _e404.w);
            }
            let _e413 = phi_1318_;
            phi_1319_ = _e413;
            if (_e382.z > 0.9f) {
                phi_1319_ = vec4<f32>(_e413.x, _e413.y, -(_e413.z), _e413.w);
            }
            let _e424 = phi_1319_;
            O0_ = vec4<f32>(_e424.x, _e424.y, _e424.z, -(bitcast<f32>(((((_e344 << bitcast<u32>(28i)) | ((u32(_e384) - 1u) << bitcast<u32>(17i))) | (u32((_e382.w * 512f)) << bitcast<u32>(8i))) | u32((fract(_e384) * 256f))))));
        }
        phi_872_ = cj;
        if cj {
            phi_872_ = ((_e342.x & 2048u) != 0u);
        }
        let _e452 = phi_872_;
        if _e452 {
            let _e453 = (_e153 * 8u);
            let _e454 = (_e453 + 4u);
            let _e461 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e454 & 255u)), bitcast<i32>((_e454 >> bitcast<u32>(8i)))), 0i);
            let _e469 = (_e453 + 5u);
            let _e476 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e469 & 255u)), bitcast<i32>((_e469 >> bitcast<u32>(8i)))), 0i);
            let _e479 = ((mat2x2<f32>(vec2<f32>(_e461.x, _e461.y), vec2<f32>(_e461.z, _e461.w)) * _e335) + _e476.xy);
            V0_ = vec3<f32>(_e479.x, _e479.y, (1f + _e476.z));
        } else {
            V0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e486 = j.Dg;
    let _e488 = j.Eg;
    let _e496 = vec4<f32>(((_e335.x * _e486) - 1f), ((_e335.y * _e488) - sign(_e488)), 0f, 1f);
    if A6_ {
        let _e498 = u32((_e119 * 254f));
        phi_1337_ = _e498;
        if ((_e75 & 1073741824i) == 0i) {
            phi_1337_ = (_e498 + bitcast<u32>(1i));
        }
        let _e503 = phi_1337_;
        phi_1336_ = _e503;
    } else {
        phi_1336_ = 255u;
    }
    let _e505 = phi_1336_;
    unnamed.gl_Position = vec4<f32>(_e496.x, _e496.y, ((f32(((_e190.x << bitcast<u32>(8u)) | _e505)) * 0.000000059604645f) + 0.000000029802322f), _e496.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e8 = O0_;
    let _e9 = V0_;
    let _e10 = P0_;
    let _e11 = unnamed.gl_Position;
    return VertexOutput(_e8, _e9, _e10, _e11);
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
