e bool v9(){
#ifdef BD
if(BD) return true;
#endif
#ifdef CD
if(CD) return true;
#endif
return false;}
#ifdef BB
e1(f0) f1
#endif
v2 F0 W(0,f,P0);
#ifdef H
MB W(1,d,Q0);
#endif
#ifdef GB
F0 W(2,M,U0);
#endif
k2
#ifdef BB
y1(RB,f0,B,M2,C6){V(P0,f);
#ifdef H
V(Q0,d);
#endif
#ifdef GB
V(U0,M);
#endif
#ifdef BD
const bool N7=BD;
#else
const bool N7=false;
#endif
#ifdef CD
const bool he=CD;
#else
const bool he=false;
#endif
bool ci=(M2&kh)!=0;bool di=(M2&mh)!=0;bool w9=(M2&Td)!=0;bool x9=(M2&lh)!=0;int y9=M2&((1<<o9)-1);int Fb,J5,z9;float I0=.0;float n=1.;bool A9=false;if(v9()){int O7=Sd(x9);Fb=y9>>O7;int P7=y9&((1<<O7)-1);int ie=ih(x9);J5=P7>>ie;int B9=P7&((1<<ie)-1);if(N7&&!x9){++B9;}I0=B9<2?1.:-1.;n=(B9==0||B9==3)?.0:1.;z9=int(qb);}else{int O7=Rd(w9);Fb=y9>>O7;int P7=y9&((1<<O7)-1);z9=w9?int(dh):int(qb);A9=!w9&&P7==jh;J5=A9?0:P7;}int K5=min(J5,z9-1);int Y3=Fb*z9+K5;O a2=r1(UB,y4(Y3));uint a0=a2.w;uint D6=max(a0&xb,1u);O L5=p0(DD,D6-1u);c Q7=uintBitsToFloat(L5.xy);uint c0=L5.z&0xffffu;uint C9=L5.w;X N0=p1(uintBitsToFloat(p0(KB,c0*4u)));O L3=p0(KB,c0*4u+1u);c y2=uintBitsToFloat(L3.xy);float C2=uintBitsToFloat(L3.z);O Z3=p0(KB,c0*4u+2u);P E6=U1(Z3.x);uint F6=a0&X2;bool R7;if(N7){R7=false;}else if(he){R7=F6!=0u;if(R7) I0=-I0;}else{R7=F6!=0u&&!w9&&!A9;}if(R7) J5=J5-1;if(J5!=K5){int D9=Y3+J5-K5;O S7=r1(UB,y4(D9));if((S7.w&(X2|0xffffu))!=(a0&(X2|0xffffu))){bool T7;if(N7) T7=Q7.x!=.0;else T7=true;if(T7){a2=r1(UB,y4(int(C9)));}}else{a2=S7;}a0=(a2.w&~X2)|F6;}c i0;if(v9()){I0*=sign(determinant(N0));float g1=E9(a2.z);c P1=c(sin(g1),-cos(g1));c G6=uintBitsToFloat(a2.xy);c M3=P1;c je=P1;float Gb=(n==.0)?I0:.0;if(N7){if((a0&y6)!=0u) I0=min(I0,.0);if((a0&wb)!=0u) I0=max(I0,.0);uint f5=a0&J3;if(f5>K7){bool H6=(a0&ub)!=0u;bool Hb=(a0&y6)!=0u;float c2=ke(a2.z);float z4=sqrt(max(1.-c2*c2,.0));if(H6==Hb) z4=-z4;X Ib=X(c2,z4,-z4,c2);c g5=C0(Ib,P1);bool F9=f5==rh||(f5!=tb&&c2<.25);bool le=(a0&vb)!=0u;if(f5==tb){M3=P1+g5;}else if(le||!F9){float t=F9?c2:1./c2;M3=g5*t;}if(F9||le) je=g5;if(!x9&&F9) Gb=.5*I0;}}i0=C0(N0,G6+M3*(I0*C2))+y2;if(Gb!=.0){i0+=sign(C0(je,inverse(N0)))*Gb;}}else{c G6=A9?Q7:uintBitsToFloat(a2.xy);i0=C0(N0,G6)+y2;}c l0=i0;
#ifdef UD
if(j.Da!=0u){l0.y=float(j.Ea)-l0.y;}
#endif
#ifdef AB
if(AB){X I3=p1(p0(JB,c0*m2+2u));f X3=p0(JB,c0*m2+3u);ob(I3,X3.xy,l0 h5);}
#endif
if(ci){P0=f(.0,.0,.0,.0);
#ifdef GB
U0=M(0.0,0.0,0.0);
#endif
}else{R0 S0=x5(XC,c0);uint w2=S0.x&0xfu;bool G2=false;
#ifdef H
if(H){uint W1=(S0.x>>4)&0xfu;Q0=float(W1);G2=W1!=U3;}
#endif
if(w2==Ma){P0=unpackUnorm4x8(S0.y);if(G2){P0.w*=n;}else{P0*=n;}}else{X Jb=p1(p0(JB,c0*m2));f U7=p0(JB,c0*m2+1u);float o4=uintBitsToFloat(S0.y);P0=Fa(l0,Jb,U7.xy,U7.zw,w2,o4,n);}
#ifdef GB
if(GB&&(S0.x&ae)!=0u){X Kb=p1(p0(JB,c0*m2+4u));f V7=p0(JB,c0*m2+5u);c v3=C0(Kb,l0)+V7.xy;U0=M(v3.x,v3.y,1.+V7.z);}else{U0=M(0.0,0.0,0.0);}
#endif
}f I=R3(i0);
#ifdef NC
I.y=-I.y;
#endif
uint G9;if(v9()){G9=uint(n*254.);if(!di)++G9;}else{G9=0xffu;}I.z=k9(E6,G9);Z(P0);
#ifdef H
Z(Q0);
#endif
#ifdef GB
Z(U0);
#endif
z1(I);}
#endif
#ifdef EB
V2(i,IB){q(P0,f);
#ifdef GB
q(U0,M);
#endif
#ifdef H
q(Q0,d);
#endif
#ifdef H
P W1=W2(Q0);bool G2=H&&W1!=U3;
#else
const bool G2=false;
#endif
i k;if(P0.w>=.0){k=V4(P0);}else{c ei=Va(P0,j.O8,j.P8);k=n2(ZC,Q8,ei,.0);if(v9()){k.w*=fi(P0);}if(!G2){k.xyz*=k.w;k.w*=me(P0);}}
#ifdef GB
if(GB&&U0.z>0.0){d Lb=U0.z-1.;i O1=E5(TB,U4,U0.xy,Lb);if(G2) O1=H0(i6(O1),O1.w);k*=O1;}
#endif
#if defined(H)&&!defined(U)
#ifdef KC
i a1=I6(RC);
#else
i a1=M5(MD);
#endif
k.xyz=N4(k.xyz,a1,W1);k.xyz*=k.w;
#endif
k.xyz=J2(k.xyz,k.w,d0.xy,j.F3,j.G3);
#ifdef KC
if(k.w!=.0){J6(RC,Oa(k,a1));}
#endif
L2(k);}
#endif
