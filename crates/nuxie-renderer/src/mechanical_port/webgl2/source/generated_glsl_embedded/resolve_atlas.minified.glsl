#ifdef DB
y1(ZF,f0,F,B,v){f W;W.x=(B!=2)?-1.:3.;W.y=(B!=1)?-1.:3.;W.zw=c(.0,1.);z1(W);}
#endif
#ifdef FB
e ivec2 ce(){return ivec2(floor(gl_FragCoord));}
#ifdef ZD
layout(location=0)inout X p0;layout(location=1)out i n4;void main(){n4.x=uintBitsToFloat(p0.x);}
#elif defined(AE)
#ifdef GE
__pixel_local_outEXT S1{layout(r32f)float p0;};
#else
__pixel_local_inEXT S1{layout(r32f)float p0;};layout(location=0)out i n4;
#endif
void main(){
#ifdef GE
p0=.0;
#else
n4.x=p0;
#endif
}
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui)uniform highp upixelLocalANGLE p0;layout(location=0)out i n4;void main(){n4.x=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);}
#elif defined(BE)
layout(binding=0,r32i)uniform highp coherent iimage2D Z8;layout(location=0)out i n4;void main(){n4.x=float(imageLoad(Z8,ce()).x)*(1./bd);}
#elif defined(ZE)
c3(e3,0,CF);layout(location=0)out i n4;void main(){i N=p1(CF,ce());n4.x=(N.x-N.y)*za+(N.z-N.w)*255.;}
#endif
#endif
