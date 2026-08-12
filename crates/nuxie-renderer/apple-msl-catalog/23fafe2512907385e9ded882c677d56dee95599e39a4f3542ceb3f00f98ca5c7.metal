// language: metal3.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
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
    thread metal::float4& f1_
) {
    float phi_2229_ = {};
    int phi_2201_ = {};
    bool phi_1461_ = {};
    int phi_2214_ = {};
    metal::uint4 phi_2206_ = {};
    int phi_2213_ = {};
    metal::uint4 phi_2205_ = {};
    int phi_2212_ = {};
    metal::uint4 phi_2210_ = {};
    uint phi_2209_ = {};
    metal::int2 phi_2216_ = {};
    metal::uint4 phi_2217_ = {};
    float phi_2221_ = {};
    float phi_2292_ = {};
    float phi_2235_ = {};
    float phi_2291_ = {};
    float phi_2239_ = {};
    float phi_2236_ = {};
    float phi_2233_ = {};
    float phi_2243_ = {};
    float phi_2289_ = {};
    float phi_2242_ = {};
    float phi_2298_ = {};
    float phi_2295_ = {};
    float phi_2352_ = {};
    int phi_2324_ = {};
    float phi_2334_ = {};
    bool phi_1773_ = {};
    float phi_2341_ = {};
    metal::float2 phi_2362_ = {};
    metal::float2 phi_2361_ = {};
    metal::float2 phi_2360_ = {};
    metal::float2 phi_2378_ = {};
    metal::float2 phi_2363_ = {};
    uint phi_2411_ = {};
    metal::float2 phi_2382_ = {};
    bool phi_2381_ = {};
    uint local = {};
    uint local_1_ = {};
    uint phi_2440_ = {};
    float phi_2441_ = {};
    float phi_2442_ = {};
    uint local_2_ = {};
    uint local_3_ = {};
    metal::float4 phi_2444_ = {};
    float phi_2443_ = {};
    metal::int2 local_4_ = {};
    metal::int2 local_5_ = {};
    metal::float4 phi_2459_ = {};
    bool local_1 = {};
    bool local_2 = {};
    bool local_3 = {};
    bool local_4 = {};
    bool local_5 = {};
    bool local_6 = {};
    int _e75_ = gl_InstanceIndex_1_;
    metal::float4 _e76_ = UB_1_;
    metal::float4 _e77_ = VB_1_;
    switch(as_type<int>(0u)) {
        default: {
            int _e80_ = naga_f2i32(_e76_.x);
            int _e83_ = as_type<int>(_e76_.w);
            int _e85_ = _e83_ >> as_type<uint>(2);
            int _e86_ = _e83_ & 3;
            int _e88_ = metal::min(_e80_, as_type<int>(as_type<uint>(_e85_) - as_type<uint>(1)));
            int _e90_ = as_type<int>(as_type<uint>(as_type<int>(as_type<uint>(_e75_) * as_type<uint>(_e85_))) + as_type<uint>(_e88_));
            uint clamped_lod_e79 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
            metal::uint4 _e95_ = LC.read(metal::min(metal::uint2(metal::int2(_e90_ & 2047, _e90_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e79), LC.get_height(clamped_lod_e79)) - 1), clamped_lod_e79);
            uint _e99_ = metal::max(_e95_.w & 65535u, 1u) - 1u;
            uint clamped_lod_e97 = metal::min(uint(0), ED.get_num_mip_levels() - 1);
            metal::uint4 _e106_ = ED.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e99_ & 127u), as_type<int>(_e99_ >> as_type<uint>(7)))), metal::uint2(ED.get_width(clamped_lod_e97), ED.get_height(clamped_lod_e97)) - 1), clamped_lod_e97);
            metal::float2 _e108_ = as_type<metal::float2>(_e106_.xy);
            uint _e110_ = _e106_.z & 65535u;
            uint _e112_ = _e110_ * 4u;
            metal::int2 _e118_ = metal::int2(as_type<int>(_e112_ & 127u), as_type<int>(_e112_ >> as_type<uint>(7)));
            uint clamped_lod_e115 = metal::min(uint(0), PB.get_num_mip_levels() - 1);
            metal::uint4 _e119_ = PB.read(metal::min(metal::uint2(_e118_), metal::uint2(PB.get_width(clamped_lod_e115), PB.get_height(clamped_lod_e115)) - 1), clamped_lod_e115);
            metal::float4 _e120_ = as_type<metal::float4>(_e119_);
            metal::float2x2 _e127_ = metal::float2x2(metal::float2(_e120_.x, _e120_.y), metal::float2(_e120_.z, _e120_.w));
            uint _e128_ = _e112_ + 1u;
            metal::int2 _e134_ = metal::int2(as_type<int>(_e128_ & 127u), as_type<int>(_e128_ >> as_type<uint>(7)));
            uint clamped_lod_e136 = metal::min(uint(0), PB.get_num_mip_levels() - 1);
            metal::uint4 _e135_ = PB.read(metal::min(metal::uint2(_e134_), metal::uint2(PB.get_width(clamped_lod_e136), PB.get_height(clamped_lod_e136)) - 1), clamped_lod_e136);
            float _e139_ = as_type<float>(_e135_.z);
            float _e141_ = as_type<float>(_e135_.w);
            uint _e142_ = _e95_.w & 8388608u;
            phi_2229_ = _e76_.y;
            phi_2201_ = _e80_;
            local = _e106_.z;
            local_1_ = _e110_;
            local_2_ = _e112_;
            local_3_ = _e112_;
            local_4_ = _e118_;
            local_5_ = _e134_;
            if (_e142_ != 0u) {
                phi_2229_ = _e77_.y;
                phi_2201_ = naga_f2i32(_e77_.x);
            }
            float _e148_ = phi_2229_;
            int _e150_ = phi_2201_;
            phi_2212_ = _e90_;
            phi_2210_ = _e95_;
            phi_2209_ = _e95_.w;
            if (_e150_ != _e88_) {
                int _e153_ = as_type<int>(as_type<uint>(as_type<int>(as_type<uint>(_e90_) + as_type<uint>(_e150_))) - as_type<uint>(_e88_));
                uint clamped_lod_e165 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e158_ = LC.read(metal::min(metal::uint2(metal::int2(_e153_ & 2047, _e153_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e165), LC.get_height(clamped_lod_e165)) - 1), clamped_lod_e165);
                if ((_e158_.w & 8454143u) != (_e95_.w & 8454143u)) {
                    bool _e163_ = _e139_ == 0.0;
                    phi_1461_ = _e163_;
                    if (!(_e163_)) {
                        phi_1461_ = _e108_.x != 0.0;
                    }
                    bool _e168_ = phi_1461_;
                    phi_2214_ = _e90_;
                    phi_2206_ = _e95_;
                    if (_e168_) {
                        int _e169_ = as_type<int>(_e106_.w);
                        uint clamped_lod_e190 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                        metal::uint4 _e174_ = LC.read(metal::min(metal::uint2(metal::int2(_e169_ & 2047, _e169_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e190), LC.get_height(clamped_lod_e190)) - 1), clamped_lod_e190);
                        phi_2214_ = _e169_;
                        phi_2206_ = _e174_;
                    }
                    int _e176_ = phi_2214_;
                    metal::uint4 _e178_ = phi_2206_;
                    phi_2213_ = _e176_;
                    phi_2205_ = _e178_;
                } else {
                    phi_2213_ = _e153_;
                    phi_2205_ = _e158_;
                }
                int _e180_ = phi_2213_;
                metal::uint4 _e182_ = phi_2205_;
                phi_2212_ = _e180_;
                phi_2210_ = _e182_;
                phi_2209_ = (_e182_.w & 4286578687u) | _e142_;
            }
            int _e187_ = phi_2212_;
            metal::uint4 _e189_ = phi_2210_;
            uint _e191_ = phi_2209_;
            uint _e192_ = _e191_ & 469762048u;
            if (_e192_ == 67108864u) {
                local_1 = _e86_ == 0;
            } else {
                local_1 = false;
            }
            bool _e211 = local_1;
            if (_e211) {
                float _e198_ = static_cast<float>(_e189_.z & 65535u);
                float _e201_ = static_cast<float>(_e189_.z >> as_type<uint>(16));
                metal::int2 _e207_ = metal::int2(naga_f2i32(-1.0 - _e198_), naga_f2i32((_e201_ - _e198_) + 1.0));
                phi_2216_ = _e207_;
                if ((_e191_ & 8388608u) != 0u) {
                    phi_2216_ = naga_neg(_e207_);
                }
                metal::int2 _e212_ = phi_2216_;
                int _e214_ = as_type<int>(as_type<uint>(_e187_) + as_type<uint>(_e212_.x));
                uint clamped_lod_e245 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e219_ = LC.read(metal::min(metal::uint2(metal::int2(_e214_ & 2047, _e214_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e245), LC.get_height(clamped_lod_e245)) - 1), clamped_lod_e245);
                int _e221_ = as_type<int>(as_type<uint>(_e187_) + as_type<uint>(_e212_.y));
                uint clamped_lod_e256 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e226_ = LC.read(metal::min(metal::uint2(metal::int2(_e221_ & 2047, _e221_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e256), LC.get_height(clamped_lod_e256)) - 1), clamped_lod_e256);
                phi_2217_ = _e226_;
                if ((_e226_.w & 8454143u) != (_e219_.w & 8454143u)) {
                    int _e232_ = as_type<int>(_e106_.w);
                    uint clamped_lod_e274 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                    metal::uint4 _e237_ = LC.read(metal::min(metal::uint2(metal::int2(_e232_ & 2047, _e232_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e274), LC.get_height(clamped_lod_e274)) - 1), clamped_lod_e274);
                    phi_2217_ = _e237_;
                }
                metal::uint4 _e239_ = phi_2217_;
                float _e241_ = as_type<float>(_e219_.z);
                float _e243_ = as_type<float>(_e239_.z);
                float _e244_ = _e243_ - _e241_;
                phi_2221_ = _e244_;
                if (metal::abs(_e244_) > 3.1415927) {
                    phi_2221_ = _e244_ - (6.2831855 * metal::sign(_e244_));
                }
                float _e251_ = phi_2221_;
                float _e252_ = _e201_ + -2.0;
                float _e258_ = metal::clamp(metal::rint((metal::abs(_e251_) * 0.31830987) * _e252_), 1.0, _e201_ + -3.0);
                float _e259_ = _e252_ - _e258_;
                if (_e198_ <= _e259_) {
                    phi_2292_ = _e148_;
                    if (_e198_ == _e259_) {
                        phi_2292_ = -(_e148_);
                    }
                    float _e268_ = phi_2292_;
                    phi_2291_ = _e268_;
                    phi_2239_ = -(((3.1415927 * metal::sign(_e251_)) - _e251_));
                    phi_2236_ = _e259_;
                    phi_2233_ = _e198_;
                } else {
                    bool _e270_ = _e198_ == (_e259_ + 1.0);
                    if (_e270_) {
                        phi_2235_ = 0.0;
                    } else {
                        phi_2235_ = _e198_ - (_e259_ + 2.0);
                    }
                    float _e274_ = phi_2235_;
                    phi_2291_ = _e270_ ? 0.0 : _e148_;
                    phi_2239_ = _e251_;
                    phi_2236_ = _e270_ ? 0.0 : _e258_;
                    phi_2233_ = _e274_;
                }
                float _e278_ = phi_2291_;
                float _e280_ = phi_2239_;
                float _e282_ = phi_2236_;
                float _e284_ = phi_2233_;
                if (_e284_ == _e282_) {
                    phi_2243_ = _e243_;
                } else {
                    phi_2243_ = _e241_ + (_e280_ * (_e284_ / _e282_));
                }
                float _e290_ = phi_2243_;
                phi_2289_ = _e278_;
                phi_2242_ = _e290_;
            } else {
                phi_2289_ = _e148_;
                phi_2242_ = as_type<float>(_e189_.z);
            }
            float _e294_ = phi_2289_;
            float _e296_ = phi_2242_;
            metal::float2 _e300_ = metal::float2(metal::sin(_e296_), -(metal::cos(_e296_)));
            metal::float2 _e302_ = as_type<metal::float2>(_e189_.xy);
            phi_2298_ = _e141_;
            if (_e141_ != 0.0) {
                phi_2298_ = metal::max(_e141_, 1.0 / metal::length(_e127_ * _e300_));
            }
            float _e309_ = phi_2298_;
            if (_e139_ != 0.0) {
                float _e313_ = _e294_ * metal::sign(metal::determinant(_e127_));
                bool _e315_ = (_e191_ & 1048576u) != 0u;
                phi_2295_ = _e313_;
                if (_e315_) {
                    phi_2295_ = metal::min(_e313_, 0.0);
                }
                float _e318_ = phi_2295_;
                phi_2352_ = _e318_;
                if ((_e191_ & 524288u) != 0u) {
                    phi_2352_ = metal::max(_e318_, 0.0);
                }
                float _e323_ = phi_2352_;
                float _e325_ = (_e309_ != 0.0) ? _e309_ : 0.0;
                if (_e325_ > _e139_) {
                    local_2 = _e309_ == 0.0;
                } else {
                    local_2 = false;
                }
                bool _e378 = local_2;
                float _e329_ = _e378 ? _e325_ : _e139_;
                float _e330_ = _e329_ + _e325_;
                metal::float2 _e331_ = _e300_ * _e330_;
                phi_2360_ = _e331_;
                if (_e192_ > 134217728u) {
                    uint _e333_ = _e191_ & 4194304u;
                    int _e335_ = (_e333_ == 0u) ? -2 : 2;
                    phi_2324_ = _e335_;
                    if ((_e191_ & 8388608u) != 0u) {
                        phi_2324_ = naga_neg(_e335_);
                    }
                    int _e340_ = phi_2324_;
                    int _e341_ = as_type<int>(as_type<uint>(_e187_) + as_type<uint>(_e340_));
                    uint clamped_lod_e406 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                    metal::uint4 _e346_ = LC.read(metal::min(metal::uint2(metal::int2(_e341_ & 2047, _e341_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e406), LC.get_height(clamped_lod_e406)) - 1), clamped_lod_e406);
                    float _e350_ = metal::abs(as_type<float>(_e346_.z) - _e296_);
                    phi_2334_ = _e350_;
                    if (_e350_ > 3.1415927) {
                        phi_2334_ = 6.2831855 - _e350_;
                    }
                    float _e354_ = phi_2334_;
                    float _e359_ = (_e354_ * (((_e333_ != 0u) == _e315_) ? -0.5 : 0.5)) + _e296_;
                    metal::float2 _e363_ = metal::float2(metal::sin(_e359_), -(metal::cos(_e359_)));
                    metal::float2 _e364_ = _e127_ * _e363_;
                    float _e374_ = metal::cos(_e354_ * 0.5);
                    bool _e375_ = _e192_ == 335544320u;
                    phi_1773_ = _e375_;
                    if (!(_e375_)) {
                        if (_e192_ == 268435456u) {
                            local_3 = _e374_ >= 0.25;
                        } else {
                            local_3 = false;
                        }
                        bool _e442 = local_3;
                        phi_1773_ = _e442;
                    }
                    bool _e381_ = phi_1773_;
                    if (_e381_) {
                        phi_2341_ = _e329_ * (1.0 / metal::max(_e374_, ((_e191_ & 33554432u) != 0u) ? 1.0 : 0.25));
                    } else {
                        phi_2341_ = (_e329_ * _e374_) + (((metal::abs(_e364_.x) + metal::abs(_e364_.y)) * (1.0 / metal::dot(_e364_, _e364_))) * 0.5);
                    }
                    float _e392_ = phi_2341_;
                    phi_2361_ = _e331_;
                    if ((_e191_ & 2097152u) != 0u) {
                        if (_e330_ <= ((_e392_ * _e374_) + (_e325_ * 0.125))) {
                            phi_2362_ = _e363_ * (_e330_ * (1.0 / _e374_));
                        } else {
                            metal::float2 _e402_ = _e363_ * _e392_;
                            metal::float2x2 _e487 = _naga_inverse_2x2_f32_(metal::float2x2(_e331_, _e402_));
                            phi_2362_ = metal::float2(metal::dot(_e331_, _e331_), metal::dot(_e402_, _e402_)) * _e487;
                        }
                        metal::float2 _e410_ = phi_2362_;
                        phi_2361_ = _e410_;
                    }
                    metal::float2 _e412_ = phi_2361_;
                    phi_2360_ = _e412_;
                }
                metal::float2 _e414_ = phi_2360_;
                if (_e86_ != 0) {
                    phi_2411_ = uint {};
                    phi_2382_ = metal::float2 {};
                    phi_2381_ = false;
                    break;
                }
                phi_2378_ = _e127_ * (_e414_ * _e323_);
                phi_2363_ = _e302_;
            } else {
                if ((_e191_ & 2147483648u) != 0u) {
                    local_4 = _e86_ != 1;
                } else {
                    local_4 = false;
                }
                bool _e508 = local_4;
                if (_e508) {
                    phi_2411_ = uint {};
                    phi_2382_ = metal::float2 {};
                    phi_2381_ = false;
                    break;
                }
                phi_2378_ = metal::float2(0.0, 0.0);
                phi_2363_ = metal::select(_e302_, _e108_, metal::bool2(_e86_ == 2));
            }
            metal::float2 _e426_ = phi_2378_;
            metal::float2 _e428_ = phi_2363_;
            uint _e432_ = _e112_ + 2u;
            uint clamped_lod_e533 = metal::min(uint(0), PB.get_num_mip_levels() - 1);
            metal::uint4 _e439_ = PB.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e432_ & 127u), as_type<int>(_e432_ >> as_type<uint>(7)))), metal::uint2(PB.get_width(clamped_lod_e533), PB.get_height(clamped_lod_e533)) - 1), clamped_lod_e533);
            phi_2411_ = _e439_.x;
            phi_2382_ = ((_e127_ * _e428_) + _e426_) + as_type<metal::float2>(_e135_.xy);
            phi_2381_ = true;
            break;
        }
    }
    uint _e442_ = phi_2411_;
    metal::float2 _e444_ = phi_2382_;
    bool _e446_ = phi_2381_;
    uint _e448_ = local;
    uint _e452_ = local_1_;
    uint clamped_lod_e556 = metal::min(uint(0), AD.get_num_mip_levels() - 1);
    metal::uint4 _e457_ = AD.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e448_ & 127u), as_type<int>(_e452_ >> as_type<uint>(7)))), metal::uint2(AD.get_width(clamped_lod_e556), AD.get_height(clamped_lod_e556)) - 1), clamped_lod_e556);
    uint _e459_ = _e457_.x & 15u;
    if (Yg) {
        bool _e460_ = _e459_ == 0u;
        if (_e460_) {
            phi_2440_ = _e457_.y;
        } else {
            phi_2440_ = _e457_.x;
        }
        uint _e463_ = phi_2440_;
        uint _e465_ = _e463_ >> as_type<uint>(16);
        uint _e467_ = n.Z5_;
        if (_e465_ == 0u) {
            phi_2441_ = 0.0;
        } else {
            phi_2441_ = float2(as_type<half2>(((_e465_ + 1023u) * _e467_))).x;
        }
        float _e474_ = phi_2441_;
        phi_2442_ = _e474_;
        if (_e460_) {
            phi_2442_ = -(_e474_);
        }
        float _e477_ = phi_2442_;
        U1_.x = _e477_;
    }
    if (ah) {
        e2_ = static_cast<float>((_e457_.x >> as_type<uint>(4)) & 15u);
    }
    if (Zg) {
        uint _e484_ = local_2_;
        uint _e485_ = _e484_ + 2u;
        uint clamped_lod_e608 = metal::min(uint(0), RB.get_num_mip_levels() - 1);
        metal::float4 _e492_ = RB.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e485_ & 127u), as_type<int>(_e485_ >> as_type<uint>(7)))), metal::uint2(RB.get_width(clamped_lod_e608), RB.get_height(clamped_lod_e608)) - 1), clamped_lod_e608);
        uint _e501_ = local_3_;
        uint _e502_ = _e501_ + 3u;
        uint clamped_lod_e622 = metal::min(uint(0), RB.get_num_mip_levels() - 1);
        metal::float4 _e509_ = RB.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e502_ & 127u), as_type<int>(_e502_ >> as_type<uint>(7)))), metal::uint2(RB.get_width(clamped_lod_e622), RB.get_height(clamped_lod_e622)) - 1), clamped_lod_e622);
        if (metal::any(_e492_ != metal::float4(0.0, 0.0, 0.0, 0.0))) {
            metal::float2 _e514_ = (metal::float2x2(metal::float2(_e492_.x, _e492_.y), metal::float2(_e492_.z, _e492_.w)) * _e444_) + _e509_.xy;
            unnamed.gl_ClipDistance.inner[0] = _e514_.x + 1.0;
            unnamed.gl_ClipDistance.inner[1] = _e514_.y + 1.0;
            unnamed.gl_ClipDistance.inner[2] = 1.0 - _e514_.x;
            unnamed.gl_ClipDistance.inner[3] = 1.0 - _e514_.y;
        } else {
            float _e530_ = _e509_.x - 0.5;
            unnamed.gl_ClipDistance.inner[3] = _e530_;
            unnamed.gl_ClipDistance.inner[2] = _e530_;
            unnamed.gl_ClipDistance.inner[1] = _e530_;
            unnamed.gl_ClipDistance.inner[0] = _e530_;
        }
    }
    if (_e459_ == 1u) {
        metal::float4 _e541_ = metal::unpack_unorm4x8_to_float(_e457_.y);
        if (ah) {
            phi_2444_ = _e541_;
        } else {
            metal::float3 _e544_ = _e541_.xyz * _e541_.w;
            metal::float4 _e550_ = metal::float4(_e544_.x, _e541_.y, _e541_.z, _e541_.w);
            metal::float4 _e556_ = metal::float4(_e550_.x, _e544_.y, _e550_.z, _e550_.w);
            phi_2444_ = metal::float4(_e556_.x, _e556_.y, _e544_.z, _e556_.w);
        }
        metal::float4 _e564_ = phi_2444_;
        f1_ = _e564_;
    } else {
        if (Yg) {
            local_5 = _e459_ == 0u;
        } else {
            local_5 = false;
        }
        bool _e710 = local_5;
        if (_e710) {
            uint _e568_ = _e457_.x >> as_type<uint>(16);
            uint _e570_ = n.Z5_;
            if (_e568_ == 0u) {
                phi_2443_ = 0.0;
            } else {
                phi_2443_ = float2(as_type<half2>(((_e568_ + 1023u) * _e570_))).x;
            }
            float _e577_ = phi_2443_;
            U1_.y = _e577_;
        } else {
            metal::int2 _e580_ = local_4_;
            uint clamped_lod_e732 = metal::min(uint(0), RB.get_num_mip_levels() - 1);
            metal::float4 _e581_ = RB.read(metal::min(metal::uint2(_e580_), metal::uint2(RB.get_width(clamped_lod_e732), RB.get_height(clamped_lod_e732)) - 1), clamped_lod_e732);
            metal::int2 _e590_ = local_5_;
            uint clamped_lod_e736 = metal::min(uint(0), RB.get_num_mip_levels() - 1);
            metal::float4 _e591_ = RB.read(metal::min(metal::uint2(_e590_), metal::uint2(RB.get_width(clamped_lod_e736), RB.get_height(clamped_lod_e736)) - 1), clamped_lod_e736);
            metal::float2 _e594_ = (metal::float2x2(metal::float2(_e581_.x, _e581_.y), metal::float2(_e581_.z, _e581_.w)) * _e444_) + _e591_.xy;
            bool _e595_ = _e459_ == 2u;
            if (!(_e595_)) {
                local_6 = _e459_ == 3u;
            } else {
                local_6 = true;
            }
            bool _e755 = local_6;
            if (_e755) {
                f1_.w = -(as_type<float>(_e457_.y));
                if (_e591_.z > 0.9) {
                    f1_.z = 2.0;
                } else {
                    f1_.z = _e591_.w;
                }
                if (_e595_) {
                    f1_.y = 0.0;
                    f1_.x = _e594_.x;
                } else {
                    float _e611_ = f1_.z;
                    f1_.z = -(_e611_);
                    f1_.x = _e594_.x;
                    f1_.y = _e594_.y;
                }
            } else {
                f1_ = metal::float4(_e594_.x, _e594_.y, as_type<float>(_e457_.y), -2.0 - _e591_.z);
            }
        }
    }
    if (_e446_) {
        float _e625_ = n.ff;
        float _e627_ = n.gf;
        metal::float4 _e635_ = metal::float4((_e444_.x * _e625_) - 1.0, (_e444_.y * _e627_) - metal::sign(_e627_), 0.0, 1.0);
        phi_2459_ = metal::float4(_e635_.x, _e635_.y, 1.0 - (static_cast<float>(_e442_) * 0.000061035156), _e635_.w);
    } else {
        float _e645_ = n.P2_;
        phi_2459_ = metal::float4(_e645_);
    }
    metal::float4 _e648_ = phi_2459_;
    unnamed.gl_Position = _e648_;
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
    main_1_(unnamed, LC, ED, PB, gl_InstanceIndex_1_, UB_1_, VB_1_, AD, n, U1_, e2_, RB, f1_);
    metal::float4 _e16_ = unnamed.gl_Position;
    type_2 _e17_ = unnamed.gl_ClipDistance;
    metal::float2 _e18_ = U1_;
    float _e19_ = e2_;
    metal::float4 _e20_ = f1_;
    const auto _tmp = VertexOutput {_e16_, _e17_, _e18_, _e19_, {}, _e20_};
    return main_Output { _tmp.gl_Position, {_tmp.gl_ClipDistance.inner[0],_tmp.gl_ClipDistance.inner[1],_tmp.gl_ClipDistance.inner[2],_tmp.gl_ClipDistance.inner[3]}, _tmp.member, _tmp.member_1_, _tmp.member_2_ };
}
