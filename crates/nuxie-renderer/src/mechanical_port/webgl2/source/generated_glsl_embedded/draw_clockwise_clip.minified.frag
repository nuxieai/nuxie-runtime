#ifdef FB
J1
#ifndef O
y0(U2,k0);
#endif
i1(V2,h0);
#ifndef O
Za(h6,m4);
#endif
i1(K6,Q0);K1 M1(IB){r(W1,E);d j1=-W1.x;
#ifdef EB
r(h1,d);d w0=h1;
#else
r(M,B2);d w0=M.x;
#endif
z2;E O0;d K5,v3;
#if defined(EB)&&defined(DC)
if(DC){v3=w0;}else
#endif
{O0=unpackHalf2x16(Y0(h0));K5=O0.y;d S4=K5==j1?O0.x:I0(.0);v3=S4+w0;}
#ifdef ZC
d I5=W1.y;if(ZC&&I5!=.0){d q4=.0;
#if defined(EB)&&defined(DC)
if(DC){O0=unpackHalf2x16(Y0(h0));K5=O0.y;}
#endif
if(K5!=j1){q4=K5==I5?O0.x:.0;c1(Q0,packHalf2x16(D2(q4,bg)));}else{q4=unpackHalf2x16(Y0(Q0)).x;f2(Q0);}v3=min(v3,q4);}else
#endif
{f2(Q0);}c1(h0,packHalf2x16(D2(v3,j1)));
#ifndef O
y2(k0);
#endif
A2;a2;}
#endif
