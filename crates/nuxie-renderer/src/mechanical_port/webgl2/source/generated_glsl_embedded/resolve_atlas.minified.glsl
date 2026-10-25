#ifdef BB
o1(EG,e0,A,C,r){e I;I.x=(C!=2)?-1.:3.;I.y=(C!=1)?-1.:3.;I.zw=c(.0,1.);A1(I);}
#endif
#ifdef DB
f ivec2 of(){return ivec2(floor(gl_FragCoord));}
#ifdef DE
layout(location=0) inout O w0;layout(location=1) out i N4;void main(){N4.x=uintBitsToFloat(w0.x);}
#elif defined(EE)
#ifdef ME
__pixel_local_outEXT j2{layout(r32f) float w0;};
#else
__pixel_local_inEXT j2{layout(r32f) float w0;};layout(location=0) out i N4;
#endif
void main(){
#ifdef ME
w0=.0;
#else
N4.x=w0;
#endif
}
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE w0;layout(location=0) out i N4;void main(){N4.x=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);}
#elif defined(FE)
layout(binding=0,r32i) uniform highp coherent iimage2D aa;layout(location=0) out i N4;void main(){N4.x=float(imageLoad(aa,of()).x)*(1./pe);}
#elif defined(GF)
q3(v3,0,JF);layout(location=0) out i N4;void main(){i U=v1(JF,of());N4.x=(U.x-U.y)*Fb+(U.z-U.w)*255.;}
#endif
#endif
