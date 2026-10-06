#ifdef BB
c1(x3) K(0,c,PC);d1 c1(K3) K(1,c,QC);d1 c1(B1) K(M9,e,YB);K(N9,e,SB);K(O9,e,PB);K(P9,uint,ZB);K(Q9,uint,AC);K(R9,uint,BC);K(S9,uint,LC);K(aa,e,HC);d1
#endif
l2 E0 V(0,c,V5);
#ifdef A
KB V(1,d,Z3);
#endif
#if defined(AB)&&!defined(CB)
E0 V(2,e,R0);
#endif
KB V(3,i,R1);
#ifdef N
Z2 V(4,Q,I1);
#endif
e2
#ifdef BB
k4 l4 T6(RB,x3,y3,K3,L3,B1,h0,G){L(G,y3,PC,c);L(G,L3,QC,c);L(r,h0,YB,e);L(r,h0,SB,e);L(r,h0,PB,e);L(r,h0,ZB,uint);L(r,h0,AC,uint);L(r,h0,BC,uint);L(r,h0,LC,uint);L(r,h0,HC,e);T(V5,c);
#ifdef A
T(Z3,d);
#endif
#if defined(AB)&&!defined(CB)
T(R0,e);
#endif
T(R1,i);
#ifdef N
T(I1,Q);
#endif
c k0=M0(n1(YB),PC)+PB.xy;V5=QC*HC.zw+HC.xy;
#ifdef A
if(A){Z3=l6(AC,j.U4);}
#endif
#ifdef AB
if(AB){
#ifndef CB
R0=h8(n1(SB),PB.zw,k0 Z4);
#else
Ha(n1(SB),PB.zw,k0 Z4);
#endif
}
#endif
e I=I3(k0);
#ifdef MC
I.y=-I.y;
#endif
#ifdef CB
I.z=H8(LC,0xffu);
#endif
R1=unpackUnorm4x8(ZB);
#ifdef N
I1=Q1(BC);
#endif
Z(V5);
#ifdef A
Z(Z3);
#endif
#if defined(AB)&&!defined(CB)
Z(R0);
#endif
Z(R1);
#ifdef N
Z(I1);
#endif
x1(I);}
#endif
