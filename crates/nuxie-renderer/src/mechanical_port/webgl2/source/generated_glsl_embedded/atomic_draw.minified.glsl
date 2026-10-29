#ifdef RD
#ifdef BB
d1(e0) K(0,e,YB);K(1,e,ZB);e1
#endif
n2
#ifdef IB
H0 W(0,e,S);
#else
H0 W(0,E,S);
#endif
i3 W(1,O,I0);M1
#ifdef BB
n1(TB,e0,A,C,r){L(C,A,YB,e);L(C,A,ZB,e);
#ifdef IB
V(S,e);
#else
V(S,E);
#endif
V(I0,O);e I;uint g0;c i0;e T;if(sa(YB,ZB,r,g0,i0,T T3)){
#ifdef IB
S=T;
#else
S.xy=E8(T.xy);
#endif
I0=X1(g0);I=U3(i0);}else{I=e(j.j3,j.j3,j.j3,j.j3);}a0(S);a0(I0);A1(I);}
#endif
#endif
#if defined(EB)||defined(FB)
#ifdef BB
d1(e0) K(0,j4,LB);e1
#endif
n2
#ifdef FB
H0 W(0,c,W2);
#else
MB W(0,d,o1);
#endif
i3 W(1,O,I0);M1
#ifdef BB
n1(TB,e0,A,C,r){L(C,A,LB,M);
#ifdef FB
V(W2,c);
#else
V(o1,d);
#endif
V(I0,O);uint g0;c i0;
#ifdef FB
i0=fd(LB,g0,W2 T3);
#else
i0=gd(LB,g0,o1 T3);
#endif
I0=X1(g0);e I=U3(i0);
#ifdef FB
a0(W2);
#else
a0(o1);
#endif
a0(I0);A1(I);}
#endif
#endif
#ifdef ED
#ifdef BB
d1(e0) K(0,e,HC);e1 d1(E1) K(ta,e,AC);K(ua,e,UB);K(va,e,PB);K(wa,uint,BC);K(xa,uint,CC);K(ya,uint,DC);K(za,uint,MC);K(ng,e,SD);K(og,e,TD);K(pg,e,FD);K(hd,e,GD);e1
#endif
n2 H0 W(0,c,o2);H0 W(1,d,w5);H0 W(2,e,x5);
#ifdef AB
H0 W(3,e,V0);
#endif
MB W(4,i,Y1);
#ifdef N
i3 W(5,O,V3);
#endif
#ifdef H
i3 W(6,O,N1);
#endif
M1
#ifdef BB
F8(TB,e0,A,E1,j0,C,r){L(C,A,HC,e);L(r,j0,AC,e);L(r,j0,UB,e);L(r,j0,PB,e);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,DC,uint);L(r,j0,MC,uint);L(r,j0,SD,e);L(r,j0,TD,e);L(r,j0,FD,e);L(r,j0,GD,e);V(o2,c);V(w5,d);V(x5,e);
#ifdef AB
V(V0,e);
#endif
V(Y1,i);
#ifdef N
V(V3,O);
#endif
#ifdef H
V(N1,O);
#endif
bool Aa=HC.z==.0||HC.w==.0;w5=Aa?.0:1.;c i0=HC.xy;X Q0=p1(AC);X h7=transpose(inverse(Q0));if(!Aa){float Ba=S4*Ca(h7[1])/dot(Q0[1],h7[1]);if(Ba>=.5){i0.x=.5;w5*=T4(.5/Ba);}else{i0.x+=Ba*HC.z;}float Da=S4*Ca(h7[0])/dot(Q0[0],h7[0]);if(Da>=.5){i0.y=.5;w5*=T4(.5/Da);}else{i0.y+=Da*HC.w;}}X qg=p1(SD);o2=D0(qg,i0)+FD.xy;i0=D0(Q0,i0)+PB.xy;if(Aa){c k4=D0(h7,HC.zw);k4*=Ca(k4)/dot(k4,k4);i0+=S4*k4;}
#ifdef AB
if(AB){V0=G8(p1(UB),PB.zw,i0);}
#endif
Y1=unpackUnorm4x8(BC);
#ifdef N
V3=X1(CC);
#endif
#ifdef H
N1=X1(DC);
#endif
e I=U3(i0);c l0=i0;
#ifdef UD
if(j.Ea!=0u){l0.y=float(j.Fa)-l0.y;}
#endif
uint y2=floatBitsToUint(GD.z);if(y2!=0){X rg=p1(TD);c sg=FD.zw;x5=Ga(l0,rg,sg,GD.xy,y2,1.0,1.0);}else{x5=e(.0,.0,.0,.0);}a0(o2);a0(w5);a0(x5);
#ifdef AB
a0(V0);
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
d1(F3) K(0,c,PC);e1 d1(W3) K(1,c,QC);e1 d1(E1) K(ta,e,AC);K(ua,e,UB);K(va,e,PB);K(wa,uint,BC);K(xa,uint,CC);K(ya,uint,DC);K(za,uint,MC);K(Ha,e,IC);e1
#endif
n2 H0 W(0,c,o2);
#ifdef AB
H0 W(1,e,V0);
#endif
MB W(3,i,Y1);
#ifdef N
i3 W(4,O,V3);
#endif
#ifdef H
i3 W(5,O,N1);
#endif
M1
#ifdef BB
i7(TB,F3,G3,W3,k3,E1,j0,C){L(C,G3,PC,c);L(C,k3,QC,c);L(r,j0,AC,e);L(r,j0,UB,e);L(r,j0,PB,e);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,DC,uint);L(r,j0,MC,uint);L(r,j0,IC,e);V(o2,c);
#ifdef AB
V(V0,e);
#endif
V(Y1,i);
#ifdef N
V(V3,O);
#endif
#ifdef H
V(N1,O);
#endif
X Q0=p1(AC);c i0=D0(Q0,PC)+PB.xy;o2=QC*IC.zw+IC.xy;
#ifdef AB
if(AB){V0=G8(p1(UB),PB.zw,i0);}
#endif
Y1=unpackUnorm4x8(BC);
#ifdef N
V3=X1(CC);
#endif
#ifdef H
N1=X1(DC);
#endif
e I=U3(i0);a0(o2);
#ifdef AB
a0(V0);
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
#ifdef MF
#ifdef BB
d1(e0) e1
#endif
n2 M1
#ifdef BB
n1(TB,e0,A,C,r){f0 G2;G2.x=(C&1)==0?j.H8.x:j.H8.z;G2.y=(C&2)==0?j.H8.y:j.H8.w;e I=U3(c(G2));A1(I);}
#endif
#endif
#ifdef SE
#endif
#if defined(TE)&&!defined(Q)
#endif
#ifdef DB
F1
#ifndef Q
#ifdef UE
#define Ia UE
#else
#define Ia z2
#endif
#ifdef HD
U4(Ia,o0);
#else
y0(Ia,o0);
#endif
#endif
#ifdef VC
#define V4 i
#define Ja K0
#define I8 E0(.0)
#define id(F) ((F).w!=.0)
#ifdef N
#ifndef WC
y0(l3,m0);
#else
U4(l3,m0);
#endif
#endif
#else
#define V4 uint
#define I8 0u
#define Ja j1
#define id(F) ((F)!=0u)
#ifdef N
q1(l3,m0);
#endif
#endif
X2(j7,W4);G1 l4 m4(Ka,jd,JB);k6(kd,ug,KB);n4 f uint vg(float x){return uint(round(x*La+Ma));}f d J8(uint x){return T4(float(x)*ld+(-Ma*ld));}O K8(O g0){
#ifdef NF
g0=min(g0,j.wg);
#endif
return g0;}
#ifdef N
f void md(uint B1,V4 W0,k7(d) n){
#ifdef VC
if(all(lessThan(abs(W0.xy-unpackUnorm4x8(B1).xy),U2(.25/255.)))) n=min(n,W0.z);else n=.0;
#else
if(B1==W0>>16) n=min(n,unpackHalf2x16(W0).x);else n=.0;
#endif
}
#endif
f void L8(uint g0,d w0,k1(i) P
#if defined(N)&&!defined(WC)
,k7(V4) H1
#endif
l7 p4){U0 r1=n0(JB,g0*4u+1u).zw;d n=w0;if((r1.x&(xg|Na))!=0u){n=abs(n);
#ifdef XC
if(XC&&(r1.x&Na)!=0u){n=1.-abs(fract(n*.5)*2.+-1.);}
#endif
}n=clamp(n,C0(.0),C0(1.));
#ifdef N
if(N){uint B1=r1.x>>16u;if(B1!=0u){md(B1,Ja(m0),n);}}
#endif
#ifdef AB
if(AB&&(r1.x&yg)!=0u){X Q0=p1(n0(KB,g0*p2+2u));e A2=n0(KB,g0*p2+3u);c zg=D0(Q0,c0)+A2.xy;E nd=E8(abs(zg)*A2.zw-A2.zw);d y5=clamp(min(nd.x,nd.y)+.5,.0,1.);n=min(n,y5);}
#endif
uint y2=r1.x&0xfu;O Z1=X1((r1.x>>4)&0xfu);
#ifdef H
bool H2=H&&Z1!=X3;
#else
const bool H2=false;
#endif
if(y2<=Oa){P=unpackUnorm4x8(r1.y);
#ifdef N
if(N&&y2==M8){
#ifndef WC
#ifdef VC
H1.xy=P.zw;H1.z=n;H1.w=1.;
#else
H1=r1.y|packHalf2x16(U2(n,.0));
#endif
#endif
P=E0(.0);}
#endif
}else{X Q0=p1(n0(KB,g0*p2));e A2=n0(KB,g0*p2+1u);c Ag=D0(Q0,c0)+A2.xy;c N8=A2.zw;bool q4=N8.x<0.0;N8.x=max(0.0,N8.x);c k3=Bg(float(y2),Ag,N8,q4,j.O8,j.P8);P=q2(YC,Q8,k3,0.0);if(!H2){P.xyz*=P.w;float r4=uintBitsToFloat(r1.y);P.w*=abs(r4);}}
#if!defined(Q)&&defined(H)
if(H2){if(P.w*n!=.0){i Z0=K0(o0);P.xyz=R4(P.xyz,Z0,Z1);}P.xyz*=P.w;}
#endif
P*=n;}
#if!defined(Q)&&!defined(HD)
f void R8(i P p4){
#ifndef VC
if(P.x+P.y+P.z+P.w==.0) return;if(P.w!=1.) P=Qa(P,K0(o0));
#endif
x0(o0,P);}
#endif
#if defined(N)&&!defined(WC)
f void Ra(V4 H1 p4){
#ifdef VC
x0(m0,H1);
#else
if(H1!=0u) l1(m0,H1);
#endif
}
#endif
#ifdef Q
#define l6 I2
#define m6 H3
#else
#define l6 I1
#define m6 O1
#endif
#ifdef RD
l6(HB){
#ifdef IB
q(S,e);
#else
q(S,E);
#endif
q(I0,O);d S8;
#ifdef IB
if(IB&&od(S)){S8=X4(S m1);}else if(IB&&pd(S)){S8=T8(S m1);}else
#endif
{S8=min(min(C0(S.x),abs(C0(S.y))),C0(1.));}i P=E0(.0);
#ifdef N
V4 H1=I8;
#endif
uint U8=vg(S8);uint qd=(rd(I0)<<n6)|U8;uint J2=z5(W4,qd);O P1=X1(J2>>n6);P1=K8(P1);if(P1==I0){if(!o6(S)){U8+=J2-max(qd,J2);U8-=Sa;A5(W4,U8);}}else{d w0=J8(J2&V8);L8(P1,w0,P
#ifdef N
,H1
#endif
n3 a2);}P.xyz=K2(P.xyz,P.w,c0.xy,j.I3,j.J3);
#ifdef Q
Q1=P;
#else
R8(P a2);
#endif
#ifdef N
Ra(H1 a2);
#endif
m6}
#endif
#if defined(EB)||defined(FB)
l6(HB){
#ifdef FB
q(W2,c);
#else
q(o1,d);
#endif
q(I0,O);uint J2=o3(W4);O P1=X1(J2>>n6);P1=K8(P1);uint Ta;
#ifndef FB
if(P1==I0){Ta=J2;}else
#endif
{Ta=(rd(I0)<<n6)+Sa;}d n;
#ifdef FB
n=clamp(q2(ID,Ua,W2,.0).x,C0(.0),C0(1.));
#else
n=o1;
#endif
int Cg=int(round(n*La));p3(W4,Ta+uint(Cg));i P=E0(.0);
#ifdef N
V4 H1=I8;
#endif
#ifndef FB
if(P1!=I0)
#endif
{d Va=J8(J2&V8);L8(P1,Va,P
#ifdef N
,H1
#endif
n3 a2);}P.xyz=K2(P.xyz,P.w,c0.xy,j.I3,j.J3);
#ifdef Q
Q1=P;
#else
R8(P a2);
#endif
#ifdef N
Ra(H1 a2);
#endif
m6}
#endif
#ifdef SE
l6(HB){q(o2,c);
#ifdef ED
q(w5,d);q(x5,e);
#endif
#ifdef AB
q(V0,e);
#endif
q(Y1,i);
#ifdef N
q(V3,O);
#endif
#ifdef H
q(N1,O);
#endif
i R1=W8(RB,v4,o2);d p6=1.;
#ifdef ED
p6=min(w5,p6);
#endif
#ifdef AB
if(AB){d y5=D3(Y4(V0));p6=clamp(y5,C0(.0),p6);}
#endif
uint J2=o3(W4);O P1=X1(J2>>n6);P1=K8(P1);d Va=J8(J2&V8);i P;
#ifdef N
V4 H1=I8;
#endif
L8(P1,Va,P
#ifdef N
,H1
#endif
n3 a2);
#ifdef N
if(N&&V3!=0u){V4 W0=id(H1)?H1:Ja(m0);md(V3,W0,p6);}
#endif
#ifdef ED
if(x5.w!=0.0){c Wa=Xa(x5,j.O8,j.P8);i Ya=q2(YC,Q8,Wa,0.0);Ya.xyz*=Ya.w;R1*=Ya;}
#endif
R1*=Y1;
#if!defined(Q)&&defined(H)
if(H&&N1!=X3){i Z0=K0(o0)*(1.-P.w)+P;R1.xyz=R4(j6(R1),Z0,N1)*R1.w;}
#endif
R1*=p6;P=P*(1.-R1.w)+R1;P.xyz=K2(P.xyz,P.w,c0.xy,j.I3,j.J3);
#ifdef Q
Q1=P;
#else
R8(P a2);
#endif
#ifdef N
Ra(H1 a2);
#endif
p3(W4,Sa);m6}
#endif
#ifdef TE
l6(HB){
#ifndef Q
#ifdef VD
if(VD){x0(o0,unpackUnorm4x8(j.Dg));}
#endif
#ifdef WD
if(WD){x0(o0,v1(RB,G));}
#endif
#ifdef OF
i k=K0(o0);x0(o0,k.zyxw);
#endif
#endif
p3(W4,j.Eg);
#ifdef N
if(N){l1(m0,0u);}
#endif
#ifdef Q
discard;
#endif
m6}
#endif
#ifdef WC
#ifdef HD
I2(HB)
#else
l6(HB)
#endif
{uint J2=o3(W4);d w0=J8(J2&V8);O P1=X1(J2>>n6);P1=K8(P1);i P;L8(P1,w0,P n3 a2);
#ifdef HD
float sd=1.-P.w;if(sd!=.0) P+=K0(o0)*sd;Q1=P;H3
#else
P.xyz=K2(P.xyz,P.w,c0.xy,j.I3,j.J3);
#ifdef Q
Q1=P;
#else
R8(P a2);
#endif
m6
#endif
}
#endif
#endif
