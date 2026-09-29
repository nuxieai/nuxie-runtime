#ifdef GB
J1
#ifndef N
x0(S2,j0);
#endif
j1(T2,g0);
#ifndef N
Va(g6,D6);
#endif
j1(J6,P0);K1
#ifdef N
p2(IB)
#else
M1(IB)
#endif
{r(f1,g);
#ifdef JB
r(A2,Q);
#endif
#ifdef EB
r(i1,c);
#else
r(L,z2);
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
rb(L);
#endif
i w0;c G1;
#if defined(EB)&&defined(DC)
if(!DC)
#endif
{w0=M7(f1,
#ifdef JB
A2,
#endif
1. U2);G1=1.;
#ifdef BB
if(BB){c wb=h3(c5(M0));G1=min(wb,G1);}
#endif
}x2;
#if defined(EB)&&defined(DC)
if(DC){c1(P0,packHalf2x16(B2(v0,B0)));
#ifndef N
w2(j0);
#endif
}else
#endif
{E R4=unpackHalf2x16(Y0(P0));c i9=R4.y;c S4=i9==B0?R4.x:G0(.0);c ue=
#ifndef EB
U5(L)?max(S4,v0):
#endif
S4+v0;
#ifdef I
if(I&&V1.x!=.0){E N0=unpackHalf2x16(Y0(g0));c L5=N0.y;c xb=L5==V1.x?N0.x:G0(.0);G1=min(xb,G1);}
#endif
G1=max(G1,.0);c a2=ca(S4,.0,G1);c F1=ca(ue,.0,G1);
#ifdef LB
c K5;if(LB){K5=fa(a0.xy,m.A3,m.B3);}
#endif
#ifndef N
i L1=I0(j0);
#ifdef AB
if(AB){if(f2!=Z5(Q5)&&F1!=.0){if(a2==.0){w0.xyz=U4(w0.xyz,L1,a6(f2));
#ifndef EB
if(F1<G1){v P7=w0.xyz;
#ifdef LB
if(LB){P7+=K5*m.vd;}
#endif
y0(D6,C0(P7,0.0));}
#endif
}else{w0.xyz=I0(D6).xyz;w2(D6);}}w0.xyz*=w0.w;}
#endif
#endif
w0*=K8(a2,F1,w0.w);
#ifdef LB
w0.xyz=F2(w0.xyz,w0.w,K5);
#endif
#ifndef EB
#ifdef AB
#define ve (!AB||f2==Z5(Q5))&&w0.w>=1.
#else
#define ve w0.w>=1.
#endif
Ld(ve,P0,packHalf2x16(B2(ue,B0)));
#else
e2(P0);
#endif
#ifndef N
Kd(w0.w==.0,j0,L1*(1.-w0.w)+w0);
#endif
}e2(g0);y2;
#ifdef N
D1=w0;m3
#else
Z1;
#endif
}
#endif
