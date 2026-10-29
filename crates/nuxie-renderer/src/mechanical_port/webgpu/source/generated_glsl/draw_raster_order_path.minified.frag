#ifdef FRAGMENT
F1 y0(z2,o0);q1(l3,m0);y0(B6,M4);q1(j7,p8);G1 I1(HB){q(M0,e);
#ifdef ENABLE_MODULATED_IMAGE
q(N0,M);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
q(o1,d);
#else
q(S,T2);
#endif
q(I0,d);
#ifdef ENABLE_CLIPPING
q(l2,E);
#endif
#ifdef ENABLE_CLIP_RECT
q(V0,e);
#endif
#ifdef ENABLE_ADVANCED_BLEND
q(R0,d);
#endif
#if!defined(DRAW_INTERIOR_TRIANGLES)
R2;
#endif
E q5=unpackHalf2x16(j1(p8));d la=q5.y;d w0=la==I0?q5.x:C0(.0);
#ifdef DRAW_INTERIOR_TRIANGLES
w0+=o1;j2(p8);
#else
w0=Xj(w0,S m1);l1(p8,packHalf2x16(U2(w0,I0)));
#endif
d n;
#ifdef CLOCKWISE_FILL
if(CLOCKWISE_FILL){n=kb(w0,C0(.0),C0(1.));}else
#endif
{n=abs(w0);
#ifdef ENABLE_EVEN_ODD
if(ENABLE_EVEN_ODD&&I0<.0){n=1.-C0(abs(fract(n*.5)*2.+-1.));}
#endif
n=min(n,C0(1.));}
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&l2.x<.0){d B1=-l2.x;
#ifdef ENABLE_NESTED_CLIPPING
if(ENABLE_NESTED_CLIPPING){d f6=l2.y;if(f6!=.0){E W0=unpackHalf2x16(j1(m0));d f7=W0.y;d Q4;if(f7!=B1){Q4=f7==f6?W0.x:.0;
#ifndef DRAW_INTERIOR_TRIANGLES
x0(M4,E0(Q4,.0,.0,.0));
#endif
}else{Q4=K0(M4).x;
#ifndef DRAW_INTERIOR_TRIANGLES
Q2(M4);
#endif
}n=min(n,Q4);}}
#endif
l1(m0,packHalf2x16(U2(n,B1)));Q2(o0);}else
#endif
{
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){d B1=l2.x;if(B1!=.0){E W0=unpackHalf2x16(j1(m0));d f7=W0.y;n=(f7==B1)?min(W0.x,n):C0(.0);}}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d y5=D3(Y4(V0));n=clamp(y5,C0(.0),n);}
#endif
i k=x8(
#ifdef ENABLE_MODULATED_IMAGE
N0,
#endif
#ifdef ENABLE_ADVANCED_BLEND
Y2(R0),
#endif
M0 n3);i Z0;if(la!=I0){Z0=K0(o0);
#ifndef DRAW_INTERIOR_TRIANGLES
x0(M4,Z0);
#endif
}else{Z0=K0(M4);
#ifndef DRAW_INTERIOR_TRIANGLES
Q2(M4);
#endif
}bool Sf=false;
#ifdef ENABLE_MODULATED_IMAGE
Sf=ENABLE_MODULATED_IMAGE&&N0.z<.0;
#endif
if(Sf){
#ifdef ENABLE_MODULATED_IMAGE
uint Pb=uint(-N0.z-1.);d Qb=we(k,Pb);k=Z0*mix(C0(1.),Qb,n);x0(o0,k);j2(m0);
#endif
}else{
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND&&R0!=r6(X3)){k.xyz=R4(k.xyz,Z0,Y2(R0))*k.w;}
#endif
k*=n;d z3=k.w;k+=Z0*(1.-z3);k.xyz=K2(k.xyz,z3,c0.xy,j.I3,j.J3);x0(o0,k);j2(m0);}}
#if!defined(DRAW_INTERIOR_TRIANGLES)
S2;
#endif
O1;}
#endif
