#ifdef DB
F1 y0(x2,n0);r1(k3,m0);y0(E6,M4);r1(l7,r8);G1 I1(HB){q(N0,e);
#ifdef GB
q(O0,M);
#endif
#ifdef EB
q(p1,d);
#else
q(T,R2);
#endif
q(I0,d);
#ifdef N
q(k2,E);
#endif
#ifdef AB
q(W0,e);
#endif
#ifdef H
q(S0,d);
#endif
#if!defined(EB)
P2;
#endif
E r5=unpackHalf2x16(k1(r8));d la=r5.y;d w0=la==I0?r5.x:C0(.0);
#ifdef EB
w0+=p1;i2(r8);
#else
w0=Xj(w0,T n1);m1(r8,packHalf2x16(S2(w0,I0)));
#endif
d n;
#ifdef NE
if(NE){n=jb(w0,C0(.0),C0(1.));}else
#endif
{n=abs(w0);
#ifdef YC
if(YC&&I0<.0){n=1.-C0(abs(fract(n*.5)*2.+-1.));}
#endif
n=min(n,C0(1.));}
#ifdef N
if(N&&k2.x<.0){d B1=-k2.x;
#ifdef ED
if(ED){d h6=k2.y;if(h6!=.0){E X0=unpackHalf2x16(k1(m0));d h7=X0.y;d Q4;if(h7!=B1){Q4=h7==h6?X0.x:.0;
#ifndef EB
x0(M4,E0(Q4,.0,.0,.0));
#endif
}else{Q4=K0(M4).x;
#ifndef EB
O2(M4);
#endif
}n=min(n,Q4);}}
#endif
m1(m0,packHalf2x16(S2(n,B1)));O2(n0);}else
#endif
{
#ifdef N
if(N){d B1=k2.x;if(B1!=.0){E X0=unpackHalf2x16(k1(m0));d h7=X0.y;n=(h7==B1)?min(X0.x,n):C0(.0);}}
#endif
#ifdef AB
if(AB){d A5=C3(Y4(W0));n=clamp(A5,C0(.0),n);}
#endif
i k=z8(
#ifdef GB
O0,
#endif
#ifdef H
W2(S0),
#endif
N0 m3);i a1;if(la!=I0){a1=K0(n0);
#ifndef EB
x0(M4,a1);
#endif
}else{a1=K0(M4);
#ifndef EB
O2(M4);
#endif
}bool Sf=false;
#ifdef GB
Sf=GB&&O0.z<.0;
#endif
if(Sf){
#ifdef GB
uint Ob=uint(-O0.z-1.);d Pb=we(k,Ob);k=a1*mix(C0(1.),Pb,n);x0(n0,k);i2(m0);
#endif
}else{
#ifdef H
if(H&&S0!=x6(X3)){k.xyz=R4(k.xyz,a1,W2(S0))*k.w;}
#endif
k*=n;d y3=k.w;k+=a1*(1.-y3);k.xyz=J2(k.xyz,y3,c0.xy,j.H3,j.I3);x0(n0,k);i2(m0);}}
#if!defined(EB)
Q2;
#endif
O1;}
#endif
