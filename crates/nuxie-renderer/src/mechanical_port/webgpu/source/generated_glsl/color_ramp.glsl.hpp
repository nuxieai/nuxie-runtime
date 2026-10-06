#pragma once

#include "color_ramp.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char color_ramp[] = R"===(#ifdef BB
c1(d0)
#ifdef ua
K(0,uint,TD);K(1,uint,UD);K(2,uint,VD);K(3,uint,WD);
#else
K(0,M,JC);
#endif
d1
#endif
l2 E0 V(0,i,g7);e2
#ifdef BB
k4 l4 Q4 R4 i bg(uint p){return Qc((M(p,p,p,p)>>M(16,8,0,24))&0xffu)/255.;}w1(MF,d0,D,G,r){
#ifdef ua
L(r,D,TD,uint);L(r,D,UD,uint);L(r,D,VD,uint);L(r,D,WD,uint);M JC=M(TD,UD,VD,WD);
#else
L(r,D,JC,M);
#endif
T(g7,i);int D8=G>>1;float x=float(D8<=1?JC.x&0xffffu:JC.x>>16)/65536.;float va=(G&1)==0?.0:1.;if(j.Rc<.0){va=1.-va;}uint h7=JC.y;float y=float(h7&~cg)+va;if((h7&Sc)!=0u&&D8==0){if((h7&wa)!=0u) x=.0;else x-=Tc;}if((h7&Uc)!=0u&&D8==3){if((h7&wa)!=0u) x=1.;else x+=Tc;}g7=bg(D8<=1?JC.z:JC.w);e I=E8(c(x,y),2.,j.Rc);
#ifdef MC
I.y=-I.y;
#endif
Z(g7);x1(I);}
#endif
#ifdef FB
O3 P3 j3(i,NF){q(g7,i);P2(g7);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive