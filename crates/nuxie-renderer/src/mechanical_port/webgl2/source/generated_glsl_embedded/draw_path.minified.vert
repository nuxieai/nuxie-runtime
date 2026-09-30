#undef B2
#ifdef GB
#define B2 f
#else
#define B2 E
#endif
#ifdef CB
h1(g0)
#if defined(DB)||defined(FB)
J(0,R3,JB);
#else
J(0,f,TB);J(1,f,UB);
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
I0 W(2,B2,M);
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
I0 W(5,f,N0);
#endif
#ifdef S
MB W(6,d,g1);
#endif
#ifdef QB
T2 W(7,c1,i3);W(8,c,r4);
#endif
#ifdef IB
I0 W(9,R,C2);
#endif
i2
#ifdef CB
#ifdef KD
Md(Ah)Nd(float,li)Od(mi)
#endif
A1(EC,g0,F,B,v){
#if defined(DB)||defined(FB)
K(B,F,JB,R);
#else
K(B,F,TB,f);K(B,F,UB,f);
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
U(M,B2);
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
U(N0,f);
#endif
#ifdef S
U(g1,d);
#endif
#ifdef QB
U(i3,c1);U(r4,c);
#endif
bool we=false;uint n0;c k0;
#ifdef BB
L k9;
#endif
#ifdef FB
k0=Mb(JB,n0,
#ifdef BB
k9,
#endif
F2 A3);
#elif defined(DB)
k0=Nb(JB,n0
#ifdef BB
,k9
#else
,j1
#endif
A3);
#else
f N;we=!x9(TB,UB,v,n0,k0
#ifndef BB
,N
#else
,k9
#endif
A3);
#ifndef BB
#ifdef GB
M=N;
#else
M.xy=U7(N.xy);
#endif
#endif
#endif
c1 q1=T5(CD,n0);
#if!defined(FB)&&!defined(BB)
D0=x8(n0,j.f6);if((q1.x&N9)!=0u)D0=-D0;
#endif
uint W3=q1.x&0xfu;
#ifdef I
if(I){uint ni=(W3==d8?q1.y:q1.x)>>16;d l1=x8(ni,j.f6);if(W3==d8)l1=-l1;
#ifdef FB
O3=l1;
#else
Y1.x=l1;
#endif
}
#endif
#ifdef S
if(S){g1=float((q1.x>>4)&0xfu);}
#endif
c v0=k0;
#ifdef ME
if(j.Pb!=0u){v0.y=float(j.Qb)-v0.y;}
#endif
#ifdef AB
if(AB){e0 d4=K1(L0(PB,n0*E3+2u));f K4=L0(PB,n0*E3+3u);
#ifndef BB
N0=W7(d4,K4.xy,v0);
#else
Nc(d4,K4.xy,v0 B5);
#endif
}
#endif
if(W3==Yb){X1=f(unpackUnorm4x8(q1.y));}
#if defined(I)&&!defined(FB)
else if(I&&W3==d8){d L5=x8(q1.x>>16,j.f6);Y1.y=L5;}
#endif
else{e0 oi=K1(L0(PB,n0*E3));f xe=L0(PB,n0*E3+1u);X1=Rb(v0,oi,xe.xy,float(W3),xe.zw,uintBitsToFloat(q1.y));X1.w=-X1.w;}
#ifdef KD
if(KD){X1*=mi.li;}
#endif
#if defined(IB)
if(IB&&(q1.x&fg)!=0u){e0 pi=K1(L0(PB,n0*E3+4u));f ye=L0(PB,n0*E3+5u);c j4=O0(pi,v0)+ye.xy;C2=R(j4.x,j4.y,1.+ye.z);}else{C2=R(0.0,0.0,0.0);}
#endif
f X;if(!we){X=Q3(k0);
#ifdef RC
X.y=-X.y;
#endif
#ifdef BB
X.z=pa(k9);
#elif defined(QB)
Y U4=L0(OB,n0*4u+3u);i3=U4.xy;r4=k0+uintBitsToFloat(U4.zw);
#endif
}else{X=f(j.U2,j.U2,j.U2,j.U2);}c0(X1);
#if defined(IB)
c0(C2);
#endif
#ifdef FB
c0(F2);
#elif!defined(BB)
#ifdef DB
c0(j1);
#else
c0(M);
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
c0(N0);
#endif
#ifdef S
c0(g1);
#endif
#ifdef QB
c0(i3);c0(r4);
#endif
B1(X);}
#endif
#ifdef EB
T3 U3 e i N7(
#ifdef IB
R vb,
#endif
#ifdef S
L n3,
#endif
f O7 N6){
#ifdef S
bool c5=S&&n3!=A4;
#else
const bool c5=false;
#endif
i k;if(O7.w>=.0){k=f5(O7);}else{O7.w=-O7.w;c V9=fc(O7);k=j2(DD,P9,V9,.0);if(!c5)k.xyz*=k.w;}
#if defined(IB)
if(IB&&vb.z>0.0){d qi=vb.z-1.;i k2=W6(GC,Y5,vb.xy,qi);if(c5)k2=E0(H6(k2),k2.w);k*=k2;}
#endif
return k;}
#if!defined(DB)&&!defined(FB)
e d ze(B2 N L3){
#ifdef GB
if(GB&&bc(N))return B4(N e1);else
#endif
return min(N.x,N.y);}e d Ae(B2 N L3){
#if defined(GB)
if(GB&&cc(N))return g8(N e1);else
#endif
return N.x;}e d wb(B2 N L3){if(X5(N))return ze(N e1);else return Ae(N e1);}e d ri(d V4,B2 N L3){if(X5(N)){d y0=ze(N e1);return max(y0,V4);}else{d y0=Ae(N e1);return V4+y0;}}
#endif
#endif
