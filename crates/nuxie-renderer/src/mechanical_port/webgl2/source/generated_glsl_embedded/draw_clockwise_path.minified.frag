#ifdef GB
I1
#ifndef Q
w0(Q2,j0);
#endif
j1(R2,h0);
#ifndef Q
Sa(d6,A6);
#endif
j1(G6,P0);J1
#ifdef Q
o2(JB)
#else
L1(JB)
#endif
{A(f1,g);
#ifdef EB
A(i1,c);
#else
A(O,z2);
#endif
A(B0,c);
#ifdef I
A(U1,E);
#endif
#ifdef BB
A(L0,g);
#endif
#ifdef AB
A(e2,c);
#endif
c r0=
#ifdef EB
i1;
#else
nb(O);
#endif
i v0;c F1;
#if defined(EB)&&defined(EC)
if(!EC)
#endif
{v0=J7(f1,1. S2);F1=1.;
#ifdef BB
if(BB){c sb=f3(Y4(L0));F1=min(sb,F1);}
#endif
}w2;
#if defined(EB)&&defined(EC)
if(EC){c1(P0,packHalf2x16(A2(r0,B0)));
#ifndef Q
v2(j0);
#endif
}else
#endif
{E N4=unpackHalf2x16(Y0(P0));c i9=N4.y;c O4=i9==B0?N4.x:G0(.0);c me=
#ifndef EB
R5(O)?max(O4,r0):
#endif
O4+r0;
#ifdef I
if(I&&U1.x!=.0){E M0=unpackHalf2x16(Y0(h0));c I5=M0.y;c tb=I5==U1.x?M0.x:G0(.0);F1=min(tb,F1);}
#endif
F1=max(F1,.0);c Z1=da(O4,.0,F1);c E1=da(me,.0,F1);
#ifdef LB
c H5;if(LB){H5=ga(Y.xy,n.z3,n.A3);}
#endif
#ifndef Q
i K1=H0(j0);
#ifdef AB
if(AB){if(e2!=W5(N5)&&E1!=.0){if(Z1==.0){v0.xyz=Q4(v0.xyz,K1,X5(e2));
#ifndef EB
if(E1<F1){v M7=v0.xyz;
#ifdef LB
if(LB){M7+=H5*n.rd;}
#endif
x0(A6,C0(M7,0.0));}
#endif
}else{v0.xyz=H0(A6).xyz;v2(A6);}}v0.xyz*=v0.w;}
#endif
#endif
v0*=H8(Z1,E1,v0.w);
#ifdef LB
v0.xyz=E2(v0.xyz,v0.w,H5);
#endif
#ifndef EB
#ifdef AB
#define ne (!AB||e2==W5(N5))&&v0.w>=1.
#else
#define ne v0.w>=1.
#endif
Ed(ne,P0,packHalf2x16(A2(me,B0)));
#else
d2(P0);
#endif
#ifndef Q
Dd(v0.w==.0,j0,K1*(1.-v0.w)+v0);
#endif
}d2(h0);x2;
#ifdef Q
C1=v0;l3
#else
Y1;
#endif
}
#endif
