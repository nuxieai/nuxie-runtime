// language: metal3.1
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
    metal::float4 member_1_;
    metal::float4 member_2_;
    metal::packed_float3 member_3_;
    uint member_4_;
    metal::float4 gl_Position;
};
metal::float4 unpackFloat32x4_(uint b0, uint b1, uint b2, uint b3, uint b4, uint b5, uint b6, uint b7, uint b8, uint b9, uint b10, uint b11, uint b12, uint b13, uint b14, uint b15) {
    return metal::float4(as_type<float>(b3 << 24 | b2 << 16 | b1 << 8 | b0), as_type<float>(b7 << 24 | b6 << 16 | b5 << 8 | b4), as_type<float>(b11 << 24 | b10 << 16 | b9 << 8 | b8), as_type<float>(b15 << 24 | b14 << 16 | b13 << 8 | b12));
}
metal::uint4 unpackUint32x4_(uint b0, uint b1, uint b2, uint b3, uint b4, uint b5, uint b6, uint b7, uint b8, uint b9, uint b10, uint b11, uint b12, uint b13, uint b14, uint b15) {
    return metal::uint4((b3 << 24 | b2 << 16 | b1 << 8 | b0), (b7 << 24 | b6 << 16 | b5 << 8 | b4), (b11 << 24 | b10 << 16 | b9 << 8 | b8), (b15 << 24 | b14 << 16 | b13 << 8 | b12));
}

uint naga_f2u32(float value) {
    return static_cast<uint>(metal::clamp(value, 0.0, 4294967000.0));
}

void main_1_(
    thread int& gl_VertexIndex_1_,
    thread metal::float4& GD_1_,
    thread metal::float4& HD_1_,
    thread metal::float4& UC_1_,
    thread metal::uint4& TB_1_,
    constant CC& n,
    metal::texture2d<uint, metal::access::sample> ED,
    metal::texture2d<uint, metal::access::sample> PB,
    metal::texture2d<float, metal::access::sample> XC,
    metal::sampler aa,
    thread metal::float4& x6_,
    thread metal::float4& y6_,
    thread metal::float4& L4_,
    thread metal::float3& C5_,
    thread uint& F7_,
    thread gl_PerVertex& unnamed
) {
    float phi_1791_ = {};
    uint phi_1792_ = {};
    int phi_1793_ = {};
    float phi_1794_ = {};
    metal::float2 phi_1991_ = {};
    uint phi_1795_ = {};
    metal::uint4 phi_1796_ = {};
    float phi_1812_ = {};
    int phi_1811_ = {};
    float phi_1813_ = {};
    float phi_1815_ = {};
    float phi_1816_ = {};
    float local = {};
    float local_1_ = {};
    float local_2_ = {};
    float phi_1821_ = {};
    float phi_1820_ = {};
    metal::float2 local_3_ = {};
    float phi_1832_ = {};
    float phi_1833_ = {};
    float phi_1835_ = {};
    float phi_1834_ = {};
    metal::float2 phi_1836_ = {};
    metal::float2 phi_1855_ = {};
    metal::float2 phi_1857_ = {};
    metal::float2 phi_1858_ = {};
    float phi_1859_ = {};
    float phi_1860_ = {};
    metal::float2 local_4_ = {};
    metal::float2 local_5_ = {};
    float phi_1861_ = {};
    float phi_1863_ = {};
    float phi_1864_ = {};
    metal::float2 phi_1913_ = {};
    metal::float2 phi_1912_ = {};
    uint phi_1935_ = {};
    metal::float2 phi_1938_ = {};
    metal::float2 phi_1967_ = {};
    float phi_1969_ = {};
    float phi_1976_ = {};
    float phi_2015_ = {};
    float phi_2016_ = {};
    float phi_2024_ = {};
    float phi_2025_ = {};
    uint phi_2029_ = {};
    float local_6_ = {};
    float local_7_ = {};
    bool local_1 = {};
    int _e61_ = gl_VertexIndex_1_;
    metal::float4 _e62_ = GD_1_;
    metal::float2 _e63_ = _e62_.xy;
    metal::float2 _e64_ = _e62_.zw;
    metal::float4 _e65_ = HD_1_;
    metal::float2 _e66_ = _e65_.xy;
    metal::float2 _e67_ = _e65_.zw;
    bool _e68_ = _e61_ < 4;
    if (_e68_) {
        float _e70_ = UC_1_.z;
        phi_1791_ = _e70_;
    } else {
        float _e72_ = UC_1_.w;
        phi_1791_ = _e72_;
    }
    float _e74_ = phi_1791_;
    if (_e68_) {
        uint _e76_ = TB_1_.x;
        phi_1792_ = _e76_;
    } else {
        uint _e78_ = TB_1_.y;
        phi_1792_ = _e78_;
    }
    uint _e80_ = phi_1792_;
    int _e81_ = as_type<int>(_e80_);
    int _e83_ = _e81_ << as_type<uint>(16);
    uint _e85_ = TB_1_.z;
    phi_1793_ = _e83_;
    if (_e85_ == 4294967295u) {
        phi_1793_ = as_type<int>(as_type<uint>(_e83_) - as_type<uint>(1));
    }
    int _e89_ = phi_1793_;
    float _e92_ = static_cast<float>(_e89_ >> as_type<uint>(16));
    float _e95_ = static_cast<float>(_e81_ >> as_type<uint>(16));
    if ((_e61_ & 2) == 0) {
        phi_1794_ = _e74_ + 1.0;
    } else {
        phi_1794_ = _e74_;
    }
    float _e103_ = phi_1794_;
    metal::float2 _e104_ = metal::float2(((_e61_ & 1) == 0) ? _e92_ : _e95_, _e103_);
    float _e107_ = n.od;
    phi_1991_ = _e104_;
    if (((_e95_ - _e92_) * _e107_) < 0.0) {
        phi_1991_ = metal::float2(_e104_.x, ((2.0 * _e74_) + 1.0) - _e103_);
    }
    metal::float2 _e117_ = phi_1991_;
    uint _e118_ = _e85_ & 1023u;
    uint _e121_ = (_e85_ >> as_type<uint>(10)) & 1023u;
    uint _e123_ = _e85_ >> as_type<uint>(20);
    uint _e125_ = TB_1_.w;
    uint _e126_ = _e125_ & 65535u;
    if (_e126_ > 0u) {
        uint _e129_ = metal::max(_e126_, 1u) - 1u;
        uint clamped_lod_e152 = metal::min(uint(0), ED.get_num_mip_levels() - 1);
        metal::uint4 _e136_ = ED.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e129_ & 127u), as_type<int>(_e129_ >> as_type<uint>(7)))), metal::uint2(ED.get_width(clamped_lod_e152), ED.get_height(clamped_lod_e152)) - 1), clamped_lod_e152);
        phi_1795_ = _e136_.z;
    } else {
        phi_1795_ = 0u;
    }
    uint _e139_ = phi_1795_;
    if (_e139_ != 0u) {
        uint _e142_ = (_e139_ * 4u) + 1u;
        uint clamped_lod_e172 = metal::min(uint(0), PB.get_num_mip_levels() - 1);
        metal::uint4 _e149_ = PB.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e142_ & 127u), as_type<int>(_e142_ >> as_type<uint>(7)))), metal::uint2(PB.get_width(clamped_lod_e172), PB.get_height(clamped_lod_e172)) - 1), clamped_lod_e172);
        phi_1796_ = _e149_;
    } else {
        phi_1796_ = metal::uint4(0u, 0u, 0u, 0u);
    }
    metal::uint4 _e151_ = phi_1796_;
    float _e155_ = as_type<float>(_e151_.w);
    phi_1913_ = _e66_;
    phi_1912_ = _e64_;
    if (_e155_ != 0.0) {
        local_1 = as_type<float>(_e151_.z) == 0.0;
    } else {
        local_1 = false;
    }
    bool _e190 = local_1;
    if (_e190) {
        switch(as_type<int>(0u)) {
            default: {
                metal::float2 _e160_ = _e67_ - _e63_;
                float _e161_ = metal::length(_e160_);
                local_3_ = _e160_;
                local_4_ = _e160_;
                local_5_ = _e160_;
                if (_e161_ == 0.0) {
                    phi_1821_ = 0.5;
                    phi_1820_ = 0.0;
                    break;
                }
                metal::float2 _e168_ = metal::float2(-(_e160_.y), _e160_.x) / metal::float2(_e161_);
                float _e172_ = metal::dot(_e168_, _e64_ - _e63_);
                float _e173_ = _e172_ - metal::dot(_e168_, _e66_ - _e63_);
                float _e174_ = 3.0 * _e173_;
                float _e176_ = -(_e172_) - _e173_;
                phi_1812_ = 0.5;
                phi_1811_ = 0;
                uint2 loop_bound = uint2(4294967295u);
                bool loop_init = true;
                while(true) {
                    if (metal::all(loop_bound == uint2(0u))) { break; }
                    loop_bound -= uint2(loop_bound.y == 0u, 1u);
                    if (!loop_init) {
                        float _e551_ = local_6_;
                        phi_1812_ = _e551_;
                        phi_1811_ = as_type<int>(as_type<uint>(phi_1811_) + as_type<uint>(1));
                    }
                    loop_init = false;
                    float _e178_ = phi_1812_;
                    int _e180_ = phi_1811_;
                    local = _e178_;
                    local_1_ = _e178_;
                    local_2_ = _e178_;
                    local_7_ = _e178_;
                    if (_e180_ < 3) {
                        float _e182_ = _e174_ * _e178_;
                        float _e184_ = (_e182_ * _e178_) - _e172_;
                        float _e186_ = 2.0 * (_e182_ + _e176_);
                        if (_e186_ < 0.0) {
                            phi_1813_ = -(_e184_);
                        } else {
                            phi_1813_ = _e184_;
                        }
                        float _e190_ = phi_1813_;
                        float _e191_ = metal::abs(_e186_);
                        if (_e190_ > 0.0) {
                            if (_e190_ < _e191_) {
                                phi_1815_ = _e190_ / _e191_;
                            } else {
                                phi_1815_ = 1.0;
                            }
                            float _e196_ = phi_1815_;
                            phi_1816_ = _e196_;
                        } else {
                            phi_1816_ = 0.0;
                        }
                        float _e198_ = phi_1816_;
                        local_6_ = _e198_;
                        continue;
                    } else {
                        break;
                    }
                }
                float _e201_ = local;
                float _e206_ = local_1_;
                float _e211_ = local_2_;
                float _e561_ = local_7_;
                phi_1821_ = _e561_;
                phi_1820_ = metal::abs(_e211_ * ((_e206_ * ((_e201_ * _e174_) + (3.0 * _e176_))) + (3.0 * _e172_)));
                break;
            }
        }
        float _e215_ = phi_1821_;
        float _e217_ = phi_1820_;
        float _e218_ = _e155_ * 0.33333334;
        switch(as_type<int>(0u)) {
            default: {
                metal::float2 _e220_ = _e64_ - _e63_;
                metal::float2 _e221_ = _e66_ - _e64_;
                metal::float2 _e222_ = _e221_ - _e220_;
                metal::float2 _e225_ = local_3_;
                metal::float2 _e226_ = (_e221_ * -3.0) + _e225_;
                metal::float2 _e232_ = ((((_e226_ * _e215_) + (_e222_ * 2.0)) * _e215_) + _e220_) * 3.0;
                float _e233_ = metal::length(_e232_);
                if (_e233_ == 0.0) {
                    phi_1860_ = 0.0;
                    break;
                }
                metal::float2 _e236_ = _e232_ * (1.0 / _e233_);
                float _e237_ = metal::dot(_e226_, _e236_);
                float _e238_ = 2.0 * _e237_;
                float _e247_ = ((3.0 * ((_e238_ * _e215_) + (4.0 * metal::dot(_e222_, _e236_)))) * _e215_) + (6.0 * metal::dot(_e220_, _e236_));
                float _e249_ = metal::min(_e215_, 1.0 - _e215_);
                float _e255_ = metal::min(_e218_, ((((_e238_ * _e249_) * _e249_) + _e247_) * _e249_) * 0.9999);
                if (_e238_ == 0.0) {
                    phi_1834_ = _e255_ / _e247_;
                } else {
                    float _e258_ = 0.5 / _e237_;
                    float _e262_ = -0.33333334 * (_e247_ * _e258_);
                    float _e263_ = 0.5 * (-(_e255_) * _e258_);
                    float _e267_ = (_e263_ * _e263_) - ((_e262_ * _e262_) * _e262_);
                    if (_e267_ < 0.0) {
                        float _e269_ = metal::sqrt(_e262_);
                        phi_1835_ = (-2.0 * _e269_) * metal::cos((metal::acos(_e263_ / ((_e269_ * _e269_) * _e269_)) * 0.33333334) + -2.0943952);
                    } else {
                        float _e282_ = metal::pow(metal::abs(_e263_) + metal::sqrt(_e267_), 0.33333334);
                        phi_1832_ = _e282_;
                        if (_e263_ < 0.0) {
                            phi_1832_ = -(_e282_);
                        }
                        float _e286_ = phi_1832_;
                        if (_e286_ != 0.0) {
                            phi_1833_ = _e286_ + (_e262_ / _e286_);
                        } else {
                            phi_1833_ = 0.0;
                        }
                        float _e291_ = phi_1833_;
                        phi_1835_ = _e291_;
                    }
                    float _e293_ = phi_1835_;
                    phi_1834_ = _e293_;
                }
                float _e295_ = phi_1834_;
                float _e296_ = metal::abs(_e295_);
                float _e297_ = -(_e296_);
                metal::float4 _e300_ = metal::float4(_e215_) + metal::float4(_e297_, _e297_, _e296_, _e296_);
                metal::float4 _e308_ = (((_e226_.xyxy * _e300_) + (_e222_.xyxy * 2.0)) * _e300_) + _e220_.xyxy;
                if (metal::any(_e63_ != _e64_)) {
                    phi_1836_ = _e64_;
                } else {
                    phi_1836_ = metal::select(_e67_, _e66_, metal::bool2(metal::any(_e64_ != _e66_)));
                }
                metal::float2 _e316_ = phi_1836_;
                if (metal::any(_e67_ != _e66_)) {
                    phi_1855_ = _e66_;
                } else {
                    phi_1855_ = metal::select(_e63_, _e64_, metal::bool2(metal::any(_e66_ != _e64_)));
                }
                metal::float2 _e325_ = phi_1855_;
                if (_e300_.x < 0.001) {
                    phi_1857_ = _e316_ - _e63_;
                } else {
                    phi_1857_ = _e308_.xy;
                }
                metal::float2 _e331_ = phi_1857_;
                if (_e300_.z > 0.999) {
                    phi_1858_ = _e67_ - _e325_;
                } else {
                    phi_1858_ = _e308_.zw;
                }
                metal::float2 _e336_ = phi_1858_;
                float _e340_ = metal::dot(_e331_, _e331_) * metal::dot(_e336_, _e336_);
                if (_e340_ == 0.0) {
                    phi_1859_ = 1.0;
                } else {
                    phi_1859_ = metal::clamp(metal::dot(_e331_, _e336_) * metal::rsqrt(_e340_), -1.0, 1.0);
                }
                float _e346_ = phi_1859_;
                phi_1860_ = metal::acos(_e346_);
                break;
            }
        }
        float _e349_ = phi_1860_;
        metal::float2 _e353_ = local_4_;
        metal::float2 _e355_ = local_5_;
        metal::float4 _e365_ = XC.sample(aa, metal::float2(0.5 * metal::min(metal::min(1.0 - (_e349_ * 0.31830987), ((metal::dot(_e353_, _e355_) / (_e218_ * _e218_)) - 1.0) * 0.5), 0.99), 1.0), metal::level(0.0));
        float _e369_ = ((_e365_.x * -2.0) + 1.0) * _e155_;
        if (_e217_ < 0.0) {
            phi_1861_ = -(_e369_);
        } else {
            phi_1861_ = _e369_;
        }
        float _e373_ = phi_1861_;
        float _e374_ = metal::abs(_e217_);
        if (_e373_ > 0.0) {
            if (_e373_ < _e374_) {
                phi_1863_ = _e373_ / _e374_;
            } else {
                phi_1863_ = 1.0;
            }
            float _e379_ = phi_1863_;
            phi_1864_ = _e379_;
        } else {
            phi_1864_ = 0.0;
        }
        float _e381_ = phi_1864_;
        metal::float4 _e384_ = metal::mix(_e62_.xyxy, _e65_.zwzw, metal::float4(0.33333334, 0.33333334, 0.6666667, 0.6666667));
        metal::float2 _e386_ = metal::float2(_e381_);
        phi_1913_ = metal::mix(_e66_, _e384_.zw, _e386_);
        phi_1912_ = metal::mix(_e64_, _e384_.xy, _e386_);
    }
    metal::float2 _e391_ = phi_1913_;
    metal::float2 _e393_ = phi_1912_;
    phi_1935_ = _e118_;
    if ((_e125_ & 536870912u) != 0u) {
        uint _e396_ = _e139_ * 4u;
        uint clamped_lod_e486 = metal::min(uint(0), PB.get_num_mip_levels() - 1);
        metal::uint4 _e403_ = PB.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e396_ & 127u), as_type<int>(_e396_ >> as_type<uint>(7)))), metal::uint2(PB.get_width(clamped_lod_e486), PB.get_height(clamped_lod_e486)) - 1), clamped_lod_e486);
        metal::float4 _e404_ = as_type<metal::float4>(_e403_);
        metal::float2x2 _e411_ = metal::float2x2(metal::float2(_e404_.x, _e404_.y), metal::float2(_e404_.z, _e404_.w));
        metal::float2 _e415_ = _e411_ * (((_e393_ * -2.0) + _e391_) + _e63_);
        metal::float2 _e419_ = _e411_ * (((_e391_ * -2.0) + _e67_) + _e393_);
        phi_1935_ = metal::min(naga_f2u32(metal::max(metal::ceil(metal::sqrt(3.0 * metal::sqrt(metal::max(metal::dot(_e415_, _e415_), metal::dot(_e419_, _e419_))))), 1.0)), _e118_);
    }
    uint _e431_ = phi_1935_;
    if (metal::any(_e63_ != _e393_)) {
        phi_1938_ = _e393_;
    } else {
        phi_1938_ = metal::select(_e67_, _e391_, metal::bool2(metal::any(_e393_ != _e391_)));
    }
    metal::float2 _e442_ = phi_1938_;
    metal::float2 _e443_ = _e442_ - _e63_;
    if (metal::any(_e67_ != _e391_)) {
        phi_1967_ = _e391_;
    } else {
        phi_1967_ = metal::select(_e63_, _e393_, metal::bool2(metal::any(_e391_ != _e393_)));
    }
    metal::float2 _e451_ = phi_1967_;
    metal::float2 _e452_ = _e67_ - _e451_;
    float _e456_ = metal::dot(_e452_, _e452_);
    float _e457_ = metal::dot(_e443_, _e443_) * _e456_;
    if (_e457_ == 0.0) {
        phi_1969_ = 1.0;
    } else {
        phi_1969_ = metal::clamp(metal::dot(_e443_, _e452_) * metal::rsqrt(_e457_), -1.0, 1.0);
    }
    float _e463_ = phi_1969_;
    float _e466_ = metal::acos(_e463_) / static_cast<float>(_e121_);
    float _e470_ = metal::determinant(metal::float2x2(_e391_ - _e63_, _e67_ - _e393_));
    phi_1976_ = _e470_;
    if (_e470_ == 0.0) {
        phi_1976_ = metal::determinant(metal::float2x2(_e443_, _e452_));
    }
    float _e474_ = phi_1976_;
    phi_2015_ = _e466_;
    if (_e474_ < 0.0) {
        phi_2015_ = -(_e466_);
    }
    float _e478_ = phi_2015_;
    x6_ = metal::float4(_e62_.x, _e62_.y, _e393_.x, _e393_.y);
    y6_ = metal::float4(_e391_.x, _e391_.y, _e65_.z, _e65_.w);
    float _e489_ = static_cast<float>(((_e431_ + _e121_) + _e123_) - 1u);
    L4_ = metal::float4(_e489_ - metal::abs(_e95_ - _e117_.x), _e489_, static_cast<float>((_e123_ << as_type<uint>(10)) | _e431_), _e478_);
    if (_e123_ > 1u) {
        metal::float4 _e500_ = UC_1_;
        metal::float2 _e503_ = metal::float2(_e500_.x, _e500_.y);
        float _e507_ = _e456_ * metal::dot(_e503_, _e503_);
        if (_e507_ == 0.0) {
            phi_2016_ = 1.0;
        } else {
            phi_2016_ = metal::clamp(metal::dot(_e452_, _e503_) * metal::rsqrt(_e507_), -1.0, 1.0);
        }
        float _e513_ = phi_2016_;
        float _e515_ = static_cast<float>(_e123_);
        phi_2024_ = _e515_;
        if ((_e125_ & 503316480u) == 167772160u) {
            phi_2024_ = _e515_ - 2.0;
        }
        float _e520_ = phi_2024_;
        float _e521_ = metal::acos(_e513_) / _e520_;
        phi_2025_ = _e521_;
        if (metal::determinant(metal::float2x2(_e452_, _e503_)) < 0.0) {
            phi_2025_ = -(_e521_);
        }
        float _e526_ = phi_2025_;
        C5_.x = _e500_.x;
        C5_.y = _e500_.y;
        C5_.z = _e526_;
    }
    phi_2029_ = _e125_;
    if (_e95_ < _e92_) {
        phi_2029_ = _e125_ | 8388608u;
    }
    uint _e533_ = phi_2029_;
    F7_ = _e533_;
    unnamed.gl_Position = metal::float4((_e117_.x * 0.0009765625) - 1.0, (_e117_.y * _e107_) - metal::sign(_e107_), 0.0, 1.0);
    return;
}

struct main_Output {
    metal::float4 member [[user(loc0), center_perspective]];
    metal::float4 member_1_ [[user(loc1), center_perspective]];
    metal::float4 member_2_ [[user(loc2), center_perspective]];
    metal::float3 member_3_ [[user(loc3), center_perspective]];
    uint member_4_ [[user(loc4), flat]];
    metal::float4 gl_Position [[position]];
};
struct vb_30_type { metal::uchar data[64]; };
vertex main_Output main_(
  uint gl_VertexIndex [[vertex_id]]
, constant CC& n [[buffer(0)]]
, metal::texture2d<uint, metal::access::sample> ED [[texture(1)]]
, metal::texture2d<uint, metal::access::sample> PB [[texture(0)]]
, metal::texture2d<float, metal::access::sample> XC [[texture(2)]]
, metal::sampler aa [[sampler(0)]]
, uint i_id [[instance_id]]
, const device vb_30_type* vb_30_in [[buffer(30)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(1)]]
) {
    metal::float4 GD = {};
    metal::float4 HD = {};
    metal::float4 UC = {};
    metal::uint4 TB = {};
    if (i_id < (_buffer_sizes.buffer_size30 / 64)) {
        const vb_30_type vb_30_elem = vb_30_in[i_id];
        GD = unpackFloat32x4_(vb_30_elem.data[0], vb_30_elem.data[1], vb_30_elem.data[2], vb_30_elem.data[3], vb_30_elem.data[4], vb_30_elem.data[5], vb_30_elem.data[6], vb_30_elem.data[7], vb_30_elem.data[8], vb_30_elem.data[9], vb_30_elem.data[10], vb_30_elem.data[11], vb_30_elem.data[12], vb_30_elem.data[13], vb_30_elem.data[14], vb_30_elem.data[15]);
        HD = unpackFloat32x4_(vb_30_elem.data[16], vb_30_elem.data[17], vb_30_elem.data[18], vb_30_elem.data[19], vb_30_elem.data[20], vb_30_elem.data[21], vb_30_elem.data[22], vb_30_elem.data[23], vb_30_elem.data[24], vb_30_elem.data[25], vb_30_elem.data[26], vb_30_elem.data[27], vb_30_elem.data[28], vb_30_elem.data[29], vb_30_elem.data[30], vb_30_elem.data[31]);
        UC = unpackFloat32x4_(vb_30_elem.data[32], vb_30_elem.data[33], vb_30_elem.data[34], vb_30_elem.data[35], vb_30_elem.data[36], vb_30_elem.data[37], vb_30_elem.data[38], vb_30_elem.data[39], vb_30_elem.data[40], vb_30_elem.data[41], vb_30_elem.data[42], vb_30_elem.data[43], vb_30_elem.data[44], vb_30_elem.data[45], vb_30_elem.data[46], vb_30_elem.data[47]);
        TB = unpackUint32x4_(vb_30_elem.data[48], vb_30_elem.data[49], vb_30_elem.data[50], vb_30_elem.data[51], vb_30_elem.data[52], vb_30_elem.data[53], vb_30_elem.data[54], vb_30_elem.data[55], vb_30_elem.data[56], vb_30_elem.data[57], vb_30_elem.data[58], vb_30_elem.data[59], vb_30_elem.data[60], vb_30_elem.data[61], vb_30_elem.data[62], vb_30_elem.data[63]);
    }
    int gl_VertexIndex_1_ = {};
    metal::float4 GD_1_ = {};
    metal::float4 HD_1_ = {};
    metal::float4 UC_1_ = {};
    metal::uint4 TB_1_ = {};
    metal::float4 x6_ = {};
    metal::float4 y6_ = {};
    metal::float4 L4_ = {};
    metal::float3 C5_ = {};
    uint F7_ = {};
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_6 {}, type_6 {}};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    GD_1_ = GD;
    HD_1_ = HD;
    UC_1_ = UC;
    TB_1_ = TB;
    main_1_(gl_VertexIndex_1_, GD_1_, HD_1_, UC_1_, TB_1_, n, ED, PB, XC, aa, x6_, y6_, L4_, C5_, F7_, unnamed);
    metal::float4 _e18_ = x6_;
    metal::float4 _e19_ = y6_;
    metal::float4 _e20_ = L4_;
    metal::float3 _e21_ = C5_;
    uint _e22_ = F7_;
    metal::float4 _e23_ = unnamed.gl_Position;
    const auto _tmp = VertexOutput {_e18_, _e19_, _e20_, _e21_, _e22_, _e23_};
    return main_Output { _tmp.member, _tmp.member_1_, _tmp.member_2_, _tmp.member_3_, _tmp.member_4_, _tmp.gl_Position };
}
