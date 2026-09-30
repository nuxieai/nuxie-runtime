#pragma once

#include "advanced_blend.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char advanced_blend[] = R"===(#ifdef EB
#ifdef ME
layout(
#ifdef GC
blend_support_all_equations
#else
blend_support_multiply,blend_support_screen,blend_support_overlay,blend_support_darken,blend_support_lighten,blend_support_colordodge,blend_support_colorburn,blend_support_hardlight,blend_support_softlight,blend_support_difference,blend_support_exclusion
#endif
) out;
#endif
#ifdef O
#ifdef GC
d ec(v O1){return dot(O1,R0(.30,.59,.11));}v G9(v fc,v H9){d I9=ec(H9);v J9=fc-ec(fc);C gc=I2(I9,1.0-I9)/max(I2(K9),I2(-v3(J9),Z5(J9)));d df=min(I0(1.0),min(gc.x,gc.y));return J9*df+I9;}v hc(v g8,v ic,v H9){float ef=Z5(ic)-v3(ic);g8-=v3(g8);float ff=Z5(g8);float J2=ef/max(K9,ff);return G9(g8*J2,H9);}
#endif
v gf(v r0,i G1,R c4){v x0=R6(G1);v g1;switch(c4){case hf:g1=r0.xyz*x0.xyz;break;case jf:g1=r0.xyz+x0.xyz-r0.xyz*x0.xyz;break;case kf:{v S6=r0*x0;g1=2.0*mix(S6,r0+x0-S6-0.5,greaterThan(x0,R0(0.5)));break;}case lf:g1=min(r0.xyz,x0.xyz);break;case mf:g1=max(r0.xyz,x0.xyz);break;case nf:{G1.xyz=clamp(G1.xyz,R0(.0),G1.www);v jc=clamp(1.-r0,R0(.0),R0(1.))*G1.w;g1=mix(min(R0(1.),G1.xyz/jc),sign(G1.xyz),equal(jc,R0(.0)));break;}case pf:{r0=clamp(r0,R0(.0),R0(1.));G1.xyz=clamp(G1.xyz,R0(.0),G1.www);if(G1.w==.0) G1.w=1.;v kc=G1.w-G1.xyz;g1=1.-mix(min(R0(1.),kc/(r0*G1.w)),sign(kc),equal(r0,R0(.0)));break;}case qf:{v S6=r0*x0;g1=2.0*mix(S6,r0+x0-S6-0.5,greaterThan(r0,R0(0.5)));break;}case rf:{for(int L0=0;L0<3;++L0){if(r0[L0]<=0.5) g1[L0]=(1.0-x0[L0]);else if(x0[L0]<=0.25) g1[L0]=((16.0*x0[L0]-12.0)*x0[L0]+3.0);else g1[L0]=(inversesqrt(x0[L0])-1.0);}g1=x0+x0*(2.0*r0-1.0)*g1;break;}case sf:g1=abs(x0.xyz-r0.xyz);break;case tf:g1=r0.xyz+x0.xyz-2.*r0.xyz*x0.xyz;break;
#ifdef GC
case uf:if(GC){r0.xyz=clamp(r0.xyz,R0(.0),R0(1.));g1=hc(r0.xyz,x0.xyz,x0.xyz);}break;case vf:if(GC){r0.xyz=clamp(r0.xyz,R0(.0),R0(1.));g1=hc(x0.xyz,r0.xyz,x0.xyz);}break;case wf:if(GC){r0.xyz=clamp(r0.xyz,R0(.0),R0(1.));g1=G9(r0.xyz,x0.xyz);}break;case xf:if(GC){r0.xyz=clamp(r0.xyz,R0(.0),R0(1.));g1=G9(x0.xyz,r0.xyz);}break;
#endif
}return g1;}e v i5(v r0,i G1,R c4){v g1=gf(r0,G1,c4);return mix(r0,g1,R0(G1.w));}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive