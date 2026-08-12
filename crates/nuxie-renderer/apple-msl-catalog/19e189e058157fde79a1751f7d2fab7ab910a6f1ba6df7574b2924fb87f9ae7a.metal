// language: metal2.4
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint buffer_size30;
};

struct CC {
    float ec;
    float od;
    float ff;
    float gf;
    uint m6_;
    uint Fg;
    uint Re;
    uint Se;
    metal::int4 R7_;
    metal::float2 Bg;
    metal::float2 pd;
    uint a2_;
    float Gg;
    uint Z5_;
    float P2_;
    float qd;
    uint Me;
    float z3_;
    float A3_;
    float rd;
    uint yg;
    char _pad21[8];
};
struct type_6 {
    float inner[1];
};
struct gl_PerVertex {
    metal::float4 gl_Position;
    float gl_PointSize;
    type_6 gl_ClipDistance;
    type_6 gl_CullDistance;
    char _pad4[4];
};
struct VertexOutput {
    metal::float4 member;
    metal::float4 gl_Position;
};
metal::float4 unpackFloat32x4_(uint b0, uint b1, uint b2, uint b3, uint b4, uint b5, uint b6, uint b7, uint b8, uint b9, uint b10, uint b11, uint b12, uint b13, uint b14, uint b15) {
    return metal::float4(as_type<float>(b3 << 24 | b2 << 16 | b1 << 8 | b0), as_type<float>(b7 << 24 | b6 << 16 | b5 << 8 | b4), as_type<float>(b11 << 24 | b10 << 16 | b9 << 8 | b8), as_type<float>(b15 << 24 | b14 << 16 | b13 << 8 | b12));
}

metal::float2x2 _naga_inverse_2x2_f32_(
    metal::float2x2 m
) {
    metal::float2x2 adj = {};
    adj[0].x = m[1].y;
    adj[0].y = -(m[0].y);
    adj[1].x = -(m[1].x);
    adj[1].y = m[0].x;
    float det = (m[0].x * m[1].y) - (m[1].x * m[0].y);
    metal::float2x2 _e31 = adj;
    return _e31 * (1.0 / det);
}

int naga_f2i32(float value) {
    return static_cast<int>(metal::clamp(value, -2147483600.0, 2147483500.0));
}

metal::int2 naga_neg(metal::int2 val) {
    return as_type<metal::int2>(-as_type<metal::uint2>(val));
}

int naga_neg(int val) {
    return as_type<int>(-as_type<uint>(val));
}

void main_1_(
    metal::texture2d<uint, metal::access::sample> LC,
    metal::texture2d<uint, metal::access::sample> ED,
    metal::texture2d<uint, metal::access::sample> PB,
    constant CC& n,
    thread int& gl_InstanceIndex_1_,
    thread metal::float4& UB_1_,
    thread metal::float4& VB_1_,
    thread metal::float4& O,
    thread gl_PerVertex& unnamed
) {
    float phi_2331_ = {};
    float phi_2269_ = {};
    int phi_2241_ = {};
    bool phi_1383_ = {};
    int phi_2254_ = {};
    metal::uint4 phi_2246_ = {};
    int phi_2253_ = {};
    metal::uint4 phi_2245_ = {};
    int phi_2252_ = {};
    metal::uint4 phi_2250_ = {};
    uint phi_2249_ = {};
    metal::int2 phi_2256_ = {};
    metal::uint4 phi_2257_ = {};
    float phi_2261_ = {};
    float phi_2341_ = {};
    float phi_2275_ = {};
    float phi_2340_ = {};
    float phi_2283_ = {};
    float phi_2276_ = {};
    float phi_2273_ = {};
    float phi_2287_ = {};
    float phi_2362_ = {};
    float phi_2353_ = {};
    float phi_2338_ = {};
    float phi_2286_ = {};
    float phi_2336_ = {};
    float phi_2419_ = {};
    float phi_2430_ = {};
    float phi_2422_ = {};
    float phi_2511_ = {};
    int phi_2468_ = {};
    float phi_2477_ = {};
    bool phi_1722_ = {};
    float phi_2484_ = {};
    metal::float2 phi_2500_ = {};
    metal::float2 phi_2499_ = {};
    metal::float4 phi_2521_ = {};
    metal::float2 phi_2536_ = {};
    metal::float4 phi_2520_ = {};
    metal::float4 phi_2567_ = {};
    float phi_2372_ = {};
    float phi_2371_ = {};
    float phi_2373_ = {};
    float phi_2377_ = {};
    float phi_2399_ = {};
    float phi_2397_ = {};
    metal::float4 phi_2415_ = {};
    metal::float2 phi_2565_ = {};
    metal::float4 phi_2414_ = {};
    metal::float4 phi_2569_ = {};
    metal::float4 phi_2566_ = {};
    metal::float2 phi_2562_ = {};
    metal::float2 phi_2538_ = {};
    metal::float4 phi_2609_ = {};
    metal::float2 phi_2571_ = {};
    bool phi_2570_ = {};
    uint local = {};
    metal::float4 phi_2610_ = {};
    bool local_1 = {};
    bool local_2 = {};
    bool local_3 = {};
    bool local_4 = {};
    int _e73_ = gl_InstanceIndex_1_;
    metal::float4 _e74_ = UB_1_;
    metal::float4 _e75_ = VB_1_;
    switch(as_type<int>(0u)) {
        default: {
            int _e78_ = naga_f2i32(_e74_.x);
            int _e82_ = as_type<int>(_e74_.w);
            int _e84_ = _e82_ >> as_type<uint>(2);
            int _e85_ = _e82_ & 3;
            int _e87_ = metal::min(_e78_, as_type<int>(as_type<uint>(_e84_) - as_type<uint>(1)));
            int _e89_ = as_type<int>(as_type<uint>(as_type<int>(as_type<uint>(_e73_) * as_type<uint>(_e84_))) + as_type<uint>(_e87_));
            uint clamped_lod_e88 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
            metal::uint4 _e94_ = LC.read(metal::min(metal::uint2(metal::int2(_e89_ & 2047, _e89_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e88), LC.get_height(clamped_lod_e88)) - 1), clamped_lod_e88);
            uint _e98_ = metal::max(_e94_.w & 65535u, 1u) - 1u;
            uint clamped_lod_e106 = metal::min(uint(0), ED.get_num_mip_levels() - 1);
            metal::uint4 _e105_ = ED.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e98_ & 127u), as_type<int>(_e98_ >> as_type<uint>(7)))), metal::uint2(ED.get_width(clamped_lod_e106), ED.get_height(clamped_lod_e106)) - 1), clamped_lod_e106);
            metal::float2 _e107_ = as_type<metal::float2>(_e105_.xy);
            uint _e111_ = (_e105_.z & 65535u) * 4u;
            uint clamped_lod_e124 = metal::min(uint(0), PB.get_num_mip_levels() - 1);
            metal::uint4 _e118_ = PB.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e111_ & 127u), as_type<int>(_e111_ >> as_type<uint>(7)))), metal::uint2(PB.get_width(clamped_lod_e124), PB.get_height(clamped_lod_e124)) - 1), clamped_lod_e124);
            metal::float4 _e119_ = as_type<metal::float4>(_e118_);
            metal::float2x2 _e126_ = metal::float2x2(metal::float2(_e119_.x, _e119_.y), metal::float2(_e119_.z, _e119_.w));
            uint _e127_ = _e111_ + 1u;
            uint clamped_lod_e145 = metal::min(uint(0), PB.get_num_mip_levels() - 1);
            metal::uint4 _e134_ = PB.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e127_ & 127u), as_type<int>(_e127_ >> as_type<uint>(7)))), metal::uint2(PB.get_width(clamped_lod_e145), PB.get_height(clamped_lod_e145)) - 1), clamped_lod_e145);
            float _e138_ = as_type<float>(_e134_.z);
            float _e140_ = as_type<float>(_e134_.w);
            uint _e141_ = _e94_.w & 8388608u;
            phi_2331_ = _e74_.z;
            phi_2269_ = _e74_.y;
            phi_2241_ = _e78_;
            local = _e111_;
            if (_e141_ != 0u) {
                phi_2331_ = _e75_.z;
                phi_2269_ = _e75_.y;
                phi_2241_ = naga_f2i32(_e75_.x);
            }
            float _e148_ = phi_2331_;
            float _e150_ = phi_2269_;
            int _e152_ = phi_2241_;
            phi_2252_ = _e89_;
            phi_2250_ = _e94_;
            phi_2249_ = _e94_.w;
            if (_e152_ != _e87_) {
                int _e155_ = as_type<int>(as_type<uint>(as_type<int>(as_type<uint>(_e89_) + as_type<uint>(_e152_))) - as_type<uint>(_e87_));
                uint clamped_lod_e176 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e160_ = LC.read(metal::min(metal::uint2(metal::int2(_e155_ & 2047, _e155_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e176), LC.get_height(clamped_lod_e176)) - 1), clamped_lod_e176);
                if ((_e160_.w & 8454143u) != (_e94_.w & 8454143u)) {
                    bool _e165_ = _e138_ == 0.0;
                    phi_1383_ = _e165_;
                    if (!(_e165_)) {
                        phi_1383_ = _e107_.x != 0.0;
                    }
                    bool _e170_ = phi_1383_;
                    phi_2254_ = _e89_;
                    phi_2246_ = _e94_;
                    if (_e170_) {
                        int _e171_ = as_type<int>(_e105_.w);
                        uint clamped_lod_e201 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                        metal::uint4 _e176_ = LC.read(metal::min(metal::uint2(metal::int2(_e171_ & 2047, _e171_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e201), LC.get_height(clamped_lod_e201)) - 1), clamped_lod_e201);
                        phi_2254_ = _e171_;
                        phi_2246_ = _e176_;
                    }
                    int _e178_ = phi_2254_;
                    metal::uint4 _e180_ = phi_2246_;
                    phi_2253_ = _e178_;
                    phi_2245_ = _e180_;
                } else {
                    phi_2253_ = _e155_;
                    phi_2245_ = _e160_;
                }
                int _e182_ = phi_2253_;
                metal::uint4 _e184_ = phi_2245_;
                phi_2252_ = _e182_;
                phi_2250_ = _e184_;
                phi_2249_ = (_e184_.w & 4286578687u) | _e141_;
            }
            int _e189_ = phi_2252_;
            metal::uint4 _e191_ = phi_2250_;
            uint _e193_ = phi_2249_;
            uint _e194_ = _e193_ & 469762048u;
            if (_e194_ == 67108864u) {
                local_1 = _e85_ == 0;
            } else {
                local_1 = false;
            }
            bool _e197_ = local_1;
            if (_e197_) {
                float _e200_ = static_cast<float>(_e191_.z & 65535u);
                float _e203_ = static_cast<float>(_e191_.z >> as_type<uint>(16));
                metal::int2 _e209_ = metal::int2(naga_f2i32(-1.0 - _e200_), naga_f2i32((_e203_ - _e200_) + 1.0));
                phi_2256_ = _e209_;
                if ((_e193_ & 8388608u) != 0u) {
                    phi_2256_ = naga_neg(_e209_);
                }
                metal::int2 _e214_ = phi_2256_;
                int _e216_ = as_type<int>(as_type<uint>(_e189_) + as_type<uint>(_e214_.x));
                uint clamped_lod_e256 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e221_ = LC.read(metal::min(metal::uint2(metal::int2(_e216_ & 2047, _e216_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e256), LC.get_height(clamped_lod_e256)) - 1), clamped_lod_e256);
                int _e223_ = as_type<int>(as_type<uint>(_e189_) + as_type<uint>(_e214_.y));
                uint clamped_lod_e267 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e228_ = LC.read(metal::min(metal::uint2(metal::int2(_e223_ & 2047, _e223_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e267), LC.get_height(clamped_lod_e267)) - 1), clamped_lod_e267);
                phi_2257_ = _e228_;
                if ((_e228_.w & 8454143u) != (_e221_.w & 8454143u)) {
                    int _e234_ = as_type<int>(_e105_.w);
                    uint clamped_lod_e285 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                    metal::uint4 _e239_ = LC.read(metal::min(metal::uint2(metal::int2(_e234_ & 2047, _e234_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e285), LC.get_height(clamped_lod_e285)) - 1), clamped_lod_e285);
                    phi_2257_ = _e239_;
                }
                metal::uint4 _e241_ = phi_2257_;
                float _e243_ = as_type<float>(_e221_.z);
                float _e245_ = as_type<float>(_e241_.z);
                float _e246_ = _e245_ - _e243_;
                phi_2261_ = _e246_;
                if (metal::abs(_e246_) > 3.1415927) {
                    phi_2261_ = _e246_ - (6.2831855 * metal::sign(_e246_));
                }
                float _e253_ = phi_2261_;
                float _e254_ = _e203_ + -2.0;
                float _e260_ = metal::clamp(metal::rint((metal::abs(_e253_) * 0.31830987) * _e254_), 1.0, _e203_ + -3.0);
                float _e261_ = _e254_ - _e260_;
                if (_e200_ <= _e261_) {
                    phi_2341_ = _e150_;
                    if (_e200_ == _e261_) {
                        phi_2341_ = -(_e150_);
                    }
                    float _e270_ = phi_2341_;
                    phi_2340_ = _e270_;
                    phi_2283_ = -(((3.1415927 * metal::sign(_e253_)) - _e253_));
                    phi_2276_ = _e261_;
                    phi_2273_ = _e200_;
                } else {
                    bool _e272_ = _e200_ == (_e261_ + 1.0);
                    if (_e272_) {
                        phi_2275_ = 0.0;
                    } else {
                        phi_2275_ = _e200_ - (_e261_ + 2.0);
                    }
                    float _e276_ = phi_2275_;
                    phi_2340_ = _e272_ ? 0.0 : _e150_;
                    phi_2283_ = _e253_;
                    phi_2276_ = _e272_ ? 0.0 : _e260_;
                    phi_2273_ = _e276_;
                }
                float _e280_ = phi_2340_;
                float _e282_ = phi_2283_;
                float _e284_ = phi_2276_;
                float _e286_ = phi_2273_;
                if (_e286_ == _e284_) {
                    phi_2287_ = _e245_;
                } else {
                    phi_2287_ = _e243_ + (_e282_ * (_e286_ / _e284_));
                }
                float _e292_ = phi_2287_;
                phi_2362_ = _e243_;
                phi_2353_ = _e282_;
                phi_2338_ = _e280_;
                phi_2286_ = _e292_;
            } else {
                phi_2362_ = float {};
                phi_2353_ = float {};
                phi_2338_ = _e150_;
                phi_2286_ = as_type<float>(_e191_.z);
            }
            float _e296_ = phi_2362_;
            float _e298_ = phi_2353_;
            float _e300_ = phi_2338_;
            float _e302_ = phi_2286_;
            metal::float2 _e306_ = metal::float2(metal::sin(_e302_), -(metal::cos(_e302_)));
            metal::float2 _e308_ = as_type<metal::float2>(_e191_.xy);
            phi_2336_ = _e140_;
            if (_e140_ != 0.0) {
                phi_2336_ = metal::max(_e140_, 1.0 / metal::length(_e126_ * _e306_));
            }
            float _e315_ = phi_2336_;
            if (_e138_ != 0.0) {
                float _e319_ = _e300_ * metal::sign(metal::determinant(_e126_));
                bool _e321_ = (_e193_ & 1048576u) != 0u;
                phi_2419_ = _e319_;
                if (_e321_) {
                    phi_2419_ = metal::min(_e319_, 0.0);
                }
                float _e324_ = phi_2419_;
                phi_2430_ = _e324_;
                if ((_e193_ & 524288u) != 0u) {
                    phi_2430_ = metal::max(_e324_, 0.0);
                }
                float _e329_ = phi_2430_;
                bool _e330_ = _e315_ != 0.0;
                if (_e330_) {
                    phi_2422_ = _e315_;
                } else {
                    metal::float2 _e331_ = _e126_ * _e306_;
                    phi_2422_ = ((metal::abs(_e331_.x) + metal::abs(_e331_.y)) * (1.0 / metal::dot(_e331_, _e331_))) * 0.5;
                }
                float _e342_ = phi_2422_;
                if (_e342_ > _e138_) {
                    local_2 = _e315_ == 0.0;
                } else {
                    local_2 = false;
                }
                bool _e345_ = local_2;
                phi_2511_ = 1.0;
                if (_e345_) {
                    phi_2511_ = _e138_ / _e342_;
                }
                float _e348_ = phi_2511_;
                float _e349_ = _e345_ ? _e342_ : _e138_;
                float _e350_ = _e349_ + _e342_;
                metal::float2 _e351_ = _e306_ * _e350_;
                float _e352_ = _e329_ * _e350_;
                metal::float2 _e359_ = ((metal::float2(_e352_, -(_e352_)) + metal::float2(_e349_)) * (0.5 / _e342_)) + metal::float2(0.5, 0.5);
                metal::float4 _e362_ = metal::float4(_e359_.x, _e359_.y, 0.0, 0.0);
                phi_2536_ = _e351_;
                phi_2520_ = _e362_;
                if (_e194_ > 134217728u) {
                    uint _e364_ = _e193_ & 4194304u;
                    int _e366_ = (_e364_ == 0u) ? -2 : 2;
                    phi_2468_ = _e366_;
                    if ((_e193_ & 8388608u) != 0u) {
                        phi_2468_ = naga_neg(_e366_);
                    }
                    int _e371_ = phi_2468_;
                    int _e372_ = as_type<int>(as_type<uint>(_e189_) + as_type<uint>(_e371_));
                    uint clamped_lod_e452 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                    metal::uint4 _e377_ = LC.read(metal::min(metal::uint2(metal::int2(_e372_ & 2047, _e372_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e452), LC.get_height(clamped_lod_e452)) - 1), clamped_lod_e452);
                    float _e381_ = metal::abs(as_type<float>(_e377_.z) - _e302_);
                    phi_2477_ = _e381_;
                    if (_e381_ > 3.1415927) {
                        phi_2477_ = 6.2831855 - _e381_;
                    }
                    float _e385_ = phi_2477_;
                    float _e390_ = (_e385_ * (((_e364_ != 0u) == _e321_) ? -0.5 : 0.5)) + _e302_;
                    metal::float2 _e394_ = metal::float2(metal::sin(_e390_), -(metal::cos(_e390_)));
                    metal::float2 _e395_ = _e126_ * _e394_;
                    float _e403_ = (metal::abs(_e395_.x) + metal::abs(_e395_.y)) * (1.0 / metal::dot(_e395_, _e395_));
                    float _e405_ = metal::cos(_e385_ * 0.5);
                    bool _e406_ = _e194_ == 335544320u;
                    phi_1722_ = _e406_;
                    if (!(_e406_)) {
                        if (_e194_ == 268435456u) {
                            local_3 = _e405_ >= 0.25;
                        } else {
                            local_3 = false;
                        }
                        bool _e497 = local_3;
                        phi_1722_ = _e497;
                    }
                    bool _e412_ = phi_1722_;
                    if (_e412_) {
                        phi_2484_ = _e349_ * (1.0 / metal::max(_e405_, ((_e193_ & 33554432u) != 0u) ? 1.0 : 0.25));
                    } else {
                        phi_2484_ = (_e349_ * _e405_) + (_e403_ * 0.5);
                    }
                    float _e423_ = phi_2484_;
                    float _e425_ = _e423_ + (_e403_ * 0.5);
                    phi_2499_ = _e351_;
                    if ((_e193_ & 2097152u) != 0u) {
                        if (_e350_ <= ((_e425_ * _e405_) + (_e342_ * 0.125))) {
                            phi_2500_ = _e394_ * (_e350_ * (1.0 / _e405_));
                        } else {
                            metal::float2 _e435_ = _e394_ * _e425_;
                            metal::float2x2 _e536 = _naga_inverse_2x2_f32_(metal::float2x2(_e351_, _e435_));
                            phi_2500_ = metal::float2(metal::dot(_e351_, _e351_), metal::dot(_e435_, _e435_)) * _e536;
                        }
                        metal::float2 _e443_ = phi_2500_;
                        phi_2499_ = _e443_;
                    }
                    metal::float2 _e445_ = phi_2499_;
                    float _e450_ = (_e425_ - metal::dot(_e445_ * metal::abs(_e329_), _e394_)) / _e403_;
                    if (_e321_) {
                        phi_2521_ = metal::float4(_e362_.x, _e450_, _e362_.z, _e362_.w);
                    } else {
                        phi_2521_ = metal::float4(_e450_, _e362_.y, _e362_.z, _e362_.w);
                    }
                    metal::float4 _e462_ = phi_2521_;
                    phi_2536_ = _e445_;
                    phi_2520_ = _e462_;
                }
                metal::float2 _e464_ = phi_2536_;
                metal::float4 _e466_ = phi_2520_;
                metal::float2 _e468_ = _e466_.xy * _e348_;
                metal::float4 _e474_ = metal::float4(_e468_.x, _e466_.y, _e466_.z, _e466_.w);
                metal::float4 _e481_ = metal::float4(_e474_.x, metal::max(_e468_.y, 0.0001), _e474_.z, _e474_.w);
                phi_2567_ = _e481_;
                if (_e330_) {
                    phi_2567_ = metal::float4(-2.0 - _e468_.x, _e481_.y, _e481_.z, _e481_.w);
                }
                metal::float4 _e489_ = phi_2567_;
                if (_e85_ != 0) {
                    phi_2609_ = _e489_;
                    phi_2571_ = metal::float2 {};
                    phi_2570_ = false;
                    break;
                }
                phi_2566_ = _e489_;
                phi_2562_ = _e126_ * (_e464_ * _e329_);
                phi_2538_ = _e308_;
            } else {
                metal::float4 _e493_ = metal::float4(_e148_, -1.0, 0.0, 0.0);
                if (_e315_ != 0.0) {
                    metal::float4 _e499_ = metal::float4(_e493_.x, -2.0, _e493_.z, _e493_.w);
                    metal::float4 _e504_ = metal::float4(_e499_.x, _e499_.y, 1000000.0, _e499_.w);
                    phi_2415_ = metal::float4(_e504_.x, _e504_.y, _e504_.z, _e148_);
                    if (_e197_) {
                        phi_2372_ = _e298_;
                        phi_2371_ = _e296_;
                        if (_e298_ < 0.0) {
                            phi_2372_ = -(_e298_);
                            phi_2371_ = _e296_ + _e298_;
                        }
                        float _e514_ = phi_2372_;
                        float _e516_ = phi_2371_;
                        float _e518_ = (_e302_ - _e516_) + 1.5707964;
                        float _e524_ = metal::clamp((_e518_ - (metal::floor(_e518_ / 6.2831855) * 6.2831855)) - 1.5707964, 0.0, _e514_);
                        phi_2373_ = _e524_;
                        if (_e524_ > (_e514_ * 0.5)) {
                            phi_2373_ = _e514_ - _e524_;
                        }
                        float _e529_ = phi_2373_;
                        metal::float2 _e536_ = (metal::float2(1.0, 1.0) - (metal::float2(metal::sin(_e529_), metal::cos(_e529_)) * metal::abs(_e300_))) * 0.5;
                        if (metal::abs(_e514_ - 1.5707964) < 0.001) {
                            phi_2399_ = 0.0;
                            phi_2397_ = 0.0;
                        } else {
                            float _e540_ = metal::tan(_e514_);
                            float _e545_ = metal::sign(1.5707964 - _e514_) / metal::max(metal::abs(_e540_), 0.000001);
                            if (_e545_ >= 0.0) {
                                phi_2377_ = _e536_.y - ((1.0 - _e536_.x) * _e540_);
                            } else {
                                phi_2377_ = _e536_.y + (_e536_.x * _e540_);
                            }
                            float _e557_ = phi_2377_;
                            phi_2399_ = _e557_;
                            phi_2397_ = _e545_;
                        }
                        float _e559_ = phi_2399_;
                        float _e561_ = phi_2397_;
                        phi_2415_ = metal::float4(metal::max(_e536_.x, 0.0) + 0.25, -2.0 - _e536_.y, _e561_, _e559_);
                    }
                    metal::float4 _e569_ = phi_2415_;
                    phi_2565_ = _e126_ * (_e306_ * (_e300_ * _e315_));
                    phi_2414_ = _e569_;
                } else {
                    metal::float2x2 _e683 = _naga_inverse_2x2_f32_(_e126_);
                    phi_2565_ = metal::sign((_e306_ * _e300_) * _e683) * 0.5;
                    phi_2414_ = _e493_;
                }
                metal::float2 _e579_ = phi_2565_;
                metal::float4 _e581_ = phi_2414_;
                phi_2569_ = _e581_;
                if (((_e193_ & 8388608u) != 0u) != ((_e193_ & 16777216u) != 0u)) {
                    phi_2569_ = _e581_ * metal::float4(-1.0, 1.0, 1.0, 1.0);
                }
                metal::float4 _e589_ = phi_2569_;
                if ((_e193_ & 2147483648u) != 0u) {
                    local_4 = _e85_ != 1;
                } else {
                    local_4 = false;
                }
                bool _e715 = local_4;
                if (_e715) {
                    phi_2609_ = _e589_;
                    phi_2571_ = metal::float2 {};
                    phi_2570_ = false;
                    break;
                }
                phi_2566_ = _e589_;
                phi_2562_ = _e579_;
                phi_2538_ = metal::select(_e308_, _e107_, metal::bool2(_e85_ == 2));
            }
            metal::float4 _e598_ = phi_2566_;
            metal::float2 _e600_ = phi_2562_;
            metal::float2 _e602_ = phi_2538_;
            uint _e608_ = n.yg;
            metal::float2 _e611_ = metal::select(_e598_.xy, metal::float2(1.0, -1.0), metal::bool2(_e608_ != 0u));
            metal::float4 _e617_ = metal::float4(_e611_.x, _e598_.y, _e598_.z, _e598_.w);
            phi_2609_ = metal::float4(_e617_.x, _e611_.y, _e617_.z, _e617_.w);
            phi_2571_ = ((_e126_ * _e602_) + _e600_) + as_type<metal::float2>(_e134_.xy);
            phi_2570_ = true;
            break;
        }
    }
    metal::float4 _e625_ = phi_2609_;
    metal::float2 _e627_ = phi_2571_;
    bool _e629_ = phi_2570_;
    O = _e625_;
    if (_e629_) {
        uint _e631_ = local;
        uint _e632_ = _e631_ + 2u;
        uint clamped_lod_e769 = metal::min(uint(0), PB.get_num_mip_levels() - 1);
        metal::uint4 _e639_ = PB.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e632_ & 127u), as_type<int>(_e632_ >> as_type<uint>(7)))), metal::uint2(PB.get_width(clamped_lod_e769), PB.get_height(clamped_lod_e769)) - 1), clamped_lod_e769);
        metal::float3 _e641_ = as_type<metal::float3>(_e639_.yzw);
        metal::float2 _e645_ = (_e627_ * _e641_.x) + _e641_.yz;
        float _e648_ = n.pd.x;
        float _e651_ = n.pd.y;
        phi_2610_ = metal::float4((_e645_.x * _e648_) - 1.0, (_e645_.y * _e651_) - metal::sign(_e651_), 0.0, 1.0);
    } else {
        float _e661_ = n.P2_;
        phi_2610_ = metal::float4(_e661_);
    }
    metal::float4 _e664_ = phi_2610_;
    unnamed.gl_Position = _e664_;
    return;
}

struct main_Output {
    metal::float4 member [[user(loc0), center_perspective]];
    metal::float4 gl_Position [[position]];
};
struct vb_30_type { metal::uchar data[32]; };
vertex main_Output main_(
  uint gl_VertexIndex [[vertex_id]]
, uint gl_InstanceIndex [[instance_id]]
, metal::texture2d<uint, metal::access::sample> LC [[texture(4)]]
, metal::texture2d<uint, metal::access::sample> ED [[texture(3)]]
, metal::texture2d<uint, metal::access::sample> PB [[texture(0)]]
, constant CC& n [[buffer(0)]]
, const device vb_30_type* vb_30_in [[buffer(30)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(1)]]
) {
    metal::float4 UB = {};
    metal::float4 VB = {};
    if (gl_VertexIndex < (_buffer_sizes.buffer_size30 / 32)) {
        const vb_30_type vb_30_elem = vb_30_in[gl_VertexIndex];
        UB = unpackFloat32x4_(vb_30_elem.data[0], vb_30_elem.data[1], vb_30_elem.data[2], vb_30_elem.data[3], vb_30_elem.data[4], vb_30_elem.data[5], vb_30_elem.data[6], vb_30_elem.data[7], vb_30_elem.data[8], vb_30_elem.data[9], vb_30_elem.data[10], vb_30_elem.data[11], vb_30_elem.data[12], vb_30_elem.data[13], vb_30_elem.data[14], vb_30_elem.data[15]);
        VB = unpackFloat32x4_(vb_30_elem.data[16], vb_30_elem.data[17], vb_30_elem.data[18], vb_30_elem.data[19], vb_30_elem.data[20], vb_30_elem.data[21], vb_30_elem.data[22], vb_30_elem.data[23], vb_30_elem.data[24], vb_30_elem.data[25], vb_30_elem.data[26], vb_30_elem.data[27], vb_30_elem.data[28], vb_30_elem.data[29], vb_30_elem.data[30], vb_30_elem.data[31]);
    }
    int gl_VertexIndex_1_ = {};
    int gl_InstanceIndex_1_ = {};
    metal::float4 UB_1_ = {};
    metal::float4 VB_1_ = {};
    metal::float4 O = {};
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_6 {}, type_6 {}};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    gl_InstanceIndex_1_ = static_cast<int>(gl_InstanceIndex);
    UB_1_ = UB;
    VB_1_ = VB;
    main_1_(LC, ED, PB, n, gl_InstanceIndex_1_, UB_1_, VB_1_, O, unnamed);
    metal::float4 _e13_ = O;
    metal::float4 _e14_ = unnamed.gl_Position;
    const auto _tmp = VertexOutput {_e13_, _e14_};
    return main_Output { _tmp.member, _tmp.gl_Position };
}
