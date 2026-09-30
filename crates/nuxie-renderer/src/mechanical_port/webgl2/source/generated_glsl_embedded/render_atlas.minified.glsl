#ifdef DB
f1(f0)J(0,f,UB);J(1,f,VB);g1
#endif
p2 H0 V(0,f,M);h2
#ifdef DB
y1(ZF,f0,F,B,v){K(B,F,UB,f);K(B,F,VB,f);T(M,f);f W;uint m0;c j0;if(v9(UB,VB,v,m0,j0,M x3)){X N4=K0(PB,m0*4u+2u);Q w7=uintBitsToFloat(N4.yzw);j0=j0*w7.x+w7.yz;W=p8(j0,n.Bd.x,n.Bd.y);
#ifdef SC
W.y=-W.y;
#endif
}else{W=f(n.U2,n.U2,n.U2,n.U2);}a0(M);z1(W);}
#endif
#ifdef FB
#ifdef NC
e d B6(f N,bool wh I3){d o=e8(N d1);if(!wh)o=-o;return o;}
#endif
#ifdef ZD
layout(location=0)inout X p0;
#ifdef NC
void main(){float o=uintBitsToFloat(p0.x);o+=B6(M,gl_FrontFacing d1);p0.x=floatBitsToUint(o);}
#endif
#ifdef UC
void main(){float o=uintBitsToFloat(p0.x);o=max(o,y4(M));p0.x=floatBitsToUint(o);}
#endif
#elif defined(AE)
__pixel_localEXT S1{layout(r32f)float p0;};
#ifdef NC
void main(){p0+=B6(M,gl_FrontFacing d1);}
#endif
#ifdef UC
void main(){p0=max(p0,y4(M));}
#endif
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui)uniform highp upixelLocalANGLE p0;
#ifdef NC
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);o+=B6(M,gl_FrontFacing d1);pixelLocalStoreANGLE(p0,X(floatBitsToUint(o)));}
#endif
#ifdef UC
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);o=max(o,y4(M));pixelLocalStoreANGLE(p0,X(floatBitsToUint(o)));}
#endif
#elif defined(BE)
layout(binding=0,r32i)uniform highp coherent iimage2D Z8;ivec2 ae(){return ivec2(floor(c0));}int be(float o){return int(o*bd);}
#ifdef NC
void main(){int o=be(B6(M,gl_FrontFacing d1));imageAtomicAdd(Z8,ae(),o);}
#endif
#ifdef UC
void main(){int o=be(y4(M));imageAtomicMax(Z8,ae(),o);}
#endif
#elif defined(AF)
#ifdef NC
x6(i,BF){r(M,f);d o=B6(M,y6 d1);if(abs(o)>Wf-1e-3){L2(o>.0?D0(.0,.0,1./255.,.0):D0(.0,.0,.0,1./255.));}else{o*=1./za;L2(D0(max(o,.0),max(-o,.0),.0,.0));}}
#endif
#ifdef UC
d3(i,CF){r(M,f);d o=y4(M d1);o*=1./za;L2(D0(o,.0,.0,.0));}
#endif
#else
#ifdef NC
x6(float,BF){r(M,f);L2(B6(M,y6 d1));}
#endif
#ifdef UC
d3(float,CF){r(M,f);L2(y4(M d1));}
#endif
#endif
#endif
