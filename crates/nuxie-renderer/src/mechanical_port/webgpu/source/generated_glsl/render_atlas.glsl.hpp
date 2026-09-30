#pragma once

#include "render_atlas.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char render_atlas[] = R"===(#ifdef BB
c1(d0) K(0,e,WB);K(1,e,XB);d1
#endif
l2 E0 W(0,e,S);d2
#ifdef BB
r1(ZF,d0,D,G,r){L(G,D,WB,e);L(G,D,XB,e);T(S,e);e I;uint a0;c k0;if(K9(WB,XB,r,a0,k0,S H3)){N W3=p0(LB,a0*4u+2u);P F7=uintBitsToFloat(W3.yzw);k0=k0*F7.x+F7.yz;I=E8(k0,j.Ud.x,j.Ud.y);
#ifdef NC
I.y=-I.y;
#endif
}else{I=e(j.c3,j.c3,j.c3,j.c3);}Z(S);v1(I);}
#endif
#ifdef EB
#ifdef OC
f d J6(e U,bool Xh S3){d o=p8(U k1);if(!Xh) o=-o;return o;}
#endif
#ifdef ZD
layout(location=0) inout N w0;
#ifdef OC
void main(){float o=uintBitsToFloat(w0.x);o+=J6(S,gl_FrontFacing k1);w0.x=floatBitsToUint(o);}
#endif
#ifdef UC
void main(){float o=uintBitsToFloat(w0.x);o=max(o,M4(S));w0.x=floatBitsToUint(o);}
#endif
#elif defined(AE)
__pixel_localEXT Z1{layout(r32f) float w0;};
#ifdef OC
void main(){w0+=J6(S,gl_FrontFacing k1);}
#endif
#ifdef UC
void main(){w0=max(w0,M4(S));}
#endif
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE w0;
#ifdef OC
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);o+=J6(S,gl_FrontFacing k1);pixelLocalStoreANGLE(w0,N(floatBitsToUint(o)));}
#endif
#ifdef UC
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);o=max(o,M4(S));pixelLocalStoreANGLE(w0,N(floatBitsToUint(o)));}
#endif
#elif defined(BE)
layout(binding=0,r32i) uniform highp coherent iimage2D o9;ivec2 qe(){return ivec2(floor(f0));}int re(float o){return int(o*Bd);}
#ifdef OC
void main(){int o=re(J6(S,gl_FrontFacing k1));imageAtomicAdd(o9,qe(),o);}
#endif
#ifdef UC
void main(){int o=re(M4(S));imageAtomicMax(o9,qe(),o);}
#endif
#elif defined(BF)
#ifdef OC
F6(i,CF){q(S,e);d o=J6(S,G6 k1);if(abs(o)>zg-1e-3){Q2(o>.0?G0(.0,.0,1./255.,.0):G0(.0,.0,.0,1./255.));}else{o*=1./Sa;Q2(G0(max(o,.0),max(-o,.0),.0,.0));}}
#endif
#ifdef UC
j3(i,DF){q(S,e);d o=M4(S k1);o*=1./Sa;Q2(G0(o,.0,.0,.0));}
#endif
#else
#ifdef OC
F6(float,CF){q(S,e);Q2(J6(S,G6 k1));}
#endif
#ifdef UC
j3(float,DF){q(S,e);Q2(M4(S k1));}
#endif
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive