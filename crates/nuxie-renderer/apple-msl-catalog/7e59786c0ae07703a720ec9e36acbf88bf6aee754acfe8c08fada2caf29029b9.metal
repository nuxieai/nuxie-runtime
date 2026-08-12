// language: metal2.4
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

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

uint naga_f2u32(float value) {
    return static_cast<uint>(metal::clamp(value, 0.0, 4294967000.0));
}

void main_1_(
    thread metal::float4& x6_1_,
    thread metal::float4& y6_1_,
    thread metal::float4& L4_1_,
    thread uint& F7_1_,
    thread metal::float3& C5_1_,
    thread metal::uint4& Jg
) {
    metal::float2 phi_818_ = {};
    metal::float2 phi_821_ = {};
    uint phi_826_ = {};
    uint phi_833_ = {};
    bool phi_250_ = {};
    float phi_844_ = {};
    float phi_839_ = {};
    float phi_841_ = {};
    float phi_837_ = {};
    uint phi_832_ = {};
    float phi_891_ = {};
    metal::float2x2 phi_884_ = {};
    uint phi_845_ = {};
    float phi_840_ = {};
    float phi_836_ = {};
    bool phi_278_ = {};
    metal::float2 phi_935_ = {};
    float phi_936_ = {};
    float phi_893_ = {};
    int phi_892_ = {};
    float phi_1021_ = {};
    float local = {};
    float local_1_ = {};
    float phi_896_ = {};
    float phi_899_ = {};
    metal::float2 phi_900_ = {};
    float phi_901_ = {};
    float phi_987_ = {};
    float phi_933_ = {};
    float phi_923_ = {};
    float phi_934_ = {};
    float phi_986_ = {};
    float phi_984_ = {};
    metal::float2 phi_940_ = {};
    float phi_983_ = {};
    metal::float2 phi_937_ = {};
    metal::uint4 phi_1020_ = {};
    float local_2_ = {};
    bool local_1 = {};
    bool local_2 = {};
    metal::float4 _e40_ = x6_1_;
    metal::float2 _e41_ = _e40_.xy;
    metal::float2 _e42_ = _e40_.zw;
    metal::float4 _e43_ = y6_1_;
    metal::float2 _e44_ = _e43_.xy;
    metal::float2 _e45_ = _e43_.zw;
    if (metal::any(_e41_ != _e42_)) {
        phi_818_ = _e42_;
    } else {
        phi_818_ = metal::select(_e45_, _e44_, metal::bool2(metal::any(_e42_ != _e44_)));
    }
    metal::float2 _e53_ = phi_818_;
    if (metal::any(_e45_ != _e44_)) {
        phi_821_ = _e44_;
    } else {
        phi_821_ = metal::select(_e41_, _e42_, metal::bool2(metal::any(_e44_ != _e42_)));
    }
    metal::float2 _e62_ = phi_821_;
    metal::float2 _e63_ = _e45_ - _e62_;
    float _e66_ = L4_1_.x;
    float _e68_ = metal::max(metal::floor(_e66_), 0.0);
    float _e70_ = L4_1_.y;
    float _e72_ = L4_1_.z;
    uint _e73_ = naga_f2u32(_e72_);
    float _e78_ = static_cast<float>(_e73_ >> as_type<uint>(10));
    float _e80_ = L4_1_.w;
    uint _e81_ = F7_1_;
    float _e82_ = _e70_ - _e78_;
    bool _e83_ = _e68_ <= _e82_;
    if (_e83_) {
        phi_891_ = _e80_;
        phi_884_ = metal::float2x2(_e53_ - _e41_, _e63_);
        phi_845_ = _e81_ & 3825205247u;
        phi_840_ = _e82_;
        phi_836_ = _e68_;
    } else {
        metal::float3 _e85_ = C5_1_;
        float _e90_ = _e68_ - _e82_;
        float _e92_ = C5_1_.z;
        uint _e93_ = _e81_ & 469762048u;
        if (_e93_ > 134217728u) {
            phi_826_ = _e81_;
            if (_e90_ < 2.5) {
                phi_826_ = _e81_ | 4194304u;
            }
            uint _e98_ = phi_826_;
            phi_833_ = _e98_;
            if (_e90_ > 1.5) {
                local_1 = _e90_ < 3.5;
            } else {
                local_1 = false;
            }
            bool _e111 = local_1;
            if (_e111) {
                phi_833_ = _e98_ | 2097152u;
            }
            uint _e104_ = phi_833_;
            phi_841_ = _e78_;
            phi_837_ = _e90_;
            phi_832_ = _e104_;
        } else {
            bool _e106_ = (_e81_ & 33554432u) != 0u;
            phi_250_ = _e106_;
            if (!(_e106_)) {
                phi_250_ = _e93_ == 67108864u;
            }
            bool _e110_ = phi_250_;
            phi_844_ = _e78_;
            phi_839_ = _e90_;
            if (_e110_) {
                phi_844_ = _e78_ - 2.0;
                phi_839_ = _e90_ - 1.0;
            }
            float _e114_ = phi_844_;
            float _e116_ = phi_839_;
            phi_841_ = _e114_;
            phi_837_ = _e116_;
            phi_832_ = _e81_;
        }
        float _e118_ = phi_841_;
        float _e120_ = phi_837_;
        uint _e122_ = phi_832_;
        phi_891_ = _e92_;
        phi_884_ = metal::float2x2(_e63_, metal::float2(_e85_.x, _e85_.y));
        phi_845_ = _e122_ | ((_e92_ < 0.0) ? 1048576u : 524288u);
        phi_840_ = _e118_;
        phi_836_ = _e120_;
    }
    float _e127_ = phi_891_;
    metal::float2x2 _e129_ = phi_884_;
    uint _e131_ = phi_845_;
    float _e133_ = phi_840_;
    float _e135_ = phi_836_;
    metal::bool2 _e136_ = metal::bool2(_e83_);
    metal::float2 _e137_ = metal::select(_e45_, _e44_, _e136_);
    metal::float2 _e138_ = metal::select(_e45_, _e41_, _e136_);
    metal::float2 _e139_ = metal::select(_e45_, _e42_, _e136_);
    float _e140_ = _e83_ ? static_cast<float>(_e73_ & 1023u) : 1.0;
    if (!((_e135_ == 0.0))) {
        local_2 = _e135_ == _e133_;
    } else {
        local_2 = true;
    }
    bool _e143_ = local_2;
    phi_278_ = _e143_;
    if (!(_e143_)) {
        phi_278_ = (_e131_ & 469762048u) > 134217728u;
    }
    bool _e148_ = phi_278_;
    if (_e148_) {
        bool _e150_ = _e135_ < (_e133_ * 0.5);
        if (_e150_) {
            phi_935_ = _e129_[0];
        } else {
            phi_935_ = _e129_[1];
        }
        metal::float2 _e156_ = phi_935_;
        metal::float2 _e157_ = metal::normalize(_e156_);
        float _e160_ = metal::acos(metal::clamp(_e157_.x, -1.0, 1.0));
        if (_e157_.y >= 0.0) {
            phi_936_ = _e160_;
        } else {
            phi_936_ = -(_e160_);
        }
        float _e165_ = phi_936_;
        phi_983_ = _e165_;
        phi_937_ = metal::select(_e45_, _e138_, metal::bool2(_e150_));
    } else {
        if ((_e131_ & 2147483648u) != 0u) {
            phi_984_ = 0.0;
            phi_940_ = _e139_;
        } else {
            if (_e140_ == _e133_) {
                phi_987_ = 0.0;
                phi_933_ = 0.0;
                phi_923_ = _e135_ / _e140_;
            } else {
                metal::float2 _e170_ = _e139_ - _e138_;
                metal::float2 _e172_ = _e137_ - _e139_;
                metal::float2 _e173_ = _e172_ - _e170_;
                metal::float2 _e175_ = (_e172_ * -3.0) + (_e45_ - _e138_);
                metal::float2 _e183_ = metal::normalize(_e129_[0]);
                float _e184_ = metal::abs(_e127_);
                phi_893_ = 0.0;
                phi_892_ = 9;
                uint2 loop_bound = uint2(4294967295u);
                bool loop_init = true;
                while(true) {
                    if (metal::all(loop_bound == uint2(0u))) { break; }
                    loop_bound -= uint2(loop_bound.y == 0u, 1u);
                    if (!loop_init) {
                        float _e373_ = local_2_;
                        phi_893_ = _e373_;
                        phi_892_ = as_type<int>(as_type<uint>(phi_892_) - as_type<uint>(1));
                    }
                    loop_init = false;
                    float _e189_ = phi_893_;
                    int _e191_ = phi_892_;
                    local = _e189_;
                    local_1_ = _e189_;
                    if (_e191_ >= 0) {
                        float _e195_ = _e189_ + metal::exp2(static_cast<float>(_e191_));
                        phi_1021_ = _e189_;
                        if (_e195_ <= metal::min(_e140_ - 1.0, _e135_)) {
                            phi_1021_ = (metal::dot(metal::normalize((((_e175_ * _e195_) + (_e173_ * (_e140_ * 2.0))) * _e195_) + (_e170_ * (_e140_ * _e140_))), _e183_) >= metal::cos(metal::min((_e195_ * -(_e184_)) + ((1.0 + _e135_) * _e184_), 3.1415927))) ? _e195_ : _e189_;
                        }
                        float _e210_ = phi_1021_;
                        local_2_ = _e210_;
                        continue;
                    } else {
                        break;
                    }
                }
                float _e213_ = local;
                float _e216_ = local_1_;
                float _e217_ = _e135_ - _e216_;
                float _e220_ = metal::acos(metal::clamp(_e183_.x, -1.0, 1.0));
                if (_e183_.y >= 0.0) {
                    phi_896_ = _e220_;
                } else {
                    phi_896_ = -(_e220_);
                }
                float _e225_ = phi_896_;
                float _e227_ = (_e217_ * _e127_) + _e225_;
                metal::float2 _e231_ = metal::float2(metal::sin(_e227_), -(metal::cos(_e227_)));
                float _e232_ = metal::dot(_e231_, _e175_);
                float _e233_ = metal::dot(_e231_, _e173_);
                float _e234_ = metal::dot(_e231_, _e170_);
                float _e236_ = _e232_ * _e234_;
                float _e239_ = metal::sqrt(metal::max((_e233_ * _e233_) - _e236_, 0.0));
                phi_899_ = _e239_;
                if (_e233_ > 0.0) {
                    phi_899_ = -(_e239_);
                }
                float _e243_ = phi_899_;
                float _e244_ = _e243_ - _e233_;
                float _e246_ = (-0.5 * _e244_) * _e232_;
                if (metal::abs((_e244_ * _e244_) + _e246_) < metal::abs(_e236_ + _e246_)) {
                    phi_900_ = metal::float2(_e244_, _e232_);
                } else {
                    phi_900_ = metal::float2(_e234_, _e244_);
                }
                metal::float2 _e256_ = phi_900_;
                if (_e256_.y != 0.0) {
                    phi_901_ = _e256_.x / _e256_.y;
                } else {
                    phi_901_ = 0.0;
                }
                float _e262_ = phi_901_;
                float _e265_ = (_e217_ == 0.0) ? 0.0 : metal::clamp(_e262_, 0.0, 1.0);
                phi_987_ = _e227_;
                phi_933_ = _e265_;
                phi_923_ = metal::max(_e213_ / _e140_, _e265_);
            }
            float _e268_ = phi_987_;
            float _e270_ = phi_933_;
            float _e272_ = phi_923_;
            metal::float2 _e275_ = ((_e139_ - _e138_) * _e272_) + _e138_;
            metal::float2 _e278_ = ((_e137_ - _e139_) * _e272_) + _e139_;
            metal::float2 _e284_ = ((_e278_ - _e275_) * _e272_) + _e275_;
            metal::float2 _e288_ = ((((((_e45_ - _e137_) * _e272_) + _e137_) - _e278_) * _e272_) + _e278_) - _e284_;
            phi_986_ = _e268_;
            if (_e272_ != _e270_) {
                metal::float2 _e292_ = metal::normalize(_e288_);
                float _e295_ = metal::acos(metal::clamp(_e292_.x, -1.0, 1.0));
                if (_e292_.y >= 0.0) {
                    phi_934_ = _e295_;
                } else {
                    phi_934_ = -(_e295_);
                }
                float _e300_ = phi_934_;
                phi_986_ = _e300_;
            }
            float _e302_ = phi_986_;
            phi_984_ = _e302_;
            phi_940_ = (_e288_ * _e272_) + _e284_;
        }
        float _e304_ = phi_984_;
        metal::float2 _e306_ = phi_940_;
        phi_983_ = _e304_;
        phi_937_ = _e306_;
    }
    float _e308_ = phi_983_;
    metal::float2 _e310_ = phi_937_;
    metal::uint2 _e311_ = as_type<metal::uint2>(_e310_);
    metal::uint4 _e317_ = metal::uint4(_e311_.x, uint {}, uint {}, uint {});
    metal::uint4 _e323_ = metal::uint4(_e317_.x, _e311_.y, _e317_.z, _e317_.w);
    if ((_e131_ & 469762048u) == 67108864u) {
        phi_1020_ = metal::uint4(_e323_.x, _e323_.y, (naga_f2u32(_e133_) << as_type<uint>(16)) | naga_f2u32(_e135_), _e323_.w);
    } else {
        phi_1020_ = metal::uint4(_e323_.x, _e323_.y, as_type<uint>(_e308_ - (metal::floor(_e308_ / 6.2831855) * 6.2831855)), _e323_.w);
    }
    metal::uint4 _e347_ = phi_1020_;
    Jg = metal::uint4(_e347_.x, _e347_.y, _e347_.z, _e131_);
    return;
}

struct main_Input {
    metal::float4 x6_ [[user(loc0), center_perspective]];
    metal::float4 y6_ [[user(loc1), center_perspective]];
    metal::float4 L4_ [[user(loc2), center_perspective]];
    uint F7_ [[user(loc4), flat]];
    metal::float3 C5_ [[user(loc3), center_perspective]];
};
struct main_Output {
    metal::uint4 member [[color(0)]];
};
fragment main_Output main_(
  main_Input varyings [[stage_in]]
) {
    metal::float4 x6_1_ = {};
    metal::float4 y6_1_ = {};
    metal::float4 L4_1_ = {};
    uint F7_1_ = {};
    metal::float3 C5_1_ = {};
    metal::uint4 Jg = {};
    const auto x6_ = varyings.x6_;
    const auto y6_ = varyings.y6_;
    const auto L4_ = varyings.L4_;
    const auto F7_ = varyings.F7_;
    const auto C5_ = varyings.C5_;
    x6_1_ = x6_;
    y6_1_ = y6_;
    L4_1_ = L4_;
    F7_1_ = F7_;
    C5_1_ = C5_;
    main_1_(x6_1_, y6_1_, L4_1_, F7_1_, C5_1_, Jg);
    metal::uint4 _e11_ = Jg;
    return main_Output { _e11_ };
}
