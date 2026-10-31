#ifdef VERTEX
f1(d0) K(0,e,YB);K(1,e,ZB);g1
#endif
n2 I0 W(0,e,S);P1
#ifdef VERTEX
l1(EG,d0,A,C,r){L(C,A,YB,e);L(C,A,ZB,e);V(S,e);e I;uint g0;c i0;if(va(YB,ZB,r,g0,i0,S T3)){R qj=o0(JB,g0*4u+3u);M e8=uintBitsToFloat(qj.xyz);i0=i0*e8.x+e8.yz;I=l9(i0,j.Ve.x,j.Ve.y);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
}else{I=e(j.n3,j.n3,j.n3,j.n3);}e0(S);B1(I);}
#endif
#ifdef FRAGMENT
#ifdef ATLAS_FEATHERED_FILL
f d c7(e T,bool rj g4){d n=Y8(T p1);if(!rj) n=-n;return n;}
#endif
#ifdef ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH
layout(location=0) inout R w0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float n=uintBitsToFloat(w0.x);n+=c7(S,gl_FrontFacing p1);w0.x=floatBitsToUint(n);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float n=uintBitsToFloat(w0.x);n=max(n,e5(S));w0.x=floatBitsToUint(n);}
#endif
#elif defined(ATLAS_RENDER_TARGET_R8_PLS_EXT)
__pixel_localEXT V1{layout(r32f) float w0;};
#ifdef ATLAS_FEATHERED_FILL
void main(){w0+=c7(S,gl_FrontFacing p1);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){w0=max(w0,e5(S));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE w0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);n+=c7(S,gl_FrontFacing p1);pixelLocalStoreANGLE(w0,R(floatBitsToUint(n)));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);n=max(n,e5(S));pixelLocalStoreANGLE(w0,R(floatBitsToUint(n)));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)
layout(binding=0,r32i) uniform highp coherent iimage2D da;ivec2 uf(){return ivec2(floor(Y));}int vf(float n){return int(n*te);}
#ifdef ATLAS_FEATHERED_FILL
void main(){int n=vf(c7(S,gl_FrontFacing p1));imageAtomicAdd(da,uf(),n);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){int n=vf(e5(S));imageAtomicMax(da,uf(),n);}
#endif
#elif defined(ATLAS_RENDER_TARGET_RGBA8_UNORM)
#ifdef ATLAS_FEATHERED_FILL
Z6(i,HF){q(S,e);d n=c7(S,a7 p1);if(abs(n)>Fh-1e-3){v2(n>.0?z0(.0,.0,1./255.,.0):z0(.0,.0,.0,1./255.));}else{n*=1./Jb;v2(z0(max(n,.0),max(-n,.0),.0,.0));}}
#endif
#ifdef ATLAS_FEATHERED_STROKE
r2(i,IF){q(S,e);d n=e5(S p1);n*=1./Jb;v2(z0(n,.0,.0,.0));}
#endif
#else
#ifdef ATLAS_FEATHERED_FILL
Z6(float,HF){q(S,e);v2(c7(S,a7 p1));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
r2(float,IF){q(S,e);v2(e5(S p1));}
#endif
#endif
#endif
