#ifdef BB
f1(f0) g1
#endif
w2 F0 X(0,e,O0);
#ifdef H
MB X(1,d,P0);
#endif
#ifdef GB
F0 X(2,M,V0);
#endif
l2
#ifdef BB
f e Th(c l0,W y6,c L1,uint l9,c H7,float ce,float l){e K0;K0.xy=y0(y6,l0)+L1;K0.z=l;if(l9!=Ea){K0.z+=2.0;}if(H7.x>0.9){K0.z=-K0.z;}uint ub=uint(ce)-1u;uint Uh=uint(fract(ce)*256.0);uint Vh=uint(H7.y*h9);uint X3=(l9<<28)|(ub<<17)|(Vh<<8)|Uh;K0.w=-uintBitsToFloat(X3);return K0;}x1(RB,f0,B,L2,z6){V(O0,e);
#ifdef H
V(P0,d);
#endif
#ifdef GB
V(V0,M);
#endif
#ifdef AD
const bool A6=AD;
#else
const bool A6=false;
#endif
bool Wh=(L2&eh)!=0;bool Xh=(L2&gh)!=0;bool m9=(L2&Od)!=0;bool n9=(L2&fh)!=0;int o9=L2&((1<<g9)-1);int vb,G5,p9;float Q0=.0;float l=1.;bool q9=false;if(A6){int I7=Nd(n9);vb=o9>>I7;int J7=o9&((1<<I7)-1);int de=ch(n9);G5=J7>>de;int r9=J7&((1<<de)-1);if(!n9){++r9;}Q0=r9<2?-1.:1.;l=(r9==0||r9==3)?.0:1.;p9=int(fb);}else{int I7=Md(m9);vb=o9>>I7;int J7=o9&((1<<I7)-1);p9=m9?int(Xg):int(fb);q9=!m9&&J7==dh;G5=q9?0:J7;}int H5=min(G5,p9-1);int Y3=vb*p9+H5;O c2=r1(UB,w4(Y3));uint a0=c2.w;uint B6=max(a0&mb,1u);O I5=p0(BD,B6-1u);c K7=uintBitsToFloat(I5.xy);uint c0=I5.z&0xffffu;uint v9=I5.w;W N0=p1(uintBitsToFloat(p0(KB,c0*4u)));O K3=p0(KB,c0*4u+1u);c L1=uintBitsToFloat(K3.xy);float B2=uintBitsToFloat(K3.z);O Z3=p0(KB,c0*4u+2u);P C6=T1(Z3.x);uint L7=a0&Y2;if(A6){}else{if(L7!=0u&&!m9&&!q9){G5=G5-1;}}if(G5!=H5){int w9=Y3+G5-H5;O M7=r1(UB,w4(w9));if((M7.w&(Y2|0xffffu))!=(a0&(Y2|0xffffu))){bool N7;if(A6) N7=K7.x!=.0;else N7=true;if(N7){c2=r1(UB,w4(int(v9)));}}else{c2=M7;}a0=(c2.w&~Y2)|L7;}c i0;if(A6){float h1=x9(c2.z);c P1=c(sin(h1),-cos(h1));c D6=uintBitsToFloat(c2.xy);Q0*=sign(determinant(N0));if((a0&r6)!=0u) Q0=min(Q0,.0);if((a0&lb)!=0u) Q0=max(Q0,.0);c L3=P1;c ee=P1;float wb=(l==.0)?Q0:.0;uint c5=a0&I3;if(c5>E7){bool E6=(a0&jb)!=0u;bool xb=(a0&r6)!=0u;float d2=fe(c2.z);float x4=sqrt(max(1.-d2*d2,.0));if(E6==xb) x4=-x4;W yb=W(d2,x4,-x4,d2);c d5=y0(yb,P1);bool y9=c5==lh||(c5!=ib&&d2<.25);bool ge=(a0&kb)!=0u;if(c5==ib){L3=P1+d5;}else if(ge||!y9){float t=y9?d2:1./d2;L3=d5*t;}if(y9||ge) ee=d5;if(!n9&&y9) wb=.5*Q0;}i0=y0(N0,D6+L3*(Q0*B2))+L1;if(wb!=.0){i0+=sign(y0(ee,inverse(N0)))*wb;}}else{c D6=q9?K7:uintBitsToFloat(c2.xy);i0=y0(N0,D6)+L1;}c l0=i0;
#ifdef RD
if(j.ua!=0u){l0.y=float(j.va)-l0.y;}
#endif
#ifdef AB
if(AB){W H3=p1(p0(JB,c0*n2+2u));e W3=p0(JB,c0*n2+3u);db(H3,W3.xy,l0 e5);}
#endif
if(Wh){O0=e(.0,.0,.0,.0);
#ifdef GB
V0=M(0.0,0.0,0.0);
#endif
}else{S0 T0=q5(WC,c0);uint j3=T0.x&0xfu;bool F2=false;
#ifdef H
if(H){uint X1=(T0.x>>4)&0xfu;P0=float(X1);F2=X1!=T3;}
#endif
if(j3==Ca){O0=unpackUnorm4x8(T0.y);if(F2){O0.w*=l;}else{O0*=l;}}else{W zb=p1(p0(JB,c0*n2));e O7=p0(JB,c0*n2+1u);O0=Th(l0,zb,O7.xy,j3,O7.zw,uintBitsToFloat(T0.y),l);}
#ifdef GB
if(GB&&(T0.x&Vd)!=0u){W Ab=p1(p0(JB,c0*n2+4u));e P7=p0(JB,c0*n2+5u);c r3=y0(Ab,l0)+P7.xy;V0=M(r3.x,r3.y,1.+P7.z);}else{V0=M(0.0,0.0,0.0);}
#endif
}e I=Q3(i0);
#ifdef MC
I.y=-I.y;
#endif
uint z9;if(A6){z9=uint(l*254.);if(!Xh)++z9;}else{z9=0xffu;}I.z=c9(C6,z9);Z(O0);
#ifdef H
Z(P0);
#endif
#ifdef GB
Z(V0);
#endif
y1(I);}
#endif
#ifdef EB
f c Yh(e K0,float Bb,float g7,c1(d) l,c1(uint) X3){const float Cb=W8;const float he=0.5*W8;X3=floatBitsToUint(K0.w);c x2;x2.y=float(X3&(0x7ffu<<17))*Bb-g7;float t;l=abs(K0.z);if(l<1.5){t=K0.x;}else{t=length(K0.xy);l-=2.0;}t=clamp(t,0.0,1.0);if(K0.z<0.0){x2.x=t*(1.0-Cb)+he;}else{float Zh=float(X3&0x1ff00u)*(Cb/256.0)+he;x2.x=t*Cb+Zh;}return x2;}f d ai(uint X3){return D5(X3&0xffu)*(1.0/255.0);}W2(i,IB){q(O0,e);
#ifdef GB
q(V0,M);
#endif
#ifdef H
q(P0,d);
#endif
#ifdef H
P X1=X2(P0);bool F2=H&&X1!=T3;
#else
const bool F2=false;
#endif
i n;if(O0.w>=.0){n=T4(O0);}else{d l;uint X3;c bi=Yh(O0,j.Bb,j.g7,l,X3);n=o2(YC,H8,bi,.0);
#ifdef AD
if(AD) n.w*=l;
#endif
if(!F2){n.xyz*=n.w;n.w*=ai(X3);}}
#ifdef GB
if(GB&&V0.z>0.0){d Db=V0.z-1.;i O1=A5(TB,S4,V0.xy,Db);if(F2) O1=H0(f6(O1),O1.w);n*=O1;}
#endif
#if defined(H)&&!defined(U)
i A1=J5(JD);n.xyz=L4(n.xyz,A1,X1);n.xyz*=n.w;
#endif
n.xyz=I2(n.xyz,n.w,d0.xy,j.E3,j.F3);K2(n);}
#endif
