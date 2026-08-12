// language: metal3.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size2;
    uint size3;
    uint size8;
    uint size12;
    uint buffer_size30;
};

struct type_2 {
    float inner[4];
};
struct type_3 {
    float inner[1];
};
struct gl_PerVertex {
    metal::float4 gl_Position;
    float gl_PointSize;
    type_2 gl_ClipDistance;
    type_3 gl_CullDistance;
    char _pad4[8];
};
typedef metal::uint4 type_6[1];
struct cg {
    type_6 c2_;
};
struct bg {
    type_6 c2_;
};
typedef metal::uint2 type_8[1];
struct Je {
    type_8 c2_;
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
typedef metal::float4 type_12[1];
struct Ke {
    type_12 c2_;
};
struct VertexOutput {
    metal::float4 gl_Position;
    type_2 gl_ClipDistance;
    metal::float2 member;
    float member_1_;
    char _pad4[4];
    metal::float4 member_2_;
};
constant bool Yg = false;
constant bool ah = true;
constant bool Zg = true;
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
    thread gl_PerVertex& unnamed,
    metal::texture2d<uint, metal::access::sample> LC,
    device cg const& ED,
    device bg const& PB,
    thread int& gl_InstanceIndex_1_,
    thread metal::float4& UB_1_,
    thread metal::float4& VB_1_,
    device Je const& AD,
    constant CC& n,
    thread metal::float2& U1_,
    thread float& e2_,
    device Ke const& RB,
    thread metal::float4& f1_,
    constant _mslBufferSizes& _buffer_sizes
) {
    float phi_2144_ = {};
    int phi_2116_ = {};
    bool phi_1384_ = {};
    int phi_2129_ = {};
    metal::uint4 phi_2121_ = {};
    int phi_2128_ = {};
    metal::uint4 phi_2120_ = {};
    int phi_2127_ = {};
    metal::uint4 phi_2125_ = {};
    uint phi_2124_ = {};
    metal::int2 phi_2131_ = {};
    metal::uint4 phi_2132_ = {};
    float phi_2136_ = {};
    float phi_2207_ = {};
    float phi_2150_ = {};
    float phi_2206_ = {};
    float phi_2154_ = {};
    float phi_2151_ = {};
    float phi_2148_ = {};
    float phi_2158_ = {};
    float phi_2204_ = {};
    float phi_2157_ = {};
    float phi_2213_ = {};
    float phi_2210_ = {};
    float phi_2267_ = {};
    int phi_2239_ = {};
    float phi_2249_ = {};
    bool phi_1696_ = {};
    float phi_2256_ = {};
    metal::float2 phi_2277_ = {};
    metal::float2 phi_2276_ = {};
    metal::float2 phi_2275_ = {};
    metal::float2 phi_2293_ = {};
    metal::float2 phi_2278_ = {};
    uint phi_2326_ = {};
    metal::float2 phi_2297_ = {};
    bool phi_2296_ = {};
    uint local = {};
    uint phi_2355_ = {};
    float phi_2356_ = {};
    float phi_2357_ = {};
    uint local_1_ = {};
    uint local_2_ = {};
    metal::float4 phi_2359_ = {};
    float phi_2358_ = {};
    uint local_3_ = {};
    uint local_4_ = {};
    metal::float4 phi_2374_ = {};
    bool local_1 = {};
    bool local_2 = {};
    bool local_3 = {};
    bool local_4 = {};
    bool local_5 = {};
    bool local_6 = {};
    int _e73_ = gl_InstanceIndex_1_;
    metal::float4 _e74_ = UB_1_;
    metal::float4 _e75_ = VB_1_;
    switch(as_type<int>(0u)) {
        default: {
            int _e78_ = naga_f2i32(_e74_.x);
            int _e81_ = as_type<int>(_e74_.w);
            int _e83_ = _e81_ >> as_type<uint>(2);
            int _e84_ = _e81_ & 3;
            int _e86_ = metal::min(_e78_, as_type<int>(as_type<uint>(_e83_) - as_type<uint>(1)));
            int _e88_ = as_type<int>(as_type<uint>(as_type<int>(as_type<uint>(_e73_) * as_type<uint>(_e83_))) + as_type<uint>(_e86_));
            uint clamped_lod_e78 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
            metal::uint4 _e93_ = LC.read(metal::min(metal::uint2(metal::int2(_e88_ & 2047, _e88_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e78), LC.get_height(clamped_lod_e78)) - 1), clamped_lod_e78);
            metal::uint4 _e100_ = ED.c2_[metal::min(unsigned(metal::max(_e93_.w & 65535u, 1u) - 1u), (_buffer_sizes.size2 - 0 - 16) / 16)];
            metal::float2 _e102_ = as_type<metal::float2>(_e100_.xy);
            uint _e104_ = _e100_.z & 65535u;
            uint _e106_ = _e104_ * 4u;
            metal::uint4 _e109_ = PB.c2_[metal::min(unsigned(_e106_), (_buffer_sizes.size3 - 0 - 16) / 16)];
            metal::float4 _e110_ = as_type<metal::float4>(_e109_);
            metal::float2x2 _e117_ = metal::float2x2(metal::float2(_e110_.x, _e110_.y), metal::float2(_e110_.z, _e110_.w));
            uint _e118_ = _e106_ + 1u;
            metal::uint4 _e121_ = PB.c2_[metal::min(unsigned(_e118_), (_buffer_sizes.size3 - 0 - 16) / 16)];
            float _e125_ = as_type<float>(_e121_.z);
            float _e127_ = as_type<float>(_e121_.w);
            uint _e128_ = _e93_.w & 8388608u;
            phi_2144_ = _e74_.y;
            phi_2116_ = _e78_;
            local = _e104_;
            local_1_ = _e106_;
            local_2_ = _e106_;
            local_3_ = _e106_;
            local_4_ = _e118_;
            if (_e128_ != 0u) {
                phi_2144_ = _e75_.y;
                phi_2116_ = naga_f2i32(_e75_.x);
            }
            float _e134_ = phi_2144_;
            int _e136_ = phi_2116_;
            phi_2127_ = _e88_;
            phi_2125_ = _e93_;
            phi_2124_ = _e93_.w;
            if (_e136_ != _e86_) {
                int _e139_ = as_type<int>(as_type<uint>(as_type<int>(as_type<uint>(_e88_) + as_type<uint>(_e136_))) - as_type<uint>(_e86_));
                uint clamped_lod_e142 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e144_ = LC.read(metal::min(metal::uint2(metal::int2(_e139_ & 2047, _e139_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e142), LC.get_height(clamped_lod_e142)) - 1), clamped_lod_e142);
                if ((_e144_.w & 8454143u) != (_e93_.w & 8454143u)) {
                    bool _e149_ = _e125_ == 0.0;
                    phi_1384_ = _e149_;
                    if (!(_e149_)) {
                        phi_1384_ = _e102_.x != 0.0;
                    }
                    bool _e154_ = phi_1384_;
                    phi_2129_ = _e88_;
                    phi_2121_ = _e93_;
                    if (_e154_) {
                        int _e155_ = as_type<int>(_e100_.w);
                        uint clamped_lod_e167 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                        metal::uint4 _e160_ = LC.read(metal::min(metal::uint2(metal::int2(_e155_ & 2047, _e155_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e167), LC.get_height(clamped_lod_e167)) - 1), clamped_lod_e167);
                        phi_2129_ = _e155_;
                        phi_2121_ = _e160_;
                    }
                    int _e162_ = phi_2129_;
                    metal::uint4 _e164_ = phi_2121_;
                    phi_2128_ = _e162_;
                    phi_2120_ = _e164_;
                } else {
                    phi_2128_ = _e139_;
                    phi_2120_ = _e144_;
                }
                int _e166_ = phi_2128_;
                metal::uint4 _e168_ = phi_2120_;
                phi_2127_ = _e166_;
                phi_2125_ = _e168_;
                phi_2124_ = (_e168_.w & 4286578687u) | _e128_;
            }
            int _e173_ = phi_2127_;
            metal::uint4 _e175_ = phi_2125_;
            uint _e177_ = phi_2124_;
            uint _e178_ = _e177_ & 469762048u;
            if (_e178_ == 67108864u) {
                local_1 = _e84_ == 0;
            } else {
                local_1 = false;
            }
            bool _e188 = local_1;
            if (_e188) {
                float _e184_ = static_cast<float>(_e175_.z & 65535u);
                float _e187_ = static_cast<float>(_e175_.z >> as_type<uint>(16));
                metal::int2 _e193_ = metal::int2(naga_f2i32(-1.0 - _e184_), naga_f2i32((_e187_ - _e184_) + 1.0));
                phi_2131_ = _e193_;
                if ((_e177_ & 8388608u) != 0u) {
                    phi_2131_ = naga_neg(_e193_);
                }
                metal::int2 _e198_ = phi_2131_;
                int _e200_ = as_type<int>(as_type<uint>(_e173_) + as_type<uint>(_e198_.x));
                uint clamped_lod_e222 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e205_ = LC.read(metal::min(metal::uint2(metal::int2(_e200_ & 2047, _e200_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e222), LC.get_height(clamped_lod_e222)) - 1), clamped_lod_e222);
                int _e207_ = as_type<int>(as_type<uint>(_e173_) + as_type<uint>(_e198_.y));
                uint clamped_lod_e233 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e212_ = LC.read(metal::min(metal::uint2(metal::int2(_e207_ & 2047, _e207_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e233), LC.get_height(clamped_lod_e233)) - 1), clamped_lod_e233);
                phi_2132_ = _e212_;
                if ((_e212_.w & 8454143u) != (_e205_.w & 8454143u)) {
                    int _e218_ = as_type<int>(_e100_.w);
                    uint clamped_lod_e251 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                    metal::uint4 _e223_ = LC.read(metal::min(metal::uint2(metal::int2(_e218_ & 2047, _e218_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e251), LC.get_height(clamped_lod_e251)) - 1), clamped_lod_e251);
                    phi_2132_ = _e223_;
                }
                metal::uint4 _e225_ = phi_2132_;
                float _e227_ = as_type<float>(_e205_.z);
                float _e229_ = as_type<float>(_e225_.z);
                float _e230_ = _e229_ - _e227_;
                phi_2136_ = _e230_;
                if (metal::abs(_e230_) > 3.1415927) {
                    phi_2136_ = _e230_ - (6.2831855 * metal::sign(_e230_));
                }
                float _e237_ = phi_2136_;
                float _e238_ = _e187_ + -2.0;
                float _e244_ = metal::clamp(metal::rint((metal::abs(_e237_) * 0.31830987) * _e238_), 1.0, _e187_ + -3.0);
                float _e245_ = _e238_ - _e244_;
                if (_e184_ <= _e245_) {
                    phi_2207_ = _e134_;
                    if (_e184_ == _e245_) {
                        phi_2207_ = -(_e134_);
                    }
                    float _e254_ = phi_2207_;
                    phi_2206_ = _e254_;
                    phi_2154_ = -(((3.1415927 * metal::sign(_e237_)) - _e237_));
                    phi_2151_ = _e245_;
                    phi_2148_ = _e184_;
                } else {
                    bool _e256_ = _e184_ == (_e245_ + 1.0);
                    if (_e256_) {
                        phi_2150_ = 0.0;
                    } else {
                        phi_2150_ = _e184_ - (_e245_ + 2.0);
                    }
                    float _e260_ = phi_2150_;
                    phi_2206_ = _e256_ ? 0.0 : _e134_;
                    phi_2154_ = _e237_;
                    phi_2151_ = _e256_ ? 0.0 : _e244_;
                    phi_2148_ = _e260_;
                }
                float _e264_ = phi_2206_;
                float _e266_ = phi_2154_;
                float _e268_ = phi_2151_;
                float _e270_ = phi_2148_;
                if (_e270_ == _e268_) {
                    phi_2158_ = _e229_;
                } else {
                    phi_2158_ = _e227_ + (_e266_ * (_e270_ / _e268_));
                }
                float _e276_ = phi_2158_;
                phi_2204_ = _e264_;
                phi_2157_ = _e276_;
            } else {
                phi_2204_ = _e134_;
                phi_2157_ = as_type<float>(_e175_.z);
            }
            float _e280_ = phi_2204_;
            float _e282_ = phi_2157_;
            metal::float2 _e286_ = metal::float2(metal::sin(_e282_), -(metal::cos(_e282_)));
            metal::float2 _e288_ = as_type<metal::float2>(_e175_.xy);
            phi_2213_ = _e127_;
            if (_e127_ != 0.0) {
                phi_2213_ = metal::max(_e127_, 1.0 / metal::length(_e117_ * _e286_));
            }
            float _e295_ = phi_2213_;
            if (_e125_ != 0.0) {
                float _e299_ = _e280_ * metal::sign(metal::determinant(_e117_));
                bool _e301_ = (_e177_ & 1048576u) != 0u;
                phi_2210_ = _e299_;
                if (_e301_) {
                    phi_2210_ = metal::min(_e299_, 0.0);
                }
                float _e304_ = phi_2210_;
                phi_2267_ = _e304_;
                if ((_e177_ & 524288u) != 0u) {
                    phi_2267_ = metal::max(_e304_, 0.0);
                }
                float _e309_ = phi_2267_;
                float _e311_ = (_e295_ != 0.0) ? _e295_ : 0.0;
                if (_e311_ > _e125_) {
                    local_2 = _e295_ == 0.0;
                } else {
                    local_2 = false;
                }
                bool _e355 = local_2;
                float _e315_ = _e355 ? _e311_ : _e125_;
                float _e316_ = _e315_ + _e311_;
                metal::float2 _e317_ = _e286_ * _e316_;
                phi_2275_ = _e317_;
                if (_e178_ > 134217728u) {
                    uint _e319_ = _e177_ & 4194304u;
                    int _e321_ = (_e319_ == 0u) ? -2 : 2;
                    phi_2239_ = _e321_;
                    if ((_e177_ & 8388608u) != 0u) {
                        phi_2239_ = naga_neg(_e321_);
                    }
                    int _e326_ = phi_2239_;
                    int _e327_ = as_type<int>(as_type<uint>(_e173_) + as_type<uint>(_e326_));
                    uint clamped_lod_e383 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                    metal::uint4 _e332_ = LC.read(metal::min(metal::uint2(metal::int2(_e327_ & 2047, _e327_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e383), LC.get_height(clamped_lod_e383)) - 1), clamped_lod_e383);
                    float _e336_ = metal::abs(as_type<float>(_e332_.z) - _e282_);
                    phi_2249_ = _e336_;
                    if (_e336_ > 3.1415927) {
                        phi_2249_ = 6.2831855 - _e336_;
                    }
                    float _e340_ = phi_2249_;
                    float _e345_ = (_e340_ * (((_e319_ != 0u) == _e301_) ? -0.5 : 0.5)) + _e282_;
                    metal::float2 _e349_ = metal::float2(metal::sin(_e345_), -(metal::cos(_e345_)));
                    metal::float2 _e350_ = _e117_ * _e349_;
                    float _e360_ = metal::cos(_e340_ * 0.5);
                    bool _e361_ = _e178_ == 335544320u;
                    phi_1696_ = _e361_;
                    if (!(_e361_)) {
                        if (_e178_ == 268435456u) {
                            local_3 = _e360_ >= 0.25;
                        } else {
                            local_3 = false;
                        }
                        bool _e419 = local_3;
                        phi_1696_ = _e419;
                    }
                    bool _e367_ = phi_1696_;
                    if (_e367_) {
                        phi_2256_ = _e315_ * (1.0 / metal::max(_e360_, ((_e177_ & 33554432u) != 0u) ? 1.0 : 0.25));
                    } else {
                        phi_2256_ = (_e315_ * _e360_) + (((metal::abs(_e350_.x) + metal::abs(_e350_.y)) * (1.0 / metal::dot(_e350_, _e350_))) * 0.5);
                    }
                    float _e378_ = phi_2256_;
                    phi_2276_ = _e317_;
                    if ((_e177_ & 2097152u) != 0u) {
                        if (_e316_ <= ((_e378_ * _e360_) + (_e311_ * 0.125))) {
                            phi_2277_ = _e349_ * (_e316_ * (1.0 / _e360_));
                        } else {
                            metal::float2 _e388_ = _e349_ * _e378_;
                            metal::float2x2 _e464 = _naga_inverse_2x2_f32_(metal::float2x2(_e317_, _e388_));
                            phi_2277_ = metal::float2(metal::dot(_e317_, _e317_), metal::dot(_e388_, _e388_)) * _e464;
                        }
                        metal::float2 _e396_ = phi_2277_;
                        phi_2276_ = _e396_;
                    }
                    metal::float2 _e398_ = phi_2276_;
                    phi_2275_ = _e398_;
                }
                metal::float2 _e400_ = phi_2275_;
                if (_e84_ != 0) {
                    phi_2326_ = uint {};
                    phi_2297_ = metal::float2 {};
                    phi_2296_ = false;
                    break;
                }
                phi_2293_ = _e117_ * (_e400_ * _e309_);
                phi_2278_ = _e288_;
            } else {
                if ((_e177_ & 2147483648u) != 0u) {
                    local_4 = _e84_ != 1;
                } else {
                    local_4 = false;
                }
                bool _e485 = local_4;
                if (_e485) {
                    phi_2326_ = uint {};
                    phi_2297_ = metal::float2 {};
                    phi_2296_ = false;
                    break;
                }
                phi_2293_ = metal::float2(0.0, 0.0);
                phi_2278_ = metal::select(_e288_, _e102_, metal::bool2(_e84_ == 2));
            }
            metal::float2 _e412_ = phi_2293_;
            metal::float2 _e414_ = phi_2278_;
            metal::uint4 _e421_ = PB.c2_[metal::min(unsigned(_e106_ + 2u), (_buffer_sizes.size3 - 0 - 16) / 16)];
            phi_2326_ = _e421_.x;
            phi_2297_ = ((_e117_ * _e414_) + _e412_) + as_type<metal::float2>(_e121_.xy);
            phi_2296_ = true;
            break;
        }
    }
    uint _e424_ = phi_2326_;
    metal::float2 _e426_ = phi_2297_;
    bool _e428_ = phi_2296_;
    uint _e431_ = local;
    metal::uint2 _e433_ = AD.c2_[metal::min(unsigned(_e431_), (_buffer_sizes.size8 - 0 - 8) / 8)];
    uint _e435_ = _e433_.x & 15u;
    if (Yg) {
        bool _e436_ = _e435_ == 0u;
        if (_e436_) {
            phi_2355_ = _e433_.y;
        } else {
            phi_2355_ = _e433_.x;
        }
        uint _e439_ = phi_2355_;
        uint _e441_ = _e439_ >> as_type<uint>(16);
        uint _e443_ = n.Z5_;
        if (_e441_ == 0u) {
            phi_2356_ = 0.0;
        } else {
            phi_2356_ = float2(as_type<half2>(((_e441_ + 1023u) * _e443_))).x;
        }
        float _e450_ = phi_2356_;
        phi_2357_ = _e450_;
        if (_e436_) {
            phi_2357_ = -(_e450_);
        }
        float _e453_ = phi_2357_;
        U1_.x = _e453_;
    }
    if (ah) {
        e2_ = static_cast<float>((_e433_.x >> as_type<uint>(4)) & 15u);
    }
    if (Zg) {
        uint _e460_ = local_1_;
        metal::float4 _e464_ = RB.c2_[metal::min(unsigned(_e460_ + 2u), (_buffer_sizes.size12 - 0 - 16) / 16)];
        uint _e473_ = local_2_;
        metal::float4 _e477_ = RB.c2_[metal::min(unsigned(_e473_ + 3u), (_buffer_sizes.size12 - 0 - 16) / 16)];
        if (metal::any(_e464_ != metal::float4(0.0, 0.0, 0.0, 0.0))) {
            metal::float2 _e482_ = (metal::float2x2(metal::float2(_e464_.x, _e464_.y), metal::float2(_e464_.z, _e464_.w)) * _e426_) + _e477_.xy;
            unnamed.gl_ClipDistance.inner[0] = _e482_.x + 1.0;
            unnamed.gl_ClipDistance.inner[1] = _e482_.y + 1.0;
            unnamed.gl_ClipDistance.inner[2] = 1.0 - _e482_.x;
            unnamed.gl_ClipDistance.inner[3] = 1.0 - _e482_.y;
        } else {
            float _e498_ = _e477_.x - 0.5;
            unnamed.gl_ClipDistance.inner[3] = _e498_;
            unnamed.gl_ClipDistance.inner[2] = _e498_;
            unnamed.gl_ClipDistance.inner[1] = _e498_;
            unnamed.gl_ClipDistance.inner[0] = _e498_;
        }
    }
    if (_e435_ == 1u) {
        metal::float4 _e509_ = metal::unpack_unorm4x8_to_float(_e433_.y);
        if (ah) {
            phi_2359_ = _e509_;
        } else {
            metal::float3 _e512_ = _e509_.xyz * _e509_.w;
            metal::float4 _e518_ = metal::float4(_e512_.x, _e509_.y, _e509_.z, _e509_.w);
            metal::float4 _e524_ = metal::float4(_e518_.x, _e512_.y, _e518_.z, _e518_.w);
            phi_2359_ = metal::float4(_e524_.x, _e524_.y, _e512_.z, _e524_.w);
        }
        metal::float4 _e532_ = phi_2359_;
        f1_ = _e532_;
    } else {
        if (Yg) {
            local_5 = _e435_ == 0u;
        } else {
            local_5 = false;
        }
        bool _e658 = local_5;
        if (_e658) {
            uint _e536_ = _e433_.x >> as_type<uint>(16);
            uint _e538_ = n.Z5_;
            if (_e536_ == 0u) {
                phi_2358_ = 0.0;
            } else {
                phi_2358_ = float2(as_type<half2>(((_e536_ + 1023u) * _e538_))).x;
            }
            float _e545_ = phi_2358_;
            U1_.y = _e545_;
        } else {
            uint _e549_ = local_3_;
            metal::float4 _e551_ = RB.c2_[metal::min(unsigned(_e549_), (_buffer_sizes.size12 - 0 - 16) / 16)];
            uint _e561_ = local_4_;
            metal::float4 _e563_ = RB.c2_[metal::min(unsigned(_e561_), (_buffer_sizes.size12 - 0 - 16) / 16)];
            metal::float2 _e566_ = (metal::float2x2(metal::float2(_e551_.x, _e551_.y), metal::float2(_e551_.z, _e551_.w)) * _e426_) + _e563_.xy;
            bool _e567_ = _e435_ == 2u;
            if (!(_e567_)) {
                local_6 = _e435_ == 3u;
            } else {
                local_6 = true;
            }
            bool _e705 = local_6;
            if (_e705) {
                f1_.w = -(as_type<float>(_e433_.y));
                if (_e563_.z > 0.9) {
                    f1_.z = 2.0;
                } else {
                    f1_.z = _e563_.w;
                }
                if (_e567_) {
                    f1_.y = 0.0;
                    f1_.x = _e566_.x;
                } else {
                    float _e583_ = f1_.z;
                    f1_.z = -(_e583_);
                    f1_.x = _e566_.x;
                    f1_.y = _e566_.y;
                }
            } else {
                f1_ = metal::float4(_e566_.x, _e566_.y, as_type<float>(_e433_.y), -2.0 - _e563_.z);
            }
        }
    }
    if (_e428_) {
        float _e597_ = n.ff;
        float _e599_ = n.gf;
        metal::float4 _e607_ = metal::float4((_e426_.x * _e597_) - 1.0, (_e426_.y * _e599_) - metal::sign(_e599_), 0.0, 1.0);
        phi_2374_ = metal::float4(_e607_.x, _e607_.y, 1.0 - (static_cast<float>(_e424_) * 0.000061035156), _e607_.w);
    } else {
        float _e617_ = n.P2_;
        phi_2374_ = metal::float4(_e617_);
    }
    metal::float4 _e620_ = phi_2374_;
    unnamed.gl_Position = _e620_;
    return;
}

struct main_Output {
    metal::float4 gl_Position [[position]];
    float gl_ClipDistance [[clip_distance]] [4];
    metal::float2 member [[user(loc4), flat]];
    float member_1_ [[user(loc6), flat]];
    metal::float4 member_2_ [[user(loc0), center_perspective]];
};
struct vb_30_type { metal::uchar data[32]; };
vertex main_Output main_(
  uint gl_VertexIndex [[vertex_id]]
, uint gl_InstanceIndex [[instance_id]]
, metal::texture2d<uint, metal::access::sample> LC [[texture(0)]]
, device cg const& ED [[buffer(4)]]
, device bg const& PB [[buffer(1)]]
, device Je const& AD [[buffer(2)]]
, constant CC& n [[buffer(0)]]
, device Ke const& RB [[buffer(3)]]
, const device vb_30_type* vb_30_in [[buffer(30)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(5)]]
) {
    metal::float4 UB = {};
    metal::float4 VB = {};
    if (gl_VertexIndex < (_buffer_sizes.buffer_size30 / 32)) {
        const vb_30_type vb_30_elem = vb_30_in[gl_VertexIndex];
        UB = unpackFloat32x4_(vb_30_elem.data[0], vb_30_elem.data[1], vb_30_elem.data[2], vb_30_elem.data[3], vb_30_elem.data[4], vb_30_elem.data[5], vb_30_elem.data[6], vb_30_elem.data[7], vb_30_elem.data[8], vb_30_elem.data[9], vb_30_elem.data[10], vb_30_elem.data[11], vb_30_elem.data[12], vb_30_elem.data[13], vb_30_elem.data[14], vb_30_elem.data[15]);
        VB = unpackFloat32x4_(vb_30_elem.data[16], vb_30_elem.data[17], vb_30_elem.data[18], vb_30_elem.data[19], vb_30_elem.data[20], vb_30_elem.data[21], vb_30_elem.data[22], vb_30_elem.data[23], vb_30_elem.data[24], vb_30_elem.data[25], vb_30_elem.data[26], vb_30_elem.data[27], vb_30_elem.data[28], vb_30_elem.data[29], vb_30_elem.data[30], vb_30_elem.data[31]);
    }
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_2 {}, type_3 {}};
    int gl_VertexIndex_1_ = {};
    int gl_InstanceIndex_1_ = {};
    metal::float4 UB_1_ = {};
    metal::float4 VB_1_ = {};
    metal::float2 U1_ = {};
    float e2_ = {};
    metal::float4 f1_ = {};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    gl_InstanceIndex_1_ = static_cast<int>(gl_InstanceIndex);
    UB_1_ = UB;
    VB_1_ = VB;
    main_1_(unnamed, LC, ED, PB, gl_InstanceIndex_1_, UB_1_, VB_1_, AD, n, U1_, e2_, RB, f1_, _buffer_sizes);
    metal::float4 _e16_ = unnamed.gl_Position;
    type_2 _e17_ = unnamed.gl_ClipDistance;
    metal::float2 _e18_ = U1_;
    float _e19_ = e2_;
    metal::float4 _e20_ = f1_;
    const auto _tmp = VertexOutput {_e16_, _e17_, _e18_, _e19_, {}, _e20_};
    return main_Output { _tmp.gl_Position, {_tmp.gl_ClipDistance.inner[0],_tmp.gl_ClipDistance.inner[1],_tmp.gl_ClipDistance.inner[2],_tmp.gl_ClipDistance.inner[3]}, _tmp.member, _tmp.member_1_, _tmp.member_2_ };
}
