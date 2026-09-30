#ifdef CB
A1(ZF,g0,F,B,v){f X;X.x=(B!=2)?-1.:3.;X.y=(B!=1)?-1.:3.;X.zw=c(.0,1.);B1(X);}
#endif
#ifdef EB
e ivec2 fe(){return ivec2(floor(gl_FragCoord));}
#ifdef YD
layout(location=0)inout Y r0;layout(location=1)out i p4;void main(){p4.x=uintBitsToFloat(r0.x);}
#elif defined(ZD)
#ifdef FE
__pixel_local_outEXT U1{layout(r32f)float r0;};
#else
__pixel_local_inEXT U1{layout(r32f)float r0;};layout(location=0)out i p4;
#endif
void main(){
#ifdef FE
r0=.0;
#else
p4.x=r0;
#endif
}
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui)uniform highp upixelLocalANGLE r0;layout(location=0)out i p4;void main(){p4.x=uintBitsToFloat(pixelLocalLoadANGLE(r0).x);}
#elif defined(AE)
layout(binding=0,r32i)uniform highp coherent iimage2D c9;layout(location=0)out i p4;void main(){p4.x=float(imageLoad(c9,fe()).x)*(1./ed);}
#elif defined(ZE)
c3(f3,0,CF);layout(location=0)out i p4;void main(){i N=r1(CF,fe());p4.x=(N.x-N.y)*Ba+(N.z-N.w)*255.;}
#endif
#endif
