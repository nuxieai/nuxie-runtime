#pragma once

#include "color_ramp.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char color_ramp[] = R"===(#ifdef BB
d1(f0)
#ifdef Xa
K(0,uint,WD);K(1,uint,XD);K(2,uint,YD);K(3,uint,ZD);
#else
K(0,O,JC);
#endif
e1
#endif
v2 F0 W(0,i,w7);k2
#ifdef BB
q4 r4 Y4 Z4 i Fg(uint l){return wd((O(l,l,l,l)>>O(16,8,0,24))&0xffu)/255.;}w1(OF,f0,B,F,r){
#ifdef Xa
L(r,B,WD,uint);L(r,B,XD,uint);L(r,B,YD,uint);L(r,B,ZD,uint);O JC=O(WD,XD,YD,ZD);
#else
L(r,B,JC,O);
#endif
V(w7,i);int d9=F>>1;float x=float(d9<=1?JC.x&0xffffu:JC.x>>16)/65536.;float Ya=(F&1)==0?.0:1.;if(j.xd<.0){Ya=1.-Ya;}uint x7=JC.y;float y=float(x7&~Gg)+Ya;if((x7&yd)!=0u&&d9==0){if((x7&Za)!=0u) x=.0;else x-=a5;}if((x7&zd)!=0u&&d9==3){if((x7&Za)!=0u) x=1.;else x+=a5;}w7=Fg(d9<=1?JC.z:JC.w);f I=e9(c(x,y),2.,j.xd);
#ifdef MC
I.y=-I.y;
#endif
Z(w7);x1(I);}
#endif
#ifdef EB
V3 W3 V2(i,PF){q(w7,i);K2(w7);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive