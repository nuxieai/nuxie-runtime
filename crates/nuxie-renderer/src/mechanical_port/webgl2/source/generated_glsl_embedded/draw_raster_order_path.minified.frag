#ifdef GB
J1 x0(S2,j0);j1(T2,h0);x0(h6,l4);j1(K6,H7);K1 M1(JB){r(f1,g);
#ifdef KB
r(A2,R);
#endif
#ifdef EB
r(i1,c);
#else
r(O,z2);
#endif
r(B0,c);
#ifdef I
r(V1,E);
#endif
#ifdef BB
r(M0,g);
#endif
#ifdef AB
r(f2,c);
#endif
#if!defined(EB)
x2;
#endif
E R4=unpackHalf2x16(Y0(H7));c i9=R4.y;c p0=i9==B0?R4.x:G0(.0);
#ifdef EB
p0+=i1;e2(H7);
#else
p0=Xh(p0,O d1);c1(H7,packHalf2x16(B2(p0,B0)));
#endif
c n;
#ifdef EE
if(EE){n=da(p0,G0(.0),G0(1.));}else
#endif
{n=abs(p0);
#ifdef XC
if(XC&&B0<.0){n=1.-G0(abs(fract(n*.5)*2.+-1.));}
#endif
n=min(n,G0(1.));}
#ifdef I
if(I&&V1.x<.0){c k1=-V1.x;
#ifdef ZC
if(ZC){c J5=V1.y;if(J5!=.0){E N0=unpackHalf2x16(Y0(h0));c F6=N0.y;c p4;if(F6!=k1){p4=F6==J5?N0.x:.0;
#ifndef EB
y0(l4,C0(p4,.0,.0,.0));
#endif
}else{p4=I0(l4).x;
#ifndef EB
w2(l4);
#endif
}n=min(n,p4);}}
#endif
c1(h0,packHalf2x16(B2(n,k1)));w2(j0);}else
#endif
{
#ifdef I
if(I){c k1=V1.x;if(k1!=.0){E N0=unpackHalf2x16(Y0(h0));c F6=N0.y;n=(F6==k1)?min(N0.x,n):G0(.0);}}
#endif
#ifdef BB
if(BB){c Y4=h3(c5(M0));n=clamp(Y4,G0(.0),n);}
#endif
i j=M7(f1,
#ifdef KB
A2,
#endif
n U2);i L1;if(i9!=B0){L1=I0(j0);
#ifndef EB
y0(l4,L1);
#endif
}else{L1=I0(l4);
#ifndef EB
w2(l4);
#endif
}
#ifdef AB
if(AB){if(f2!=a6(R5)){j.xyz=U4(j.xyz,L1,c6(f2));}j.xyz*=j.w;}
#endif
#ifdef CC
if(CC){j=m3(j);}
#endif
c v2=j.w;j+=L1*(1.-v2);j.xyz=F2(j.xyz,v2,a0.xy,m.B3,m.C3);y0(j0,j);e2(h0);}
#if!defined(EB)
y2;
#endif
Z1;}
#endif
