#pragma once

#include "render_atlas.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char render_atlas[] = R"===(#ifdef DB
g1(e0)O(0,g,UB);O(1,g,VB);h1
#endif
m2 H0 W(0,g,L);g2
#ifdef DB
z1(TF,e0,F,B,A){P(B,F,UB,g);P(B,F,VB,g);U(L,g);g V;uint l0;d m0;if(q9(UB,VB,A,l0,m0,L v3)){X N4=J0(PB,l0*4u+2u);Q v7=uintBitsToFloat(N4.yzw);m0=m0*v7.x+v7.yz;V=o8(m0,m.td.x,m.td.y);
#ifdef RC
V.y=-V.y;
#endif
}else{V=g(m.R2,m.R2,m.R2,m.R2);}c0(L);A1(V);}
#endif
#ifdef GB
#ifdef NC
e c z6(g M,bool eh H3){c n=d8(M d1);if(!eh)n=-n;return n;}
#endif
#ifdef VD
layout(location=0)inout X p0;
#ifdef NC
void main(){float n=uintBitsToFloat(p0.x);n+=z6(L,gl_FrontFacing d1);p0.x=floatBitsToUint(n);}
#endif
#ifdef TC
void main(){float n=uintBitsToFloat(p0.x);n=max(n,y4(L));p0.x=floatBitsToUint(n);}
#endif
#elif defined(WD)
__pixel_localEXT S1{layout(r32f)float p0;};
#ifdef NC
void main(){p0+=z6(L,gl_FrontFacing d1);}
#endif
#ifdef TC
void main(){p0=max(p0,y4(L));}
#endif
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui)uniform highp upixelLocalANGLE p0;
#ifdef NC
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);n+=z6(L,gl_FrontFacing d1);pixelLocalStoreANGLE(p0,X(floatBitsToUint(n)));}
#endif
#ifdef TC
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);n=max(n,y4(L));pixelLocalStoreANGLE(p0,X(floatBitsToUint(n)));}
#endif
#elif defined(XD)
layout(binding=0,r32i)uniform highp coherent iimage2D X8;ivec2 Pd(){return ivec2(floor(a0));}int Qd(float n){return int(n*Tc);}
#ifdef NC
void main(){int n=Qd(z6(L,gl_FrontFacing d1));imageAtomicAdd(X8,Pd(),n);}
#endif
#ifdef TC
void main(){int n=Qd(y4(L));imageAtomicMax(X8,Pd(),n);}
#endif
#elif defined(UE)
#ifdef NC
w6(i,VE){r(L,g);c n=z6(L,x6 d1);if(abs(n)>Gf-1e-3){I2(n>.0?C0(.0,.0,1./255.,.0):C0(.0,.0,.0,1./255.));}else{n*=1./va;I2(C0(max(n,.0),max(-n,.0),.0,.0));}}
#endif
#ifdef TC
a3(i,WE){r(L,g);c n=y4(L d1);n*=1./va;I2(C0(n,.0,.0,.0));}
#endif
#else
#ifdef NC
w6(float,VE){r(L,g);I2(z6(L,x6 d1));}
#endif
#ifdef TC
a3(float,WE){r(L,g);I2(y4(L d1));}
#endif
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive