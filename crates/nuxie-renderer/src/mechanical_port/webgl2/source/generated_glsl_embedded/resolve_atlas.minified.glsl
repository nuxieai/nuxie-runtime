#ifdef DB
z1(UF,e0,F,B,A){g V;V.x=(B!=2)?-1.:3.;V.y=(B!=1)?-1.:3.;V.zw=d(.0,1.);A1(V);}
#endif
#ifdef GB
e ivec2 Rd(){return ivec2(floor(gl_FragCoord));}
#ifdef VD
layout(location=0)inout X p0;layout(location=1)out i m4;void main(){m4.x=uintBitsToFloat(p0.x);}
#elif defined(WD)
#ifdef CE
__pixel_local_outEXT S1{layout(r32f)float p0;};
#else
__pixel_local_inEXT S1{layout(r32f)float p0;};layout(location=0)out i m4;
#endif
void main(){
#ifdef CE
p0=.0;
#else
m4.x=p0;
#endif
}
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui)uniform highp upixelLocalANGLE p0;layout(location=0)out i m4;void main(){m4.x=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);}
#elif defined(XD)
layout(binding=0,r32i)uniform highp coherent iimage2D X8;layout(location=0)out i m4;void main(){m4.x=float(imageLoad(X8,Rd()).x)*(1./Tc);}
#elif defined(UE)
Z2(d3,0,XE);layout(location=0)out i m4;void main(){i M=q1(XE,Rd());m4.x=(M.x-M.y)*va+(M.z-M.w)*255.;}
#endif
#endif
