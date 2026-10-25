#ifdef SD
#ifdef BB
e1(e0) K(0,e,YB);K(1,e,ZB);f1
#endif
m2
#ifdef IB
H0 W(0,e,T);
#else
H0 W(0,E,T);
#endif
h3 W(1,P,I0);M1
#ifdef BB
o1(TB,e0,A,C,r){L(C,A,YB,e);L(C,A,ZB,e);
#ifdef IB
V(T,e);
#else
V(T,E);
#endif
V(I0,P);e I;uint f0;c i0;e U;if(sa(YB,ZB,r,f0,i0,U T3)){
#ifdef IB
T=U;
#else
T.xy=G8(U.xy);
#endif
I0=X1(f0);I=U3(i0);}else{I=e(j.i3,j.i3,j.i3,j.i3);}a0(T);a0(I0);A1(I);}
#endif
#endif
#if defined(EB)||defined(FB)
#ifdef BB
e1(e0) K(0,k4,LB);f1
#endif
m2
#ifdef FB
H0 W(0,c,U2);
#else
MB W(0,d,p1);
#endif
h3 W(1,P,I0);M1
#ifdef BB
o1(TB,e0,A,C,r){L(C,A,LB,M);
#ifdef FB
V(U2,c);
#else
V(p1,d);
#endif
V(I0,P);uint f0;c i0;
#ifdef FB
i0=ed(LB,f0,U2 T3);
#else
i0=fd(LB,f0,p1 T3);
#endif
I0=X1(f0);e I=U3(i0);
#ifdef FB
a0(U2);
#else
a0(p1);
#endif
a0(I0);A1(I);}
#endif
#endif
#ifdef FD
#ifdef BB
e1(e0) K(0,e,HC);f1 e1(E1) K(ta,e,AC);K(ua,e,UB);K(va,e,PB);K(wa,uint,BC);K(xa,uint,CC);K(ya,uint,DC);K(za,uint,MC);K(ng,e,TD);K(og,e,UD);K(pg,e,GD);K(gd,e,HD);f1
#endif
m2 H0 W(0,c,n2);H0 W(1,d,x5);H0 W(2,e,y5);
#ifdef AB
H0 W(3,e,W0);
#endif
MB W(4,i,Y1);
#ifdef N
h3 W(5,P,V3);
#endif
#ifdef H
h3 W(6,P,N1);
#endif
M1
#ifdef BB
H8(TB,e0,A,E1,j0,C,r){L(C,A,HC,e);L(r,j0,AC,e);L(r,j0,UB,e);L(r,j0,PB,e);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,DC,uint);L(r,j0,MC,uint);L(r,j0,TD,e);L(r,j0,UD,e);L(r,j0,GD,e);L(r,j0,HD,e);V(n2,c);V(x5,d);V(y5,e);
#ifdef AB
V(W0,e);
#endif
V(Y1,i);
#ifdef N
V(V3,P);
#endif
#ifdef H
V(N1,P);
#endif
bool Aa=HC.z==.0||HC.w==.0;x5=Aa?.0:1.;c i0=HC.xy;X R0=q1(AC);X k7=transpose(inverse(R0));if(!Aa){float Ba=S4*Ca(k7[1])/dot(R0[1],k7[1]);if(Ba>=.5){i0.x=.5;x5*=T4(.5/Ba);}else{i0.x+=Ba*HC.z;}float Da=S4*Ca(k7[0])/dot(R0[0],k7[0]);if(Da>=.5){i0.y=.5;x5*=T4(.5/Da);}else{i0.y+=Da*HC.w;}}X qg=q1(TD);n2=D0(qg,i0)+GD.xy;i0=D0(R0,i0)+PB.xy;if(Aa){c l4=D0(k7,HC.zw);l4*=Ca(l4)/dot(l4,l4);i0+=S4*l4;}
#ifdef AB
if(AB){W0=I8(q1(UB),PB.zw,i0);}
#endif
Y1=unpackUnorm4x8(BC);
#ifdef N
V3=X1(CC);
#endif
#ifdef H
N1=X1(DC);
#endif
e I=U3(i0);c l0=i0;
#ifdef VD
if(j.Ea!=0u){l0.y=float(j.Fa)-l0.y;}
#endif
uint w2=floatBitsToUint(HD.z);if(w2!=0){X rg=q1(UD);c sg=GD.zw;y5=Ga(l0,rg,sg,HD.xy,w2,1.0,1.0);}else{y5=e(.0,.0,.0,.0);}a0(n2);a0(x5);a0(y5);
#ifdef AB
a0(W0);
#endif
a0(Y1);
#ifdef N
a0(V3);
#endif
#ifdef H
a0(N1);
#endif
A1(I);}
#endif
#elif defined(NB)
#ifdef BB
e1(E3) K(0,c,PC);f1 e1(W3) K(1,c,QC);f1 e1(E1) K(ta,e,AC);K(ua,e,UB);K(va,e,PB);K(wa,uint,BC);K(xa,uint,CC);K(ya,uint,DC);K(za,uint,MC);K(Ha,e,IC);f1
#endif
m2 H0 W(0,c,n2);
#ifdef AB
H0 W(1,e,W0);
#endif
MB W(3,i,Y1);
#ifdef N
h3 W(4,P,V3);
#endif
#ifdef H
h3 W(5,P,N1);
#endif
M1
#ifdef BB
l7(TB,E3,F3,W3,j3,E1,j0,C){L(C,F3,PC,c);L(C,j3,QC,c);L(r,j0,AC,e);L(r,j0,UB,e);L(r,j0,PB,e);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,DC,uint);L(r,j0,MC,uint);L(r,j0,IC,e);V(n2,c);
#ifdef AB
V(W0,e);
#endif
V(Y1,i);
#ifdef N
V(V3,P);
#endif
#ifdef H
V(N1,P);
#endif
X R0=q1(AC);c i0=D0(R0,PC)+PB.xy;n2=QC*IC.zw+IC.xy;
#ifdef AB
if(AB){W0=I8(q1(UB),PB.zw,i0);}
#endif
Y1=unpackUnorm4x8(BC);
#ifdef N
V3=X1(CC);
#endif
#ifdef H
N1=X1(DC);
#endif
e I=U3(i0);a0(n2);
#ifdef AB
a0(W0);
#endif
a0(Y1);
#ifdef N
a0(V3);
#endif
#ifdef H
a0(N1);
#endif
A1(I);}
#endif
#endif
#ifdef NF
#ifdef BB
e1(e0) f1
#endif
m2 M1
#ifdef BB
o1(TB,e0,A,C,r){g0 F2;F2.x=(C&1)==0?j.J8.x:j.J8.z;F2.y=(C&2)==0?j.J8.y:j.J8.w;e I=U3(c(F2));A1(I);}
#endif
#endif
#ifdef TE
#endif
#if defined(UE)&&!defined(R)
#endif
#ifdef DB
F1
#ifndef R
#ifdef VE
#define Ia VE
#else
#define Ia x2
#endif
#ifdef ID
U4(Ia,n0);
#else
y0(Ia,n0);
#endif
#endif
#ifdef VC
#define V4 i
#define Ja K0
#define K8 E0(.0)
#define hd(F) ((F).w!=.0)
#ifdef N
#ifndef WC
y0(k3,m0);
#else
U4(k3,m0);
#endif
#endif
#else
#define V4 uint
#define K8 0u
#define Ja k1
#define hd(F) ((F)!=0u)
#ifdef N
r1(k3,m0);
#endif
#endif
V2(m7,W4);G1 m4 m6(id,ug,XC);n6(jd,vg,JB);n4 f uint wg(float x){return uint(round(x*Ka+La));}f d L8(uint x){return T4(float(x)*kd+(-La*kd));}P M8(P f0){
#ifdef OF
f0=min(f0,j.xg);
#endif
return f0;}
#ifdef N
f void ld(uint B1,V4 X0,n7(d) n){
#ifdef VC
if(all(lessThan(abs(X0.xy-unpackUnorm4x8(B1).xy),S2(.25/255.)))) n=min(n,X0.z);else n=.0;
#else
if(B1==X0>>16) n=min(n,unpackHalf2x16(X0).x);else n=.0;
#endif
}
#endif
f void N8(uint f0,d w0,l1(i) Q
#if defined(N)&&!defined(WC)
,n7(V4) H1
#endif
o7 p4){T0 L0=z5(XC,f0);d n=w0;if((L0.x&(yg|Ma))!=0u){n=abs(n);
#ifdef YC
if(YC&&(L0.x&Ma)!=0u){n=1.-abs(fract(n*.5)*2.+-1.);}
#endif
}n=clamp(n,C0(.0),C0(1.));
#ifdef N
if(N){uint B1=L0.x>>16u;if(B1!=0u){ld(B1,Ja(m0),n);}}
#endif
#ifdef AB
if(AB&&(L0.x&zg)!=0u){X R0=q1(q0(JB,f0*o2+2u));e y2=q0(JB,f0*o2+3u);c Ag=D0(R0,c0)+y2.xy;E md=G8(abs(Ag)*y2.zw-y2.zw);d A5=clamp(min(md.x,md.y)+.5,.0,1.);n=min(n,A5);}
#endif
uint w2=L0.x&0xfu;P Z1=X1((L0.x>>4)&0xfu);
#ifdef H
bool G2=H&&Z1!=X3;
#else
const bool G2=false;
#endif
if(w2<=Na){Q=unpackUnorm4x8(L0.y);
#ifdef N
if(N&&w2==O8){
#ifndef WC
#ifdef VC
H1.xy=Q.zw;H1.z=n;H1.w=1.;
#else
H1=L0.y|packHalf2x16(S2(n,.0));
#endif
#endif
Q=E0(.0);}
#endif
}else{X R0=q1(q0(JB,f0*o2));e y2=q0(JB,f0*o2+1u);c Bg=D0(R0,c0)+y2.xy;c P8=y2.zw;bool q4=P8.x<0.0;P8.x=max(0.0,P8.x);c j3=Cg(float(w2),Bg,P8,q4,j.Q8,j.R8);Q=p2(ZC,S8,j3,0.0);if(!G2){Q.xyz*=Q.w;float r4=uintBitsToFloat(L0.y);Q.w*=abs(r4);}}
#if!defined(R)&&defined(H)
if(G2){if(Q.w*n!=.0){i a1=K0(n0);Q.xyz=R4(Q.xyz,a1,Z1);}Q.xyz*=Q.w;}
#endif
Q*=n;}
#if!defined(R)&&!defined(ID)
f void T8(i Q p4){
#ifndef VC
if(Q.x+Q.y+Q.z+Q.w==.0) return;if(Q.w!=1.) Q=Pa(Q,K0(n0));
#endif
x0(n0,Q);}
#endif
#if defined(N)&&!defined(WC)
f void Qa(V4 H1 p4){
#ifdef VC
x0(m0,H1);
#else
if(H1!=0u) m1(m0,H1);
#endif
}
#endif
#ifdef R
#define o6 H2
#define p6 G3
#else
#define o6 I1
#define p6 O1
#endif
#ifdef SD
o6(HB){
#ifdef IB
q(T,e);
#else
q(T,E);
#endif
q(I0,P);d U8;
#ifdef IB
if(IB&&nd(T)){U8=X4(T n1);}else if(IB&&od(T)){U8=V8(T n1);}else
#endif
{U8=min(min(C0(T.x),abs(C0(T.y))),C0(1.));}i Q=E0(.0);
#ifdef N
V4 H1=K8;
#endif
uint W8=wg(U8);uint pd=(qd(I0)<<q6)|W8;uint I2=B5(W4,pd);P P1=X1(I2>>q6);P1=M8(P1);if(P1==I0){if(!r6(T)){W8+=I2-max(pd,I2);W8-=Ra;C5(W4,W8);}}else{d w0=L8(I2&X8);N8(P1,w0,Q
#ifdef N
,H1
#endif
m3 a2);}Q.xyz=J2(Q.xyz,Q.w,c0.xy,j.H3,j.I3);
#ifdef R
Q1=Q;
#else
T8(Q a2);
#endif
#ifdef N
Qa(H1 a2);
#endif
p6}
#endif
#if defined(EB)||defined(FB)
o6(HB){
#ifdef FB
q(U2,c);
#else
q(p1,d);
#endif
q(I0,P);uint I2=n3(W4);P P1=X1(I2>>q6);P1=M8(P1);uint Sa;
#ifndef FB
if(P1==I0){Sa=I2;}else
#endif
{Sa=(qd(I0)<<q6)+Ra;}d n;
#ifdef FB
n=clamp(p2(JD,Ta,U2,.0).x,C0(.0),C0(1.));
#else
n=p1;
#endif
int Dg=int(round(n*Ka));o3(W4,Sa+uint(Dg));i Q=E0(.0);
#ifdef N
V4 H1=K8;
#endif
#ifndef FB
if(P1!=I0)
#endif
{d Ua=L8(I2&X8);N8(P1,Ua,Q
#ifdef N
,H1
#endif
m3 a2);}Q.xyz=J2(Q.xyz,Q.w,c0.xy,j.H3,j.I3);
#ifdef R
Q1=Q;
#else
T8(Q a2);
#endif
#ifdef N
Qa(H1 a2);
#endif
p6}
#endif
#ifdef TE
o6(HB){q(n2,c);
#ifdef FD
q(x5,d);q(y5,e);
#endif
#ifdef AB
q(W0,e);
#endif
q(Y1,i);
#ifdef N
q(V3,P);
#endif
#ifdef H
q(N1,P);
#endif
i R1=Y8(RB,v4,n2);d v6=1.;
#ifdef FD
v6=min(x5,v6);
#endif
#ifdef AB
if(AB){d A5=C3(Y4(W0));v6=clamp(A5,C0(.0),v6);}
#endif
uint I2=n3(W4);P P1=X1(I2>>q6);P1=M8(P1);d Ua=L8(I2&X8);i Q;
#ifdef N
V4 H1=K8;
#endif
N8(P1,Ua,Q
#ifdef N
,H1
#endif
m3 a2);
#ifdef N
if(N&&V3!=0u){V4 X0=hd(H1)?H1:Ja(m0);ld(V3,X0,v6);}
#endif
#ifdef FD
if(y5.w!=0.0){c Va=Wa(y5,j.Q8,j.R8);i Xa=p2(ZC,S8,Va,0.0);Xa.xyz*=Xa.w;R1*=Xa;}
#endif
R1*=Y1;
#if!defined(R)&&defined(H)
if(H&&N1!=X3){i a1=K0(n0)*(1.-Q.w)+Q;R1.xyz=R4(l6(R1),a1,N1)*R1.w;}
#endif
R1*=v6;Q=Q*(1.-R1.w)+R1;Q.xyz=J2(Q.xyz,Q.w,c0.xy,j.H3,j.I3);
#ifdef R
Q1=Q;
#else
T8(Q a2);
#endif
#ifdef N
Qa(H1 a2);
#endif
o3(W4,Ra);p6}
#endif
#ifdef UE
o6(HB){
#ifndef R
#ifdef WD
if(WD){x0(n0,unpackUnorm4x8(j.Eg));}
#endif
#ifdef XD
if(XD){x0(n0,v1(RB,G));}
#endif
#ifdef PF
i k=K0(n0);x0(n0,k.zyxw);
#endif
#endif
o3(W4,j.Fg);
#ifdef N
if(N){m1(m0,0u);}
#endif
#ifdef R
discard;
#endif
p6}
#endif
#ifdef WC
#ifdef ID
H2(HB)
#else
o6(HB)
#endif
{uint I2=n3(W4);d w0=L8(I2&X8);P P1=X1(I2>>q6);P1=M8(P1);i Q;N8(P1,w0,Q m3 a2);
#ifdef ID
float rd=1.-Q.w;if(rd!=.0) Q+=K0(n0)*rd;Q1=Q;G3
#else
Q.xyz=J2(Q.xyz,Q.w,c0.xy,j.H3,j.I3);
#ifdef R
Q1=Q;
#else
T8(Q a2);
#endif
p6
#endif
}
#endif
#endif
