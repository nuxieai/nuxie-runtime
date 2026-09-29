#ifdef DB
g1(i3)O(0,d,OC);h1 g1(x3)O(1,d,PC);h1 g1(n1)O(r9,g,WB);O(v9,g,SB);O(w9,g,NB);O(x9,float,XB);O(y9,uint,YB);O(z9,uint,ZB);O(A9,uint,MC);h1
#endif
m2 H0 W(0,d,G5);
#ifdef I
MB W(1,c,J3);
#endif
#if defined(BB)&&!defined(CB)
H0 W(2,g,M0);
#endif
MB W(3,c,I1);
#ifdef AB
Q2 W(4,K,B1);
#endif
g2
#ifdef DB
T3 U3 I6(FC,i3,j3,x3,y3,n1,i0,B){P(B,j3,OC,d);P(B,y3,PC,d);P(A,i0,WB,g);P(A,i0,SB,g);P(A,i0,NB,g);P(A,i0,XB,float);P(A,i0,YB,uint);P(A,i0,ZB,uint);P(A,i0,MC,uint);U(G5,d);
#ifdef I
U(J3,c);
#endif
#if defined(BB)&&!defined(CB)
U(M0,g);
#endif
U(I1,c);
#ifdef AB
U(B1,K);
#endif
d m0=R0(h2(WB),OC)+NB.xy;G5=PC;
#ifdef I
if(I){J3=r8(YB,m.d6);}
#endif
#ifdef BB
if(BB){
#ifndef CB
M0=T7(h2(SB),NB.zw,m0 x5);
#else
Dc(h2(SB),NB.zw,m0 x5);
#endif
}
#endif
g V=L3(m0);
#ifdef RC
V.y=-V.y;
#endif
#ifdef CB
V.z=ia(MC);
#endif
I1=XB;
#ifdef AB
B1=X1(ZB);
#endif
c0(G5);
#ifdef I
c0(J3);
#endif
#if defined(BB)&&!defined(CB)
c0(M0);
#endif
c0(I1);
#ifdef AB
c0(B1);
#endif
A1(V);}
#endif
