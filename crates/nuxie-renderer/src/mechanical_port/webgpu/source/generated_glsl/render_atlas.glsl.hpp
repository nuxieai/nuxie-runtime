#pragma once

#include "render_atlas.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char render_atlas[] = R"===(#ifdef CB
g1(h0) I(0,f,VB);I(1,f,WB);h1
#endif
r2 I0 W(0,f,O);i2
#ifdef CB
A1(YF,h0,F,A,q){J(A,F,VB,f);J(A,F,WB,f);V(O,f);f X;uint o0;c k0;if(r9(VB,WB,q,o0,k0,O A3)){R R4=L0(OB,o0*4u+2u);S r7=uintBitsToFloat(R4.yzw);k0=k0*r7.x+r7.yz;X=o8(k0,j.Cd.x,j.Cd.y);
#ifdef SC
X.y=-X.y;
#endif
}else{X=f(j.X2,j.X2,j.X2,j.X2);}c0(O);B1(X);}
#endif
#ifdef EB
#ifdef NC
e d A6(f P,bool Ch K3){d o=c8(P e1);if(!Ch) o=-o;return o;}
#endif
#ifdef ZD
layout(location=0) inout R r0;
#ifdef NC
void main(){float o=uintBitsToFloat(r0.x);o+=A6(O,gl_FrontFacing e1);r0.x=floatBitsToUint(o);}
#endif
#ifdef UC
void main(){float o=uintBitsToFloat(r0.x);o=max(o,D4(O));r0.x=floatBitsToUint(o);}
#endif
#elif defined(AE)
__pixel_localEXT V1{layout(r32f) float r0;};
#ifdef NC
void main(){r0+=A6(O,gl_FrontFacing e1);}
#endif
#ifdef UC
void main(){r0=max(r0,D4(O));}
#endif
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE r0;
#ifdef NC
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(r0).x);o+=A6(O,gl_FrontFacing e1);pixelLocalStoreANGLE(r0,R(floatBitsToUint(o)));}
#endif
#ifdef UC
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(r0).x);o=max(o,D4(O));pixelLocalStoreANGLE(r0,R(floatBitsToUint(o)));}
#endif
#elif defined(BE)
layout(binding=0,r32i) uniform highp coherent iimage2D Y8;ivec2 be(){return ivec2(floor(e0));}int ce(float o){return int(o*dd);}
#ifdef NC
void main(){int o=ce(A6(O,gl_FrontFacing e1));imageAtomicAdd(Y8,be(),o);}
#endif
#ifdef UC
void main(){int o=ce(D4(O));imageAtomicMax(Y8,be(),o);}
#endif
#elif defined(AF)
#ifdef NC
w6(i,BF){r(O,f);d o=A6(O,x6 e1);if(abs(o)>cg-1e-3){N2(o>.0?E0(.0,.0,1./255.,.0):E0(.0,.0,.0,1./255.));}else{o*=1./wa;N2(E0(max(o,.0),max(-o,.0),.0,.0));}}
#endif
#ifdef UC
f3(i,CF){r(O,f);d o=D4(O e1);o*=1./wa;N2(E0(o,.0,.0,.0));}
#endif
#else
#ifdef NC
w6(float,BF){r(O,f);N2(A6(O,x6 e1));}
#endif
#ifdef UC
f3(float,CF){r(O,f);N2(D4(O e1));}
#endif
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive