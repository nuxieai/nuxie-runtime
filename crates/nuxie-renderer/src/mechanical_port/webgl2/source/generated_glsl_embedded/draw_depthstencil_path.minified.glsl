f bool z9(){
#ifdef AD
if(AD) return true;
#endif
#ifdef BD
if(BD) return true;
#endif
return false;}
#ifdef BB
f1(d0) g1
#endif
n2 I0 W(0,e,N0);
#ifdef H
NB W(1,d,S0);
#endif
#ifdef GB
I0 W(2,M,O0);
#endif
P1
#ifdef BB
l1(UB,d0,A,O2,F6){V(N0,e);
#ifdef H
V(S0,d);
#endif
#ifdef GB
V(O0,M);
#endif
#ifdef AD
const bool G6=AD;
#else
const bool G6=false;
#endif
#ifdef BD
const bool ue=BD;
#else
const bool ue=false;
#endif
bool si=(O2&Bh)!=0;bool ti=(O2&Dh)!=0;bool A9=(O2&fe)!=0;bool B9=(O2&Ch)!=0;int C9=O2&((1<<v9)-1);int Lb,P5,D9;float K0=.0;float n=1.;bool E9=false;if(z9()){int O7=ee(B9);Lb=C9>>O7;int P7=C9&((1<<O7)-1);int ve=zh(B9);P5=P7>>ve;int F9=P7&((1<<ve)-1);if(G6&&!B9){++F9;}K0=F9<2?1.:-1.;n=(F9==0||F9==3)?.0:1.;D9=int(vb);}else{int O7=de(A9);Lb=C9>>O7;int P7=C9&((1<<O7)-1);D9=A9?int(uh):int(vb);E9=!A9&&P7==Ah;P5=E9?0:P7;}int Q5=min(P5,D9-1);int d4=Lb*D9+Q5;R e2=q1(WB,F4(d4));uint c0=e2.w;uint H6=max(c0&Cb,1u);R R5=o0(CD,H6-1u);c Q7=uintBitsToFloat(R5.xy);uint g0=R5.z&0xffffu;uint G9=R5.w;X Q0=v1(uintBitsToFloat(o0(JB,g0*4u)));R w2=o0(JB,g0*4u+1u);c C2=uintBitsToFloat(w2.xy);O I6=Z1(w2.z>>16u);float f2=.0;if(G6){f2=uintBitsToFloat(o0(JB,g0*4u+2u).x);}uint J6=c0&a3;bool R7;if(G6){R7=false;}else if(ue){R7=J6!=0u;if(R7) K0=-K0;}else{R7=J6!=0u&&!A9&&!E9;}if(R7) P5=P5-1;if(P5!=Q5){int H9=d4+P5-Q5;R S7=q1(WB,F4(H9));if((S7.w&(a3|0xffffu))!=(c0&(a3|0xffffu))){bool T7;if(G6) T7=Q7.x!=.0;else T7=true;if(T7){e2=q1(WB,F4(int(G9)));}}else{e2=S7;}c0=(e2.w&~a3)|J6;}c i0;if(z9()){K0*=sign(determinant(Q0));float h1=I9(e2.z);c M1=c(sin(h1),-cos(h1));c K6=uintBitsToFloat(e2.xy);c P3=M1;c we=M1;float Mb=(n==.0)?K0:.0;if(G6){if((c0&C6)!=0u) K0=min(K0,.0);if((c0&Bb)!=0u) K0=max(K0,.0);uint o5=c0&y3;if(o5>K7){bool L6=(c0&zb)!=0u;bool Nb=(c0&C6)!=0u;float g2=xe(e2.z);float G4=sqrt(max(1.-g2*g2,.0));if(L6==Nb) G4=-G4;X Ob=X(g2,G4,-G4,g2);c p5=F0(Ob,M1);bool J9=o5==Ih||(o5!=yb&&g2<.25);bool ye=(c0&Ab)!=0u;if(o5==yb){P3=M1+p5;}else if(ye||!J9){float t=J9?g2:1./g2;P3=p5*t;}if(J9||ye) we=p5;if(!B9&&J9) Mb=.5*K0;}}i0=F0(Q0,K6+P3*(K0*f2))+C2;if(Mb!=.0){i0+=sign(F0(we,inverse(Q0)))*Mb;}}else{c K6=E9?Q7:uintBitsToFloat(e2.xy);i0=F0(Q0,K6)+C2;}c m0=i0;
#ifdef VD
if(j.Ha!=0u){m0.y=float(j.Ia)-m0.y;}
#endif
#ifdef AB
if(AB){X N3=v1(o0(KB,g0*p2+2u));e a4=o0(KB,g0*p2+3u);tb(N3,a4.xy,m0 H4);}
#endif
if(si){N0=e(.0,.0,.0,.0);
#ifdef GB
O0=M(0.0,0.0,0.0);
#endif
}else{uint A2=w2.z&0xfu;bool J2=false;
#ifdef H
if(H){uint c2=(w2.z>>4)&0xfu;S0=float(c2);J2=c2!=X3;}
#endif
if(A2==Ra){N0=e(unpackUnorm4x8(w2.w));if(J2){N0.w*=n;}else{N0*=n;}}else{X Pb=v1(o0(KB,g0*p2));e U7=o0(KB,g0*p2+1u);float w4=uintBitsToFloat(w2.w);N0=Ja(m0,Pb,U7.xy,U7.zw,A2,w4,n);}
#ifdef GB
if(GB&&(w2.z&le)!=0u){X Qb=v1(o0(KB,g0*p2+4u));e V7=o0(KB,g0*p2+5u);c z3=F0(Qb,m0)+V7.xy;float W7=1.+V7.z;if((w2.z&me)!=0u){uint G3=(w2.z&oe)>>ne;W7=-(1.+float(G3));}O0=M(z3.x,z3.y,W7);}else{O0=M(0.0,0.0,0.0);}
#endif
}e I=U3(i0);
#ifdef OC
I.y=-I.y;
#endif
uint K9;if(z9()){K9=uint(n*254.);if(!ti)++K9;}else{K9=0xffu;}I.z=o9(I6,K9);e0(N0);
#ifdef H
e0(S0);
#endif
#ifdef GB
e0(O0);
#endif
B1(I);}
#endif
#ifdef EB
#ifdef LB
D1(HB)
#else
r2(i,HB)
#endif
{q(N0,e);
#ifdef GB
q(O0,M);
#endif
#ifdef H
q(S0,d);
#endif
#ifdef H
O c2=Z2(S0);bool J2=H&&c2!=X3;
#else
const bool J2=false;
#endif
i k;if(N0.w>=.0){k=y4(N0);}else{c ui=ab(N0,j.T8,j.U8);k=q2(YC,V8,ui,.0);if(z9()){k.w*=vi(N0);}if(!J2){k.xyz*=k.w;k.w*=ze(N0);}}
#ifdef GB
if(GB&&O0.z>0.0){d Rb=O0.z-1.;i T1=j5(SB,x4,O0.xy,Rb);if(J2) T1=z0(m6(T1),T1.w);k*=T1;}
#endif
#if defined(H)&&!defined(Q)
#ifdef LB
i X0=e4(LC);
#else
i X0=I4(LD);
#endif
k.xyz=X4(k.xyz,X0,c2);k.xyz*=k.w;
#elif defined(LB)
i X0=e4(LC);
#endif
k.xyz=M2(k.xyz,k.w,Y.xy,j.K3,j.L3);
#ifdef GB
if(GB&&O0.z<.0){i wi=j5(SB,x4,O0.xy,A0(.0));uint Sb=uint(-O0.z-1.);d Tb=Ae(wi,Sb);k=z0(mix(A0(1.),Tb,Z4(N0.w)));}
#endif
#ifdef LB
if(any(notEqual(k,z0(.0)))){q5(LC,Ta(k,X0));}
#endif
#ifdef LB
K1
#else
v2(k);
#endif
}
#endif
