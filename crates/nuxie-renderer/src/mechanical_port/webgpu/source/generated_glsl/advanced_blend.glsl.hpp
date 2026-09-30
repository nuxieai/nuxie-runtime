#pragma once

#include "advanced_blend.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char advanced_blend[] = R"===(#ifdef EB
#ifdef LE
layout(
#ifdef DC
blend_support_all_equations
#else
blend_support_multiply,blend_support_screen,blend_support_overlay,blend_support_darken,blend_support_lighten,blend_support_colordodge,blend_support_colorburn,blend_support_hardlight,blend_support_softlight,blend_support_difference,blend_support_exclusion
#endif
)out;
#endif
#ifdef S
#ifdef DC
d Fb(A I1){return dot(I1,S0(.30,.59,.11));}A o9(A Gb,A p9){d q9=Fb(p9);A r9=Gb-Fb(Gb);E Hb=D2(q9,1.0-q9)/max(D2(v9),D2(-k3(r9),O5(r9)));d Je=min(J0(1.0),min(Hb.x,Hb.y));return r9*Je+q9;}A Ib(A T7,A Jb,A p9){float Ke=O5(Jb)-k3(Jb);T7-=k3(T7);float Le=O5(T7);float E2=Ke/max(v9,Le);return o9(T7*E2,p9);}
#endif
A Me(A o0,i z1,L w9){A w0=H6(z1);A Y0;switch(w9){case Ne:Y0=o0.xyz*w0.xyz;break;case Oe:Y0=o0.xyz+w0.xyz-o0.xyz*w0.xyz;break;case Pe:{A I6=o0*w0;Y0=2.0*mix(I6,o0+w0-I6-0.5,greaterThan(w0,S0(0.5)));break;}case Qe:Y0=min(o0.xyz,w0.xyz);break;case Re:Y0=max(o0.xyz,w0.xyz);break;case Se:{z1.xyz=clamp(z1.xyz,S0(.0),z1.www);A Kb=clamp(1.-o0,S0(.0),S0(1.))*z1.w;Y0=mix(min(S0(1.),z1.xyz/Kb),sign(z1.xyz),equal(Kb,S0(.0)));break;}case Ue:{o0=clamp(o0,S0(.0),S0(1.));z1.xyz=clamp(z1.xyz,S0(.0),z1.www);if(z1.w==.0)z1.w=1.;A Lb=z1.w-z1.xyz;Y0=1.-mix(min(S0(1.),Lb/(o0*z1.w)),sign(Lb),equal(o0,S0(.0)));break;}case Ve:{A I6=o0*w0;Y0=2.0*mix(I6,o0+w0-I6-0.5,greaterThan(o0,S0(0.5)));break;}case We:{for(int H0=0;H0<3;++H0){if(o0[H0]<=0.5)Y0[H0]=(1.0-w0[H0]);else if(w0[H0]<=0.25)Y0[H0]=((16.0*w0[H0]-12.0)*w0[H0]+3.0);else Y0[H0]=(inversesqrt(w0[H0])-1.0);}Y0=w0+w0*(2.0*o0-1.0)*Y0;break;}case Xe:Y0=abs(w0.xyz-o0.xyz);break;case Ye:Y0=o0.xyz+w0.xyz-2.*o0.xyz*w0.xyz;break;
#ifdef DC
case Ze:if(DC){o0.xyz=clamp(o0.xyz,S0(.0),S0(1.));Y0=Ib(o0.xyz,w0.xyz,w0.xyz);}break;case af:if(DC){o0.xyz=clamp(o0.xyz,S0(.0),S0(1.));Y0=Ib(w0.xyz,o0.xyz,w0.xyz);}break;case bf:if(DC){o0.xyz=clamp(o0.xyz,S0(.0),S0(1.));Y0=o9(o0.xyz,w0.xyz);}break;case cf:if(DC){o0.xyz=clamp(o0.xyz,S0(.0),S0(1.));Y0=o9(w0.xyz,o0.xyz);}break;
#endif
}return Y0;}e A X4(A o0,i z1,L w9){A Y0=Me(o0,z1,w9);return mix(o0,Y0,S0(z1.w));}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive