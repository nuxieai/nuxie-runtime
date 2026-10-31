#ifdef BB
l1(FG,d0,A,C,r){e I;I.x=(C!=2)?-1.:3.;I.y=(C!=1)?-1.:3.;I.zw=c(.0,1.);B1(I);}
#endif
#ifdef EB
f ivec2 wf(){return ivec2(floor(gl_FragCoord));}
#ifdef DE
layout(location=0) inout R w0;layout(location=1) out i T4;void main(){T4.x=uintBitsToFloat(w0.x);}
#elif defined(EE)
#ifdef LE
__pixel_local_outEXT V1{layout(r32f) float w0;};
#else
__pixel_local_inEXT V1{layout(r32f) float w0;};layout(location=0) out i T4;
#endif
void main(){
#ifdef LE
w0=.0;
#else
T4.x=w0;
#endif
}
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui) uniform highp upixelLocalANGLE w0;layout(location=0) out i T4;void main(){T4.x=uintBitsToFloat(pixelLocalLoadANGLE(w0).x);}
#elif defined(FE)
layout(binding=0,r32i) uniform highp coherent iimage2D da;layout(location=0) out i T4;void main(){T4.x=float(imageLoad(da,wf()).x)*(1./te);}
#elif defined(GF)
Y2(c3,0,JF);layout(location=0) out i T4;void main(){i T=q1(JF,wf());T4.x=(T.x-T.y)*Jb+(T.z-T.w)*255.;}
#endif
#endif
