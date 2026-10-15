#ifdef EB
E1 z0(x2,n0);q1(k3,m0);z0(A6,I4);q1(l7,p8);F1 X1(IB){q(P0,f);
#ifdef GB
q(U0,M);
#endif
#ifdef DB
q(o1,d);
#else
q(S,Q2);
#endif
q(G0,d);
#ifdef N
q(i2,D);
#endif
#ifdef AB
q(V0,f);
#endif
#ifdef H
q(Q0,d);
#endif
#if!defined(DB)
O2;
#endif
D o5=unpackHalf2x16(k1(p8));d ka=o5.y;d w0=ka==G0?o5.x:J0(.0);
#ifdef DB
w0+=o1;g2(p8);
#else
w0=Nj(w0,S n1);m1(p8,packHalf2x16(R2(w0,G0)));
#endif
d n;
#ifdef KE
if(KE){n=ib(w0,J0(.0),J0(1.));}else
#endif
{n=abs(w0);
#ifdef YC
if(YC&&G0<.0){n=1.-J0(abs(fract(n*.5)*2.+-1.));}
#endif
n=min(n,J0(1.));}
#ifdef N
if(N&&i2.x<.0){d A1=-i2.x;
#ifdef ED
if(ED){d e6=i2.y;if(e6!=.0){D W0=unpackHalf2x16(k1(m0));d h7=W0.y;d M4;if(h7!=A1){M4=h7==e6?W0.x:.0;
#ifndef DB
x0(I4,H0(M4,.0,.0,.0));
#endif
}else{M4=O0(I4).x;
#ifndef DB
N2(I4);
#endif
}n=min(n,M4);}}
#endif
m1(m0,packHalf2x16(R2(n,A1)));N2(n0);}else
#endif
{
#ifdef N
if(N){d A1=i2.x;if(A1!=.0){D W0=unpackHalf2x16(k1(m0));d h7=W0.y;n=(h7==A1)?min(W0.x,n):J0(.0);}}
#endif
#ifdef AB
if(AB){d y5=B3(V4(V0));n=clamp(y5,J0(.0),n);}
#endif
i k=x8(
#ifdef GB
U0,
#endif
#ifdef H
W2(Q0),
#endif
P0 m3);i a1;if(ka!=G0){a1=O0(n0);
#ifndef DB
x0(I4,a1);
#endif
}else{a1=O0(I4);
#ifndef DB
N2(I4);
#endif
}bool Jf=false;
#ifdef GB
Jf=GB&&U0.z<.0;
#endif
if(Jf){
#ifdef GB
uint Tj=uint(-U0.z-1.);d Uj=Mj(k,Tj);k=a1*mix(J0(1.),Uj,n);x0(n0,k);g2(m0);
#endif
}else{
#ifdef H
if(H&&Q0!=r6(U3)){k.xyz=N4(k.xyz,a1,W2(Q0))*k.w;}
#endif
k*=n;d x3=k.w;k+=a1*(1.-x3);k.xyz=J2(k.xyz,x3,d0.xy,j.F3,j.G3);x0(n0,k);g2(m0);}}
#if!defined(DB)
P2;
#endif
o2;}
#endif
