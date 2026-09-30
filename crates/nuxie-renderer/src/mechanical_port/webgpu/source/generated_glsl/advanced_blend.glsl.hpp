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
d Hb(A J1){return dot(J1,T0(.30,.59,.11));}A o9(A Ib,A p9){d q9=Hb(p9);A r9=Ib-Hb(Ib);E Jb=D2(q9,1.0-q9)/max(D2(v9),D2(-m3(r9),P5(r9)));d Oe=min(J0(1.0),min(Jb.x,Jb.y));return r9*Oe+q9;}A Kb(A T7,A Lb,A p9){float Pe=P5(Lb)-m3(Lb);T7-=m3(T7);float Qe=P5(T7);float E2=Pe/max(v9,Qe);return o9(T7*E2,p9);}
#endif
A Re(A p0,i A1,N w9){A w0=I6(A1);A Z0;switch(w9){case Se:Z0=p0.xyz*w0.xyz;break;case Te:Z0=p0.xyz+w0.xyz-p0.xyz*w0.xyz;break;case Ue:{A J6=p0*w0;Z0=2.0*mix(J6,p0+w0-J6-0.5,greaterThan(w0,T0(0.5)));break;}case Ve:Z0=min(p0.xyz,w0.xyz);break;case We:Z0=max(p0.xyz,w0.xyz);break;case Xe:{A1.xyz=clamp(A1.xyz,T0(.0),A1.www);A Mb=clamp(1.-p0,T0(.0),T0(1.))*A1.w;Z0=mix(min(T0(1.),A1.xyz/Mb),sign(A1.xyz),equal(Mb,T0(.0)));break;}case Ze:{p0=clamp(p0,T0(.0),T0(1.));A1.xyz=clamp(A1.xyz,T0(.0),A1.www);if(A1.w==.0)A1.w=1.;A Nb=A1.w-A1.xyz;Z0=1.-mix(min(T0(1.),Nb/(p0*A1.w)),sign(Nb),equal(p0,T0(.0)));break;}case af:{A J6=p0*w0;Z0=2.0*mix(J6,p0+w0-J6-0.5,greaterThan(p0,T0(0.5)));break;}case bf:{for(int H0=0;H0<3;++H0){if(p0[H0]<=0.5)Z0[H0]=(1.0-w0[H0]);else if(w0[H0]<=0.25)Z0[H0]=((16.0*w0[H0]-12.0)*w0[H0]+3.0);else Z0[H0]=(inversesqrt(w0[H0])-1.0);}Z0=w0+w0*(2.0*p0-1.0)*Z0;break;}case cf:Z0=abs(w0.xyz-p0.xyz);break;case df:Z0=p0.xyz+w0.xyz-2.*p0.xyz*w0.xyz;break;
#ifdef DC
case ef:if(DC){p0.xyz=clamp(p0.xyz,T0(.0),T0(1.));Z0=Kb(p0.xyz,w0.xyz,w0.xyz);}break;case ff:if(DC){p0.xyz=clamp(p0.xyz,T0(.0),T0(1.));Z0=Kb(w0.xyz,p0.xyz,w0.xyz);}break;case gf:if(DC){p0.xyz=clamp(p0.xyz,T0(.0),T0(1.));Z0=o9(p0.xyz,w0.xyz);}break;case hf:if(DC){p0.xyz=clamp(p0.xyz,T0(.0),T0(1.));Z0=o9(w0.xyz,p0.xyz);}break;
#endif
}return Z0;}e A Z4(A p0,i A1,N w9){A Z0=Re(p0,A1,w9);return mix(p0,Z0,T0(A1.w));}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive