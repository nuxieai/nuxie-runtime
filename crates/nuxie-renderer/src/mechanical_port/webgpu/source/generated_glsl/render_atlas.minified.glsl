#ifdef VERTEX
g1(e0)L(0,g,VB);L(1,g,WB);h1
#endif
m2 H0 X(0,g,O);g2
#ifdef VERTEX
z1(UF,e0,F,B,v){M(B,F,VB,g);M(B,F,WB,g);V(O,g);g W;uint l0;d m0;if(r9(VB,WB,v,l0,m0,O w3)){H O4=J0(QB,l0*4u+2u);R w7=uintBitsToFloat(O4.yzw);m0=m0*w7.x+w7.yz;W=p8(m0,m.td.x,m.td.y);
#ifdef POST_INVERT_Y
W.y=-W.y;
#endif
}else{W=g(m.R2,m.R2,m.R2,m.R2);}c0(O);A1(W);}
#endif
#ifdef FRAGMENT
#ifdef ATLAS_FEATHERED_FILL
e c B6(g P,bool fh I3){c n=e8(P d1);if(!fh)n=-n;return n;}
#endif
#ifdef ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH
layout(location=0)inout H p0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float n=uintBitsToFloat(p0.x);n+=B6(O,gl_FrontFacing d1);p0.x=floatBitsToUint(n);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float n=uintBitsToFloat(p0.x);n=max(n,z4(O));p0.x=floatBitsToUint(n);}
#endif
#elif defined(ATLAS_RENDER_TARGET_R8_PLS_EXT)
__pixel_localEXT S1{layout(r32f)float p0;};
#ifdef ATLAS_FEATHERED_FILL
void main(){p0+=B6(O,gl_FrontFacing d1);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){p0=max(p0,z4(O));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui)uniform highp upixelLocalANGLE p0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);n+=B6(O,gl_FrontFacing d1);pixelLocalStoreANGLE(p0,H(floatBitsToUint(n)));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);n=max(n,z4(O));pixelLocalStoreANGLE(p0,H(floatBitsToUint(n)));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)
layout(binding=0,r32i)uniform highp coherent iimage2D Y8;ivec2 Pd(){return ivec2(floor(a0));}int Qd(float n){return int(n*Tc);}
#ifdef ATLAS_FEATHERED_FILL
void main(){int n=Qd(B6(O,gl_FrontFacing d1));imageAtomicAdd(Y8,Pd(),n);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){int n=Qd(z4(O));imageAtomicMax(Y8,Pd(),n);}
#endif
#elif defined(ATLAS_RENDER_TARGET_RGBA8_UNORM)
#ifdef ATLAS_FEATHERED_FILL
y6(i,WE){r(O,g);c n=B6(O,z6 d1);if(abs(n)>Gf-1e-3){I2(n>.0?C0(.0,.0,1./255.,.0):C0(.0,.0,.0,1./255.));}else{n*=1./va;I2(C0(max(n,.0),max(-n,.0),.0,.0));}}
#endif
#ifdef ATLAS_FEATHERED_STROKE
a3(i,XE){r(O,g);c n=z4(O d1);n*=1./va;I2(C0(n,.0,.0,.0));}
#endif
#else
#ifdef ATLAS_FEATHERED_FILL
y6(float,WE){r(O,g);I2(B6(O,z6 d1));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
a3(float,XE){r(O,g);I2(z4(O d1));}
#endif
#endif
#endif
