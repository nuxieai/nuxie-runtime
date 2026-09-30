#ifdef VERTEX
h1(g0)J(0,f,TB);J(1,f,UB);i1
#endif
q2 I0 W(0,f,M);i2
#ifdef VERTEX
A1(YF,g0,F,B,v){K(B,F,TB,f);K(B,F,UB,f);U(M,f);f X;uint n0;c k0;if(x9(TB,UB,v,n0,k0,M A3)){Y Q4=L0(OB,n0*4u+2u);R x7=uintBitsToFloat(Q4.yzw);k0=k0*x7.x+x7.yz;X=r8(k0,j.Ed.x,j.Ed.y);
#ifdef POST_INVERT_Y
X.y=-X.y;
#endif
}else{X=f(j.U2,j.U2,j.U2,j.U2);}c0(M);B1(X);}
#endif
#ifdef FRAGMENT
#ifdef ATLAS_FEATHERED_FILL
e d B6(f N,bool zh L3){d o=g8(N e1);if(!zh)o=-o;return o;}
#endif
#ifdef ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH
layout(location=0)inout Y r0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float o=uintBitsToFloat(r0.x);o+=B6(M,gl_FrontFacing e1);r0.x=floatBitsToUint(o);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float o=uintBitsToFloat(r0.x);o=max(o,B4(M));r0.x=floatBitsToUint(o);}
#endif
#elif defined(ATLAS_RENDER_TARGET_R8_PLS_EXT)
__pixel_localEXT U1{layout(r32f)float r0;};
#ifdef ATLAS_FEATHERED_FILL
void main(){r0+=B6(M,gl_FrontFacing e1);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){r0=max(r0,B4(M));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui)uniform highp upixelLocalANGLE r0;
#ifdef ATLAS_FEATHERED_FILL
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(r0).x);o+=B6(M,gl_FrontFacing e1);pixelLocalStoreANGLE(r0,Y(floatBitsToUint(o)));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){float o=uintBitsToFloat(pixelLocalLoadANGLE(r0).x);o=max(o,B4(M));pixelLocalStoreANGLE(r0,Y(floatBitsToUint(o)));}
#endif
#elif defined(ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)
layout(binding=0,r32i)uniform highp coherent iimage2D c9;ivec2 de(){return ivec2(floor(d0));}int ee(float o){return int(o*ed);}
#ifdef ATLAS_FEATHERED_FILL
void main(){int o=ee(B6(M,gl_FrontFacing e1));imageAtomicAdd(c9,de(),o);}
#endif
#ifdef ATLAS_FEATHERED_STROKE
void main(){int o=ee(B4(M));imageAtomicMax(c9,de(),o);}
#endif
#elif defined(ATLAS_RENDER_TARGET_RGBA8_UNORM)
#ifdef ATLAS_FEATHERED_FILL
x6(i,AF){r(M,f);d o=B6(M,y6 e1);if(abs(o)>Zf-1e-3){L2(o>.0?E0(.0,.0,1./255.,.0):E0(.0,.0,.0,1./255.));}else{o*=1./Ba;L2(E0(max(o,.0),max(-o,.0),.0,.0));}}
#endif
#ifdef ATLAS_FEATHERED_STROKE
d3(i,BF){r(M,f);d o=B4(M e1);o*=1./Ba;L2(E0(o,.0,.0,.0));}
#endif
#else
#ifdef ATLAS_FEATHERED_FILL
x6(float,AF){r(M,f);L2(B6(M,y6 e1));}
#endif
#ifdef ATLAS_FEATHERED_STROKE
d3(float,BF){r(M,f);L2(B4(M e1));}
#endif
#endif
#endif
