#ifdef DB
f1(j3)J(0,c,PC);g1 f1(y3)J(1,c,QC);g1 f1(m1)J(w9,f,WB);J(x9,f,SB);J(y9,f,NB);J(z9,uint,XB);J(A9,uint,YB);J(B9,uint,ZB);J(C9,uint,MC);g1
#endif
p2 H0 V(0,c,G5);
#ifdef I
MB V(1,d,K3);
#endif
#if defined(BB)&&!defined(CB)
H0 V(2,f,M0);
#endif
MB V(3,i,H1);
#ifdef AB
S2 V(4,L,A1);
#endif
h2
#ifdef DB
U3 V3 J6(FC,j3,k3,y3,z3,m1,g0,B){K(B,k3,PC,c);K(B,z3,QC,c);K(v,g0,WB,f);K(v,g0,SB,f);K(v,g0,NB,f);K(v,g0,XB,uint);K(v,g0,YB,uint);K(v,g0,ZB,uint);K(v,g0,MC,uint);T(G5,c);
#ifdef I
T(K3,d);
#endif
#if defined(BB)&&!defined(CB)
T(M0,f);
#endif
T(H1,i);
#ifdef AB
T(A1,L);
#endif
c j0=N0(I1(WB),PC)+NB.xy;G5=QC;
#ifdef I
if(I){K3=v8(YB,n.e6);}
#endif
#ifdef BB
if(BB){
#ifndef CB
M0=U7(I1(SB),NB.zw,j0 x5);
#else
Kc(I1(SB),NB.zw,j0 x5);
#endif
}
#endif
f W=M3(j0);
#ifdef SC
W.y=-W.y;
#endif
#ifdef CB
W.z=na(MC);
#endif
H1=unpackUnorm4x8(XB);
#ifdef AB
A1=Y1(ZB);
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
