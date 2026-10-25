#ifdef VERTEX
e1(e0) K(0,e,YB);K(1,e,ZB);f1
#endif
m2 F0 W(0,e,T);M1
#ifdef VERTEX
o1(DG,e0,A,C,r){L(C,A,YB,e);L(C,A,ZB,e);V(T,e);e I;uint f0;c i0;if(ra(YB,ZB,r,f0,i0,T S3)){O c4=q0(KB,f0*4u+2u);M e8=uintBitsToFloat(c4.yzw);i0=i0*e8.x+e8.yz;I=h9(i0,j.Ie.x,j.Ie.y);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
}else{I=e(j.i3,j.i3,j.i3,j.i3);}a0(T);A1(I);}
#endif
#ifdef FRAGMENT
#ifdef ATLAS_FEATHERED_FILL
f d d7(e U,bool cj e4){d n=U8(U n1);if(!cj) n=-n;return n;}
#endif
#ifdef ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH
layout(location=0) inout O w0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float n=uintBitsToFloat(w0.x);n+=d7(T,gl_FrontFacing n1);w0.x=floatBitsToUint(n);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float n=uintBitsToFloat(w0.x);n=max(n,V4(T));w0.x=floatBitsToUint(n);}
#endif
#elif defined(ATLAS_RENDER_TARGET_R8_PLS_EXT)
__pixel_localEXT j2{layout(r32f) float w0;};
#ifdef ATLAS_FEATHERED_FILL
void main(){w0+=d7(T,gl_FrontFacing n1);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){w0=max(w0,V4(T));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE w0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);n+=d7(T,gl_FrontFacing n1);pixelLocalStoreANGLE(w0,O(floatBitsToUint(n)));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);n=max(n,V4(T));pixelLocalStoreANGLE(w0,O(floatBitsToUint(n)));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)
layout(binding=0,r32i) uniform highp coherent iimage2D Z9;ivec2 df(){return ivec2(floor(c0));}int ef(float n){return int(n*je);}
#ifdef ATLAS_FEATHERED_FILL
void main(){int n=ef(d7(T,gl_FrontFacing n1));imageAtomicAdd(Z9,df(),n);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){int n=ef(V4(T));imageAtomicMax(Z9,df(),n);}
#endif
#elif defined(ATLAS_RENDER_TARGET_RGBA8_UNORM)
#ifdef ATLAS_FEATHERED_FILL
Y6(i,HF){q(T,e);d n=d7(T,Z6 n1);if(abs(n)>rh-1e-3){M2(n>.0?H0(.0,.0,1./255.,.0):H0(.0,.0,.0,1./255.));}else{n*=1./Eb;M2(H0(max(n,.0),max(-n,.0),.0,.0));}}
#endif
#ifdef ATLAS_FEATHERED_STROKE
L2(i,IF){q(T,e);d n=V4(T n1);n*=1./Eb;M2(H0(n,.0,.0,.0));}
#endif
#else
#ifdef ATLAS_FEATHERED_FILL
Y6(float,HF){q(T,e);M2(d7(T,Z6 n1));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
L2(float,IF){q(T,e);M2(V4(T n1));}
#endif
#endif
#endif
