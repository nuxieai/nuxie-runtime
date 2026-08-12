// language: metal4.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size6;
    uint size7;
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
typedef metal::uint4 type_6[1];
struct cg {
    type_6 c2_;
};
struct bg {
    type_6 c2_;
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
    device cg const& ED,
    device bg const& PB,
    metal::texture2d<float, metal::access::sample> XC,
    metal::sampler aa,
    thread metal::float4& x6_,
    thread metal::float4& y6_,
    thread metal::float4& L4_,
    thread metal::float3& C5_,
    thread uint& F7_,
    thread gl_PerVertex& unnamed,
    constant _mslBufferSizes& _buffer_sizes
) {
    float phi_1770_ = {};
    uint phi_1771_ = {};
    int phi_1772_ = {};
    float phi_1773_ = {};
    metal::float2 phi_1970_ = {};
    uint phi_1774_ = {};
    metal::uint4 phi_1775_ = {};
    float phi_1791_ = {};
    int phi_1790_ = {};
    float phi_1792_ = {};
    float phi_1794_ = {};
    float phi_1795_ = {};
    float local = {};
    float local_1_ = {};
    float local_2_ = {};
    float phi_1800_ = {};
    float phi_1799_ = {};
    metal::float2 local_3_ = {};
    float phi_1811_ = {};
    float phi_1812_ = {};
    float phi_1814_ = {};
    float phi_1813_ = {};
    metal::float2 phi_1815_ = {};
    metal::float2 phi_1834_ = {};
    metal::float2 phi_1836_ = {};
    metal::float2 phi_1837_ = {};
    float phi_1838_ = {};
    float phi_1839_ = {};
    metal::float2 local_4_ = {};
    metal::float2 local_5_ = {};
    float phi_1840_ = {};
    float phi_1842_ = {};
    float phi_1843_ = {};
    metal::float2 phi_1892_ = {};
    metal::float2 phi_1891_ = {};
    uint phi_1914_ = {};
    metal::float2 phi_1917_ = {};
    metal::float2 phi_1946_ = {};
    float phi_1948_ = {};
    float phi_1955_ = {};
    float phi_1994_ = {};
    float phi_1995_ = {};
    float phi_2003_ = {};
    float phi_2004_ = {};
    uint phi_2008_ = {};
    float local_6_ = {};
    float local_7_ = {};
    bool local_1 = {};
    int _e59_ = gl_VertexIndex_1_;
    metal::float4 _e60_ = GD_1_;
    metal::float2 _e61_ = _e60_.xy;
    metal::float2 _e62_ = _e60_.zw;
    metal::float4 _e63_ = HD_1_;
    metal::float2 _e64_ = _e63_.xy;
    metal::float2 _e65_ = _e63_.zw;
    bool _e66_ = _e59_ < 4;
    if (_e66_) {
        float _e68_ = UC_1_.z;
        phi_1770_ = _e68_;
    } else {
        float _e70_ = UC_1_.w;
        phi_1770_ = _e70_;
    }
    float _e72_ = phi_1770_;
    if (_e66_) {
        uint _e74_ = TB_1_.x;
        phi_1771_ = _e74_;
    } else {
        uint _e76_ = TB_1_.y;
        phi_1771_ = _e76_;
    }
    uint _e78_ = phi_1771_;
    int _e79_ = as_type<int>(_e78_);
    int _e81_ = _e79_ << as_type<uint>(16);
    uint _e83_ = TB_1_.z;
    phi_1772_ = _e81_;
    if (_e83_ == 4294967295u) {
        phi_1772_ = as_type<int>(as_type<uint>(_e81_) - as_type<uint>(1));
    }
    int _e87_ = phi_1772_;
    float _e90_ = static_cast<float>(_e87_ >> as_type<uint>(16));
    float _e93_ = static_cast<float>(_e79_ >> as_type<uint>(16));
    if ((_e59_ & 2) == 0) {
        phi_1773_ = _e72_ + 1.0;
    } else {
        phi_1773_ = _e72_;
    }
    float _e101_ = phi_1773_;
    metal::float2 _e102_ = metal::float2(((_e59_ & 1) == 0) ? _e90_ : _e93_, _e101_);
    float _e105_ = n.od;
    phi_1970_ = _e102_;
    if (((_e93_ - _e90_) * _e105_) < 0.0) {
        phi_1970_ = metal::float2(_e102_.x, ((2.0 * _e72_) + 1.0) - _e101_);
    }
    metal::float2 _e115_ = phi_1970_;
    uint _e116_ = _e83_ & 1023u;
    uint _e119_ = (_e83_ >> as_type<uint>(10)) & 1023u;
    uint _e121_ = _e83_ >> as_type<uint>(20);
    uint _e123_ = TB_1_.w;
    uint _e124_ = _e123_ & 65535u;
    if (_e124_ > 0u) {
        uint _e131_ = ED.c2_[metal::min(unsigned(metal::max(_e124_, 1u) - 1u), (_buffer_sizes.size6 - 0 - 16) / 16)].z;
        phi_1774_ = _e131_;
    } else {
        phi_1774_ = 0u;
    }
    uint _e133_ = phi_1774_;
    if (_e133_ != 0u) {
        metal::uint4 _e139_ = PB.c2_[metal::min(unsigned((_e133_ * 4u) + 1u), (_buffer_sizes.size7 - 0 - 16) / 16)];
        phi_1775_ = _e139_;
    } else {
        phi_1775_ = metal::uint4(0u, 0u, 0u, 0u);
    }
    metal::uint4 _e141_ = phi_1775_;
    float _e145_ = as_type<float>(_e141_.w);
    phi_1892_ = _e64_;
    phi_1891_ = _e62_;
    if (_e145_ != 0.0) {
        local_1 = as_type<float>(_e141_.z) == 0.0;
    } else {
        local_1 = false;
    }
    bool _e176 = local_1;
    if (_e176) {
        switch(as_type<int>(0u)) {
            default: {
                metal::float2 _e150_ = _e65_ - _e61_;
                float _e151_ = metal::length(_e150_);
                local_3_ = _e150_;
                local_4_ = _e150_;
                local_5_ = _e150_;
                if (_e151_ == 0.0) {
                    phi_1800_ = 0.5;
                    phi_1799_ = 0.0;
                    break;
                }
                metal::float2 _e158_ = metal::float2(-(_e150_.y), _e150_.x) / metal::float2(_e151_);
                float _e162_ = metal::dot(_e158_, _e62_ - _e61_);
                float _e163_ = _e162_ - metal::dot(_e158_, _e64_ - _e61_);
                float _e164_ = 3.0 * _e163_;
                float _e166_ = -(_e162_) - _e163_;
                phi_1791_ = 0.5;
                phi_1790_ = 0;
                uint2 loop_bound = uint2(4294967295u);
                bool loop_init = true;
                while(true) {
                    if (metal::all(loop_bound == uint2(0u))) { break; }
                    loop_bound -= uint2(loop_bound.y == 0u, 1u);
                    if (!loop_init) {
                        float _e537_ = local_6_;
                        phi_1791_ = _e537_;
                        phi_1790_ = as_type<int>(as_type<uint>(phi_1790_) + as_type<uint>(1));
                    }
                    loop_init = false;
                    float _e168_ = phi_1791_;
                    int _e170_ = phi_1790_;
                    local = _e168_;
                    local_1_ = _e168_;
                    local_2_ = _e168_;
                    local_7_ = _e168_;
                    if (_e170_ < 3) {
                        float _e172_ = _e164_ * _e168_;
                        float _e174_ = (_e172_ * _e168_) - _e162_;
                        float _e176_ = 2.0 * (_e172_ + _e166_);
                        if (_e176_ < 0.0) {
                            phi_1792_ = -(_e174_);
                        } else {
                            phi_1792_ = _e174_;
                        }
                        float _e180_ = phi_1792_;
                        float _e181_ = metal::abs(_e176_);
                        if (_e180_ > 0.0) {
                            if (_e180_ < _e181_) {
                                phi_1794_ = _e180_ / _e181_;
                            } else {
                                phi_1794_ = 1.0;
                            }
                            float _e186_ = phi_1794_;
                            phi_1795_ = _e186_;
                        } else {
                            phi_1795_ = 0.0;
                        }
                        float _e188_ = phi_1795_;
                        local_6_ = _e188_;
                        continue;
                    } else {
                        break;
                    }
                }
                float _e191_ = local;
                float _e196_ = local_1_;
                float _e201_ = local_2_;
                float _e547_ = local_7_;
                phi_1800_ = _e547_;
                phi_1799_ = metal::abs(_e201_ * ((_e196_ * ((_e191_ * _e164_) + (3.0 * _e166_))) + (3.0 * _e162_)));
                break;
            }
        }
        float _e205_ = phi_1800_;
        float _e207_ = phi_1799_;
        float _e208_ = _e145_ * 0.33333334;
        switch(as_type<int>(0u)) {
            default: {
                metal::float2 _e210_ = _e62_ - _e61_;
                metal::float2 _e211_ = _e64_ - _e62_;
                metal::float2 _e212_ = _e211_ - _e210_;
                metal::float2 _e215_ = local_3_;
                metal::float2 _e216_ = (_e211_ * -3.0) + _e215_;
                metal::float2 _e222_ = ((((_e216_ * _e205_) + (_e212_ * 2.0)) * _e205_) + _e210_) * 3.0;
                float _e223_ = metal::length(_e222_);
                if (_e223_ == 0.0) {
                    phi_1839_ = 0.0;
                    break;
                }
                metal::float2 _e226_ = _e222_ * (1.0 / _e223_);
                float _e227_ = metal::dot(_e216_, _e226_);
                float _e228_ = 2.0 * _e227_;
                float _e237_ = ((3.0 * ((_e228_ * _e205_) + (4.0 * metal::dot(_e212_, _e226_)))) * _e205_) + (6.0 * metal::dot(_e210_, _e226_));
                float _e239_ = metal::min(_e205_, 1.0 - _e205_);
                float _e245_ = metal::min(_e208_, ((((_e228_ * _e239_) * _e239_) + _e237_) * _e239_) * 0.9999);
                if (_e228_ == 0.0) {
                    phi_1813_ = _e245_ / _e237_;
                } else {
                    float _e248_ = 0.5 / _e227_;
                    float _e252_ = -0.33333334 * (_e237_ * _e248_);
                    float _e253_ = 0.5 * (-(_e245_) * _e248_);
                    float _e257_ = (_e253_ * _e253_) - ((_e252_ * _e252_) * _e252_);
                    if (_e257_ < 0.0) {
                        float _e259_ = metal::sqrt(_e252_);
                        phi_1814_ = (-2.0 * _e259_) * metal::cos((metal::acos(_e253_ / ((_e259_ * _e259_) * _e259_)) * 0.33333334) + -2.0943952);
                    } else {
                        float _e272_ = metal::pow(metal::abs(_e253_) + metal::sqrt(_e257_), 0.33333334);
                        phi_1811_ = _e272_;
                        if (_e253_ < 0.0) {
                            phi_1811_ = -(_e272_);
                        }
                        float _e276_ = phi_1811_;
                        if (_e276_ != 0.0) {
                            phi_1812_ = _e276_ + (_e252_ / _e276_);
                        } else {
                            phi_1812_ = 0.0;
                        }
                        float _e281_ = phi_1812_;
                        phi_1814_ = _e281_;
                    }
                    float _e283_ = phi_1814_;
                    phi_1813_ = _e283_;
                }
                float _e285_ = phi_1813_;
                float _e286_ = metal::abs(_e285_);
                float _e287_ = -(_e286_);
                metal::float4 _e290_ = metal::float4(_e205_) + metal::float4(_e287_, _e287_, _e286_, _e286_);
                metal::float4 _e298_ = (((_e216_.xyxy * _e290_) + (_e212_.xyxy * 2.0)) * _e290_) + _e210_.xyxy;
                if (metal::any(_e61_ != _e62_)) {
                    phi_1815_ = _e62_;
                } else {
                    phi_1815_ = metal::select(_e65_, _e64_, metal::bool2(metal::any(_e62_ != _e64_)));
                }
                metal::float2 _e306_ = phi_1815_;
                if (metal::any(_e65_ != _e64_)) {
                    phi_1834_ = _e64_;
                } else {
                    phi_1834_ = metal::select(_e61_, _e62_, metal::bool2(metal::any(_e64_ != _e62_)));
                }
                metal::float2 _e315_ = phi_1834_;
                if (_e290_.x < 0.001) {
                    phi_1836_ = _e306_ - _e61_;
                } else {
                    phi_1836_ = _e298_.xy;
                }
                metal::float2 _e321_ = phi_1836_;
                if (_e290_.z > 0.999) {
                    phi_1837_ = _e65_ - _e315_;
                } else {
                    phi_1837_ = _e298_.zw;
                }
                metal::float2 _e326_ = phi_1837_;
                float _e330_ = metal::dot(_e321_, _e321_) * metal::dot(_e326_, _e326_);
                if (_e330_ == 0.0) {
                    phi_1838_ = 1.0;
                } else {
                    phi_1838_ = metal::clamp(metal::dot(_e321_, _e326_) * metal::rsqrt(_e330_), -1.0, 1.0);
                }
                float _e336_ = phi_1838_;
                phi_1839_ = metal::acos(_e336_);
                break;
            }
        }
        float _e339_ = phi_1839_;
        metal::float2 _e343_ = local_4_;
        metal::float2 _e345_ = local_5_;
        metal::float4 _e355_ = XC.sample(aa, metal::float2(0.5 * metal::min(metal::min(1.0 - (_e339_ * 0.31830987), ((metal::dot(_e343_, _e345_) / (_e208_ * _e208_)) - 1.0) * 0.5), 0.99), 1.0), metal::level(0.0));
        float _e359_ = ((_e355_.x * -2.0) + 1.0) * _e145_;
        if (_e207_ < 0.0) {
            phi_1840_ = -(_e359_);
        } else {
            phi_1840_ = _e359_;
        }
        float _e363_ = phi_1840_;
        float _e364_ = metal::abs(_e207_);
        if (_e363_ > 0.0) {
            if (_e363_ < _e364_) {
                phi_1842_ = _e363_ / _e364_;
            } else {
                phi_1842_ = 1.0;
            }
            float _e369_ = phi_1842_;
            phi_1843_ = _e369_;
        } else {
            phi_1843_ = 0.0;
        }
        float _e371_ = phi_1843_;
        metal::float4 _e374_ = metal::mix(_e60_.xyxy, _e63_.zwzw, metal::float4(0.33333334, 0.33333334, 0.6666667, 0.6666667));
        metal::float2 _e376_ = metal::float2(_e371_);
        phi_1892_ = metal::mix(_e64_, _e374_.zw, _e376_);
        phi_1891_ = metal::mix(_e62_, _e374_.xy, _e376_);
    }
    metal::float2 _e381_ = phi_1892_;
    metal::float2 _e383_ = phi_1891_;
    phi_1914_ = _e116_;
    if ((_e123_ & 536870912u) != 0u) {
        metal::uint4 _e389_ = PB.c2_[metal::min(unsigned(_e133_ * 4u), (_buffer_sizes.size7 - 0 - 16) / 16)];
        metal::float4 _e390_ = as_type<metal::float4>(_e389_);
        metal::float2x2 _e397_ = metal::float2x2(metal::float2(_e390_.x, _e390_.y), metal::float2(_e390_.z, _e390_.w));
        metal::float2 _e401_ = _e397_ * (((_e383_ * -2.0) + _e381_) + _e61_);
        metal::float2 _e405_ = _e397_ * (((_e381_ * -2.0) + _e65_) + _e383_);
        phi_1914_ = metal::min(naga_f2u32(metal::max(metal::ceil(metal::sqrt(3.0 * metal::sqrt(metal::max(metal::dot(_e401_, _e401_), metal::dot(_e405_, _e405_))))), 1.0)), _e116_);
    }
    uint _e417_ = phi_1914_;
    if (metal::any(_e61_ != _e383_)) {
        phi_1917_ = _e383_;
    } else {
        phi_1917_ = metal::select(_e65_, _e381_, metal::bool2(metal::any(_e383_ != _e381_)));
    }
    metal::float2 _e428_ = phi_1917_;
    metal::float2 _e429_ = _e428_ - _e61_;
    if (metal::any(_e65_ != _e381_)) {
        phi_1946_ = _e381_;
    } else {
        phi_1946_ = metal::select(_e61_, _e383_, metal::bool2(metal::any(_e381_ != _e383_)));
    }
    metal::float2 _e437_ = phi_1946_;
    metal::float2 _e438_ = _e65_ - _e437_;
    float _e442_ = metal::dot(_e438_, _e438_);
    float _e443_ = metal::dot(_e429_, _e429_) * _e442_;
    if (_e443_ == 0.0) {
        phi_1948_ = 1.0;
    } else {
        phi_1948_ = metal::clamp(metal::dot(_e429_, _e438_) * metal::rsqrt(_e443_), -1.0, 1.0);
    }
    float _e449_ = phi_1948_;
    float _e452_ = metal::acos(_e449_) / static_cast<float>(_e119_);
    float _e456_ = metal::determinant(metal::float2x2(_e381_ - _e61_, _e65_ - _e383_));
    phi_1955_ = _e456_;
    if (_e456_ == 0.0) {
        phi_1955_ = metal::determinant(metal::float2x2(_e429_, _e438_));
    }
    float _e460_ = phi_1955_;
    phi_1994_ = _e452_;
    if (_e460_ < 0.0) {
        phi_1994_ = -(_e452_);
    }
    float _e464_ = phi_1994_;
    x6_ = metal::float4(_e60_.x, _e60_.y, _e383_.x, _e383_.y);
    y6_ = metal::float4(_e381_.x, _e381_.y, _e63_.z, _e63_.w);
    float _e475_ = static_cast<float>(((_e417_ + _e119_) + _e121_) - 1u);
    L4_ = metal::float4(_e475_ - metal::abs(_e93_ - _e115_.x), _e475_, static_cast<float>((_e121_ << as_type<uint>(10)) | _e417_), _e464_);
    if (_e121_ > 1u) {
        metal::float4 _e486_ = UC_1_;
        metal::float2 _e489_ = metal::float2(_e486_.x, _e486_.y);
        float _e493_ = _e442_ * metal::dot(_e489_, _e489_);
        if (_e493_ == 0.0) {
            phi_1995_ = 1.0;
        } else {
            phi_1995_ = metal::clamp(metal::dot(_e438_, _e489_) * metal::rsqrt(_e493_), -1.0, 1.0);
        }
        float _e499_ = phi_1995_;
        float _e501_ = static_cast<float>(_e121_);
        phi_2003_ = _e501_;
        if ((_e123_ & 503316480u) == 167772160u) {
            phi_2003_ = _e501_ - 2.0;
        }
        float _e506_ = phi_2003_;
        float _e507_ = metal::acos(_e499_) / _e506_;
        phi_2004_ = _e507_;
        if (metal::determinant(metal::float2x2(_e438_, _e489_)) < 0.0) {
            phi_2004_ = -(_e507_);
        }
        float _e512_ = phi_2004_;
        C5_.x = _e486_.x;
        C5_.y = _e486_.y;
        C5_.z = _e512_;
    }
    phi_2008_ = _e123_;
    if (_e93_ < _e90_) {
        phi_2008_ = _e123_ | 8388608u;
    }
    uint _e519_ = phi_2008_;
    F7_ = _e519_;
    unnamed.gl_Position = metal::float4((_e115_.x * 0.0009765625) - 1.0, (_e115_.y * _e105_) - metal::sign(_e105_), 0.0, 1.0);
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
, device cg const& ED [[buffer(2)]]
, device bg const& PB [[buffer(1)]]
, metal::texture2d<float, metal::access::sample> XC [[texture(0)]]
, metal::sampler aa [[sampler(0)]]
, uint i_id [[instance_id]]
, const device vb_30_type* vb_30_in [[buffer(30)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(3)]]
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
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_8 {}, type_8 {}};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    GD_1_ = GD;
    HD_1_ = HD;
    UC_1_ = UC;
    TB_1_ = TB;
    main_1_(gl_VertexIndex_1_, GD_1_, HD_1_, UC_1_, TB_1_, n, ED, PB, XC, aa, x6_, y6_, L4_, C5_, F7_, unnamed, _buffer_sizes);
    metal::float4 _e18_ = x6_;
    metal::float4 _e19_ = y6_;
    metal::float4 _e20_ = L4_;
    metal::float3 _e21_ = C5_;
    uint _e22_ = F7_;
    metal::float4 _e23_ = unnamed.gl_Position;
    const auto _tmp = VertexOutput {_e18_, _e19_, _e20_, _e21_, _e22_, _e23_};
    return main_Output { _tmp.member, _tmp.member_1_, _tmp.member_2_, _tmp.member_3_, _tmp.member_4_, _tmp.gl_Position };
}
