#ifdef SD
#ifdef BB
f1(d0) K(0,e,YB);K(1,e,ZB);g1
#endif
n2
#ifdef IB
I0 W(0,e,S);
#else
I0 W(0,F,S);
#endif
m3 W(1,O,J0);P1
#ifdef BB
l1(UB,d0,A,C,r){L(C,A,YB,e);L(C,A,ZB,e);
#ifdef IB
V(S,e);
#else
V(S,F);
#endif
V(J0,O);e I;uint g0;c i0;e T;if(va(YB,ZB,r,g0,i0,T T3)){
#ifdef IB
S=T;
#else
S.xy=J8(T.xy);
#endif
J0=Z1(g0);I=U3(i0);}else{I=e(j.n3,j.n3,j.n3,j.n3);}e0(S);e0(J0);B1(I);}
#endif
#endif
#if defined(DB)||defined(FB)
#ifdef BB
f1(d0) K(0,l4,MB);g1
#endif
n2
#ifdef FB
I0 W(0,c,W2);
#else
NB W(0,d,r1);
#endif
m3 W(1,O,J0);P1
#ifdef BB
l1(UB,d0,A,C,r){L(C,A,MB,M);
#ifdef FB
V(W2,c);
#else
V(r1,d);
#endif
V(J0,O);uint g0;c i0;
#ifdef FB
i0=jd(MB,g0,W2 T3);
#else
i0=kd(MB,g0,r1 T3);
#endif
J0=Z1(g0);e I=U3(i0);
#ifdef FB
e0(W2);
#else
e0(r1);
#endif
e0(J0);B1(I);}
#endif
#endif
#ifdef ED
#ifdef BB
f1(d0) K(0,e,HC);g1 f1(G1) K(wa,e,AC);K(xa,e,VB);K(ya,e,QB);K(za,uint,BC);K(Aa,uint,CC);K(Ba,uint,DC);K(Ca,uint,NC);K(vg,e,TD);K(wg,e,UD);K(xg,e,FD);K(ld,e,GD);g1
#endif
n2 I0 W(0,c,o2);I0 W(1,d,D5);I0 W(2,e,E5);
#ifdef AB
I0 W(3,e,Y0);
#endif
NB W(4,i,a2);
#ifdef N
m3 W(5,O,V3);
#endif
#ifdef H
m3 W(6,O,Q1);
#endif
P1
#ifdef BB
K8(UB,d0,A,G1,j0,C,r){L(C,A,HC,e);L(r,j0,AC,e);L(r,j0,VB,e);L(r,j0,QB,e);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,DC,uint);L(r,j0,NC,uint);L(r,j0,TD,e);L(r,j0,UD,e);L(r,j0,FD,e);L(r,j0,GD,e);V(o2,c);V(D5,d);V(E5,e);
#ifdef AB
V(Y0,e);
#endif
V(a2,i);
#ifdef N
V(V3,O);
#endif
#ifdef H
V(Q1,O);
#endif
bool Da=HC.z==.0||HC.w==.0;D5=Da?.0:1.;c i0=HC.xy;X Q0=v1(AC);X j7=transpose(inverse(Q0));if(!Da){float Ea=Y4*Fa(j7[1])/dot(Q0[1],j7[1]);if(Ea>=.5){i0.x=.5;D5*=Z4(.5/Ea);}else{i0.x+=Ea*HC.z;}float Ga=Y4*Fa(j7[0])/dot(Q0[0],j7[0]);if(Ga>=.5){i0.y=.5;D5*=Z4(.5/Ga);}else{i0.y+=Ga*HC.w;}}X yg=v1(TD);o2=F0(yg,i0)+FD.xy;i0=F0(Q0,i0)+QB.xy;if(Da){c m4=F0(j7,HC.zw);m4*=Fa(m4)/dot(m4,m4);i0+=Y4*m4;}
#ifdef AB
if(AB){Y0=L8(v1(VB),QB.zw,i0);}
#endif
a2=unpackUnorm4x8(BC);
#ifdef N
V3=Z1(CC);
#endif
#ifdef H
Q1=Z1(DC);
#endif
e I=U3(i0);c m0=i0;
#ifdef VD
if(j.Ha!=0u){m0.y=float(j.Ia)-m0.y;}
#endif
uint A2=floatBitsToUint(GD.z);if(A2!=0){X zg=v1(UD);c Ag=FD.zw;E5=Ja(m0,zg,Ag,GD.xy,A2,1.0,1.0);}else{E5=e(.0,.0,.0,.0);}e0(o2);e0(D5);e0(E5);
#ifdef AB
e0(Y0);
#endif
e0(a2);
#ifdef N
e0(V3);
#endif
#ifdef H
e0(Q1);
#endif
B1(I);}
#endif
#elif defined(OB)
#ifdef BB
f1(H3) K(0,c,QC);g1 f1(W3) K(1,c,RC);g1 f1(G1) K(wa,e,AC);K(xa,e,VB);K(ya,e,QB);K(za,uint,BC);K(Aa,uint,CC);K(Ba,uint,DC);K(Ca,uint,NC);K(Ka,e,IC);g1
#endif
n2 I0 W(0,c,o2);
#ifdef AB
I0 W(1,e,Y0);
#endif
NB W(3,i,a2);
#ifdef N
m3 W(4,O,V3);
#endif
#ifdef H
m3 W(5,O,Q1);
#endif
P1
#ifdef BB
k7(UB,H3,I3,W3,o3,G1,j0,C){L(C,I3,QC,c);L(C,o3,RC,c);L(r,j0,AC,e);L(r,j0,VB,e);L(r,j0,QB,e);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,DC,uint);L(r,j0,NC,uint);L(r,j0,IC,e);V(o2,c);
#ifdef AB
V(Y0,e);
#endif
V(a2,i);
#ifdef N
V(V3,O);
#endif
#ifdef H
V(Q1,O);
#endif
X Q0=v1(AC);c i0=F0(Q0,QC)+QB.xy;o2=RC*IC.zw+IC.xy;
#ifdef AB
if(AB){Y0=L8(v1(VB),QB.zw,i0);}
#endif
a2=unpackUnorm4x8(BC);
#ifdef N
V3=Z1(CC);
#endif
#ifdef H
Q1=Z1(DC);
#endif
e I=U3(i0);e0(o2);
#ifdef AB
e0(Y0);
#endif
e0(a2);
#ifdef N
e0(V3);
#endif
#ifdef H
e0(Q1);
#endif
B1(I);}
#endif
#endif
#ifdef NF
#ifdef BB
f1(d0) g1
#endif
n2 P1
#ifdef BB
l1(UB,d0,A,C,r){f0 I2;I2.x=(C&1)==0?j.M8.x:j.M8.z;I2.y=(C&2)==0?j.M8.y:j.M8.w;e I=U3(c(I2));B1(I);}
#endif
#endif
#ifdef SE
#endif
#if defined(TE)&&!defined(Q)
#endif
#ifdef EB
H1
#ifndef Q
#ifdef UE
#define La UE
#else
#define La B2
#endif
#ifdef HD
a5(La,p0);
#else
y0(La,p0);
#endif
#endif
#ifdef VC
#define c5 i
#define Ma L0
#define N8 z0(.0)
#define md(G) ((G).w!=.0)
#ifdef N
#ifndef WC
y0(p3,n0);
#else
a5(p3,n0);
#endif
#endif
#else
#define c5 uint
#define N8 0u
#define Ma m1
#define md(G) ((G)!=0u)
#ifdef N
w1(p3,n0);
#endif
#endif
X2(l7,d5);I1 n4 o4(Na,nd,JB);n6(od,Cg,KB);p4 f uint Dg(float x){return uint(round(x*Oa+Pa));}f d O8(uint x){return Z4(float(x)*pd+(-Pa*pd));}O P8(O g0){
#ifdef OF
g0=min(g0,j.Eg);
#endif
return g0;}
#ifdef N
f void qd(uint C1,c5 Z0,m7(d) n){
#ifdef VC
if(all(lessThan(abs(Z0.xy-unpackUnorm4x8(C1).xy),U2(.25/255.)))) n=min(n,Z0.z);else n=.0;
#else
if(C1==Z0>>16) n=min(n,unpackHalf2x16(Z0).x);else n=.0;
#endif
}
#endif
f void Q8(uint g0,d w0,n1(i) P
#if defined(N)&&!defined(WC)
,m7(c5) J1
#endif
n7 r4){R0 x1=o0(JB,g0*4u+1u).zw;d n=w0;if((x1.x&(Fg|Qa))!=0u){n=abs(n);
#ifdef XC
if(XC&&(x1.x&Qa)!=0u){n=1.-abs(fract(n*.5)*2.+-1.);}
#endif
}n=clamp(n,A0(.0),A0(1.));
#ifdef N
if(N){uint C1=x1.x>>16u;if(C1!=0u){qd(C1,Ma(n0),n);}}
#endif
#ifdef AB
if(AB&&(x1.x&Gg)!=0u){X Q0=v1(o0(KB,g0*p2+2u));e C2=o0(KB,g0*p2+3u);c Hg=F0(Q0,Y)+C2.xy;F rd=J8(abs(Hg)*C2.zw-C2.zw);d F5=clamp(min(rd.x,rd.y)+.5,.0,1.);n=min(n,F5);}
#endif
uint A2=x1.x&0xfu;O c2=Z1((x1.x>>4)&0xfu);
#ifdef H
bool J2=H&&c2!=X3;
#else
const bool J2=false;
#endif
if(A2<=Ra){P=unpackUnorm4x8(x1.y);
#ifdef N
if(N&&A2==R8){
#ifndef WC
#ifdef VC
J1.xy=P.zw;J1.z=n;J1.w=1.;
#else
J1=x1.y|packHalf2x16(U2(n,.0));
#endif
#endif
P=z0(.0);}
#endif
}else{X Q0=v1(o0(KB,g0*p2));e C2=o0(KB,g0*p2+1u);c Ig=F0(Q0,Y)+C2.xy;c S8=C2.zw;bool v4=S8.x<0.0;S8.x=max(0.0,S8.x);c o3=Jg(float(A2),Ig,S8,v4,j.T8,j.U8);P=q2(YC,V8,o3,0.0);if(!J2){P.xyz*=P.w;float w4=uintBitsToFloat(x1.y);P.w*=abs(w4);}}
#if!defined(Q)&&defined(H)
if(J2){if(P.w*n!=.0){i X0=L0(p0);P.xyz=X4(P.xyz,X0,c2);}P.xyz*=P.w;}
#endif
P*=n;}
#if!defined(Q)&&!defined(HD)
f void W8(i P r4){
#ifndef VC
if(P.x+P.y+P.z+P.w==.0) return;if(P.w!=1.) P=Ta(P,L0(p0));
#endif
x0(p0,P);}
#endif
#if defined(N)&&!defined(WC)
f void Ua(c5 J1 r4){
#ifdef VC
x0(n0,J1);
#else
if(J1!=0u) o1(n0,J1);
#endif
}
#endif
#ifdef Q
#define o6 K2
#define p6 J3
#else
#define o6 D1
#define p6 K1
#endif
#ifdef SD
o6(HB){
#ifdef IB
q(S,e);
#else
q(S,F);
#endif
q(J0,O);d X8;
#ifdef IB
if(IB&&sd(S)){X8=e5(S p1);}else if(IB&&td(S)){X8=Y8(S p1);}else
#endif
{X8=min(min(A0(S.x),abs(A0(S.y))),A0(1.));}i P=z0(.0);
#ifdef N
c5 J1=N8;
#endif
uint Z8=Dg(X8);uint ud=(vd(J0)<<q6)|Z8;uint L2=G5(d5,ud);O R1=Z1(L2>>q6);R1=P8(R1);if(R1==J0){if(!r6(S)){Z8+=L2-max(ud,L2);Z8-=Va;H5(d5,Z8);}}else{d w0=O8(L2&a9);Q8(R1,w0,P
#ifdef N
,J1
#endif
r3 d2);}P.xyz=M2(P.xyz,P.w,Y.xy,j.K3,j.L3);
#ifdef Q
S1=P;
#else
W8(P d2);
#endif
#ifdef N
Ua(J1 d2);
#endif
p6}
#endif
#if defined(DB)||defined(FB)
o6(HB){
#ifdef FB
q(W2,c);
#else
q(r1,d);
#endif
q(J0,O);uint L2=v3(d5);O R1=Z1(L2>>q6);R1=P8(R1);uint Wa;
#ifndef FB
if(R1==J0){Wa=L2;}else
#endif
{Wa=(vd(J0)<<q6)+Va;}d n;
#ifdef FB
n=clamp(q2(ID,Xa,W2,.0).x,A0(.0),A0(1.));
#else
n=r1;
#endif
int Kg=int(round(n*Oa));w3(d5,Wa+uint(Kg));i P=z0(.0);
#ifdef N
c5 J1=N8;
#endif
#ifndef FB
if(R1!=J0)
#endif
{d Ya=O8(L2&a9);Q8(R1,Ya,P
#ifdef N
,J1
#endif
r3 d2);}P.xyz=M2(P.xyz,P.w,Y.xy,j.K3,j.L3);
#ifdef Q
S1=P;
#else
W8(P d2);
#endif
#ifdef N
Ua(J1 d2);
#endif
p6}
#endif
#ifdef SE
o6(HB){q(o2,c);
#ifdef ED
q(D5,d);q(E5,e);
#endif
#ifdef AB
q(Y0,e);
#endif
q(a2,i);
#ifdef N
q(V3,O);
#endif
#ifdef H
q(Q1,O);
#endif
i T1=c9(SB,x4,o2);d v6=1.;
#ifdef ED
v6=min(D5,v6);
#endif
#ifdef AB
if(AB){d F5=F3(y4(Y0));v6=clamp(F5,A0(.0),v6);}
#endif
uint L2=v3(d5);O R1=Z1(L2>>q6);R1=P8(R1);d Ya=O8(L2&a9);i P;
#ifdef N
c5 J1=N8;
#endif
Q8(R1,Ya,P
#ifdef N
,J1
#endif
r3 d2);
#ifdef N
if(N&&V3!=0u){c5 Z0=md(J1)?J1:Ma(n0);qd(V3,Z0,v6);}
#endif
#ifdef ED
if(E5.w!=0.0){c Za=ab(E5,j.T8,j.U8);i bb=q2(YC,V8,Za,0.0);bb.xyz*=bb.w;T1*=bb;}
#endif
T1*=a2;
#if!defined(Q)&&defined(H)
if(H&&Q1!=X3){i X0=L0(p0)*(1.-P.w)+P;T1.xyz=X4(m6(T1),X0,Q1)*T1.w;}
#endif
T1*=v6;P=P*(1.-T1.w)+T1;P.xyz=M2(P.xyz,P.w,Y.xy,j.K3,j.L3);
#ifdef Q
S1=P;
#else
W8(P d2);
#endif
#ifdef N
Ua(J1 d2);
#endif
w3(d5,Va);p6}
#endif
#ifdef TE
o6(HB){
#ifndef Q
#ifdef WD
if(WD){x0(p0,unpackUnorm4x8(j.Lg));}
#endif
#ifdef XD
if(XD){x0(p0,q1(SB,D));}
#endif
#ifdef PF
i k=L0(p0);x0(p0,k.zyxw);
#endif
#endif
w3(d5,j.Ng);
#ifdef N
if(N){o1(n0,0u);}
#endif
#ifdef Q
discard;
#endif
p6}
#endif
#ifdef WC
#ifdef HD
K2(HB)
#else
o6(HB)
#endif
{uint L2=v3(d5);d w0=O8(L2&a9);O R1=Z1(L2>>q6);R1=P8(R1);i P;Q8(R1,w0,P r3 d2);
#ifdef HD
float wd=1.-P.w;if(wd!=.0) P+=L0(p0)*wd;S1=P;J3
#else
P.xyz=M2(P.xyz,P.w,Y.xy,j.K3,j.L3);
#ifdef Q
S1=P;
#else
W8(P d2);
#endif
p6
#endif
}
#endif
#endif
