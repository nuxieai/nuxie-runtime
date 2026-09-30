#ifdef BB
c1(d0) K(0,f,WB);K(1,f,XB);d1
#endif
l2 E0 W(0,f,S);e2
#ifdef BB
v1(ZF,d0,D,G,r){L(G,D,WB,f);L(G,D,XB,f);T(S,f);f I;uint a0;c k0;if(L9(WB,XB,r,a0,k0,S H3)){N W3=p0(LB,a0*4u+2u);P G7=uintBitsToFloat(W3.yzw);k0=k0*G7.x+G7.yz;I=G8(k0,j.Vd.x,j.Vd.y);
#ifdef NC
I.y=-I.y;
#endif
}else{I=f(j.c3,j.c3,j.c3,j.c3);}Z(S);w1(I);}
#endif
#ifdef EB
#ifdef OC
e d L6(f U,bool hi S3){d o=r8(U k1);if(!hi) o=-o;return o;}
#endif
#ifdef ZD
layout(location=0) inout N w0;
#ifdef OC
void main(){float o=uintBitsToFloat(w0.x);o+=L6(S,gl_FrontFacing k1);w0.x=floatBitsToUint(o);}
#endif
#ifdef UC
void main(){float o=uintBitsToFloat(w0.x);o=max(o,N4(S));w0.x=floatBitsToUint(o);}
#endif
#elif defined(AE)
__pixel_localEXT a2{layout(r32f) float w0;};
#ifdef OC
void main(){w0+=L6(S,gl_FrontFacing k1);}
#endif
#ifdef UC
void main(){w0=max(w0,N4(S));}
#endif
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE w0;
#ifdef OC
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);o+=L6(S,gl_FrontFacing k1);pixelLocalStoreANGLE(w0,N(floatBitsToUint(o)));}
#endif
#ifdef UC
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);o=max(o,N4(S));pixelLocalStoreANGLE(w0,N(floatBitsToUint(o)));}
#endif
#elif defined(BE)
layout(binding=0,r32i) uniform highp coherent iimage2D q9;ivec2 re(){return ivec2(floor(f0));}int se(float o){return int(o*Cd);}
#ifdef OC
void main(){int o=se(L6(S,gl_FrontFacing k1));imageAtomicAdd(q9,re(),o);}
#endif
#ifdef UC
void main(){int o=se(N4(S));imageAtomicMax(q9,re(),o);}
#endif
#elif defined(BF)
#ifdef OC
H6(i,CF){q(S,f);d o=L6(S,I6 k1);if(abs(o)>Dg-1e-3){Q2(o>.0?G0(.0,.0,1./255.,.0):G0(.0,.0,.0,1./255.));}else{o*=1./Ta;Q2(G0(max(o,.0),max(-o,.0),.0,.0));}}
#endif
#ifdef UC
j3(i,DF){q(S,f);d o=N4(S k1);o*=1./Ta;Q2(G0(o,.0,.0,.0));}
#endif
#else
#ifdef OC
H6(float,CF){q(S,f);Q2(L6(S,I6 k1));}
#endif
#ifdef UC
j3(float,DF){q(S,f);Q2(N4(S k1));}
#endif
#endif
#endif
