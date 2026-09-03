#ifdef GB
I1
#ifndef Q
x0(S2,j0);
#endif
j1(T2,h0);
#ifndef Q
Ta(g6,D6);
#endif
j1(J6,P0);J1
#ifdef Q
p2(JB)
#else
L1(JB)
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
qb(O);
#endif
i w0;c F1;
#if defined(EB)&&defined(FC)
if(!FC)
#endif
{w0=M7(f1,
#ifdef KB
A2,
#endif
1. U2);F1=1.;
#ifdef BB
if(BB){c vb=h3(a5(M0));F1=min(vb,F1);}
#endif
}x2;
#if defined(EB)&&defined(FC)
if(FC){c1(P0,packHalf2x16(B2(v0,B0)));
#ifndef Q
w2(j0);
#endif
}else
#endif
{E Q4=unpackHalf2x16(Y0(P0));c i9=Q4.y;c R4=i9==B0?Q4.x:G0(.0);c qe=
#ifndef EB
U5(O)?max(R4,v0):
#endif
R4+v0;
#ifdef I
if(I&&V1.x!=.0){E N0=unpackHalf2x16(Y0(h0));c L5=N0.y;c wb=L5==V1.x?N0.x:G0(.0);F1=min(wb,F1);}
#endif
F1=max(F1,.0);c a2=da(R4,.0,F1);c E1=da(qe,.0,F1);
#ifdef MB
c K5;if(MB){K5=ga(Z.xy,m.B3,m.C3);}
#endif
#ifndef Q
i K1=I0(j0);
#ifdef AB
if(AB){if(f2!=Z5(Q5)&&E1!=.0){if(a2==.0){w0.xyz=T4(w0.xyz,K1,a6(f2));
#ifndef EB
if(E1<F1){A P7=w0.xyz;
#ifdef MB
if(MB){P7+=K5*m.ud;}
#endif
y0(D6,C0(P7,0.0));}
#endif
}else{w0.xyz=I0(D6).xyz;w2(D6);}}w0.xyz*=w0.w;}
#endif
#endif
w0*=K8(a2,E1,w0.w);
#ifdef MB
w0.xyz=F2(w0.xyz,w0.w,K5);
#endif
#ifndef EB
#ifdef AB
#define re (!AB||f2==Z5(Q5))&&w0.w>=1.
#else
#define re w0.w>=1.
#endif
Hd(re,P0,packHalf2x16(B2(qe,B0)));
#else
e2(P0);
#endif
#ifndef Q
Gd(w0.w==.0,j0,K1*(1.-w0.w)+w0);
#endif
}e2(h0);y2;
#ifdef Q
C1=w0;n3
#else
Z1;
#endif
}
#endif
