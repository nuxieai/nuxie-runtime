#pragma once

#include "color_ramp.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char color_ramp[] = R"===(#ifdef BB
e1(f0)
#ifdef ab
K(0,uint,YD);K(1,uint,ZD);K(2,uint,AE);K(3,uint,BE);
#else
K(0,O,JC);
#endif
f1
#endif
v2 F0 W(0,i,y7);k2
#ifdef BB
q4 r4 Y4 Z4 i Hg(uint k){return xd((O(k,k,k,k)>>O(16,8,0,24))&0xffu)/255.;}y1(QF,f0,B,F,r){
#ifdef ab
L(r,B,YD,uint);L(r,B,ZD,uint);L(r,B,AE,uint);L(r,B,BE,uint);O JC=O(YD,ZD,AE,BE);
#else
L(r,B,JC,O);
#endif
V(y7,i);int f9=F>>1;float x=float(f9<=1?JC.x&0xffffu:JC.x>>16)/65536.;float bb=(F&1)==0?.0:1.;if(j.yd<.0){bb=1.-bb;}uint z7=JC.y;float y=float(z7&~Ig)+bb;if((z7&zd)!=0u&&f9==0){if((z7&cb)!=0u) x=.0;else x-=a5;}if((z7&Ad)!=0u&&f9==3){if((z7&cb)!=0u) x=1.;else x+=a5;}y7=Hg(f9<=1?JC.z:JC.w);f I=g9(c(x,y),2.,j.yd);
#ifdef NC
I.y=-I.y;
#endif
Z(y7);z1(I);}
#endif
#ifdef EB
V3 W3 V2(i,RF){q(y7,i);L2(y7);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive