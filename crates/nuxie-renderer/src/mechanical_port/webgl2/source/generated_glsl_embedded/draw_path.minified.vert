#undef C6
#ifdef AB
#define C6 AB
#else
#define C6 false
#endif
#undef A2
#ifdef HB
#define A2 f
#else
#define A2 E
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
H0 V(1,c,E2);
#elif!defined(CB)
#ifdef EB
MB V(1,d,h1);
#else
H0 V(2,A2,M);
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
S2 V(7,a1,g3);V(8,c,p4);
#endif
#ifdef JB
H0 V(9,Q,B2);
#endif
h2
#ifdef DB
#ifdef LD
Ld(zh)Md(float,ki)Nd(li)
#endif
y1(FC,f0,F,B,v){
#if defined(EB)||defined(GB)
K(B,F,KB,Q);
#else
K(B,F,UB,f);K(B,F,VB,f);
#endif
T(V1,f);
#if defined(JB)
T(B2,Q);
#endif
#ifdef GB
T(E2,c);
#elif!defined(CB)
#ifdef EB
T(h1,d);
#else
T(M,A2);
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
bool ve=false;uint m0;c j0;
#ifdef CB
L i9;
#endif
#ifdef GB
j0=Lb(KB,m0,
#ifdef CB
i9,
#endif
E2 x3);
#elif defined(EB)
j0=Mb(KB,m0
#ifdef CB
,i9
#else
,h1
#endif
x3);
#else
f N;ve=!w9(UB,VB,v,m0,j0
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
a1 o1=P5(DD,m0);
#if!defined(GB)&&!defined(CB)
C0=v8(m0,l.d6);if((o1.x&M9)!=0u)C0=-C0;
#endif
uint T3=o1.x&0xfu;
#ifdef I
if(I){uint mi=(T3==a8?o1.y:o1.x)>>16;d j1=v8(mi,l.d6);if(T3==a8)j1=-j1;
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
c r0=j0;
#ifdef NE
if(l.Ob!=0u){r0.y=float(l.Pb)-r0.y;}
#endif
#ifdef BB
if(BB){d0 a4=I1(K0(QB,m0*B3+2u));f H4=K0(QB,m0*B3+3u);
#ifndef CB
M0=U7(a4,H4.xy,r0);
#else
Mc(a4,H4.xy,r0 x5);
#endif
}
#endif
if(T3==Xb){i j=unpackUnorm4x8(o1.y);if(C6){}else{j.xyz*=j.w;}V1=f(j);}
#if defined(I)&&!defined(GB)
else if(I&&T3==a8){d H5=v8(o1.x>>16,l.d6);W1.y=H5;}
#endif
else{d0 ni=I1(K0(QB,m0*B3));f we=K0(QB,m0*B3+1u);V1=Qb(r0,ni,we.xy,float(T3),we.zw,uintBitsToFloat(o1.y));V1.w=-V1.w;}
#ifdef LD
if(LD){V1*=li.ki;}
#endif
#if defined(JB)
if(JB&&(o1.x&eg)!=0u){d0 oi=I1(K0(QB,m0*B3+4u));f xe=K0(QB,m0*B3+5u);c h4=N0(oi,r0)+xe.xy;B2=Q(h4.x,h4.y,1.+xe.z);}else{B2=Q(0.0,0.0,0.0);}
#endif
f W;if(!ve){W=N3(j0);
#ifdef SC
W.y=-W.y;
#endif
#ifdef CB
W.z=oa(i9);
#elif defined(RB)
X R4=K0(PB,m0*4u+3u);g3=R4.xy;p4=j0+uintBitsToFloat(R4.zw);
#endif
}else{W=f(l.T2,l.T2,l.T2,l.T2);}a0(V1);
#if defined(JB)
a0(B2);
#endif
#ifdef GB
a0(E2);
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
Q ub,
#endif
float o M6){i j;if(N7.w>=.0){j=a5(N7);if(C6)j.w*=o;else j*=o;}else{N7.w=-N7.w;c U9=ec(N7);j=i2(ED,O9,U9,.0);j.w*=o;if(C6){}else{j.xyz*=j.w;}}
#if defined(JB)
if(JB&&ub.z>0.0){d pi=ub.z-1.;i j2=V6(HC,V5,ub.xy,pi);if(C6)j2=D0(G6(j2),j2.w);j*=j2;}
#endif
return j;}
#if!defined(EB)&&!defined(GB)
e d ye(A2 N I3){
#ifdef HB
if(HB&&ac(N))return y4(N d1);else
#endif
return min(N.x,N.y);}e d ze(A2 N I3){
#if defined(HB)
if(HB&&bc(N))return e8(N d1);else
#endif
return N.x;}e d vb(A2 N I3){if(U5(N))return ye(N d1);else return ze(N d1);}e d qi(d S4,A2 N I3){if(U5(N)){d x0=ye(N d1);return max(x0,S4);}else{d x0=ze(N d1);return S4+x0;}}
#endif
#endif
