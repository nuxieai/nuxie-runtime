#pragma once

#include "render_atlas.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char render_atlas[] = R"===(#ifdef BB
e1(e0) K(0,e,YB);K(1,e,ZB);f1
#endif
m2 H0 W(0,e,T);M1
#ifdef BB
o1(DG,e0,A,C,r){L(C,A,YB,e);L(C,A,ZB,e);V(T,e);e I;uint f0;c i0;if(sa(YB,ZB,r,f0,i0,T T3)){O d4=q0(KB,f0*4u+2u);M e8=uintBitsToFloat(d4.yzw);i0=i0*e8.x+e8.yz;I=i9(i0,j.Qe.x,j.Qe.y);
#ifdef NC
I.y=-I.y;
#endif
}else{I=e(j.i3,j.i3,j.i3,j.i3);}a0(T);A1(I);}
#endif
#ifdef DB
#ifdef OC
f d c7(e U,bool gj f4){d n=V8(U n1);if(!gj) n=-n;return n;}
#endif
#ifdef DE
layout(location=0) inout O w0;
#ifdef OC
void main(){float n=uintBitsToFloat(w0.x);n+=c7(T,gl_FrontFacing n1);w0.x=floatBitsToUint(n);}
#endif
#ifdef TC
void main(){float n=uintBitsToFloat(w0.x);n=max(n,X4(T));w0.x=floatBitsToUint(n);}
#endif
#elif defined(EE)
__pixel_localEXT j2{layout(r32f) float w0;};
#ifdef OC
void main(){w0+=c7(T,gl_FrontFacing n1);}
#endif
#ifdef TC
void main(){w0=max(w0,X4(T));}
#endif
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE w0;
#ifdef OC
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);n+=c7(T,gl_FrontFacing n1);pixelLocalStoreANGLE(w0,O(floatBitsToUint(n)));}
#endif
#ifdef TC
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);n=max(n,X4(T));pixelLocalStoreANGLE(w0,O(floatBitsToUint(n)));}
#endif
#elif defined(FE)
layout(binding=0,r32i) uniform highp coherent iimage2D aa;ivec2 mf(){return ivec2(floor(c0));}int nf(float n){return int(n*pe);}
#ifdef OC
void main(){int n=nf(c7(T,gl_FrontFacing n1));imageAtomicAdd(aa,mf(),n);}
#endif
#ifdef TC
void main(){int n=nf(X4(T));imageAtomicMax(aa,mf(),n);}
#endif
#elif defined(GF)
#ifdef OC
Y6(i,HF){q(T,e);d n=c7(T,Z6 n1);if(abs(n)>xh-1e-3){M2(n>.0?E0(.0,.0,1./255.,.0):E0(.0,.0,.0,1./255.));}else{n*=1./Fb;M2(E0(max(n,.0),max(-n,.0),.0,.0));}}
#endif
#ifdef TC
L2(i,IF){q(T,e);d n=X4(T n1);n*=1./Fb;M2(E0(n,.0,.0,.0));}
#endif
#else
#ifdef OC
Y6(float,HF){q(T,e);M2(c7(T,Z6 n1));}
#endif
#ifdef TC
L2(float,IF){q(T,e);M2(X4(T n1));}
#endif
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive