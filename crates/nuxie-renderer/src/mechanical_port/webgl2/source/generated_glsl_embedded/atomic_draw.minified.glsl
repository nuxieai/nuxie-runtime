#ifdef PD
#ifdef BB
d1(f0) K(0,f,XB);K(1,f,YB);e1
#endif
v2
#ifdef HB
F0 W(0,f,S);
#else
F0 W(0,D,S);
#endif
g3 W(1,P,G0);k2
#ifdef BB
w1(RB,f0,B,F,r){L(F,B,XB,f);L(F,B,YB,f);
#ifdef HB
V(S,f);
#else
V(S,D);
#endif
V(G0,P);f I;uint c0;c i0;f T;if(pa(XB,YB,r,c0,i0,T Q3)){
#ifdef HB
S=T;
#else
S.xy=B8(T.xy);
#endif
G0=S1(c0);I=R3(i0);}else{I=f(j.h3,j.h3,j.h3,j.h3);}Z(S);Z(G0);x1(I);}
#endif
#endif
#if defined(DB)||defined(FB)
#ifdef BB
d1(f0) K(0,i4,LB);e1
#endif
v2
#ifdef FB
F0 W(0,c,S2);
#else
MB W(0,d,n1);
#endif
g3 W(1,P,G0);k2
#ifdef BB
w1(RB,f0,B,F,r){L(F,B,LB,M);
#ifdef FB
V(S2,c);
#else
V(n1,d);
#endif
V(G0,P);uint c0;c i0;
#ifdef FB
i0=Wc(LB,c0,S2 Q3);
#else
i0=Xc(LB,c0,n1 Q3);
#endif
G0=S1(c0);f I=R3(i0);
#ifdef FB
Z(S2);
#else
Z(n1);
#endif
Z(G0);x1(I);}
#endif
#endif
#ifdef DD
#ifdef BB
d1(f0) K(0,f,GC);e1 d1(C1) K(qa,f,ZB);K(ra,f,SB);K(sa,f,PB);K(ta,uint,AC);K(ua,uint,BC);K(va,uint,CC);K(wa,uint,LC);K(cg,f,QD);K(dg,f,RD);K(eg,f,ED);K(Yc,f,FD);e1
#endif
v2 F0 W(0,c,l2);F0 W(1,d,q5);F0 W(2,f,r5);
#ifdef AB
F0 W(3,f,V0);
#endif
MB W(4,i,T1);
#ifdef N
g3 W(5,P,S3);
#endif
#ifdef H
g3 W(6,P,J1);
#endif
k2
#ifdef BB
C8(RB,f0,B,C1,j0,F,r){L(F,B,GC,f);L(r,j0,ZB,f);L(r,j0,SB,f);L(r,j0,PB,f);L(r,j0,AC,uint);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,LC,uint);L(r,j0,QD,f);L(r,j0,RD,f);L(r,j0,ED,f);L(r,j0,FD,f);V(l2,c);V(q5,d);V(r5,f);
#ifdef AB
V(V0,f);
#endif
V(T1,i);
#ifdef N
V(S3,P);
#endif
#ifdef H
V(J1,P);
#endif
bool xa=GC.z==.0||GC.w==.0;q5=xa?.0:1.;c i0=GC.xy;X N0=o1(ZB);X f7=transpose(inverse(N0));if(!xa){float ya=O4*za(f7[1])/dot(N0[1],f7[1]);if(ya>=.5){i0.x=.5;q5*=v5(.5/ya);}else{i0.x+=ya*GC.z;}float Aa=O4*za(f7[0])/dot(N0[0],f7[0]);if(Aa>=.5){i0.y=.5;q5*=v5(.5/Aa);}else{i0.y+=Aa*GC.w;}}X fg=o1(QD);l2=B0(fg,i0)+ED.xy;i0=B0(N0,i0)+PB.xy;if(xa){c j4=B0(f7,GC.zw);j4*=za(j4)/dot(j4,j4);i0+=O4*j4;}
#ifdef AB
if(AB){V0=D8(o1(SB),PB.zw,i0);}
#endif
T1=unpackUnorm4x8(AC);
#ifdef N
S3=S1(BC);
#endif
#ifdef H
J1=S1(CC);
#endif
f I=R3(i0);c l0=i0;
#ifdef SD
if(j.Ba!=0u){l0.y=float(j.Ca)-l0.y;}
#endif
uint w2=floatBitsToUint(FD.z);if(w2!=0){X gg=o1(RD);c hg=ED.zw;r5=Da(l0,gg,hg,FD.xy,w2,1.0,1.0);}else{r5=f(.0,.0,.0,.0);}Z(l2);Z(q5);Z(r5);
#ifdef AB
Z(V0);
#endif
Z(T1);
#ifdef N
Z(S3);
#endif
#ifdef H
Z(J1);
#endif
x1(I);}
#endif
#elif defined(NB)
#ifdef BB
d1(C3) K(0,c,OC);e1 d1(T3) K(1,c,PC);e1 d1(C1) K(qa,f,ZB);K(ra,f,SB);K(sa,f,PB);K(ta,uint,AC);K(ua,uint,BC);K(va,uint,CC);K(wa,uint,LC);K(Ea,f,HC);e1
#endif
v2 F0 W(0,c,l2);
#ifdef AB
F0 W(1,f,V0);
#endif
MB W(3,i,T1);
#ifdef N
g3 W(4,P,S3);
#endif
#ifdef H
g3 W(5,P,J1);
#endif
k2
#ifdef BB
g7(RB,C3,D3,T3,i3,C1,j0,F){L(F,D3,OC,c);L(F,i3,PC,c);L(r,j0,ZB,f);L(r,j0,SB,f);L(r,j0,PB,f);L(r,j0,AC,uint);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,LC,uint);L(r,j0,HC,f);V(l2,c);
#ifdef AB
V(V0,f);
#endif
V(T1,i);
#ifdef N
V(S3,P);
#endif
#ifdef H
V(J1,P);
#endif
X N0=o1(ZB);c i0=B0(N0,OC)+PB.xy;l2=PC*HC.zw+HC.xy;
#ifdef AB
if(AB){V0=D8(o1(SB),PB.zw,i0);}
#endif
T1=unpackUnorm4x8(AC);
#ifdef N
S3=S1(BC);
#endif
#ifdef H
J1=S1(CC);
#endif
f I=R3(i0);Z(l2);
#ifdef AB
Z(V0);
#endif
Z(T1);
#ifdef N
Z(S3);
#endif
#ifdef H
Z(J1);
#endif
x1(I);}
#endif
#endif
#ifdef KF
#ifdef BB
d1(f0) e1
#endif
v2 k2
#ifdef BB
w1(RB,f0,B,F,r){g0 E2;E2.x=(F&1)==0?j.E8.x:j.E8.z;E2.y=(F&2)==0?j.E8.y:j.E8.w;f I=R3(c(E2));x1(I);}
#endif
#endif
#ifdef OE
#endif
#if defined(PE)&&!defined(U)
#endif
#ifdef EB
U1
#ifndef U
#ifdef QE
#define Fa QE
#else
#define Fa T2
#endif
#ifdef GD
P4(Fa,n0);
#else
C0(Fa,n0);
#endif
#endif
#ifdef TC
#define Q4 i
#define Ga Q0
#define F8 H0(.0)
#define Zc(E) ((E).w!=.0)
#ifdef N
#ifndef UC
C0(j3,m0);
#else
P4(j3,m0);
#endif
#endif
#else
#define Q4 uint
#define F8 0u
#define Ga j1
#define Zc(E) ((E)!=0u)
#ifdef N
p1(j3,m0);
#endif
#endif
U2(h7,R4);V1 k4 j6(ad,jg,VC);k6(bd,kg,JB);l4 e uint lg(float x){return uint(round(x*Ha+Ia));}e d G8(uint x){return v5(float(x)*cd+(-Ia*cd));}P H8(P c0){
#ifdef LF
c0=min(c0,j.mg);
#endif
return c0;}
#ifdef N
e void dd(uint y1,Q4 W0,i7(d) n){
#ifdef TC
if(all(lessThan(abs(W0.xy-unpackUnorm4x8(y1).xy),Q2(.25/255.)))) n=min(n,W0.z);else n=.0;
#else
if(y1==W0>>16) n=min(n,unpackHalf2x16(W0).x);else n=.0;
#endif
}
#endif
e void I8(uint c0,d w0,k1(i) Q
#if defined(N)&&!defined(UC)
,i7(Q4) D1
#endif
j7 m4){R0 S0=w5(VC,c0);d n=w0;if((S0.x&(ng|Ja))!=0u){n=abs(n);
#ifdef WC
if(WC&&(S0.x&Ja)!=0u){n=1.-abs(fract(n*.5)*2.+-1.);}
#endif
}n=clamp(n,J0(.0),J0(1.));
#ifdef N
if(N){uint y1=S0.x>>16u;if(y1!=0u){dd(y1,Ga(m0),n);}}
#endif
#ifdef AB
if(AB&&(S0.x&og)!=0u){X N0=o1(p0(JB,c0*m2+2u));f x2=p0(JB,c0*m2+3u);c pg=B0(N0,d0)+x2.xy;D ed=B8(abs(pg)*x2.zw-x2.zw);d x5=clamp(min(ed.x,ed.y)+.5,.0,1.);n=min(n,x5);}
#endif
uint w2=S0.x&0xfu;P W1=S1((S0.x>>4)&0xfu);
#ifdef H
bool F2=H&&W1!=U3;
#else
const bool F2=false;
#endif
if(w2<=Ka){Q=unpackUnorm4x8(S0.y);
#ifdef N
if(N&&w2==J8){
#ifndef UC
#ifdef TC
D1.xy=Q.zw;D1.z=n;D1.w=1.;
#else
D1=S0.y|packHalf2x16(Q2(n,.0));
#endif
#endif
Q=H0(.0);}
#endif
}else{X N0=o1(p0(JB,c0*m2));f x2=p0(JB,c0*m2+1u);c qg=B0(N0,d0)+x2.xy;c K8=x2.zw;bool n4=K8.x<0.0;K8.x=max(0.0,K8.x);c i3=rg(float(w2),qg,K8,n4,j.L8,j.M8);Q=n2(XC,N8,i3,0.0);if(!F2){Q.xyz*=Q.w;float o4=uintBitsToFloat(S0.y);Q.w*=abs(o4);}}
#if!defined(U)&&defined(H)
if(F2){if(Q.w*n!=.0){i z1=Q0(n0);Q.xyz=N4(Q.xyz,z1,W1);}Q.xyz*=Q.w;}
#endif
Q*=n;}
#if!defined(U)&&!defined(GD)
e void O8(i Q m4){
#ifndef TC
if(Q.x+Q.y+Q.z+Q.w==.0) return;float k7=1.-Q.w;if(k7!=.0) Q+=Q0(n0)*k7;
#endif
y0(n0,Q);}
#endif
#if defined(N)&&!defined(UC)
e void Ma(Q4 D1 m4){
#ifdef TC
y0(m0,D1);
#else
if(D1!=0u) l1(m0,D1);
#endif
}
#endif
#ifdef U
#define l6 G2
#define m6 E3
#else
#define l6 X1
#define m6 o2
#endif
#ifdef PD
l6(IB){
#ifdef HB
q(S,f);
#else
q(S,D);
#endif
q(G0,P);d P8;
#ifdef HB
if(HB&&fd(S)){P8=T4(S m1);}else if(HB&&gd(S)){P8=Q8(S m1);}else
#endif
{P8=min(min(J0(S.x),abs(J0(S.y))),J0(1.));}i Q=H0(.0);
#ifdef N
Q4 D1=F8;
#endif
uint R8=lg(P8);uint hd=(id(G0)<<n6)|R8;uint H2=y5(R4,hd);P K1=S1(H2>>n6);K1=H8(K1);if(K1==G0){if(!o6(S)){R8+=H2-max(hd,H2);R8-=Na;z5(R4,R8);}}else{d w0=G8(H2&S8);I8(K1,w0,Q
#ifdef N
,D1
#endif
l3 Y1);}Q.xyz=I2(Q.xyz,Q.w,d0.xy,j.F3,j.G3);
#ifdef U
L1=Q;
#else
O8(Q Y1);
#endif
#ifdef N
Ma(D1 Y1);
#endif
m6}
#endif
#if defined(DB)||defined(FB)
l6(IB){
#ifdef FB
q(S2,c);
#else
q(n1,d);
#endif
q(G0,P);uint H2=m3(R4);P K1=S1(H2>>n6);K1=H8(K1);uint Oa;
#ifndef FB
if(K1==G0){Oa=H2;}else
#endif
{Oa=(id(G0)<<n6)+Na;}d n;
#ifdef FB
n=clamp(n2(HD,Pa,S2,.0).x,J0(.0),J0(1.));
#else
n=n1;
#endif
int sg=int(round(n*Ha));n3(R4,Oa+uint(sg));i Q=H0(.0);
#ifdef N
Q4 D1=F8;
#endif
#ifndef FB
if(K1!=G0)
#endif
{d Qa=G8(H2&S8);I8(K1,Qa,Q
#ifdef N
,D1
#endif
l3 Y1);}Q.xyz=I2(Q.xyz,Q.w,d0.xy,j.F3,j.G3);
#ifdef U
L1=Q;
#else
O8(Q Y1);
#endif
#ifdef N
Ma(D1 Y1);
#endif
m6}
#endif
#ifdef OE
l6(IB){q(l2,c);
#ifdef DD
q(q5,d);q(r5,f);
#endif
#ifdef AB
q(V0,f);
#endif
q(T1,i);
#ifdef N
q(S3,P);
#endif
#ifdef H
q(J1,P);
#endif
i M1=T8(TB,U4,l2);d p6=1.;
#ifdef DD
p6=min(q5,p6);
#endif
#ifdef AB
if(AB){d x5=B3(V4(V0));p6=clamp(x5,J0(.0),p6);}
#endif
uint H2=m3(R4);P K1=S1(H2>>n6);K1=H8(K1);d Qa=G8(H2&S8);i Q;
#ifdef N
Q4 D1=F8;
#endif
I8(K1,Qa,Q
#ifdef N
,D1
#endif
l3 Y1);
#ifdef N
if(N&&S3!=0u){Q4 W0=Zc(D1)?D1:Ga(m0);dd(S3,W0,p6);}
#endif
#ifdef DD
if(r5.w!=0.0){c Ra=Sa(r5,j.L8,j.M8);i Ta=n2(XC,N8,Ra,0.0);Ta.xyz*=Ta.w;M1*=Ta;}
#endif
M1*=T1;
#if!defined(U)&&defined(H)
if(H&&J1!=U3){i z1=Q0(n0)*(1.-Q.w)+Q;M1.xyz=N4(i6(M1),z1,J1)*M1.w;}
#endif
M1*=p6;Q=Q*(1.-M1.w)+M1;Q.xyz=I2(Q.xyz,Q.w,d0.xy,j.F3,j.G3);
#ifdef U
L1=Q;
#else
O8(Q Y1);
#endif
#ifdef N
Ma(D1 Y1);
#endif
n3(R4,Na);m6}
#endif
#ifdef PE
l6(IB){
#ifndef U
#ifdef TD
if(TD){y0(n0,unpackUnorm4x8(j.tg));}
#endif
#ifdef UD
if(UD){y0(n0,q1(TB,G));}
#endif
#ifdef MF
i l=Q0(n0);y0(n0,l.zyxw);
#endif
#endif
n3(R4,j.ug);
#ifdef N
if(N){l1(m0,0u);}
#endif
#ifdef U
discard;
#endif
m6}
#endif
#ifdef UC
#ifdef GD
G2(IB)
#else
l6(IB)
#endif
{uint H2=m3(R4);d w0=G8(H2&S8);P K1=S1(H2>>n6);K1=H8(K1);i Q;I8(K1,w0,Q l3 Y1);
#ifdef GD
float k7=1.-Q.w;if(k7!=.0) Q+=Q0(n0)*k7;L1=Q;E3
#else
Q.xyz=I2(Q.xyz,Q.w,d0.xy,j.F3,j.G3);
#ifdef U
L1=Q;
#else
O8(Q Y1);
#endif
m6
#endif
}
#endif
#endif
