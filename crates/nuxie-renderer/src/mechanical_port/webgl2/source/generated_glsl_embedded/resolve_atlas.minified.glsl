#ifdef BB
r1(AG,d0,D,G,r){e I;I.x=(G!=2)?-1.:3.;I.y=(G!=1)?-1.:3.;I.zw=c(.0,1.);v1(I);}
#endif
#ifdef EB
f ivec2 se(){return ivec2(floor(gl_FragCoord));}
#ifdef ZD
layout(location=0) inout N w0;layout(location=1) out i C4;void main(){C4.x=uintBitsToFloat(w0.x);}
#elif defined(AE)
#ifdef GE
__pixel_local_outEXT Z1{layout(r32f) float w0;};
#else
__pixel_local_inEXT Z1{layout(r32f) float w0;};layout(location=0) out i C4;
#endif
void main(){
#ifdef GE
w0=.0;
#else
C4.x=w0;
#endif
}
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE w0;layout(location=0) out i C4;void main(){C4.x=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);}
#elif defined(BE)
layout(binding=0,r32i) uniform highp coherent iimage2D o9;layout(location=0) out i C4;void main(){C4.x=float(imageLoad(o9,se()).x)*(1./Bd);}
#elif defined(BF)
i3(l3,0,EF);layout(location=0) out i C4;void main(){i U=p1(EF,se());C4.x=(U.x-U.y)*Sa+(U.z-U.w)*255.;}
#endif
#endif
