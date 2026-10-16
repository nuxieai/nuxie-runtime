#ifdef EB
E1
#ifndef U
z0(x2,n0);
#endif
q1(k3,m0);
#ifndef U
fc(A6,g7);
#endif
q1(l7,Y0);F1
#ifdef U
H2(IB)
#else
X1(IB)
#endif
{q(P0,f);
#ifdef GB
q(U0,M);
#endif
#ifdef DB
q(o1,d);
#else
q(S,Q2);
#endif
q(G0,d);
#ifdef N
q(i2,D);
#endif
#ifdef AB
q(V0,f);
#endif
#ifdef H
q(Q0,d);
#endif
d B0=
#ifdef DB
o1;
#else
Hc(S);
#endif
i o0;d S1;
#if defined(DB)&&defined(EC)
if(!EC)
#endif
{o0=x8(
#ifdef GB
U0,
#endif
#ifdef H
W2(Q0),
#endif
P0 m3);S1=1.;
#ifdef AB
if(AB){d Mc=C3(V4(V0));S1=min(Mc,S1);}
#endif
}O2;
#if defined(DB)&&defined(EC)
if(EC){m1(Y0,packHalf2x16(R2(B0,G0)));
#ifndef U
N2(n0);
#endif
}else
#endif
{D o5=unpackHalf2x16(k1(Y0));d ka=o5.y;d p5=ka==G0?o5.x:J0(.0);d Hf=
#ifndef DB
o6(S)?max(p5,B0):
#endif
p5+B0;
#ifdef N
if(N&&i2.x!=.0){D W0=unpackHalf2x16(k1(m0));d g6=W0.y;d Nc=g6==i2.x?W0.x:J0(.0);S1=min(Nc,S1);}
#endif
S1=max(S1,.0);d p2=ib(p5,.0,S1);d Q1=ib(Hf,.0,S1);
#ifdef OB
d f6;if(OB){f6=lb(d0.xy,j.G3,j.H3);}
#endif
#ifndef U
i a1=O0(n0);
#ifdef H
if(H&&Q0!=r6(U3)){if(Q1!=.0){if(p2==.0){o0.xyz=N4(o0.xyz,a1,W2(Q0));
#ifndef DB
if(Q1<S1){v C8=o0.xyz;
#ifdef OB
if(OB){C8+=f6*j.He;}
#endif
x0(g7,H0(C8,0.0));}
#endif
}else{o0.xyz=O0(g7).xyz;N2(g7);}}o0.xyz*=o0.w;}
#endif
#endif
o0*=M9(p2,Q1,o0.w);
#ifdef OB
o0.xyz=J2(o0.xyz,o0.w,f6);
#endif
#ifndef DB
#ifdef H
#define If (!H||Q0==r6(U3))&&o0.w>=1.
#else
#define If o0.w>=1.
#endif
Ue(If,Y0,packHalf2x16(R2(Hf,G0)));
#else
g2(Y0);
#endif
#ifndef U
Te(o0.x+o0.y+o0.z+o0.w==.0,n0,a1*(1.-o0.w)+o0);
#endif
}g2(m0);P2;
#ifdef U
N1=o0;F3
#else
o2;
#endif
}
#endif
