#ifdef GB
J1
#ifndef Q
x0(S2,j0);
#endif
j1(T2,h0);
#ifndef Q
Ta(h6,E6);
#endif
j1(K6,P0);K1
#ifdef Q
p2(JB)
#else
M1(JB)
#endif
{r(f1,g);
#ifdef KB
r(A2,R);
#endif
#ifdef EB
r(i1,c);
#else
r(O,z2);
#endif
r(B0,c);
#ifdef I
r(V1,E);
#endif
#ifdef BB
r(M0,g);
#endif
#ifdef AB
r(f2,c);
#endif
c v0=
#ifdef EB
i1;
#else
rb(O);
#endif
i w0;c G1;
#if defined(EB)&&defined(FC)
if(!FC)
#endif
{w0=M7(f1,
#ifdef KB
A2,
#endif
1. U2);G1=1.;
#ifdef BB
if(BB){c wb=h3(c5(M0));G1=min(wb,G1);}
#endif
}x2;
#if defined(EB)&&defined(FC)
if(FC){c1(P0,packHalf2x16(B2(v0,B0)));
#ifndef Q
w2(j0);
#endif
}else
#endif
{E R4=unpackHalf2x16(Y0(P0));c i9=R4.y;c S4=i9==B0?R4.x:G0(.0);c ue=
#ifndef EB
V5(O)?max(S4,v0):
#endif
S4+v0;
#ifdef I
if(I&&V1.x!=.0){E N0=unpackHalf2x16(Y0(h0));c M5=N0.y;c xb=M5==V1.x?N0.x:G0(.0);G1=min(xb,G1);}
#endif
G1=max(G1,.0);c a2=da(S4,.0,G1);c F1=da(ue,.0,G1);
#ifdef MB
c L5;if(MB){L5=ga(a0.xy,m.B3,m.C3);}
#endif
#ifndef Q
i L1=I0(j0);
#ifdef AB
if(AB){if(f2!=a6(R5)&&F1!=.0){if(a2==.0){w0.xyz=U4(w0.xyz,L1,c6(f2));
#ifndef EB
if(F1<G1){A P7=w0.xyz;
#ifdef MB
if(MB){P7+=L5*m.vd;}
#endif
y0(E6,C0(P7,0.0));}
#endif
}else{w0.xyz=I0(E6).xyz;w2(E6);}}w0.xyz*=w0.w;}
#endif
#endif
w0*=K8(a2,F1,w0.w);
#ifdef MB
w0.xyz=F2(w0.xyz,w0.w,L5);
#endif
#ifndef EB
#ifdef AB
#define ve (!AB||f2==a6(R5))&&w0.w>=1.
#else
#define ve w0.w>=1.
#endif
Ld(ve,P0,packHalf2x16(B2(ue,B0)));
#else
e2(P0);
#endif
#ifndef Q
Kd(w0.w==.0,j0,L1*(1.-w0.w)+w0);
#endif
}e2(h0);y2;
#ifdef Q
D1=w0;n3
#else
Z1;
#endif
}
#endif
