#ifdef BB
v1(AG,d0,D,G,r){f I;I.x=(G!=2)?-1.:3.;I.y=(G!=1)?-1.:3.;I.zw=c(.0,1.);w1(I);}
#endif
#ifdef EB
e ivec2 te(){return ivec2(floor(gl_FragCoord));}
#ifdef ZD
layout(location=0) inout N w0;layout(location=1) out i D4;void main(){D4.x=uintBitsToFloat(w0.x);}
#elif defined(AE)
#ifdef GE
__pixel_local_outEXT a2{layout(r32f) float w0;};
#else
__pixel_local_inEXT a2{layout(r32f) float w0;};layout(location=0) out i D4;
#endif
void main(){
#ifdef GE
w0=.0;
#else
D4.x=w0;
#endif
}
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE w0;layout(location=0) out i D4;void main(){D4.x=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);}
#elif defined(BE)
layout(binding=0,r32i) uniform highp coherent iimage2D q9;layout(location=0) out i D4;void main(){D4.x=float(imageLoad(q9,te()).x)*(1./Cd);}
#elif defined(BF)
i3(l3,0,EF);layout(location=0) out i D4;void main(){i U=p1(EF,te());D4.x=(U.x-U.y)*Ta+(U.z-U.w)*255.;}
#endif
#endif
