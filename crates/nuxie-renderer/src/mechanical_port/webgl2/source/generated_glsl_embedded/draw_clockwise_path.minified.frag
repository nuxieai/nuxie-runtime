#ifdef EB
Q1
#ifndef V
A0(L2,o0);
#endif
o1(d3,m0);
#ifndef V
sb(n6,N6);
#endif
o1(T6,V0);R1
#ifdef V
A2(IB)
#else
T1(IB)
#endif
{q(a1,e);
#ifdef GB
q(F1,P);
#endif
#ifdef DB
q(m1,d);
#else
q(S,H2);
#endif
q(F0,d);
#ifdef A
q(l1,C);
#endif
#ifdef AB
q(R0,e);
#endif
#ifdef O
q(Q0,d);
#endif
d z0=
#ifdef DB
m1;
#else
Ub(S);
#endif
i n0;d M1;
#if defined(DB)&&defined(EC)
if(!EC)
#endif
{n0=Y7(
#ifdef GB
F1,
#endif
#ifdef O
k3(Q0),
#endif
a1 e3);M1=1.;
#ifdef AB
if(AB){d Zb=v3(q5(R0));M1=min(Zb,M1);}
#endif
}F2;
#if defined(DB)&&defined(EC)
if(EC){j1(V0,packHalf2x16(I2(z0,F0)));
#ifndef V
E2(o0);
#endif
}else
#endif
{C d5=unpackHalf2x16(h1(V0));d C9=d5.y;d f5=C9==F0?d5.x:M0(.0);d Xe=
#ifndef DB
e6(S)?max(f5,z0):
#endif
f5+z0;
#ifdef A
if(A&&l1.x!=.0){C T0=unpackHalf2x16(h1(m0));d V5=T0.y;d ac=V5==l1.x?T0.x:M0(.0);M1=min(ac,M1);}
#endif
M1=max(M1,.0);d h2=Aa(f5,.0,M1);d L1=Aa(Xe,.0,M1);
#ifdef OB
d U5;if(OB){U5=Da(f0.xy,j.M3,j.N3);}
#endif
#ifndef V
i S1=N0(o0);
#ifdef O
if(O&&Q0!=i6(L4)){if(L1!=.0){if(h2==.0){n0.xyz=h5(n0.xyz,S1,k3(Q0));
#ifndef DB
if(L1<M1){v d8=n0.xyz;
#ifdef OB
if(OB){d8+=U5*j.Wd;}
#endif
B0(N6,G0(d8,0.0));}
#endif
}else{n0.xyz=N0(N6).xyz;E2(N6);}}n0.xyz*=n0.w;}
#endif
#endif
n0*=a9(h2,L1,n0.w);
#ifdef OB
n0.xyz=O2(n0.xyz,n0.w,U5);
#endif
#ifndef DB
#ifdef O
#define Ye (!O||Q0==i6(L4))&&n0.w>=1.
#else
#define Ye n0.w>=1.
#endif
je(Ye,V0,packHalf2x16(I2(Xe,F0)));
#else
k2(V0);
#endif
#ifndef V
ie(n0.x+n0.y+n0.z+n0.w==.0,o0,S1*(1.-n0.w)+n0);
#endif
}k2(m0);G2;
#ifdef V
J1=n0;A3
#else
g2;
#endif
}
#endif
