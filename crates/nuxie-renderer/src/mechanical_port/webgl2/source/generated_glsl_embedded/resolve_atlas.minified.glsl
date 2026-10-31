#ifdef BB
n1(DG,e0,A,C,r){e I;I.x=(C!=2)?-1.:3.;I.y=(C!=1)?-1.:3.;I.zw=c(.0,1.);A1(I);}
#endif
#ifdef DB
f ivec2 pf(){return ivec2(floor(gl_FragCoord));}
#ifdef CE
layout(location=0) inout R w0;layout(location=1) out i N4;void main(){N4.x=uintBitsToFloat(w0.x);}
#elif defined(DE)
#ifdef LE
__pixel_local_outEXT k2{layout(r32f) float w0;};
#else
__pixel_local_inEXT k2{layout(r32f) float w0;};layout(location=0) out i N4;
#endif
void main(){
#ifdef LE
w0=.0;
#else
N4.x=w0;
#endif
}
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE w0;layout(location=0) out i N4;void main(){N4.x=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);}
#elif defined(EE)
layout(binding=0,r32i) uniform highp coherent iimage2D aa;layout(location=0) out i N4;void main(){N4.x=float(imageLoad(aa,pf()).x)*(1./pe);}
#elif defined(FF)
r3(w3,0,IF);layout(location=0) out i N4;void main(){i T=v1(IF,pf());N4.x=(T.x-T.y)*Hb+(T.z-T.w)*255.;}
#endif
#endif
