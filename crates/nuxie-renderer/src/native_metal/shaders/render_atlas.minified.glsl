#ifdef VERTEX
c1(d0) K(0,e,WB);K(1,e,XB);d1
#endif
l2 E0 V(0,e,S);e2
#ifdef VERTEX
w1(YF,d0,D,G,r){L(G,D,WB,e);L(G,D,XB,e);T(S,e);e I;uint a0;c k0;if(L9(WB,XB,r,a0,k0,S H3)){M W3=p0(LB,a0*4u+2u);O E7=uintBitsToFloat(W3.yzw);k0=k0*E7.x+E7.yz;I=E8(k0,j.Vd.x,j.Vd.y);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
}else{I=e(j.a3,j.a3,j.a3,j.a3);}Z(S);x1(I);}
#endif
#ifdef FRAGMENT
#ifdef ATLAS_FEATHERED_FILL
f d K6(e U,bool ji S3){d n=p8(U k1);if(!ji) n=-n;return n;}
#endif
#ifdef ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH
layout(location=0) inout M w0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float n=uintBitsToFloat(w0.x);n+=K6(S,gl_FrontFacing k1);w0.x=floatBitsToUint(n);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float n=uintBitsToFloat(w0.x);n=max(n,N4(S));w0.x=floatBitsToUint(n);}
#endif
#elif defined(ATLAS_RENDER_TARGET_R8_PLS_EXT)
__pixel_localEXT c2{layout(r32f) float w0;};
#ifdef ATLAS_FEATHERED_FILL
void main(){w0+=K6(S,gl_FrontFacing k1);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){w0=max(w0,N4(S));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE w0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);n+=K6(S,gl_FrontFacing k1);pixelLocalStoreANGLE(w0,M(floatBitsToUint(n)));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);n=max(n,N4(S));pixelLocalStoreANGLE(w0,M(floatBitsToUint(n)));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)
layout(binding=0,r32i) uniform highp coherent iimage2D p9;ivec2 se(){return ivec2(floor(f0));}int te(float n){return int(n*Cd);}
#ifdef ATLAS_FEATHERED_FILL
void main(){int n=te(K6(S,gl_FrontFacing k1));imageAtomicAdd(p9,se(),n);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){int n=te(N4(S));imageAtomicMax(p9,se(),n);}
#endif
#elif defined(ATLAS_RENDER_TARGET_RGBA8_UNORM)
#ifdef ATLAS_FEATHERED_FILL
G6(i,BF){q(S,e);d n=K6(S,H6 k1);if(abs(n)>Fg-1e-3){P2(n>.0?I0(.0,.0,1./255.,.0):I0(.0,.0,.0,1./255.));}else{n*=1./Ta;P2(I0(max(n,.0),max(-n,.0),.0,.0));}}
#endif
#ifdef ATLAS_FEATHERED_STROKE
j3(i,CF){q(S,e);d n=N4(S k1);n*=1./Ta;P2(I0(n,.0,.0,.0));}
#endif
#else
#ifdef ATLAS_FEATHERED_FILL
G6(float,BF){q(S,e);P2(K6(S,H6 k1));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
j3(float,CF){q(S,e);P2(N4(S k1));}
#endif
#endif
#endif
