#ifdef CB
h1(n3)K(0,c,OC);i1 h1(C3)K(1,c,PC);i1 h1(p1)K(y9,f,WB);K(z9,f,RB);K(A9,f,NB);K(B9,uint,XB);K(C9,uint,YB);K(D9,uint,ZB);K(E9,uint,LC);i1
#endif
q2 I0 W(0,c,L5);
#ifdef I
MB W(1,d,O3);
#endif
#if defined(AB)&&!defined(BB)
I0 W(2,f,O0);
#endif
MB W(3,i,K1);
#ifdef S
V2 W(4,N,D1);
#endif
i2
#ifdef CB
Y3 Z3 L6(EC,n3,o3,C3,D3,p1,h0,B){L(B,o3,OC,c);L(B,D3,PC,c);L(v,h0,WB,f);L(v,h0,RB,f);L(v,h0,NB,f);L(v,h0,XB,uint);L(v,h0,YB,uint);L(v,h0,ZB,uint);L(v,h0,LC,uint);U(L5,c);
#ifdef I
U(O3,d);
#endif
#if defined(AB)&&!defined(BB)
U(O0,f);
#endif
U(K1,i);
#ifdef S
U(D1,N);
#endif
c l0=P0(L1(WB),OC)+NB.xy;L5=PC;
#ifdef I
if(I){O3=x8(YB,j.g6);}
#endif
#ifdef AB
if(AB){
#ifndef BB
O0=W7(L1(RB),NB.zw,l0 C5);
#else
Rc(L1(RB),NB.zw,l0 C5);
#endif
}
#endif
f X=Q3(l0);
#ifdef RC
X.y=-X.y;
#endif
#ifdef BB
X.z=qa(LC);
#endif
K1=unpackUnorm4x8(XB);
#ifdef S
D1=a2(ZB);
#endif
c0(L5);
#ifdef I
c0(O3);
#endif
#if defined(AB)&&!defined(BB)
c0(O0);
#endif
c0(K1);
#ifdef S
c0(D1);
#endif
C1(X);}
#endif
