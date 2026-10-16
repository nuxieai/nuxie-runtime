#undef Q2
#ifdef HB
#define Q2 f
#else
#define Q2 D
#endif
#ifdef BB
e1(f0)
#if defined(DB)||defined(FB)
K(0,i4,LB);
#else
K(0,f,XB);K(1,f,YB);
#endif
f1
#endif
v2 F0 W(0,f,P0);
#ifdef FB
F0 W(1,c,T2);
#elif!defined(CB)
#ifdef DB
MB W(1,d,o1);
#else
F0 W(2,Q2,S);
#endif
MB W(3,d,G0);
#endif
#ifdef N
#ifdef FB
MB W(4,d,f4);
#else
MB W(4,D,i2);
#endif
#endif
#if defined(AB)&&!defined(CB)
F0 W(5,f,V0);
#endif
#ifdef H
MB W(6,d,Q0);
#endif
#ifdef QB
h3 W(7,R0,A3);W(8,c,L4);
#endif
#ifdef GB
F0 W(9,M,U0);
#endif
k2
#ifdef BB
y1(RB,f0,B,F,r){
#if defined(DB)||defined(FB)
L(F,B,LB,M);
#else
L(F,B,XB,f);L(F,B,YB,f);
#endif
V(P0,f);
#if defined(GB)
V(U0,M);
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
V(f4,d);
#else
V(i2,D);
#endif
#endif
#if defined(AB)&&!defined(CB)
V(V0,f);
#endif
#ifdef H
V(Q0,d);
#endif
#ifdef QB
V(A3,R0);V(L4,c);
#endif
bool wf=false;uint c0;c i0;
#ifdef CB
P E6;
#endif
#ifdef FB
i0=Xc(LB,c0,
#ifdef CB
E6,
#endif
T2 Q3);
#elif defined(DB)
i0=Yc(LB,c0
#ifdef CB
,E6
#else
,o1
#endif
Q3);
#else
f T;wf=!ra(XB,YB,r,c0,i0
#ifndef CB
,T
#else
,E6
#endif
Q3);
#ifndef CB
#ifdef HB
S=T;
#else
S.xy=E8(T.xy);
#endif
#endif
#endif
R0 S0=x5(XC,c0);
#if!defined(FB)&&!defined(CB)
G0=j9(c0,j.w6);if((S0.x&La)!=0u) G0=-G0;
#endif
uint w2=S0.x&0xfu;
#ifdef N
if(N){uint Mj=(w2==M8?S0.y:S0.x)>>16;d A1=j9(Mj,j.w6);if(w2==M8) A1=-A1;
#ifdef FB
f4=A1;
#else
i2.x=A1;
#endif
}
#endif
#ifdef H
if(H){Q0=float((S0.x>>4)&0xfu);}
#endif
c l0=i0;
#ifdef UD
if(j.Da!=0u){l0.y=float(j.Ea)-l0.y;}
#endif
#ifdef AB
if(AB){X J3=p1(p0(JB,c0*m2+2u));f X3=p0(JB,c0*m2+3u);
#ifndef CB
V0=G8(J3,X3.xy,l0);
#else
ob(J3,X3.xy,l0 h5);
#endif
}
#endif
if(w2==Ma){P0=f(unpackUnorm4x8(S0.y));}
#if defined(N)&&!defined(FB)
else if(N&&w2==M8){d e6=j9(S0.x>>16,j.w6);i2.y=e6;}
#endif
else{X Jb=p1(p0(JB,c0*m2));f U7=p0(JB,c0*m2+1u);float o4=uintBitsToFloat(S0.y);P0=Fa(l0,Jb,U7.xy,U7.zw,w2,o4,1.0);}
#if defined(GB)
if(GB&&(S0.x&ae)!=0u){X Kb=p1(p0(JB,c0*m2+4u));f V7=p0(JB,c0*m2+5u);c w3=C0(Kb,l0)+V7.xy;float xf=1.+V7.z;if((S0.x&xh)!=0u){uint h4=(S0.x&zh)>>yh;xf=-(1.+float(h4));}U0=M(w3.x,w3.y,xf);}else{U0=M(0.0,0.0,0.0);}
#endif
f I;if(!wf){I=R3(i0);
#ifdef NC
I.y=-I.y;
#endif
#ifdef CB
I.z=k9(E6,0xffu);
#elif defined(QB)
O o5=p0(KB,c0*4u+3u);A3=o5.xy;L4=i0+uintBitsToFloat(o5.zw);
#endif
}else{I=f(j.i3,j.i3,j.i3,j.i3);}a0(P0);
#if defined(GB)
a0(U0);
#endif
#ifdef FB
a0(T2);
#elif!defined(CB)
#ifdef DB
a0(o1);
#else
a0(S);
#endif
a0(G0);
#endif
#ifdef N
#ifdef FB
a0(f4);
#else
a0(i2);
#endif
#endif
#if defined(AB)&&!defined(CB)
a0(V0);
#endif
#ifdef H
a0(Q0);
#endif
#ifdef QB
a0(A3);a0(L4);
#endif
z1(I);}
#endif
#ifdef EB
k4 l4 e d Nj(i Gc,uint h4){d yf=dot(Gc.xyz,Z0(.30,.59,.11));if(h4==Ah) return Gc.w;if(h4==Bh) return 1.-Gc.w;if(h4==Ch) return yf;return 1.-yf;}e i x8(
#ifdef GB
M y8,
#endif
#ifdef H
P W1,
#endif
f ia n7){
#ifdef H
bool G2=H&&W1!=U3;
#else
const bool G2=false;
#endif
i k;if(ia.w>=.0){k=V4(ia);}else{c Ua=Va(ia,j.O8,j.P8);k=n2(ZC,Q8,Ua,.0);if(!G2){d o4=me(ia);k.xyz*=k.w;k.w*=o4;}}
#if defined(GB)
if(GB&&y8.z<0.0){return E5(TB,U4,y8.xy,J0(.0));}if(GB&&y8.z>0.0){d Lb=y8.z-1.;i O1=E5(TB,U4,y8.xy,Lb);if(G2) O1=H0(i6(O1),O1.w);k*=O1;}
#endif
return k;}
#if!defined(DB)&&!defined(FB)
e d zf(Q2 T c4){
#ifdef HB
if(HB&&gd(T)) return T4(T n1);else
#endif
return min(T.x,T.y);}e d Af(Q2 T c4){
#if defined(HB)
if(HB&&hd(T)) return T8(T n1);else
#endif
return T.x;}e d Hc(Q2 T c4){if(o6(T)) return zf(T n1);else return Af(T n1);}e d Oj(d p5,Q2 T c4){if(o6(T)){d B0=zf(T n1);return max(B0,p5);}else{d B0=Af(T n1);return p5+B0;}}
#endif
#endif
