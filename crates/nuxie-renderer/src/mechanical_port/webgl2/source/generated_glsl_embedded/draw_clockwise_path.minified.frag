#ifdef FB
J1
#ifndef O
y0(U2,k0);
#endif
i1(V2,h0);
#ifndef O
Za(h6,E6);
#endif
i1(K6,Q0);K1
#ifdef O
r2(IB)
#else
M1(IB)
#endif
{r(V1,f);
#ifdef JB
r(C2,Q);
#endif
#ifdef EB
r(h1,d);
#else
r(M,B2);
#endif
r(C0,d);
#ifdef I
r(W1,E);
#endif
#ifdef BB
r(M0,f);
#endif
#ifdef AB
r(g2,d);
#endif
d w0=
#ifdef EB
h1;
#else
ub(M);
#endif
i x0;d F1;
#if defined(EB)&&defined(DC)
if(!DC)
#endif
{x0=M7(V1,
#ifdef JB
C2,
#endif
1. W2);F1=1.;
#ifdef BB
if(BB){d zb=i3(c5(M0));F1=min(zb,F1);}
#endif
}z2;
#if defined(EB)&&defined(DC)
if(DC){c1(Q0,packHalf2x16(D2(w0,C0)));
#ifndef O
y2(k0);
#endif
}else
#endif
{E R4=unpackHalf2x16(Y0(Q0));d k9=R4.y;d S4=k9==C0?R4.x:I0(.0);d Ee=
#ifndef EB
V5(M)?max(S4,w0):
#endif
S4+w0;
#ifdef I
if(I&&W1.x!=.0){E O0=unpackHalf2x16(Y0(h0));d K5=O0.y;d Ab=K5==W1.x?O0.x:I0(.0);F1=min(Ab,F1);}
#endif
F1=max(F1,.0);d c2=ha(S4,.0,F1);d E1=ha(Ee,.0,F1);
#ifdef LB
d J5;if(LB){J5=ka(c0.xy,n.B3,n.C3);}
#endif
#ifndef O
i L1=J0(k0);
#ifdef AB
if(AB){if(g2!=a6(R5)&&E1!=.0){if(c2==.0){x0.xyz=U4(x0.xyz,L1,c6(g2));
#ifndef EB
if(E1<F1){A Q7=x0.xyz;
#ifdef LB
if(LB){Q7+=J5*n.Dd;}
#endif
z0(E6,D0(Q7,0.0));}
#endif
}else{x0.xyz=J0(E6).xyz;y2(E6);}}x0.xyz*=x0.w;}
#endif
#endif
x0*=L8(c2,E1,x0.w);
#ifdef LB
x0.xyz=I2(x0.xyz,x0.w,J5);
#endif
#ifndef EB
#ifdef AB
#define Fe (!AB||g2==a6(R5))&&x0.w>=1.
#else
#define Fe x0.w>=1.
#endif
Td(Fe,Q0,packHalf2x16(D2(Ee,C0)));
#else
f2(Q0);
#endif
#ifndef O
Sd(x0.w==.0,k0,L1*(1.-x0.w)+x0);
#endif
}f2(h0);A2;
#ifdef O
C1=x0;m3
#else
a2;
#endif
}
#endif
