#undef Q2
#ifdef HB
#define Q2 e
#else
#define Q2 D
#endif
#ifdef BB
f1(f0)
#if defined(DB)||defined(FB)
K(0,h4,LB);
#else
K(0,e,XB);K(1,e,YB);
#endif
g1
#endif
w2 F0 X(0,e,P0);
#ifdef FB
F0 X(1,c,T2);
#elif!defined(CB)
#ifdef DB
MB X(1,d,o1);
#else
F0 X(2,Q2,S);
#endif
MB X(3,d,G0);
#endif
#ifdef N
#ifdef FB
MB X(4,d,e4);
#else
MB X(4,D,j2);
#endif
#endif
#if defined(AB)&&!defined(CB)
F0 X(5,e,W0);
#endif
#ifdef H
MB X(6,d,Q0);
#endif
#ifdef QB
g3 X(7,S0,y3);X(8,c,J4);
#endif
#ifdef GB
F0 X(9,M,V0);
#endif
l2
#ifdef BB
x1(RB,f0,B,F,r){
#if defined(DB)||defined(FB)
L(F,B,LB,M);
#else
L(F,B,XB,e);L(F,B,YB,e);
#endif
V(P0,e);
#if defined(GB)
V(V0,M);
#endif
#ifdef FB
V(T2,c);
#elif!defined(CB)
#ifdef DB
V(o1,d);
#else
V(S,Q2);
#endif
V(G0,d);
#endif
#ifdef N
#ifdef FB
V(e4,d);
#else
V(j2,D);
#endif
#endif
#if defined(AB)&&!defined(CB)
V(W0,e);
#endif
#ifdef H
V(Q0,d);
#endif
#ifdef QB
V(y3,S0);V(J4,c);
#endif
bool uf=false;uint c0;c i0;
#ifdef CB
P B6;
#endif
#ifdef FB
i0=Qc(LB,c0,
#ifdef CB
B6,
#endif
T2 P3);
#elif defined(DB)
i0=Rc(LB,c0
#ifdef CB
,B6
#else
,o1
#endif
P3);
#else
e T;uf=!ka(XB,YB,r,c0,i0
#ifndef CB
,T
#else
,B6
#endif
P3);
#ifndef CB
#ifdef HB
S=T;
#else
S.xy=z8(T.xy);
#endif
#endif
#endif
S0 T0=q5(WC,c0);
#if!defined(FB)&&!defined(CB)
G0=c9(c0,j.p6);if((T0.x&Da)!=0u) G0=-G0;
#endif
uint j3=T0.x&0xfu;
#ifdef N
if(N){uint Kj=(j3==H8?T0.y:T0.x)>>16;d z1=c9(Kj,j.p6);if(j3==H8) z1=-z1;
#ifdef FB
e4=z1;
#else
j2.x=z1;
#endif
}
#endif
#ifdef H
if(H){Q0=float((T0.x>>4)&0xfu);}
#endif
c l0=i0;
#ifdef SD
if(j.wa!=0u){l0.y=float(j.xa)-l0.y;}
#endif
#ifdef AB
if(AB){W H3=p1(p0(JB,c0*n2+2u));e W3=p0(JB,c0*n2+3u);
#ifndef CB
W0=B8(H3,W3.xy,l0);
#else
fb(H3,W3.xy,l0 e5);
#endif
}
#endif
if(j3==Ea){P0=e(unpackUnorm4x8(T0.y));}
#if defined(N)&&!defined(FB)
else if(N&&j3==H8){d a6=c9(T0.x>>16,j.p6);j2.y=a6;}
#endif
else{W Bb=p1(p0(JB,c0*n2));e P7=p0(JB,c0*n2+1u);P0=Tc(l0,Bb,P7.xy,float(j3),P7.zw,uintBitsToFloat(T0.y));P0.w=-P0.w;}
#if defined(GB)
if(GB&&(T0.x&Xd)!=0u){W Cb=p1(p0(JB,c0*n2+4u));e Q7=p0(JB,c0*n2+5u);c r3=y0(Cb,l0)+Q7.xy;float vf=1.+Q7.z;if((T0.x&rh)!=0u){uint g4=(T0.x&th)>>sh;vf=-(1.+float(g4));}V0=M(r3.x,r3.y,vf);}else{V0=M(0.0,0.0,0.0);}
#endif
e I;if(!uf){I=Q3(i0);
#ifdef MC
I.y=-I.y;
#endif
#ifdef CB
I.z=d9(B6,0xffu);
#elif defined(QB)
O k5=p0(KB,c0*4u+3u);y3=k5.xy;J4=i0+uintBitsToFloat(k5.zw);
#endif
}else{I=e(j.h3,j.h3,j.h3,j.h3);}Z(P0);
#if defined(GB)
Z(V0);
#endif
#ifdef FB
Z(T2);
#elif!defined(CB)
#ifdef DB
Z(o1);
#else
Z(S);
#endif
Z(G0);
#endif
#ifdef N
#ifdef FB
Z(e4);
#else
Z(j2);
#endif
#endif
#if defined(AB)&&!defined(CB)
Z(W0);
#endif
#ifdef H
Z(Q0);
#endif
#ifdef QB
Z(y3);Z(J4);
#endif
y1(I);}
#endif
#ifdef EB
k4 l4 f d Lj(i zc,uint g4){d wf=dot(zc.xyz,a1(.30,.59,.11));if(g4==uh) return zc.w;if(g4==vh) return 1.-zc.w;if(g4==wh) return wf;return 1.-wf;}f i p8(
#ifdef GB
M q8,
#endif
#ifdef H
P X1,
#endif
e l5 f7){
#ifdef H
bool F2=H&&X1!=T3;
#else
const bool F2=false;
#endif
i n;if(l5.w>=.0){n=T4(l5);}else{l5.w=-l5.w;d Ha=i4(fract(l5.w)*(256./255.));l5.w=floor(l5.w)*j.cd+j.g7;c Na=hd(l5);n=o2(YC,I8,Na,.0);if(!F2){n.xyz*=n.w;n.w*=Ha;}}
#if defined(GB)
if(GB&&q8.z<0.0){return A5(TB,S4,q8.xy,J0(.0));}if(GB&&q8.z>0.0){d Fb=q8.z-1.;i O1=A5(TB,S4,q8.xy,Fb);if(F2) O1=H0(f6(O1),O1.w);n*=O1;}
#endif
return n;}
#if!defined(DB)&&!defined(FB)
f d xf(Q2 T a4){
#ifdef HB
if(HB&&dd(T)) return R4(T n1);else
#endif
return min(T.x,T.y);}f d yf(Q2 T a4){
#if defined(HB)
if(HB&&ed(T)) return L8(T n1);else
#endif
return T.x;}f d Ac(Q2 T a4){if(l6(T)) return xf(T n1);else return yf(T n1);}f d Mj(d m5,Q2 T a4){if(l6(T)){d B0=xf(T n1);return max(B0,m5);}else{d B0=yf(T n1);return m5+B0;}}
#endif
#endif
