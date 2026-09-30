#ifdef EB
#if(defined(V)&&!defined(A))||defined(QB)
#undef cc
#else
#define cc
#endif
Q1
#ifndef V
A0(L2,o0);
#endif
#ifndef QB
o1(d3,m0);
#ifndef V
A0(n6,B4);
#endif
o1(T6,V0);
#else
A0(d3,m0);
#endif
R1
#ifdef NB
O3 i3(r5,l4,IC);P3 v5 m4(f6) w5 f4 g4
#endif
#ifdef V
#ifdef NB
A2(IB)
#else
A2(IB)
#endif
#else
#ifdef NB
T1(IB)
#else
T1(IB)
#endif
#endif
{
#ifdef FB
q(a1,e);
#if defined(GB)
q(F1,P);
#endif
q(K2,c);
#endif
#ifdef A
q(Z3,d);
#endif
#ifdef AB
q(R0,e);
#endif
#if defined(FB)&&defined(O)
q(Q0,d);
#endif
#ifdef NB
q(T5,c);q(P1,i);
#ifdef O
q(H1,R);
#endif
#endif
#ifdef FB
i l=Y7(
#ifdef GB
F1,
#endif
#ifdef O
k3(Q0),
#endif
a1 e3);d o=clamp(o2(GD,ma,K2,.0).x,M0(.0),M0(1.));
#endif
#ifdef NB
i l=K7(IC,f6,T5,j.Vd);d o=1.;
#endif
#ifdef AB
if(AB){d l5=max(v3(q5(R0)),M0(.0));o=min(l5,o);}
#endif
#ifdef cc
F2;
#endif
#if defined(A)
if(A&&Z3!=.0){d G3;
#ifndef QB
C T0=unpackHalf2x16(h1(m0));d O6=T0.y;G3=max(O6==Z3?T0.x:M0(.0),M0(.0));
#else
G3=N0(m0).x;
#endif
G3=max(G3,M0(.0));o=min(o,G3);}
#endif
#ifdef NB
l*=P1;
#endif
#if!defined(V)
i S1=N0(o0);
#ifdef O
#ifdef FB
R y3=k3(Q0);
#endif
#ifdef NB
R y3=H1;
#endif
if(O&&y3!=L4){
#ifdef NB
l.xyz=P6(l);
#endif
l.xyz=h5(l.xyz,S1,y3)*l.w;}
#endif
l*=o;
#ifdef CC
if(CC){l=z3(l);}
#endif
l.xyz=O2(l.xyz,l.w,f0.xy,j.M3,j.N3);
#ifndef QB
l=S1*(1.-l.w)+l;
#endif
B0(o0,l);
#endif
#ifndef QB
k2(m0);k2(V0);
#else
B0(m0,G0(.0));
#endif
#ifdef cc
G2;
#endif
#ifdef V
l=(l*o);l.xyz=O2(l.xyz,l.w,f0.xy,j.M3,j.N3);J1=l;A3
#else
g2;
#endif
}
#endif
