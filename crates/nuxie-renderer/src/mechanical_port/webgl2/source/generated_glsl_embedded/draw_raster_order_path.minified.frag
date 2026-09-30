#ifdef EB
Q1 A0(L2,o0);o1(d3,m0);A0(n6,B4);o1(T6,R7);R1 T1(IB){q(a1,e);
#ifdef GB
q(F1,P);
#endif
#ifdef DB
q(m1,d);
#else
q(S,H2);
#endif
q(F0,d);
#ifdef A
q(l1,C);
#endif
#ifdef AB
q(R0,e);
#endif
#ifdef O
q(Q0,d);
#endif
#if!defined(DB)
F2;
#endif
C d5=unpackHalf2x16(h1(R7));d C9=d5.y;d w0=C9==F0?d5.x:M0(.0);
#ifdef DB
w0+=m1;k2(R7);
#else
w0=Ni(w0,S k1);j1(R7,packHalf2x16(I2(w0,F0)));
#endif
d o;
#ifdef HE
if(HE){o=Aa(w0,M0(.0),M0(1.));}else
#endif
{o=abs(w0);
#ifdef YC
if(YC&&F0<.0){o=1.-M0(abs(fract(o*.5)*2.+-1.));}
#endif
o=min(o,M0(1.));}
#ifdef A
if(A&&l1.x<.0){d X0=-l1.x;
#ifdef BD
if(BD){d E4=l1.y;if(E4!=.0){C T0=unpackHalf2x16(h1(m0));d O6=T0.y;d G4;if(O6!=X0){G4=O6==E4?T0.x:.0;
#ifndef DB
B0(B4,G0(G4,.0,.0,.0));
#endif
}else{G4=N0(B4).x;
#ifndef DB
E2(B4);
#endif
}o=min(o,G4);}}
#endif
j1(m0,packHalf2x16(I2(o,X0)));E2(o0);}else
#endif
{
#ifdef A
if(A){d X0=l1.x;if(X0!=.0){C T0=unpackHalf2x16(h1(m0));d O6=T0.y;o=(O6==X0)?min(T0.x,o):M0(.0);}}
#endif
#ifdef AB
if(AB){d l5=v3(q5(R0));o=clamp(l5,M0(.0),o);}
#endif
i l=Y7(
#ifdef GB
F1,
#endif
#ifdef O
k3(Q0),
#endif
a1 e3);i S1;if(C9!=F0){S1=N0(o0);
#ifndef DB
B0(B4,S1);
#endif
}else{S1=N0(B4);
#ifndef DB
E2(B4);
#endif
}
#ifdef O
if(O&&Q0!=i6(L4)){l.xyz=h5(l.xyz,S1,k3(Q0))*l.w;}
#endif
l*=o;
#ifdef CC
if(CC){l=z3(l);}
#endif
d n3=l.w;l+=S1*(1.-n3);l.xyz=O2(l.xyz,n3,f0.xy,j.M3,j.N3);B0(o0,l);k2(m0);}
#if!defined(DB)
G2;
#endif
g2;}
#endif
