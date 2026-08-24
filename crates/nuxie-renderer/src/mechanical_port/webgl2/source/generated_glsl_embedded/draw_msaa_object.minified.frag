#ifdef GB
#ifdef OB
C3 X2(Z4,U3,IC);
#ifdef AB
g7(SD);
#endif
D3 a5 V3(S5)c5
#endif
Y2(i,JB){
#ifdef OB
A(E5,d);A(H1,c);
#ifdef AB
A(A1,N);
#endif
#else
A(f1,g);
#ifdef FB
A(C2,d);
#endif
#ifdef AB
A(e2,c);
#endif
#endif
#ifdef OB
i j=x7(IC,S5,E5,n.qd)*H1;
#else
c o=
#ifdef FB
clamp(n2(BD,Q9,C2,.0).x,G0(.0),G0(1.));
#else
1.;
#endif
i j=J7(f1,o S2);
#endif
#if defined(AB)&&!defined(Q)
#ifdef OB
j.xyz=C6(j);N R3=A1;
#else
N R3=X5(e2);
#endif
i K1=Q8(SD);j.xyz=Q4(j.xyz,K1,R3);j.xyz*=j.w;
#endif
#ifdef BC
if(BC){j=k3(j);}
#endif
j.xyz=E2(j.xyz,j.w,Y.xy,n.z3,n.A3);G2(j);}
#endif
