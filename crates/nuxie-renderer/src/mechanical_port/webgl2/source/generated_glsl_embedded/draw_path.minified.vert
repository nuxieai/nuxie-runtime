#undef B2
#ifdef GB
#define B2 f
#else
#define B2 E
#endif
#ifdef CB
h1(g0)
#if defined(DB)||defined(FB)
K(0,R3,JB);
#else
K(0,f,UB);K(1,f,VB);
#endif
i1
#endif
q2 I0 W(0,f,X1);
#ifdef FB
I0 W(1,c,F2);
#elif!defined(BB)
#ifdef DB
MB W(1,d,j1);
#else
I0 W(2,B2,O);
#endif
MB W(3,d,D0);
#endif
#ifdef I
#ifdef FB
MB W(4,d,O3);
#else
MB W(4,E,Y1);
#endif
#endif
#if defined(AB)&&!defined(BB)
I0 W(5,f,O0);
#endif
#ifdef S
MB W(6,d,g1);
#endif
#ifdef QB
V2 W(7,N0,k3);W(8,c,v4);
#endif
#ifdef IB
I0 W(9,R,C2);
#endif
i2
#ifdef CB
#ifdef KD
Rd(Gh)Sd(float,ri)Td(si)
#endif
B1(EC,g0,F,B,v){
#if defined(DB)||defined(FB)
L(B,F,JB,R);
#else
L(B,F,UB,f);L(B,F,VB,f);
#endif
U(X1,f);
#if defined(IB)
U(C2,R);
#endif
#ifdef FB
U(F2,c);
#elif!defined(BB)
#ifdef DB
U(j1,d);
#else
U(O,B2);
#endif
U(D0,d);
#endif
#ifdef I
#ifdef FB
U(O3,d);
#else
U(Y1,E);
#endif
#endif
#if defined(AB)&&!defined(BB)
U(O0,f);
#endif
#ifdef S
U(g1,d);
#endif
#ifdef QB
U(k3,N0);U(v4,c);
#endif
bool Be=false;uint o0;c l0;
#ifdef BB
N k9;
#endif
#ifdef FB
l0=Ob(JB,o0,
#ifdef BB
k9,
#endif
F2 A3);
#elif defined(DB)
l0=Pb(JB,o0
#ifdef BB
,k9
#else
,j1
#endif
A3);
#else
f P;Be=!x9(UB,VB,v,o0,l0
#ifndef BB
,P
#else
,k9
#endif
A3);
#ifndef BB
#ifdef GB
O=P;
#else
O.xy=U7(P.xy);
#endif
#endif
#endif
N0 r1=U5(CD,o0);
#if!defined(FB)&&!defined(BB)
D0=x8(o0,j.g6);if((r1.x&N9)!=0u)D0=-D0;
#endif
uint X3=r1.x&0xfu;
#ifdef I
if(I){uint ti=(X3==d8?r1.y:r1.x)>>16;d m1=x8(ti,j.g6);if(X3==d8)m1=-m1;
#ifdef FB
O3=m1;
#else
Y1.x=m1;
#endif
}
#endif
#ifdef S
if(S){g1=float((r1.x>>4)&0xfu);}
#endif
c v0=l0;
#ifdef ME
if(j.Rb!=0u){v0.y=float(j.Sb)-v0.y;}
#endif
#ifdef AB
if(AB){e0 e4=L1(L0(PB,o0*E3+2u));f L4=L0(PB,o0*E3+3u);
#ifndef BB
O0=W7(e4,L4.xy,v0);
#else
Sc(e4,L4.xy,v0 C5);
#endif
}
#endif
if(X3==ac){X1=f(unpackUnorm4x8(r1.y));}
#if defined(I)&&!defined(FB)
else if(I&&X3==d8){d M5=x8(r1.x>>16,j.g6);Y1.y=M5;}
#endif
else{e0 ui=L1(L0(PB,o0*E3));f Ce=L0(PB,o0*E3+1u);X1=Tb(v0,ui,Ce.xy,float(X3),Ce.zw,uintBitsToFloat(r1.y));X1.w=-X1.w;}
#ifdef KD
if(KD){X1*=si.ri;}
#endif
#if defined(IB)
if(IB&&(r1.x&lg)!=0u){e0 vi=L1(L0(PB,o0*E3+4u));f De=L0(PB,o0*E3+5u);c k4=P0(vi,v0)+De.xy;C2=R(k4.x,k4.y,1.+De.z);}else{C2=R(0.0,0.0,0.0);}
#endif
f X;if(!Be){X=Q3(l0);
#ifdef RC
X.y=-X.y;
#endif
#ifdef BB
X.z=qa(k9);
#elif defined(QB)
Y V4=L0(OB,o0*4u+3u);k3=V4.xy;v4=l0+uintBitsToFloat(V4.zw);
#endif
}else{X=f(j.W2,j.W2,j.W2,j.W2);}c0(X1);
#if defined(IB)
c0(C2);
#endif
#ifdef FB
c0(F2);
#elif!defined(BB)
#ifdef DB
c0(j1);
#else
c0(O);
#endif
c0(D0);
#endif
#ifdef I
#ifdef FB
c0(O3);
#else
c0(Y1);
#endif
#endif
#if defined(AB)&&!defined(BB)
c0(O0);
#endif
#ifdef S
c0(g1);
#endif
#ifdef QB
c0(k3);c0(v4);
#endif
C1(X);}
#endif
#ifdef EB
U3 V3 e i O7(
#ifdef IB
R xb,
#endif
#ifdef S
N p3,
#endif
f W4 O6){
#ifdef S
bool d5=S&&p3!=B4;
#else
const bool d5=false;
#endif
i k;if(W4.w>=.0){k=g5(W4);}else{W4.w=-W4.w;d Q9=S3(fract(W4.w)*(256./255.));W4.w=floor(W4.w)*j.ec+j.fc;c W9=kc(W4);k=j2(DD,P9,W9,.0);if(!d5){k.xyz*=k.w;k.w*=Q9;}}
#if defined(IB)
if(IB&&xb.z>0.0){d wi=xb.z-1.;i k2=X6(GC,Z5,xb.xy,wi);if(d5)k2=E0(I6(k2),k2.w);k*=k2;}
#endif
return k;}
#if!defined(DB)&&!defined(FB)
e d Ee(B2 P L3){
#ifdef GB
if(GB&&gc(P))return C4(P e1);else
#endif
return min(P.x,P.y);}e d Fe(B2 P L3){
#if defined(GB)
if(GB&&hc(P))return g8(P e1);else
#endif
return P.x;}e d yb(B2 P L3){if(Y5(P))return Ee(P e1);else return Fe(P e1);}e d xi(d X4,B2 P L3){if(Y5(P)){d y0=Ee(P e1);return max(y0,X4);}else{d y0=Fe(P e1);return X4+y0;}}
#endif
#endif
