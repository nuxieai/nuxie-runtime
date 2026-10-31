#ifdef EB
H1 y0(B2,p0);w1(p3,n0);y0(D6,S4);w1(l7,x8);I1 D1(HB){q(N0,e);
#ifdef GB
q(O0,M);
#endif
#ifdef DB
q(r1,d);
#else
q(S,T2);
#endif
q(J0,d);
#ifdef N
q(l2,F);
#endif
#ifdef AB
q(Y0,e);
#endif
#ifdef H
q(S0,d);
#endif
#if!defined(DB)
R2;
#endif
F A5=unpackHalf2x16(m1(x8));d oa=A5.y;d w0=oa==J0?A5.x:A0(.0);
#ifdef DB
w0+=r1;k2(x8);
#else
w0=ik(w0,S p1);o1(x8,packHalf2x16(U2(w0,J0)));
#endif
d n;
#ifdef ME
if(ME){n=nb(w0,A0(.0),A0(1.));}else
#endif
{n=abs(w0);
#ifdef XC
if(XC&&J0<.0){n=1.-A0(abs(fract(n*.5)*2.+-1.));}
#endif
n=min(n,A0(1.));}
#ifdef N
if(N&&l2.x<.0){d C1=-l2.x;
#ifdef DD
if(DD){d i6=l2.y;if(i6!=.0){F Z0=unpackHalf2x16(m1(n0));d h7=Z0.y;d W4;if(h7!=C1){W4=h7==i6?Z0.x:.0;
#ifndef DB
x0(S4,z0(W4,.0,.0,.0));
#endif
}else{W4=L0(S4).x;
#ifndef DB
Q2(S4);
#endif
}n=min(n,W4);}}
#endif
o1(n0,packHalf2x16(U2(n,C1)));Q2(p0);}else
#endif
{
#ifdef N
if(N){d C1=l2.x;if(C1!=.0){F Z0=unpackHalf2x16(m1(n0));d h7=Z0.y;n=(h7==C1)?min(Z0.x,n):A0(.0);}}
#endif
#ifdef AB
if(AB){d F5=F3(y4(Y0));n=clamp(F5,A0(.0),n);}
#endif
i k=C8(
#ifdef GB
O0,
#endif
#ifdef H
Z2(S0),
#endif
N0 r3);i X0;if(oa!=J0){X0=L0(p0);
#ifndef DB
x0(S4,X0);
#endif
}else{X0=L0(S4);
#ifndef DB
Q2(S4);
#endif
}bool ag=false;
#ifdef GB
ag=GB&&O0.z<.0;
#endif
if(ag){
#ifdef GB
uint Sb=uint(-O0.z-1.);d Tb=Ae(k,Sb);k=X0*mix(A0(1.),Tb,n);x0(p0,k);k2(n0);
#endif
}else{
#ifdef H
if(H&&S0!=x6(X3)){k.xyz=X4(k.xyz,X0,Z2(S0))*k.w;}
#endif
k*=n;d B3=k.w;k+=X0*(1.-B3);k.xyz=M2(k.xyz,B3,Y.xy,j.K3,j.L3);x0(p0,k);k2(n0);}}
#if!defined(DB)
S2;
#endif
K1;}
#endif
