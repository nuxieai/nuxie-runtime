f bool r9(){
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
M1
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
bool ki=(O2&sh)!=0;bool li=(O2&uh)!=0;bool v9=(O2&be)!=0;bool w9=(O2&th)!=0;int x9=O2&((1<<n9)-1);int Ib,I5,y9;float J0=.0;float n=1.;bool z9=false;if(r9()){int L7=ae(w9);Ib=x9>>L7;int M7=x9&((1<<L7)-1);int re=qh(w9);I5=M7>>re;int A9=M7&((1<<re)-1);if(E6&&!w9){++A9;}J0=A9<2?1.:-1.;n=(A9==0||A9==3)?.0:1.;y9=int(sb);}else{int L7=Zd(v9);Ib=x9>>L7;int M7=x9&((1<<L7)-1);y9=v9?int(lh):int(sb);z9=!v9&&M7==rh;I5=z9?0:M7;}int J5=min(I5,y9-1);int c4=Ib*y9+J5;R d2=v1(VB,C4(c4));uint Z=d2.w;uint F6=max(Z&zb,1u);R K5=n0(CD,F6-1u);c N7=uintBitsToFloat(K5.xy);uint g0=K5.z&0xffffu;uint B9=K5.w;X Q0=p1(uintBitsToFloat(n0(JB,g0*4u)));R r2=n0(JB,g0*4u+1u);c A2=uintBitsToFloat(r2.xy);O G6=X1(r2.z>>16u);float e2=.0;if(E6){e2=uintBitsToFloat(n0(JB,g0*4u+2u).x);}uint H6=Z&Z2;bool O7;if(E6){O7=false;}else if(qe){O7=H6!=0u;if(O7) J0=-J0;}else{O7=H6!=0u&&!v9&&!z9;}if(O7) I5=I5-1;if(I5!=J5){int C9=c4+I5-J5;R P7=v1(VB,C4(C9));if((P7.w&(Z2|0xffffu))!=(Z&(Z2|0xffffu))){bool Q7;if(E6) Q7=N7.x!=.0;else Q7=true;if(Q7){d2=v1(VB,C4(int(B9)));}}else{d2=P7;}Z=(d2.w&~Z2)|H6;}c i0;if(r9()){J0*=sign(determinant(Q0));float f1=D9(d2.z);c S1=c(sin(f1),-cos(f1));c I6=uintBitsToFloat(d2.xy);c N3=S1;c se=S1;float Jb=(n==.0)?J0:.0;if(E6){if((Z&z6)!=0u) J0=min(J0,.0);if((Z&yb)!=0u) J0=max(J0,.0);uint j5=Z&v3;if(j5>I7){bool J6=(Z&wb)!=0u;bool Kb=(Z&z6)!=0u;float f2=te(d2.z);float D4=sqrt(max(1.-f2*f2,.0));if(J6==Kb) D4=-D4;X Lb=X(f2,D4,-D4,f2);c k5=D0(Lb,S1);bool E9=j5==zh||(j5!=vb&&f2<.25);bool ue=(Z&xb)!=0u;if(j5==vb){N3=S1+k5;}else if(ue||!E9){float t=E9?f2:1./f2;N3=k5*t;}if(E9||ue) se=k5;if(!w9&&E9) Jb=.5*J0;}}i0=D0(Q0,I6+N3*(J0*e2))+A2;if(Jb!=.0){i0+=sign(D0(se,inverse(Q0)))*Jb;}}else{c I6=z9?N7:uintBitsToFloat(d2.xy);i0=D0(Q0,I6)+A2;}c l0=i0;
#ifdef UD
if(j.Ea!=0u){l0.y=float(j.Fa)-l0.y;}
#endif
#ifdef AB
if(AB){X L3=p1(n0(KB,g0*p2+2u));e a4=n0(KB,g0*p2+3u);qb(L3,a4.xy,l0 E4);}
#endif
if(ki){M0=e(.0,.0,.0,.0);
#ifdef GB
N0=M(0.0,0.0,0.0);
#endif
}else{uint y2=r2.z&0xfu;bool H2=false;
#ifdef H
if(H){uint Z1=(r2.z>>4)&0xfu;R0=float(Z1);H2=Z1!=X3;}
#endif
if(y2==Oa){M0=unpackUnorm4x8(r2.w);if(H2){M0.w*=n;}else{M0*=n;}}else{X Mb=p1(n0(KB,g0*p2));e R7=n0(KB,g0*p2+1u);float r4=uintBitsToFloat(r2.w);M0=Ga(l0,Mb,R7.xy,R7.zw,y2,r4,n);}
#ifdef GB
if(GB&&(r2.z&ie)!=0u){X Nb=p1(n0(KB,g0*p2+4u));e S7=n0(KB,g0*p2+5u);c x3=D0(Nb,l0)+S7.xy;float T7=1.+S7.z;if((r2.z&je)!=0u){uint E3=(r2.z&le)>>ke;T7=-(1.+float(E3));}N0=M(x3.x,x3.y,T7);}else{N0=M(0.0,0.0,0.0);}
#endif
}e I=U3(i0);
#ifdef NC
I.y=-I.y;
#endif
uint F9;if(r9()){F9=uint(n*254.);if(!li)++F9;}else{F9=0xffu;}I.z=j9(G6,F9);a0(M0);
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
i k;if(M0.w>=.0){k=Y4(M0);}else{c mi=Xa(M0,j.O8,j.P8);k=q2(YC,Q8,mi,.0);if(r9()){k.w*=ni(M0);}if(!H2){k.xyz*=k.w;k.w*=ve(M0);}}
#ifdef GB
if(GB&&N0.z>0.0){d Ob=N0.z-1.;i R1=e5(RB,v4,N0.xy,Ob);if(H2) R1=E0(j6(R1),R1.w);k*=R1;}
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
if(GB&&N0.z<.0){i oi=e5(RB,v4,N0.xy,C0(.0));uint Pb=uint(-N0.z-1.);d Qb=we(oi,Pb);k=E0(mix(C0(1.),Qb,T4(M0.w)));}
#endif
#ifdef QB
if(k.w!=.0){N5(RC,Qa(k,Z0));}
#endif
#ifdef QB
O1
#else
N2(k);
#endif
}
#endif
