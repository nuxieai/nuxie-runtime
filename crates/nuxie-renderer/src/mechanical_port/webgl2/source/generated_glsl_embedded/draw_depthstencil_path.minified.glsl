f bool v9(){
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
m2 F0 W(0,e,P0);
#ifdef H
MB W(1,d,Q0);
#endif
#ifdef GB
F0 W(2,M,V0);
#endif
M1
#ifdef BB
o1(SB,e0,A,N2,G6){V(P0,e);
#ifdef H
V(Q0,d);
#endif
#ifdef GB
V(V0,M);
#endif
#ifdef BD
const bool N7=BD;
#else
const bool N7=false;
#endif
#ifdef CD
const bool ke=CD;
#else
const bool ke=false;
#endif
bool gi=(N2&nh)!=0;bool hi=(N2&ph)!=0;bool w9=(N2&Xd)!=0;bool x9=(N2&oh)!=0;int y9=N2&((1<<o9)-1);int Gb,K5,z9;float I0=.0;float n=1.;bool A9=false;if(v9()){int O7=Wd(x9);Gb=y9>>O7;int P7=y9&((1<<O7)-1);int le=lh(x9);K5=P7>>le;int B9=P7&((1<<le)-1);if(N7&&!x9){++B9;}I0=B9<2?1.:-1.;n=(B9==0||B9==3)?.0:1.;z9=int(qb);}else{int O7=Vd(w9);Gb=y9>>O7;int P7=y9&((1<<O7)-1);z9=w9?int(gh):int(qb);A9=!w9&&P7==mh;K5=A9?0:P7;}int L5=min(K5,z9-1);int a4=Gb*z9+L5;O d2=v1(VB,A4(a4));uint Z=d2.w;uint H6=max(Z&xb,1u);O M5=q0(DD,H6-1u);c Q7=uintBitsToFloat(M5.xy);uint f0=M5.z&0xffffu;uint C9=M5.w;X O0=q1(uintBitsToFloat(q0(KB,f0*4u)));O L3=q0(KB,f0*4u+1u);c y2=uintBitsToFloat(L3.xy);float C2=uintBitsToFloat(L3.z);O c4=q0(KB,f0*4u+2u);P I6=X1(c4.x);uint J6=Z&X2;bool R7;if(N7){R7=false;}else if(ke){R7=J6!=0u;if(R7) I0=-I0;}else{R7=J6!=0u&&!w9&&!A9;}if(R7) K5=K5-1;if(K5!=L5){int D9=a4+K5-L5;O S7=v1(VB,A4(D9));if((S7.w&(X2|0xffffu))!=(Z&(X2|0xffffu))){bool T7;if(N7) T7=Q7.x!=.0;else T7=true;if(T7){d2=v1(VB,A4(int(C9)));}}else{d2=S7;}Z=(d2.w&~X2)|J6;}c i0;if(v9()){I0*=sign(determinant(O0));float g1=E9(d2.z);c S1=c(sin(g1),-cos(g1));c K6=uintBitsToFloat(d2.xy);c M3=S1;c me=S1;float Hb=(n==.0)?I0:.0;if(N7){if((Z&C6)!=0u) I0=min(I0,.0);if((Z&wb)!=0u) I0=max(I0,.0);uint h5=Z&r3;if(h5>K7){bool L6=(Z&ub)!=0u;bool Ib=(Z&C6)!=0u;float e2=ne(d2.z);float B4=sqrt(max(1.-e2*e2,.0));if(L6==Ib) B4=-B4;X Jb=X(e2,B4,-B4,e2);c i5=C0(Jb,S1);bool F9=h5==uh||(h5!=tb&&e2<.25);bool oe=(Z&vb)!=0u;if(h5==tb){M3=S1+i5;}else if(oe||!F9){float t=F9?e2:1./e2;M3=i5*t;}if(F9||oe) me=i5;if(!x9&&F9) Hb=.5*I0;}}i0=C0(O0,K6+M3*(I0*C2))+y2;if(Hb!=.0){i0+=sign(C0(me,inverse(O0)))*Hb;}}else{c K6=A9?Q7:uintBitsToFloat(d2.xy);i0=C0(O0,K6)+y2;}c l0=i0;
#ifdef VD
if(j.Da!=0u){l0.y=float(j.Ea)-l0.y;}
#endif
#ifdef AB
if(AB){X J3=q1(q0(JB,f0*o2+2u));e Z3=q0(JB,f0*o2+3u);ob(J3,Z3.xy,l0 C4);}
#endif
if(gi){P0=e(.0,.0,.0,.0);
#ifdef GB
V0=M(0.0,0.0,0.0);
#endif
}else{R0 S0=y5(XC,f0);uint w2=S0.x&0xfu;bool G2=false;
#ifdef H
if(H){uint Z1=(S0.x>>4)&0xfu;Q0=float(Z1);G2=Z1!=W3;}
#endif
if(w2==Ma){P0=unpackUnorm4x8(S0.y);if(G2){P0.w*=n;}else{P0*=n;}}else{X Kb=q1(q0(JB,f0*o2));e U7=q0(JB,f0*o2+1u);float q4=uintBitsToFloat(S0.y);P0=Fa(l0,Kb,U7.xy,U7.zw,w2,q4,n);}
#ifdef GB
if(GB&&(S0.x&ee)!=0u){X Lb=q1(q0(JB,f0*o2+4u));e V7=q0(JB,f0*o2+5u);c w3=C0(Lb,l0)+V7.xy;V0=M(w3.x,w3.y,1.+V7.z);}else{V0=M(0.0,0.0,0.0);}
#endif
}e I=T3(i0);
#ifdef NC
I.y=-I.y;
#endif
uint G9;if(v9()){G9=uint(n*254.);if(!hi)++G9;}else{G9=0xffu;}I.z=k9(I6,G9);a0(P0);
#ifdef H
a0(Q0);
#endif
#ifdef GB
a0(V0);
#endif
A1(I);}
#endif
#ifdef EB
#ifdef QB
I1(HB)
#else
L2(i,HB)
#endif
{q(P0,e);
#ifdef GB
q(V0,M);
#endif
#ifdef H
q(Q0,d);
#endif
#ifdef H
P Z1=W2(Q0);bool G2=H&&Z1!=W3;
#else
const bool G2=false;
#endif
i k;if(P0.w>=.0){k=X4(P0);}else{c ii=Va(P0,j.O8,j.P8);k=p2(ZC,Q8,ii,.0);if(v9()){k.w*=ji(P0);}if(!G2){k.xyz*=k.w;k.w*=pe(P0);}}
#ifdef GB
if(GB&&V0.z>0.0){d Mb=V0.z-1.;i R1=F5(UB,W4,V0.xy,Mb);if(G2) R1=H0(l6(R1),R1.w);k*=R1;}
#endif
#if defined(H)&&!defined(R)
#ifdef QB
i a1=N5(RC);
#else
i a1=O5(MD);
#endif
k.xyz=P4(k.xyz,a1,Z1);k.xyz*=k.w;
#endif
k.xyz=J2(k.xyz,k.w,c0.xy,j.G3,j.H3);
#ifdef QB
if(k.w!=.0){P5(RC,Oa(k,a1));}
#endif
#ifdef QB
O1
#else
M2(k);
#endif
}
#endif
