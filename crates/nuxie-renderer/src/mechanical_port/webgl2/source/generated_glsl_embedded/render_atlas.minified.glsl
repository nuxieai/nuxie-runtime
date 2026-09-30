#ifdef CB
h1(g0)K(0,f,UB);K(1,f,VB);i1
#endif
q2 I0 W(0,f,O);i2
#ifdef CB
B1(YF,g0,F,B,v){L(B,F,UB,f);L(B,F,VB,f);U(O,f);f X;uint o0;c l0;if(x9(UB,VB,v,o0,l0,O A3)){Y R4=L0(OB,o0*4u+2u);R y7=uintBitsToFloat(R4.yzw);l0=l0*y7.x+y7.yz;X=r8(l0,j.Jd.x,j.Jd.y);
#ifdef RC
X.y=-X.y;
#endif
}else{X=f(j.W2,j.W2,j.W2,j.W2);}c0(O);C1(X);}
#endif
#ifdef EB
#ifdef MC
e d C6(f P,bool Fh L3){d o=g8(P e1);if(!Fh)o=-o;return o;}
#endif
#ifdef YD
layout(location=0)inout Y r0;
#ifdef MC
void main(){float o=uintBitsToFloat(r0.x);o+=C6(O,gl_FrontFacing e1);r0.x=floatBitsToUint(o);}
#endif
#ifdef TC
void main(){float o=uintBitsToFloat(r0.x);o=max(o,C4(O));r0.x=floatBitsToUint(o);}
#endif
#elif defined(ZD)
__pixel_localEXT V1{layout(r32f)float r0;};
#ifdef MC
void main(){r0+=C6(O,gl_FrontFacing e1);}
#endif
#ifdef TC
void main(){r0=max(r0,C4(O));}
#endif
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui)uniform highp upixelLocalANGLE r0;
#ifdef MC
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(r0).x);o+=C6(O,gl_FrontFacing e1);pixelLocalStoreANGLE(r0,Y(floatBitsToUint(o)));}
#endif
#ifdef TC
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(r0).x);o=max(o,C4(O));pixelLocalStoreANGLE(r0,Y(floatBitsToUint(o)));}
#endif
#elif defined(AE)
layout(binding=0,r32i)uniform highp coherent iimage2D c9;ivec2 ie(){return ivec2(floor(d0));}int je(float o){return int(o*jd);}
#ifdef MC
void main(){int o=je(C6(O,gl_FrontFacing e1));imageAtomicAdd(c9,ie(),o);}
#endif
#ifdef TC
void main(){int o=je(C4(O));imageAtomicMax(c9,ie(),o);}
#endif
#elif defined(ZE)
#ifdef MC
y6(i,AF){r(O,f);d o=C6(O,z6 e1);if(abs(o)>fg-1e-3){M2(o>.0?E0(.0,.0,1./255.,.0):E0(.0,.0,.0,1./255.));}else{o*=1./Ca;M2(E0(max(o,.0),max(-o,.0),.0,.0));}}
#endif
#ifdef TC
f3(i,BF){r(O,f);d o=C4(O e1);o*=1./Ca;M2(E0(o,.0,.0,.0));}
#endif
#else
#ifdef MC
y6(float,AF){r(O,f);M2(C6(O,z6 e1));}
#endif
#ifdef TC
f3(float,BF){r(O,f);M2(C4(O e1));}
#endif
#endif
#endif
