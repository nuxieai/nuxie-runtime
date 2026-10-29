#pragma once

#include "render_atlas.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char render_atlas[] = R"===(#ifdef BB
d1(e0) K(0,e,YB);K(1,e,ZB);e1
#endif
n2 H0 W(0,e,S);M1
#ifdef BB
n1(CG,e0,A,C,r){L(C,A,YB,e);L(C,A,ZB,e);V(S,e);e I;uint g0;c i0;if(sa(YB,ZB,r,g0,i0,S T3)){R fj=n0(JB,g0*4u+3u);M c8=uintBitsToFloat(fj.xyz);i0=i0*c8.x+c8.yz;I=g9(i0,j.Qe.x,j.Qe.y);
#ifdef NC
I.y=-I.y;
#endif
}else{I=e(j.j3,j.j3,j.j3,j.j3);}a0(S);A1(I);}
#endif
#ifdef DB
#ifdef OC
f d Z6(e T,bool gj e4){d n=T8(T m1);if(!gj) n=-n;return n;}
#endif
#ifdef CE
layout(location=0) inout R w0;
#ifdef OC
void main(){float n=uintBitsToFloat(w0.x);n+=Z6(S,gl_FrontFacing m1);w0.x=floatBitsToUint(n);}
#endif
#ifdef TC
void main(){float n=uintBitsToFloat(w0.x);n=max(n,X4(S));w0.x=floatBitsToUint(n);}
#endif
#elif defined(DE)
__pixel_localEXT k2{layout(r32f) float w0;};
#ifdef OC
void main(){w0+=Z6(S,gl_FrontFacing m1);}
#endif
#ifdef TC
void main(){w0=max(w0,X4(S));}
#endif
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE w0;
#ifdef OC
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);n+=Z6(S,gl_FrontFacing m1);pixelLocalStoreANGLE(w0,R(floatBitsToUint(n)));}
#endif
#ifdef TC
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);n=max(n,X4(S));pixelLocalStoreANGLE(w0,R(floatBitsToUint(n)));}
#endif
#elif defined(EE)
layout(binding=0,r32i) uniform highp coherent iimage2D aa;ivec2 mf(){return ivec2(floor(c0));}int nf(float n){return int(n*pe);}
#ifdef OC
void main(){int n=nf(Z6(S,gl_FrontFacing m1));imageAtomicAdd(aa,mf(),n);}
#endif
#ifdef TC
void main(){int n=nf(X4(S));imageAtomicMax(aa,mf(),n);}
#endif
#elif defined(FF)
#ifdef OC
W6(i,GF){q(S,e);d n=Z6(S,X6 m1);if(abs(n)>wh-1e-3){N2(n>.0?E0(.0,.0,1./255.,.0):E0(.0,.0,.0,1./255.));}else{n*=1./Gb;N2(E0(max(n,.0),max(-n,.0),.0,.0));}}
#endif
#ifdef TC
M2(i,HF){q(S,e);d n=X4(S m1);n*=1./Gb;N2(E0(n,.0,.0,.0));}
#endif
#else
#ifdef OC
W6(float,GF){q(S,e);N2(Z6(S,X6 m1));}
#endif
#ifdef TC
M2(float,HF){q(S,e);N2(X4(S m1));}
#endif
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive