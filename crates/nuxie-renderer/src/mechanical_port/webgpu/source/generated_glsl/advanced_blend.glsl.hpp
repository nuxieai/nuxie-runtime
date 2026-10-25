#pragma once

#include "advanced_blend.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char advanced_blend[] = R"===(#ifdef DB
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
d Xc(v W1){return dot(W1,Z0(.30,.59,.11));}v na(v Yc,v oa){d pa=Xc(oa);v qa=Yc-Xc(Yc);E Zc=S2(pa,1.0-pa)/max(S2(ra),S2(-C3(qa),k6(qa)));d Tf=min(C0(1.0),min(Zc.x,Zc.y));return qa*Tf+pa;}v ad(v F8,v bd,v oa){float Uf=k6(bd)-C3(bd);F8-=C3(F8);float Vf=k6(F8);float T2=Uf/max(ra,Vf);return na(F8*T2,oa);}
#endif
v Wf(v v0,i z1,P D3){v z0=l6(z1);v j1;switch(D3){case Xf:j1=v0.xyz*z0.xyz;break;case Yf:j1=v0.xyz+z0.xyz-v0.xyz*z0.xyz;break;case Zf:{v i7=v0*z0;j1=2.0*mix(i7,v0+z0-i7-0.5,greaterThan(z0,Z0(0.5)));break;}case ag:j1=min(v0.xyz,z0.xyz);break;case bg:j1=max(v0.xyz,z0.xyz);break;case cg:{z1.xyz=clamp(z1.xyz,Z0(.0),z1.www);v cd=clamp(1.-v0,Z0(.0),Z0(1.))*z1.w;j1=mix(min(Z0(1.),z1.xyz/cd),sign(z1.xyz),equal(cd,Z0(.0)));break;}case eg:{v0=clamp(v0,Z0(.0),Z0(1.));z1.xyz=clamp(z1.xyz,Z0(.0),z1.www);if(z1.w==.0) z1.w=1.;v dd=z1.w-z1.xyz;j1=1.-mix(min(Z0(1.),dd/(v0*z1.w)),sign(dd),equal(v0,Z0(.0)));break;}case fg:{v i7=v0*z0;j1=2.0*mix(i7,v0+z0-i7-0.5,greaterThan(v0,Z0(0.5)));break;}case gg:{for(int Q0=0;Q0<3;++Q0){if(v0[Q0]<=0.5) j1[Q0]=(1.0-z0[Q0]);else if(z0[Q0]<=0.25) j1[Q0]=((16.0*z0[Q0]-12.0)*z0[Q0]+3.0);else j1[Q0]=(inversesqrt(z0[Q0])-1.0);}j1=z0+z0*(2.0*v0-1.0)*j1;break;}case hg:j1=abs(z0.xyz-v0.xyz);break;case ig:j1=v0.xyz+z0.xyz-2.*v0.xyz*z0.xyz;break;
#ifdef GC
case jg:if(GC){v0.xyz=clamp(v0.xyz,Z0(.0),Z0(1.));j1=ad(v0.xyz,z0.xyz,z0.xyz);}break;case kg:if(GC){v0.xyz=clamp(v0.xyz,Z0(.0),Z0(1.));j1=ad(z0.xyz,v0.xyz,z0.xyz);}break;case lg:if(GC){v0.xyz=clamp(v0.xyz,Z0(.0),Z0(1.));j1=na(v0.xyz,z0.xyz);}break;case mg:if(GC){v0.xyz=clamp(v0.xyz,Z0(.0),Z0(1.));j1=na(z0.xyz,v0.xyz);}break;
#endif
}return j1;}f v R4(v v0,i z1,P D3){v j1=Wf(v0,z1,D3);return mix(v0,j1,Z0(z1.w));}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive