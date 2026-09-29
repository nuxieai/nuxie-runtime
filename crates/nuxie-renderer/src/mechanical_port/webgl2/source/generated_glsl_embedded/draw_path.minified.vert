#undef H5
#ifdef CG
#define H5 true
#elif defined(AB)
#define H5 AB
#else
#define H5 false
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
O(0,M3,KB);
#else
O(0,g,UB);O(1,g,VB);
#endif
h1
#endif
m2 H0 W(0,g,f1);
#ifdef FB
H0 W(1,d,D2);
#elif!defined(CB)
#ifdef EB
MB W(1,c,i1);
#else
H0 W(2,z2,L);
#endif
MB W(3,c,B0);
#endif
#ifdef I
#ifdef FB
MB W(4,c,J3);
#else
MB W(4,E,V1);
#endif
#endif
#if defined(BB)&&!defined(CB)
H0 W(5,g,M0);
#endif
#ifdef AB
MB W(6,c,f2);
#endif
#ifdef RB
Q2 W(7,a1,f3);W(8,d,o4);
#endif
#ifdef JB
H0 W(9,Q,A2);
#endif
g2
#ifdef DB
#ifdef HD
Bd(fh)Cd(float,Qh)Dd(Rh)
#endif
z1(FC,e0,F,B,A){
#if defined(EB)||defined(FB)
P(B,F,KB,Q);
#else
P(B,F,UB,g);P(B,F,VB,g);
#endif
U(f1,g);
#if defined(JB)
U(A2,Q);
#endif
#ifdef FB
U(D2,d);
#elif!defined(CB)
#ifdef EB
U(i1,c);
#else
U(L,z2);
#endif
U(B0,c);
#endif
#ifdef I
#ifdef FB
U(J3,c);
#else
U(V1,E);
#endif
#endif
#if defined(BB)&&!defined(CB)
U(M0,g);
#endif
#ifdef AB
U(f2,c);
#endif
#ifdef RB
U(f3,a1);U(o4,d);
#endif
bool je=false;uint l0;d m0;
#ifdef CB
K g9;
#endif
#ifdef FB
m0=Hb(KB,l0,
#ifdef CB
g9,
#endif
D2 v3);
#elif defined(EB)
m0=Ib(KB,l0
#ifdef CB
,g9
#else
,i1
#endif
v3);
#else
g M;je=!q9(UB,VB,A,l0,m0
#ifndef CB
,M
#else
,g9
#endif
v3);
#ifndef CB
#ifdef HB
L=M;
#else
L.xy=R7(M.xy);
#endif
#endif
#endif
a1 p1=P5(AD,l0);
#if!defined(FB)&&!defined(CB)
B0=r8(l0,m.d6);if((p1.x&J9)!=0u)B0=-B0;
#endif
uint R3=p1.x&0xfu;
#ifdef I
if(I){uint Sh=(R3==Z7?p1.y:p1.x)>>16;c k1=r8(Sh,m.d6);if(R3==Z7)k1=-k1;
#ifdef FB
J3=k1;
#else
V1.x=k1;
#endif
}
#endif
#ifdef AB
if(AB){f2=float((p1.x>>4)&0xfu);}
#endif
d L0=m0;
#ifdef DG
L0.y=float(m.Pg)-L0.y;
#endif
#ifdef BB
if(BB){f0 Y3=h2(J0(QB,l0*z3+2u));g H4=J0(QB,l0*z3+3u);
#ifndef CB
M0=T7(Y3,H4.xy,L0);
#else
Dc(Y3,H4.xy,L0 x5);
#endif
}
#endif
if(R3==Pb){i j=unpackUnorm4x8(p1.y);if(H5){}else{j.xyz*=j.w;}f1=g(j);}
#if defined(I)&&!defined(FB)
else if(I&&R3==Z7){c I5=r8(p1.x>>16,m.d6);V1.y=I5;}
#endif
else{f0 Th=h2(J0(QB,l0*z3));g pb=J0(QB,l0*z3+1u);d L6=R0(Th,L0)+pb.xy;f1.w=-uintBitsToFloat(p1.y);float Uh=pb.z;if(Uh>.9){f1.z=2.;}else{f1.z=pb.w;}if(R3==Qb){f1.y=.0;f1.x=L6.x;}else{f1.z=-f1.z;f1.xy=L6.xy;}}
#ifdef HD
if(HD){f1*=Rh.Qh;}
#endif
#if defined(JB)
if(JB&&(p1.x&Mf)!=0u){f0 Vh=h2(J0(QB,l0*z3+4u));g ke=J0(QB,l0*z3+5u);d f4=R0(Vh,L0)+ke.xy;A2=Q(f4.x,f4.y,1.+ke.z);}else{A2=Q(0.0,0.0,0.0);}
#endif
g V;if(!je){V=L3(m0);
#ifdef RC
V.y=-V.y;
#endif
#ifdef CB
V.z=ia(g9);
#elif defined(RB)
X R4=J0(PB,l0*4u+3u);f3=R4.xy;o4=m0+uintBitsToFloat(R4.zw);
#endif
}else{V=g(m.R2,m.R2,m.R2,m.R2);}c0(f1);
#if defined(JB)
c0(A2);
#endif
#ifdef FB
c0(D2);
#elif!defined(CB)
#ifdef EB
c0(i1);
#else
c0(L);
#endif
c0(B0);
#endif
#ifdef I
#ifdef FB
c0(J3);
#else
c0(V1);
#endif
#endif
#if defined(BB)&&!defined(CB)
c0(M0);
#endif
#ifdef AB
c0(f2);
#endif
#ifdef RB
c0(f3);c0(o4);
#endif
A1(V);}
#endif
#ifdef GB
O3 P3 e i M7(g J5,
#ifdef JB
Q qb,
#endif
float n K6){i j;if(J5.w>=.0){j=c5(J5);if(H5)j.w*=n;else j*=n;}else{float t=J5.z>.0?J5.x:length(J5.xy);t=clamp(t,.0,1.);float le=abs(J5.z);float x=le>1.?(1.-1./ma)*t+(.5/ma):(1./ma)*t+le;float Wh=-J5.w;j=o2(MD,Rb,d(x,Wh),.0);j.w*=n;if(H5){}else{j.xyz*=j.w;}}
#if defined(JB)
if(JB&&qb.z>0.0){c Xh=qb.z-1.;i G2=U6(HC,V5,qb.xy,Xh);if(H5)G2=C0(F6(G2),G2.w);j*=G2;}
#endif
return j;}
#if!defined(EB)&&!defined(FB)
e c me(z2 M H3){
#ifdef HB
if(HB&&Sb(M))return y4(M d1);else
#endif
return min(M.x,M.y);}e c ne(z2 M H3){
#if defined(HB)
if(HB&&Tb(M))return d8(M d1);else
#endif
return M.x;}e c rb(z2 M H3){if(U5(M))return me(M d1);else return ne(M d1);}e c Yh(c S4,z2 M H3){if(U5(M)){c v0=me(M d1);return max(v0,S4);}else{c v0=ne(M d1);return S4+v0;}}
#endif
#endif
