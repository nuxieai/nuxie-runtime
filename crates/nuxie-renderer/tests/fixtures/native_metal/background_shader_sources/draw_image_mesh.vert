#ifdef DB
g1(i3)L(0,d,PC);h1 g1(y3)L(1,d,QC);h1 g1(n1)L(r9,g,XB);L(v9,g,SB);L(w9,g,OB);
#ifdef O3
L(x9,uint,YB);L(y9,uint,ZB);L(z9,uint,AC);L(A9,uint,BC);
#else
L(B9,G,IB);
#endif
h1
#endif
m2 H0 X(0,d,G5);
#ifdef I
NB X(1,c,K3);
#endif
#if defined(BB)&&!defined(CB)
H0 X(2,g,M0);
#endif
NB X(3,c,H1);
#ifdef AB
Q2 X(4,N,A1);
#endif
g2
#ifdef DB
U3 V3 I6(HC,i3,j3,y3,z3,n1,g0,B){M(B,j3,PC,d);M(B,z3,QC,d);M(v,g0,XB,g);M(v,g0,SB,g);M(v,g0,OB,g);
#ifdef O3
M(v,g0,YB,uint);M(v,g0,ZB,uint);M(v,g0,AC,uint);M(v,g0,BC,uint);G IB=G(YB,ZB,AC,BC);
#else
M(v,g0,IB,G);
#endif
V(G5,d);
#ifdef I
V(K3,c);
#endif
#if defined(BB)&&!defined(CB)
V(M0,g);
#endif
V(H1,c);
#ifdef AB
V(A1,N);
#endif
d m0=R0(h2(XB),PC)+OB.xy;G5=QC;
#ifdef I
if(I){K3=r8(IB.y,m.d6);}
#endif
#ifdef BB
if(BB){
#ifndef CB
M0=T7(h2(SB),OB.zw,m0 x5);
#else
Bc(h2(SB),OB.zw,m0 x5);
#endif
}
#endif
g W=M3(m0);
#ifdef SC
W.y=-W.y;
#endif
#ifdef CB
W.z=ja(IB.w);
#endif
H1=uintBitsToFloat(IB.x);
#ifdef AB
A1=X1(IB.z);
#endif
a0(G5);
#ifdef I
a0(K3);
#endif
#if defined(BB)&&!defined(CB)
a0(M0);
#endif
a0(H1);
#ifdef AB
a0(A1);
#endif
z1(W);}
#endif
