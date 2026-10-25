#ifdef FRAGMENT
F1 y0(x2,n0);r1(k3,m0);y0(E6,M4);r1(l7,r8);G1 I1(HB){q(N0,e);
#ifdef ENABLE_MODULATED_IMAGE
q(O0,M);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
q(p1,d);
#else
q(T,R2);
#endif
q(I0,d);
#ifdef ENABLE_CLIPPING
q(k2,E);
#endif
#ifdef ENABLE_CLIP_RECT
q(W0,e);
#endif
#ifdef ENABLE_ADVANCED_BLEND
q(S0,d);
#endif
#if!defined(DRAW_INTERIOR_TRIANGLES)
P2;
#endif
E r5=unpackHalf2x16(k1(r8));d la=r5.y;d w0=la==I0?r5.x:C0(.0);
#ifdef DRAW_INTERIOR_TRIANGLES
w0+=p1;i2(r8);
#else
w0=Xj(w0,T n1);m1(r8,packHalf2x16(S2(w0,I0)));
#endif
d n;
#ifdef CLOCKWISE_FILL
if(CLOCKWISE_FILL){n=jb(w0,C0(.0),C0(1.));}else
#endif
{n=abs(w0);
#ifdef ENABLE_EVEN_ODD
if(ENABLE_EVEN_ODD&&I0<.0){n=1.-C0(abs(fract(n*.5)*2.+-1.));}
#endif
n=min(n,C0(1.));}
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&k2.x<.0){d B1=-k2.x;
#ifdef ENABLE_NESTED_CLIPPING
if(ENABLE_NESTED_CLIPPING){d h6=k2.y;if(h6!=.0){E X0=unpackHalf2x16(k1(m0));d h7=X0.y;d Q4;if(h7!=B1){Q4=h7==h6?X0.x:.0;
#ifndef DRAW_INTERIOR_TRIANGLES
x0(M4,E0(Q4,.0,.0,.0));
#endif
}else{Q4=K0(M4).x;
#ifndef DRAW_INTERIOR_TRIANGLES
O2(M4);
#endif
}n=min(n,Q4);}}
#endif
m1(m0,packHalf2x16(S2(n,B1)));O2(n0);}else
#endif
{
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){d B1=k2.x;if(B1!=.0){E X0=unpackHalf2x16(k1(m0));d h7=X0.y;n=(h7==B1)?min(X0.x,n):C0(.0);}}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d A5=C3(Y4(W0));n=clamp(A5,C0(.0),n);}
#endif
i k=z8(
#ifdef ENABLE_MODULATED_IMAGE
O0,
#endif
#ifdef ENABLE_ADVANCED_BLEND
W2(S0),
#endif
N0 m3);i a1;if(la!=I0){a1=K0(n0);
#ifndef DRAW_INTERIOR_TRIANGLES
x0(M4,a1);
#endif
}else{a1=K0(M4);
#ifndef DRAW_INTERIOR_TRIANGLES
O2(M4);
#endif
}bool Sf=false;
#ifdef ENABLE_MODULATED_IMAGE
Sf=ENABLE_MODULATED_IMAGE&&O0.z<.0;
#endif
if(Sf){
#ifdef ENABLE_MODULATED_IMAGE
uint Ob=uint(-O0.z-1.);d Pb=we(k,Ob);k=a1*mix(C0(1.),Pb,n);x0(n0,k);i2(m0);
#endif
}else{
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND&&S0!=x6(X3)){k.xyz=R4(k.xyz,a1,W2(S0))*k.w;}
#endif
k*=n;d y3=k.w;k+=a1*(1.-y3);k.xyz=J2(k.xyz,y3,c0.xy,j.H3,j.I3);x0(n0,k);i2(m0);}}
#if!defined(DRAW_INTERIOR_TRIANGLES)
Q2;
#endif
O1;}
#endif
