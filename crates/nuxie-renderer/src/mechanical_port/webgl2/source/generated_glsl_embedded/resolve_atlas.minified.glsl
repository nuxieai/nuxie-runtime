#ifdef CB
B1(ZF,g0,F,B,v){f X;X.x=(B!=2)?-1.:3.;X.y=(B!=1)?-1.:3.;X.zw=c(.0,1.);C1(X);}
#endif
#ifdef EB
e ivec2 ke(){return ivec2(floor(gl_FragCoord));}
#ifdef YD
layout(location=0)inout Y r0;layout(location=1)out i q4;void main(){q4.x=uintBitsToFloat(r0.x);}
#elif defined(ZD)
#ifdef FE
__pixel_local_outEXT V1{layout(r32f)float r0;};
#else
__pixel_local_inEXT V1{layout(r32f)float r0;};layout(location=0)out i q4;
#endif
void main(){
#ifdef FE
r0=.0;
#else
q4.x=r0;
#endif
}
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui)uniform highp upixelLocalANGLE r0;layout(location=0)out i q4;void main(){q4.x=uintBitsToFloat(pixelLocalLoadANGLE(r0).x);}
#elif defined(AE)
layout(binding=0,r32i)uniform highp coherent iimage2D c9;layout(location=0)out i q4;void main(){q4.x=float(imageLoad(c9,ke()).x)*(1./jd);}
#elif defined(ZE)
e3(h3,0,CF);layout(location=0)out i q4;void main(){i P=v1(CF,ke());q4.x=(P.x-P.y)*Ca+(P.z-P.w)*255.;}
#endif
#endif
