#pragma once

#include "color_ramp.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char color_ramp[] = R"===(#ifdef BB
e1(e0)
#ifdef ab
K(0,uint,ZD);K(1,uint,AE);K(2,uint,BE);K(3,uint,CE);
#else
K(0,O,KC);
#endif
f1
#endif
m2 F0 W(0,i,z7);M1
#ifdef BB
v4 w4 a5 c5 i Kg(uint k){return Bd((O(k,k,k,k)>>O(16,8,0,24))&0xffu)/255.;}o1(RF,e0,A,C,r){
#ifdef ab
L(r,A,ZD,uint);L(r,A,AE,uint);L(r,A,BE,uint);L(r,A,CE,uint);O KC=O(ZD,AE,BE,CE);
#else
L(r,A,KC,O);
#endif
V(z7,i);int g9=C>>1;float x=float(g9<=1?KC.x&0xffffu:KC.x>>16)/65536.;float bb=(C&1)==0?.0:1.;if(j.Cd<.0){bb=1.-bb;}uint A7=KC.y;float y=float(A7&~Lg)+bb;if((A7&Dd)!=0u&&g9==0){if((A7&cb)!=0u) x=.0;else x-=d5;}if((A7&Ed)!=0u&&g9==3){if((A7&cb)!=0u) x=1.;else x+=d5;}z7=Kg(g9<=1?KC.z:KC.w);e I=h9(c(x,y),2.,j.Cd);
#ifdef NC
I.y=-I.y;
#endif
a0(z7);A1(I);}
#endif
#ifdef EB
X3 Y3 L2(i,SF){q(z7,i);M2(z7);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive