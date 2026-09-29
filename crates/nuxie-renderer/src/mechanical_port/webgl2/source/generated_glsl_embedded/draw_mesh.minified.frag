#ifdef GB
#if(defined(Q)&&!defined(I))||defined(TB)
#undef zb
#else
#define zb
#endif
I1
#ifndef Q
x0(S2,j0);
#endif
#ifndef TB
j1(T2,h0);
#ifndef Q
x0(h6,l4);
#endif
j1(K6,P0);
#else
x0(T2,h0);
#endif
J1
#ifdef PB
E3 Z2(d5,W3,JC);F3 e5 X3(W5)f5 Q3 R3
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
r(H5,d);r(H1,c);
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
i j=B7(JC,W5,H5,m.ud);c n=1.;
#endif
#ifdef BB
if(BB){c Y4=max(h3(c5(M0)),G0(.0));n=min(Y4,n);}
#endif
#ifdef zb
x2;
#endif
#if defined(I)
if(I&&K3!=.0){c v3;
#ifndef TB
E N0=unpackHalf2x16(Y0(h0));c F6=N0.y;v3=max(F6==K3?N0.x:G0(.0),G0(.0));
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
N T3=c6(f2);
#endif
#ifdef PB
j.xyz=G6(j);N T3=A1;
#endif
if(T3!=R5){j.xyz=U4(j.xyz,K1,T3);}j.w*=n;j.xyz*=j.w;}else
#endif
{j*=n;}
#ifdef CC
if(CC){j=m3(j);}
#endif
j.xyz=F2(j.xyz,j.w,a0.xy,m.B3,m.C3);
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
#ifdef zb
y2;
#endif
#ifdef Q
j=(j*n);j.xyz=F2(j.xyz,j.w,a0.xy,m.B3,m.C3);C1=j;n3
#else
Z1;
#endif
}
#endif
