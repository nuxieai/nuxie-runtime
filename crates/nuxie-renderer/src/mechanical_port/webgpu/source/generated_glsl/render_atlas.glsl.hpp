#pragma once

#include "render_atlas.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char render_atlas[] = R"===(#ifdef CB
h1(h0) I(0,f,VB);I(1,f,WB);i1
#endif
q2 I0 W(0,f,O);i2
#ifdef CB
B1(YF,h0,F,A,q){J(A,F,VB,f);J(A,F,WB,f);V(O,f);f X;uint o0;c l0;if(r9(VB,WB,q,o0,l0,O A3)){R Q4=L0(OB,o0*4u+2u);S v7=uintBitsToFloat(Q4.yzw);l0=l0*v7.x+v7.yz;X=o8(l0,j.Dd.x,j.Dd.y);
#ifdef SC
X.y=-X.y;
#endif
}else{X=f(j.W2,j.W2,j.W2,j.W2);}c0(O);C1(X);}
#endif
#ifdef EB
#ifdef NC
e d z6(f P,bool Dh L3){d o=d8(P e1);if(!Dh) o=-o;return o;}
#endif
#ifdef ZD
layout(location=0) inout R r0;
#ifdef NC
void main(){float o=uintBitsToFloat(r0.x);o+=z6(O,gl_FrontFacing e1);r0.x=floatBitsToUint(o);}
#endif
#ifdef UC
void main(){float o=uintBitsToFloat(r0.x);o=max(o,C4(O));r0.x=floatBitsToUint(o);}
#endif
#elif defined(AE)
__pixel_localEXT V1{layout(r32f) float r0;};
#ifdef NC
void main(){r0+=z6(O,gl_FrontFacing e1);}
#endif
#ifdef UC
void main(){r0=max(r0,C4(O));}
#endif
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE r0;
#ifdef NC
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(r0).x);o+=z6(O,gl_FrontFacing e1);pixelLocalStoreANGLE(r0,R(floatBitsToUint(o)));}
#endif
#ifdef UC
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(r0).x);o=max(o,C4(O));pixelLocalStoreANGLE(r0,R(floatBitsToUint(o)));}
#endif
#elif defined(BE)
layout(binding=0,r32i) uniform highp coherent iimage2D Y8;ivec2 ce(){return ivec2(floor(d0));}int de(float o){return int(o*dd);}
#ifdef NC
void main(){int o=de(z6(O,gl_FrontFacing e1));imageAtomicAdd(Y8,ce(),o);}
#endif
#ifdef UC
void main(){int o=de(C4(O));imageAtomicMax(Y8,ce(),o);}
#endif
#elif defined(AF)
#ifdef NC
v6(i,BF){r(O,f);d o=z6(O,w6 e1);if(abs(o)>cg-1e-3){M2(o>.0?E0(.0,.0,1./255.,.0):E0(.0,.0,.0,1./255.));}else{o*=1./ya;M2(E0(max(o,.0),max(-o,.0),.0,.0));}}
#endif
#ifdef UC
f3(i,CF){r(O,f);d o=C4(O e1);o*=1./ya;M2(E0(o,.0,.0,.0));}
#endif
#else
#ifdef NC
v6(float,BF){r(O,f);M2(z6(O,w6 e1));}
#endif
#ifdef UC
f3(float,CF){r(O,f);M2(C4(O e1));}
#endif
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive