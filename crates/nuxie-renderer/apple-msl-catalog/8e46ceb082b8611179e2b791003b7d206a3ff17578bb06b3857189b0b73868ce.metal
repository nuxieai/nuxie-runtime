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
    metal::texture2d<uint, metal::access::sample> ED,
    metal::texture2d<uint, metal::access::sample> PB,
    thread int& gl_InstanceIndex_1_,
    thread metal::float4& UB_1_,
    thread metal::float4& VB_1_,
    metal::texture2d<uint, metal::access::sample> AD,
    constant CC& n,
    thread metal::float2& U1_,
    thread float& e2_,
    metal::texture2d<float, metal::access::sample> RB,
    thread metal::float4& f1_,
    thread gl_PerVertex& unnamed
) {
    float phi_2141_ = {};
    int phi_2113_ = {};
    bool phi_1418_ = {};
    int phi_2126_ = {};
    metal::uint4 phi_2118_ = {};
    int phi_2125_ = {};
    metal::uint4 phi_2117_ = {};
    int phi_2124_ = {};
    metal::uint4 phi_2122_ = {};
    uint phi_2121_ = {};
    metal::int2 phi_2128_ = {};
    metal::uint4 phi_2129_ = {};
    float phi_2133_ = {};
    float phi_2204_ = {};
    float phi_2147_ = {};
    float phi_2203_ = {};
    float phi_2151_ = {};
    float phi_2148_ = {};
    float phi_2145_ = {};
    float phi_2155_ = {};
    float phi_2201_ = {};
    float phi_2154_ = {};
    float phi_2210_ = {};
    float phi_2207_ = {};
    float phi_2264_ = {};
    int phi_2236_ = {};
    float phi_2246_ = {};
    bool phi_1730_ = {};
    float phi_2253_ = {};
    metal::float2 phi_2274_ = {};
    metal::float2 phi_2273_ = {};
    metal::float2 phi_2272_ = {};
    metal::float2 phi_2290_ = {};
    metal::float2 phi_2275_ = {};
    uint phi_2323_ = {};
    metal::float2 phi_2294_ = {};
    bool phi_2293_ = {};
    uint local = {};
    uint local_1_ = {};
    uint phi_2352_ = {};
    float phi_2353_ = {};
    float phi_2354_ = {};
    metal::float4 phi_2356_ = {};
    float phi_2355_ = {};
    metal::int2 local_2_ = {};
    metal::int2 local_3_ = {};
    metal::float4 phi_2369_ = {};
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
            uint clamped_lod_e77 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
            metal::uint4 _e93_ = LC.read(metal::min(metal::uint2(metal::int2(_e88_ & 2047, _e88_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e77), LC.get_height(clamped_lod_e77)) - 1), clamped_lod_e77);
            uint _e97_ = metal::max(_e93_.w & 65535u, 1u) - 1u;
            uint clamped_lod_e95 = metal::min(uint(0), ED.get_num_mip_levels() - 1);
            metal::uint4 _e104_ = ED.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e97_ & 127u), as_type<int>(_e97_ >> as_type<uint>(7)))), metal::uint2(ED.get_width(clamped_lod_e95), ED.get_height(clamped_lod_e95)) - 1), clamped_lod_e95);
            metal::float2 _e106_ = as_type<metal::float2>(_e104_.xy);
            uint _e108_ = _e104_.z & 65535u;
            uint _e110_ = _e108_ * 4u;
            metal::int2 _e116_ = metal::int2(as_type<int>(_e110_ & 127u), as_type<int>(_e110_ >> as_type<uint>(7)));
            uint clamped_lod_e113 = metal::min(uint(0), PB.get_num_mip_levels() - 1);
            metal::uint4 _e117_ = PB.read(metal::min(metal::uint2(_e116_), metal::uint2(PB.get_width(clamped_lod_e113), PB.get_height(clamped_lod_e113)) - 1), clamped_lod_e113);
            metal::float4 _e118_ = as_type<metal::float4>(_e117_);
            metal::float2x2 _e125_ = metal::float2x2(metal::float2(_e118_.x, _e118_.y), metal::float2(_e118_.z, _e118_.w));
            uint _e126_ = _e110_ + 1u;
            metal::int2 _e132_ = metal::int2(as_type<int>(_e126_ & 127u), as_type<int>(_e126_ >> as_type<uint>(7)));
            uint clamped_lod_e134 = metal::min(uint(0), PB.get_num_mip_levels() - 1);
            metal::uint4 _e133_ = PB.read(metal::min(metal::uint2(_e132_), metal::uint2(PB.get_width(clamped_lod_e134), PB.get_height(clamped_lod_e134)) - 1), clamped_lod_e134);
            float _e137_ = as_type<float>(_e133_.z);
            float _e139_ = as_type<float>(_e133_.w);
            uint _e140_ = _e93_.w & 8388608u;
            phi_2141_ = _e74_.y;
            phi_2113_ = _e78_;
            local = _e104_.z;
            local_1_ = _e108_;
            local_2_ = _e116_;
            local_3_ = _e132_;
            if (_e140_ != 0u) {
                phi_2141_ = _e75_.y;
                phi_2113_ = naga_f2i32(_e75_.x);
            }
            float _e146_ = phi_2141_;
            int _e148_ = phi_2113_;
            phi_2124_ = _e88_;
            phi_2122_ = _e93_;
            phi_2121_ = _e93_.w;
            if (_e148_ != _e86_) {
                int _e151_ = as_type<int>(as_type<uint>(as_type<int>(as_type<uint>(_e88_) + as_type<uint>(_e148_))) - as_type<uint>(_e86_));
                uint clamped_lod_e163 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e156_ = LC.read(metal::min(metal::uint2(metal::int2(_e151_ & 2047, _e151_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e163), LC.get_height(clamped_lod_e163)) - 1), clamped_lod_e163);
                if ((_e156_.w & 8454143u) != (_e93_.w & 8454143u)) {
                    bool _e161_ = _e137_ == 0.0;
                    phi_1418_ = _e161_;
                    if (!(_e161_)) {
                        phi_1418_ = _e106_.x != 0.0;
                    }
                    bool _e166_ = phi_1418_;
                    phi_2126_ = _e88_;
                    phi_2118_ = _e93_;
                    if (_e166_) {
                        int _e167_ = as_type<int>(_e104_.w);
                        uint clamped_lod_e188 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                        metal::uint4 _e172_ = LC.read(metal::min(metal::uint2(metal::int2(_e167_ & 2047, _e167_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e188), LC.get_height(clamped_lod_e188)) - 1), clamped_lod_e188);
                        phi_2126_ = _e167_;
                        phi_2118_ = _e172_;
                    }
                    int _e174_ = phi_2126_;
                    metal::uint4 _e176_ = phi_2118_;
                    phi_2125_ = _e174_;
                    phi_2117_ = _e176_;
                } else {
                    phi_2125_ = _e151_;
                    phi_2117_ = _e156_;
                }
                int _e178_ = phi_2125_;
                metal::uint4 _e180_ = phi_2117_;
                phi_2124_ = _e178_;
                phi_2122_ = _e180_;
                phi_2121_ = (_e180_.w & 4286578687u) | _e140_;
            }
            int _e185_ = phi_2124_;
            metal::uint4 _e187_ = phi_2122_;
            uint _e189_ = phi_2121_;
            uint _e190_ = _e189_ & 469762048u;
            if (_e190_ == 67108864u) {
                local_1 = _e84_ == 0;
            } else {
                local_1 = false;
            }
            bool _e209 = local_1;
            if (_e209) {
                float _e196_ = static_cast<float>(_e187_.z & 65535u);
                float _e199_ = static_cast<float>(_e187_.z >> as_type<uint>(16));
                metal::int2 _e205_ = metal::int2(naga_f2i32(-1.0 - _e196_), naga_f2i32((_e199_ - _e196_) + 1.0));
                phi_2128_ = _e205_;
                if ((_e189_ & 8388608u) != 0u) {
                    phi_2128_ = naga_neg(_e205_);
                }
                metal::int2 _e210_ = phi_2128_;
                int _e212_ = as_type<int>(as_type<uint>(_e185_) + as_type<uint>(_e210_.x));
                uint clamped_lod_e243 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e217_ = LC.read(metal::min(metal::uint2(metal::int2(_e212_ & 2047, _e212_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e243), LC.get_height(clamped_lod_e243)) - 1), clamped_lod_e243);
                int _e219_ = as_type<int>(as_type<uint>(_e185_) + as_type<uint>(_e210_.y));
                uint clamped_lod_e254 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e224_ = LC.read(metal::min(metal::uint2(metal::int2(_e219_ & 2047, _e219_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e254), LC.get_height(clamped_lod_e254)) - 1), clamped_lod_e254);
                phi_2129_ = _e224_;
                if ((_e224_.w & 8454143u) != (_e217_.w & 8454143u)) {
                    int _e230_ = as_type<int>(_e104_.w);
                    uint clamped_lod_e272 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                    metal::uint4 _e235_ = LC.read(metal::min(metal::uint2(metal::int2(_e230_ & 2047, _e230_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e272), LC.get_height(clamped_lod_e272)) - 1), clamped_lod_e272);
                    phi_2129_ = _e235_;
                }
                metal::uint4 _e237_ = phi_2129_;
                float _e239_ = as_type<float>(_e217_.z);
                float _e241_ = as_type<float>(_e237_.z);
                float _e242_ = _e241_ - _e239_;
                phi_2133_ = _e242_;
                if (metal::abs(_e242_) > 3.1415927) {
                    phi_2133_ = _e242_ - (6.2831855 * metal::sign(_e242_));
                }
                float _e249_ = phi_2133_;
                float _e250_ = _e199_ + -2.0;
                float _e256_ = metal::clamp(metal::rint((metal::abs(_e249_) * 0.31830987) * _e250_), 1.0, _e199_ + -3.0);
                float _e257_ = _e250_ - _e256_;
                if (_e196_ <= _e257_) {
                    phi_2204_ = _e146_;
                    if (_e196_ == _e257_) {
                        phi_2204_ = -(_e146_);
                    }
                    float _e266_ = phi_2204_;
                    phi_2203_ = _e266_;
                    phi_2151_ = -(((3.1415927 * metal::sign(_e249_)) - _e249_));
                    phi_2148_ = _e257_;
                    phi_2145_ = _e196_;
                } else {
                    bool _e268_ = _e196_ == (_e257_ + 1.0);
                    if (_e268_) {
                        phi_2147_ = 0.0;
                    } else {
                        phi_2147_ = _e196_ - (_e257_ + 2.0);
                    }
                    float _e272_ = phi_2147_;
                    phi_2203_ = _e268_ ? 0.0 : _e146_;
                    phi_2151_ = _e249_;
                    phi_2148_ = _e268_ ? 0.0 : _e256_;
                    phi_2145_ = _e272_;
                }
                float _e276_ = phi_2203_;
                float _e278_ = phi_2151_;
                float _e280_ = phi_2148_;
                float _e282_ = phi_2145_;
                if (_e282_ == _e280_) {
                    phi_2155_ = _e241_;
                } else {
                    phi_2155_ = _e239_ + (_e278_ * (_e282_ / _e280_));
                }
                float _e288_ = phi_2155_;
                phi_2201_ = _e276_;
                phi_2154_ = _e288_;
            } else {
                phi_2201_ = _e146_;
                phi_2154_ = as_type<float>(_e187_.z);
            }
            float _e292_ = phi_2201_;
            float _e294_ = phi_2154_;
            metal::float2 _e298_ = metal::float2(metal::sin(_e294_), -(metal::cos(_e294_)));
            metal::float2 _e300_ = as_type<metal::float2>(_e187_.xy);
            phi_2210_ = _e139_;
            if (_e139_ != 0.0) {
                phi_2210_ = metal::max(_e139_, 1.0 / metal::length(_e125_ * _e298_));
            }
            float _e307_ = phi_2210_;
            if (_e137_ != 0.0) {
                float _e311_ = _e292_ * metal::sign(metal::determinant(_e125_));
                bool _e313_ = (_e189_ & 1048576u) != 0u;
                phi_2207_ = _e311_;
                if (_e313_) {
                    phi_2207_ = metal::min(_e311_, 0.0);
                }
                float _e316_ = phi_2207_;
                phi_2264_ = _e316_;
                if ((_e189_ & 524288u) != 0u) {
                    phi_2264_ = metal::max(_e316_, 0.0);
                }
                float _e321_ = phi_2264_;
                float _e323_ = (_e307_ != 0.0) ? _e307_ : 0.0;
                if (_e323_ > _e137_) {
                    local_2 = _e307_ == 0.0;
                } else {
                    local_2 = false;
                }
                bool _e376 = local_2;
                float _e327_ = _e376 ? _e323_ : _e137_;
                float _e328_ = _e327_ + _e323_;
                metal::float2 _e329_ = _e298_ * _e328_;
                phi_2272_ = _e329_;
                if (_e190_ > 134217728u) {
                    uint _e331_ = _e189_ & 4194304u;
                    int _e333_ = (_e331_ == 0u) ? -2 : 2;
                    phi_2236_ = _e333_;
                    if ((_e189_ & 8388608u) != 0u) {
                        phi_2236_ = naga_neg(_e333_);
                    }
                    int _e338_ = phi_2236_;
                    int _e339_ = as_type<int>(as_type<uint>(_e185_) + as_type<uint>(_e338_));
                    uint clamped_lod_e404 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                    metal::uint4 _e344_ = LC.read(metal::min(metal::uint2(metal::int2(_e339_ & 2047, _e339_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e404), LC.get_height(clamped_lod_e404)) - 1), clamped_lod_e404);
                    float _e348_ = metal::abs(as_type<float>(_e344_.z) - _e294_);
                    phi_2246_ = _e348_;
                    if (_e348_ > 3.1415927) {
                        phi_2246_ = 6.2831855 - _e348_;
                    }
                    float _e352_ = phi_2246_;
                    float _e357_ = (_e352_ * (((_e331_ != 0u) == _e313_) ? -0.5 : 0.5)) + _e294_;
                    metal::float2 _e361_ = metal::float2(metal::sin(_e357_), -(metal::cos(_e357_)));
                    metal::float2 _e362_ = _e125_ * _e361_;
                    float _e372_ = metal::cos(_e352_ * 0.5);
                    bool _e373_ = _e190_ == 335544320u;
                    phi_1730_ = _e373_;
                    if (!(_e373_)) {
                        if (_e190_ == 268435456u) {
                            local_3 = _e372_ >= 0.25;
                        } else {
                            local_3 = false;
                        }
                        bool _e440 = local_3;
                        phi_1730_ = _e440;
                    }
                    bool _e379_ = phi_1730_;
                    if (_e379_) {
                        phi_2253_ = _e327_ * (1.0 / metal::max(_e372_, ((_e189_ & 33554432u) != 0u) ? 1.0 : 0.25));
                    } else {
                        phi_2253_ = (_e327_ * _e372_) + (((metal::abs(_e362_.x) + metal::abs(_e362_.y)) * (1.0 / metal::dot(_e362_, _e362_))) * 0.5);
                    }
                    float _e390_ = phi_2253_;
                    phi_2273_ = _e329_;
                    if ((_e189_ & 2097152u) != 0u) {
                        if (_e328_ <= ((_e390_ * _e372_) + (_e323_ * 0.125))) {
                            phi_2274_ = _e361_ * (_e328_ * (1.0 / _e372_));
                        } else {
                            metal::float2 _e400_ = _e361_ * _e390_;
                            metal::float2x2 _e485 = _naga_inverse_2x2_f32_(metal::float2x2(_e329_, _e400_));
                            phi_2274_ = metal::float2(metal::dot(_e329_, _e329_), metal::dot(_e400_, _e400_)) * _e485;
                        }
                        metal::float2 _e408_ = phi_2274_;
                        phi_2273_ = _e408_;
                    }
                    metal::float2 _e410_ = phi_2273_;
                    phi_2272_ = _e410_;
                }
                metal::float2 _e412_ = phi_2272_;
                if (_e84_ != 0) {
                    phi_2323_ = uint {};
                    phi_2294_ = metal::float2 {};
                    phi_2293_ = false;
                    break;
                }
                phi_2290_ = _e125_ * (_e412_ * _e321_);
                phi_2275_ = _e300_;
            } else {
                if ((_e189_ & 2147483648u) != 0u) {
                    local_4 = _e84_ != 1;
                } else {
                    local_4 = false;
                }
                bool _e506 = local_4;
                if (_e506) {
                    phi_2323_ = uint {};
                    phi_2294_ = metal::float2 {};
                    phi_2293_ = false;
                    break;
                }
                phi_2290_ = metal::float2(0.0, 0.0);
                phi_2275_ = metal::select(_e300_, _e106_, metal::bool2(_e84_ == 2));
            }
            metal::float2 _e424_ = phi_2290_;
            metal::float2 _e426_ = phi_2275_;
            uint _e430_ = _e110_ + 2u;
            uint clamped_lod_e531 = metal::min(uint(0), PB.get_num_mip_levels() - 1);
            metal::uint4 _e437_ = PB.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e430_ & 127u), as_type<int>(_e430_ >> as_type<uint>(7)))), metal::uint2(PB.get_width(clamped_lod_e531), PB.get_height(clamped_lod_e531)) - 1), clamped_lod_e531);
            phi_2323_ = _e437_.x;
            phi_2294_ = ((_e125_ * _e426_) + _e424_) + as_type<metal::float2>(_e133_.xy);
            phi_2293_ = true;
            break;
        }
    }
    uint _e440_ = phi_2323_;
    metal::float2 _e442_ = phi_2294_;
    bool _e444_ = phi_2293_;
    uint _e446_ = local;
    uint _e450_ = local_1_;
    uint clamped_lod_e554 = metal::min(uint(0), AD.get_num_mip_levels() - 1);
    metal::uint4 _e455_ = AD.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e446_ & 127u), as_type<int>(_e450_ >> as_type<uint>(7)))), metal::uint2(AD.get_width(clamped_lod_e554), AD.get_height(clamped_lod_e554)) - 1), clamped_lod_e554);
    uint _e457_ = _e455_.x & 15u;
    if (Yg) {
        bool _e458_ = _e457_ == 0u;
        if (_e458_) {
            phi_2352_ = _e455_.y;
        } else {
            phi_2352_ = _e455_.x;
        }
        uint _e461_ = phi_2352_;
        uint _e463_ = _e461_ >> as_type<uint>(16);
        uint _e465_ = n.Z5_;
        if (_e463_ == 0u) {
            phi_2353_ = 0.0;
        } else {
            phi_2353_ = float2(as_type<half2>(((_e463_ + 1023u) * _e465_))).x;
        }
        float _e472_ = phi_2353_;
        phi_2354_ = _e472_;
        if (_e458_) {
            phi_2354_ = -(_e472_);
        }
        float _e475_ = phi_2354_;
        U1_.x = _e475_;
    }
    if (ah) {
        e2_ = static_cast<float>((_e455_.x >> as_type<uint>(4)) & 15u);
    }
    if (_e457_ == 1u) {
        metal::float4 _e483_ = metal::unpack_unorm4x8_to_float(_e455_.y);
        if (ah) {
            phi_2356_ = _e483_;
        } else {
            metal::float3 _e486_ = _e483_.xyz * _e483_.w;
            metal::float4 _e492_ = metal::float4(_e486_.x, _e483_.y, _e483_.z, _e483_.w);
            metal::float4 _e498_ = metal::float4(_e492_.x, _e486_.y, _e492_.z, _e492_.w);
            phi_2356_ = metal::float4(_e498_.x, _e498_.y, _e486_.z, _e498_.w);
        }
        metal::float4 _e506_ = phi_2356_;
        f1_ = _e506_;
    } else {
        if (Yg) {
            local_5 = _e457_ == 0u;
        } else {
            local_5 = false;
        }
        bool _e623 = local_5;
        if (_e623) {
            uint _e510_ = _e455_.x >> as_type<uint>(16);
            uint _e512_ = n.Z5_;
            if (_e510_ == 0u) {
                phi_2355_ = 0.0;
            } else {
                phi_2355_ = float2(as_type<half2>(((_e510_ + 1023u) * _e512_))).x;
            }
            float _e519_ = phi_2355_;
            U1_.y = _e519_;
        } else {
            metal::int2 _e522_ = local_2_;
            uint clamped_lod_e645 = metal::min(uint(0), RB.get_num_mip_levels() - 1);
            metal::float4 _e523_ = RB.read(metal::min(metal::uint2(_e522_), metal::uint2(RB.get_width(clamped_lod_e645), RB.get_height(clamped_lod_e645)) - 1), clamped_lod_e645);
            metal::int2 _e532_ = local_3_;
            uint clamped_lod_e649 = metal::min(uint(0), RB.get_num_mip_levels() - 1);
            metal::float4 _e533_ = RB.read(metal::min(metal::uint2(_e532_), metal::uint2(RB.get_width(clamped_lod_e649), RB.get_height(clamped_lod_e649)) - 1), clamped_lod_e649);
            metal::float2 _e536_ = (metal::float2x2(metal::float2(_e523_.x, _e523_.y), metal::float2(_e523_.z, _e523_.w)) * _e442_) + _e533_.xy;
            bool _e537_ = _e457_ == 2u;
            if (!(_e537_)) {
                local_6 = _e457_ == 3u;
            } else {
                local_6 = true;
            }
            bool _e668 = local_6;
            if (_e668) {
                f1_.w = -(as_type<float>(_e455_.y));
                if (_e533_.z > 0.9) {
                    f1_.z = 2.0;
                } else {
                    f1_.z = _e533_.w;
                }
                if (_e537_) {
                    f1_.y = 0.0;
                    f1_.x = _e536_.x;
                } else {
                    float _e553_ = f1_.z;
                    f1_.z = -(_e553_);
                    f1_.x = _e536_.x;
                    f1_.y = _e536_.y;
                }
            } else {
                f1_ = metal::float4(_e536_.x, _e536_.y, as_type<float>(_e455_.y), -2.0 - _e533_.z);
            }
        }
    }
    if (_e444_) {
        float _e567_ = n.ff;
        float _e569_ = n.gf;
        metal::float4 _e577_ = metal::float4((_e442_.x * _e567_) - 1.0, (_e442_.y * _e569_) - metal::sign(_e569_), 0.0, 1.0);
        phi_2369_ = metal::float4(_e577_.x, _e577_.y, 1.0 - (static_cast<float>(_e440_) * 0.000061035156), _e577_.w);
    } else {
        float _e587_ = n.P2_;
        phi_2369_ = metal::float4(_e587_);
    }
    metal::float4 _e590_ = phi_2369_;
    unnamed.gl_Position = _e590_;
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
, metal::texture2d<uint, metal::access::sample> LC [[texture(4)]]
, metal::texture2d<uint, metal::access::sample> ED [[texture(3)]]
, metal::texture2d<uint, metal::access::sample> PB [[texture(0)]]
, metal::texture2d<uint, metal::access::sample> AD [[texture(1)]]
, constant CC& n [[buffer(0)]]
, metal::texture2d<float, metal::access::sample> RB [[texture(2)]]
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
    metal::float2 U1_ = {};
    float e2_ = {};
    metal::float4 f1_ = {};
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_6 {}, type_6 {}};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    gl_InstanceIndex_1_ = static_cast<int>(gl_InstanceIndex);
    UB_1_ = UB;
    VB_1_ = VB;
    main_1_(LC, ED, PB, gl_InstanceIndex_1_, UB_1_, VB_1_, AD, n, U1_, e2_, RB, f1_, unnamed);
    metal::float2 _e15_ = U1_;
    float _e16_ = e2_;
    metal::float4 _e17_ = f1_;
    metal::float4 _e18_ = unnamed.gl_Position;
    const auto _tmp = VertexOutput {_e15_, _e16_, {}, _e17_, _e18_};
    return main_Output { _tmp.member, _tmp.member_1_, _tmp.member_2_, _tmp.gl_Position };
}
