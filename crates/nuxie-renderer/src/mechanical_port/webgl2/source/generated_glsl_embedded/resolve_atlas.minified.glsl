#ifdef DB
y1(UF,e0,F,B,v){g W;W.x=(B!=2)?-1.:3.;W.y=(B!=1)?-1.:3.;W.zw=d(.0,1.);z1(W);}
#endif
#ifdef GB
e ivec2 Md(){return ivec2(floor(gl_FragCoord));}
#ifdef VD
layout(location=0)inout G p0;layout(location=1)out i l4;void main(){l4.x=uintBitsToFloat(p0.x);}
#elif defined(WD)
#ifdef DE
__pixel_local_outEXT R1{layout(r32f)float p0;};
#else
__pixel_local_inEXT R1{layout(r32f)float p0;};layout(location=0)out i l4;
#endif
void main(){
#ifdef DE
p0=.0;
#else
l4.x=p0;
#endif
}
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui)uniform highp upixelLocalANGLE p0;layout(location=0)out i l4;void main(){l4.x=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);}
#elif defined(XD)
layout(binding=0,r32i)uniform highp coherent iimage2D X8;layout(location=0)out i l4;void main(){l4.x=float(imageLoad(X8,Md()).x)*(1./Rc);}
#elif defined(VE)
Z2(d3,0,YE);layout(location=0)out i l4;void main(){i P=q1(YE,Md());l4.x=(P.x-P.y)*sa+(P.z-P.w)*255.;}
#endif
#endif
