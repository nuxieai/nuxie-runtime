#ifdef DB
y1(SF,e0,F,B,r){g U;U.x=(B!=2)?-1.:3.;U.y=(B!=1)?-1.:3.;U.zw=d(.0,1.);z1(U);}
#endif
#ifdef GB
e ivec2 Kd(){return ivec2(floor(gl_FragCoord));}
#ifdef TD
layout(location=0)inout G p0;layout(location=1)out i j4;void main(){j4.x=uintBitsToFloat(p0.x);}
#elif defined(UD)
#ifdef AE
__pixel_local_outEXT R1{layout(r32f)float p0;};
#else
__pixel_local_inEXT R1{layout(r32f)float p0;};layout(location=0)out i j4;
#endif
void main(){
#ifdef AE
p0=.0;
#else
j4.x=p0;
#endif
}
#elif defined(EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)
layout(binding=0,r32ui)uniform highp upixelLocalANGLE p0;layout(location=0)out i j4;void main(){j4.x=uintBitsToFloat(pixelLocalLoadANGLE(p0).x);}
#elif defined(VD)
layout(binding=0,r32i)uniform highp coherent iimage2D V8;layout(location=0)out i j4;void main(){j4.x=float(imageLoad(V8,Kd()).x)*(1./Pc);}
#elif defined(TE)
X2(a3,0,WE);layout(location=0)out i j4;void main(){i P=q1(WE,Kd());j4.x=(P.x-P.y)*sa+(P.z-P.w)*255.;}
#endif
#endif
