#ifdef BB
o1(DG,e0,A,C,r){e I;I.x=(C!=2)?-1.:3.;I.y=(C!=1)?-1.:3.;I.zw=c(.0,1.);A1(I);}
#endif
#ifdef EB
f ivec2 ff(){return ivec2(floor(gl_FragCoord));}
#ifdef DE
layout(location=0) inout O w0;layout(location=1) out i L4;void main(){L4.x=uintBitsToFloat(w0.x);}
#elif defined(EE)
#ifdef LE
__pixel_local_outEXT j2{layout(r32f) float w0;};
#else
__pixel_local_inEXT j2{layout(r32f) float w0;};layout(location=0) out i L4;
#endif
void main(){
#ifdef LE
w0=.0;
#else
L4.x=w0;
#endif
}
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE w0;layout(location=0) out i L4;void main(){L4.x=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);}
#elif defined(FE)
layout(binding=0,r32i) uniform highp coherent iimage2D Z9;layout(location=0) out i L4;void main(){L4.x=float(imageLoad(Z9,ff()).x)*(1./je);}
#elif defined(FF)
q3(v3,0,IF);layout(location=0) out i L4;void main(){i U=v1(IF,ff());L4.x=(U.x-U.y)*Eb+(U.z-U.w)*255.;}
#endif
#endif
