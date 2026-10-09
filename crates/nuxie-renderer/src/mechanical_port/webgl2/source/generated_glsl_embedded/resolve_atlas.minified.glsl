#ifdef BB
w1(BG,f0,B,F,r){f I;I.x=(F!=2)?-1.:3.;I.y=(F!=1)?-1.:3.;I.zw=c(.0,1.);x1(I);}
#endif
#ifdef EB
e ivec2 af(){return ivec2(floor(gl_FragCoord));}
#ifdef AE
layout(location=0) inout O w0;layout(location=1) out i J4;void main(){J4.x=uintBitsToFloat(w0.x);}
#elif defined(BE)
#ifdef HE
__pixel_local_outEXT h2{layout(r32f) float w0;};
#else
__pixel_local_inEXT h2{layout(r32f) float w0;};layout(location=0) out i J4;
#endif
void main(){
#ifdef HE
w0=.0;
#else
J4.x=w0;
#endif
}
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE w0;layout(location=0) out i J4;void main(){J4.x=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);}
#elif defined(CE)
layout(binding=0,r32i) uniform highp coherent iimage2D X9;layout(location=0) out i J4;void main(){J4.x=float(imageLoad(X9,af()).x)*(1./ee);}
#elif defined(CF)
p3(q3,0,FF);layout(location=0) out i J4;void main(){i T=q1(FF,af());J4.x=(T.x-T.y)*zb+(T.z-T.w)*255.;}
#endif
#endif
