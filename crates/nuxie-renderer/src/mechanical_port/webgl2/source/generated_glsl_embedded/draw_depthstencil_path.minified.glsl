f bool w9(){
#ifdef BD
if(BD) return true;
#endif
#ifdef CD
if(CD) return true;
#endif
return false;}
#ifdef BB
e1(e0) f1
#endif
m2 H0 W(0,e,N0);
#ifdef H
MB W(1,d,S0);
#endif
#ifdef GB
H0 W(2,M,O0);
#endif
M1
#ifdef BB
o1(TB,e0,A,N2,G6){V(N0,e);
#ifdef H
V(S0,d);
#endif
#ifdef GB
V(O0,M);
#endif
#ifdef BD
const bool N7=BD;
#else
const bool N7=false;
#endif
#ifdef CD
const bool qe=CD;
#else
const bool qe=false;
#endif
bool ki=(N2&th)!=0;bool li=(N2&vh)!=0;bool x9=(N2&ae)!=0;bool y9=(N2&uh)!=0;int z9=N2&((1<<p9)-1);int Hb,K5,A9;float J0=.0;float n=1.;bool B9=false;if(w9()){int O7=Zd(y9);Hb=z9>>O7;int P7=z9&((1<<O7)-1);int re=rh(y9);K5=P7>>re;int C9=P7&((1<<re)-1);if(N7&&!y9){++C9;}J0=C9<2?1.:-1.;n=(C9==0||C9==3)?.0:1.;A9=int(rb);}else{int O7=Yd(x9);Hb=z9>>O7;int P7=z9&((1<<O7)-1);A9=x9?int(mh):int(rb);B9=!x9&&P7==sh;K5=B9?0:P7;}int L5=min(K5,A9-1);int c4=Hb*A9+L5;O d2=v1(VB,C4(c4));uint Z=d2.w;uint H6=max(Z&yb,1u);O M5=q0(DD,H6-1u);c Q7=uintBitsToFloat(M5.xy);uint f0=M5.z&0xffffu;uint D9=M5.w;X R0=q1(uintBitsToFloat(q0(KB,f0*4u)));O M3=q0(KB,f0*4u+1u);c y2=uintBitsToFloat(M3.xy);float C2=uintBitsToFloat(M3.z);O d4=q0(KB,f0*4u+2u);P I6=X1(d4.x);uint J6=Z&X2;bool R7;if(N7){R7=false;}else if(qe){R7=J6!=0u;if(R7) J0=-J0;}else{R7=J6!=0u&&!x9&&!B9;}if(R7) K5=K5-1;if(K5!=L5){int E9=c4+K5-L5;O S7=v1(VB,C4(E9));if((S7.w&(X2|0xffffu))!=(Z&(X2|0xffffu))){bool T7;if(N7) T7=Q7.x!=.0;else T7=true;if(T7){d2=v1(VB,C4(int(D9)));}}else{d2=S7;}Z=(d2.w&~X2)|J6;}c i0;if(w9()){J0*=sign(determinant(R0));float g1=F9(d2.z);c S1=c(sin(g1),-cos(g1));c K6=uintBitsToFloat(d2.xy);c N3=S1;c se=S1;float Ib=(n==.0)?J0:.0;if(N7){if((Z&C6)!=0u) J0=min(J0,.0);if((Z&xb)!=0u) J0=max(J0,.0);uint j5=Z&r3;if(j5>K7){bool L6=(Z&vb)!=0u;bool Jb=(Z&C6)!=0u;float e2=te(d2.z);float D4=sqrt(max(1.-e2*e2,.0));if(L6==Jb) D4=-D4;X Kb=X(e2,D4,-D4,e2);c k5=D0(Kb,S1);bool G9=j5==Ah||(j5!=ub&&e2<.25);bool ue=(Z&wb)!=0u;if(j5==ub){N3=S1+k5;}else if(ue||!G9){float t=G9?e2:1./e2;N3=k5*t;}if(G9||ue) se=k5;if(!y9&&G9) Ib=.5*J0;}}i0=D0(R0,K6+N3*(J0*C2))+y2;if(Ib!=.0){i0+=sign(D0(se,inverse(R0)))*Ib;}}else{c K6=B9?Q7:uintBitsToFloat(d2.xy);i0=D0(R0,K6)+y2;}c l0=i0;
#ifdef VD
if(j.Ea!=0u){l0.y=float(j.Fa)-l0.y;}
#endif
#ifdef AB
if(AB){X K3=q1(q0(JB,f0*o2+2u));e a4=q0(JB,f0*o2+3u);pb(K3,a4.xy,l0 E4);}
#endif
if(ki){N0=e(.0,.0,.0,.0);
#ifdef GB
O0=M(0.0,0.0,0.0);
#endif
}else{T0 L0=z5(XC,f0);uint w2=L0.x&0xfu;bool G2=false;
#ifdef H
if(H){uint Z1=(L0.x>>4)&0xfu;S0=float(Z1);G2=Z1!=X3;}
#endif
if(w2==Na){N0=unpackUnorm4x8(L0.y);if(G2){N0.w*=n;}else{N0*=n;}}else{X Lb=q1(q0(JB,f0*o2));e U7=q0(JB,f0*o2+1u);float r4=uintBitsToFloat(L0.y);N0=Ga(l0,Lb,U7.xy,U7.zw,w2,r4,n);}
#ifdef GB
if(GB&&(L0.x&he)!=0u){X Mb=q1(q0(JB,f0*o2+4u));e V7=q0(JB,f0*o2+5u);c w3=D0(Mb,l0)+V7.xy;float W7=1.+V7.z;if((L0.x&ie)!=0u){uint D3=(L0.x&ke)>>je;W7=-(1.+float(D3));}O0=M(w3.x,w3.y,W7);}else{O0=M(0.0,0.0,0.0);}
#endif
}e I=U3(i0);
#ifdef NC
I.y=-I.y;
#endif
uint H9;if(w9()){H9=uint(n*254.);if(!li)++H9;}else{H9=0xffu;}I.z=l9(I6,H9);a0(N0);
#ifdef H
a0(S0);
#endif
#ifdef GB
a0(O0);
#endif
A1(I);}
#endif
#ifdef DB
#ifdef QB
I1(HB)
#else
L2(i,HB)
#endif
{q(N0,e);
#ifdef GB
q(O0,M);
#endif
#ifdef H
q(S0,d);
#endif
#ifdef H
P Z1=W2(S0);bool G2=H&&Z1!=X3;
#else
const bool G2=false;
#endif
i k;if(N0.w>=.0){k=Y4(N0);}else{c mi=Wa(N0,j.Q8,j.R8);k=p2(ZC,S8,mi,.0);if(w9()){k.w*=ni(N0);}if(!G2){k.xyz*=k.w;k.w*=ve(N0);}}
#ifdef GB
if(GB&&O0.z>0.0){d Nb=O0.z-1.;i R1=e5(RB,v4,O0.xy,Nb);if(G2) R1=E0(l6(R1),R1.w);k*=R1;}
#endif
#if defined(H)&&!defined(R)
#ifdef QB
i a1=N5(RC);
#else
i a1=O5(MD);
#endif
k.xyz=R4(k.xyz,a1,Z1);k.xyz*=k.w;
#endif
k.xyz=J2(k.xyz,k.w,c0.xy,j.H3,j.I3);
#ifdef GB
if(GB&&O0.z<.0){i oi=e5(RB,v4,O0.xy,C0(.0));uint Ob=uint(-O0.z-1.);d Pb=we(oi,Ob);k=E0(mix(C0(1.),Pb,T4(N0.w)));}
#endif
#ifdef QB
if(k.w!=.0){P5(RC,Pa(k,a1));}
#endif
#ifdef QB
O1
#else
M2(k);
#endif
}
#endif
