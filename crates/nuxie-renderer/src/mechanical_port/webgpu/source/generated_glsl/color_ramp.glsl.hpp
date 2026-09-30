#pragma once

#include "color_ramp.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char color_ramp[] = R"===(#ifdef CB
h1(g0)
#ifdef ba
J(0,uint,TD);J(1,uint,UD);J(2,uint,VD);J(3,uint,WD);
#else
J(0,Y,IC);
#endif
i1
#endif
q2 I0 W(0,i,X6);i2
#ifdef CB
X3 Y3 E4 F4 i Ff(uint k){return sc((Y(k,k,k,k)>>Y(16,8,0,24))&0xffu)/255.;}A1(LF,g0,F,B,v){
#ifdef ba
K(v,F,TD,uint);K(v,F,UD,uint);K(v,F,VD,uint);K(v,F,WD,uint);Y IC=Y(TD,UD,VD,WD);
#else
K(v,F,IC,Y);
#endif
U(X6,i);int q8=B>>1;float x=float(q8<=1?IC.x&0xffffu:IC.x>>16)/65536.;float ca=(B&1)==0?.0:1.;if(j.tc<.0){ca=1.-ca;}uint Y6=IC.y;float y=float(Y6&~Gf)+ca;if((Y6&uc)!=0u&&q8==0){if((Y6&da)!=0u)x=.0;else x-=vc;}if((Y6&wc)!=0u&&q8==3){if((Y6&da)!=0u)x=1.;else x+=vc;}X6=Ff(q8<=1?IC.z:IC.w);f X=r8(c(x,y),2.,j.tc);
#ifdef RC
X.y=-X.y;
#endif
c0(X6);B1(X);}
#endif
#ifdef EB
I3 J3 d3(i,MF){r(X6,i);L2(X6);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive