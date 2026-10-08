#ifdef VERTEX
f1(f0) K(0,e,XB);K(1,e,YB);g1
#endif
w2 F0 X(0,e,S);l2
#ifdef VERTEX
x1(ZF,f0,B,F,r){L(F,B,XB,e);L(F,B,YB,e);V(S,e);e I;uint c0;c i0;if(ia(XB,YB,r,c0,i0,S P3)){O Z3=p0(KB,c0*4u+2u);M W7=uintBitsToFloat(Z3.yzw);i0=i0*W7.x+W7.yz;I=X8(i0,j.Ae.x,j.Ae.y);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
}else{I=e(j.h3,j.h3,j.h3,j.h3);}Z(S);y1(I);}
#endif
#ifdef FRAGMENT
#ifdef ATLAS_FEATHERED_FILL
f d T6(e T,bool Qi a4){d l=K8(T n1);if(!Qi) l=-l;return l;}
#endif
#ifdef ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH
layout(location=0) inout O w0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float l=uintBitsToFloat(w0.x);l+=T6(S,gl_FrontFacing n1);w0.x=floatBitsToUint(l);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float l=uintBitsToFloat(w0.x);l=max(l,R4(S));w0.x=floatBitsToUint(l);}
#endif
#elif defined(ATLAS_RENDER_TARGET_R8_PLS_EXT)
__pixel_localEXT i2{layout(r32f) float w0;};
#ifdef ATLAS_FEATHERED_FILL
void main(){w0+=T6(S,gl_FrontFacing n1);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){w0=max(w0,R4(S));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE w0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float l=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);l+=T6(S,gl_FrontFacing n1);pixelLocalStoreANGLE(w0,O(floatBitsToUint(l)));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float l=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);l=max(l,R4(S));pixelLocalStoreANGLE(w0,O(floatBitsToUint(l)));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)
layout(binding=0,r32i) uniform highp coherent iimage2D R9;ivec2 Ve(){return ivec2(floor(d0));}int We(float l){return int(l*be);}
#ifdef ATLAS_FEATHERED_FILL
void main(){int l=We(T6(S,gl_FrontFacing n1));imageAtomicAdd(R9,Ve(),l);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){int l=We(R4(S));imageAtomicMax(R9,Ve(),l);}
#endif
#elif defined(ATLAS_RENDER_TARGET_RGBA8_UNORM)
#ifdef ATLAS_FEATHERED_FILL
Q6(i,CF){q(S,e);d l=T6(S,R6 n1);if(abs(l)>ih-1e-3){K2(l>.0?H0(.0,.0,1./255.,.0):H0(.0,.0,.0,1./255.));}else{l*=1./sb;K2(H0(max(l,.0),max(-l,.0),.0,.0));}}
#endif
#ifdef ATLAS_FEATHERED_STROKE
W2(i,DF){q(S,e);d l=R4(S n1);l*=1./sb;K2(H0(l,.0,.0,.0));}
#endif
#else
#ifdef ATLAS_FEATHERED_FILL
Q6(float,CF){q(S,e);K2(T6(S,R6 n1));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
W2(float,DF){q(S,e);K2(R4(S n1));}
#endif
#endif
#endif
