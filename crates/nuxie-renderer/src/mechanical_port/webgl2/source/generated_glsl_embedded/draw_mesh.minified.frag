#ifdef FB
#if(defined(O)&&!defined(I))||defined(RB)
#undef Db
#else
#define Db
#endif
J1
#ifndef O
y0(F2,k0);
#endif
#ifndef RB
i1(U2,h0);
#ifndef O
y0(g6,m4);
#endif
i1(K6,Q0);
#else
y0(U2,h0);
#endif
K1
#ifdef OB
F3 a3(c5,X3,HC);G3 d5 Y3(V5)e5 Q3 R3
#endif
#ifdef O
#ifdef OB
r2(IB)
#else
r2(IB)
#endif
#else
#ifdef OB
M1(IB)
#else
M1(IB)
#endif
#endif
{
#ifdef GB
r(V1,f);
#if defined(JB)
r(B2,Q);
#endif
r(E2,c);
#endif
#ifdef I
r(L3,d);
#endif
#ifdef BB
r(M0,f);
#endif
#if defined(GB)&&defined(AB)
r(g2,d);
#endif
#ifdef OB
r(G5,c);r(H1,i);
#ifdef AB
r(A1,L);
#endif
#endif
#ifdef GB
i j=M7(V1,
#ifdef JB
B2,
#endif
1. V2);d o=clamp(i2(FD,S9,E2,.0).x,I0(.0),I0(1.));
#endif
#ifdef OB
i j=B7(HC,V5,G5,l.Ed);d o=1.;
#endif
#ifdef BB
if(BB){d X4=max(i3(a5(M0)),I0(.0));o=min(X4,o);}
#endif
#ifdef Db
y2;
#endif
#if defined(I)
if(I&&L3!=.0){d w3;
#ifndef RB
E O0=unpackHalf2x16(Y0(h0));d F6=O0.y;w3=max(F6==L3?O0.x:I0(.0),I0(.0));
#else
w3=J0(h0).x;
#endif
w3=max(w3,I0(.0));o=min(o,w3);}
#endif
#ifdef OB
j*=H1;
#endif
#if!defined(O)
i L1=J0(k0);
#ifdef AB
if(AB){
#ifdef GB
L U3=a6(g2);
#endif
#ifdef OB
j.xyz=G6(j);L U3=A1;
#endif
if(U3!=Q5){j.xyz=U4(j.xyz,L1,U3);}j.w*=o;j.xyz*=j.w;}else
#endif
{j*=o;}
#ifdef AC
if(AC){j=l3(j);}
#endif
j.xyz=I2(j.xyz,j.w,c0.xy,l.C3,l.D3);
#ifndef RB
j=L1*(1.-j.w)+j;
#endif
z0(k0,j);
#endif
#ifndef RB
f2(h0);f2(Q0);
#else
z0(h0,D0(.0));
#endif
#ifdef Db
z2;
#endif
#ifdef O
j=(j*o);j.xyz=I2(j.xyz,j.w,c0.xy,l.C3,l.D3);C1=j;m3
#else
a2;
#endif
}
#endif
