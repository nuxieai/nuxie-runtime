#ifdef VERTEX
f1(f0)J(0,f,UB);J(1,f,VB);g1
#endif
p2 H0 V(0,f,M);h2
#ifdef VERTEX
y1(YF,f0,F,B,v){K(B,F,UB,f);K(B,F,VB,f);T(M,f);f W;uint m0;c j0;if(v9(UB,VB,v,m0,j0,M w3)){X N4=K0(PB,m0*4u+2u);Q v7=uintBitsToFloat(N4.yzw);j0=j0*v7.x+v7.yz;W=p8(j0,n.Bd.x,n.Bd.y);
#ifdef POST_INVERT_Y
W.y=-W.y;
#endif
}else{W=f(n.T2,n.T2,n.T2,n.T2);}a0(M);z1(W);}
#endif
#ifdef FRAGMENT
#ifdef ATLAS_FEATHERED_FILL
e d A6(f N,bool wh I3){d o=e8(N d1);if(!wh)o=-o;return o;}
#endif
#ifdef ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH
layout(location=0)inout X p0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float o=uintBitsToFloat(p0.x);o+=A6(M,gl_FrontFacing d1);p0.x=floatBitsToUint(o);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float o=uintBitsToFloat(p0.x);o=max(o,y4(M));p0.x=floatBitsToUint(o);}
#endif
#elif defined(ATLAS_RENDER_TARGET_R8_PLS_EXT)
__pixel_localEXT S1{layout(r32f)float p0;};
#ifdef ATLAS_FEATHERED_FILL
void main(){p0+=A6(M,gl_FrontFacing d1);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){p0=max(p0,y4(M));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui)uniform highp upixelLocalANGLE p0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);o+=A6(M,gl_FrontFacing d1);pixelLocalStoreANGLE(p0,X(floatBitsToUint(o)));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);o=max(o,y4(M));pixelLocalStoreANGLE(p0,X(floatBitsToUint(o)));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)
layout(binding=0,r32i)uniform highp coherent iimage2D Z8;ivec2 ae(){return ivec2(floor(c0));}int be(float o){return int(o*bd);}
#ifdef ATLAS_FEATHERED_FILL
void main(){int o=be(A6(M,gl_FrontFacing d1));imageAtomicAdd(Z8,ae(),o);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){int o=be(y4(M));imageAtomicMax(Z8,ae(),o);}
#endif
#elif defined(ATLAS_RENDER_TARGET_RGBA8_UNORM)
#ifdef ATLAS_FEATHERED_FILL
x6(i,AF){r(M,f);d o=A6(M,y6 d1);if(abs(o)>Wf-1e-3){K2(o>.0?D0(.0,.0,1./255.,.0):D0(.0,.0,.0,1./255.));}else{o*=1./za;K2(D0(max(o,.0),max(-o,.0),.0,.0));}}
#endif
#ifdef ATLAS_FEATHERED_STROKE
d3(i,BF){r(M,f);d o=y4(M d1);o*=1./za;K2(D0(o,.0,.0,.0));}
#endif
#else
#ifdef ATLAS_FEATHERED_FILL
x6(float,AF){r(M,f);K2(A6(M,y6 d1));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
d3(float,BF){r(M,f);K2(y4(M d1));}
#endif
#endif
#endif
