#ifdef GB
I1 w0(Q2,j0);j1(R2,h0);w0(d6,i4);j1(G6,E7);J1 L1(JB){A(f1,g);
#ifdef EB
A(i1,c);
#else
A(O,z2);
#endif
A(B0,c);
#ifdef I
A(U1,E);
#endif
#ifdef BB
A(L0,g);
#endif
#ifdef AB
A(e2,c);
#endif
#if!defined(EB)
w2;
#endif
E N4=unpackHalf2x16(Y0(E7));c i9=N4.y;c p0=i9==B0?N4.x:G0(.0);
#ifdef EB
p0+=i1;d2(E7);
#else
p0=Hh(p0,O d1);c1(E7,packHalf2x16(A2(p0,B0)));
#endif
c o;
#ifdef BE
if(BE){o=da(p0,G0(.0),G0(1.));}else
#endif
{o=abs(p0);
#ifdef WC
if(WC&&B0<.0){o=1.-G0(abs(fract(o*.5)*2.+-1.));}
#endif
o=min(o,G0(1.));}
#ifdef I
if(I&&U1.x<.0){c k1=-U1.x;
#ifdef YC
if(YC){c G5=U1.y;if(G5!=.0){E M0=unpackHalf2x16(Y0(h0));c B6=M0.y;c m4;if(B6!=k1){m4=B6==G5?M0.x:.0;
#ifndef EB
x0(i4,C0(m4,.0,.0,.0));
#endif
}else{m4=H0(i4).x;
#ifndef EB
v2(i4);
#endif
}o=min(o,m4);}}
#endif
c1(h0,packHalf2x16(A2(o,k1)));v2(j0);}else
#endif
{
#ifdef I
if(I){c k1=U1.x;if(k1!=.0){E M0=unpackHalf2x16(Y0(h0));c B6=M0.y;o=(B6==k1)?min(M0.x,o):G0(.0);}}
#endif
#ifdef BB
if(BB){c U4=f3(Y4(L0));o=clamp(U4,G0(.0),o);}
#endif
i j=J7(f1,o S2);i K1;if(i9!=B0){K1=H0(j0);
#ifndef EB
x0(i4,K1);
#endif
}else{K1=H0(i4);
#ifndef EB
v2(i4);
#endif
}
#ifdef AB
if(AB){if(e2!=W5(N5)){j.xyz=Q4(j.xyz,K1,X5(e2));}j.xyz*=j.w;}
#endif
#ifdef BC
if(BC){j=k3(j);}
#endif
c r2=j.w;j+=K1*(1.-r2);j.xyz=E2(j.xyz,r2,Y.xy,n.z3,n.A3);x0(j0,j);d2(h0);}
#if!defined(EB)
x2;
#endif
Y1;}
#endif
