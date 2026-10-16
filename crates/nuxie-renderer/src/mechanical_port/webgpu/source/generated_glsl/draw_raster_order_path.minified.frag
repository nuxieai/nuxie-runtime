#ifdef FRAGMENT
E1 z0(x2,n0);q1(k3,m0);z0(A6,I4);q1(l7,p8);F1 X1(IB){q(P0,f);
#ifdef ENABLE_MODULATED_IMAGE
q(U0,M);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
q(o1,d);
#else
q(S,Q2);
#endif
q(G0,d);
#ifdef ENABLE_CLIPPING
q(i2,D);
#endif
#ifdef ENABLE_CLIP_RECT
q(V0,f);
#endif
#ifdef ENABLE_ADVANCED_BLEND
q(Q0,d);
#endif
#if!defined(DRAW_INTERIOR_TRIANGLES)
O2;
#endif
D o5=unpackHalf2x16(k1(p8));d ka=o5.y;d w0=ka==G0?o5.x:J0(.0);
#ifdef DRAW_INTERIOR_TRIANGLES
w0+=o1;g2(p8);
#else
w0=Oj(w0,S n1);m1(p8,packHalf2x16(R2(w0,G0)));
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
if(ENABLE_CLIPPING&&i2.x<.0){d A1=-i2.x;
#ifdef ENABLE_NESTED_CLIPPING
if(ENABLE_NESTED_CLIPPING){d e6=i2.y;if(e6!=.0){D W0=unpackHalf2x16(k1(m0));d h7=W0.y;d M4;if(h7!=A1){M4=h7==e6?W0.x:.0;
#ifndef DRAW_INTERIOR_TRIANGLES
x0(I4,H0(M4,.0,.0,.0));
#endif
}else{M4=O0(I4).x;
#ifndef DRAW_INTERIOR_TRIANGLES
N2(I4);
#endif
}n=min(n,M4);}}
#endif
m1(m0,packHalf2x16(R2(n,A1)));N2(n0);}else
#endif
{
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){d A1=i2.x;if(A1!=.0){D W0=unpackHalf2x16(k1(m0));d h7=W0.y;n=(h7==A1)?min(W0.x,n):J0(.0);}}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d y5=C3(V4(V0));n=clamp(y5,J0(.0),n);}
#endif
i k=x8(
#ifdef ENABLE_MODULATED_IMAGE
U0,
#endif
#ifdef ENABLE_ADVANCED_BLEND
W2(Q0),
#endif
P0 m3);i a1;if(ka!=G0){a1=O0(n0);
#ifndef DRAW_INTERIOR_TRIANGLES
x0(I4,a1);
#endif
}else{a1=O0(I4);
#ifndef DRAW_INTERIOR_TRIANGLES
N2(I4);
#endif
}bool Jf=false;
#ifdef ENABLE_MODULATED_IMAGE
Jf=ENABLE_MODULATED_IMAGE&&U0.z<.0;
#endif
if(Jf){
#ifdef ENABLE_MODULATED_IMAGE
uint Uj=uint(-U0.z-1.);d Vj=Nj(k,Uj);k=a1*mix(J0(1.),Vj,n);x0(n0,k);g2(m0);
#endif
}else{
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND&&Q0!=r6(U3)){k.xyz=N4(k.xyz,a1,W2(Q0))*k.w;}
#endif
k*=n;d y3=k.w;k+=a1*(1.-y3);k.xyz=J2(k.xyz,y3,d0.xy,j.G3,j.H3);x0(n0,k);g2(m0);}}
#if!defined(DRAW_INTERIOR_TRIANGLES)
P2;
#endif
o2;}
#endif
