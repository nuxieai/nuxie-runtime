#ifdef OD
#ifdef BB
f1(f0) K(0,e,XB);K(1,e,YB);g1
#endif
w2
#ifdef HB
F0 X(0,e,S);
#else
F0 X(0,D,S);
#endif
g3 X(1,P,G0);l2
#ifdef BB
x1(RB,f0,B,F,r){L(F,B,XB,e);L(F,B,YB,e);
#ifdef HB
V(S,e);
#else
V(S,D);
#endif
V(G0,P);e I;uint c0;c i0;e T;if(ia(XB,YB,r,c0,i0,T P3)){
#ifdef HB
S=T;
#else
S.xy=y8(T.xy);
#endif
G0=T1(c0);I=Q3(i0);}else{I=e(j.h3,j.h3,j.h3,j.h3);}Z(S);Z(G0);y1(I);}
#endif
#endif
#if defined(DB)||defined(FB)
#ifdef BB
f1(f0) K(0,h4,LB);g1
#endif
w2
#ifdef FB
F0 X(0,c,T2);
#else
MB X(0,d,o1);
#endif
g3 X(1,P,G0);l2
#ifdef BB
x1(RB,f0,B,F,r){L(F,B,LB,M);
#ifdef FB
V(T2,c);
#else
V(o1,d);
#endif
V(G0,P);uint c0;c i0;
#ifdef FB
i0=Oc(LB,c0,T2 P3);
#else
i0=Pc(LB,c0,o1 P3);
#endif
G0=T1(c0);e I=Q3(i0);
#ifdef FB
Z(T2);
#else
Z(o1);
#endif
Z(G0);y1(I);}
#endif
#endif
#ifdef DD
#ifdef BB
f1(f0) K(0,e,GC);g1 f1(D1) K(ja,e,ZB);K(ka,e,SB);K(la,e,PB);K(ma,uint,AC);K(na,uint,BC);K(oa,uint,CC);K(pa,uint,LC);K(Zf,e,PD);K(ag,e,QD);K(bg,e,ED);K(Qc,e,OC);g1
#endif
w2 F0 X(0,c,m2);F0 X(1,d,o5);F0 X(2,e,p5);
#ifdef AB
F0 X(3,e,W0);
#endif
MB X(4,i,U1);
#ifdef N
g3 X(5,P,R3);
#endif
#ifdef H
g3 X(6,P,K1);
#endif
l2
#ifdef BB
z8(RB,f0,B,D1,j0,F,r){L(F,B,GC,e);L(r,j0,ZB,e);L(r,j0,SB,e);L(r,j0,PB,e);L(r,j0,AC,uint);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,LC,uint);L(r,j0,PD,e);L(r,j0,QD,e);L(r,j0,ED,e);L(r,j0,OC,e);V(m2,c);V(o5,d);V(p5,e);
#ifdef AB
V(W0,e);
#endif
V(U1,i);
#ifdef N
V(R3,P);
#endif
#ifdef H
V(K1,P);
#endif
bool qa=GC.z==.0||GC.w==.0;o5=qa?.0:1.;c i0=GC.xy;W N0=p1(ZB);W a7=transpose(inverse(N0));if(!qa){float ra=M4*sa(a7[1])/dot(N0[1],a7[1]);if(ra>=.5){i0.x=.5;o5*=i4(.5/ra);}else{i0.x+=ra*GC.z;}float ta=M4*sa(a7[0])/dot(N0[0],a7[0]);if(ta>=.5){i0.y=.5;o5*=i4(.5/ta);}else{i0.y+=ta*GC.w;}}W cg=p1(PD);m2=y0(cg,i0)+ED.xy;i0=y0(N0,i0)+PB.xy;if(qa){c j4=y0(a7,GC.zw);j4*=sa(j4)/dot(j4,j4);i0+=M4*j4;}
#ifdef AB
if(AB){W0=A8(p1(SB),PB.zw,i0);}
#endif
U1=unpackUnorm4x8(AC);
#ifdef N
R3=T1(BC);
#endif
#ifdef H
K1=T1(CC);
#endif
e I=Q3(i0);c l0=i0;
#ifdef RD
if(j.ua!=0u){l0.y=float(j.va)-l0.y;}
#endif
if(OC.w!=0.0){W dg=p1(QD);c eg=ED.zw;p5=Rc(l0,dg,eg,OC.w,OC.xy,OC.z);}else{p5=e(.0,.0,.0,.0);}Z(m2);Z(o5);Z(p5);
#ifdef AB
Z(W0);
#endif
Z(U1);
#ifdef N
Z(R3);
#endif
#ifdef H
Z(K1);
#endif
y1(I);}
#endif
#elif defined(NB)
#ifdef BB
f1(B3) K(0,c,PC);g1 f1(S3) K(1,c,QC);g1 f1(D1) K(ja,e,ZB);K(ka,e,SB);K(la,e,PB);K(ma,uint,AC);K(na,uint,BC);K(oa,uint,CC);K(pa,uint,LC);K(wa,e,HC);g1
#endif
w2 F0 X(0,c,m2);
#ifdef AB
F0 X(1,e,W0);
#endif
MB X(3,i,U1);
#ifdef N
g3 X(4,P,R3);
#endif
#ifdef H
g3 X(5,P,K1);
#endif
l2
#ifdef BB
c7(RB,B3,C3,S3,x2,D1,j0,F){L(F,C3,PC,c);L(F,x2,QC,c);L(r,j0,ZB,e);L(r,j0,SB,e);L(r,j0,PB,e);L(r,j0,AC,uint);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,LC,uint);L(r,j0,HC,e);V(m2,c);
#ifdef AB
V(W0,e);
#endif
V(U1,i);
#ifdef N
V(R3,P);
#endif
#ifdef H
V(K1,P);
#endif
W N0=p1(ZB);c i0=y0(N0,PC)+PB.xy;m2=QC*HC.zw+HC.xy;
#ifdef AB
if(AB){W0=A8(p1(SB),PB.zw,i0);}
#endif
U1=unpackUnorm4x8(AC);
#ifdef N
R3=T1(BC);
#endif
#ifdef H
K1=T1(CC);
#endif
e I=Q3(i0);Z(m2);
#ifdef AB
Z(W0);
#endif
Z(U1);
#ifdef N
Z(R3);
#endif
#ifdef H
Z(K1);
#endif
y1(I);}
#endif
#endif
#ifdef JF
#ifdef BB
f1(f0) g1
#endif
w2 l2
#ifdef BB
x1(RB,f0,B,F,r){g0 E2;E2.x=(F&1)==0?j.B8.x:j.B8.z;E2.y=(F&2)==0?j.B8.y:j.B8.w;e I=Q3(c(E2));y1(I);}
#endif
#endif
#ifdef NE
#endif
#if defined(OE)&&!defined(U)
#endif
#ifdef EB
V1
#ifndef U
#ifdef PE
#define xa PE
#else
#define xa U2
#endif
#ifdef FD
N4(xa,n0);
#else
C0(xa,n0);
#endif
#endif
#ifdef UC
#define O4 i
#define ya R0
#define C8 H0(.0)
#define Sc(E) ((E).w!=.0)
#ifdef N
#ifndef VC
C0(i3,m0);
#else
N4(i3,m0);
#endif
#endif
#else
#define O4 uint
#define C8 0u
#define ya l1
#define Sc(E) ((E)!=0u)
#ifdef N
q1(i3,m0);
#endif
#endif
V2(d7,P4);W1 k4 g6(Tc,gg,WC);h6(Uc,hg,JB);l4 f uint ig(float x){return uint(round(x*za+Aa));}f d D8(uint x){return i4(float(x)*Vc+(-Aa*Vc));}P E8(P c0){
#ifdef KF
c0=min(c0,j.jg);
#endif
return c0;}
#ifdef N
f void Wc(uint z1,O4 X0,e7(d) l){
#ifdef UC
if(all(lessThan(abs(X0.xy-unpackUnorm4x8(z1).xy),R2(.25/255.)))) l=min(l,X0.z);else l=.0;
#else
if(z1==X0>>16) l=min(l,unpackHalf2x16(X0).x);else l=.0;
#endif
}
#endif
f void F8(uint c0,d w0,c1(i) Q
#if defined(N)&&!defined(VC)
,e7(O4) E1
#endif
f7 m4){S0 T0=q5(WC,c0);d l=w0;if((T0.x&(kg|Ba))!=0u){l=abs(l);
#ifdef XC
if(XC&&(T0.x&Ba)!=0u){l=1.-abs(fract(l*.5)*2.+-1.);}
#endif
}l=clamp(l,I0(.0),I0(1.));
#ifdef N
if(N){uint z1=T0.x>>16u;if(z1!=0u){Wc(z1,ya(m0),l);}}
#endif
#ifdef AB
if(AB&&(T0.x&lg)!=0u){W N0=p1(p0(JB,c0*n2+2u));e L1=p0(JB,c0*n2+3u);c mg=y0(N0,d0)+L1.xy;D Xc=y8(abs(mg)*L1.zw-L1.zw);d r5=clamp(min(Xc.x,Xc.y)+.5,.0,1.);l=min(l,r5);}
#endif
uint j3=T0.x&0xfu;P X1=T1((T0.x>>4)&0xfu);
#ifdef H
bool F2=H&&X1!=T3;
#else
const bool F2=false;
#endif
if(j3<=Ca){Q=unpackUnorm4x8(T0.y);
#ifdef N
if(N&&j3==G8){
#ifndef VC
#ifdef UC
E1.xy=Q.zw;E1.z=l;E1.w=1.;
#else
E1=T0.y|packHalf2x16(R2(l,.0));
#endif
#endif
Q=H0(.0);}
#endif
}else{W N0=p1(p0(JB,c0*n2));e L1=p0(JB,c0*n2+1u);c Yc=y0(N0,d0)+L1.xy;float t=j3==Ea?Yc.x:length(Yc);t=clamp(t,.0,1.);float x=t*L1.z+L1.w;float Zc=uintBitsToFloat(T0.y);float ng=floor(Zc)*j.ad+j.g7;Q=o2(YC,H8,c(x,ng),.0);if(!F2){Q.xyz*=Q.w;d Fa=i4(fract(Zc)*(256./255.));Q.w*=Fa;}}
#if!defined(U)&&defined(H)
if(F2){if(Q.w*l!=.0){i A1=R0(n0);Q.xyz=L4(Q.xyz,A1,X1);}Q.xyz*=Q.w;}
#endif
Q*=l;}
#if!defined(U)&&!defined(FD)
f void I8(i Q m4){
#ifndef UC
if(Q.x+Q.y+Q.z+Q.w==.0) return;float h7=1.-Q.w;if(h7!=.0) Q+=R0(n0)*h7;
#endif
z0(n0,Q);}
#endif
#if defined(N)&&!defined(VC)
f void Ga(O4 E1 m4){
#ifdef UC
z0(m0,E1);
#else
if(E1!=0u) m1(m0,E1);
#endif
}
#endif
#ifdef U
#define i6 G2
#define j6 D3
#else
#define i6 Y1
#define j6 p2
#endif
#ifdef OD
i6(IB){
#ifdef HB
q(S,e);
#else
q(S,D);
#endif
q(G0,P);d J8;
#ifdef HB
if(HB&&bd(S)){J8=R4(S n1);}else if(HB&&cd(S)){J8=K8(S n1);}else
#endif
{J8=min(min(I0(S.x),abs(I0(S.y))),I0(1.));}i Q=H0(.0);
#ifdef N
O4 E1=C8;
#endif
uint L8=ig(J8);uint dd=(ed(G0)<<k6)|L8;uint H2=v5(P4,dd);P M1=T1(H2>>k6);M1=E8(M1);if(M1==G0){if(!l6(S)){L8+=H2-max(dd,H2);L8-=Ha;w5(P4,L8);}}else{d w0=D8(H2&M8);F8(M1,w0,Q
#ifdef N
,E1
#endif
l3 Z1);}Q.xyz=I2(Q.xyz,Q.w,d0.xy,j.E3,j.F3);
#ifdef U
N1=Q;
#else
I8(Q Z1);
#endif
#ifdef N
Ga(E1 Z1);
#endif
j6}
#endif
#if defined(DB)||defined(FB)
i6(IB){
#ifdef FB
q(T2,c);
#else
q(o1,d);
#endif
q(G0,P);uint H2=m3(P4);P M1=T1(H2>>k6);M1=E8(M1);uint Ia;
#ifndef FB
if(M1==G0){Ia=H2;}else
#endif
{Ia=(ed(G0)<<k6)+Ha;}d l;
#ifdef FB
l=clamp(o2(GD,Ja,T2,.0).x,I0(.0),I0(1.));
#else
l=o1;
#endif
int og=int(round(l*za));n3(P4,Ia+uint(og));i Q=H0(.0);
#ifdef N
O4 E1=C8;
#endif
#ifndef FB
if(M1!=G0)
#endif
{d Ka=D8(H2&M8);F8(M1,Ka,Q
#ifdef N
,E1
#endif
l3 Z1);}Q.xyz=I2(Q.xyz,Q.w,d0.xy,j.E3,j.F3);
#ifdef U
N1=Q;
#else
I8(Q Z1);
#endif
#ifdef N
Ga(E1 Z1);
#endif
j6}
#endif
#ifdef NE
i6(IB){q(m2,c);
#ifdef DD
q(o5,d);q(p5,e);
#endif
#ifdef AB
q(W0,e);
#endif
q(U1,i);
#ifdef N
q(R3,P);
#endif
#ifdef H
q(K1,P);
#endif
i O1=N8(TB,S4,m2);d m6=1.;
#ifdef DD
m6=min(o5,m6);
#endif
#ifdef AB
if(AB){d r5=A3(T4(W0));m6=clamp(r5,I0(.0),m6);}
#endif
uint H2=m3(P4);P M1=T1(H2>>k6);M1=E8(M1);d Ka=D8(H2&M8);i Q;
#ifdef N
O4 E1=C8;
#endif
F8(M1,Ka,Q
#ifdef N
,E1
#endif
l3 Z1);
#ifdef N
if(N&&R3!=0u){O4 X0=Sc(E1)?E1:ya(m0);Wc(R3,X0,m6);}
#endif
#ifdef DD
if(p5.w!=0.0){c La=fd(p5);i Ma=o2(YC,H8,La,0.0);Ma.xyz*=Ma.w;O1*=Ma;}
#endif
O1*=U1;
#if!defined(U)&&defined(H)
if(H&&K1!=T3){i A1=R0(n0)*(1.-Q.w)+Q;O1.xyz=L4(f6(O1),A1,K1)*O1.w;}
#endif
O1*=m6;Q=Q*(1.-O1.w)+O1;Q.xyz=I2(Q.xyz,Q.w,d0.xy,j.E3,j.F3);
#ifdef U
N1=Q;
#else
I8(Q Z1);
#endif
#ifdef N
Ga(E1 Z1);
#endif
n3(P4,Ha);j6}
#endif
#ifdef OE
i6(IB){
#ifndef U
#ifdef SD
if(SD){z0(n0,unpackUnorm4x8(j.pg));}
#endif
#ifdef TD
if(TD){z0(n0,r1(TB,G));}
#endif
#ifdef LF
i n=R0(n0);z0(n0,n.zyxw);
#endif
#endif
n3(P4,j.qg);
#ifdef N
if(N){m1(m0,0u);}
#endif
#ifdef U
discard;
#endif
j6}
#endif
#ifdef VC
#ifdef FD
G2(IB)
#else
i6(IB)
#endif
{uint H2=m3(P4);d w0=D8(H2&M8);P M1=T1(H2>>k6);M1=E8(M1);i Q;F8(M1,w0,Q l3 Z1);
#ifdef FD
float h7=1.-Q.w;if(h7!=.0) Q+=R0(n0)*h7;N1=Q;D3
#else
Q.xyz=I2(Q.xyz,Q.w,d0.xy,j.E3,j.F3);
#ifdef U
N1=Q;
#else
I8(Q Z1);
#endif
j6
#endif
}
#endif
#endif
