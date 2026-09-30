#ifdef EB
#ifdef NB
O3 i3(r5,l4,IC);
#ifdef O
C5(YD);
#endif
P3 v5 m4(f6) w5
#endif
j3(i,IB){
#ifdef NB
q(T5,c);q(P1,i);
#ifdef O
q(H1,R);
#endif
#else
q(a1,e);
#ifdef GB
q(F1,P);
#endif
#ifdef FB
q(K2,c);
#endif
#ifdef O
q(Q0,d);
#endif
#endif
#ifdef NB
i l=K7(IC,f6,T5,j.Vd)*P1;
#else
d o=
#ifdef FB
clamp(o2(GD,ma,K2,.0).x,M0(.0),M0(1.));
#else
1.;
#endif
i l=Y7(
#ifdef GB
F1,
#endif
#ifdef O
k3(Q0),
#endif
a1 e3);
#endif
#if defined(O)&&!defined(V)
#ifdef NB
l.xyz=P6(l);R y3=H1;
#else
R y3=k3(Q0);
#endif
i S1=H6(YD);l.xyz=h5(l.xyz,S1,y3)*l.w;
#endif
#ifndef NB
l*=o;
#endif
#ifdef CC
if(CC){l=z3(l);}
#endif
l.xyz=O2(l.xyz,l.w,f0.xy,j.M3,j.N3);Q2(l);}
#endif
