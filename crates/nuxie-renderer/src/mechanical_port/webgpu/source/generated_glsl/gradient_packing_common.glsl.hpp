#pragma once

#include "gradient_packing_common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char gradient_packing_common[] = R"===(#define gc 8.0
#ifdef BB
e f Da(c l0,X R9,c x2,c k8,uint T5,float o4,float n){f P1;P1.xy=B0(R9,l0)+x2;bool n4=(k8.x<0.0);k8.x=max(k8.x,0.0);P1.z=k8.y+(float(T5)/gc)+(k8.x*(a5/gc));P1.z=(n4)?-P1.z:P1.z;P1.w=(n*-0.5-0.25)-round(o4*255.0);return P1;}
#endif
#ifdef EB
e c Te(float T5,c S9,c x3,bool n4){float t=(T5==float(sh))?S9.x:length(S9.xy);t=clamp(t,0.0,1.0);float Hi=n4?(1.0-a5):a5;float x=t*Hi+x3.x;return c(x,x3.y);}e c rg(float T5,c S9,c Ue,bool n4,float hc,float ic){const float jc=0.5*a5;c x3=c(Ue.x*a5+jc,Ue.y*hc+ic);return Te(T5,S9,x3,n4);}e c Sa(f P1,float hc,float ic){const float jc=0.5*a5;bool n4=P1.z<0.0;P1.z=abs(P1.z);c x3;x3.y=floor(P1.z);x3.x=(P1.z-x3.y)*gc+jc;x3.y=x3.y*hc+ic;float T5=floor(x3.x);return Te(T5,P1.xy,x3,n4);}e d ci(f P1){return fract(P1.w)*-2.0+1.5;}e d ke(f P1){const float Ii=-1.0/255.0;return ceil(P1.w)*Ii;}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive