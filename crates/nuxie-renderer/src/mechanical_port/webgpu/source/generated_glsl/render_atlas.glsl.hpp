#pragma once

#include "render_atlas.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char render_atlas[] = R"===(#ifdef BB
d1(f0) K(0,f,XB);K(1,f,YB);e1
#endif
v2 F0 W(0,f,S);k2
#ifdef BB
w1(AG,f0,B,F,r){L(F,B,XB,f);L(F,B,YB,f);V(S,f);f I;uint c0;c i0;if(pa(XB,YB,r,c0,i0,S Q3)){O Z3=p0(KB,c0*4u+2u);M Z7=uintBitsToFloat(Z3.yzw);i0=i0*Z7.x+Z7.yz;I=d9(i0,j.De.x,j.De.y);
#ifdef MC
I.y=-I.y;
#endif
}else{I=f(j.h3,j.h3,j.h3,j.h3);}Z(S);x1(I);}
#endif
#ifdef EB
#ifdef NC
e d X6(f T,bool Si c4){d n=Q8(T m1);if(!Si) n=-n;return n;}
#endif
#ifdef AE
layout(location=0) inout O w0;
#ifdef NC
void main(){float n=uintBitsToFloat(w0.x);n+=X6(S,gl_FrontFacing m1);w0.x=floatBitsToUint(n);}
#endif
#ifdef RC
void main(){float n=uintBitsToFloat(w0.x);n=max(n,T4(S));w0.x=floatBitsToUint(n);}
#endif
#elif defined(BE)
__pixel_localEXT h2{layout(r32f) float w0;};
#ifdef NC
void main(){w0+=X6(S,gl_FrontFacing m1);}
#endif
#ifdef RC
void main(){w0=max(w0,T4(S));}
#endif
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE w0;
#ifdef NC
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);n+=X6(S,gl_FrontFacing m1);pixelLocalStoreANGLE(w0,O(floatBitsToUint(n)));}
#endif
#ifdef RC
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);n=max(n,T4(S));pixelLocalStoreANGLE(w0,O(floatBitsToUint(n)));}
#endif
#elif defined(CE)
layout(binding=0,r32i) uniform highp coherent iimage2D X9;ivec2 Ye(){return ivec2(floor(d0));}int Ze(float n){return int(n*ee);}
#ifdef NC
void main(){int n=Ze(X6(S,gl_FrontFacing m1));imageAtomicAdd(X9,Ye(),n);}
#endif
#ifdef RC
void main(){int n=Ze(T4(S));imageAtomicMax(X9,Ye(),n);}
#endif
#elif defined(CF)
#ifdef NC
U6(i,DF){q(S,f);d n=X6(S,V6 m1);if(abs(n)>mh-1e-3){K2(n>.0?H0(.0,.0,1./255.,.0):H0(.0,.0,.0,1./255.));}else{n*=1./zb;K2(H0(max(n,.0),max(-n,.0),.0,.0));}}
#endif
#ifdef RC
V2(i,EF){q(S,f);d n=T4(S m1);n*=1./zb;K2(H0(n,.0,.0,.0));}
#endif
#else
#ifdef NC
U6(float,DF){q(S,f);K2(X6(S,V6 m1));}
#endif
#ifdef RC
V2(float,EF){q(S,f);K2(T4(S m1));}
#endif
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive