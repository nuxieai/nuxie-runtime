e bool p9(){
#ifdef ZC
if(ZC) return true;
#endif
#ifdef AD
if(AD) return true;
#endif
return false;}
#ifdef BB
d1(f0) e1
#endif
v2 F0 W(0,f,O0);
#ifdef H
MB W(1,d,P0);
#endif
#ifdef GB
F0 W(2,M,U0);
#endif
k2
#ifdef BB
w1(RB,f0,B,L2,C6){V(O0,f);
#ifdef H
V(P0,d);
#endif
#ifdef GB
V(U0,M);
#endif
#ifdef ZC
const bool K7=ZC;
#else
const bool K7=false;
#endif
#ifdef AD
const bool fe=AD;
#else
const bool fe=false;
#endif
bool Zh=(L2&ih)!=0;bool ai=(L2&kh)!=0;bool q9=(L2&Rd)!=0;bool r9=(L2&jh)!=0;int v9=L2&((1<<l9)-1);int Bb,I5,w9;float I0=.0;float n=1.;bool x9=false;if(p9()){int L7=Qd(r9);Bb=v9>>L7;int M7=v9&((1<<L7)-1);int ge=gh(r9);I5=M7>>ge;int y9=M7&((1<<ge)-1);if(K7&&!r9){++y9;}I0=y9<2?1.:-1.;n=(y9==0||y9==3)?.0:1.;w9=int(mb);}else{int L7=Pd(q9);Bb=v9>>L7;int M7=v9&((1<<L7)-1);w9=q9?int(bh):int(mb);x9=!q9&&M7==hh;I5=x9?0:M7;}int J5=min(I5,w9-1);int Y3=Bb*w9+J5;O a2=q1(UB,y4(Y3));uint a0=a2.w;uint D6=max(a0&tb,1u);O K5=p0(BD,D6-1u);c N7=uintBitsToFloat(K5.xy);uint c0=K5.z&0xffffu;uint z9=K5.w;X N0=o1(uintBitsToFloat(p0(KB,c0*4u)));O L3=p0(KB,c0*4u+1u);c x2=uintBitsToFloat(L3.xy);float B2=uintBitsToFloat(L3.z);O Z3=p0(KB,c0*4u+2u);P E6=S1(Z3.x);uint F6=a0&X2;bool O7;if(K7){O7=false;}else if(fe){O7=F6!=0u;if(O7) I0=-I0;}else{O7=F6!=0u&&!q9&&!x9;}if(O7) I5=I5-1;if(I5!=J5){int A9=Y3+I5-J5;O P7=q1(UB,y4(A9));if((P7.w&(X2|0xffffu))!=(a0&(X2|0xffffu))){bool Q7;if(K7) Q7=N7.x!=.0;else Q7=true;if(Q7){a2=q1(UB,y4(int(z9)));}}else{a2=P7;}a0=(a2.w&~X2)|F6;}c i0;if(p9()){I0*=sign(determinant(N0));float f1=B9(a2.z);c N1=c(sin(f1),-cos(f1));c G6=uintBitsToFloat(a2.xy);c M3=N1;c he=N1;float Cb=(n==.0)?I0:.0;if(K7){if((a0&y6)!=0u) I0=min(I0,.0);if((a0&sb)!=0u) I0=max(I0,.0);uint f5=a0&J3;if(f5>H7){bool H6=(a0&qb)!=0u;bool Db=(a0&y6)!=0u;float c2=ie(a2.z);float z4=sqrt(max(1.-c2*c2,.0));if(H6==Db) z4=-z4;X Eb=X(c2,z4,-z4,c2);c g5=B0(Eb,N1);bool C9=f5==ph||(f5!=pb&&c2<.25);bool je=(a0&rb)!=0u;if(f5==pb){M3=N1+g5;}else if(je||!C9){float t=C9?c2:1./c2;M3=g5*t;}if(C9||je) he=g5;if(!r9&&C9) Cb=.5*I0;}}i0=B0(N0,G6+M3*(I0*B2))+x2;if(Cb!=.0){i0+=sign(B0(he,inverse(N0)))*Cb;}}else{c G6=x9?N7:uintBitsToFloat(a2.xy);i0=B0(N0,G6)+x2;}c l0=i0;
#ifdef SD
if(j.Ba!=0u){l0.y=float(j.Ca)-l0.y;}
#endif
#ifdef AB
if(AB){X I3=o1(p0(JB,c0*m2+2u));f X3=p0(JB,c0*m2+3u);kb(I3,X3.xy,l0 h5);}
#endif
if(Zh){O0=f(.0,.0,.0,.0);
#ifdef GB
U0=M(0.0,0.0,0.0);
#endif
}else{R0 S0=w5(VC,c0);uint w2=S0.x&0xfu;bool F2=false;
#ifdef H
if(H){uint W1=(S0.x>>4)&0xfu;P0=float(W1);F2=W1!=U3;}
#endif
if(w2==Ka){O0=unpackUnorm4x8(S0.y);if(F2){O0.w*=n;}else{O0*=n;}}else{X Fb=o1(p0(JB,c0*m2));f R7=p0(JB,c0*m2+1u);float o4=uintBitsToFloat(S0.y);O0=Da(l0,Fb,R7.xy,R7.zw,w2,o4,n);}
#ifdef GB
if(GB&&(S0.x&Yd)!=0u){X Gb=o1(p0(JB,c0*m2+4u));f S7=p0(JB,c0*m2+5u);c r3=B0(Gb,l0)+S7.xy;U0=M(r3.x,r3.y,1.+S7.z);}else{U0=M(0.0,0.0,0.0);}
#endif
}f I=R3(i0);
#ifdef MC
I.y=-I.y;
#endif
uint D9;if(p9()){D9=uint(n*254.);if(!ai)++D9;}else{D9=0xffu;}I.z=h9(E6,D9);Z(O0);
#ifdef H
Z(P0);
#endif
#ifdef GB
Z(U0);
#endif
x1(I);}
#endif
#ifdef EB
V2(i,IB){q(O0,f);
#ifdef GB
q(U0,M);
#endif
#ifdef H
q(P0,d);
#endif
#ifdef H
P W1=W2(P0);bool F2=H&&W1!=U3;
#else
const bool F2=false;
#endif
i l;if(O0.w>=.0){l=V4(O0);}else{c bi=Sa(O0,j.L8,j.M8);l=n2(XC,N8,bi,.0);if(p9()){l.w*=ci(O0);}if(!F2){l.xyz*=l.w;l.w*=ke(O0);}}
#ifdef GB
if(GB&&U0.z>0.0){d Hb=U0.z-1.;i M1=D5(TB,U4,U0.xy,Hb);if(F2) M1=H0(i6(M1),M1.w);l*=M1;}
#endif
#if defined(H)&&!defined(U)
i z1=L5(KD);l.xyz=N4(l.xyz,z1,W1);l.xyz*=l.w;
#endif
l.xyz=I2(l.xyz,l.w,d0.xy,j.F3,j.G3);K2(l);}
#endif
