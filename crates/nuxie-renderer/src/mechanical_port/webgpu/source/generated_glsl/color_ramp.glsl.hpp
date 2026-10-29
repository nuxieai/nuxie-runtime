#pragma once

#include "color_ramp.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char color_ramp[] = R"===(#ifdef BB
d1(e0)
#ifdef cb
K(0,uint,YD);K(1,uint,ZD);K(2,uint,AE);K(3,uint,BE);
#else
K(0,R,KC);
#endif
e1
#endif
n2 H0 W(0,i,w7);M1
#ifdef BB
x4 y4 c5 d5 i Pg(uint k){return Fd((R(k,k,k,k)>>R(16,8,0,24))&0xffu)/255.;}n1(QF,e0,A,C,r){
#ifdef cb
L(r,A,YD,uint);L(r,A,ZD,uint);L(r,A,AE,uint);L(r,A,BE,uint);R KC=R(YD,ZD,AE,BE);
#else
L(r,A,KC,R);
#endif
V(w7,i);int f9=C>>1;float x=float(f9<=1?KC.x&0xffffu:KC.x>>16)/65536.;float db=(C&1)==0?.0:1.;if(j.Gd<.0){db=1.-db;}uint x7=KC.y;float y=float(x7&~Qg)+db;if((x7&Hd)!=0u&&f9==0){if((x7&eb)!=0u) x=.0;else x-=f5;}if((x7&Id)!=0u&&f9==3){if((x7&eb)!=0u) x=1.;else x+=f5;}w7=Pg(f9<=1?KC.z:KC.w);e I=g9(c(x,y),2.,j.Gd);
#ifdef NC
I.y=-I.y;
#endif
a0(w7);A1(I);}
#endif
#ifdef DB
Y3 Z3 M2(i,RF){q(w7,i);N2(w7);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive