#pragma once

#include "render_atlas.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char render_atlas[] = R"===(#ifdef DB
f1(f0)J(0,f,UB);J(1,f,VB);g1
#endif
p2 H0 V(0,f,M);h2
#ifdef DB
y1(ZF,f0,F,B,v){K(B,F,UB,f);K(B,F,VB,f);T(M,f);f W;uint m0;c j0;if(w9(UB,VB,v,m0,j0,M x3)){X N4=K0(PB,m0*4u+2u);Q w7=uintBitsToFloat(N4.yzw);j0=j0*w7.x+w7.yz;W=p8(j0,l.Dd.x,l.Dd.y);
#ifdef SC
W.y=-W.y;
#endif
}else{W=f(l.T2,l.T2,l.T2,l.T2);}a0(M);z1(W);}
#endif
#ifdef FB
#ifdef NC
e d z6(f N,bool yh I3){d o=e8(N d1);if(!yh)o=-o;return o;}
#endif
#ifdef ZD
layout(location=0)inout X p0;
#ifdef NC
void main(){float o=uintBitsToFloat(p0.x);o+=z6(M,gl_FrontFacing d1);p0.x=floatBitsToUint(o);}
#endif
#ifdef UC
void main(){float o=uintBitsToFloat(p0.x);o=max(o,y4(M));p0.x=floatBitsToUint(o);}
#endif
#elif defined(AE)
__pixel_localEXT S1{layout(r32f)float p0;};
#ifdef NC
void main(){p0+=z6(M,gl_FrontFacing d1);}
#endif
#ifdef UC
void main(){p0=max(p0,y4(M));}
#endif
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui)uniform highp upixelLocalANGLE p0;
#ifdef NC
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);o+=z6(M,gl_FrontFacing d1);pixelLocalStoreANGLE(p0,X(floatBitsToUint(o)));}
#endif
#ifdef UC
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);o=max(o,y4(M));pixelLocalStoreANGLE(p0,X(floatBitsToUint(o)));}
#endif
#elif defined(BE)
layout(binding=0,r32i)uniform highp coherent iimage2D Z8;ivec2 ce(){return ivec2(floor(c0));}int de(float o){return int(o*dd);}
#ifdef NC
void main(){int o=de(z6(M,gl_FrontFacing d1));imageAtomicAdd(Z8,ce(),o);}
#endif
#ifdef UC
void main(){int o=de(y4(M));imageAtomicMax(Z8,ce(),o);}
#endif
#elif defined(AF)
#ifdef NC
v6(i,BF){r(M,f);d o=z6(M,w6 d1);if(abs(o)>Yf-1e-3){K2(o>.0?D0(.0,.0,1./255.,.0):D0(.0,.0,.0,1./255.));}else{o*=1./Aa;K2(D0(max(o,.0),max(-o,.0),.0,.0));}}
#endif
#ifdef UC
c3(i,CF){r(M,f);d o=y4(M d1);o*=1./Aa;K2(D0(o,.0,.0,.0));}
#endif
#else
#ifdef NC
v6(float,BF){r(M,f);K2(z6(M,w6 d1));}
#endif
#ifdef UC
c3(float,CF){r(M,f);K2(y4(M d1));}
#endif
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive