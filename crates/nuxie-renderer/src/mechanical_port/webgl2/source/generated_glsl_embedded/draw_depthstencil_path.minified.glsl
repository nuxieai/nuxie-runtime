e bool q9(){
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
const bool L7=ZC;
#else
const bool L7=false;
#endif
#ifdef AD
const bool fe=AD;
#else
const bool fe=false;
#endif
bool ai=(L2&ih)!=0;bool bi=(L2&kh)!=0;bool r9=(L2&Rd)!=0;bool v9=(L2&jh)!=0;int w9=L2&((1<<m9)-1);int Cb,J5,x9;float I0=.0;float n=1.;bool y9=false;if(q9()){int M7=Qd(v9);Cb=w9>>M7;int N7=w9&((1<<M7)-1);int ge=gh(v9);J5=N7>>ge;int z9=N7&((1<<ge)-1);if(L7&&!v9){++z9;}I0=z9<2?1.:-1.;n=(z9==0||z9==3)?.0:1.;x9=int(nb);}else{int M7=Pd(r9);Cb=w9>>M7;int N7=w9&((1<<M7)-1);x9=r9?int(bh):int(nb);y9=!r9&&N7==hh;J5=y9?0:N7;}int K5=min(J5,x9-1);int Y3=Cb*x9+K5;O a2=q1(UB,y4(Y3));uint a0=a2.w;uint D6=max(a0&ub,1u);O L5=p0(BD,D6-1u);c O7=uintBitsToFloat(L5.xy);uint c0=L5.z&0xffffu;uint A9=L5.w;X N0=o1(uintBitsToFloat(p0(KB,c0*4u)));O L3=p0(KB,c0*4u+1u);c x2=uintBitsToFloat(L3.xy);float B2=uintBitsToFloat(L3.z);O Z3=p0(KB,c0*4u+2u);P E6=S1(Z3.x);uint F6=a0&X2;bool P7;if(L7){P7=false;}else if(fe){P7=F6!=0u;if(P7) I0=-I0;}else{P7=F6!=0u&&!r9&&!y9;}if(P7) J5=J5-1;if(J5!=K5){int B9=Y3+J5-K5;O Q7=q1(UB,y4(B9));if((Q7.w&(X2|0xffffu))!=(a0&(X2|0xffffu))){bool R7;if(L7) R7=O7.x!=.0;else R7=true;if(R7){a2=q1(UB,y4(int(A9)));}}else{a2=Q7;}a0=(a2.w&~X2)|F6;}c i0;if(q9()){I0*=sign(determinant(N0));float f1=C9(a2.z);c N1=c(sin(f1),-cos(f1));c G6=uintBitsToFloat(a2.xy);c M3=N1;c he=N1;float Db=(n==.0)?I0:.0;if(L7){if((a0&y6)!=0u) I0=min(I0,.0);if((a0&tb)!=0u) I0=max(I0,.0);uint f5=a0&J3;if(f5>I7){bool H6=(a0&rb)!=0u;bool Eb=(a0&y6)!=0u;float c2=ie(a2.z);float z4=sqrt(max(1.-c2*c2,.0));if(H6==Eb) z4=-z4;X Fb=X(c2,z4,-z4,c2);c g5=B0(Fb,N1);bool D9=f5==ph||(f5!=qb&&c2<.25);bool je=(a0&sb)!=0u;if(f5==qb){M3=N1+g5;}else if(je||!D9){float t=D9?c2:1./c2;M3=g5*t;}if(D9||je) he=g5;if(!v9&&D9) Db=.5*I0;}}i0=B0(N0,G6+M3*(I0*B2))+x2;if(Db!=.0){i0+=sign(B0(he,inverse(N0)))*Db;}}else{c G6=y9?O7:uintBitsToFloat(a2.xy);i0=B0(N0,G6)+x2;}c l0=i0;
#ifdef SD
if(j.Ba!=0u){l0.y=float(j.Ca)-l0.y;}
#endif
#ifdef AB
if(AB){X I3=o1(p0(JB,c0*m2+2u));f X3=p0(JB,c0*m2+3u);lb(I3,X3.xy,l0 h5);}
#endif
if(ai){O0=f(.0,.0,.0,.0);
#ifdef GB
U0=M(0.0,0.0,0.0);
#endif
}else{R0 S0=x5(VC,c0);uint w2=S0.x&0xfu;bool F2=false;
#ifdef H
if(H){uint W1=(S0.x>>4)&0xfu;P0=float(W1);F2=W1!=U3;}
#endif
if(w2==Ka){O0=unpackUnorm4x8(S0.y);if(F2){O0.w*=n;}else{O0*=n;}}else{X Gb=o1(p0(JB,c0*m2));f S7=p0(JB,c0*m2+1u);float o4=uintBitsToFloat(S0.y);O0=Da(l0,Gb,S7.xy,S7.zw,w2,o4,n);}
#ifdef GB
if(GB&&(S0.x&Yd)!=0u){X Hb=o1(p0(JB,c0*m2+4u));f T7=p0(JB,c0*m2+5u);c v3=B0(Hb,l0)+T7.xy;U0=M(v3.x,v3.y,1.+T7.z);}else{U0=M(0.0,0.0,0.0);}
#endif
}f I=R3(i0);
#ifdef MC
I.y=-I.y;
#endif
uint E9;if(q9()){E9=uint(n*254.);if(!bi)++E9;}else{E9=0xffu;}I.z=i9(E6,E9);Z(O0);
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
i l;if(O0.w>=.0){l=V4(O0);}else{c ci=Sa(O0,j.M8,j.N8);l=n2(XC,O8,ci,.0);if(q9()){l.w*=di(O0);}if(!F2){l.xyz*=l.w;l.w*=ke(O0);}}
#ifdef GB
if(GB&&U0.z>0.0){d Ib=U0.z-1.;i M1=E5(TB,U4,U0.xy,Ib);if(F2) M1=H0(i6(M1),M1.w);l*=M1;}
#endif
#if defined(H)&&!defined(U)
i z1=M5(KD);l.xyz=N4(l.xyz,z1,W1);l.xyz*=l.w;
#endif
l.xyz=I2(l.xyz,l.w,d0.xy,j.F3,j.G3);K2(l);}
#endif
