#pragma once

#include "color_ramp.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char color_ramp[] = R"===(#ifdef BB
e1(e0)
#ifdef bb
K(0,uint,ZD);K(1,uint,AE);K(2,uint,BE);K(3,uint,CE);
#else
K(0,O,KC);
#endif
f1
#endif
m2 H0 W(0,i,z7);M1
#ifdef BB
x4 y4 c5 d5 i Qg(uint k){return Ed((O(k,k,k,k)>>O(16,8,0,24))&0xffu)/255.;}o1(RF,e0,A,C,r){
#ifdef bb
L(r,A,ZD,uint);L(r,A,AE,uint);L(r,A,BE,uint);L(r,A,CE,uint);O KC=O(ZD,AE,BE,CE);
#else
L(r,A,KC,O);
#endif
V(z7,i);int h9=C>>1;float x=float(h9<=1?KC.x&0xffffu:KC.x>>16)/65536.;float cb=(C&1)==0?.0:1.;if(j.Fd<.0){cb=1.-cb;}uint A7=KC.y;float y=float(A7&~Rg)+cb;if((A7&Gd)!=0u&&h9==0){if((A7&db)!=0u) x=.0;else x-=f5;}if((A7&Hd)!=0u&&h9==3){if((A7&db)!=0u) x=1.;else x+=f5;}z7=Qg(h9<=1?KC.z:KC.w);e I=i9(c(x,y),2.,j.Fd);
#ifdef NC
I.y=-I.y;
#endif
a0(z7);A1(I);}
#endif
#ifdef DB
Y3 Z3 L2(i,SF){q(z7,i);M2(z7);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive