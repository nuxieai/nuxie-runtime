#ifdef VERTEX
g1(e0)L(0,g,UB);L(1,g,VB);h1
#endif
k2 J0 W(0,g,O);f2
#ifdef VERTEX
y1(RF,e0,F,B,r){M(B,F,UB,g);M(B,F,VB,g);V(O,g);g U;uint o0;d l0;if(q9(UB,VB,r,o0,l0,O v3)){G K4=N0(PB,o0*4u+2u);c0 p7=uintBitsToFloat(K4.yzw);l0=l0*p7.x+p7.yz;U=l8(l0,n.pd.x,n.pd.y);
#ifdef POST_INVERT_Y
U.y=-U.y;
#endif
}else{U=g(n.P2,n.P2,n.P2,n.P2);}a0(O);z1(U);}
#endif
#ifdef FRAGMENT
#ifdef ATLAS_FEATHERED_FILL
e c w6(g P,bool Ug G3){c o=Z7(P d1);if(!Ug)o=-o;return o;}
#endif
#ifdef ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH
layout(location=0)inout G p0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float o=uintBitsToFloat(p0.x);o+=w6(O,gl_FrontFacing d1);p0.x=floatBitsToUint(o);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float o=uintBitsToFloat(p0.x);o=max(o,v4(O));p0.x=floatBitsToUint(o);}
#endif
#elif defined(ATLAS_RENDER_TARGET_R8_PLS_EXT)
__pixel_localEXT R1{layout(r32f)float p0;};
#ifdef ATLAS_FEATHERED_FILL
void main(){p0+=w6(O,gl_FrontFacing d1);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){p0=max(p0,v4(O));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui)uniform highp upixelLocalANGLE p0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);o+=w6(O,gl_FrontFacing d1);pixelLocalStoreANGLE(p0,G(floatBitsToUint(o)));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);o=max(o,v4(O));pixelLocalStoreANGLE(p0,G(floatBitsToUint(o)));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)
layout(binding=0,r32i)uniform highp coherent iimage2D V8;ivec2 Id(){return ivec2(floor(Y));}int Jd(float o){return int(o*Pc);}
#ifdef ATLAS_FEATHERED_FILL
void main(){int o=Jd(w6(O,gl_FrontFacing d1));imageAtomicAdd(V8,Id(),o);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){int o=Jd(v4(O));imageAtomicMax(V8,Id(),o);}
#endif
#elif defined(ATLAS_RENDER_TARGET_RGBA8_UNORM)
#ifdef ATLAS_FEATHERED_FILL
q6(i,UE){A(O,g);c o=w6(O,r6 d1);if(abs(o)>yf-1e-3){G2(o>.0?C0(.0,.0,1./255.,.0):C0(.0,.0,.0,1./255.));}else{o*=1./sa;G2(C0(max(o,.0),max(-o,.0),.0,.0));}}
#endif
#ifdef ATLAS_FEATHERED_STROKE
Y2(i,VE){A(O,g);c o=v4(O d1);o*=1./sa;G2(C0(o,.0,.0,.0));}
#endif
#else
#ifdef ATLAS_FEATHERED_FILL
q6(float,UE){A(O,g);G2(w6(O,r6 d1));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
Y2(float,VE){A(O,g);G2(v4(O d1));}
#endif
#endif
#endif
