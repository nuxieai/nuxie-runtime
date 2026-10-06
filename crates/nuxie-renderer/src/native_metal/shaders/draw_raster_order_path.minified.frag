#ifdef FRAGMENT
S1 B0(K2,n0);o1(c3,m0);B0(o6,C4);o1(U6,Q7);T1 U1(IB){q(a1,e);
#ifdef ENABLE_MODULATED_IMAGE
q(v1,O);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
q(m1,d);
#else
q(S,G2);
#endif
q(F0,d);
#ifdef ENABLE_CLIPPING
q(l1,C);
#endif
#ifdef ENABLE_CLIP_RECT
q(R0,e);
#endif
#ifdef ENABLE_ADVANCED_BLEND
q(Q0,d);
#endif
#if!defined(DRAW_INTERIOR_TRIANGLES)
E2;
#endif
C d5=unpackHalf2x16(h1(Q7));d E9=d5.y;d w0=E9==F0?d5.x:H0(.0);
#ifdef DRAW_INTERIOR_TRIANGLES
w0+=m1;a2(Q7);
#else
w0=bj(w0,S k1);j1(Q7,packHalf2x16(H2(w0,F0)));
#endif
d n;
#ifdef CLOCKWISE_FILL
if(CLOCKWISE_FILL){n=Ba(w0,H0(.0),H0(1.));}else
#endif
{n=abs(w0);
#ifdef ENABLE_EVEN_ODD
if(ENABLE_EVEN_ODD&&F0<.0){n=1.-H0(abs(fract(n*.5)*2.+-1.));}
#endif
n=min(n,H0(1.));}
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&l1.x<.0){d X0=-l1.x;
#ifdef ENABLE_NESTED_CLIPPING
if(ENABLE_NESTED_CLIPPING){d F4=l1.y;if(F4!=.0){C T0=unpackHalf2x16(h1(m0));d P6=T0.y;d H4;if(P6!=X0){H4=P6==F4?T0.x:.0;
#ifndef DRAW_INTERIOR_TRIANGLES
y0(C4,I0(H4,.0,.0,.0));
#endif
}else{H4=N0(C4).x;
#ifndef DRAW_INTERIOR_TRIANGLES
D2(C4);
#endif
}n=min(n,H4);}}
#endif
j1(m0,packHalf2x16(H2(n,X0)));D2(n0);}else
#endif
{
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){d X0=l1.x;if(X0!=.0){C T0=unpackHalf2x16(h1(m0));d P6=T0.y;n=(P6==X0)?min(T0.x,n):H0(.0);}}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d m5=w3(v5(R0));n=clamp(m5,H0(.0),n);}
#endif
i p=X7(
#ifdef ENABLE_MODULATED_IMAGE
v1,
#endif
#ifdef ENABLE_ADVANCED_BLEND
k3(Q0),
#endif
a1 e3);i J1;if(E9!=F0){J1=N0(n0);
#ifndef DRAW_INTERIOR_TRIANGLES
y0(C4,J1);
#endif
}else{J1=N0(C4);
#ifndef DRAW_INTERIOR_TRIANGLES
D2(C4);
#endif
}bool ef=false;
#ifdef ENABLE_MODULATED_IMAGE
ef=ENABLE_MODULATED_IMAGE&&v1.z<.0;
#endif
if(ef){
#ifdef ENABLE_MODULATED_IMAGE
uint hj=uint(-v1.z-1.);d ij=Zi(p,hj);p=J1*mix(H0(1.),ij,n);y0(n0,p);a2(m0);
#endif
}else{
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND&&Q0!=j6(M4)){p.xyz=h5(p.xyz,J1,k3(Q0))*p.w;}
#endif
p*=n;d n3=p.w;p+=J1*(1.-n3);p.xyz=M2(p.xyz,n3,f0.xy,j.M3,j.N3);y0(n0,p);a2(m0);}}
#if!defined(DRAW_INTERIOR_TRIANGLES)
F2;
#endif
h2;}
#endif
