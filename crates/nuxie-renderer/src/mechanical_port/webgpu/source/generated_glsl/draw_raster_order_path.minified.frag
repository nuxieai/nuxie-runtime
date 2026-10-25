#ifdef FRAGMENT
F1 y0(x2,n0);r1(k3,m0);y0(E6,K4);r1(m7,p8);G1 I1(HB){q(P0,e);
#ifdef ENABLE_MODULATED_IMAGE
q(V0,M);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
q(p1,d);
#else
q(T,R2);
#endif
q(G0,d);
#ifdef ENABLE_CLIPPING
q(k2,E);
#endif
#ifdef ENABLE_CLIP_RECT
q(W0,e);
#endif
#ifdef ENABLE_ADVANCED_BLEND
q(Q0,d);
#endif
#if!defined(DRAW_INTERIOR_TRIANGLES)
P2;
#endif
E p5=unpackHalf2x16(k1(p8));d ka=p5.y;d w0=ka==G0?p5.x:J0(.0);
#ifdef DRAW_INTERIOR_TRIANGLES
w0+=p1;i2(p8);
#else
w0=Sj(w0,T n1);m1(p8,packHalf2x16(S2(w0,G0)));
#endif
d n;
#ifdef CLOCKWISE_FILL
if(CLOCKWISE_FILL){n=ib(w0,J0(.0),J0(1.));}else
#endif
{n=abs(w0);
#ifdef ENABLE_EVEN_ODD
if(ENABLE_EVEN_ODD&&G0<.0){n=1.-J0(abs(fract(n*.5)*2.+-1.));}
#endif
n=min(n,J0(1.));}
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&k2.x<.0){d B1=-k2.x;
#ifdef ENABLE_NESTED_CLIPPING
if(ENABLE_NESTED_CLIPPING){d h6=k2.y;if(h6!=.0){E X0=unpackHalf2x16(k1(m0));d i7=X0.y;d O4;if(i7!=B1){O4=i7==h6?X0.x:.0;
#ifndef DRAW_INTERIOR_TRIANGLES
x0(K4,H0(O4,.0,.0,.0));
#endif
}else{O4=K0(K4).x;
#ifndef DRAW_INTERIOR_TRIANGLES
O2(K4);
#endif
}n=min(n,O4);}}
#endif
m1(m0,packHalf2x16(S2(n,B1)));O2(n0);}else
#endif
{
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){d B1=k2.x;if(B1!=.0){E X0=unpackHalf2x16(k1(m0));d i7=X0.y;n=(i7==B1)?min(X0.x,n):J0(.0);}}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d z5=C3(X4(W0));n=clamp(z5,J0(.0),n);}
#endif
i k=x8(
#ifdef ENABLE_MODULATED_IMAGE
V0,
#endif
#ifdef ENABLE_ADVANCED_BLEND
W2(Q0),
#endif
P0 m3);i a1;if(ka!=G0){a1=K0(n0);
#ifndef DRAW_INTERIOR_TRIANGLES
x0(K4,a1);
#endif
}else{a1=K0(K4);
#ifndef DRAW_INTERIOR_TRIANGLES
O2(K4);
#endif
}bool Mf=false;
#ifdef ENABLE_MODULATED_IMAGE
Mf=ENABLE_MODULATED_IMAGE&&V0.z<.0;
#endif
if(Mf){
#ifdef ENABLE_MODULATED_IMAGE
uint Yj=uint(-V0.z-1.);d Zj=Rj(k,Yj);k=a1*mix(J0(1.),Zj,n);x0(n0,k);i2(m0);
#endif
}else{
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND&&Q0!=x6(W3)){k.xyz=P4(k.xyz,a1,W2(Q0))*k.w;}
#endif
k*=n;d y3=k.w;k+=a1*(1.-y3);k.xyz=J2(k.xyz,y3,c0.xy,j.G3,j.H3);x0(n0,k);i2(m0);}}
#if!defined(DRAW_INTERIOR_TRIANGLES)
Q2;
#endif
O1;}
#endif
