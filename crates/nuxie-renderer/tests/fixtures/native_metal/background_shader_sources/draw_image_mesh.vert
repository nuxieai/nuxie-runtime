#ifdef DB
g1(g3)L(0,d,OC);h1 g1(x3)L(1,d,PC);h1 g1(n1)L(r9,g,WB);L(v9,g,QB);L(w9,g,NB);
#ifdef M3
L(x9,uint,XB);L(y9,uint,YB);L(z9,uint,ZB);L(A9,uint,AC);
#else
L(B9,G,IB);
#endif
h1
#endif
k2 J0 W(0,d,E5);
#ifdef I
MB W(1,c,I3);
#endif
#if defined(BB)&&!defined(CB)
J0 W(2,g,L0);
#endif
MB W(3,c,H1);
#ifdef AB
O2 W(4,N,A1);
#endif
f2
#ifdef DB
S3 T3 F6(GC,g3,h3,x3,y3,n1,f0,B){M(B,h3,OC,d);M(B,y3,PC,d);M(r,f0,WB,g);M(r,f0,QB,g);M(r,f0,NB,g);
#ifdef M3
M(r,f0,XB,uint);M(r,f0,YB,uint);M(r,f0,ZB,uint);M(r,f0,AC,uint);G IB=G(XB,YB,ZB,AC);
#else
M(r,f0,IB,G);
#endif
V(E5,d);
#ifdef I
V(I3,c);
#endif
#if defined(BB)&&!defined(CB)
V(L0,g);
#endif
V(H1,c);
#ifdef AB
V(A1,N);
#endif
d l0=U0(l2(WB),OC)+NB.xy;E5=PC;
#ifdef I
if(I){I3=o8(IB.y,n.Z5);}
#endif
#ifdef BB
if(BB){
#ifndef CB
L0=Q7(l2(QB),NB.zw,l0 v5);
#else
yc(l2(QB),NB.zw,l0 v5);
#endif
}
#endif
g U=K3(l0);
#ifdef RC
U.y=-U.y;
#endif
#ifdef CB
U.z=ja(IB.w);
#endif
H1=uintBitsToFloat(IB.x);
#ifdef AB
A1=W1(IB.z);
#endif
a0(E5);
#ifdef I
a0(I3);
#endif
#if defined(BB)&&!defined(CB)
a0(L0);
#endif
a0(H1);
#ifdef AB
a0(A1);
#endif
z1(U);}
#endif
