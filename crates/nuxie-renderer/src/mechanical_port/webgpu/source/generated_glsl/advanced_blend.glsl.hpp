#pragma once

#include "advanced_blend.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char advanced_blend[] = R"===(#ifdef EB
#ifdef NE
layout(
#ifdef FC
blend_support_all_equations
#else
blend_support_multiply,blend_support_screen,blend_support_overlay,blend_support_darken,blend_support_lighten,blend_support_colordodge,blend_support_colorburn,blend_support_hardlight,blend_support_softlight,blend_support_difference,blend_support_exclusion
#endif
) out;
#endif
#ifdef H
#ifdef FC
d Jc(v S1){return dot(S1,a1(.30,.59,.11));}v fa(v Kc,v ga){d ha=Jc(ga);v ia=Kc-Jc(Kc);D Lc=R2(ha,1.0-ha)/max(R2(ja),R2(-A3(ia),e6(ia)));d If=min(J0(1.0),min(Lc.x,Lc.y));return ia*If+ha;}v Mc(v y8,v Nc,v ga){float Jf=e6(Nc)-A3(Nc);y8-=A3(y8);float Kf=e6(y8);float S2=Jf/max(ja,Kf);return fa(y8*S2,ga);}
#endif
v Lf(v r0,i J1,P g4){v x0=f6(J1);v k1;switch(g4){case Mf:k1=r0.xyz*x0.xyz;break;case Nf:k1=r0.xyz+x0.xyz-r0.xyz*x0.xyz;break;case Of:{v Z6=r0*x0;k1=2.0*mix(Z6,r0+x0-Z6-0.5,greaterThan(x0,a1(0.5)));break;}case Pf:k1=min(r0.xyz,x0.xyz);break;case Qf:k1=max(r0.xyz,x0.xyz);break;case Rf:{J1.xyz=clamp(J1.xyz,a1(.0),J1.www);v Oc=clamp(1.-r0,a1(.0),a1(1.))*J1.w;k1=mix(min(a1(1.),J1.xyz/Oc),sign(J1.xyz),equal(Oc,a1(.0)));break;}case Tf:{r0=clamp(r0,a1(.0),a1(1.));J1.xyz=clamp(J1.xyz,a1(.0),J1.www);if(J1.w==.0) J1.w=1.;v Pc=J1.w-J1.xyz;k1=1.-mix(min(a1(1.),Pc/(r0*J1.w)),sign(Pc),equal(r0,a1(.0)));break;}case Uf:{v Z6=r0*x0;k1=2.0*mix(Z6,r0+x0-Z6-0.5,greaterThan(r0,a1(0.5)));break;}case Vf:{for(int N0=0;N0<3;++N0){if(r0[N0]<=0.5) k1[N0]=(1.0-x0[N0]);else if(x0[N0]<=0.25) k1[N0]=((16.0*x0[N0]-12.0)*x0[N0]+3.0);else k1[N0]=(inversesqrt(x0[N0])-1.0);}k1=x0+x0*(2.0*r0-1.0)*k1;break;}case Wf:k1=abs(x0.xyz-r0.xyz);break;case Xf:k1=r0.xyz+x0.xyz-2.*r0.xyz*x0.xyz;break;
#ifdef FC
case Yf:if(FC){r0.xyz=clamp(r0.xyz,a1(.0),a1(1.));k1=Mc(r0.xyz,x0.xyz,x0.xyz);}break;case Zf:if(FC){r0.xyz=clamp(r0.xyz,a1(.0),a1(1.));k1=Mc(x0.xyz,r0.xyz,x0.xyz);}break;case ag:if(FC){r0.xyz=clamp(r0.xyz,a1(.0),a1(1.));k1=fa(r0.xyz,x0.xyz);}break;case bg:if(FC){r0.xyz=clamp(r0.xyz,a1(.0),a1(1.));k1=fa(x0.xyz,r0.xyz);}break;
#endif
}return k1;}f v L4(v r0,i J1,P g4){v k1=Lf(r0,J1,g4);return mix(r0,k1,a1(J1.w));}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive