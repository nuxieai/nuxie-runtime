#pragma once

#include "color_ramp.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char color_ramp[] = R"===(#ifdef CB
h1(g0)
#ifdef ca
K(0,uint,TD);K(1,uint,UD);K(2,uint,VD);K(3,uint,WD);
#else
K(0,Y,IC);
#endif
i1
#endif
q2 I0 W(0,i,Y6);i2
#ifdef CB
Y3 Z3 F4 G4 i Lf(uint k){return xc((Y(k,k,k,k)>>Y(16,8,0,24))&0xffu)/255.;}B1(LF,g0,F,B,v){
#ifdef ca
L(v,F,TD,uint);L(v,F,UD,uint);L(v,F,VD,uint);L(v,F,WD,uint);Y IC=Y(TD,UD,VD,WD);
#else
L(v,F,IC,Y);
#endif
U(Y6,i);int q8=B>>1;float x=float(q8<=1?IC.x&0xffffu:IC.x>>16)/65536.;float da=(B&1)==0?.0:1.;if(j.yc<.0){da=1.-da;}uint Z6=IC.y;float y=float(Z6&~Mf)+da;if((Z6&zc)!=0u&&q8==0){if((Z6&ea)!=0u)x=.0;else x-=Ac;}if((Z6&Bc)!=0u&&q8==3){if((Z6&ea)!=0u)x=1.;else x+=Ac;}Y6=Lf(q8<=1?IC.z:IC.w);f X=r8(c(x,y),2.,j.yc);
#ifdef RC
X.y=-X.y;
#endif
c0(Y6);C1(X);}
#endif
#ifdef EB
I3 J3 f3(i,MF){r(Y6,i);M2(Y6);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive