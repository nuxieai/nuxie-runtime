#ifdef VERTEX
g1(e0)L(0,g,VB);L(1,g,WB);h1
#endif
l2 H0 X(0,g,O);f2
#ifdef VERTEX
y1(TF,e0,F,B,v){M(B,F,VB,g);M(B,F,WB,g);V(O,g);g W;uint l0;d m0;if(q9(VB,WB,v,l0,m0,O w3)){G M4=J0(QB,l0*4u+2u);R r7=uintBitsToFloat(M4.yzw);m0=m0*r7.x+r7.yz;W=n8(m0,m.rd.x,m.rd.y);
#ifdef POST_INVERT_Y
W.y=-W.y;
#endif
}else{W=g(m.R2,m.R2,m.R2,m.R2);}a0(O);z1(W);}
#endif
#ifdef FRAGMENT
#ifdef ATLAS_FEATHERED_FILL
e c y6(g P,bool ah I3){c n=c8(P d1);if(!ah)n=-n;return n;}
#endif
#ifdef ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH
layout(location=0)inout G p0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float n=uintBitsToFloat(p0.x);n+=y6(O,gl_FrontFacing d1);p0.x=floatBitsToUint(n);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float n=uintBitsToFloat(p0.x);n=max(n,y4(O));p0.x=floatBitsToUint(n);}
#endif
#elif defined(ATLAS_RENDER_TARGET_R8_PLS_EXT)
__pixel_localEXT R1{layout(r32f)float p0;};
#ifdef ATLAS_FEATHERED_FILL
void main(){p0+=y6(O,gl_FrontFacing d1);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){p0=max(p0,y4(O));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui)uniform highp upixelLocalANGLE p0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);n+=y6(O,gl_FrontFacing d1);pixelLocalStoreANGLE(p0,G(floatBitsToUint(n)));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);n=max(n,y4(O));pixelLocalStoreANGLE(p0,G(floatBitsToUint(n)));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)
layout(binding=0,r32i)uniform highp coherent iimage2D X8;ivec2 Kd(){return ivec2(floor(Z));}int Ld(float n){return int(n*Rc);}
#ifdef ATLAS_FEATHERED_FILL
void main(){int n=Ld(y6(O,gl_FrontFacing d1));imageAtomicAdd(X8,Kd(),n);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){int n=Ld(y4(O));imageAtomicMax(X8,Kd(),n);}
#endif
#elif defined(ATLAS_RENDER_TARGET_RGBA8_UNORM)
#ifdef ATLAS_FEATHERED_FILL
v6(i,WE){r(O,g);c n=y6(O,w6 d1);if(abs(n)>Bf-1e-3){I2(n>.0?C0(.0,.0,1./255.,.0):C0(.0,.0,.0,1./255.));}else{n*=1./sa;I2(C0(max(n,.0),max(-n,.0),.0,.0));}}
#endif
#ifdef ATLAS_FEATHERED_STROKE
a3(i,XE){r(O,g);c n=y4(O d1);n*=1./sa;I2(C0(n,.0,.0,.0));}
#endif
#else
#ifdef ATLAS_FEATHERED_FILL
v6(float,WE){r(O,g);I2(y6(O,w6 d1));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
a3(float,XE){r(O,g);I2(y4(O d1));}
#endif
#endif
#endif
