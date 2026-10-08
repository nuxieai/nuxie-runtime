#ifdef EB
#if(defined(U)&&!defined(N))||defined(QB)
#undef Gc
#else
#define Gc
#endif
V1
#ifndef U
C0(U2,n0);
#endif
#ifndef QB
q1(i3,m0);
#ifndef U
C0(w6,G4);
#endif
q1(d7,Z0);
#else
C0(i3,m0);
#endif
W1
#ifdef NB
U3 p3(x5,q4,TB);V3 y5 r4(S4) z5 k4 l4
#endif
#ifdef U
#ifdef NB
G2(IB)
#else
G2(IB)
#endif
#else
#ifdef NB
Y1(IB)
#else
Y1(IB)
#endif
#endif
{
#ifdef FB
q(O0,e);
#if defined(GB)
q(V0,M);
#endif
q(T2,c);
#endif
#ifdef N
q(e4,d);
#endif
#ifdef AB
q(W0,e);
#endif
#if defined(FB)&&defined(H)
q(P0,d);
#endif
#ifdef NB
q(Z5,c);q(U1,i);
#ifdef H
q(K1,P);
#endif
#endif
#ifdef FB
i n=o8(
#ifdef GB
V0,
#endif
#ifdef H
X2(P0),
#endif
O0 l3);d l=clamp(o2(GD,Ja,T2,.0).x,I0(.0),I0(1.));
#endif
#ifdef NB
i n=c8(TB,S4,Z5,j.Be);d l=1.;
#endif
#ifdef AB
if(AB){d r5=max(A3(T4(W0)),I0(.0));l=min(r5,l);}
#endif
#ifdef Gc
O2;
#endif
#if defined(N)
if(N&&e4!=.0){d O3;
#ifndef QB
D X0=unpackHalf2x16(l1(m0));d Y6=X0.y;O3=max(Y6==e4?X0.x:I0(.0),I0(.0));
#else
O3=R0(m0).x;
#endif
O3=max(O3,I0(.0));l=min(l,O3);}
#endif
#ifdef NB
n*=U1;
#endif
#if!defined(U)
i A1=R0(n0);
#ifdef H
#ifdef FB
P X1=X2(P0);
#endif
#ifdef NB
P X1=K1;
#endif
if(H&&X1!=T3){
#ifdef NB
n.xyz=f6(n);
#endif
n.xyz=L4(n.xyz,A1,X1)*n.w;}
#endif
n*=l;n.xyz=I2(n.xyz,n.w,d0.xy,j.E3,j.F3);
#ifndef QB
n=A1*(1.-n.w)+n;
#endif
z0(n0,n);
#endif
#ifndef QB
h2(m0);h2(Z0);
#else
z0(m0,H0(.0));
#endif
#ifdef Gc
P2;
#endif
#ifdef U
n=(n*l);n.xyz=I2(n.xyz,n.w,d0.xy,j.E3,j.F3);N1=n;D3
#else
p2;
#endif
}
#endif
