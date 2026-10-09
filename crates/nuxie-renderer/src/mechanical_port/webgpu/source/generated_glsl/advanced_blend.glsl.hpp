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
d Pc(v R1){return dot(R1,Z0(.30,.59,.11));}v ka(v Qc,v la){d ma=Pc(la);v na=Qc-Pc(Qc);D Rc=Q2(ma,1.0-ma)/max(Q2(oa),Q2(-B3(na),h6(na)));d If=min(J0(1.0),min(Rc.x,Rc.y));return na*If+ma;}v Sc(v A8,v Tc,v la){float Jf=h6(Tc)-B3(Tc);A8-=B3(A8);float Kf=h6(A8);float R2=Jf/max(oa,Kf);return ka(A8*R2,la);}
#endif
v Lf(v r0,i I1,P h4){v x0=i6(I1);v i1;switch(h4){case Mf:i1=r0.xyz*x0.xyz;break;case Nf:i1=r0.xyz+x0.xyz-r0.xyz*x0.xyz;break;case Of:{v e7=r0*x0;i1=2.0*mix(e7,r0+x0-e7-0.5,greaterThan(x0,Z0(0.5)));break;}case Pf:i1=min(r0.xyz,x0.xyz);break;case Qf:i1=max(r0.xyz,x0.xyz);break;case Rf:{I1.xyz=clamp(I1.xyz,Z0(.0),I1.www);v Uc=clamp(1.-r0,Z0(.0),Z0(1.))*I1.w;i1=mix(min(Z0(1.),I1.xyz/Uc),sign(I1.xyz),equal(Uc,Z0(.0)));break;}case Tf:{r0=clamp(r0,Z0(.0),Z0(1.));I1.xyz=clamp(I1.xyz,Z0(.0),I1.www);if(I1.w==.0) I1.w=1.;v Vc=I1.w-I1.xyz;i1=1.-mix(min(Z0(1.),Vc/(r0*I1.w)),sign(Vc),equal(r0,Z0(.0)));break;}case Uf:{v e7=r0*x0;i1=2.0*mix(e7,r0+x0-e7-0.5,greaterThan(r0,Z0(0.5)));break;}case Vf:{for(int M0=0;M0<3;++M0){if(r0[M0]<=0.5) i1[M0]=(1.0-x0[M0]);else if(x0[M0]<=0.25) i1[M0]=((16.0*x0[M0]-12.0)*x0[M0]+3.0);else i1[M0]=(inversesqrt(x0[M0])-1.0);}i1=x0+x0*(2.0*r0-1.0)*i1;break;}case Wf:i1=abs(x0.xyz-r0.xyz);break;case Xf:i1=r0.xyz+x0.xyz-2.*r0.xyz*x0.xyz;break;
#ifdef FC
case Yf:if(FC){r0.xyz=clamp(r0.xyz,Z0(.0),Z0(1.));i1=Sc(r0.xyz,x0.xyz,x0.xyz);}break;case Zf:if(FC){r0.xyz=clamp(r0.xyz,Z0(.0),Z0(1.));i1=Sc(x0.xyz,r0.xyz,x0.xyz);}break;case ag:if(FC){r0.xyz=clamp(r0.xyz,Z0(.0),Z0(1.));i1=ka(r0.xyz,x0.xyz);}break;case bg:if(FC){r0.xyz=clamp(r0.xyz,Z0(.0),Z0(1.));i1=ka(x0.xyz,r0.xyz);}break;
#endif
}return i1;}e v N4(v r0,i I1,P h4){v i1=Lf(r0,I1,h4);return mix(r0,i1,Z0(I1.w));}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive