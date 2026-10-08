#pragma once

#include "render_atlas.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char render_atlas[] = R"===(#ifdef BB
f1(f0) K(0,e,XB);K(1,e,YB);g1
#endif
w2 F0 X(0,e,S);l2
#ifdef BB
x1(ZF,f0,B,F,r){L(F,B,XB,e);L(F,B,YB,e);V(S,e);e I;uint c0;c i0;if(ia(XB,YB,r,c0,i0,S P3)){O Z3=p0(KB,c0*4u+2u);M W7=uintBitsToFloat(Z3.yzw);i0=i0*W7.x+W7.yz;I=X8(i0,j.Ae.x,j.Ae.y);
#ifdef MC
I.y=-I.y;
#endif
}else{I=e(j.h3,j.h3,j.h3,j.h3);}Z(S);y1(I);}
#endif
#ifdef EB
#ifdef NC
f d T6(e T,bool Qi a4){d l=K8(T n1);if(!Qi) l=-l;return l;}
#endif
#ifdef ZD
layout(location=0) inout O w0;
#ifdef NC
void main(){float l=uintBitsToFloat(w0.x);l+=T6(S,gl_FrontFacing n1);w0.x=floatBitsToUint(l);}
#endif
#ifdef SC
void main(){float l=uintBitsToFloat(w0.x);l=max(l,R4(S));w0.x=floatBitsToUint(l);}
#endif
#elif defined(AE)
__pixel_localEXT i2{layout(r32f) float w0;};
#ifdef NC
void main(){w0+=T6(S,gl_FrontFacing n1);}
#endif
#ifdef SC
void main(){w0=max(w0,R4(S));}
#endif
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE w0;
#ifdef NC
void main(){float l=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);l+=T6(S,gl_FrontFacing n1);pixelLocalStoreANGLE(w0,O(floatBitsToUint(l)));}
#endif
#ifdef SC
void main(){float l=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);l=max(l,R4(S));pixelLocalStoreANGLE(w0,O(floatBitsToUint(l)));}
#endif
#elif defined(BE)
layout(binding=0,r32i) uniform highp coherent iimage2D R9;ivec2 Ve(){return ivec2(floor(d0));}int We(float l){return int(l*be);}
#ifdef NC
void main(){int l=We(T6(S,gl_FrontFacing n1));imageAtomicAdd(R9,Ve(),l);}
#endif
#ifdef SC
void main(){int l=We(R4(S));imageAtomicMax(R9,Ve(),l);}
#endif
#elif defined(BF)
#ifdef NC
Q6(i,CF){q(S,e);d l=T6(S,R6 n1);if(abs(l)>ih-1e-3){K2(l>.0?H0(.0,.0,1./255.,.0):H0(.0,.0,.0,1./255.));}else{l*=1./sb;K2(H0(max(l,.0),max(-l,.0),.0,.0));}}
#endif
#ifdef SC
W2(i,DF){q(S,e);d l=R4(S n1);l*=1./sb;K2(H0(l,.0,.0,.0));}
#endif
#else
#ifdef NC
Q6(float,CF){q(S,e);K2(T6(S,R6 n1));}
#endif
#ifdef SC
W2(float,DF){q(S,e);K2(R4(S n1));}
#endif
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive