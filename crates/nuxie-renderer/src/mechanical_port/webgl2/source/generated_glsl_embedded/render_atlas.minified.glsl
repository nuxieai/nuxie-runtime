#ifdef BB
e1(f0) K(0,f,XB);K(1,f,YB);f1
#endif
v2 F0 W(0,f,S);k2
#ifdef BB
y1(CG,f0,B,F,r){L(F,B,XB,f);L(F,B,YB,f);V(S,f);f I;uint c0;c i0;if(ra(XB,YB,r,c0,i0,S Q3)){O Z3=p0(KB,c0*4u+2u);M d8=uintBitsToFloat(Z3.yzw);i0=i0*d8.x+d8.yz;I=g9(i0,j.Fe.x,j.Fe.y);
#ifdef NC
I.y=-I.y;
#endif
}else{I=f(j.i3,j.i3,j.i3,j.i3);}a0(S);z1(I);}
#endif
#ifdef EB
#ifdef OC
e d c7(f T,bool Wi c4){d n=T8(T n1);if(!Wi) n=-n;return n;}
#endif
#ifdef CE
layout(location=0) inout O w0;
#ifdef OC
void main(){float n=uintBitsToFloat(w0.x);n+=c7(S,gl_FrontFacing n1);w0.x=floatBitsToUint(n);}
#endif
#ifdef TC
void main(){float n=uintBitsToFloat(w0.x);n=max(n,T4(S));w0.x=floatBitsToUint(n);}
#endif
#elif defined(DE)
__pixel_localEXT h2{layout(r32f) float w0;};
#ifdef OC
void main(){w0+=c7(S,gl_FrontFacing n1);}
#endif
#ifdef TC
void main(){w0=max(w0,T4(S));}
#endif
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE w0;
#ifdef OC
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);n+=c7(S,gl_FrontFacing n1);pixelLocalStoreANGLE(w0,O(floatBitsToUint(n)));}
#endif
#ifdef TC
void main(){float n=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);n=max(n,T4(S));pixelLocalStoreANGLE(w0,O(floatBitsToUint(n)));}
#endif
#elif defined(EE)
layout(binding=0,r32i) uniform highp coherent iimage2D Z9;ivec2 af(){return ivec2(floor(d0));}int bf(float n){return int(n*ge);}
#ifdef OC
void main(){int n=bf(c7(S,gl_FrontFacing n1));imageAtomicAdd(Z9,af(),n);}
#endif
#ifdef TC
void main(){int n=bf(T4(S));imageAtomicMax(Z9,af(),n);}
#endif
#elif defined(EF)
#ifdef OC
X6(i,FF){q(S,f);d n=c7(S,Y6 n1);if(abs(n)>oh-1e-3){L2(n>.0?H0(.0,.0,1./255.,.0):H0(.0,.0,.0,1./255.));}else{n*=1./Db;L2(H0(max(n,.0),max(-n,.0),.0,.0));}}
#endif
#ifdef TC
V2(i,GF){q(S,f);d n=T4(S n1);n*=1./Db;L2(H0(n,.0,.0,.0));}
#endif
#else
#ifdef OC
X6(float,FF){q(S,f);L2(c7(S,Y6 n1));}
#endif
#ifdef TC
V2(float,GF){q(S,f);L2(T4(S n1));}
#endif
#endif
#endif
