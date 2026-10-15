#ifdef RD
#ifdef BB
e1(f0) K(0,f,XB);K(1,f,YB);f1
#endif
v2
#ifdef HB
F0 W(0,f,S);
#else
F0 W(0,D,S);
#endif
h3 W(1,P,G0);k2
#ifdef BB
y1(RB,f0,B,F,r){L(F,B,XB,f);L(F,B,YB,f);
#ifdef HB
V(S,f);
#else
V(S,D);
#endif
V(G0,P);f I;uint c0;c i0;f T;if(ra(XB,YB,r,c0,i0,T Q3)){
#ifdef HB
S=T;
#else
S.xy=E8(T.xy);
#endif
G0=U1(c0);I=R3(i0);}else{I=f(j.i3,j.i3,j.i3,j.i3);}Z(S);Z(G0);z1(I);}
#endif
#endif
#if defined(DB)||defined(FB)
#ifdef BB
e1(f0) K(0,i4,LB);f1
#endif
v2
#ifdef FB
F0 W(0,c,T2);
#else
MB W(0,d,o1);
#endif
h3 W(1,P,G0);k2
#ifdef BB
y1(RB,f0,B,F,r){L(F,B,LB,M);
#ifdef FB
V(T2,c);
#else
V(o1,d);
#endif
V(G0,P);uint c0;c i0;
#ifdef FB
i0=Xc(LB,c0,T2 Q3);
#else
i0=Yc(LB,c0,o1 Q3);
#endif
G0=U1(c0);f I=R3(i0);
#ifdef FB
Z(T2);
#else
Z(o1);
#endif
Z(G0);z1(I);}
#endif
#endif
#ifdef FD
#ifdef BB
e1(f0) K(0,f,GC);f1 e1(D1) K(sa,f,ZB);K(ta,f,SB);K(ua,f,PB);K(va,uint,AC);K(wa,uint,BC);K(xa,uint,CC);K(ya,uint,MC);K(eg,f,SD);K(fg,f,TD);K(gg,f,GD);K(Zc,f,HD);f1
#endif
v2 F0 W(0,c,l2);F0 W(1,d,r5);F0 W(2,f,v5);
#ifdef AB
F0 W(3,f,V0);
#endif
MB W(4,i,V1);
#ifdef N
h3 W(5,P,S3);
#endif
#ifdef H
h3 W(6,P,L1);
#endif
k2
#ifdef BB
F8(RB,f0,B,D1,j0,F,r){L(F,B,GC,f);L(r,j0,ZB,f);L(r,j0,SB,f);L(r,j0,PB,f);L(r,j0,AC,uint);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,MC,uint);L(r,j0,SD,f);L(r,j0,TD,f);L(r,j0,GD,f);L(r,j0,HD,f);V(l2,c);V(r5,d);V(v5,f);
#ifdef AB
V(V0,f);
#endif
V(V1,i);
#ifdef N
V(S3,P);
#endif
#ifdef H
V(L1,P);
#endif
bool za=GC.z==.0||GC.w==.0;r5=za?.0:1.;c i0=GC.xy;X N0=p1(ZB);X j7=transpose(inverse(N0));if(!za){float Aa=O4*Ba(j7[1])/dot(N0[1],j7[1]);if(Aa>=.5){i0.x=.5;r5*=w5(.5/Aa);}else{i0.x+=Aa*GC.z;}float Ca=O4*Ba(j7[0])/dot(N0[0],j7[0]);if(Ca>=.5){i0.y=.5;r5*=w5(.5/Ca);}else{i0.y+=Ca*GC.w;}}X hg=p1(SD);l2=C0(hg,i0)+GD.xy;i0=C0(N0,i0)+PB.xy;if(za){c j4=C0(j7,GC.zw);j4*=Ba(j4)/dot(j4,j4);i0+=O4*j4;}
#ifdef AB
if(AB){V0=G8(p1(SB),PB.zw,i0);}
#endif
V1=unpackUnorm4x8(AC);
#ifdef N
S3=U1(BC);
#endif
#ifdef H
L1=U1(CC);
#endif
f I=R3(i0);c l0=i0;
#ifdef UD
if(j.Da!=0u){l0.y=float(j.Ea)-l0.y;}
#endif
uint w2=floatBitsToUint(HD.z);if(w2!=0){X ig=p1(TD);c jg=GD.zw;v5=Fa(l0,ig,jg,HD.xy,w2,1.0,1.0);}else{v5=f(.0,.0,.0,.0);}Z(l2);Z(r5);Z(v5);
#ifdef AB
Z(V0);
#endif
Z(V1);
#ifdef N
Z(S3);
#endif
#ifdef H
Z(L1);
#endif
z1(I);}
#endif
#elif defined(NB)
#ifdef BB
e1(C3) K(0,c,PC);f1 e1(T3) K(1,c,QC);f1 e1(D1) K(sa,f,ZB);K(ta,f,SB);K(ua,f,PB);K(va,uint,AC);K(wa,uint,BC);K(xa,uint,CC);K(ya,uint,MC);K(Ga,f,HC);f1
#endif
v2 F0 W(0,c,l2);
#ifdef AB
F0 W(1,f,V0);
#endif
MB W(3,i,V1);
#ifdef N
h3 W(4,P,S3);
#endif
#ifdef H
h3 W(5,P,L1);
#endif
k2
#ifdef BB
k7(RB,C3,D3,T3,j3,D1,j0,F){L(F,D3,PC,c);L(F,j3,QC,c);L(r,j0,ZB,f);L(r,j0,SB,f);L(r,j0,PB,f);L(r,j0,AC,uint);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,MC,uint);L(r,j0,HC,f);V(l2,c);
#ifdef AB
V(V0,f);
#endif
V(V1,i);
#ifdef N
V(S3,P);
#endif
#ifdef H
V(L1,P);
#endif
X N0=p1(ZB);c i0=C0(N0,PC)+PB.xy;l2=QC*HC.zw+HC.xy;
#ifdef AB
if(AB){V0=G8(p1(SB),PB.zw,i0);}
#endif
V1=unpackUnorm4x8(AC);
#ifdef N
S3=U1(BC);
#endif
#ifdef H
L1=U1(CC);
#endif
f I=R3(i0);Z(l2);
#ifdef AB
Z(V0);
#endif
Z(V1);
#ifdef N
Z(S3);
#endif
#ifdef H
Z(L1);
#endif
z1(I);}
#endif
#endif
#ifdef MF
#ifdef BB
e1(f0) f1
#endif
v2 k2
#ifdef BB
y1(RB,f0,B,F,r){g0 F2;F2.x=(F&1)==0?j.H8.x:j.H8.z;F2.y=(F&2)==0?j.H8.y:j.H8.w;f I=R3(c(F2));z1(I);}
#endif
#endif
#ifdef QE
#endif
#if defined(RE)&&!defined(U)
#endif
#ifdef EB
E1
#ifndef U
#ifdef SE
#define Ha SE
#else
#define Ha x2
#endif
#ifdef ID
P4(Ha,n0);
#else
z0(Ha,n0);
#endif
#endif
#ifdef VC
#define Q4 i
#define Ia O0
#define I8 H0(.0)
#define ad(E) ((E).w!=.0)
#ifdef N
#ifndef WC
z0(k3,m0);
#else
P4(k3,m0);
#endif
#endif
#else
#define Q4 uint
#define I8 0u
#define Ia k1
#define ad(E) ((E)!=0u)
#ifdef N
q1(k3,m0);
#endif
#endif
U2(l7,R4);F1 k4 j6(bd,lg,XC);k6(cd,mg,JB);l4 e uint ng(float x){return uint(round(x*Ja+Ka));}e d J8(uint x){return w5(float(x)*dd+(-Ka*dd));}P K8(P c0){
#ifdef NF
c0=min(c0,j.og);
#endif
return c0;}
#ifdef N
e void ed(uint A1,Q4 W0,m7(d) n){
#ifdef VC
if(all(lessThan(abs(W0.xy-unpackUnorm4x8(A1).xy),R2(.25/255.)))) n=min(n,W0.z);else n=.0;
#else
if(A1==W0>>16) n=min(n,unpackHalf2x16(W0).x);else n=.0;
#endif
}
#endif
e void L8(uint c0,d w0,l1(i) Q
#if defined(N)&&!defined(WC)
,m7(Q4) G1
#endif
n7 m4){R0 S0=x5(XC,c0);d n=w0;if((S0.x&(pg|La))!=0u){n=abs(n);
#ifdef YC
if(YC&&(S0.x&La)!=0u){n=1.-abs(fract(n*.5)*2.+-1.);}
#endif
}n=clamp(n,J0(.0),J0(1.));
#ifdef N
if(N){uint A1=S0.x>>16u;if(A1!=0u){ed(A1,Ia(m0),n);}}
#endif
#ifdef AB
if(AB&&(S0.x&qg)!=0u){X N0=p1(p0(JB,c0*m2+2u));f y2=p0(JB,c0*m2+3u);c rg=C0(N0,d0)+y2.xy;D fd=E8(abs(rg)*y2.zw-y2.zw);d y5=clamp(min(fd.x,fd.y)+.5,.0,1.);n=min(n,y5);}
#endif
uint w2=S0.x&0xfu;P W1=U1((S0.x>>4)&0xfu);
#ifdef H
bool G2=H&&W1!=U3;
#else
const bool G2=false;
#endif
if(w2<=Ma){Q=unpackUnorm4x8(S0.y);
#ifdef N
if(N&&w2==M8){
#ifndef WC
#ifdef VC
G1.xy=Q.zw;G1.z=n;G1.w=1.;
#else
G1=S0.y|packHalf2x16(R2(n,.0));
#endif
#endif
Q=H0(.0);}
#endif
}else{X N0=p1(p0(JB,c0*m2));f y2=p0(JB,c0*m2+1u);c sg=C0(N0,d0)+y2.xy;c N8=y2.zw;bool n4=N8.x<0.0;N8.x=max(0.0,N8.x);c j3=tg(float(w2),sg,N8,n4,j.O8,j.P8);Q=n2(ZC,Q8,j3,0.0);if(!G2){Q.xyz*=Q.w;float o4=uintBitsToFloat(S0.y);Q.w*=abs(o4);}}
#if!defined(U)&&defined(H)
if(G2){if(Q.w*n!=.0){i a1=O0(n0);Q.xyz=N4(Q.xyz,a1,W1);}Q.xyz*=Q.w;}
#endif
Q*=n;}
#if!defined(U)&&!defined(ID)
e void R8(i Q m4){
#ifndef VC
if(Q.x+Q.y+Q.z+Q.w==.0) return;if(Q.w!=1.) Q=Oa(Q,O0(n0));
#endif
x0(n0,Q);}
#endif
#if defined(N)&&!defined(WC)
e void Pa(Q4 G1 m4){
#ifdef VC
x0(m0,G1);
#else
if(G1!=0u) m1(m0,G1);
#endif
}
#endif
#ifdef U
#define l6 H2
#define m6 E3
#else
#define l6 X1
#define m6 o2
#endif
#ifdef RD
l6(IB){
#ifdef HB
q(S,f);
#else
q(S,D);
#endif
q(G0,P);d S8;
#ifdef HB
if(HB&&gd(S)){S8=T4(S n1);}else if(HB&&hd(S)){S8=T8(S n1);}else
#endif
{S8=min(min(J0(S.x),abs(J0(S.y))),J0(1.));}i Q=H0(.0);
#ifdef N
Q4 G1=I8;
#endif
uint U8=ng(S8);uint id=(jd(G0)<<n6)|U8;uint I2=z5(R4,id);P M1=U1(I2>>n6);M1=K8(M1);if(M1==G0){if(!o6(S)){U8+=I2-max(id,I2);U8-=Qa;A5(R4,U8);}}else{d w0=J8(I2&V8);L8(M1,w0,Q
#ifdef N
,G1
#endif
m3 Y1);}Q.xyz=J2(Q.xyz,Q.w,d0.xy,j.F3,j.G3);
#ifdef U
N1=Q;
#else
R8(Q Y1);
#endif
#ifdef N
Pa(G1 Y1);
#endif
m6}
#endif
#if defined(DB)||defined(FB)
l6(IB){
#ifdef FB
q(T2,c);
#else
q(o1,d);
#endif
q(G0,P);uint I2=n3(R4);P M1=U1(I2>>n6);M1=K8(M1);uint Ra;
#ifndef FB
if(M1==G0){Ra=I2;}else
#endif
{Ra=(jd(G0)<<n6)+Qa;}d n;
#ifdef FB
n=clamp(n2(JD,Sa,T2,.0).x,J0(.0),J0(1.));
#else
n=o1;
#endif
int ug=int(round(n*Ja));o3(R4,Ra+uint(ug));i Q=H0(.0);
#ifdef N
Q4 G1=I8;
#endif
#ifndef FB
if(M1!=G0)
#endif
{d Ta=J8(I2&V8);L8(M1,Ta,Q
#ifdef N
,G1
#endif
m3 Y1);}Q.xyz=J2(Q.xyz,Q.w,d0.xy,j.F3,j.G3);
#ifdef U
N1=Q;
#else
R8(Q Y1);
#endif
#ifdef N
Pa(G1 Y1);
#endif
m6}
#endif
#ifdef QE
l6(IB){q(l2,c);
#ifdef FD
q(r5,d);q(v5,f);
#endif
#ifdef AB
q(V0,f);
#endif
q(V1,i);
#ifdef N
q(S3,P);
#endif
#ifdef H
q(L1,P);
#endif
i O1=W8(TB,U4,l2);d p6=1.;
#ifdef FD
p6=min(r5,p6);
#endif
#ifdef AB
if(AB){d y5=B3(V4(V0));p6=clamp(y5,J0(.0),p6);}
#endif
uint I2=n3(R4);P M1=U1(I2>>n6);M1=K8(M1);d Ta=J8(I2&V8);i Q;
#ifdef N
Q4 G1=I8;
#endif
L8(M1,Ta,Q
#ifdef N
,G1
#endif
m3 Y1);
#ifdef N
if(N&&S3!=0u){Q4 W0=ad(G1)?G1:Ia(m0);ed(S3,W0,p6);}
#endif
#ifdef FD
if(v5.w!=0.0){c Ua=Va(v5,j.O8,j.P8);i Wa=n2(ZC,Q8,Ua,0.0);Wa.xyz*=Wa.w;O1*=Wa;}
#endif
O1*=V1;
#if!defined(U)&&defined(H)
if(H&&L1!=U3){i a1=O0(n0)*(1.-Q.w)+Q;O1.xyz=N4(i6(O1),a1,L1)*O1.w;}
#endif
O1*=p6;Q=Q*(1.-O1.w)+O1;Q.xyz=J2(Q.xyz,Q.w,d0.xy,j.F3,j.G3);
#ifdef U
N1=Q;
#else
R8(Q Y1);
#endif
#ifdef N
Pa(G1 Y1);
#endif
o3(R4,Qa);m6}
#endif
#ifdef RE
l6(IB){
#ifndef U
#ifdef VD
if(VD){x0(n0,unpackUnorm4x8(j.vg));}
#endif
#ifdef WD
if(WD){x0(n0,r1(TB,G));}
#endif
#ifdef OF
i k=O0(n0);x0(n0,k.zyxw);
#endif
#endif
o3(R4,j.wg);
#ifdef N
if(N){m1(m0,0u);}
#endif
#ifdef U
discard;
#endif
m6}
#endif
#ifdef WC
#ifdef ID
H2(IB)
#else
l6(IB)
#endif
{uint I2=n3(R4);d w0=J8(I2&V8);P M1=U1(I2>>n6);M1=K8(M1);i Q;L8(M1,w0,Q m3 Y1);
#ifdef ID
float kd=1.-Q.w;if(kd!=.0) Q+=O0(n0)*kd;N1=Q;E3
#else
Q.xyz=J2(Q.xyz,Q.w,d0.xy,j.F3,j.G3);
#ifdef U
N1=Q;
#else
R8(Q Y1);
#endif
m6
#endif
}
#endif
#endif
