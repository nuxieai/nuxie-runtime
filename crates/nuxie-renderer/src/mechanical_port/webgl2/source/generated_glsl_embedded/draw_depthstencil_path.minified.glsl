f bool m9(){
#ifdef AD
if(AD) return true;
#endif
#ifdef BD
if(BD) return true;
#endif
return false;}
#ifdef BB
f1(f0) g1
#endif
w2 F0 X(0,e,P0);
#ifdef H
MB X(1,d,Q0);
#endif
#ifdef GB
F0 X(2,M,V0);
#endif
l2
#ifdef BB
f e Xh(c l0,W y6,c L1,uint n9,c H7,float ee,float l){e L0;L0.xy=y0(y6,l0)+L1;L0.z=l;if(n9!=Ga){L0.z+=2.0;}if(H7.x>0.9){L0.z=-L0.z;}uint wb=uint(ee)-1u;uint Yh=uint(fract(ee)*256.0);uint Zh=uint(H7.y*i9);uint X3=(n9<<28)|(wb<<17)|(Zh<<8)|Yh;L0.w=-uintBitsToFloat(X3);return L0;}x1(RB,f0,B,L2,z6){V(P0,e);
#ifdef H
V(Q0,d);
#endif
#ifdef GB
V(V0,M);
#endif
#ifdef AD
const bool I7=AD;
#else
const bool I7=false;
#endif
#ifdef BD
const bool fe=BD;
#else
const bool fe=false;
#endif
bool ai=(L2&hh)!=0;bool bi=(L2&jh)!=0;bool o9=(L2&Qd)!=0;bool p9=(L2&ih)!=0;int q9=L2&((1<<h9)-1);int xb,G5,r9;float I0=.0;float l=1.;bool v9=false;if(m9()){int J7=Pd(p9);xb=q9>>J7;int K7=q9&((1<<J7)-1);int ge=fh(p9);G5=K7>>ge;int w9=K7&((1<<ge)-1);if(I7&&!p9){++w9;}I0=w9<2?1.:-1.;l=(w9==0||w9==3)?.0:1.;r9=int(hb);}else{int J7=Od(o9);xb=q9>>J7;int K7=q9&((1<<J7)-1);r9=o9?int(ah):int(hb);v9=!o9&&K7==gh;G5=v9?0:K7;}int H5=min(G5,r9-1);int Y3=xb*r9+H5;O c2=r1(UB,w4(Y3));uint a0=c2.w;uint A6=max(a0&ob,1u);O I5=p0(CD,A6-1u);c L7=uintBitsToFloat(I5.xy);uint c0=I5.z&0xffffu;uint x9=I5.w;W O0=p1(uintBitsToFloat(p0(KB,c0*4u)));O K3=p0(KB,c0*4u+1u);c L1=uintBitsToFloat(K3.xy);float B2=uintBitsToFloat(K3.z);O Z3=p0(KB,c0*4u+2u);P B6=T1(Z3.x);uint C6=a0&Y2;bool M7;if(I7){M7=false;}else if(fe){M7=C6!=0u;if(M7) I0=-I0;}else{M7=C6!=0u&&!o9&&!v9;}if(M7) G5=G5-1;if(G5!=H5){int y9=Y3+G5-H5;O N7=r1(UB,w4(y9));if((N7.w&(Y2|0xffffu))!=(a0&(Y2|0xffffu))){bool O7;if(I7) O7=L7.x!=.0;else O7=true;if(O7){c2=r1(UB,w4(int(x9)));}}else{c2=N7;}a0=(c2.w&~Y2)|C6;}c i0;if(m9()){I0*=sign(determinant(O0));float h1=z9(c2.z);c P1=c(sin(h1),-cos(h1));c D6=uintBitsToFloat(c2.xy);c L3=P1;c he=P1;float yb=(l==.0)?I0:.0;if(I7){if((a0&r6)!=0u) I0=min(I0,.0);if((a0&nb)!=0u) I0=max(I0,.0);uint c5=a0&I3;if(c5>E7){bool E6=(a0&lb)!=0u;bool zb=(a0&r6)!=0u;float d2=ie(c2.z);float x4=sqrt(max(1.-d2*d2,.0));if(E6==zb) x4=-x4;W Ab=W(d2,x4,-x4,d2);c d5=y0(Ab,P1);bool A9=c5==oh||(c5!=kb&&d2<.25);bool je=(a0&mb)!=0u;if(c5==kb){L3=P1+d5;}else if(je||!A9){float t=A9?d2:1./d2;L3=d5*t;}if(A9||je) he=d5;if(!p9&&A9) yb=.5*I0;}}i0=y0(O0,D6+L3*(I0*B2))+L1;if(yb!=.0){i0+=sign(y0(he,inverse(O0)))*yb;}}else{c D6=v9?L7:uintBitsToFloat(c2.xy);i0=y0(O0,D6)+L1;}c l0=i0;
#ifdef SD
if(j.wa!=0u){l0.y=float(j.xa)-l0.y;}
#endif
#ifdef AB
if(AB){W H3=p1(p0(JB,c0*n2+2u));e W3=p0(JB,c0*n2+3u);fb(H3,W3.xy,l0 e5);}
#endif
if(ai){P0=e(.0,.0,.0,.0);
#ifdef GB
V0=M(0.0,0.0,0.0);
#endif
}else{S0 T0=q5(WC,c0);uint j3=T0.x&0xfu;bool F2=false;
#ifdef H
if(H){uint X1=(T0.x>>4)&0xfu;Q0=float(X1);F2=X1!=T3;}
#endif
if(j3==Ea){P0=unpackUnorm4x8(T0.y);if(F2){P0.w*=l;}else{P0*=l;}}else{W Bb=p1(p0(JB,c0*n2));e P7=p0(JB,c0*n2+1u);P0=Xh(l0,Bb,P7.xy,j3,P7.zw,uintBitsToFloat(T0.y),l);}
#ifdef GB
if(GB&&(T0.x&Xd)!=0u){W Cb=p1(p0(JB,c0*n2+4u));e Q7=p0(JB,c0*n2+5u);c r3=y0(Cb,l0)+Q7.xy;V0=M(r3.x,r3.y,1.+Q7.z);}else{V0=M(0.0,0.0,0.0);}
#endif
}e I=Q3(i0);
#ifdef MC
I.y=-I.y;
#endif
uint B9;if(m9()){B9=uint(l*254.);if(!bi)++B9;}else{B9=0xffu;}I.z=d9(B6,B9);Z(P0);
#ifdef H
Z(Q0);
#endif
#ifdef GB
Z(V0);
#endif
y1(I);}
#endif
#ifdef EB
f c ci(e L0,float Db,float g7,c1(d) l,c1(uint) X3){const float Eb=X8;const float ke=0.5*X8;X3=floatBitsToUint(L0.w);c x2;x2.y=float(X3&(0x7ffu<<17))*Db-g7;float t;l=abs(L0.z);if(l<1.5){t=L0.x;}else{t=length(L0.xy);l-=2.0;}t=clamp(t,0.0,1.0);if(L0.z<0.0){x2.x=t*(1.0-Eb)+ke;}else{float di=float(X3&0x1ff00u)*(Eb/256.0)+ke;x2.x=t*Eb+di;}return x2;}f d ei(uint X3){return D5(X3&0xffu)*(1.0/255.0);}W2(i,IB){q(P0,e);
#ifdef GB
q(V0,M);
#endif
#ifdef H
q(Q0,d);
#endif
#ifdef H
P X1=X2(Q0);bool F2=H&&X1!=T3;
#else
const bool F2=false;
#endif
i n;if(P0.w>=.0){n=T4(P0);}else{d l;uint X3;c fi=ci(P0,j.Db,j.g7,l,X3);n=o2(YC,I8,fi,.0);if(m9()) n.w*=l;if(!F2){n.xyz*=n.w;n.w*=ei(X3);}}
#ifdef GB
if(GB&&V0.z>0.0){d Fb=V0.z-1.;i O1=A5(TB,S4,V0.xy,Fb);if(F2) O1=H0(f6(O1),O1.w);n*=O1;}
#endif
#if defined(H)&&!defined(U)
i A1=J5(KD);n.xyz=L4(n.xyz,A1,X1);n.xyz*=n.w;
#endif
n.xyz=I2(n.xyz,n.w,d0.xy,j.E3,j.F3);K2(n);}
#endif
