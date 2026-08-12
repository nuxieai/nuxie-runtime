// language: metal2.4
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size1;
    uint size2;
    uint size7;
    uint size11;
    uint buffer_size30;
};

typedef metal::uint4 type_2[1];
struct cg {
    type_2 c2_;
};
struct bg {
    type_2 c2_;
};
typedef metal::uint2 type_4[1];
struct Je {
    type_4 c2_;
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
typedef metal::float4 type_10[1];
struct Ke {
    type_10 c2_;
};
struct type_11 {
    float inner[1];
};
struct gl_PerVertex {
    metal::float4 gl_Position;
    float gl_PointSize;
    type_11 gl_ClipDistance;
    type_11 gl_CullDistance;
    char _pad4[4];
};
struct VertexOutput {
    metal::float2 member;
    float member_1_;
    char _pad2[4];
    metal::float4 member_2_;
    metal::float4 gl_Position;
};
constant bool Yg = false;
constant bool ah = true;
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
    thread gl_PerVertex& unnamed,
    constant _mslBufferSizes& _buffer_sizes
) {
    float phi_2056_ = {};
    int phi_2028_ = {};
    bool phi_1341_ = {};
    int phi_2041_ = {};
    metal::uint4 phi_2033_ = {};
    int phi_2040_ = {};
    metal::uint4 phi_2032_ = {};
    int phi_2039_ = {};
    metal::uint4 phi_2037_ = {};
    uint phi_2036_ = {};
    metal::int2 phi_2043_ = {};
    metal::uint4 phi_2044_ = {};
    float phi_2048_ = {};
    float phi_2119_ = {};
    float phi_2062_ = {};
    float phi_2118_ = {};
    float phi_2066_ = {};
    float phi_2063_ = {};
    float phi_2060_ = {};
    float phi_2070_ = {};
    float phi_2116_ = {};
    float phi_2069_ = {};
    float phi_2125_ = {};
    float phi_2122_ = {};
    float phi_2179_ = {};
    int phi_2151_ = {};
    float phi_2161_ = {};
    bool phi_1653_ = {};
    float phi_2168_ = {};
    metal::float2 phi_2189_ = {};
    metal::float2 phi_2188_ = {};
    metal::float2 phi_2187_ = {};
    metal::float2 phi_2205_ = {};
    metal::float2 phi_2190_ = {};
    uint phi_2238_ = {};
    metal::float2 phi_2209_ = {};
    bool phi_2208_ = {};
    uint local = {};
    uint phi_2267_ = {};
    float phi_2268_ = {};
    float phi_2269_ = {};
    metal::float4 phi_2271_ = {};
    float phi_2270_ = {};
    uint local_1_ = {};
    uint local_2_ = {};
    metal::float4 phi_2284_ = {};
    bool local_1 = {};
    bool local_2 = {};
    bool local_3 = {};
    bool local_4 = {};
    bool local_5 = {};
    bool local_6 = {};
    int _e71_ = gl_InstanceIndex_1_;
    metal::float4 _e72_ = UB_1_;
    metal::float4 _e73_ = VB_1_;
    switch(as_type<int>(0u)) {
        default: {
            int _e76_ = naga_f2i32(_e72_.x);
            int _e79_ = as_type<int>(_e72_.w);
            int _e81_ = _e79_ >> as_type<uint>(2);
            int _e82_ = _e79_ & 3;
            int _e84_ = metal::min(_e76_, as_type<int>(as_type<uint>(_e81_) - as_type<uint>(1)));
            int _e86_ = as_type<int>(as_type<uint>(as_type<int>(as_type<uint>(_e71_) * as_type<uint>(_e81_))) + as_type<uint>(_e84_));
            uint clamped_lod_e76 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
            metal::uint4 _e91_ = LC.read(metal::min(metal::uint2(metal::int2(_e86_ & 2047, _e86_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e76), LC.get_height(clamped_lod_e76)) - 1), clamped_lod_e76);
            metal::uint4 _e98_ = ED.c2_[metal::min(unsigned(metal::max(_e91_.w & 65535u, 1u) - 1u), (_buffer_sizes.size1 - 0 - 16) / 16)];
            metal::float2 _e100_ = as_type<metal::float2>(_e98_.xy);
            uint _e102_ = _e98_.z & 65535u;
            uint _e104_ = _e102_ * 4u;
            metal::uint4 _e107_ = PB.c2_[metal::min(unsigned(_e104_), (_buffer_sizes.size2 - 0 - 16) / 16)];
            metal::float4 _e108_ = as_type<metal::float4>(_e107_);
            metal::float2x2 _e115_ = metal::float2x2(metal::float2(_e108_.x, _e108_.y), metal::float2(_e108_.z, _e108_.w));
            uint _e116_ = _e104_ + 1u;
            metal::uint4 _e119_ = PB.c2_[metal::min(unsigned(_e116_), (_buffer_sizes.size2 - 0 - 16) / 16)];
            float _e123_ = as_type<float>(_e119_.z);
            float _e125_ = as_type<float>(_e119_.w);
            uint _e126_ = _e91_.w & 8388608u;
            phi_2056_ = _e72_.y;
            phi_2028_ = _e76_;
            local = _e102_;
            local_1_ = _e104_;
            local_2_ = _e116_;
            if (_e126_ != 0u) {
                phi_2056_ = _e73_.y;
                phi_2028_ = naga_f2i32(_e73_.x);
            }
            float _e132_ = phi_2056_;
            int _e134_ = phi_2028_;
            phi_2039_ = _e86_;
            phi_2037_ = _e91_;
            phi_2036_ = _e91_.w;
            if (_e134_ != _e84_) {
                int _e137_ = as_type<int>(as_type<uint>(as_type<int>(as_type<uint>(_e86_) + as_type<uint>(_e134_))) - as_type<uint>(_e84_));
                uint clamped_lod_e140 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e142_ = LC.read(metal::min(metal::uint2(metal::int2(_e137_ & 2047, _e137_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e140), LC.get_height(clamped_lod_e140)) - 1), clamped_lod_e140);
                if ((_e142_.w & 8454143u) != (_e91_.w & 8454143u)) {
                    bool _e147_ = _e123_ == 0.0;
                    phi_1341_ = _e147_;
                    if (!(_e147_)) {
                        phi_1341_ = _e100_.x != 0.0;
                    }
                    bool _e152_ = phi_1341_;
                    phi_2041_ = _e86_;
                    phi_2033_ = _e91_;
                    if (_e152_) {
                        int _e153_ = as_type<int>(_e98_.w);
                        uint clamped_lod_e165 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                        metal::uint4 _e158_ = LC.read(metal::min(metal::uint2(metal::int2(_e153_ & 2047, _e153_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e165), LC.get_height(clamped_lod_e165)) - 1), clamped_lod_e165);
                        phi_2041_ = _e153_;
                        phi_2033_ = _e158_;
                    }
                    int _e160_ = phi_2041_;
                    metal::uint4 _e162_ = phi_2033_;
                    phi_2040_ = _e160_;
                    phi_2032_ = _e162_;
                } else {
                    phi_2040_ = _e137_;
                    phi_2032_ = _e142_;
                }
                int _e164_ = phi_2040_;
                metal::uint4 _e166_ = phi_2032_;
                phi_2039_ = _e164_;
                phi_2037_ = _e166_;
                phi_2036_ = (_e166_.w & 4286578687u) | _e126_;
            }
            int _e171_ = phi_2039_;
            metal::uint4 _e173_ = phi_2037_;
            uint _e175_ = phi_2036_;
            uint _e176_ = _e175_ & 469762048u;
            if (_e176_ == 67108864u) {
                local_1 = _e82_ == 0;
            } else {
                local_1 = false;
            }
            bool _e186 = local_1;
            if (_e186) {
                float _e182_ = static_cast<float>(_e173_.z & 65535u);
                float _e185_ = static_cast<float>(_e173_.z >> as_type<uint>(16));
                metal::int2 _e191_ = metal::int2(naga_f2i32(-1.0 - _e182_), naga_f2i32((_e185_ - _e182_) + 1.0));
                phi_2043_ = _e191_;
                if ((_e175_ & 8388608u) != 0u) {
                    phi_2043_ = naga_neg(_e191_);
                }
                metal::int2 _e196_ = phi_2043_;
                int _e198_ = as_type<int>(as_type<uint>(_e171_) + as_type<uint>(_e196_.x));
                uint clamped_lod_e220 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e203_ = LC.read(metal::min(metal::uint2(metal::int2(_e198_ & 2047, _e198_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e220), LC.get_height(clamped_lod_e220)) - 1), clamped_lod_e220);
                int _e205_ = as_type<int>(as_type<uint>(_e171_) + as_type<uint>(_e196_.y));
                uint clamped_lod_e231 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e210_ = LC.read(metal::min(metal::uint2(metal::int2(_e205_ & 2047, _e205_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e231), LC.get_height(clamped_lod_e231)) - 1), clamped_lod_e231);
                phi_2044_ = _e210_;
                if ((_e210_.w & 8454143u) != (_e203_.w & 8454143u)) {
                    int _e216_ = as_type<int>(_e98_.w);
                    uint clamped_lod_e249 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                    metal::uint4 _e221_ = LC.read(metal::min(metal::uint2(metal::int2(_e216_ & 2047, _e216_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e249), LC.get_height(clamped_lod_e249)) - 1), clamped_lod_e249);
                    phi_2044_ = _e221_;
                }
                metal::uint4 _e223_ = phi_2044_;
                float _e225_ = as_type<float>(_e203_.z);
                float _e227_ = as_type<float>(_e223_.z);
                float _e228_ = _e227_ - _e225_;
                phi_2048_ = _e228_;
                if (metal::abs(_e228_) > 3.1415927) {
                    phi_2048_ = _e228_ - (6.2831855 * metal::sign(_e228_));
                }
                float _e235_ = phi_2048_;
                float _e236_ = _e185_ + -2.0;
                float _e242_ = metal::clamp(metal::rint((metal::abs(_e235_) * 0.31830987) * _e236_), 1.0, _e185_ + -3.0);
                float _e243_ = _e236_ - _e242_;
                if (_e182_ <= _e243_) {
                    phi_2119_ = _e132_;
                    if (_e182_ == _e243_) {
                        phi_2119_ = -(_e132_);
                    }
                    float _e252_ = phi_2119_;
                    phi_2118_ = _e252_;
                    phi_2066_ = -(((3.1415927 * metal::sign(_e235_)) - _e235_));
                    phi_2063_ = _e243_;
                    phi_2060_ = _e182_;
                } else {
                    bool _e254_ = _e182_ == (_e243_ + 1.0);
                    if (_e254_) {
                        phi_2062_ = 0.0;
                    } else {
                        phi_2062_ = _e182_ - (_e243_ + 2.0);
                    }
                    float _e258_ = phi_2062_;
                    phi_2118_ = _e254_ ? 0.0 : _e132_;
                    phi_2066_ = _e235_;
                    phi_2063_ = _e254_ ? 0.0 : _e242_;
                    phi_2060_ = _e258_;
                }
                float _e262_ = phi_2118_;
                float _e264_ = phi_2066_;
                float _e266_ = phi_2063_;
                float _e268_ = phi_2060_;
                if (_e268_ == _e266_) {
                    phi_2070_ = _e227_;
                } else {
                    phi_2070_ = _e225_ + (_e264_ * (_e268_ / _e266_));
                }
                float _e274_ = phi_2070_;
                phi_2116_ = _e262_;
                phi_2069_ = _e274_;
            } else {
                phi_2116_ = _e132_;
                phi_2069_ = as_type<float>(_e173_.z);
            }
            float _e278_ = phi_2116_;
            float _e280_ = phi_2069_;
            metal::float2 _e284_ = metal::float2(metal::sin(_e280_), -(metal::cos(_e280_)));
            metal::float2 _e286_ = as_type<metal::float2>(_e173_.xy);
            phi_2125_ = _e125_;
            if (_e125_ != 0.0) {
                phi_2125_ = metal::max(_e125_, 1.0 / metal::length(_e115_ * _e284_));
            }
            float _e293_ = phi_2125_;
            if (_e123_ != 0.0) {
                float _e297_ = _e278_ * metal::sign(metal::determinant(_e115_));
                bool _e299_ = (_e175_ & 1048576u) != 0u;
                phi_2122_ = _e297_;
                if (_e299_) {
                    phi_2122_ = metal::min(_e297_, 0.0);
                }
                float _e302_ = phi_2122_;
                phi_2179_ = _e302_;
                if ((_e175_ & 524288u) != 0u) {
                    phi_2179_ = metal::max(_e302_, 0.0);
                }
                float _e307_ = phi_2179_;
                float _e309_ = (_e293_ != 0.0) ? _e293_ : 0.0;
                if (_e309_ > _e123_) {
                    local_2 = _e293_ == 0.0;
                } else {
                    local_2 = false;
                }
                bool _e353 = local_2;
                float _e313_ = _e353 ? _e309_ : _e123_;
                float _e314_ = _e313_ + _e309_;
                metal::float2 _e315_ = _e284_ * _e314_;
                phi_2187_ = _e315_;
                if (_e176_ > 134217728u) {
                    uint _e317_ = _e175_ & 4194304u;
                    int _e319_ = (_e317_ == 0u) ? -2 : 2;
                    phi_2151_ = _e319_;
                    if ((_e175_ & 8388608u) != 0u) {
                        phi_2151_ = naga_neg(_e319_);
                    }
                    int _e324_ = phi_2151_;
                    int _e325_ = as_type<int>(as_type<uint>(_e171_) + as_type<uint>(_e324_));
                    uint clamped_lod_e381 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                    metal::uint4 _e330_ = LC.read(metal::min(metal::uint2(metal::int2(_e325_ & 2047, _e325_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e381), LC.get_height(clamped_lod_e381)) - 1), clamped_lod_e381);
                    float _e334_ = metal::abs(as_type<float>(_e330_.z) - _e280_);
                    phi_2161_ = _e334_;
                    if (_e334_ > 3.1415927) {
                        phi_2161_ = 6.2831855 - _e334_;
                    }
                    float _e338_ = phi_2161_;
                    float _e343_ = (_e338_ * (((_e317_ != 0u) == _e299_) ? -0.5 : 0.5)) + _e280_;
                    metal::float2 _e347_ = metal::float2(metal::sin(_e343_), -(metal::cos(_e343_)));
                    metal::float2 _e348_ = _e115_ * _e347_;
                    float _e358_ = metal::cos(_e338_ * 0.5);
                    bool _e359_ = _e176_ == 335544320u;
                    phi_1653_ = _e359_;
                    if (!(_e359_)) {
                        if (_e176_ == 268435456u) {
                            local_3 = _e358_ >= 0.25;
                        } else {
                            local_3 = false;
                        }
                        bool _e417 = local_3;
                        phi_1653_ = _e417;
                    }
                    bool _e365_ = phi_1653_;
                    if (_e365_) {
                        phi_2168_ = _e313_ * (1.0 / metal::max(_e358_, ((_e175_ & 33554432u) != 0u) ? 1.0 : 0.25));
                    } else {
                        phi_2168_ = (_e313_ * _e358_) + (((metal::abs(_e348_.x) + metal::abs(_e348_.y)) * (1.0 / metal::dot(_e348_, _e348_))) * 0.5);
                    }
                    float _e376_ = phi_2168_;
                    phi_2188_ = _e315_;
                    if ((_e175_ & 2097152u) != 0u) {
                        if (_e314_ <= ((_e376_ * _e358_) + (_e309_ * 0.125))) {
                            phi_2189_ = _e347_ * (_e314_ * (1.0 / _e358_));
                        } else {
                            metal::float2 _e386_ = _e347_ * _e376_;
                            metal::float2x2 _e462 = _naga_inverse_2x2_f32_(metal::float2x2(_e315_, _e386_));
                            phi_2189_ = metal::float2(metal::dot(_e315_, _e315_), metal::dot(_e386_, _e386_)) * _e462;
                        }
                        metal::float2 _e394_ = phi_2189_;
                        phi_2188_ = _e394_;
                    }
                    metal::float2 _e396_ = phi_2188_;
                    phi_2187_ = _e396_;
                }
                metal::float2 _e398_ = phi_2187_;
                if (_e82_ != 0) {
                    phi_2238_ = uint {};
                    phi_2209_ = metal::float2 {};
                    phi_2208_ = false;
                    break;
                }
                phi_2205_ = _e115_ * (_e398_ * _e307_);
                phi_2190_ = _e286_;
            } else {
                if ((_e175_ & 2147483648u) != 0u) {
                    local_4 = _e82_ != 1;
                } else {
                    local_4 = false;
                }
                bool _e483 = local_4;
                if (_e483) {
                    phi_2238_ = uint {};
                    phi_2209_ = metal::float2 {};
                    phi_2208_ = false;
                    break;
                }
                phi_2205_ = metal::float2(0.0, 0.0);
                phi_2190_ = metal::select(_e286_, _e100_, metal::bool2(_e82_ == 2));
            }
            metal::float2 _e410_ = phi_2205_;
            metal::float2 _e412_ = phi_2190_;
            metal::uint4 _e419_ = PB.c2_[metal::min(unsigned(_e104_ + 2u), (_buffer_sizes.size2 - 0 - 16) / 16)];
            phi_2238_ = _e419_.x;
            phi_2209_ = ((_e115_ * _e412_) + _e410_) + as_type<metal::float2>(_e119_.xy);
            phi_2208_ = true;
            break;
        }
    }
    uint _e422_ = phi_2238_;
    metal::float2 _e424_ = phi_2209_;
    bool _e426_ = phi_2208_;
    uint _e429_ = local;
    metal::uint2 _e431_ = AD.c2_[metal::min(unsigned(_e429_), (_buffer_sizes.size7 - 0 - 8) / 8)];
    uint _e433_ = _e431_.x & 15u;
    if (Yg) {
        bool _e434_ = _e433_ == 0u;
        if (_e434_) {
            phi_2267_ = _e431_.y;
        } else {
            phi_2267_ = _e431_.x;
        }
        uint _e437_ = phi_2267_;
        uint _e439_ = _e437_ >> as_type<uint>(16);
        uint _e441_ = n.Z5_;
        if (_e439_ == 0u) {
            phi_2268_ = 0.0;
        } else {
            phi_2268_ = float2(as_type<half2>(((_e439_ + 1023u) * _e441_))).x;
        }
        float _e448_ = phi_2268_;
        phi_2269_ = _e448_;
        if (_e434_) {
            phi_2269_ = -(_e448_);
        }
        float _e451_ = phi_2269_;
        U1_.x = _e451_;
    }
    if (ah) {
        e2_ = static_cast<float>((_e431_.x >> as_type<uint>(4)) & 15u);
    }
    if (_e433_ == 1u) {
        metal::float4 _e459_ = metal::unpack_unorm4x8_to_float(_e431_.y);
        if (ah) {
            phi_2271_ = _e459_;
        } else {
            metal::float3 _e462_ = _e459_.xyz * _e459_.w;
            metal::float4 _e468_ = metal::float4(_e462_.x, _e459_.y, _e459_.z, _e459_.w);
            metal::float4 _e474_ = metal::float4(_e468_.x, _e462_.y, _e468_.z, _e468_.w);
            phi_2271_ = metal::float4(_e474_.x, _e474_.y, _e462_.z, _e474_.w);
        }
        metal::float4 _e482_ = phi_2271_;
        f1_ = _e482_;
    } else {
        if (Yg) {
            local_5 = _e433_ == 0u;
        } else {
            local_5 = false;
        }
        bool _e585 = local_5;
        if (_e585) {
            uint _e486_ = _e431_.x >> as_type<uint>(16);
            uint _e488_ = n.Z5_;
            if (_e486_ == 0u) {
                phi_2270_ = 0.0;
            } else {
                phi_2270_ = float2(as_type<half2>(((_e486_ + 1023u) * _e488_))).x;
            }
            float _e495_ = phi_2270_;
            U1_.y = _e495_;
        } else {
            uint _e499_ = local_1_;
            metal::float4 _e501_ = RB.c2_[metal::min(unsigned(_e499_), (_buffer_sizes.size11 - 0 - 16) / 16)];
            uint _e511_ = local_2_;
            metal::float4 _e513_ = RB.c2_[metal::min(unsigned(_e511_), (_buffer_sizes.size11 - 0 - 16) / 16)];
            metal::float2 _e516_ = (metal::float2x2(metal::float2(_e501_.x, _e501_.y), metal::float2(_e501_.z, _e501_.w)) * _e424_) + _e513_.xy;
            bool _e517_ = _e433_ == 2u;
            if (!(_e517_)) {
                local_6 = _e433_ == 3u;
            } else {
                local_6 = true;
            }
            bool _e632 = local_6;
            if (_e632) {
                f1_.w = -(as_type<float>(_e431_.y));
                if (_e513_.z > 0.9) {
                    f1_.z = 2.0;
                } else {
                    f1_.z = _e513_.w;
                }
                if (_e517_) {
                    f1_.y = 0.0;
                    f1_.x = _e516_.x;
                } else {
                    float _e533_ = f1_.z;
                    f1_.z = -(_e533_);
                    f1_.x = _e516_.x;
                    f1_.y = _e516_.y;
                }
            } else {
                f1_ = metal::float4(_e516_.x, _e516_.y, as_type<float>(_e431_.y), -2.0 - _e513_.z);
            }
        }
    }
    if (_e426_) {
        float _e547_ = n.ff;
        float _e549_ = n.gf;
        metal::float4 _e557_ = metal::float4((_e424_.x * _e547_) - 1.0, (_e424_.y * _e549_) - metal::sign(_e549_), 0.0, 1.0);
        phi_2284_ = metal::float4(_e557_.x, _e557_.y, 1.0 - (static_cast<float>(_e422_) * 0.000061035156), _e557_.w);
    } else {
        float _e567_ = n.P2_;
        phi_2284_ = metal::float4(_e567_);
    }
    metal::float4 _e570_ = phi_2284_;
    unnamed.gl_Position = _e570_;
    return;
}

struct main_Output {
    metal::float2 member [[user(loc4), flat]];
    float member_1_ [[user(loc6), flat]];
    metal::float4 member_2_ [[user(loc0), center_perspective]];
    metal::float4 gl_Position [[position]];
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
    int gl_VertexIndex_1_ = {};
    int gl_InstanceIndex_1_ = {};
    metal::float4 UB_1_ = {};
    metal::float4 VB_1_ = {};
    metal::float2 U1_ = {};
    float e2_ = {};
    metal::float4 f1_ = {};
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_11 {}, type_11 {}};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    gl_InstanceIndex_1_ = static_cast<int>(gl_InstanceIndex);
    UB_1_ = UB;
    VB_1_ = VB;
    main_1_(LC, ED, PB, gl_InstanceIndex_1_, UB_1_, VB_1_, AD, n, U1_, e2_, RB, f1_, unnamed, _buffer_sizes);
    metal::float2 _e15_ = U1_;
    float _e16_ = e2_;
    metal::float4 _e17_ = f1_;
    metal::float4 _e18_ = unnamed.gl_Position;
    const auto _tmp = VertexOutput {_e15_, _e16_, {}, _e17_, _e18_};
    return main_Output { _tmp.member, _tmp.member_1_, _tmp.member_2_, _tmp.gl_Position };
}
