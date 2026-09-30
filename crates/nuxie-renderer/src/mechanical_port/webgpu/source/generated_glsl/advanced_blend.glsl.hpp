#pragma once

#include "advanced_blend.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char advanced_blend[] = R"===(#ifdef FB
#ifdef ME
layout(
#ifdef EC
blend_support_all_equations
#else
blend_support_multiply,blend_support_screen,blend_support_overlay,blend_support_darken,blend_support_lighten,blend_support_colordodge,blend_support_colorburn,blend_support_hardlight,blend_support_softlight,blend_support_difference,blend_support_exclusion
#endif
)out;
#endif
#ifdef AB
#ifdef EC
d Eb(A G1){return dot(G1,R0(.30,.59,.11));}A n9(A Fb,A o9){d p9=Eb(o9);A q9=Fb-Eb(Fb);E Gb=C2(p9,1.0-p9)/max(C2(r9),C2(-i3(q9),K5(q9)));d Ie=min(I0(1.0),min(Gb.x,Gb.y));return q9*Ie+p9;}A Hb(A R7,A Ib,A o9){float Je=K5(Ib)-i3(Ib);R7-=i3(R7);float Ke=K5(R7);float D2=Je/max(r9,Ke);return n9(R7*D2,o9);}
#endif
A Le(A n0,i x1,L v9){A v0=G6(x1);A X0;switch(v9){case Me:X0=n0.xyz*v0.xyz;break;case Ne:X0=n0.xyz+v0.xyz-n0.xyz*v0.xyz;break;case Oe:{A H6=n0*v0;X0=2.0*mix(H6,n0+v0-H6-0.5,greaterThan(v0,R0(0.5)));break;}case Pe:X0=min(n0.xyz,v0.xyz);break;case Qe:X0=max(n0.xyz,v0.xyz);break;case Re:{x1.xyz=clamp(x1.xyz,R0(.0),x1.www);A Jb=clamp(1.-n0,R0(.0),R0(1.))*x1.w;X0=mix(min(R0(1.),x1.xyz/Jb),sign(x1.xyz),equal(Jb,R0(.0)));break;}case Te:{n0=clamp(n0,R0(.0),R0(1.));x1.xyz=clamp(x1.xyz,R0(.0),x1.www);if(x1.w==.0)x1.w=1.;A Kb=x1.w-x1.xyz;X0=1.-mix(min(R0(1.),Kb/(n0*x1.w)),sign(Kb),equal(n0,R0(.0)));break;}case Ue:{A H6=n0*v0;X0=2.0*mix(H6,n0+v0-H6-0.5,greaterThan(n0,R0(0.5)));break;}case Ve:{for(int G0=0;G0<3;++G0){if(n0[G0]<=0.5)X0[G0]=(1.0-v0[G0]);else if(v0[G0]<=0.25)X0[G0]=((16.0*v0[G0]-12.0)*v0[G0]+3.0);else X0[G0]=(inversesqrt(v0[G0])-1.0);}X0=v0+v0*(2.0*n0-1.0)*X0;break;}case We:X0=abs(v0.xyz-n0.xyz);break;case Xe:X0=n0.xyz+v0.xyz-2.*n0.xyz*v0.xyz;break;
#ifdef EC
case Ye:if(EC){n0.xyz=clamp(n0.xyz,R0(.0),R0(1.));X0=Hb(n0.xyz,v0.xyz,v0.xyz);}break;case Ze:if(EC){n0.xyz=clamp(n0.xyz,R0(.0),R0(1.));X0=Hb(v0.xyz,n0.xyz,v0.xyz);}break;case af:if(EC){n0.xyz=clamp(n0.xyz,R0(.0),R0(1.));X0=n9(n0.xyz,v0.xyz);}break;case bf:if(EC){n0.xyz=clamp(n0.xyz,R0(.0),R0(1.));X0=n9(v0.xyz,n0.xyz);}break;
#endif
}return X0;}e A U4(A n0,i x1,L v9){A X0=Le(n0,x1,v9);return mix(n0,X0,R0(x1.w));}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive