// language: metal4.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size1;
    uint size2;
    uint size12;
    uint size13;
    uint buffer_size30;
};

typedef metal::uint4 type_2[1];
struct cg {
    type_2 c2_;
};
struct bg {
    type_2 c2_;
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
struct type_8 {
    float inner[1];
};
struct gl_PerVertex {
    metal::float4 gl_Position;
    float gl_PointSize;
    type_8 gl_ClipDistance;
    type_8 gl_CullDistance;
    char _pad4[4];
};
typedef metal::uint2 type_10[1];
struct Je {
    type_10 c2_;
};
typedef metal::float4 type_11[1];
struct Ke {
    type_11 c2_;
};
struct VertexOutput {
    metal::float4 member;
    uint member_1_;
    char _pad2[12];
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
    device cg const& ED,
    device bg const& PB,
    constant CC& n,
    thread int& gl_InstanceIndex_1_,
    thread metal::float4& UB_1_,
    thread metal::float4& VB_1_,
    thread metal::float4& O,
    thread uint& B0_,
    thread gl_PerVertex& unnamed,
    constant _mslBufferSizes& _buffer_sizes
) {
    float phi_2303_ = {};
    float phi_2241_ = {};
    int phi_2213_ = {};
    bool phi_1351_ = {};
    int phi_2226_ = {};
    metal::uint4 phi_2218_ = {};
    int phi_2225_ = {};
    metal::uint4 phi_2217_ = {};
    int phi_2224_ = {};
    metal::uint4 phi_2222_ = {};
    uint phi_2221_ = {};
    metal::int2 phi_2228_ = {};
    metal::uint4 phi_2229_ = {};
    float phi_2233_ = {};
    float phi_2313_ = {};
    float phi_2247_ = {};
    float phi_2312_ = {};
    float phi_2255_ = {};
    float phi_2248_ = {};
    float phi_2245_ = {};
    float phi_2259_ = {};
    float phi_2334_ = {};
    float phi_2325_ = {};
    float phi_2310_ = {};
    float phi_2258_ = {};
    float phi_2308_ = {};
    float phi_2391_ = {};
    float phi_2402_ = {};
    float phi_2394_ = {};
    float phi_2483_ = {};
    int phi_2440_ = {};
    float phi_2449_ = {};
    bool phi_1690_ = {};
    float phi_2456_ = {};
    metal::float2 phi_2472_ = {};
    metal::float2 phi_2471_ = {};
    metal::float4 phi_2493_ = {};
    metal::float2 phi_2508_ = {};
    metal::float4 phi_2492_ = {};
    metal::float4 phi_2539_ = {};
    float phi_2344_ = {};
    float phi_2343_ = {};
    float phi_2345_ = {};
    float phi_2349_ = {};
    float phi_2371_ = {};
    float phi_2369_ = {};
    metal::float4 phi_2387_ = {};
    metal::float2 phi_2537_ = {};
    metal::float4 phi_2386_ = {};
    metal::float4 phi_2541_ = {};
    metal::float4 phi_2538_ = {};
    metal::float2 phi_2534_ = {};
    metal::float2 phi_2510_ = {};
    metal::float4 phi_2581_ = {};
    metal::float2 phi_2543_ = {};
    bool phi_2542_ = {};
    uint local = {};
    metal::float4 phi_2582_ = {};
    bool local_1 = {};
    bool local_2 = {};
    bool local_3 = {};
    bool local_4 = {};
    int _e70_ = gl_InstanceIndex_1_;
    metal::float4 _e71_ = UB_1_;
    metal::float4 _e72_ = VB_1_;
    switch(as_type<int>(0u)) {
        default: {
            int _e75_ = naga_f2i32(_e71_.x);
            int _e79_ = as_type<int>(_e71_.w);
            int _e81_ = _e79_ >> as_type<uint>(2);
            int _e82_ = _e79_ & 3;
            int _e84_ = metal::min(_e75_, as_type<int>(as_type<uint>(_e81_) - as_type<uint>(1)));
            int _e86_ = as_type<int>(as_type<uint>(as_type<int>(as_type<uint>(_e70_) * as_type<uint>(_e81_))) + as_type<uint>(_e84_));
            uint clamped_lod_e88 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
            metal::uint4 _e91_ = LC.read(metal::min(metal::uint2(metal::int2(_e86_ & 2047, _e86_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e88), LC.get_height(clamped_lod_e88)) - 1), clamped_lod_e88);
            metal::uint4 _e98_ = ED.c2_[metal::min(unsigned(metal::max(_e91_.w & 65535u, 1u) - 1u), (_buffer_sizes.size1 - 0 - 16) / 16)];
            metal::float2 _e100_ = as_type<metal::float2>(_e98_.xy);
            uint _e102_ = _e98_.z & 65535u;
            uint _e104_ = _e102_ * 4u;
            metal::uint4 _e107_ = PB.c2_[metal::min(unsigned(_e104_), (_buffer_sizes.size2 - 0 - 16) / 16)];
            metal::float4 _e108_ = as_type<metal::float4>(_e107_);
            metal::float2x2 _e115_ = metal::float2x2(metal::float2(_e108_.x, _e108_.y), metal::float2(_e108_.z, _e108_.w));
            metal::uint4 _e119_ = PB.c2_[metal::min(unsigned(_e104_ + 1u), (_buffer_sizes.size2 - 0 - 16) / 16)];
            float _e123_ = as_type<float>(_e119_.z);
            float _e125_ = as_type<float>(_e119_.w);
            uint _e126_ = _e91_.w & 8388608u;
            phi_2303_ = _e71_.z;
            phi_2241_ = _e71_.y;
            phi_2213_ = _e75_;
            local = _e102_;
            if (_e126_ != 0u) {
                phi_2303_ = _e72_.z;
                phi_2241_ = _e72_.y;
                phi_2213_ = naga_f2i32(_e72_.x);
            }
            float _e133_ = phi_2303_;
            float _e135_ = phi_2241_;
            int _e137_ = phi_2213_;
            phi_2224_ = _e86_;
            phi_2222_ = _e91_;
            phi_2221_ = _e91_.w;
            if (_e137_ != _e84_) {
                int _e140_ = as_type<int>(as_type<uint>(as_type<int>(as_type<uint>(_e86_) + as_type<uint>(_e137_))) - as_type<uint>(_e84_));
                uint clamped_lod_e155 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e145_ = LC.read(metal::min(metal::uint2(metal::int2(_e140_ & 2047, _e140_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e155), LC.get_height(clamped_lod_e155)) - 1), clamped_lod_e155);
                if ((_e145_.w & 8454143u) != (_e91_.w & 8454143u)) {
                    bool _e150_ = _e123_ == 0.0;
                    phi_1351_ = _e150_;
                    if (!(_e150_)) {
                        phi_1351_ = _e100_.x != 0.0;
                    }
                    bool _e155_ = phi_1351_;
                    phi_2226_ = _e86_;
                    phi_2218_ = _e91_;
                    if (_e155_) {
                        int _e156_ = as_type<int>(_e98_.w);
                        uint clamped_lod_e180 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                        metal::uint4 _e161_ = LC.read(metal::min(metal::uint2(metal::int2(_e156_ & 2047, _e156_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e180), LC.get_height(clamped_lod_e180)) - 1), clamped_lod_e180);
                        phi_2226_ = _e156_;
                        phi_2218_ = _e161_;
                    }
                    int _e163_ = phi_2226_;
                    metal::uint4 _e165_ = phi_2218_;
                    phi_2225_ = _e163_;
                    phi_2217_ = _e165_;
                } else {
                    phi_2225_ = _e140_;
                    phi_2217_ = _e145_;
                }
                int _e167_ = phi_2225_;
                metal::uint4 _e169_ = phi_2217_;
                phi_2224_ = _e167_;
                phi_2222_ = _e169_;
                phi_2221_ = (_e169_.w & 4286578687u) | _e126_;
            }
            int _e174_ = phi_2224_;
            metal::uint4 _e176_ = phi_2222_;
            uint _e178_ = phi_2221_;
            uint _e179_ = _e178_ & 469762048u;
            if (_e179_ == 67108864u) {
                local_1 = _e82_ == 0;
            } else {
                local_1 = false;
            }
            bool _e182_ = local_1;
            if (_e182_) {
                float _e185_ = static_cast<float>(_e176_.z & 65535u);
                float _e188_ = static_cast<float>(_e176_.z >> as_type<uint>(16));
                metal::int2 _e194_ = metal::int2(naga_f2i32(-1.0 - _e185_), naga_f2i32((_e188_ - _e185_) + 1.0));
                phi_2228_ = _e194_;
                if ((_e178_ & 8388608u) != 0u) {
                    phi_2228_ = naga_neg(_e194_);
                }
                metal::int2 _e199_ = phi_2228_;
                int _e201_ = as_type<int>(as_type<uint>(_e174_) + as_type<uint>(_e199_.x));
                uint clamped_lod_e235 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e206_ = LC.read(metal::min(metal::uint2(metal::int2(_e201_ & 2047, _e201_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e235), LC.get_height(clamped_lod_e235)) - 1), clamped_lod_e235);
                int _e208_ = as_type<int>(as_type<uint>(_e174_) + as_type<uint>(_e199_.y));
                uint clamped_lod_e246 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e213_ = LC.read(metal::min(metal::uint2(metal::int2(_e208_ & 2047, _e208_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e246), LC.get_height(clamped_lod_e246)) - 1), clamped_lod_e246);
                phi_2229_ = _e213_;
                if ((_e213_.w & 8454143u) != (_e206_.w & 8454143u)) {
                    int _e219_ = as_type<int>(_e98_.w);
                    uint clamped_lod_e264 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                    metal::uint4 _e224_ = LC.read(metal::min(metal::uint2(metal::int2(_e219_ & 2047, _e219_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e264), LC.get_height(clamped_lod_e264)) - 1), clamped_lod_e264);
                    phi_2229_ = _e224_;
                }
                metal::uint4 _e226_ = phi_2229_;
                float _e228_ = as_type<float>(_e206_.z);
                float _e230_ = as_type<float>(_e226_.z);
                float _e231_ = _e230_ - _e228_;
                phi_2233_ = _e231_;
                if (metal::abs(_e231_) > 3.1415927) {
                    phi_2233_ = _e231_ - (6.2831855 * metal::sign(_e231_));
                }
                float _e238_ = phi_2233_;
                float _e239_ = _e188_ + -2.0;
                float _e245_ = metal::clamp(metal::rint((metal::abs(_e238_) * 0.31830987) * _e239_), 1.0, _e188_ + -3.0);
                float _e246_ = _e239_ - _e245_;
                if (_e185_ <= _e246_) {
                    phi_2313_ = _e135_;
                    if (_e185_ == _e246_) {
                        phi_2313_ = -(_e135_);
                    }
                    float _e255_ = phi_2313_;
                    phi_2312_ = _e255_;
                    phi_2255_ = -(((3.1415927 * metal::sign(_e238_)) - _e238_));
                    phi_2248_ = _e246_;
                    phi_2245_ = _e185_;
                } else {
                    bool _e257_ = _e185_ == (_e246_ + 1.0);
                    if (_e257_) {
                        phi_2247_ = 0.0;
                    } else {
                        phi_2247_ = _e185_ - (_e246_ + 2.0);
                    }
                    float _e261_ = phi_2247_;
                    phi_2312_ = _e257_ ? 0.0 : _e135_;
                    phi_2255_ = _e238_;
                    phi_2248_ = _e257_ ? 0.0 : _e245_;
                    phi_2245_ = _e261_;
                }
                float _e265_ = phi_2312_;
                float _e267_ = phi_2255_;
                float _e269_ = phi_2248_;
                float _e271_ = phi_2245_;
                if (_e271_ == _e269_) {
                    phi_2259_ = _e230_;
                } else {
                    phi_2259_ = _e228_ + (_e267_ * (_e271_ / _e269_));
                }
                float _e277_ = phi_2259_;
                phi_2334_ = _e228_;
                phi_2325_ = _e267_;
                phi_2310_ = _e265_;
                phi_2258_ = _e277_;
            } else {
                phi_2334_ = float {};
                phi_2325_ = float {};
                phi_2310_ = _e135_;
                phi_2258_ = as_type<float>(_e176_.z);
            }
            float _e281_ = phi_2334_;
            float _e283_ = phi_2325_;
            float _e285_ = phi_2310_;
            float _e287_ = phi_2258_;
            metal::float2 _e291_ = metal::float2(metal::sin(_e287_), -(metal::cos(_e287_)));
            metal::float2 _e293_ = as_type<metal::float2>(_e176_.xy);
            phi_2308_ = _e125_;
            if (_e125_ != 0.0) {
                phi_2308_ = metal::max(_e125_, 1.0 / metal::length(_e115_ * _e291_));
            }
            float _e300_ = phi_2308_;
            if (_e123_ != 0.0) {
                float _e304_ = _e285_ * metal::sign(metal::determinant(_e115_));
                bool _e306_ = (_e178_ & 1048576u) != 0u;
                phi_2391_ = _e304_;
                if (_e306_) {
                    phi_2391_ = metal::min(_e304_, 0.0);
                }
                float _e309_ = phi_2391_;
                phi_2402_ = _e309_;
                if ((_e178_ & 524288u) != 0u) {
                    phi_2402_ = metal::max(_e309_, 0.0);
                }
                float _e314_ = phi_2402_;
                bool _e315_ = _e300_ != 0.0;
                if (_e315_) {
                    phi_2394_ = _e300_;
                } else {
                    metal::float2 _e316_ = _e115_ * _e291_;
                    phi_2394_ = ((metal::abs(_e316_.x) + metal::abs(_e316_.y)) * (1.0 / metal::dot(_e316_, _e316_))) * 0.5;
                }
                float _e327_ = phi_2394_;
                if (_e327_ > _e123_) {
                    local_2 = _e300_ == 0.0;
                } else {
                    local_2 = false;
                }
                bool _e330_ = local_2;
                phi_2483_ = 1.0;
                if (_e330_) {
                    phi_2483_ = _e123_ / _e327_;
                }
                float _e333_ = phi_2483_;
                float _e334_ = _e330_ ? _e327_ : _e123_;
                float _e335_ = _e334_ + _e327_;
                metal::float2 _e336_ = _e291_ * _e335_;
                float _e337_ = _e314_ * _e335_;
                metal::float2 _e344_ = ((metal::float2(_e337_, -(_e337_)) + metal::float2(_e334_)) * (0.5 / _e327_)) + metal::float2(0.5, 0.5);
                metal::float4 _e347_ = metal::float4(_e344_.x, _e344_.y, 0.0, 0.0);
                phi_2508_ = _e336_;
                phi_2492_ = _e347_;
                if (_e179_ > 134217728u) {
                    uint _e349_ = _e178_ & 4194304u;
                    int _e351_ = (_e349_ == 0u) ? -2 : 2;
                    phi_2440_ = _e351_;
                    if ((_e178_ & 8388608u) != 0u) {
                        phi_2440_ = naga_neg(_e351_);
                    }
                    int _e356_ = phi_2440_;
                    int _e357_ = as_type<int>(as_type<uint>(_e174_) + as_type<uint>(_e356_));
                    uint clamped_lod_e431 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                    metal::uint4 _e362_ = LC.read(metal::min(metal::uint2(metal::int2(_e357_ & 2047, _e357_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e431), LC.get_height(clamped_lod_e431)) - 1), clamped_lod_e431);
                    float _e366_ = metal::abs(as_type<float>(_e362_.z) - _e287_);
                    phi_2449_ = _e366_;
                    if (_e366_ > 3.1415927) {
                        phi_2449_ = 6.2831855 - _e366_;
                    }
                    float _e370_ = phi_2449_;
                    float _e375_ = (_e370_ * (((_e349_ != 0u) == _e306_) ? -0.5 : 0.5)) + _e287_;
                    metal::float2 _e379_ = metal::float2(metal::sin(_e375_), -(metal::cos(_e375_)));
                    metal::float2 _e380_ = _e115_ * _e379_;
                    float _e388_ = (metal::abs(_e380_.x) + metal::abs(_e380_.y)) * (1.0 / metal::dot(_e380_, _e380_));
                    float _e390_ = metal::cos(_e370_ * 0.5);
                    bool _e391_ = _e179_ == 335544320u;
                    phi_1690_ = _e391_;
                    if (!(_e391_)) {
                        if (_e179_ == 268435456u) {
                            local_3 = _e390_ >= 0.25;
                        } else {
                            local_3 = false;
                        }
                        bool _e476 = local_3;
                        phi_1690_ = _e476;
                    }
                    bool _e397_ = phi_1690_;
                    if (_e397_) {
                        phi_2456_ = _e334_ * (1.0 / metal::max(_e390_, ((_e178_ & 33554432u) != 0u) ? 1.0 : 0.25));
                    } else {
                        phi_2456_ = (_e334_ * _e390_) + (_e388_ * 0.5);
                    }
                    float _e408_ = phi_2456_;
                    float _e410_ = _e408_ + (_e388_ * 0.5);
                    phi_2471_ = _e336_;
                    if ((_e178_ & 2097152u) != 0u) {
                        if (_e335_ <= ((_e410_ * _e390_) + (_e327_ * 0.125))) {
                            phi_2472_ = _e379_ * (_e335_ * (1.0 / _e390_));
                        } else {
                            metal::float2 _e420_ = _e379_ * _e410_;
                            metal::float2x2 _e515 = _naga_inverse_2x2_f32_(metal::float2x2(_e336_, _e420_));
                            phi_2472_ = metal::float2(metal::dot(_e336_, _e336_), metal::dot(_e420_, _e420_)) * _e515;
                        }
                        metal::float2 _e428_ = phi_2472_;
                        phi_2471_ = _e428_;
                    }
                    metal::float2 _e430_ = phi_2471_;
                    float _e435_ = (_e410_ - metal::dot(_e430_ * metal::abs(_e314_), _e379_)) / _e388_;
                    if (_e306_) {
                        phi_2493_ = metal::float4(_e347_.x, _e435_, _e347_.z, _e347_.w);
                    } else {
                        phi_2493_ = metal::float4(_e435_, _e347_.y, _e347_.z, _e347_.w);
                    }
                    metal::float4 _e447_ = phi_2493_;
                    phi_2508_ = _e430_;
                    phi_2492_ = _e447_;
                }
                metal::float2 _e449_ = phi_2508_;
                metal::float4 _e451_ = phi_2492_;
                metal::float2 _e453_ = _e451_.xy * _e333_;
                metal::float4 _e459_ = metal::float4(_e453_.x, _e451_.y, _e451_.z, _e451_.w);
                metal::float4 _e466_ = metal::float4(_e459_.x, metal::max(_e453_.y, 0.0001), _e459_.z, _e459_.w);
                phi_2539_ = _e466_;
                if (_e315_) {
                    phi_2539_ = metal::float4(-2.0 - _e453_.x, _e466_.y, _e466_.z, _e466_.w);
                }
                metal::float4 _e474_ = phi_2539_;
                if (_e82_ != 0) {
                    phi_2581_ = _e474_;
                    phi_2543_ = metal::float2 {};
                    phi_2542_ = false;
                    break;
                }
                phi_2538_ = _e474_;
                phi_2534_ = _e115_ * (_e449_ * _e314_);
                phi_2510_ = _e293_;
            } else {
                metal::float4 _e478_ = metal::float4(_e133_, -1.0, 0.0, 0.0);
                if (_e300_ != 0.0) {
                    metal::float4 _e484_ = metal::float4(_e478_.x, -2.0, _e478_.z, _e478_.w);
                    metal::float4 _e489_ = metal::float4(_e484_.x, _e484_.y, 1000000.0, _e484_.w);
                    phi_2387_ = metal::float4(_e489_.x, _e489_.y, _e489_.z, _e133_);
                    if (_e182_) {
                        phi_2344_ = _e283_;
                        phi_2343_ = _e281_;
                        if (_e283_ < 0.0) {
                            phi_2344_ = -(_e283_);
                            phi_2343_ = _e281_ + _e283_;
                        }
                        float _e499_ = phi_2344_;
                        float _e501_ = phi_2343_;
                        float _e503_ = (_e287_ - _e501_) + 1.5707964;
                        float _e509_ = metal::clamp((_e503_ - (metal::floor(_e503_ / 6.2831855) * 6.2831855)) - 1.5707964, 0.0, _e499_);
                        phi_2345_ = _e509_;
                        if (_e509_ > (_e499_ * 0.5)) {
                            phi_2345_ = _e499_ - _e509_;
                        }
                        float _e514_ = phi_2345_;
                        metal::float2 _e521_ = (metal::float2(1.0, 1.0) - (metal::float2(metal::sin(_e514_), metal::cos(_e514_)) * metal::abs(_e285_))) * 0.5;
                        if (metal::abs(_e499_ - 1.5707964) < 0.001) {
                            phi_2371_ = 0.0;
                            phi_2369_ = 0.0;
                        } else {
                            float _e525_ = metal::tan(_e499_);
                            float _e530_ = metal::sign(1.5707964 - _e499_) / metal::max(metal::abs(_e525_), 0.000001);
                            if (_e530_ >= 0.0) {
                                phi_2349_ = _e521_.y - ((1.0 - _e521_.x) * _e525_);
                            } else {
                                phi_2349_ = _e521_.y + (_e521_.x * _e525_);
                            }
                            float _e542_ = phi_2349_;
                            phi_2371_ = _e542_;
                            phi_2369_ = _e530_;
                        }
                        float _e544_ = phi_2371_;
                        float _e546_ = phi_2369_;
                        phi_2387_ = metal::float4(metal::max(_e521_.x, 0.0) + 0.25, -2.0 - _e521_.y, _e546_, _e544_);
                    }
                    metal::float4 _e554_ = phi_2387_;
                    phi_2537_ = _e115_ * (_e291_ * (_e285_ * _e300_));
                    phi_2386_ = _e554_;
                } else {
                    metal::float2x2 _e662 = _naga_inverse_2x2_f32_(_e115_);
                    phi_2537_ = metal::sign((_e291_ * _e285_) * _e662) * 0.5;
                    phi_2386_ = _e478_;
                }
                metal::float2 _e564_ = phi_2537_;
                metal::float4 _e566_ = phi_2386_;
                phi_2541_ = _e566_;
                if (((_e178_ & 8388608u) != 0u) != ((_e178_ & 16777216u) != 0u)) {
                    phi_2541_ = _e566_ * metal::float4(-1.0, 1.0, 1.0, 1.0);
                }
                metal::float4 _e574_ = phi_2541_;
                if ((_e178_ & 2147483648u) != 0u) {
                    local_4 = _e82_ != 1;
                } else {
                    local_4 = false;
                }
                bool _e694 = local_4;
                if (_e694) {
                    phi_2581_ = _e574_;
                    phi_2543_ = metal::float2 {};
                    phi_2542_ = false;
                    break;
                }
                phi_2538_ = _e574_;
                phi_2534_ = _e564_;
                phi_2510_ = metal::select(_e293_, _e100_, metal::bool2(_e82_ == 2));
            }
            metal::float4 _e583_ = phi_2538_;
            metal::float2 _e585_ = phi_2534_;
            metal::float2 _e587_ = phi_2510_;
            uint _e593_ = n.yg;
            metal::float2 _e596_ = metal::select(_e583_.xy, metal::float2(1.0, -1.0), metal::bool2(_e593_ != 0u));
            metal::float4 _e602_ = metal::float4(_e596_.x, _e583_.y, _e583_.z, _e583_.w);
            phi_2581_ = metal::float4(_e602_.x, _e596_.y, _e602_.z, _e602_.w);
            phi_2543_ = ((_e115_ * _e587_) + _e585_) + as_type<metal::float2>(_e119_.xy);
            phi_2542_ = true;
            break;
        }
    }
    metal::float4 _e610_ = phi_2581_;
    metal::float2 _e612_ = phi_2543_;
    bool _e614_ = phi_2542_;
    if (_e614_) {
        O = _e610_;
        uint _e616_ = local;
        B0_ = _e616_;
        float _e618_ = n.ff;
        float _e620_ = n.gf;
        phi_2582_ = metal::float4((_e612_.x * _e618_) - 1.0, (_e612_.y * _e620_) - metal::sign(_e620_), 0.0, 1.0);
    } else {
        float _e630_ = n.P2_;
        phi_2582_ = metal::float4(_e630_);
    }
    metal::float4 _e633_ = phi_2582_;
    unnamed.gl_Position = _e633_;
    return;
}

struct main_Output {
    metal::float4 member [[user(loc0), center_perspective]];
    uint member_1_ [[user(loc1), flat]];
    metal::float4 gl_Position [[position]];
};
struct vb_30_type { metal::uchar data[32]; };
vertex main_Output main_(
  uint gl_VertexIndex [[vertex_id]]
, uint gl_InstanceIndex [[instance_id]]
, metal::texture2d<uint, metal::access::sample> LC [[texture(0)]]
, device cg const& ED [[buffer(2)]]
, device bg const& PB [[buffer(1)]]
, constant CC& n [[buffer(0)]]
, const device vb_30_type* vb_30_in [[buffer(30)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(3)]]
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
    uint B0_ = {};
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_8 {}, type_8 {}};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    gl_InstanceIndex_1_ = static_cast<int>(gl_InstanceIndex);
    UB_1_ = UB;
    VB_1_ = VB;
    main_1_(LC, ED, PB, n, gl_InstanceIndex_1_, UB_1_, VB_1_, O, B0_, unnamed, _buffer_sizes);
    metal::float4 _e14_ = O;
    uint _e15_ = B0_;
    metal::float4 _e16_ = unnamed.gl_Position;
    const auto _tmp = VertexOutput {_e14_, _e15_, {}, _e16_};
    return main_Output { _tmp.member, _tmp.member_1_, _tmp.gl_Position };
}
