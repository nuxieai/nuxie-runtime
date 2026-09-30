#ifdef EB
M1
#ifndef Q
z0(H2,l0);
#endif
j1(Y2,i0);
#ifndef Q
Ya(f6,E6);
#endif
j1(K6,S0);N1
#ifdef Q
w2(HB)
#else
P1(HB)
#endif
{r(X1,f);
#ifdef IB
r(D2,S);
#endif
#ifdef DB
r(i1,d);
#else
r(O,C2);
#endif
r(D0,d);
#ifdef K
r(Y1,D);
#endif
#ifdef AB
r(P0,f);
#endif
#ifdef T
r(f1,d);
#endif
d y0=
#ifdef DB
i1;
#else
tb(O);
#endif
i j0;d I1;
#if defined(DB)&&defined(CC)
if(!CC)
#endif
{j0=K7(
#ifdef IB
D2,
#endif
#ifdef T
g3(f1),
#endif
X1 Z2);I1=1.;
#ifdef AB
if(AB){d yb=m3(h5(P0));I1=min(yb,I1);}
#endif
}A2;
#if defined(DB)&&defined(CC)
if(CC){d1(S0,packHalf2x16(E2(y0,D0)));
#ifndef Q
z2(l0);
#endif
}else
#endif
{D V4=unpackHalf2x16(a1(S0));d j9=V4.y;d X4=j9==D0?V4.x:J0(.0);d Ge=
#ifndef DB
V5(O)?max(X4,y0):
#endif
X4+y0;
#ifdef K
if(K&&Y1.x!=.0){D Q0=unpackHalf2x16(a1(i0));d M5=Q0.y;d zb=M5==Y1.x?Q0.x:J0(.0);I1=min(zb,I1);}
#endif
I1=max(I1,.0);d e2=ga(X4,.0,I1);d H1=ga(Ge,.0,I1);
#ifdef LB
d L5;if(LB){L5=ja(e0.xy,j.F3,j.G3);}
#endif
#ifndef Q
i O1=K0(l0);
#ifdef T
if(T&&f1!=Z5(C4)){if(H1!=.0){if(e2==.0){j0.xyz=Z4(j0.xyz,O1,g3(f1));
#ifndef DB
if(H1<I1){v O7=j0.xyz;
#ifdef LB
if(LB){O7+=L5*j.Ed;}
#endif
A0(E6,E0(O7,0.0));}
#endif
}else{j0.xyz=K0(E6).xyz;z2(E6);}}j0.xyz*=j0.w;}
#endif
#endif
j0*=K8(e2,H1,j0.w);
#ifdef LB
j0.xyz=L2(j0.xyz,j0.w,L5);
#endif
#ifndef DB
#ifdef T
#define He (!T||f1==Z5(C4))&&j0.w>=1.
#else
#define He j0.w>=1.
#endif
Ud(He,S0,packHalf2x16(E2(Ge,D0)));
#else
h2(S0);
#endif
#ifndef Q
Td(j0.x+j0.y+j0.z+j0.w==.0,l0,O1*(1.-j0.w)+j0);
#endif
}h2(i0);B2;
#ifdef Q
E1=j0;r3
#else
d2;
#endif
}
#endif
