f bool v9(){
#ifdef AD
if(AD) return true;
#endif
#ifdef BD
if(BD) return true;
#endif
return false;}
#ifdef BB
d1(e0) e1
#endif
n2 H0 W(0,e,M0);
#ifdef H
MB W(1,d,R0);
#endif
#ifdef GB
H0 W(2,M,N0);
#endif
N1
#ifdef BB
n1(TB,e0,A,O2,D6){V(M0,e);
#ifdef H
V(R0,d);
#endif
#ifdef GB
V(N0,M);
#endif
#ifdef AD
const bool E6=AD;
#else
const bool E6=false;
#endif
#ifdef BD
const bool qe=BD;
#else
const bool qe=false;
#endif
bool li=(O2&th)!=0;bool mi=(O2&vh)!=0;bool w9=(O2&ce)!=0;bool x9=(O2&uh)!=0;int y9=O2&((1<<o9)-1);int Jb,I5,z9;float J0=.0;float n=1.;bool A9=false;if(v9()){int N7=be(x9);Jb=y9>>N7;int O7=y9&((1<<N7)-1);int re=rh(x9);I5=O7>>re;int B9=O7&((1<<re)-1);if(E6&&!x9){++B9;}J0=B9<2?1.:-1.;n=(B9==0||B9==3)?.0:1.;z9=int(sb);}else{int N7=ae(w9);Jb=y9>>N7;int O7=y9&((1<<N7)-1);z9=w9?int(mh):int(sb);A9=!w9&&O7==sh;I5=A9?0:O7;}int J5=min(I5,z9-1);int c4=Jb*z9+J5;R d2=v1(VB,C4(c4));uint Z=d2.w;uint F6=max(Z&zb,1u);R K5=n0(CD,F6-1u);c P7=uintBitsToFloat(K5.xy);uint g0=K5.z&0xffffu;uint C9=K5.w;X Q0=p1(uintBitsToFloat(n0(JB,g0*4u)));R r2=n0(JB,g0*4u+1u);c A2=uintBitsToFloat(r2.xy);O G6=X1(r2.z>>16u);float e2=.0;if(E6){e2=uintBitsToFloat(n0(JB,g0*4u+2u).x);}uint H6=Z&Z2;bool Q7;if(E6){Q7=false;}else if(qe){Q7=H6!=0u;if(Q7) J0=-J0;}else{Q7=H6!=0u&&!w9&&!A9;}if(Q7) I5=I5-1;if(I5!=J5){int D9=c4+I5-J5;R R7=v1(VB,C4(D9));if((R7.w&(Z2|0xffffu))!=(Z&(Z2|0xffffu))){bool S7;if(E6) S7=P7.x!=.0;else S7=true;if(S7){d2=v1(VB,C4(int(C9)));}}else{d2=R7;}Z=(d2.w&~Z2)|H6;}c i0;if(v9()){J0*=sign(determinant(Q0));float f1=E9(d2.z);c K1=c(sin(f1),-cos(f1));c I6=uintBitsToFloat(d2.xy);c N3=K1;c se=K1;float Kb=(n==.0)?J0:.0;if(E6){if((Z&z6)!=0u) J0=min(J0,.0);if((Z&yb)!=0u) J0=max(J0,.0);uint j5=Z&v3;if(j5>K7){bool J6=(Z&wb)!=0u;bool Lb=(Z&z6)!=0u;float f2=te(d2.z);float D4=sqrt(max(1.-f2*f2,.0));if(J6==Lb) D4=-D4;X Mb=X(f2,D4,-D4,f2);c k5=D0(Mb,K1);bool F9=j5==Ah||(j5!=vb&&f2<.25);bool ue=(Z&xb)!=0u;if(j5==vb){N3=K1+k5;}else if(ue||!F9){float t=F9?f2:1./f2;N3=k5*t;}if(F9||ue) se=k5;if(!x9&&F9) Kb=.5*J0;}}i0=D0(Q0,I6+N3*(J0*e2))+A2;if(Kb!=.0){i0+=sign(D0(se,inverse(Q0)))*Kb;}}else{c I6=A9?P7:uintBitsToFloat(d2.xy);i0=D0(Q0,I6)+A2;}c l0=i0;
#ifdef UD
if(j.Ea!=0u){l0.y=float(j.Fa)-l0.y;}
#endif
#ifdef AB
if(AB){X L3=p1(n0(KB,g0*p2+2u));e a4=n0(KB,g0*p2+3u);qb(L3,a4.xy,l0 E4);}
#endif
if(li){M0=e(.0,.0,.0,.0);
#ifdef GB
N0=M(0.0,0.0,0.0);
#endif
}else{uint y2=r2.z&0xfu;bool H2=false;
#ifdef H
if(H){uint Z1=(r2.z>>4)&0xfu;R0=float(Z1);H2=Z1!=X3;}
#endif
if(y2==Oa){M0=unpackUnorm4x8(r2.w);if(H2){M0.w*=n;}else{M0*=n;}}else{X Nb=p1(n0(KB,g0*p2));e T7=n0(KB,g0*p2+1u);float r4=uintBitsToFloat(r2.w);M0=Ga(l0,Nb,T7.xy,T7.zw,y2,r4,n);}
#ifdef GB
if(GB&&(r2.z&ie)!=0u){X Ob=p1(n0(KB,g0*p2+4u));e U7=n0(KB,g0*p2+5u);c x3=D0(Ob,l0)+U7.xy;float V7=1.+U7.z;if((r2.z&je)!=0u){uint E3=(r2.z&le)>>ke;V7=-(1.+float(E3));}N0=M(x3.x,x3.y,V7);}else{N0=M(0.0,0.0,0.0);}
#endif
}e I=U3(i0);
#ifdef NC
I.y=-I.y;
#endif
uint G9;if(v9()){G9=uint(n*254.);if(!mi)++G9;}else{G9=0xffu;}I.z=k9(G6,G9);a0(M0);
#ifdef H
a0(R0);
#endif
#ifdef GB
a0(N0);
#endif
A1(I);}
#endif
#ifdef DB
#ifdef QB
I1(HB)
#else
M2(i,HB)
#endif
{q(M0,e);
#ifdef GB
q(N0,M);
#endif
#ifdef H
q(R0,d);
#endif
#ifdef H
O Z1=Y2(R0);bool H2=H&&Z1!=X3;
#else
const bool H2=false;
#endif
i k;if(M0.w>=.0){k=Y4(M0);}else{c ni=Xa(M0,j.P8,j.Q8);k=q2(YC,R8,ni,.0);if(v9()){k.w*=oi(M0);}if(!H2){k.xyz*=k.w;k.w*=ve(M0);}}
#ifdef GB
if(GB&&N0.z>0.0){d Pb=N0.z-1.;i S1=e5(RB,v4,N0.xy,Pb);if(H2) S1=E0(j6(S1),S1.w);k*=S1;}
#endif
#if defined(H)&&!defined(Q)
#ifdef QB
i Z0=L5(RC);
#else
i Z0=M5(LD);
#endif
k.xyz=R4(k.xyz,Z0,Z1);k.xyz*=k.w;
#endif
k.xyz=K2(k.xyz,k.w,c0.xy,j.I3,j.J3);
#ifdef GB
if(GB&&N0.z<.0){i pi=e5(RB,v4,N0.xy,C0(.0));uint Qb=uint(-N0.z-1.);d Rb=we(pi,Qb);k=E0(mix(C0(1.),Rb,T4(M0.w)));}
#endif
#ifdef QB
if(k.w!=.0){N5(RC,Qa(k,Z0));}
#endif
#ifdef QB
P1
#else
N2(k);
#endif
}
#endif
