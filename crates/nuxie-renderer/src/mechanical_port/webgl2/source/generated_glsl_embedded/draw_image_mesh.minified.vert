#ifdef BB
d1(C3) K(0,c,OC);e1 d1(T3) K(1,c,PC);e1 d1(C1) K(qa,f,ZB);K(ra,f,SB);K(sa,f,PB);K(ta,uint,AC);K(ua,uint,BC);K(va,uint,CC);K(wa,uint,LC);K(Ea,f,HC);e1
#endif
v2 F0 W(0,c,d6);
#ifdef N
MB W(1,d,f4);
#endif
#if defined(AB)&&!defined(CB)
F0 W(2,f,V0);
#endif
MB W(3,i,T1);
#ifdef H
g3 W(4,P,J1);
#endif
k2
#ifdef BB
q4 r4 g7(RB,C3,D3,T3,i3,C1,j0,F){L(F,D3,OC,c);L(F,i3,PC,c);L(r,j0,ZB,f);L(r,j0,SB,f);L(r,j0,PB,f);L(r,j0,AC,uint);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,LC,uint);L(r,j0,HC,f);V(d6,c);
#ifdef N
V(f4,d);
#endif
#if defined(AB)&&!defined(CB)
V(V0,f);
#endif
V(T1,i);
#ifdef H
V(J1,P);
#endif
c i0=B0(o1(ZB),OC)+PB.xy;d6=PC*HC.zw+HC.xy;
#ifdef N
if(N){f4=g9(BC,j.w6);}
#endif
#ifdef AB
if(AB){
#ifndef CB
V0=D8(o1(SB),PB.zw,i0 h5);
#else
kb(o1(SB),PB.zw,i0 h5);
#endif
}
#endif
f I=R3(i0);
#ifdef MC
I.y=-I.y;
#endif
#ifdef CB
I.z=h9(LC,0xffu);
#endif
T1=unpackUnorm4x8(AC);
#ifdef H
J1=S1(CC);
#endif
Z(d6);
#ifdef N
Z(f4);
#endif
#if defined(AB)&&!defined(CB)
Z(V0);
#endif
Z(T1);
#ifdef H
Z(J1);
#endif
x1(I);}
#endif
