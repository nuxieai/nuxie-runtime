#ifdef EB
R1 B0(L2,n0);o1(d3,m0);B0(p6,C4);o1(V6,S7);S1 T1(IB){q(a1,f);
#ifdef GB
q(r1,P);
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
q(S0,f);
#endif
#ifdef O
q(Q0,d);
#endif
#if!defined(DB)
F2;
#endif
C e5=unpackHalf2x16(h1(S7));d E9=e5.y;d w0=E9==F0?e5.x:I0(.0);
#ifdef DB
w0+=m1;Z1(S7);
#else
w0=Yi(w0,S k1);j1(S7,packHalf2x16(I2(w0,F0)));
#endif
d o;
#ifdef HE
if(HE){o=Ba(w0,I0(.0),I0(1.));}else
#endif
{o=abs(w0);
#ifdef YC
if(YC&&F0<.0){o=1.-I0(abs(fract(o*.5)*2.+-1.));}
#endif
o=min(o,I0(1.));}
#ifdef A
if(A&&l1.x<.0){d X0=-l1.x;
#ifdef BD
if(BD){d F4=l1.y;if(F4!=.0){C U0=unpackHalf2x16(h1(m0));d Q6=U0.y;d H4;if(Q6!=X0){H4=Q6==F4?U0.x:.0;
#ifndef DB
y0(C4,G0(H4,.0,.0,.0));
#endif
}else{H4=N0(C4).x;
#ifndef DB
E2(C4);
#endif
}o=min(o,H4);}}
#endif
j1(m0,packHalf2x16(I2(o,X0)));E2(n0);}else
#endif
{
#ifdef A
if(A){d X0=l1.x;if(X0!=.0){C U0=unpackHalf2x16(h1(m0));d Q6=U0.y;o=(Q6==X0)?min(U0.x,o):I0(.0);}}
#endif
#ifdef AB
if(AB){d n5=v3(w5(S0));o=clamp(n5,I0(.0),o);}
#endif
i l=Z7(
#ifdef GB
r1,
#endif
#ifdef O
k3(Q0),
#endif
a1 e3);i I1;if(E9!=F0){I1=N0(n0);
#ifndef DB
y0(C4,I1);
#endif
}else{I1=N0(C4);
#ifndef DB
E2(C4);
#endif
}bool cf=false;
#ifdef GB
cf=GB&&r1.z<.0;
#endif
if(cf){
#ifdef GB
uint ej=uint(-r1.z-1.);d fj=Wi(l,ej);l=I1*mix(I0(1.),fj,o);y0(n0,l);Z1(m0);
#endif
}else{
#ifdef O
if(O&&Q0!=k6(M4)){l.xyz=i5(l.xyz,I1,k3(Q0))*l.w;}
#endif
l*=o;
#ifdef CC
if(CC){l=z3(l);}
#endif
d n3=l.w;l+=I1*(1.-n3);l.xyz=O2(l.xyz,n3,f0.xy,j.M3,j.N3);y0(n0,l);Z1(m0);}}
#if!defined(DB)
G2;
#endif
h2;}
#endif
