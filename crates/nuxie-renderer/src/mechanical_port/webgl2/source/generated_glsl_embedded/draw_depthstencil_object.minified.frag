#ifdef FB
#ifdef NB
O3 i3(w5,m4,CC);
#ifdef N
E5(XD);
#endif
P3 x5 n4(r5) y5
#endif
j3(i,IB){
#ifdef NB
q(V5,c);q(R1,i);
#ifdef N
q(I1,Q);
#endif
#else
q(a1,e);
#ifdef GB
q(v1,O);
#endif
#ifdef EB
q(J2,c);
#endif
#ifdef N
q(Q0,d);
#endif
#endif
#ifdef NB
i p=J7(CC,r5,V5,j.Wd)*R1;
#else
d n=
#ifdef EB
clamp(o2(FD,na,J2,.0).x,H0(.0),H0(1.));
#else
1.;
#endif
i p=X7(
#ifdef GB
v1,
#endif
#ifdef N
k3(Q0),
#endif
a1 e3);
#endif
#if defined(N)&&!defined(W)
#ifdef NB
p.xyz=Q6(p);Q z3=I1;
#else
Q z3=k3(Q0);
#endif
i J1=I6(XD);p.xyz=h5(p.xyz,J1,z3)*p.w;
#endif
#ifndef NB
p*=n;
#endif
p.xyz=M2(p.xyz,p.w,f0.xy,j.M3,j.N3);P2(p);}
#endif
