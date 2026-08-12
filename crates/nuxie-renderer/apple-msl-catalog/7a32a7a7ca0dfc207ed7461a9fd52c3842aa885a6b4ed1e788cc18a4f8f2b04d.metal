// language: metal3.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size1;
    uint size2;
    uint size11;
    uint size12;
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
    thread gl_PerVertex& unnamed,
    constant _mslBufferSizes& _buffer_sizes
) {
    float phi_2290_ = {};
    float phi_2228_ = {};
    int phi_2200_ = {};
    bool phi_1342_ = {};
    int phi_2213_ = {};
    metal::uint4 phi_2205_ = {};
    int phi_2212_ = {};
    metal::uint4 phi_2204_ = {};
    int phi_2211_ = {};
    metal::uint4 phi_2209_ = {};
    uint phi_2208_ = {};
    metal::int2 phi_2215_ = {};
    metal::uint4 phi_2216_ = {};
    float phi_2220_ = {};
    float phi_2300_ = {};
    float phi_2234_ = {};
    float phi_2299_ = {};
    float phi_2242_ = {};
    float phi_2235_ = {};
    float phi_2232_ = {};
    float phi_2246_ = {};
    float phi_2321_ = {};
    float phi_2312_ = {};
    float phi_2297_ = {};
    float phi_2245_ = {};
    float phi_2295_ = {};
    float phi_2378_ = {};
    float phi_2389_ = {};
    float phi_2381_ = {};
    float phi_2470_ = {};
    int phi_2427_ = {};
    float phi_2436_ = {};
    bool phi_1681_ = {};
    float phi_2443_ = {};
    metal::float2 phi_2459_ = {};
    metal::float2 phi_2458_ = {};
    metal::float4 phi_2480_ = {};
    metal::float2 phi_2495_ = {};
    metal::float4 phi_2479_ = {};
    metal::float4 phi_2526_ = {};
    float phi_2331_ = {};
    float phi_2330_ = {};
    float phi_2332_ = {};
    float phi_2336_ = {};
    float phi_2358_ = {};
    float phi_2356_ = {};
    metal::float4 phi_2374_ = {};
    metal::float2 phi_2524_ = {};
    metal::float4 phi_2373_ = {};
    metal::float4 phi_2528_ = {};
    metal::float4 phi_2525_ = {};
    metal::float2 phi_2521_ = {};
    metal::float2 phi_2497_ = {};
    metal::float4 phi_2568_ = {};
    metal::float2 phi_2530_ = {};
    bool phi_2529_ = {};
    uint local = {};
    metal::float4 phi_2569_ = {};
    bool local_1 = {};
    bool local_2 = {};
    bool local_3 = {};
    bool local_4 = {};
    int _e71_ = gl_InstanceIndex_1_;
    metal::float4 _e72_ = UB_1_;
    metal::float4 _e73_ = VB_1_;
    switch(as_type<int>(0u)) {
        default: {
            int _e76_ = naga_f2i32(_e72_.x);
            int _e80_ = as_type<int>(_e72_.w);
            int _e82_ = _e80_ >> as_type<uint>(2);
            int _e83_ = _e80_ & 3;
            int _e85_ = metal::min(_e76_, as_type<int>(as_type<uint>(_e82_) - as_type<uint>(1)));
            int _e87_ = as_type<int>(as_type<uint>(as_type<int>(as_type<uint>(_e71_) * as_type<uint>(_e82_))) + as_type<uint>(_e85_));
            uint clamped_lod_e88 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
            metal::uint4 _e92_ = LC.read(metal::min(metal::uint2(metal::int2(_e87_ & 2047, _e87_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e88), LC.get_height(clamped_lod_e88)) - 1), clamped_lod_e88);
            metal::uint4 _e99_ = ED.c2_[metal::min(unsigned(metal::max(_e92_.w & 65535u, 1u) - 1u), (_buffer_sizes.size1 - 0 - 16) / 16)];
            metal::float2 _e101_ = as_type<metal::float2>(_e99_.xy);
            uint _e105_ = (_e99_.z & 65535u) * 4u;
            metal::uint4 _e108_ = PB.c2_[metal::min(unsigned(_e105_), (_buffer_sizes.size2 - 0 - 16) / 16)];
            metal::float4 _e109_ = as_type<metal::float4>(_e108_);
            metal::float2x2 _e116_ = metal::float2x2(metal::float2(_e109_.x, _e109_.y), metal::float2(_e109_.z, _e109_.w));
            metal::uint4 _e120_ = PB.c2_[metal::min(unsigned(_e105_ + 1u), (_buffer_sizes.size2 - 0 - 16) / 16)];
            float _e124_ = as_type<float>(_e120_.z);
            float _e126_ = as_type<float>(_e120_.w);
            uint _e127_ = _e92_.w & 8388608u;
            phi_2290_ = _e72_.z;
            phi_2228_ = _e72_.y;
            phi_2200_ = _e76_;
            local = _e105_;
            if (_e127_ != 0u) {
                phi_2290_ = _e73_.z;
                phi_2228_ = _e73_.y;
                phi_2200_ = naga_f2i32(_e73_.x);
            }
            float _e134_ = phi_2290_;
            float _e136_ = phi_2228_;
            int _e138_ = phi_2200_;
            phi_2211_ = _e87_;
            phi_2209_ = _e92_;
            phi_2208_ = _e92_.w;
            if (_e138_ != _e85_) {
                int _e141_ = as_type<int>(as_type<uint>(as_type<int>(as_type<uint>(_e87_) + as_type<uint>(_e138_))) - as_type<uint>(_e85_));
                uint clamped_lod_e155 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e146_ = LC.read(metal::min(metal::uint2(metal::int2(_e141_ & 2047, _e141_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e155), LC.get_height(clamped_lod_e155)) - 1), clamped_lod_e155);
                if ((_e146_.w & 8454143u) != (_e92_.w & 8454143u)) {
                    bool _e151_ = _e124_ == 0.0;
                    phi_1342_ = _e151_;
                    if (!(_e151_)) {
                        phi_1342_ = _e101_.x != 0.0;
                    }
                    bool _e156_ = phi_1342_;
                    phi_2213_ = _e87_;
                    phi_2205_ = _e92_;
                    if (_e156_) {
                        int _e157_ = as_type<int>(_e99_.w);
                        uint clamped_lod_e180 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                        metal::uint4 _e162_ = LC.read(metal::min(metal::uint2(metal::int2(_e157_ & 2047, _e157_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e180), LC.get_height(clamped_lod_e180)) - 1), clamped_lod_e180);
                        phi_2213_ = _e157_;
                        phi_2205_ = _e162_;
                    }
                    int _e164_ = phi_2213_;
                    metal::uint4 _e166_ = phi_2205_;
                    phi_2212_ = _e164_;
                    phi_2204_ = _e166_;
                } else {
                    phi_2212_ = _e141_;
                    phi_2204_ = _e146_;
                }
                int _e168_ = phi_2212_;
                metal::uint4 _e170_ = phi_2204_;
                phi_2211_ = _e168_;
                phi_2209_ = _e170_;
                phi_2208_ = (_e170_.w & 4286578687u) | _e127_;
            }
            int _e175_ = phi_2211_;
            metal::uint4 _e177_ = phi_2209_;
            uint _e179_ = phi_2208_;
            uint _e180_ = _e179_ & 469762048u;
            if (_e180_ == 67108864u) {
                local_1 = _e83_ == 0;
            } else {
                local_1 = false;
            }
            bool _e183_ = local_1;
            if (_e183_) {
                float _e186_ = static_cast<float>(_e177_.z & 65535u);
                float _e189_ = static_cast<float>(_e177_.z >> as_type<uint>(16));
                metal::int2 _e195_ = metal::int2(naga_f2i32(-1.0 - _e186_), naga_f2i32((_e189_ - _e186_) + 1.0));
                phi_2215_ = _e195_;
                if ((_e179_ & 8388608u) != 0u) {
                    phi_2215_ = naga_neg(_e195_);
                }
                metal::int2 _e200_ = phi_2215_;
                int _e202_ = as_type<int>(as_type<uint>(_e175_) + as_type<uint>(_e200_.x));
                uint clamped_lod_e235 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e207_ = LC.read(metal::min(metal::uint2(metal::int2(_e202_ & 2047, _e202_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e235), LC.get_height(clamped_lod_e235)) - 1), clamped_lod_e235);
                int _e209_ = as_type<int>(as_type<uint>(_e175_) + as_type<uint>(_e200_.y));
                uint clamped_lod_e246 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                metal::uint4 _e214_ = LC.read(metal::min(metal::uint2(metal::int2(_e209_ & 2047, _e209_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e246), LC.get_height(clamped_lod_e246)) - 1), clamped_lod_e246);
                phi_2216_ = _e214_;
                if ((_e214_.w & 8454143u) != (_e207_.w & 8454143u)) {
                    int _e220_ = as_type<int>(_e99_.w);
                    uint clamped_lod_e264 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                    metal::uint4 _e225_ = LC.read(metal::min(metal::uint2(metal::int2(_e220_ & 2047, _e220_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e264), LC.get_height(clamped_lod_e264)) - 1), clamped_lod_e264);
                    phi_2216_ = _e225_;
                }
                metal::uint4 _e227_ = phi_2216_;
                float _e229_ = as_type<float>(_e207_.z);
                float _e231_ = as_type<float>(_e227_.z);
                float _e232_ = _e231_ - _e229_;
                phi_2220_ = _e232_;
                if (metal::abs(_e232_) > 3.1415927) {
                    phi_2220_ = _e232_ - (6.2831855 * metal::sign(_e232_));
                }
                float _e239_ = phi_2220_;
                float _e240_ = _e189_ + -2.0;
                float _e246_ = metal::clamp(metal::rint((metal::abs(_e239_) * 0.31830987) * _e240_), 1.0, _e189_ + -3.0);
                float _e247_ = _e240_ - _e246_;
                if (_e186_ <= _e247_) {
                    phi_2300_ = _e136_;
                    if (_e186_ == _e247_) {
                        phi_2300_ = -(_e136_);
                    }
                    float _e256_ = phi_2300_;
                    phi_2299_ = _e256_;
                    phi_2242_ = -(((3.1415927 * metal::sign(_e239_)) - _e239_));
                    phi_2235_ = _e247_;
                    phi_2232_ = _e186_;
                } else {
                    bool _e258_ = _e186_ == (_e247_ + 1.0);
                    if (_e258_) {
                        phi_2234_ = 0.0;
                    } else {
                        phi_2234_ = _e186_ - (_e247_ + 2.0);
                    }
                    float _e262_ = phi_2234_;
                    phi_2299_ = _e258_ ? 0.0 : _e136_;
                    phi_2242_ = _e239_;
                    phi_2235_ = _e258_ ? 0.0 : _e246_;
                    phi_2232_ = _e262_;
                }
                float _e266_ = phi_2299_;
                float _e268_ = phi_2242_;
                float _e270_ = phi_2235_;
                float _e272_ = phi_2232_;
                if (_e272_ == _e270_) {
                    phi_2246_ = _e231_;
                } else {
                    phi_2246_ = _e229_ + (_e268_ * (_e272_ / _e270_));
                }
                float _e278_ = phi_2246_;
                phi_2321_ = _e229_;
                phi_2312_ = _e268_;
                phi_2297_ = _e266_;
                phi_2245_ = _e278_;
            } else {
                phi_2321_ = float {};
                phi_2312_ = float {};
                phi_2297_ = _e136_;
                phi_2245_ = as_type<float>(_e177_.z);
            }
            float _e282_ = phi_2321_;
            float _e284_ = phi_2312_;
            float _e286_ = phi_2297_;
            float _e288_ = phi_2245_;
            metal::float2 _e292_ = metal::float2(metal::sin(_e288_), -(metal::cos(_e288_)));
            metal::float2 _e294_ = as_type<metal::float2>(_e177_.xy);
            phi_2295_ = _e126_;
            if (_e126_ != 0.0) {
                phi_2295_ = metal::max(_e126_, 1.0 / metal::length(_e116_ * _e292_));
            }
            float _e301_ = phi_2295_;
            if (_e124_ != 0.0) {
                float _e305_ = _e286_ * metal::sign(metal::determinant(_e116_));
                bool _e307_ = (_e179_ & 1048576u) != 0u;
                phi_2378_ = _e305_;
                if (_e307_) {
                    phi_2378_ = metal::min(_e305_, 0.0);
                }
                float _e310_ = phi_2378_;
                phi_2389_ = _e310_;
                if ((_e179_ & 524288u) != 0u) {
                    phi_2389_ = metal::max(_e310_, 0.0);
                }
                float _e315_ = phi_2389_;
                bool _e316_ = _e301_ != 0.0;
                if (_e316_) {
                    phi_2381_ = _e301_;
                } else {
                    metal::float2 _e317_ = _e116_ * _e292_;
                    phi_2381_ = ((metal::abs(_e317_.x) + metal::abs(_e317_.y)) * (1.0 / metal::dot(_e317_, _e317_))) * 0.5;
                }
                float _e328_ = phi_2381_;
                if (_e328_ > _e124_) {
                    local_2 = _e301_ == 0.0;
                } else {
                    local_2 = false;
                }
                bool _e331_ = local_2;
                phi_2470_ = 1.0;
                if (_e331_) {
                    phi_2470_ = _e124_ / _e328_;
                }
                float _e334_ = phi_2470_;
                float _e335_ = _e331_ ? _e328_ : _e124_;
                float _e336_ = _e335_ + _e328_;
                metal::float2 _e337_ = _e292_ * _e336_;
                float _e338_ = _e315_ * _e336_;
                metal::float2 _e345_ = ((metal::float2(_e338_, -(_e338_)) + metal::float2(_e335_)) * (0.5 / _e328_)) + metal::float2(0.5, 0.5);
                metal::float4 _e348_ = metal::float4(_e345_.x, _e345_.y, 0.0, 0.0);
                phi_2495_ = _e337_;
                phi_2479_ = _e348_;
                if (_e180_ > 134217728u) {
                    uint _e350_ = _e179_ & 4194304u;
                    int _e352_ = (_e350_ == 0u) ? -2 : 2;
                    phi_2427_ = _e352_;
                    if ((_e179_ & 8388608u) != 0u) {
                        phi_2427_ = naga_neg(_e352_);
                    }
                    int _e357_ = phi_2427_;
                    int _e358_ = as_type<int>(as_type<uint>(_e175_) + as_type<uint>(_e357_));
                    uint clamped_lod_e431 = metal::min(uint(0), LC.get_num_mip_levels() - 1);
                    metal::uint4 _e363_ = LC.read(metal::min(metal::uint2(metal::int2(_e358_ & 2047, _e358_ >> as_type<uint>(11))), metal::uint2(LC.get_width(clamped_lod_e431), LC.get_height(clamped_lod_e431)) - 1), clamped_lod_e431);
                    float _e367_ = metal::abs(as_type<float>(_e363_.z) - _e288_);
                    phi_2436_ = _e367_;
                    if (_e367_ > 3.1415927) {
                        phi_2436_ = 6.2831855 - _e367_;
                    }
                    float _e371_ = phi_2436_;
                    float _e376_ = (_e371_ * (((_e350_ != 0u) == _e307_) ? -0.5 : 0.5)) + _e288_;
                    metal::float2 _e380_ = metal::float2(metal::sin(_e376_), -(metal::cos(_e376_)));
                    metal::float2 _e381_ = _e116_ * _e380_;
                    float _e389_ = (metal::abs(_e381_.x) + metal::abs(_e381_.y)) * (1.0 / metal::dot(_e381_, _e381_));
                    float _e391_ = metal::cos(_e371_ * 0.5);
                    bool _e392_ = _e180_ == 335544320u;
                    phi_1681_ = _e392_;
                    if (!(_e392_)) {
                        if (_e180_ == 268435456u) {
                            local_3 = _e391_ >= 0.25;
                        } else {
                            local_3 = false;
                        }
                        bool _e476 = local_3;
                        phi_1681_ = _e476;
                    }
                    bool _e398_ = phi_1681_;
                    if (_e398_) {
                        phi_2443_ = _e335_ * (1.0 / metal::max(_e391_, ((_e179_ & 33554432u) != 0u) ? 1.0 : 0.25));
                    } else {
                        phi_2443_ = (_e335_ * _e391_) + (_e389_ * 0.5);
                    }
                    float _e409_ = phi_2443_;
                    float _e411_ = _e409_ + (_e389_ * 0.5);
                    phi_2458_ = _e337_;
                    if ((_e179_ & 2097152u) != 0u) {
                        if (_e336_ <= ((_e411_ * _e391_) + (_e328_ * 0.125))) {
                            phi_2459_ = _e380_ * (_e336_ * (1.0 / _e391_));
                        } else {
                            metal::float2 _e421_ = _e380_ * _e411_;
                            metal::float2x2 _e515 = _naga_inverse_2x2_f32_(metal::float2x2(_e337_, _e421_));
                            phi_2459_ = metal::float2(metal::dot(_e337_, _e337_), metal::dot(_e421_, _e421_)) * _e515;
                        }
                        metal::float2 _e429_ = phi_2459_;
                        phi_2458_ = _e429_;
                    }
                    metal::float2 _e431_ = phi_2458_;
                    float _e436_ = (_e411_ - metal::dot(_e431_ * metal::abs(_e315_), _e380_)) / _e389_;
                    if (_e307_) {
                        phi_2480_ = metal::float4(_e348_.x, _e436_, _e348_.z, _e348_.w);
                    } else {
                        phi_2480_ = metal::float4(_e436_, _e348_.y, _e348_.z, _e348_.w);
                    }
                    metal::float4 _e448_ = phi_2480_;
                    phi_2495_ = _e431_;
                    phi_2479_ = _e448_;
                }
                metal::float2 _e450_ = phi_2495_;
                metal::float4 _e452_ = phi_2479_;
                metal::float2 _e454_ = _e452_.xy * _e334_;
                metal::float4 _e460_ = metal::float4(_e454_.x, _e452_.y, _e452_.z, _e452_.w);
                metal::float4 _e467_ = metal::float4(_e460_.x, metal::max(_e454_.y, 0.0001), _e460_.z, _e460_.w);
                phi_2526_ = _e467_;
                if (_e316_) {
                    phi_2526_ = metal::float4(-2.0 - _e454_.x, _e467_.y, _e467_.z, _e467_.w);
                }
                metal::float4 _e475_ = phi_2526_;
                if (_e83_ != 0) {
                    phi_2568_ = _e475_;
                    phi_2530_ = metal::float2 {};
                    phi_2529_ = false;
                    break;
                }
                phi_2525_ = _e475_;
                phi_2521_ = _e116_ * (_e450_ * _e315_);
                phi_2497_ = _e294_;
            } else {
                metal::float4 _e479_ = metal::float4(_e134_, -1.0, 0.0, 0.0);
                if (_e301_ != 0.0) {
                    metal::float4 _e485_ = metal::float4(_e479_.x, -2.0, _e479_.z, _e479_.w);
                    metal::float4 _e490_ = metal::float4(_e485_.x, _e485_.y, 1000000.0, _e485_.w);
                    phi_2374_ = metal::float4(_e490_.x, _e490_.y, _e490_.z, _e134_);
                    if (_e183_) {
                        phi_2331_ = _e284_;
                        phi_2330_ = _e282_;
                        if (_e284_ < 0.0) {
                            phi_2331_ = -(_e284_);
                            phi_2330_ = _e282_ + _e284_;
                        }
                        float _e500_ = phi_2331_;
                        float _e502_ = phi_2330_;
                        float _e504_ = (_e288_ - _e502_) + 1.5707964;
                        float _e510_ = metal::clamp((_e504_ - (metal::floor(_e504_ / 6.2831855) * 6.2831855)) - 1.5707964, 0.0, _e500_);
                        phi_2332_ = _e510_;
                        if (_e510_ > (_e500_ * 0.5)) {
                            phi_2332_ = _e500_ - _e510_;
                        }
                        float _e515_ = phi_2332_;
                        metal::float2 _e522_ = (metal::float2(1.0, 1.0) - (metal::float2(metal::sin(_e515_), metal::cos(_e515_)) * metal::abs(_e286_))) * 0.5;
                        if (metal::abs(_e500_ - 1.5707964) < 0.001) {
                            phi_2358_ = 0.0;
                            phi_2356_ = 0.0;
                        } else {
                            float _e526_ = metal::tan(_e500_);
                            float _e531_ = metal::sign(1.5707964 - _e500_) / metal::max(metal::abs(_e526_), 0.000001);
                            if (_e531_ >= 0.0) {
                                phi_2336_ = _e522_.y - ((1.0 - _e522_.x) * _e526_);
                            } else {
                                phi_2336_ = _e522_.y + (_e522_.x * _e526_);
                            }
                            float _e543_ = phi_2336_;
                            phi_2358_ = _e543_;
                            phi_2356_ = _e531_;
                        }
                        float _e545_ = phi_2358_;
                        float _e547_ = phi_2356_;
                        phi_2374_ = metal::float4(metal::max(_e522_.x, 0.0) + 0.25, -2.0 - _e522_.y, _e547_, _e545_);
                    }
                    metal::float4 _e555_ = phi_2374_;
                    phi_2524_ = _e116_ * (_e292_ * (_e286_ * _e301_));
                    phi_2373_ = _e555_;
                } else {
                    metal::float2x2 _e662 = _naga_inverse_2x2_f32_(_e116_);
                    phi_2524_ = metal::sign((_e292_ * _e286_) * _e662) * 0.5;
                    phi_2373_ = _e479_;
                }
                metal::float2 _e565_ = phi_2524_;
                metal::float4 _e567_ = phi_2373_;
                phi_2528_ = _e567_;
                if (((_e179_ & 8388608u) != 0u) != ((_e179_ & 16777216u) != 0u)) {
                    phi_2528_ = _e567_ * metal::float4(-1.0, 1.0, 1.0, 1.0);
                }
                metal::float4 _e575_ = phi_2528_;
                if ((_e179_ & 2147483648u) != 0u) {
                    local_4 = _e83_ != 1;
                } else {
                    local_4 = false;
                }
                bool _e694 = local_4;
                if (_e694) {
                    phi_2568_ = _e575_;
                    phi_2530_ = metal::float2 {};
                    phi_2529_ = false;
                    break;
                }
                phi_2525_ = _e575_;
                phi_2521_ = _e565_;
                phi_2497_ = metal::select(_e294_, _e101_, metal::bool2(_e83_ == 2));
            }
            metal::float4 _e584_ = phi_2525_;
            metal::float2 _e586_ = phi_2521_;
            metal::float2 _e588_ = phi_2497_;
            uint _e594_ = n.yg;
            metal::float2 _e597_ = metal::select(_e584_.xy, metal::float2(1.0, -1.0), metal::bool2(_e594_ != 0u));
            metal::float4 _e603_ = metal::float4(_e597_.x, _e584_.y, _e584_.z, _e584_.w);
            phi_2568_ = metal::float4(_e603_.x, _e597_.y, _e603_.z, _e603_.w);
            phi_2530_ = ((_e116_ * _e588_) + _e586_) + as_type<metal::float2>(_e120_.xy);
            phi_2529_ = true;
            break;
        }
    }
    metal::float4 _e611_ = phi_2568_;
    metal::float2 _e613_ = phi_2530_;
    bool _e615_ = phi_2529_;
    O = _e611_;
    if (_e615_) {
        uint _e617_ = local;
        metal::uint4 _e621_ = PB.c2_[metal::min(unsigned(_e617_ + 2u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float3 _e623_ = as_type<metal::float3>(_e621_.yzw);
        metal::float2 _e627_ = (_e613_ * _e623_.x) + _e623_.yz;
        float _e630_ = n.pd.x;
        float _e633_ = n.pd.y;
        phi_2569_ = metal::float4((_e627_.x * _e630_) - 1.0, (_e627_.y * _e633_) - metal::sign(_e633_), 0.0, 1.0);
    } else {
        float _e643_ = n.P2_;
        phi_2569_ = metal::float4(_e643_);
    }
    metal::float4 _e646_ = phi_2569_;
    unnamed.gl_Position = _e646_;
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
, metal::texture2d<uint, metal::access::sample> LC [[texture(0)]]
, device cg const& ED [[buffer(4)]]
, device bg const& PB [[buffer(1)]]
, constant CC& n [[buffer(0)]]
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
    metal::float4 O = {};
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_8 {}, type_8 {}};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    gl_InstanceIndex_1_ = static_cast<int>(gl_InstanceIndex);
    UB_1_ = UB;
    VB_1_ = VB;
    main_1_(LC, ED, PB, n, gl_InstanceIndex_1_, UB_1_, VB_1_, O, unnamed, _buffer_sizes);
    metal::float4 _e13_ = O;
    metal::float4 _e14_ = unnamed.gl_Position;
    const auto _tmp = VertexOutput {_e13_, _e14_};
    return main_Output { _tmp.member, _tmp.gl_Position };
}
