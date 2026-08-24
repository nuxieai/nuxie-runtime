#undef F5
#ifdef YF
#define F5 true
#elif defined(AB)
#define F5 AB
#else
#define F5 false
#endif
#undef z2
#ifdef HB
#define z2 g
#else
#define z2 E
#endif
#ifdef DB
g1(e0)
#if defined(EB)||defined(FB)
L(0,L3,KB);
#else
L(0,g,UB);L(1,g,VB);
#endif
h1
#endif
k2 J0 W(0,g,f1);
#ifdef FB
J0 W(1,d,C2);
#elif!defined(CB)
#ifdef EB
MB W(1,c,i1);
#else
J0 W(2,z2,O);
#endif
MB W(3,c,B0);
#endif
#ifdef I
#ifdef FB
MB W(4,c,I3);
#else
MB W(4,E,U1);
#endif
#endif
#if defined(BB)&&!defined(CB)
J0 W(5,g,L0);
#endif
#ifdef AB
MB W(6,c,e2);
#endif
#ifdef SB
O2 W(7,a1,d3);W(8,d,l4);
#endif
f2
#ifdef DB
y1(GC,e0,F,B,r){
#if defined(EB)||defined(FB)
M(B,F,KB,c0);
#else
M(B,F,UB,g);M(B,F,VB,g);
#endif
V(f1,g);
#ifdef FB
V(C2,d);
#elif!defined(CB)
#ifdef EB
V(i1,c);
#else
V(O,z2);
#endif
V(B0,c);
#endif
#ifdef I
#ifdef FB
V(I3,c);
#else
V(U1,E);
#endif
#endif
#if defined(BB)&&!defined(CB)
V(L0,g);
#endif
#ifdef AB
V(e2,c);
#endif
#ifdef SB
V(d3,a1);V(l4,d);
#endif
bool ce=false;uint o0;d l0;
#ifdef CB
N e9;
#endif
#ifdef FB
l0=Db(KB,o0,
#ifdef CB
e9,
#endif
C2 v3);
#elif defined(EB)
l0=Eb(KB,o0
#ifdef CB
,e9
#else
,i1
#endif
v3);
#else
g P;ce=!q9(UB,VB,r,o0,l0
#ifndef CB
,P
#else
,e9
#endif
v3);
#ifndef CB
#ifdef HB
O=P;
#else
O.xy=O7(P.xy);
#endif
#endif
#endif
a1 p1=M5(AD,o0);
#if!defined(FB)&&!defined(CB)
B0=o8(o0,n.Z5);if((p1.x&K9)!=0u)B0=-B0;
#endif
uint j3=p1.x&0xfu;
#ifdef I
if(I){uint Dh=(j3==W7?p1.y:p1.x)>>16;c k1=o8(Dh,n.Z5);if(j3==W7)k1=-k1;
#ifdef FB
I3=k1;
#else
U1.x=k1;
#endif
}
#endif
#ifdef AB
if(AB){e2=float((p1.x>>4)&0xfu);}
#endif
d K0=l0;
#ifdef ZF
K0.y=float(n.Fg)-K0.y;
#endif
#ifdef BB
if(BB){g0 X3=l2(N0(RB,o0*4u+2u));g E4=N0(RB,o0*4u+3u);
#ifndef CB
L0=Q7(X3,E4.xy,K0);
#else
yc(X3,E4.xy,K0 v5);
#endif
}
#endif
if(j3==Lb){i j=unpackUnorm4x8(p1.y);if(F5){}else{j.xyz*=j.w;}f1=g(j);}
#if defined(I)&&!defined(FB)
else if(I&&j3==W7){c G5=o8(p1.x>>16,n.Z5);U1.y=G5;}
#endif
else{g0 Eh=l2(N0(RB,o0*4u));g f9=N0(RB,o0*4u+1u);d V4=U0(Eh,K0)+f9.xy;if(j3==M9||j3==Ef){f1.w=-uintBitsToFloat(p1.y);float Fh=f9.z;if(Fh>.9){f1.z=2.;}else{f1.z=f9.w;}if(j3==M9){f1.y=.0;f1.x=V4.x;}else{f1.z=-f1.z;f1.xy=V4.xy;}}else{float g9=uintBitsToFloat(p1.y);float mb=f9.z;f1=g(V4.x,V4.y,g9,-2.-mb);}}g U;if(!ce){U=K3(l0);
#ifdef RC
U.y=-U.y;
#endif
#ifdef CB
U.z=ja(e9);
#elif defined(SB)
G N4=N0(PB,o0*4u+3u);d3=N4.xy;l4=l0+uintBitsToFloat(N4.zw);
#endif
}else{U=g(n.P2,n.P2,n.P2,n.P2);}a0(f1);
#ifdef FB
a0(C2);
#elif!defined(CB)
#ifdef EB
a0(i1);
#else
a0(O);
#endif
a0(B0);
#endif
#ifdef I
#ifdef FB
a0(I3);
#else
a0(U1);
#endif
#endif
#if defined(BB)&&!defined(CB)
a0(L0);
#endif
#ifdef AB
a0(e2);
#endif
#ifdef SB
a0(d3);a0(l4);
#endif
z1(U);}
#endif
#ifdef GB
O3 P3 e i J7(g q3,float o H6){i j;if(q3.w>=.0){j=Y4(q3);if(F5)j.w*=o;else j*=o;}else if(q3.w>-1.){float t=q3.z>.0?q3.x:length(q3.xy);t=clamp(t,.0,1.);float de=abs(q3.z);float x=de>1.?(1.-1./la)*t+(.5/la):(1./la)*t+de;float Gh=-q3.w;j=n2(KD,Mb,d(x,Gh),.0);j.w*=o;if(F5){}else{j.xyz*=j.w;}}else{c mb=-q3.w-2.;j=Q6(IC,S5,q3.xy,mb);c g9=q3.z*o;if(F5)j=C0(C6(j),j.w*g9);else j*=g9;}return j;}
#if!defined(EB)&&!defined(FB)
e c ee(z2 P G3){
#ifdef HB
if(HB&&Nb(P))return v4(P d1);else
#endif
return min(P.x,P.y);}e c fe(z2 P G3){
#if defined(HB)
if(HB&&Ob(P))return Z7(P d1);else
#endif
return P.x;}e c nb(z2 P G3){if(R5(P))return ee(P d1);else return fe(P d1);}e c Hh(c O4,z2 P G3){if(R5(P)){c r0=ee(P d1);return max(r0,O4);}else{c r0=fe(P d1);return O4+r0;}}
#endif
#endif
