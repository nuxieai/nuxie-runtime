#pragma once

#include "advanced_blend.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char advanced_blend[] = R"===(#ifdef GB
#ifdef JE
layout(
#ifdef GC
blend_support_all_equations
#else
blend_support_multiply,blend_support_screen,blend_support_overlay,blend_support_darken,blend_support_lighten,blend_support_colordodge,blend_support_colorburn,blend_support_hardlight,blend_support_softlight,blend_support_difference,blend_support_exclusion
#endif
)out;
#endif
#ifdef AB
#ifdef GC
c Ab(A H1){return dot(H1,Q0(.30,.59,.11));}A k9(A Bb,A l9){c m9=Ab(l9);A n9=Bb-Ab(Bb);E Cb=B2(m9,1.0-m9)/max(B2(o9),B2(-h3(n9),N5(n9)));c we=min(G0(1.0),min(Cb.x,Cb.y));return n9*we+m9;}A Db(A Q7,A Eb,A l9){float xe=N5(Eb)-h3(Eb);Q7-=h3(Q7);float ye=N5(Q7);float C2=xe/max(o9,ye);return k9(Q7*C2,l9);}
#endif
A ze(A n0,i y1,N p9){A q0=G6(y1);A X0;switch(p9){case Ae:X0=n0.xyz*q0.xyz;break;case Be:X0=n0.xyz+q0.xyz-n0.xyz*q0.xyz;break;case Ce:{A H6=n0*q0;X0=2.0*mix(H6,n0+q0-H6-0.5,greaterThan(q0,Q0(0.5)));break;}case De:X0=min(n0.xyz,q0.xyz);break;case Ee:X0=max(n0.xyz,q0.xyz);break;case Fe:{y1.xyz=clamp(y1.xyz,Q0(.0),y1.www);A Fb=clamp(1.-n0,Q0(.0),Q0(1.))*y1.w;X0=mix(min(Q0(1.),y1.xyz/Fb),sign(y1.xyz),equal(Fb,Q0(.0)));break;}case He:{n0=clamp(n0,Q0(.0),Q0(1.));y1.xyz=clamp(y1.xyz,Q0(.0),y1.www);if(y1.w==.0)y1.w=1.;A Gb=y1.w-y1.xyz;X0=1.-mix(min(Q0(1.),Gb/(n0*y1.w)),sign(Gb),equal(n0,Q0(.0)));break;}case Ie:{A H6=n0*q0;X0=2.0*mix(H6,n0+q0-H6-0.5,greaterThan(n0,Q0(0.5)));break;}case Je:{for(int F0=0;F0<3;++F0){if(n0[F0]<=0.5)X0[F0]=(1.0-q0[F0]);else if(q0[F0]<=0.25)X0[F0]=((16.0*q0[F0]-12.0)*q0[F0]+3.0);else X0[F0]=(inversesqrt(q0[F0])-1.0);}X0=q0+q0*(2.0*n0-1.0)*X0;break;}case Ke:X0=abs(q0.xyz-n0.xyz);break;case Le:X0=n0.xyz+q0.xyz-2.*n0.xyz*q0.xyz;break;
#ifdef GC
case Me:if(GC){n0.xyz=clamp(n0.xyz,Q0(.0),Q0(1.));X0=Db(n0.xyz,q0.xyz,q0.xyz);}break;case Ne:if(GC){n0.xyz=clamp(n0.xyz,Q0(.0),Q0(1.));X0=Db(q0.xyz,n0.xyz,q0.xyz);}break;case Oe:if(GC){n0.xyz=clamp(n0.xyz,Q0(.0),Q0(1.));X0=k9(n0.xyz,q0.xyz);}break;case Pe:if(GC){n0.xyz=clamp(n0.xyz,Q0(.0),Q0(1.));X0=k9(q0.xyz,n0.xyz);}break;
#endif
}return X0;}e A U4(A n0,i y1,N p9){A X0=ze(n0,y1,p9);return mix(n0,X0,Q0(y1.w));}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive