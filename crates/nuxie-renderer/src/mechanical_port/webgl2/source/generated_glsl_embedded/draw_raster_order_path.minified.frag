#ifdef EB
U1 C0(T2,n0);p1(j3,m0);C0(A6,I4);p1(h7,m8);V1 X1(IB){q(O0,f);
#ifdef GB
q(U0,M);
#endif
#ifdef DB
q(n1,d);
#else
q(S,P2);
#endif
q(G0,d);
#ifdef N
q(i2,D);
#endif
#ifdef AB
q(V0,f);
#endif
#ifdef H
q(P0,d);
#endif
#if!defined(DB)
N2;
#endif
D n5=unpackHalf2x16(j1(m8));d ia=n5.y;d w0=ia==G0?n5.x:J0(.0);
#ifdef DB
w0+=n1;g2(m8);
#else
w0=Kj(w0,S m1);l1(m8,packHalf2x16(Q2(w0,G0)));
#endif
d n;
#ifdef IE
if(IE){n=eb(w0,J0(.0),J0(1.));}else
#endif
{n=abs(w0);
#ifdef WC
if(WC&&G0<.0){n=1.-J0(abs(fract(n*.5)*2.+-1.));}
#endif
n=min(n,J0(1.));}
#ifdef N
if(N&&i2.x<.0){d y1=-i2.x;
#ifdef CD
if(CD){d e6=i2.y;if(e6!=.0){D W0=unpackHalf2x16(j1(m0));d d7=W0.y;d M4;if(d7!=y1){M4=d7==e6?W0.x:.0;
#ifndef DB
y0(I4,H0(M4,.0,.0,.0));
#endif
}else{M4=Q0(I4).x;
#ifndef DB
M2(I4);
#endif
}n=min(n,M4);}}
#endif
l1(m0,packHalf2x16(Q2(n,y1)));M2(n0);}else
#endif
{
#ifdef N
if(N){d y1=i2.x;if(y1!=.0){D W0=unpackHalf2x16(j1(m0));d d7=W0.y;n=(d7==y1)?min(W0.x,n):J0(.0);}}
#endif
#ifdef AB
if(AB){d x5=B3(V4(V0));n=clamp(x5,J0(.0),n);}
#endif
i l=r8(
#ifdef GB
U0,
#endif
#ifdef H
W2(P0),
#endif
O0 l3);i z1;if(ia!=G0){z1=Q0(n0);
#ifndef DB
y0(I4,z1);
#endif
}else{z1=Q0(I4);
#ifndef DB
M2(I4);
#endif
}bool Hf=false;
#ifdef GB
Hf=GB&&U0.z<.0;
#endif
if(Hf){
#ifdef GB
uint Qj=uint(-U0.z-1.);d Rj=Jj(l,Qj);l=z1*mix(J0(1.),Rj,n);y0(n0,l);g2(m0);
#endif
}else{
#ifdef H
if(H&&P0!=r6(U3)){l.xyz=N4(l.xyz,z1,W2(P0))*l.w;}
#endif
l*=n;d w3=l.w;l+=z1*(1.-w3);l.xyz=I2(l.xyz,w3,d0.xy,j.F3,j.G3);y0(n0,l);g2(m0);}}
#if!defined(DB)
O2;
#endif
o2;}
#endif
