#ifdef FRAGMENT
U1 C0(T2,n0);p1(j3,m0);C0(A6,I4);p1(h7,m8);V1 X1(IB){q(O0,f);
#ifdef ENABLE_MODULATED_IMAGE
q(U0,M);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
q(n1,d);
#else
q(S,P2);
#endif
q(G0,d);
#ifdef ENABLE_CLIPPING
q(i2,D);
#endif
#ifdef ENABLE_CLIP_RECT
q(V0,f);
#endif
#ifdef ENABLE_ADVANCED_BLEND
q(P0,d);
#endif
#if!defined(DRAW_INTERIOR_TRIANGLES)
N2;
#endif
D n5=unpackHalf2x16(j1(m8));d ia=n5.y;d w0=ia==G0?n5.x:J0(.0);
#ifdef DRAW_INTERIOR_TRIANGLES
w0+=n1;g2(m8);
#else
w0=Kj(w0,S m1);l1(m8,packHalf2x16(Q2(w0,G0)));
#endif
d n;
#ifdef CLOCKWISE_FILL
if(CLOCKWISE_FILL){n=eb(w0,J0(.0),J0(1.));}else
#endif
{n=abs(w0);
#ifdef ENABLE_EVEN_ODD
if(ENABLE_EVEN_ODD&&G0<.0){n=1.-J0(abs(fract(n*.5)*2.+-1.));}
#endif
n=min(n,J0(1.));}
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&i2.x<.0){d y1=-i2.x;
#ifdef ENABLE_NESTED_CLIPPING
if(ENABLE_NESTED_CLIPPING){d e6=i2.y;if(e6!=.0){D W0=unpackHalf2x16(j1(m0));d d7=W0.y;d M4;if(d7!=y1){M4=d7==e6?W0.x:.0;
#ifndef DRAW_INTERIOR_TRIANGLES
y0(I4,H0(M4,.0,.0,.0));
#endif
}else{M4=Q0(I4).x;
#ifndef DRAW_INTERIOR_TRIANGLES
M2(I4);
#endif
}n=min(n,M4);}}
#endif
l1(m0,packHalf2x16(Q2(n,y1)));M2(n0);}else
#endif
{
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){d y1=i2.x;if(y1!=.0){D W0=unpackHalf2x16(j1(m0));d d7=W0.y;n=(d7==y1)?min(W0.x,n):J0(.0);}}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d x5=B3(V4(V0));n=clamp(x5,J0(.0),n);}
#endif
i l=r8(
#ifdef ENABLE_MODULATED_IMAGE
U0,
#endif
#ifdef ENABLE_ADVANCED_BLEND
W2(P0),
#endif
O0 l3);i z1;if(ia!=G0){z1=Q0(n0);
#ifndef DRAW_INTERIOR_TRIANGLES
y0(I4,z1);
#endif
}else{z1=Q0(I4);
#ifndef DRAW_INTERIOR_TRIANGLES
M2(I4);
#endif
}bool Hf=false;
#ifdef ENABLE_MODULATED_IMAGE
Hf=ENABLE_MODULATED_IMAGE&&U0.z<.0;
#endif
if(Hf){
#ifdef ENABLE_MODULATED_IMAGE
uint Qj=uint(-U0.z-1.);d Rj=Jj(l,Qj);l=z1*mix(J0(1.),Rj,n);y0(n0,l);g2(m0);
#endif
}else{
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND&&P0!=r6(U3)){l.xyz=N4(l.xyz,z1,W2(P0))*l.w;}
#endif
l*=n;d w3=l.w;l+=z1*(1.-w3);l.xyz=I2(l.xyz,w3,d0.xy,j.F3,j.G3);y0(n0,l);g2(m0);}}
#if!defined(DRAW_INTERIOR_TRIANGLES)
O2;
#endif
o2;}
#endif
