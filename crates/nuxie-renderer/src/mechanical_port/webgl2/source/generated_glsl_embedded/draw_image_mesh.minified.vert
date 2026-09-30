#ifdef BB
c1(w3) K(0,c,QC);d1 c1(K3) K(1,c,RC);d1 c1(A1) K(M9,f,YB);K(N9,f,SB);K(O9,f,PB);K(P9,uint,ZB);K(Q9,uint,AC);K(R9,uint,BC);K(S9,uint,MC);K(aa,f,IC);d1
#endif
l2 E0 W(0,c,W5);
#ifdef A
KB W(1,d,Z3);
#endif
#if defined(AB)&&!defined(CB)
E0 W(2,f,S0);
#endif
KB W(3,i,Q1);
#ifdef O
a3 W(4,R,H1);
#endif
e2
#ifdef BB
k4 l4 U6(RB,w3,x3,K3,L3,A1,h0,G){L(G,x3,QC,c);L(G,L3,RC,c);L(r,h0,YB,f);L(r,h0,SB,f);L(r,h0,PB,f);L(r,h0,ZB,uint);L(r,h0,AC,uint);L(r,h0,BC,uint);L(r,h0,MC,uint);L(r,h0,IC,f);T(W5,c);
#ifdef A
T(Z3,d);
#endif
#if defined(AB)&&!defined(CB)
T(S0,f);
#endif
T(Q1,i);
#ifdef O
T(H1,R);
#endif
c k0=M0(n1(YB),QC)+PB.xy;W5=RC*IC.zw+IC.xy;
#ifdef A
if(A){Z3=m6(AC,j.U4);}
#endif
#ifdef AB
if(AB){
#ifndef CB
S0=j8(n1(SB),PB.zw,k0 Z4);
#else
Ha(n1(SB),PB.zw,k0 Z4);
#endif
}
#endif
f I=I3(k0);
#ifdef NC
I.y=-I.y;
#endif
#ifdef CB
I.z=J8(MC,0xffu);
#endif
Q1=unpackUnorm4x8(ZB);
#ifdef O
H1=P1(BC);
#endif
Z(W5);
#ifdef A
Z(Z3);
#endif
#if defined(AB)&&!defined(CB)
Z(S0);
#endif
Z(Q1);
#ifdef O
Z(H1);
#endif
w1(I);}
#endif
