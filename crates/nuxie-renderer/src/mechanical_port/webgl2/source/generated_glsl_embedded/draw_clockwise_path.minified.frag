#ifdef EB
V1
#ifndef U
C0(U2,n0);
#endif
q1(i3,m0);
#ifndef U
ac(w6,X6);
#endif
q1(d7,Z0);W1
#ifdef U
G2(IB)
#else
Y1(IB)
#endif
{q(O0,e);
#ifdef GB
q(V0,M);
#endif
#ifdef DB
q(o1,d);
#else
q(S,Q2);
#endif
q(G0,d);
#ifdef N
q(j2,D);
#endif
#ifdef AB
q(W0,e);
#endif
#ifdef H
q(P0,d);
#endif
d B0=
#ifdef DB
o1;
#else
yc(S);
#endif
i o0;d R1;
#if defined(DB)&&defined(EC)
if(!EC)
#endif
{o0=o8(
#ifdef GB
V0,
#endif
#ifdef H
X2(P0),
#endif
O0 l3);R1=1.;
#ifdef AB
if(AB){d Dc=A3(T4(W0));R1=min(Dc,R1);}
#endif
}O2;
#if defined(DB)&&defined(EC)
if(EC){m1(Z0,packHalf2x16(R2(B0,G0)));
#ifndef U
N2(n0);
#endif
}else
#endif
{D k5=unpackHalf2x16(l1(Z0));d ba=k5.y;d m5=ba==G0?k5.x:I0(.0);d Cf=
#ifndef DB
l6(S)?max(m5,B0):
#endif
m5+B0;
#ifdef N
if(N&&j2.x!=.0){D X0=unpackHalf2x16(l1(m0));d d6=X0.y;d Ec=d6==j2.x?X0.x:I0(.0);R1=min(Ec,R1);}
#endif
R1=max(R1,.0);d q2=Xa(m5,.0,R1);d Q1=Xa(Cf,.0,R1);
#ifdef OB
d c6;if(OB){c6=ab(d0.xy,j.E3,j.F3);}
#endif
#ifndef U
i A1=R0(n0);
#ifdef H
if(H&&P0!=D5(T3)){if(Q1!=.0){if(q2==.0){o0.xyz=L4(o0.xyz,A1,X2(P0));
#ifndef DB
if(Q1<R1){v w8=o0.xyz;
#ifdef OB
if(OB){w8+=c6*j.Ce;}
#endif
z0(X6,H0(w8,0.0));}
#endif
}else{o0.xyz=R0(X6).xyz;N2(X6);}}o0.xyz*=o0.w;}
#endif
#endif
o0*=F9(q2,Q1,o0.w);
#ifdef OB
o0.xyz=I2(o0.xyz,o0.w,c6);
#endif
#ifndef DB
#ifdef H
#define Df (!H||P0==D5(T3))&&o0.w>=1.
#else
#define Df o0.w>=1.
#endif
Pe(Df,Z0,packHalf2x16(R2(Cf,G0)));
#else
h2(Z0);
#endif
#ifndef U
Oe(o0.x+o0.y+o0.z+o0.w==.0,n0,A1*(1.-o0.w)+o0);
#endif
}h2(m0);P2;
#ifdef U
N1=o0;D3
#else
p2;
#endif
}
#endif
