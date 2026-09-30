#undef I5
#ifdef IG
#define I5 true
#elif defined(AB)
#define I5 AB
#else
#define I5 false
#endif
#undef B2
#ifdef HB
#define B2 f
#else
#define B2 E
#endif
#ifdef DB
f1(f0)
#if defined(EB)||defined(GB)
J(0,O3,KB);
#else
J(0,f,UB);J(1,f,VB);
#endif
g1
#endif
p2 H0 V(0,f,V1);
#ifdef GB
H0 V(1,c,F2);
#elif!defined(CB)
#ifdef EB
MB V(1,d,h1);
#else
H0 V(2,B2,M);
#endif
MB V(3,d,C0);
#endif
#ifdef I
#ifdef GB
MB V(4,d,L3);
#else
MB V(4,E,W1);
#endif
#endif
#if defined(BB)&&!defined(CB)
H0 V(5,f,M0);
#endif
#ifdef AB
MB V(6,d,g2);
#endif
#ifdef RB
T2 V(7,a1,g3);V(8,c,p4);
#endif
#ifdef JB
H0 V(9,Q,C2);
#endif
h2
#ifdef DB
#ifdef LD
Kd(yh)Ld(float,ji)Md(ki)
#endif
y1(FC,f0,F,B,v){
#if defined(EB)||defined(GB)
K(B,F,KB,Q);
#else
K(B,F,UB,f);K(B,F,VB,f);
#endif
T(V1,f);
#if defined(JB)
T(C2,Q);
#endif
#ifdef GB
T(F2,c);
#elif!defined(CB)
#ifdef EB
T(h1,d);
#else
T(M,B2);
#endif
T(C0,d);
#endif
#ifdef I
#ifdef GB
T(L3,d);
#else
T(W1,E);
#endif
#endif
#if defined(BB)&&!defined(CB)
T(M0,f);
#endif
#ifdef AB
T(g2,d);
#endif
#ifdef RB
T(g3,a1);T(p4,c);
#endif
bool ue=false;uint m0;c j0;
#ifdef CB
L i9;
#endif
#ifdef GB
j0=Kb(KB,m0,
#ifdef CB
i9,
#endif
F2 x3);
#elif defined(EB)
j0=Lb(KB,m0
#ifdef CB
,i9
#else
,h1
#endif
x3);
#else
f N;ue=!v9(UB,VB,v,m0,j0
#ifndef CB
,N
#else
,i9
#endif
x3);
#ifndef CB
#ifdef HB
M=N;
#else
M.xy=S7(N.xy);
#endif
#endif
#endif
a1 o1=R5(DD,m0);
#if!defined(GB)&&!defined(CB)
C0=v8(m0,l.f6);if((o1.x&L9)!=0u)C0=-C0;
#endif
uint T3=o1.x&0xfu;
#ifdef I
if(I){uint li=(T3==a8?o1.y:o1.x)>>16;d j1=v8(li,l.f6);if(T3==a8)j1=-j1;
#ifdef GB
L3=j1;
#else
W1.x=j1;
#endif
}
#endif
#ifdef AB
if(AB){g2=float((o1.x>>4)&0xfu);}
#endif
c q0=j0;
#ifdef NE
if(l.Nb!=0u){q0.y=float(l.Ob)-q0.y;}
#endif
#ifdef BB
if(BB){d0 a4=I1(K0(QB,m0*B3+2u));f H4=K0(QB,m0*B3+3u);
#ifndef CB
M0=U7(a4,H4.xy,q0);
#else
Lc(a4,H4.xy,q0 y5);
#endif
}
#endif
if(T3==Wb){i j=unpackUnorm4x8(o1.y);if(I5){}else{j.xyz*=j.w;}V1=f(j);}
#if defined(I)&&!defined(GB)
else if(I&&T3==a8){d J5=v8(o1.x>>16,l.f6);W1.y=J5;}
#endif
else{d0 mi=I1(K0(QB,m0*B3));f ve=K0(QB,m0*B3+1u);V1=Pb(q0,mi,ve.xy,float(T3),ve.zw,uintBitsToFloat(o1.y));V1.w=-V1.w;}
#ifdef LD
if(LD){V1*=ki.ji;}
#endif
#if defined(JB)
if(JB&&(o1.x&dg)!=0u){d0 ni=I1(K0(QB,m0*B3+4u));f we=K0(QB,m0*B3+5u);c h4=N0(ni,q0)+we.xy;C2=Q(h4.x,h4.y,1.+we.z);}else{C2=Q(0.0,0.0,0.0);}
#endif
f W;if(!ue){W=N3(j0);
#ifdef SC
W.y=-W.y;
#endif
#ifdef CB
W.z=na(i9);
#elif defined(RB)
X R4=K0(PB,m0*4u+3u);g3=R4.xy;p4=j0+uintBitsToFloat(R4.zw);
#endif
}else{W=f(l.U2,l.U2,l.U2,l.U2);}a0(V1);
#if defined(JB)
a0(C2);
#endif
#ifdef GB
a0(F2);
#elif!defined(CB)
#ifdef EB
a0(h1);
#else
a0(M);
#endif
a0(C0);
#endif
#ifdef I
#ifdef GB
a0(L3);
#else
a0(W1);
#endif
#endif
#if defined(BB)&&!defined(CB)
a0(M0);
#endif
#ifdef AB
a0(g2);
#endif
#ifdef RB
a0(g3);a0(p4);
#endif
z1(W);}
#endif
#ifdef FB
Q3 R3 e i M7(f N7,
#ifdef JB
Q tb,
#endif
float o M6){i j;if(N7.w>=.0){j=c5(N7);if(I5)j.w*=o;else j*=o;}else{N7.w=-N7.w;c T9=dc(N7);j=i2(ED,N9,T9,.0);j.w*=o;if(I5){}else{j.xyz*=j.w;}}
#if defined(JB)
if(JB&&tb.z>0.0){d oi=tb.z-1.;i j2=V6(HC,X5,tb.xy,oi);if(I5)j2=D0(H6(j2),j2.w);j*=j2;}
#endif
return j;}
#if!defined(EB)&&!defined(GB)
e d xe(B2 N I3){
#ifdef HB
if(HB&&Zb(N))return y4(N d1);else
#endif
return min(N.x,N.y);}e d ye(B2 N I3){
#if defined(HB)
if(HB&&ac(N))return e8(N d1);else
#endif
return N.x;}e d ub(B2 N I3){if(W5(N))return xe(N d1);else return ye(N d1);}e d pi(d S4,B2 N I3){if(W5(N)){d w0=xe(N d1);return max(w0,S4);}else{d w0=ye(N d1);return S4+w0;}}
#endif
#endif
