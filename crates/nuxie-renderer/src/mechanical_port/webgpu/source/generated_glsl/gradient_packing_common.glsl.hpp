#pragma once

#include "gradient_packing_common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char gradient_packing_common[] = R"===(#define qc 8.0
#ifdef BB
f e Ja(c m0,X Z9,c C2,c o8,uint w5,float w4,float n){e W1;W1.xy=F0(Z9,m0)+C2;bool v4=(o8.x<0.0);o8.x=max(o8.x,0.0);W1.z=o8.y+(float(w5)/qc)+(o8.x*(k5/qc));W1.z=(v4)?-W1.z:W1.z;W1.w=(n*-0.5-0.25)-round(w4*255.0);return W1;}
#endif
#ifdef EB
f c mf(float w5,c p8,c e3,bool v4){
#ifdef JE
if(JE&&w5==float(Nh)){float t=lb(p8)*(1.0/A7);return c(t,e3.y);}else
#endif
{float t=(w5==float(Mh))?p8.x:length(p8.xy);t=clamp(t,0.0,1.0);float aj=v4?(1.0-k5):k5;float x=t*aj+e3.x;return c(x,e3.y);}}f c Jg(float w5,c p8,c nf,bool v4,float rc,float sc){const float tc=0.5*k5;c e3=c(nf.x*k5+tc,nf.y*rc+sc);return mf(w5,p8,e3,v4);}f c ab(e W1,float rc,float sc){const float tc=0.5*k5;bool v4=W1.z<0.0;W1.z=abs(W1.z);c e3;e3.y=floor(W1.z);e3.x=(W1.z-e3.y)*qc+tc;e3.y=e3.y*rc+sc;float w5=floor(e3.x);return mf(w5,W1.xy,e3,v4);}f d vi(e W1){return fract(W1.w)*-2.0+1.5;}f d ze(e W1){const float bj=-1.0/255.0;return ceil(W1.w)*bj;}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive