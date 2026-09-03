#ifdef GB
#if(defined(Q)&&!defined(I))||defined(TB)
#undef yb
#else
#define yb
#endif
I1
#ifndef Q
x0(S2,j0);
#endif
#ifndef TB
j1(T2,h0);
#ifndef Q
x0(g6,k4);
#endif
j1(J6,P0);
#else
x0(T2,h0);
#endif
J1
#ifdef PB
E3 Z2(c5,W3,JC);F3 d5 X3(V5)e5 Q3 R3
#endif
#ifdef Q
#ifdef PB
p2(JB)
#else
p2(JB)
#endif
#else
#ifdef PB
L1(JB)
#else
L1(JB)
#endif
#endif
{
#ifdef FB
r(f1,g);
#if defined(KB)
r(A2,R);
#endif
r(D2,d);
#endif
#ifdef I
r(K3,c);
#endif
#ifdef BB
r(M0,g);
#endif
#if defined(FB)&&defined(AB)
r(f2,c);
#endif
#ifdef PB
r(G5,d);r(H1,c);
#ifdef AB
r(A1,N);
#endif
#endif
#ifdef FB
i j=M7(f1,
#ifdef KB
A2,
#endif
1. U2);c n=clamp(o2(CD,Q9,D2,.0).x,G0(.0),G0(1.));
#endif
#ifdef PB
i j=A7(JC,V5,G5,m.td);c n=1.;
#endif
#ifdef BB
if(BB){c X4=max(h3(a5(M0)),G0(.0));n=min(X4,n);}
#endif
#ifdef yb
x2;
#endif
#if defined(I)
if(I&&K3!=.0){c v3;
#ifndef TB
E N0=unpackHalf2x16(Y0(h0));c E6=N0.y;v3=max(E6==K3?N0.x:G0(.0),G0(.0));
#else
v3=I0(h0).x;
#endif
v3=max(v3,G0(.0));n=min(n,v3);}
#endif
#ifdef PB
n*=H1;
#endif
#if!defined(Q)
i K1=I0(j0);
#ifdef AB
if(AB){
#ifdef FB
N T3=a6(f2);
#endif
#ifdef PB
j.xyz=F6(j);N T3=A1;
#endif
if(T3!=Q5){j.xyz=T4(j.xyz,K1,T3);}j.w*=n;j.xyz*=j.w;}else
#endif
{j*=n;}
#ifdef CC
if(CC){j=m3(j);}
#endif
j.xyz=F2(j.xyz,j.w,Z.xy,m.B3,m.C3);
#ifndef TB
j=K1*(1.-j.w)+j;
#endif
y0(j0,j);
#endif
#ifndef TB
e2(h0);e2(P0);
#else
y0(h0,C0(.0));
#endif
#ifdef yb
y2;
#endif
#ifdef Q
j=(j*n);j.xyz=F2(j.xyz,j.w,Z.xy,m.B3,m.C3);C1=j;n3
#else
Z1;
#endif
}
#endif
