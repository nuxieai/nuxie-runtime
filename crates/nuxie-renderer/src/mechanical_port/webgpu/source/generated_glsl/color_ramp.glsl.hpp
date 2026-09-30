#pragma once

#include "color_ramp.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char color_ramp[] = R"===(#ifdef BB
c1(d0)
#ifdef ua
K(0,uint,UD);K(1,uint,VD);K(2,uint,WD);K(3,uint,XD);
#else
K(0,N,KC);
#endif
d1
#endif
l2 E0 W(0,i,h7);e2
#ifdef BB
k4 l4 Q4 R4 i ag(uint l){return Qc((N(l,l,l,l)>>N(16,8,0,24))&0xffu)/255.;}v1(NF,d0,D,G,r){
#ifdef ua
L(r,D,UD,uint);L(r,D,VD,uint);L(r,D,WD,uint);L(r,D,XD,uint);N KC=N(UD,VD,WD,XD);
#else
L(r,D,KC,N);
#endif
T(h7,i);int F8=G>>1;float x=float(F8<=1?KC.x&0xffffu:KC.x>>16)/65536.;float va=(G&1)==0?.0:1.;if(j.Rc<.0){va=1.-va;}uint i7=KC.y;float y=float(i7&~bg)+va;if((i7&Sc)!=0u&&F8==0){if((i7&wa)!=0u) x=.0;else x-=Tc;}if((i7&Uc)!=0u&&F8==3){if((i7&wa)!=0u) x=1.;else x+=Tc;}h7=ag(F8<=1?KC.z:KC.w);f I=G8(c(x,y),2.,j.Rc);
#ifdef NC
I.y=-I.y;
#endif
Z(h7);w1(I);}
#endif
#ifdef EB
O3 P3 j3(i,OF){q(h7,i);Q2(h7);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive