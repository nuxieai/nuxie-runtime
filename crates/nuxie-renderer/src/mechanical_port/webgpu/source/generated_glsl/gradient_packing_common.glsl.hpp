#pragma once

#include "gradient_packing_common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char gradient_packing_common[] = R"===(#define nc 8.0
#ifdef BB
f e Ga(c l0,X V9,c A2,c m8,uint n5,float r4,float n){e U1;U1.xy=D0(V9,l0)+A2;bool q4=(m8.x<0.0);m8.x=max(m8.x,0.0);U1.z=m8.y+(float(n5)/nc)+(m8.x*(f5/nc));U1.z=(q4)?-U1.z:U1.z;U1.w=(n*-0.5-0.25)-round(r4*255.0);return U1;}
#endif
#ifdef DB
f c gf(float n5,c n8,c c3,bool q4){
#ifdef JE
if(JE&&n5==float(Eh)){float t=ib(n8)*(1.0/y7);return c(t,c3.y);}else
#endif
{float t=(n5==float(Dh))?n8.x:length(n8.xy);t=clamp(t,0.0,1.0);float Si=q4?(1.0-f5):f5;float x=t*Si+c3.x;return c(x,c3.y);}}f c Bg(float n5,c n8,c hf,bool q4,float oc,float pc){const float qc=0.5*f5;c c3=c(hf.x*f5+qc,hf.y*oc+pc);return gf(n5,n8,c3,q4);}f c Xa(e U1,float oc,float pc){const float qc=0.5*f5;bool q4=U1.z<0.0;U1.z=abs(U1.z);c c3;c3.y=floor(U1.z);c3.x=(U1.z-c3.y)*nc+qc;c3.y=c3.y*oc+pc;float n5=floor(c3.x);return gf(n5,U1.xy,c3,q4);}f d ni(e U1){return fract(U1.w)*-2.0+1.5;}f d ve(e U1){const float Ti=-1.0/255.0;return ceil(U1.w)*Ti;}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive