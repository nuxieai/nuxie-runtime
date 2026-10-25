#pragma once

#include "advanced_blend.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char advanced_blend[] = R"===(#ifdef EB
#ifdef SE
layout(
#ifdef GC
blend_support_all_equations
#else
blend_support_multiply,blend_support_screen,blend_support_overlay,blend_support_darken,blend_support_lighten,blend_support_colordodge,blend_support_colorburn,blend_support_hardlight,blend_support_softlight,blend_support_difference,blend_support_exclusion
#endif
) out;
#endif
#ifdef H
#ifdef GC
d Uc(v W1){return dot(W1,Z0(.30,.59,.11));}v ma(v Vc,v na){d oa=Uc(na);v pa=Vc-Uc(Vc);E Wc=S2(oa,1.0-oa)/max(S2(qa),S2(-C3(pa),k6(pa)));d Nf=min(J0(1.0),min(Wc.x,Wc.y));return pa*Nf+oa;}v Xc(v E8,v Yc,v na){float Of=k6(Yc)-C3(Yc);E8-=C3(E8);float Pf=k6(E8);float T2=Of/max(qa,Pf);return ma(E8*T2,na);}
#endif
v Qf(v v0,i z1,P j4){v z0=l6(z1);v j1;switch(j4){case Rf:j1=v0.xyz*z0.xyz;break;case Sf:j1=v0.xyz+z0.xyz-v0.xyz*z0.xyz;break;case Tf:{v j7=v0*z0;j1=2.0*mix(j7,v0+z0-j7-0.5,greaterThan(z0,Z0(0.5)));break;}case Uf:j1=min(v0.xyz,z0.xyz);break;case Vf:j1=max(v0.xyz,z0.xyz);break;case Wf:{z1.xyz=clamp(z1.xyz,Z0(.0),z1.www);v Zc=clamp(1.-v0,Z0(.0),Z0(1.))*z1.w;j1=mix(min(Z0(1.),z1.xyz/Zc),sign(z1.xyz),equal(Zc,Z0(.0)));break;}case Yf:{v0=clamp(v0,Z0(.0),Z0(1.));z1.xyz=clamp(z1.xyz,Z0(.0),z1.www);if(z1.w==.0) z1.w=1.;v ad=z1.w-z1.xyz;j1=1.-mix(min(Z0(1.),ad/(v0*z1.w)),sign(ad),equal(v0,Z0(.0)));break;}case Zf:{v j7=v0*z0;j1=2.0*mix(j7,v0+z0-j7-0.5,greaterThan(v0,Z0(0.5)));break;}case ag:{for(int N0=0;N0<3;++N0){if(v0[N0]<=0.5) j1[N0]=(1.0-z0[N0]);else if(z0[N0]<=0.25) j1[N0]=((16.0*z0[N0]-12.0)*z0[N0]+3.0);else j1[N0]=(inversesqrt(z0[N0])-1.0);}j1=z0+z0*(2.0*v0-1.0)*j1;break;}case bg:j1=abs(z0.xyz-v0.xyz);break;case cg:j1=v0.xyz+z0.xyz-2.*v0.xyz*z0.xyz;break;
#ifdef GC
case dg:if(GC){v0.xyz=clamp(v0.xyz,Z0(.0),Z0(1.));j1=Xc(v0.xyz,z0.xyz,z0.xyz);}break;case eg:if(GC){v0.xyz=clamp(v0.xyz,Z0(.0),Z0(1.));j1=Xc(z0.xyz,v0.xyz,z0.xyz);}break;case fg:if(GC){v0.xyz=clamp(v0.xyz,Z0(.0),Z0(1.));j1=ma(v0.xyz,z0.xyz);}break;case gg:if(GC){v0.xyz=clamp(v0.xyz,Z0(.0),Z0(1.));j1=ma(z0.xyz,v0.xyz);}break;
#endif
}return j1;}f v P4(v v0,i z1,P j4){v j1=Qf(v0,z1,j4);return mix(v0,j1,Z0(z1.w));}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive