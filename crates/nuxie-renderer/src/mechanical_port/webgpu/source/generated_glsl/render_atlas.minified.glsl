#ifdef VERTEX
h1(g0)K(0,f,UB);K(1,f,VB);i1
#endif
q2 I0 W(0,f,O);i2
#ifdef VERTEX
B1(YF,g0,F,B,v){L(B,F,UB,f);L(B,F,VB,f);U(O,f);f X;uint o0;c l0;if(x9(UB,VB,v,o0,l0,O A3)){Y R4=L0(OB,o0*4u+2u);R y7=uintBitsToFloat(R4.yzw);l0=l0*y7.x+y7.yz;X=r8(l0,j.Id.x,j.Id.y);
#ifdef POST_INVERT_Y
X.y=-X.y;
#endif
}else{X=f(j.W2,j.W2,j.W2,j.W2);}c0(O);C1(X);}
#endif
#ifdef FRAGMENT
#ifdef ATLAS_FEATHERED_FILL
e d C6(f P,bool Eh L3){d o=g8(P e1);if(!Eh)o=-o;return o;}
#endif
#ifdef ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH
layout(location=0)inout Y r0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float o=uintBitsToFloat(r0.x);o+=C6(O,gl_FrontFacing e1);r0.x=floatBitsToUint(o);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float o=uintBitsToFloat(r0.x);o=max(o,C4(O));r0.x=floatBitsToUint(o);}
#endif
#elif defined(ATLAS_RENDER_TARGET_R8_PLS_EXT)
__pixel_localEXT V1{layout(r32f)float r0;};
#ifdef ATLAS_FEATHERED_FILL
void main(){r0+=C6(O,gl_FrontFacing e1);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){r0=max(r0,C4(O));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui)uniform highp upixelLocalANGLE r0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(r0).x);o+=C6(O,gl_FrontFacing e1);pixelLocalStoreANGLE(r0,Y(floatBitsToUint(o)));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(r0).x);o=max(o,C4(O));pixelLocalStoreANGLE(r0,Y(floatBitsToUint(o)));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)
layout(binding=0,r32i)uniform highp coherent iimage2D c9;ivec2 he(){return ivec2(floor(d0));}int ie(float o){return int(o*id);}
#ifdef ATLAS_FEATHERED_FILL
void main(){int o=ie(C6(O,gl_FrontFacing e1));imageAtomicAdd(c9,he(),o);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){int o=ie(C4(O));imageAtomicMax(c9,he(),o);}
#endif
#elif defined(ATLAS_RENDER_TARGET_RGBA8_UNORM)
#ifdef ATLAS_FEATHERED_FILL
y6(i,AF){r(O,f);d o=C6(O,z6 e1);if(abs(o)>eg-1e-3){M2(o>.0?E0(.0,.0,1./255.,.0):E0(.0,.0,.0,1./255.));}else{o*=1./Ca;M2(E0(max(o,.0),max(-o,.0),.0,.0));}}
#endif
#ifdef ATLAS_FEATHERED_STROKE
f3(i,BF){r(O,f);d o=C4(O e1);o*=1./Ca;M2(E0(o,.0,.0,.0));}
#endif
#else
#ifdef ATLAS_FEATHERED_FILL
y6(float,AF){r(O,f);M2(C6(O,z6 e1));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
f3(float,BF){r(O,f);M2(C4(O e1));}
#endif
#endif
#endif
